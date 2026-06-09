use std::{
    iter,
    sync::atomic::{AtomicU64, Ordering},
    thread,
};

use chrono::Weekday::{self, *};
use rand::{Rng, RngExt, SeedableRng, rngs::SysRng};
use serde::{Deserialize, Serialize};
use strum::{EnumCount, IntoEnumIterator};

use crate::{
    defs::{AppRng, ProblemInput},
    fitness::ScheduleEvaluator,
    solver::{context::Context, types::WeeklyMask},
    tools::controller::{ExecutionController, InterruptRequest},
    types::{DayIdx, PersonIdx, Role, Slot, SlotArray, Solution, WeekIdx, Weights},
};

const TEMPERATURE_THRESHOLD: f32 = 1e-6;

pub struct Refiner<'a> {
    context: Context,
    pub evaluator: ScheduleEvaluator<'a>,
}

impl Refiner<'_> {
    pub fn new<'a>(input: &ProblemInput, weights: &'a Weights) -> Refiner<'a> {
        Refiner {
            context: Context::new(input),
            evaluator: ScheduleEvaluator::new(input, weights),
        }
    }

    pub fn execute(
        &self,
        solution: &Solution,
        parameters: &RefinementParameters,
        threads: Option<u16>,
        controller: &ExecutionController,
        progress: &RefinerProgress,
    ) -> Result<Option<(f32, Solution)>, InterruptRequest> {
        let threads = threads.unwrap_or(num_cpus::get() as u16);
        let extra_threads = threads.saturating_sub(1) as usize;

        let counter = AtomicU64::new(parameters.searches);

        let worker = RefinerWorker {
            context: &self.context,
            evaluator: &self.evaluator,
            parameters,
            controller,
            progress,
            counter: &counter,
            base_solution: solution,
        };

        let best_solution = thread::scope(
            |scope| -> Result<Option<(f32, Solution)>, InterruptRequest> {
                let extra_handles: Vec<_> = (0..extra_threads)
                    .map(|_| scope.spawn(|| worker.spin()))
                    .collect();

                let local = worker.spin();

                let mut all = iter::once(local).chain(
                    extra_handles
                        .into_iter()
                        .map(|h| h.join().expect("refiner thread panicked")),
                );

                all.try_fold(None, |acc, result| match (acc, result?) {
                    (None, solution) => Ok(solution),
                    (Some((old_fitness, old_solution)), Some((new_fitness, new_solution)))
                        if old_fitness > new_fitness =>
                    {
                        Ok(Some((new_fitness, new_solution)))
                    }
                    (acc, _) => Ok(acc),
                })
            },
        )?;

        Ok(best_solution)
    }

    /// Perform a greedy pass to minimize annual hour discrepancies.
    ///
    /// This method identifies the most overworked and underworked people and attempts
    /// to swap their shifts, provided it reduces the overall fitness score and respects
    /// all hard constraints.
    pub fn polish(&self, solution: &mut Solution) {
        let Context { n_people, .. } = self.context;

        let mut deltas = self.evaluator.calculate_annual_deltas(solution);
        let mut best_total = self.evaluator.evaluate(solution).total();
        let mut improved = true;

        while improved {
            improved = false;

            // Sort people by their current annual drift (most overworked first)
            let sorted_people = &mut PersonIdx::array().map(|p| (p, deltas[p]))[..n_people];

            // Sort people by their current annual drift (most overworked first)
            sorted_people.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

            'outer: for i in 0..n_people {
                let (over_p, over_val) = sorted_people[i];

                if over_val <= 1. {
                    break;
                } // Not significantly overworked

                // Try to find an underworked person to take a shift
                for j in (0..n_people).rev() {
                    let (under_p, under_val) = sorted_people[j];
                    if under_val >= -1. {
                        break;
                    } // Not significantly underworked
                    if i == j {
                        continue;
                    }

                    // Look for a day where over_p works and under_p does not
                    for day in DayIdx::iter() {
                        // Skip Fridays, Saturdays, and Sundays
                        if matches!(day.weekday(), Fri | Sat | Sun) {
                            continue;
                        }

                        for role in Role::iter() {
                            if solution[day].get(role) == Some(over_p)
                                && !solution[day].has(under_p)
                            {
                                // Attempt a replacement (Move shift from over_p to under_p)
                                let perturbation =
                                    Perturbation::ReplacePerson(day, role, Some(under_p));

                                let patch = perturbation.apply(solution);

                                if PerturbationGenerator::new(
                                    solution,
                                    &self.context.holidays,
                                    n_people,
                                )
                                .is_valid(perturbation)
                                {
                                    let new_total = self.evaluator.evaluate(solution).total();

                                    if new_total <= best_total {
                                        best_total = new_total;
                                        deltas = self.evaluator.calculate_annual_deltas(solution);
                                        improved = true;

                                        break 'outer;
                                    }
                                }
                                patch.revert(solution);
                            }
                        }

                        // Try swapping roles if both are assigned
                        if solution[day].has(over_p) && solution[day].has(under_p) {
                            let perturbation = Perturbation::SwapRoles(day);
                            let patch = perturbation.apply(solution);

                            if PerturbationGenerator::new(
                                solution,
                                &self.context.holidays,
                                n_people,
                            )
                            .is_valid(perturbation)
                            {
                                let new_total = self.evaluator.evaluate(solution).total();

                                if new_total <= best_total {
                                    best_total = new_total;
                                    deltas = self.evaluator.calculate_annual_deltas(solution);
                                    improved = true;

                                    break 'outer;
                                }
                            }
                            patch.revert(solution);
                        }
                    }
                }
            }
        }
    }

    /// Greedy pass that breaks consecutive-day overlaps (Mon–Fri).
    ///
    /// Scans for any person working two consecutive days and attempts to
    /// reassign one of the slots to someone who doesn't work the adjacent
    /// day. Uses full fitness evaluation to ensure no regression.
    pub fn cleanup_consecutive(&self, solution: &mut Solution) {
        let Context { n_people, .. } = self.context;

        let mut best_total = self.evaluator.evaluate(solution).total();
        let mut improved = true;

        while improved {
            improved = false;

            for day in DayIdx::iter() {
                // Skip Saturday: Sat→Sun is structural (Sunday = Saturday.swapped())
                if day.weekday() == Sat {
                    continue;
                }

                let Some(next_day) = day.next() else {
                    continue;
                };

                // Fast check: do these two days overlap at all?
                if !solution[day].overlaps(solution[next_day]) {
                    continue;
                }

                for role in Role::iter() {
                    let Some(person) = solution[day].get(role) else {
                        continue;
                    };

                    // Only care if this person also works the next day
                    if !solution[next_day].has(person) {
                        continue;
                    }

                    // Try replacing with someone who works neither day
                    for other in PersonIdx::iter(n_people) {
                        if other == person {
                            continue;
                        }

                        if solution[day].has(other) || solution[next_day].has(other) {
                            continue;
                        }

                        let perturbation = Perturbation::ReplacePerson(day, role, Some(other));
                        let patch = perturbation.apply(solution);

                        if PerturbationGenerator::new(solution, &self.context.holidays, n_people)
                            .is_valid(perturbation)
                        {
                            let new_total = self.evaluator.evaluate(solution).total();

                            if new_total <= best_total {
                                best_total = new_total;
                                improved = true;
                                break;
                            }
                        }

                        patch.revert(solution);
                    }

                    if improved {
                        break;
                    }
                }

                if improved {
                    break;
                }
            }
        }
    }
}

/* -- RefinerProgress -- */

/// Live progress metrics for the refinement phase.
///
/// All fields use atomic operations so multiple threads can update them
/// concurrently. The frontend polls these values via serialized events.
/// Fitness and temperature values are stored as `f32::to_bits()` in `u64`.
pub struct RefinerProgress {
    pub accepted: AtomicU64,
    pub rejected: AtomicU64,
    pub current_fitness: AtomicU64,
    pub best_fitness: AtomicU64,
    pub temperature: AtomicU64,
    pub iteration: AtomicU64,
    pub search: AtomicU64,
}

impl RefinerProgress {
    pub fn new() -> Self {
        Self::default()
    }
}

impl Default for RefinerProgress {
    fn default() -> Self {
        Self {
            accepted: AtomicU64::new(0),
            rejected: AtomicU64::new(0),
            current_fitness: AtomicU64::new(0),
            best_fitness: AtomicU64::new(f32::MAX.to_bits() as u64),
            temperature: AtomicU64::new(0),
            iteration: AtomicU64::new(0),
            search: AtomicU64::new(0),
        }
    }
}

impl Clone for RefinerProgress {
    fn clone(&self) -> Self {
        Self {
            accepted: AtomicU64::new(self.accepted.load(Ordering::Relaxed)),
            rejected: AtomicU64::new(self.rejected.load(Ordering::Relaxed)),
            current_fitness: AtomicU64::new(self.current_fitness.load(Ordering::Relaxed)),
            best_fitness: AtomicU64::new(self.best_fitness.load(Ordering::Relaxed)),
            temperature: AtomicU64::new(self.temperature.load(Ordering::Relaxed)),
            iteration: AtomicU64::new(self.iteration.load(Ordering::Relaxed)),
            search: AtomicU64::new(self.search.load(Ordering::Relaxed)),
        }
    }
}

impl Serialize for RefinerProgress {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let current_fitness = f32::from_bits(self.current_fitness.load(Ordering::Relaxed) as u32);
        let best_fitness = f32::from_bits(self.best_fitness.load(Ordering::Relaxed) as u32);
        let temperature = f32::from_bits(self.temperature.load(Ordering::Relaxed) as u32);

        let mut s = serializer.serialize_struct("RefinerProgress", 7)?;
        s.serialize_field("accepted", &self.accepted.load(Ordering::Relaxed))?;
        s.serialize_field("rejected", &self.rejected.load(Ordering::Relaxed))?;
        s.serialize_field("current_fitness", &current_fitness)?;
        s.serialize_field("best_fitness", &best_fitness)?;
        s.serialize_field("temperature", &temperature)?;
        s.serialize_field("iteration", &self.iteration.load(Ordering::Relaxed))?;
        s.serialize_field("search", &self.search.load(Ordering::Relaxed))?;
        s.end()
    }
}

/* -- RefinerWorker -- */

struct RefinerWorker<'a> {
    context: &'a Context,
    evaluator: &'a ScheduleEvaluator<'a>,
    parameters: &'a RefinementParameters,
    controller: &'a ExecutionController,
    progress: &'a RefinerProgress,
    counter: &'a AtomicU64,
    base_solution: &'a Solution,
}

impl RefinerWorker<'_> {
    fn spin(&self) -> Result<Option<(f32, Solution)>, InterruptRequest> {
        let rng = &mut AppRng::try_from_rng(&mut SysRng).unwrap();

        let mut best: Option<(f32, Solution)> = None;

        let base_fitness = self.evaluator.evaluate(self.base_solution).total();
        let mut current_fitness = base_fitness;

        loop {
            let current = self.counter.load(Ordering::Relaxed);

            if current == 0 {
                break;
            }

            if self
                .counter
                .compare_exchange(current, current - 1, Ordering::Relaxed, Ordering::Relaxed)
                .is_err()
            {
                continue;
            }

            // Report which search we're starting (1-indexed for display)
            let search_num = self.parameters.searches - current + 1;
            self.progress.search.store(search_num, Ordering::Relaxed);

            self.controller.assert()?;

            let mut state = RefinementState {
                iteration: 0,
                temperature: self.parameters.initial_temperature,
            };

            let mut current_solution = self.base_solution.clone();

            for _ in 0..self.parameters.num_iterations {
                self.controller.assert()?;

                state.iteration += 1;
                self.progress.iteration.fetch_add(1, Ordering::Relaxed);
                self.progress
                    .temperature
                    .store(state.temperature.to_bits() as u64, Ordering::Relaxed);

                let Some(patch) = PerturbationGenerator::new(
                    &mut current_solution,
                    &self.context.holidays,
                    self.context.n_people,
                )
                .find(rng) else {
                    // Could not find a valid perturbation, skip this iteration
                    self.progress.increment_rejected();
                    state.temperature *= self.parameters.cooling_rate;
                    continue;
                };

                let new_fitness = self.evaluator.evaluate(&current_solution).total();
                self.progress
                    .current_fitness
                    .store(new_fitness.to_bits() as u64, Ordering::Relaxed);

                let delta = current_fitness - new_fitness;

                if delta > 0. || rng.random::<f32>() < (delta / state.temperature).exp() {
                    // Accept the move (update current state)
                    current_fitness = new_fitness;
                    self.progress.increment_accepted();

                    // If this is the best we've ever seen, save it
                    if best.as_ref().is_none_or(|(b, _)| new_fitness < *b) {
                        self.progress.report_best_fitness(new_fitness);
                        best = Some((new_fitness, current_solution.clone()));
                    }
                } else {
                    patch.revert(&mut current_solution);
                    self.progress.increment_rejected();
                }

                state.temperature *= self.parameters.cooling_rate;

                if state.temperature < TEMPERATURE_THRESHOLD {
                    break;
                }
            }
        }

        Ok(best)
    }
}

impl RefinerProgress {
    fn increment_accepted(&self) {
        self.accepted.fetch_add(1, Ordering::Relaxed);
    }

    fn increment_rejected(&self) {
        self.rejected.fetch_add(1, Ordering::Relaxed);
    }

    fn report_best_fitness(&self, fitness: f32) {
        let mut current = self.best_fitness.load(Ordering::Relaxed);
        loop {
            if fitness >= f32::from_bits(current as u32) {
                break;
            }
            match self.best_fitness.compare_exchange_weak(
                current,
                fitness.to_bits() as u64,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }
}

/* -- Parameters -- */

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RefinementParameters {
    pub cooling_rate: f32,
    pub initial_temperature: f32,
    pub num_iterations: u64,
    pub polish: bool,
    pub searches: u64,
}

/* -- State -- */

#[derive(Default)]
struct RefinementState {
    iteration: u64,
    temperature: f32,
}

/* -- Perturbation -- */

/// A small modification to the schedule that can be applied during simulated annealing.
///
/// Each perturbation represents a different type of local change to explore the solution space:
/// - `ReplacePerson`: Substitute one person for another in a specific role on a specific day
/// - `SwapDays`: Exchange the complete assignments of two days (preserving weekend constraints)
/// - `SwapRoles`: Swap the lead and support roles within a single day
///
/// Perturbations are designed to be their own inverse (except `ReplacePerson` which stores the
/// old person), allowing efficient reversion when a change is rejected. Sunday is excluded from
/// direct modification since it's always derived from Saturday.
#[derive(Copy, Clone, Debug, EnumCount)]
enum Perturbation {
    /// Replace the person assigned to a role on a specific day.
    ReplacePerson(DayIdx, Role, Option<PersonIdx>),

    /// Swap the complete slot assignments between two days.
    SwapDays(DayIdx, DayIdx),

    /// Swap the lead and support roles within a single day.
    SwapRoles(DayIdx),

    /// Swap a role between two days.
    SwapRoleBetweenDays(DayIdx, DayIdx, Role),
}

/// A patch that can revert a perturbation to restore the previous schedule state.
///
/// Created when a perturbation is applied, this stores enough information to undo the change.
/// For `SwapDays` and `SwapRoles`, the perturbation is its own inverse. For `ReplacePerson`,
/// the patch stores the old person that was replaced.
struct Patch(Perturbation);

impl Patch {
    /// Revert the perturbation applied to the given slots.
    ///
    /// For `SwapDays` and `SwapRoles`, this re-applies the same perturbation (as they are
    /// involutions). For `ReplacePerson`, this applies a perturbation that restores the
    /// old person stored in the patch.
    fn revert(self, slots: &mut SlotArray) {
        self.0.apply(slots);
    }
}

impl Perturbation {
    /// Apply this perturbation to the schedule and return a patch for reverting it.
    ///
    /// This method modifies the slots array in-place and maintains the weekend constraint
    /// that Sunday assignments must be the reverse of Saturday's (lead ↔ support).
    fn apply(self, slots: &mut SlotArray) -> Patch {
        match self {
            // Replace a person in a role, remembering the old person for reverting
            Perturbation::ReplacePerson(day, role, mut person) => {
                person = slots[day].replace(role, person);

                // If modifying Saturday, update Sunday to maintain the swapped constraint
                if let Some(sunday) = saturday_with_next(day) {
                    slots[sunday] = slots[day].swapped();
                }

                Patch(Perturbation::ReplacePerson(day, role, person))
            }

            // Swap the complete assignments between two days
            Perturbation::SwapDays(a, b) => {
                (slots[a], slots[b]) = (slots[b], slots[a]);

                // Update Sunday assignments if either swapped day is a Saturday
                if let Some(sunday) = saturday_with_next(a) {
                    slots[sunday] = slots[a].swapped();
                }

                if let Some(sunday) = saturday_with_next(b) {
                    slots[sunday] = slots[b].swapped();
                }

                Patch(self)
            }

            // Swap lead and support roles within a single day
            Perturbation::SwapRoles(day) => {
                slots[day] = slots[day].swapped();

                if let Some(sunday) = saturday_with_next(day) {
                    slots[sunday] = slots[day].swapped();
                }

                Patch(self)
            }

            Perturbation::SwapRoleBetweenDays(a, b, role) => {
                // Normalise Sunday to Saturday, is_valid() already checks for different weeks
                let [a, b] = [a, b].map(|day| match day.weekday() {
                    Sun => day.prev().unwrap(),
                    _ => day,
                });

                let p_a = slots[a].get(role);
                let p_b = slots[b].get(role);

                slots[a].replace(role, p_b);
                slots[b].replace(role, p_a);

                // Update sundays
                if let Some(sunday) = saturday_with_next(a) {
                    slots[sunday] = slots[a].swapped();
                }

                if let Some(sunday) = saturday_with_next(b) {
                    slots[sunday] = slots[b].swapped();
                }

                Patch(self)
            }
        }
    }
}

/// Generates valid perturbations for simulated annealing refinement.
///
/// The generator ensures that all produced perturbations maintain the hard constraints from
/// the specification, including:
/// - No person assigned to both roles on the same day
/// - No support assigned to someone who was lead the previous day (except weekends)
/// - Weekend workers cannot work the preceding Friday
/// - Weekend workers can have at most one weekday shift that week
/// - Respect employee holidays
///
/// The generator takes a conservative approach, rejecting some perturbations that might
/// technically be valid but could lead to complex validation scenarios.
struct PerturbationGenerator<'a> {
    holidays: &'a WeeklyMask,
    slots: &'a mut SlotArray,
    n_people: usize,
}

impl PerturbationGenerator<'_> {
    pub fn new<'a>(
        slots: &'a mut SlotArray,
        holidays: &'a WeeklyMask,
        n_people: usize,
    ) -> PerturbationGenerator<'a> {
        PerturbationGenerator {
            slots,
            holidays,
            n_people,
        }
    }

    /// Maximum number of rejected perturbations before giving up on this iteration.
    const MAX_REJECTION_ATTEMPTS: u32 = 1_000;

    /// Generate and apply a random valid perturbation.
    ///
    /// Samples random perturbations until finding one that satisfies all
    /// validation rules, or until [`Self::MAX_REJECTION_ATTEMPTS`] is exhausted.
    pub fn find<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Option<Patch> {
        for _ in 0..Self::MAX_REJECTION_ATTEMPTS {
            let perturbation = self.random_perturbation(rng);

            let patch = perturbation.apply(self.slots);

            if self.is_valid(perturbation) {
                return Some(patch);
            }

            patch.revert(self.slots);
        }

        None
    }

    /// Generate a completely random perturbation without validation. Sunday
    /// is excluded from random day selection since it's derived from Saturday.
    fn random_perturbation<R: Rng + ?Sized>(&self, rng: &mut R) -> Perturbation {
        let day = Self::random_day(rng);

        match rng.random_range(0..Perturbation::COUNT) {
            0 => {
                let p_idx = rng.random_range(0..self.n_people as u8);
                let person = unsafe { PersonIdx::new_unchecked(p_idx) };
                Perturbation::ReplacePerson(day, rng.random(), Some(person))
            }
            1 => Perturbation::SwapDays(day, Self::random_day(rng)),
            2 => Perturbation::SwapRoles(day),
            3 => Perturbation::SwapRoleBetweenDays(day, Self::random_day(rng), rng.random()),
            _ => unreachable!(),
        }
    }

    /// Validate that a perturbation maintains all hard constraints.
    ///
    /// This performs comprehensive validation specific to each perturbation type,
    /// ensuring that applying it won't violate any scheduling rules.
    fn is_valid(&self, perturbation: Perturbation) -> bool {
        match perturbation {
            Perturbation::ReplacePerson(day, role, person) => {
                // Reject Friday AND Saturday modifications to preserve distribution
                if matches!(day.weekday(), Fri | Sat | Sun) {
                    return false;
                }

                let person = person.unwrap(); // valid forward perturbation should never be None

                self.slots[day].is_valid()
                    && self.no_adjacent_day_conflict(day)
                    && self.no_friday_weekend_conflict(day)
                    && self.no_work_count_conflict(day)
                    && self.no_role_holiday_conflict(day, role, person)
            }

            Perturbation::SwapDays(a, b) => {
                let [class_a, class_b] = [a, b].map(|day| match day.weekday() {
                    Mon | Tue | Wed | Thu => 0,
                    Fri => 1,
                    Sat | Sun => 2,
                });

                // Ensure we only swap equivalent day types (Sat with Sat, Fri with Fri, Weekday with Weekday)
                if a == b || class_a != class_b {
                    return false;
                }

                self.no_adjacent_day_conflict(a)
                    && self.no_adjacent_day_conflict(b)
                    && self.no_friday_weekend_conflict(a)
                    && self.no_friday_weekend_conflict(b)
                    && self.no_work_count_conflict(a)
                    && self.no_work_count_conflict(b)
                    && self.no_slot_holiday_conflict(a)
                    && self.no_slot_holiday_conflict(b)
            }

            Perturbation::SwapRoles(day) => {
                // Don't swap Friday or Weekends (protect distribution)
                matches!(day.weekday(), Mon | Tue | Wed | Thu)
                    && self.no_adjacent_day_conflict(day)
                    && self.no_slot_holiday_conflict(day)
            }

            Perturbation::SwapRoleBetweenDays(a, b, _) => {
                let [class_a, class_b] = [a, b].map(|day| match day.weekday() {
                    Mon | Tue | Wed | Thu => 0,
                    Fri => 1,
                    Sat | Sun => 2,
                });

                if a == b || class_a != class_b {
                    return false;
                }

                // Must be different weeks for weekends, Sat and Sun are analogous
                if class_a == 2 && a.week() == b.week() {
                    return false;
                }

                self.slots[a].is_valid()
                    && self.slots[b].is_valid()
                    && self.no_adjacent_day_conflict(a)
                    && self.no_adjacent_day_conflict(b)
                    && self.no_friday_weekend_conflict(a)
                    && self.no_friday_weekend_conflict(b)
                    && self.no_work_count_conflict(a)
                    && self.no_work_count_conflict(b)
                    // Must recheck holidays since we moved individuals to new weeks
                    && self.no_slot_holiday_conflict(a)
                    && self.no_slot_holiday_conflict(b)
            }
        }
    }

    /// Check that the slot doesn't violate the weekend work count constraint.
    ///
    /// From spec: "If working a weekend, at most one weekday shift that week."
    /// This validates that any person in the slot, if also working the weekend,
    /// doesn't exceed one weekday shift in the same week.
    fn no_work_count_conflict(&self, day: DayIdx) -> bool {
        let week = day.week();

        Role::iter().all(|role| {
            self.slots[day]
                .get(role)
                .is_none_or(|person| self.is_correct_working_day_count(week, person))
        })
    }

    /// Check if a person working this week has a valid weekday count.
    ///
    /// Returns true if:
    /// - The person is not working the weekend, OR
    /// - The person is working the weekend AND has at most 1 weekday shift (Mon-Thu)
    fn is_correct_working_day_count(&self, week: WeekIdx, person: PersonIdx) -> bool {
        if !self.slots[week.weekday(Sat)].has(person) {
            return true;
        }

        week.weekdays() // Mon-Thu only
            .into_iter()
            .flat_map(|d| Role::iter().flat_map(move |role| self.slots[d].get(role)))
            .filter(|&p| p == person)
            .count()
            <= 1
    }

    /// Check that the slot doesn't create invalid adjacent day sequences.
    ///
    /// From spec: "Cannot assign support to an employee who worked lead the previous day."
    /// This checks both the previous and next day to ensure the succession is valid.
    fn no_adjacent_day_conflict(&self, day: DayIdx) -> bool {
        [
            (day.prev(), Some(day)),
            match saturday_with_next(day) {
                Some(sunday) => (Some(sunday), sunday.next()),
                _ => (Some(day), day.next()),
            },
        ]
        .into_iter()
        .all(|(prev, next)| {
            prev.zip(next)
                .is_none_or(|(p, n)| Self::valid_succession((self.slots[p], self.slots[n])))
        })
    }

    /// Check that the slot doesn't violate the Friday-weekend work restriction.
    ///
    /// From spec: "If assigned to a weekend, cannot work the preceding Friday (either role)."
    /// This checks:
    /// - If the day is Friday, ensure no overlap with next day's Saturday
    /// - If the day is Saturday, ensure no overlap with previous day's Friday
    fn no_friday_weekend_conflict(&self, day: DayIdx) -> bool {
        if let Some(saturday) = friday_with_next(day)
            && self.slots[saturday].overlaps(self.slots[day])
        {
            return false;
        }

        if let Some(friday) = saturday_with_prev(day)
            && self.slots[friday].overlaps(self.slots[day])
        {
            return false;
        }

        true
    }

    /// Check that assigning a slot on a day doesn't conflict with holidays.
    fn no_slot_holiday_conflict(&self, day: DayIdx) -> bool {
        Role::iter().all(|role| {
            self.slots[day]
                .get(role)
                .is_none_or(|person| self.no_role_holiday_conflict(day, role, person))
        })
    }

    /// Check that assigning a person to a role on a day doesn't conflict with holidays.
    ///
    /// From spec: "Employee unavailable for the specified week, preceding weekend, and
    /// preceding Friday's lead role."
    ///
    /// This checks:
    /// - The person is not on holiday during the assignment week
    /// - If the assignment affects the next week (Sat/Sun or Fri lead), the person
    ///   is not on holiday next week
    fn no_role_holiday_conflict(&self, day: DayIdx, role: Role, person: PersonIdx) -> bool {
        let week = day.week();

        !self.holidays.get(week, person)
            && self.no_holiday_conflict_with_next(week, day.weekday(), role, person)
    }

    /// Check if an assignment would conflict with next week's holiday.
    ///
    /// Certain assignments affect the following week:
    /// - Saturday assignments (both roles cover weekend into next week)
    /// - Friday lead (affects next week per holiday rules)
    fn no_holiday_conflict_with_next(
        &self,
        week: WeekIdx,
        weekday: Weekday,
        role: Role,
        person: PersonIdx,
    ) -> bool {
        if (weekday == Sat || (weekday == Fri && role == Role::Lead))
            && let Some(next_week) = week.next()
        {
            !self.holidays.get(next_week, person)
        } else {
            true
        }
    }

    /// Check if two consecutive slots form a valid succession.
    ///
    /// From spec: "Cannot assign support to an employee who worked lead the previous day."
    /// Returns true if the previous day's lead is different from the next day's support.
    fn valid_succession(pair: (Slot, Slot)) -> bool {
        pair.0.get(Role::Lead) != pair.1.get(Role::Support)
    }

    /// Generate a random day index (Mon-Sat, excludes Sunday).
    ///
    /// Sunday is excluded because it's automatically derived from Saturday
    /// (Sunday = Saturday.swapped()), so there's no need to directly modify it.
    fn random_day<R: Rng + ?Sized>(rng: &mut R) -> DayIdx {
        let week: WeekIdx = rng.random();
        let weekday = Weekday::try_from(rng.random_range(0..6)).unwrap(); // 0=Mon, 5=Sat
        week.weekday(weekday)
    }
}

/* -- Miscellaneous -- */

/// If the day is Friday, return the next day (Saturday) without bounds checking.
///
/// # Safety
/// Safe because Friday + 1 is always a valid day index (Saturday exists in any week
/// that contains a Friday, and the schedule covers complete weeks).
fn friday_with_next(day: DayIdx) -> Option<DayIdx> {
    (day.weekday() == Fri).then(|| unsafe { DayIdx::new_unchecked(day.get().wrapping_add(1)) })
}

/// If the day is Saturday, return the previous day (Friday) without bounds checking.
///
/// # Safety
/// Safe because Saturday - 1 is always a valid day index (Friday exists in any week
/// that contains a Saturday, and the schedule covers complete weeks).
fn saturday_with_prev(day: DayIdx) -> Option<DayIdx> {
    (day.weekday() == Sat).then(|| unsafe { DayIdx::new_unchecked(day.get().wrapping_sub(1)) })
}

/// If the day is Saturday, return the next day (Sunday) without bounds checking.
///
/// # Safety
/// Safe because Saturday + 1 is always a valid day index (Sunday exists in any week
/// that contains a Saturday, and the schedule covers complete weeks).
fn saturday_with_next(day: DayIdx) -> Option<DayIdx> {
    (day.weekday() == Sat).then(|| unsafe { DayIdx::new_unchecked(day.get().wrapping_add(1)) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{N_DAYS, solver::types::WeeklyMask};

    /// Verify that swapping a role between two Saturdays correctly synchronizes
    /// their corresponding Sundays.
    #[test]
    fn swap_role_between_days_syncs_sunday_correctly() {
        let mut slots = SlotArray::new([Slot::default(); N_DAYS]);
        let p1 = PersonIdx::new(0).unwrap();
        let p2 = PersonIdx::new(1).unwrap();
        let p3 = PersonIdx::new(2).unwrap();

        let week0 = WeekIdx::try_new(0).unwrap();
        let week1 = WeekIdx::try_new(1).unwrap();
        let sat0 = week0.weekday(Sat);
        let sun0 = week0.weekday(Sun);
        let sat1 = week1.weekday(Sat);
        let sun1 = week1.weekday(Sun);

        slots[sat0] = Slot::new(Some(p1), Some(p2));
        slots[sun0] = Slot::new(Some(p2), Some(p1));
        slots[sat1] = Slot::new(Some(p3), Some(p2));
        slots[sun1] = Slot::new(Some(p2), Some(p3));

        let perturbation = Perturbation::SwapRoleBetweenDays(sat0, sat1, Role::Lead);
        perturbation.apply(&mut slots);

        // Sat0 Lead was p1, now should be p3.
        assert_eq!(slots[sat0].get(Role::Lead), Some(p3));
        // Sun0 should be mirror of Sat0: Lead=p2, Support=p3
        assert_eq!(
            slots[sun0].get(Role::Support),
            Some(p3),
            "Sunday Support should follow Saturday Lead"
        );
        assert_eq!(
            slots[sun0].get(Role::Lead),
            Some(p2),
            "Sunday Lead should follow Saturday Support"
        );

        // Sat1 Lead was p3, now should be p1.
        assert_eq!(slots[sat1].get(Role::Lead), Some(p1));
        // Sun1 should be mirror of Sat1: Lead=p2, Support=p1
        assert_eq!(
            slots[sun1].get(Role::Support),
            Some(p1),
            "Sunday Support should follow Saturday Lead"
        );
    }

    /// Verify that swapping two Fridays is rejected when it would put a person
    /// on both Friday and Saturday of the same week (Friday-weekend constraint).
    #[test]
    fn swap_fridays_rejects_friday_weekend_overlap() {
        let mut slots = SlotArray::new([Slot::default(); N_DAYS]);

        let a = PersonIdx::new(0).unwrap();
        let b = PersonIdx::new(1).unwrap();
        let c = PersonIdx::new(2).unwrap();
        let d = PersonIdx::new(3).unwrap();
        let e = PersonIdx::new(4).unwrap();
        let f = PersonIdx::new(5).unwrap();
        let g = PersonIdx::new(6).unwrap();

        let week0 = WeekIdx::try_new(0).unwrap();
        let week5 = WeekIdx::try_new(5).unwrap();

        let fri0 = week0.weekday(Fri);
        let sat0 = week0.weekday(Sat);
        let fri5 = week5.weekday(Fri);
        let sat5 = week5.weekday(Sat);

        // Week 0: A and B on Friday, C and D on Saturday (no overlap)
        slots[fri0] = Slot::new(Some(a), Some(b));
        slots[sat0] = Slot::new(Some(c), Some(d));

        // Week 5: E and F on Friday, A and G on Saturday
        // Person A works Saturday week 5
        slots[fri5] = Slot::new(Some(e), Some(f));
        slots[sat5] = Slot::new(Some(a), Some(g));

        let holidays = WeeklyMask::default();

        // Mirror find()'s apply-then-check flow
        let perturbation = Perturbation::SwapDays(fri0, fri5);
        let patch = perturbation.apply(&mut slots);

        let valid = {
            let pg = PerturbationGenerator::new(&mut slots, &holidays, 7);
            pg.is_valid(perturbation)
        };

        patch.revert(&mut slots);

        // Swapping Friday[0] and Friday[5] puts A on Friday[5],
        // and A is already on Saturday[5] — violates the Friday-weekend rule.
        assert!(!valid);
    }

    /// Verify cleanup_consecutive eliminates a deliberate Mon-Tue overlap
    /// without worsening other fitness components.
    #[test]
    fn cleanup_consecutive_breaks_overlap() {
        use crate::{ProblemInput, ProblemOverrides, utils::sample_people};
        use chrono::NaiveDate;

        let start_date = NaiveDate::from_ymd_opt(2024, 8, 19).unwrap();
        let weekday_hours = [[14.5, 7.5]; 7];

        let problem = ProblemInput::try_new(
            start_date,
            sample_people().to_vec(),
            ProblemOverrides::default(),
            weekday_hours,
            3,
        )
        .unwrap();

        let weights = Weights::STANDARD;
        let refiner = Refiner::new(&problem, &weights);

        // Build a schedule with a deliberate Mon-Tue overlap
        let mut slots = [Slot::default(); N_DAYS];
        let p0 = PersonIdx::new(0).unwrap();
        let p1 = PersonIdx::new(1).unwrap();
        let p2 = PersonIdx::new(2).unwrap();

        let week0 = WeekIdx::try_new(0).unwrap();
        let mon0 = week0.weekday(Mon);
        let tue0 = week0.weekday(Tue);

        // Person 0 works Monday Lead AND Tuesday Support → consecutive overlap
        slots[mon0] = Slot::new(Some(p0), Some(p1));
        slots[tue0] = Slot::new(Some(p2), Some(p0));

        // Fill rest with a valid baseline (no overlaps elsewhere)
        for week in WeekIdx::iter() {
            if week == week0 {
                continue;
            }
            let mon = week.weekday(Mon);
            let tue = week.weekday(Tue);
            let wed = week.weekday(Wed);
            let thu = week.weekday(Thu);
            let fri = week.weekday(Fri);
            let sat = week.weekday(Sat);

            slots[mon] = Slot::new(Some(p0), Some(p1));
            slots[tue] = Slot::new(Some(p1), Some(p0));
            slots[wed] = Slot::new(Some(p0), Some(p2));
            slots[thu] = Slot::new(Some(p2), Some(p0));
            slots[fri] = Slot::new(Some(p1), Some(p2));
            slots[sat] = Slot::new(Some(p0), Some(p1));

            // Sunday auto-derived via swapped() in the evaluator
            let sun = week.weekday(Sun);
            slots[sun] = slots[sat].swapped();
        }

        let mut solution = Solution::from_slot_array(&slots);

        let fitness_before = refiner.evaluator.evaluate(&solution).total();
        let cons_before = refiner.evaluator.evaluate(&solution).consecutive_days;

        refiner.cleanup_consecutive(&mut solution);

        let fitness_after = refiner.evaluator.evaluate(&solution).total();
        let cons_after = refiner.evaluator.evaluate(&solution).consecutive_days;

        // Fitness should not regress
        assert!(
            fitness_after <= fitness_before,
            "cleanup_consecutive worsened fitness: {fitness_before:.2} → {fitness_after:.2}"
        );

        // Consecutive days should decrease (the Mon-Tue overlap was removed)
        assert!(
            cons_after < cons_before,
            "cleanup_consecutive did not reduce consecutive_days: {cons_before:.2} → {cons_after:.2}"
        );
    }
}

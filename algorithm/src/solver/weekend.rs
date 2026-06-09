use std::array;

use chrono::Weekday;
use rand::{Rng, RngExt, seq::SliceRandom};
use strum::EnumCount;

use crate::{
    Weights,
    defs::*,
    fitness::ScheduleEvaluator,
    solver::{
        context::{Context, PlacementContext, SingleAssignment},
        defs::WeekendParameters,
        types::WeeklyRoleMask,
    },
    types::{PersonIdx, Role, Slot, WeekIdx},
    utils::AtomicProgress,
};

pub struct WeekendSolver<'a> {
    context: &'a Context,
    distribution: Distribution,
    parameters: &'a WeekendParameters,
    state: WeekendState,
}

struct WeekendState {
    assign_order: [WeekIdx; N_WEEKS],
    assignment: SingleAssignment,
    cursor: u64,
    mask: WeeklyRoleMask,
}

impl Default for WeekendState {
    fn default() -> Self {
        WeekendState {
            assign_order: WeekIdx::array(),
            assignment: SingleAssignment::default(),
            cursor: 0,
            mask: WeeklyRoleMask::default(),
        }
    }
}

impl WeekendSolver<'_> {
    pub fn new<'a>(parameters: &'a WeekendParameters, context: &'a Context) -> WeekendSolver<'a> {
        WeekendSolver {
            context,
            distribution: WeekendRoleDistribution(context).proportionate_to_rate(),
            parameters,
            state: WeekendState::default(),
        }
    }

    /// Generate a single valid weekend arrangement (one fill + mini hill-climb refiner).
    /// Returns the best assignment found or None if no valid arrangement was produced.
    pub fn generate<R: Rng + ?Sized>(
        &mut self,
        weights: &Weights,
        progress: &AtomicProgress,
        rng: &mut R,
    ) -> Option<&SingleAssignment> {
        self.state.cursor = 0;
        self.state.mask = self.weekend_mask();

        // 1. Shuffle to randomize tie-breakers
        self.state.assign_order.shuffle(rng);

        // 2. Sort by DOF (most constrained first).
        // Use `sort_by_key` (stable) so the random shuffle is preserved for identical DOFs!
        self.state
            .assign_order
            .sort_by_key(|&week| self.state.mask.dof(week));

        // 3. Try to generate ONE valid starting configuration
        if self.fill_new_slots(rng).is_err() {
            return None; // If it backed itself into a corner, abort and try again
        }

        // 4. Mini Hill-Climb Refiner

        // Helper closure to calculate the weighted score for the mini-refiner
        let calc_score = |assignment: &SingleAssignment| -> f32 {
            let (spacing, alt) = ScheduleEvaluator::evaluate_weekend_metrics(
                assignment.iter().copied(),
                self.context.n_people,
            );

            // Apply the actual weights so the mini-refiner matches the global evaluator
            (spacing * weights.weekend_regularity) + (alt * weights.weekend_alternation)
        };

        let mut current_score = calc_score(&self.state.assignment);

        for _ in 0..self.parameters.hill_climb_iterations {
            let week_a: WeekIdx = rng.random();
            let week_b: WeekIdx = rng.random();

            let roles = [Role::Lead, Role::Support];
            let role = roles[rng.random_range(0..2)];

            if week_a == week_b {
                continue;
            }

            let p_a = self.state.assignment[week_a].get(role).unwrap();
            let p_b = self.state.assignment[week_b].get(role).unwrap();

            if p_a == p_b {
                continue;
            }

            // Validity Check 1: Hard constraints (Holidays + Margins)
            if self.state.mask.get_role(week_a, p_b, role)
                || self.state.mask.get_role(week_b, p_a, role)
            {
                continue;
            }

            // Validity Check 2: A person cannot work both Lead and Support on the same
            // weekend. If self.state.assignment[week_a] already has p_b, it means
            // p_b is in the *other* role.
            if self.state.assignment[week_a].has(p_b) || self.state.assignment[week_b].has(p_a) {
                continue;
            }

            // Apply swap
            self.state.assignment[week_a].replace(role, Some(p_b));
            self.state.assignment[week_b].replace(role, Some(p_a));

            let new_score = calc_score(&self.state.assignment);

            // Accept if better or equal (allowing it to walk across score plateaus)
            if new_score <= current_score {
                current_score = new_score;
            } else {
                // Revert swap
                self.state.assignment[week_a].replace(role, Some(p_a));
                self.state.assignment[week_b].replace(role, Some(p_b));
            }
        }

        progress.increment_accepted();
        Some(&self.state.assignment)
    }

    fn weekend_mask(&self) -> WeeklyRoleMask {
        let mut mask = WeeklyRoleMask::from_holidays(&self.context.holidays, self.context.n_people);
        mask.flood_left();
        mask
    }

    fn fill_new_slots<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), WeekIdx> {
        self.assign_slots(rng)?;
        self.resolve_double_slots(rng)?;
        self.resolve_conflicts(rng)
    }

    fn assign_slots<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), WeekIdx> {
        let mut counts = self.distribution;

        // In debug, clear the array to check if all were properly assigned below
        #[cfg(debug_assertions)]
        {
            self.state.assignment = SingleAssignment::default();
        }

        for &week in &self.state.assign_order {
            let lead = self.pick_role(&mut counts[Role::Lead], week, Role::Lead, rng)?;
            let support = self.pick_role(&mut counts[Role::Support], week, Role::Support, rng)?;

            self.state.assignment[week] = Slot::new(Some(lead), Some(support));
        }

        // In debug, check that all slots were assigned
        debug_assert!((self.state.assignment.iter().copied()).all(Slot::is_assigned));

        Ok(())
    }

    fn pick_role<R: Rng + ?Sized>(
        &self,
        counts: &mut [u8; MAX_PEOPLE],
        week: WeekIdx,
        role: Role,
        rng: &mut R,
    ) -> Result<PersonIdx, WeekIdx> {
        let mut mask = self.state.mask.get_mask_for_role(week, role);

        for (i, count) in counts.iter().enumerate() {
            if *count == 0 {
                mask.0 |= 1 << i;
            }
        }

        let real_all_set = ((1u16 << self.context.n_people) - 1) & mask.0;
        if real_all_set == (1u16 << self.context.n_people) - 1 {
            return Err(week);
        }

        // Bits n_people..14 are 1 (dummies have count=0 → set above).
        // Bit 15 forced via | 0x8000. count_zeros() thus counts only
        // available real people.
        let candidate_offset = rng.random_range(0..(mask.0 | 0x8000).count_zeros() as u8);

        let person = PersonIdx::iter(self.context.n_people)
            .filter(|p| mask.0 & (1 << p.get()) == 0)
            .nth(candidate_offset as usize)
            .expect("mask inconsistent with count_zeros");

        counts[person] -= 1;
        Ok(person)
    }

    fn resolve_double_slots<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), WeekIdx> {
        let context = PlacementContext {
            holidays: &self.context.holidays,
            mask: &self.state.mask,
            weekday: Weekday::Sat,
        };

        for week in WeekIdx::iter() {
            let slot = self.state.assignment[week];

            if slot.is_valid() {
                continue;
            }

            let valid_swaps = self
                .state
                .assignment
                .number_valid_support_swaps(week, &context);

            if valid_swaps == 0 {
                return Err(week);
            }

            let offset = rng.random_range(0..valid_swaps);

            let (other_week, _) = self
                .state
                .assignment
                .walk_valid_support_swaps_for(week, &context)
                .nth(offset)
                .expect("valid_swaps inconsistent with walk");

            if !self
                .state
                .assignment
                .try_swap_support(week, other_week, &context)
            {
                panic!("expected support swap to be valid");
            }
        }

        #[cfg(debug_assertions)]
        if !self.state.assignment.iter().copied().all(Slot::is_valid) {
            panic!("expected all slots to be valid after support resolution");
        }

        Ok(())
    }

    fn resolve_conflicts<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Result<(), WeekIdx> {
        let context = PlacementContext {
            holidays: &self.context.holidays,
            mask: &self.state.mask,
            weekday: Weekday::Sat,
        };

        let mut last_week = WeekIdx::default();

        'outer: for _ in 0..self.parameters.max_resolve_attempts {
            for week in WeekIdx::iter() {
                if self.state.assignment.valid_at(week, &context) {
                    continue;
                }

                let valid_swaps = self.state.assignment.number_valid_swaps(week, &context);

                if valid_swaps == 0 {
                    self.state.assignment.shuffle(rng);
                    last_week = week;
                    continue 'outer;
                }

                let offset = rng.random_range(0..valid_swaps);

                let (j, _) = self
                    .state
                    .assignment
                    .walk_swaps_for(week, &context)
                    .nth(offset)
                    .expect("valid_swaps inconsistent with walk");

                if !self.state.assignment.try_swap(week, j, &context) {
                    panic!("expected swap to be valid");
                }
            }

            #[cfg(debug_assertions)]
            if !self.state.assignment.validate(&context) {
                panic!("expected assignment to be valid after conflict resolution");
            }

            return Ok(());
        }

        Err(last_week)
    }
}

/* == WeekendRoleDistribution == */

type Distribution = [[u8; MAX_PEOPLE]; Role::COUNT];

struct WeekendRoleDistribution<'a>(&'a Context);

impl WeekendRoleDistribution<'_> {
    fn proportionate_to_rate(&self) -> Distribution {
        let mut counters = [[0; MAX_PEOPLE]; Role::COUNT];
        let n_people = self.0.n_people;
        let work_shares: [f32; MAX_PEOPLE] = array::from_fn(|i| self.0.people[i].work_share);

        let total_slots = (N_WEEKS * 2) as f32; // Pool all 96 slots together

        let ideal_roles: [f32; MAX_PEOPLE] = work_shares.map(|share| share * total_slots);
        let mut assigned_counts: [u8; MAX_PEOPLE] = ideal_roles.map(|x| x.floor() as u8);

        let mut remaining = (N_WEEKS * 2) as u8 - assigned_counts.iter().sum::<u8>();

        // Distribute remainder
        while remaining > 0 {
            let (person_to_increment, _) = assigned_counts
                .iter()
                .zip(ideal_roles.iter())
                .take(n_people)
                .enumerate()
                .map(|(i, (assigned, ideal))| (i, ideal - *assigned as f32))
                .max_by(|(_, error1), (_, error2)| f32::total_cmp(error1, error2))
                .unwrap();

            assigned_counts[person_to_increment] += 1;
            remaining -= 1;
        }

        // Now cleanly divide the pool into Lead and Support
        let mut leads_assigned = 0;

        for i in 0..n_people {
            let total = assigned_counts[i];
            counters[Role::Lead][i] = total / 2;
            counters[Role::Support][i] = total / 2;
            leads_assigned += total / 2;
        }

        // Hand out the odd remainders, ensuring Lead/Support arrays both sum exactly to 48
        for i in 0..n_people {
            if !assigned_counts[i].is_multiple_of(2) {
                if leads_assigned < N_WEEKS as u8 {
                    counters[Role::Lead][i] += 1;
                    leads_assigned += 1;
                } else {
                    counters[Role::Support][i] += 1;
                }
            }
        }

        counters
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::solver::defs::WeekdayParameters;

    use super::*;

    fn make_test_context() -> Context {
        let standard_rates: [u8; 8] = [80, 80, 75, 70, 60, 60, 55, 40];

        let people: Vec<Person> = standard_rates
            .iter()
            .enumerate()
            .map(|(i, &rate)| Person {
                name: format!("Person {}", i),
                holidays: vec![],
                rate: Rate::try_new(rate).unwrap(),
            })
            .collect();

        let start_date = NaiveDate::from_ymd_opt(2024, 8, 19).unwrap();
        let weekday_hours = [[14.5, 7.5]; N_WEEKDAYS];

        let problem = ProblemInput::try_new(
            start_date,
            people,
            ProblemOverrides::default(),
            weekday_hours,
            3,
        )
        .unwrap();

        Context::new(&problem)
    }

    #[test]
    fn test_role_distribution() {
        let context = make_test_context();

        let distribution = WeekendRoleDistribution(&context).proportionate_to_rate();

        let expected_leads = [8, 8, 7, 7, 5, 5, 5, 3, 0, 0, 0, 0, 0, 0, 0];
        let expected_supports = [7, 7, 7, 6, 6, 6, 5, 4, 0, 0, 0, 0, 0, 0, 0];

        assert_eq!(distribution[Role::Lead], expected_leads);
        assert_eq!(distribution[Role::Support], expected_supports);

        // Each role should sum to N_WEEKS (48)
        assert_eq!(distribution[Role::Lead].iter().sum::<u8>(), N_WEEKS as u8);
        assert_eq!(
            distribution[Role::Support].iter().sum::<u8>(),
            N_WEEKS as u8
        );
    }

    /// Smoke test: solver produces valid schedules for realistic person counts.
    /// Uses minimal parameters — just checking correctness, not quality.
    #[test]
    fn test_variable_people_count() {
        use crate::{
            ExecutionController, ScheduleValidator, Solver, SolverParameters, SolverProgress,
            solver::defs::WeekendParameters,
        };

        for n_people in [8_usize, 15] {
            let people: Vec<Person> = (0..n_people)
                .map(|i| Person {
                    name: format!("Person {}", i),
                    holidays: vec![],
                    rate: Rate::try_new(80).unwrap(),
                })
                .collect();

            let start_date = NaiveDate::from_ymd_opt(2024, 8, 19).unwrap();
            let weekday_hours = [[14.5, 7.5]; N_WEEKDAYS];

            let problem = ProblemInput::try_new(
                start_date,
                people,
                ProblemOverrides::default(),
                weekday_hours,
                3,
            )
            .unwrap();

            let context = Context::new(&problem);

            // Minimal parameters for smoke test: one permutation per stage
            let params = SolverParameters {
                weekend: WeekendParameters {
                    number_permutations: 1,
                    max_resolve_attempts: 50,
                    hill_climb_iterations: 1_000,
                },
                friday: WeekdayParameters {
                    number_permutations: 1,
                    max_resolve_attempts: 50,
                },
                weekday: WeekdayParameters {
                    number_permutations: 1,
                    max_resolve_attempts: 10,
                },
            };

            let mut solver = Solver::new(&problem, &Weights::STANDARD);
            let controller = ExecutionController::default();
            let progress = SolverProgress::default();

            let result = solver.execute(params, Some(1), &controller, &progress);

            let solution = result.expect(&format!("Solver failed for n_people={n_people}"));

            let errors: Vec<_> = ScheduleValidator::new(&context, &solution.solution)
                .validate()
                .collect();

            assert!(
                errors.is_empty(),
                "Validation errors for n_people={n_people}: {errors:?}"
            );
        }
    }
}

use rand::{Rng, RngExt};
use std::{
    array,
    ops::{Index, IndexMut},
};

use chrono::Weekday;
use strum::{EnumCount, EnumIter, IntoEnumIterator};

use crate::{
    defs::MAX_PEOPLE,
    solver::{
        context::{Context, ContextPerson, MultiAssignment, SingleAssignment},
        defs::PhaseParameters,
        types::{PersonMask, WeeklyRoleMask},
    },
    types::{PersonIdx, Role, Slot, WeekIdx},
    utils::AtomicProgress,
};

pub struct WeekdaySolver<'a> {
    context: &'a Context,
    parameters: &'a PhaseParameters,
    sampler: PersonSampler,
    state: WeekdayState,
}

#[derive(Default)]
struct WeekdayState {
    assignment: WeekdayAssignment,
    cursor: u64,
    friday_mask: WeeklyRoleMask,
    weekend_mask: WeeklyRoleMask,
}

pub type WeekdayAssignment = MultiAssignment<{ FirstWeekdays::COUNT }>;

#[derive(Copy, Clone, EnumCount, EnumIter, PartialEq, Eq)]
pub enum FirstWeekdays {
    Mon,
    Tue,
    Wed,
    Thu,
}

impl Index<FirstWeekdays> for WeekdayAssignment {
    type Output = SingleAssignment;

    fn index(&self, index: FirstWeekdays) -> &Self::Output {
        unsafe { self.0.get_unchecked(index as usize) }
    }
}

impl IndexMut<FirstWeekdays> for WeekdayAssignment {
    fn index_mut(&mut self, index: FirstWeekdays) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(index as usize) }
    }
}

impl TryFrom<Weekday> for FirstWeekdays {
    type Error = ();

    fn try_from(value: Weekday) -> Result<Self, Self::Error> {
        match value {
            Weekday::Mon => Ok(FirstWeekdays::Mon),
            Weekday::Tue => Ok(FirstWeekdays::Tue),
            Weekday::Wed => Ok(FirstWeekdays::Wed),
            Weekday::Thu => Ok(FirstWeekdays::Thu),
            _ => Err(()),
        }
    }
}

impl WeekdaySolver<'_> {
    pub fn new<'a>(parameters: &'a PhaseParameters, context: &'a Context) -> WeekdaySolver<'a> {
        WeekdaySolver {
            context,
            parameters,
            sampler: PersonSampler::new(&context.people, context.n_people),
            state: WeekdayState::default(),
        }
    }

    pub fn prime(&mut self, fridays: &SingleAssignment, saturdays: &SingleAssignment) {
        self.state.cursor = 0;
        self.state.friday_mask = WeeklyRoleMask::from_role_assignment(fridays);
        self.state.weekend_mask = WeeklyRoleMask::from_role_assignment(saturdays);
    }

    pub fn generate<'a, R: Rng + ?Sized>(
        &'a mut self,
        saturdays: &SingleAssignment,
        progress: &AtomicProgress,
        rng: &mut R,
    ) -> Option<&'a WeekdayAssignment> {
        while self.state.cursor < self.parameters.number_permutations {
            self.state.cursor += 1;

            match self.assign_weekly_slots(saturdays, rng) {
                Ok(()) => {
                    progress.increment_accepted();
                    return Some(&self.state.assignment);
                }

                Err(failed_week) => {
                    progress.increment_rejected();
                    progress.increment_week_failure(failed_week);
                }
            }
        }

        None
    }

    fn assign_weekly_slots<R: Rng + ?Sized>(
        &mut self,
        saturdays: &SingleAssignment,
        rng: &mut R,
    ) -> Result<(), WeekIdx> {
        let mut sunday_lead = None;

        for week in WeekIdx::iter() {
            self.assign_week_slots(week, sunday_lead, rng)?;
            sunday_lead = Some(saturdays[week].get(Role::Support).unwrap());
        }

        Ok(())
    }

    fn assign_week_slots<R: Rng + ?Sized>(
        &mut self,
        week: WeekIdx,
        mut sunday_lead: Option<PersonIdx>,
        rng: &mut R,
    ) -> Result<(), WeekIdx> {
        let mut last_failed_week = WeekIdx::default();
        let mut person_assigned_mask = PersonMask::default();

        'outer: for _ in 0..self.parameters.max_resolve_attempts {
            for weekday in FirstWeekdays::iter() {
                let support = match self.try_place_weekday_role(
                    week,
                    sunday_lead,
                    &mut person_assigned_mask,
                    rng,
                ) {
                    Ok(support) => support,

                    Err(week) => {
                        last_failed_week = week;
                        continue 'outer;
                    }
                };

                // Prevent Friday supports from working Thursday leads
                if weekday == FirstWeekdays::Thu {
                    person_assigned_mask |= self
                        .state
                        .friday_mask
                        .get_mask_for_role(week, Role::Support);
                }

                let lead = match self.try_place_weekday_role(
                    week,
                    Some(support),
                    &mut person_assigned_mask,
                    rng,
                ) {
                    Ok(lead) => lead,

                    Err(week) => {
                        last_failed_week = week;
                        continue 'outer;
                    }
                };

                sunday_lead = Some(lead);
                self.state.assignment[weekday][week] = Slot::new(Some(lead), Some(support));
            }

            return Ok(());
        }

        Err(last_failed_week)
    }

    fn try_place_weekday_role<R: Rng + ?Sized>(
        &mut self,
        week: WeekIdx,
        except: Option<PersonIdx>,
        limit_mask: &mut PersonMask,
        rng: &mut R,
    ) -> Result<PersonIdx, WeekIdx> {
        let mut assignment_mask = self.context.holidays.mask(week);

        if let Some(role) = except {
            assignment_mask.set(role);
        }

        let person = self
            .sampler
            .sample(assignment_mask | *limit_mask, rng)
            .ok_or(week)?;

        if self.state.weekend_mask.get_person(week, person) {
            limit_mask.set(person);
        }

        Ok(person)
    }
}

struct PersonSampler {
    n_people: usize,
    weights: [u16; MAX_PEOPLE],
}

impl PersonSampler {
    fn new(people: &[ContextPerson; MAX_PEOPLE], n_people: usize) -> Self {
        let weights = array::from_fn(|i| {
            if i < n_people {
                people[i].rate.factor() as u16
            } else {
                0
            }
        });

        Self { n_people, weights }
    }

    fn sample<R: Rng + ?Sized>(&self, mask: PersonMask, rng: &mut R) -> Option<PersonIdx> {
        let total_weight: u32 = (0..self.n_people)
            .map(|i| unsafe { PersonIdx::new_unchecked(i as u8) })
            .filter_map(|p| (!mask.get(p)).then_some(self.weights[p] as u32))
            .sum();

        if total_weight == 0 {
            return None;
        }

        let mut cumulative = 0;
        let target = rng.random_range(0..total_weight);

        for i in 0..self.n_people {
            let p = unsafe { PersonIdx::new_unchecked(i as u8) };
            if !mask.get(p) {
                cumulative += self.weights[p] as u32;
                if cumulative > target {
                    return Some(p);
                }
            }
        }

        None
    }
}

use std::{
    array,
    ops::{Deref, DerefMut},
};

use chrono::Weekday;
use strum::IntoEnumIterator;

use crate::{
    defs::*,
    solver::types::{WeeklyMask, WeeklyRoleMask},
    types::{PersonIdx, Role, Slot, WeekIdx},
};

/* -- Context -- */

pub struct Context {
    pub holidays: WeeklyMask,
    pub n_people: usize,
    pub people: [ContextPerson; MAX_PEOPLE],
}

#[derive(Debug)]
pub struct ContextPerson {
    pub rate: Rate,
    pub work_share: f32,
}

impl Context {
    pub fn new(problem: &ProblemInput) -> Self {
        let n_people = problem.people.len();
        let holidays = Self::create_holidays(&problem.people, n_people);

        let total_factor = problem.people.iter().map(|p| p.rate.factor() as f32).sum();
        let people = array::from_fn(|i| {
            if i < n_people {
                ContextPerson::new(&problem.people[i], total_factor)
            } else {
                ContextPerson::dummy()
            }
        });

        Context {
            holidays,
            n_people,
            people,
        }
    }

    fn create_holidays(people: &[Person], n_people: usize) -> WeeklyMask {
        let mut holidays = WeeklyMask::default();

        for person in (0..n_people).map(|i| unsafe { PersonIdx::new_unchecked(i as u8) }) {
            for holiday in &people[person.get() as usize].holidays {
                if let Some(week) = WeekIdx::try_new(*holiday) {
                    holidays.set(week, person);
                }
            }
        }

        holidays
    }
}

impl ContextPerson {
    pub fn new(person: &Person, total_factor: f32) -> Self {
        Self {
            rate: person.rate,
            work_share: person.rate.factor() as f32 / total_factor,
        }
    }

    fn dummy() -> Self {
        Self {
            rate: Rate::try_new(100).unwrap(),
            work_share: 0.0,
        }
    }
}

/* -- Assignments -- */

pub struct PlacementContext<'a> {
    pub holidays: &'a WeeklyMask,
    pub mask: &'a WeeklyRoleMask,
    pub weekday: Weekday,
}

#[derive(Copy, Clone)]
pub struct SingleAssignment([Slot; N_WEEKS]);

impl SingleAssignment {
    pub fn try_swap(&mut self, i: WeekIdx, j: WeekIdx, context: &PlacementContext) -> bool {
        if !self.is_valid_swap(i, j, context) {
            return false;
        }

        (self.0[i], self.0[j]) = (self.0[j], self.0[i]);
        true
    }

    pub fn is_valid_swap(&self, i: WeekIdx, j: WeekIdx, context: &PlacementContext) -> bool {
        [(i, j), (j, i)]
            .iter()
            .all(|&(a, b)| Self::valid_placement(self.0[a], b, context))
    }

    /// Attempts to swap the support position of two slots.
    pub fn try_swap_support(&mut self, i: WeekIdx, j: WeekIdx, context: &PlacementContext) -> bool {
        if !self.is_valid_support_swap(i, j, context) {
            return false;
        }

        (self.0[i], self.0[j]) = self.0[i].mix_with(self.0[j]);
        true
    }

    pub fn is_valid_support_swap(
        &self,
        i: WeekIdx,
        j: WeekIdx,
        context: &PlacementContext,
    ) -> bool {
        let (a, b) = self.0[i].mix_with(self.0[j]);

        [(i, a), (j, b)]
            .iter()
            .all(|&(week, slot)| slot.is_valid() && Self::valid_placement(slot, week, context))
    }

    pub fn validate(&self, context: &PlacementContext) -> bool {
        WeekIdx::iter().all(|week| {
            let slot = self.0[week];
            slot.is_valid() && Self::valid_placement(slot, week, context)
        })
    }

    pub fn valid_at(&self, week: WeekIdx, context: &PlacementContext) -> bool {
        Self::valid_placement(self.0[week], week, context)
    }

    pub fn number_valid_swaps(&self, week: WeekIdx, context: &PlacementContext) -> usize {
        self.walk_swaps_for(week, context).count()
    }

    pub fn number_valid_support_swaps(&self, week: WeekIdx, context: &PlacementContext) -> usize {
        self.walk_valid_support_swaps_for(week, context).count()
    }

    pub fn walk_swaps_for(
        &self,
        week: WeekIdx,
        context: &PlacementContext,
    ) -> impl Iterator<Item = (WeekIdx, Slot)> {
        WeekIdx::iter().filter_map(move |other| {
            (week != other && self.is_valid_swap(week, other, context))
                .then_some((other, self.0[other]))
        })
    }

    pub fn walk_valid_support_swaps_for(
        &self,
        week: WeekIdx,
        context: &PlacementContext,
    ) -> impl Iterator<Item = (WeekIdx, Slot)> {
        WeekIdx::iter().filter_map(move |other| {
            (week != other && self.is_valid_support_swap(week, other, context))
                .then_some((other, self.0[other]))
        })
    }

    pub fn valid_placement(slot: Slot, week: WeekIdx, context: &PlacementContext) -> bool {
        for role in Role::iter() {
            if let Some(person) = slot.get(role)
                && (context.holidays.get(week, person) || context.mask.get_role(week, person, role))
            {
                return false;
            }
        }

        if let Some(next_week) = week.next() {
            match context.weekday {
                Weekday::Fri => {
                    if let Some(person) = slot.get(Role::Lead)
                        && context.holidays.get(next_week, person)
                    {
                        return false;
                    }
                }

                Weekday::Sat | Weekday::Sun => {
                    for role in Role::iter() {
                        if let Some(person) = slot.get(role)
                            && context.holidays.get(next_week, person)
                        {
                            return false;
                        }
                    }
                }

                _ => {}
            }
        }

        true
    }
}

impl Default for SingleAssignment {
    fn default() -> Self {
        Self([Slot::default(); N_WEEKS])
    }
}

impl Deref for SingleAssignment {
    type Target = [Slot; N_WEEKS];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SingleAssignment {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[derive(Clone)]
pub struct MultiAssignment<const N: usize>(pub [SingleAssignment; N]);

impl<const N: usize> Default for MultiAssignment<N> {
    fn default() -> Self {
        Self([SingleAssignment::default(); N])
    }
}

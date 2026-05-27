use std::iter;

use chrono::Weekday::{self, *};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{
    misc::BoxedArray,
    solver::{
        context::SingleAssignment,
        friday::FridayParameters,
        weekday::{FirstWeekdays, WeekdayAssignment, WeekdayParameters},
        weekend::WeekendParameters,
    },
    types::{ScheduleView, Slot, Solution, WeekIdx},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SolverParameters {
    pub weekend_parameters: WeekendParameters,
    pub friday_parameters: FridayParameters,
    pub weekday_parameters: WeekdayParameters,
    pub max_solutions: u32,
    pub skip_last_shifts: u8,
}

impl SolverParameters {
    pub const FAST: Self = Self {
        weekend_parameters: WeekendParameters {
            number_permutations: 50,
            max_resolve_attempts: 50,
        },

        friday_parameters: FridayParameters {
            number_permutations: 100,
            max_resolve_attempts: 500,
        },

        weekday_parameters: WeekdayParameters {
            number_permutations: 50,
            max_resolve_attempts: 25,
        },

        max_solutions: 10,
        skip_last_shifts: 3,
    };

    pub const SLOW: Self = Self {
        weekend_parameters: WeekendParameters {
            number_permutations: 100,
            max_resolve_attempts: 50,
        },

        friday_parameters: FridayParameters {
            number_permutations: 10_000,
            max_resolve_attempts: 100,
        },

        weekday_parameters: WeekdayParameters {
            number_permutations: 100,
            max_resolve_attempts: 10,
        },

        max_solutions: 20,
        skip_last_shifts: 3,
    };
}

/* -- DraftSchedule -- */

#[derive(Copy, Clone)]
pub struct DraftSchedule<'a> {
    pub fridays: &'a SingleAssignment,
    pub saturdays: &'a SingleAssignment,
    pub weekdays: &'a WeekdayAssignment,
}

impl<'a> From<&DraftSchedule<'a>> for Solution {
    fn from(value: &DraftSchedule) -> Self {
        Solution::from_boxed_array(BoxedArray::from_iter(value.iter_slots()))
    }
}

impl ScheduleView for DraftSchedule<'_> {
    fn iter_slots(&self) -> impl Iterator<Item = Slot> + '_ {
        WeekIdx::iter().flat_map(move |i| {
            FirstWeekdays::iter()
                .map(move |j| self.weekdays[j][i])
                .chain(iter::once(self.fridays[i]))
                .chain(iter::once(self.saturdays[i]))
                .chain(iter::once(self.saturdays[i].swapped()))
        })
    }

    fn iter_slots_weekday(&self, weekday: Weekday) -> impl Iterator<Item = Slot> + '_ {
        let slots = match weekday {
            Mon | Tue | Wed | Thu => {
                let weekday = FirstWeekdays::try_from(weekday).unwrap();
                &self.weekdays[weekday]
            }

            Fri => self.fridays,
            Sat => self.saturdays,
            Sun => self.saturdays,
        };

        slots.iter().copied()
    }
}

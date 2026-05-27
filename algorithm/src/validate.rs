use chrono::Weekday;
use itertools::Itertools;
use serde::Serialize;
use strum::IntoEnumIterator;

use crate::{
    solver::context::Context,
    types::{DayIdx, PersonIdx, Role, ScheduleView, WeekIdx},
};

#[derive(Copy, Clone, Debug, Serialize)]
pub enum Conflict {
    ConsecutiveDay(PersonIdx, DayIdx, DayIdx),
    Holiday(PersonIdx, DayIdx),
    Role(PersonIdx, DayIdx),
    WorkCount(PersonIdx, WeekIdx),
}

pub struct ScheduleValidator<'a, S: ScheduleView> {
    context: &'a Context,
    schedule: &'a S,
}

impl<'a, S: ScheduleView> ScheduleValidator<'a, S> {
    pub fn new(context: &'a Context, schedule: &'a S) -> Self {
        Self { context, schedule }
    }

    pub fn validate(&self) -> impl Iterator<Item = Conflict> {
        self.consecutive_day_conflicts()
            .chain(self.holiday_conflicts())
            .chain(self.role_conflicts())
            .chain(self.work_count_conflicts())
    }

    fn consecutive_day_conflicts(&self) -> impl Iterator<Item = Conflict> {
        (self.schedule.iter_slots())
            .zip(DayIdx::iter())
            .tuple_windows()
            .filter(|((_, a), _)| a.weekday() != Weekday::Sat) // exempt Sat→Sun per spec
            .flat_map(
                |((a, a_idx), (b, b_idx))| match (a.get(Role::Lead), b.get(Role::Support)) {
                    (Some(lead_a), Some(lead_b)) if lead_a == lead_b => {
                        Some(Conflict::ConsecutiveDay(lead_a, a_idx, b_idx))
                    }

                    _ => None,
                },
            )
    }

    fn holiday_conflicts(&self) -> impl Iterator<Item = Conflict> {
        WeekIdx::iter().flat_map(move |week| {
            PersonIdx::iter(self.context.n_people).flat_map(move |person| {
                let on_holiday = self.context.holidays.get(week, person);

                let week_iter = self.schedule.iter_week(week).flat_map(move |(day, slot)| {
                    (on_holiday && slot.has(person)).then_some(Conflict::Holiday(person, day))
                });

                let next_week_iter = self.next_week_holiday_conflicts(week, person);

                week_iter.chain(next_week_iter)
            })
        })
    }

    /// Holiday margin conflicts: if the person is on holiday next week,
    /// they cannot work Friday Lead or Saturday (either role) of this week.
    ///
    /// Relies on `iter_week()` always yielding 7 days Mon→Sun.
    fn next_week_holiday_conflicts(&self, week: WeekIdx, person: PersonIdx) -> Vec<Conflict> {
        let Some(next_week) = week.next() else {
            return vec![];
        };

        if !self.context.holidays.get(next_week, person) {
            return vec![];
        }

        let mut iter = self.schedule.iter_week(week).skip(4);

        // SAFETY: iter_week() always yields exactly 7 days
        let (friday_idx, friday) = iter.next().unwrap();
        let (saturday_idx, saturday) = iter.next().unwrap();

        let mut conflicts = Vec::with_capacity(2);

        if friday.get(Role::Lead) == Some(person) {
            conflicts.push(Conflict::Holiday(person, friday_idx));
        }

        if saturday.has(person) {
            conflicts.push(Conflict::Holiday(person, saturday_idx));
        }

        conflicts
    }

    fn role_conflicts(&self) -> impl Iterator<Item = Conflict> {
        self.schedule
            .iter_slots()
            .zip(DayIdx::iter())
            .flat_map(|(slot, day_idx)| {
                slot.get(Role::Lead)
                    .zip(slot.get(Role::Support))
                    .into_iter()
                    .flat_map(move |(lead, support)| {
                        (lead == support).then_some(Conflict::Role(lead, day_idx))
                    })
            })
    }

    fn work_count_conflicts(&self) -> impl Iterator<Item = Conflict> {
        WeekIdx::iter().flat_map(move |week| {
            PersonIdx::iter(self.context.n_people).flat_map(move |person| {
                let mut iter = self.schedule.iter_week(week);

                let weekday_count = iter
                    .by_ref()
                    .take(4)
                    .flat_map(|(_, slot)| {
                        Role::iter().flat_map(move |role| (slot.get(role)? == person).then_some(()))
                    })
                    .count();

                iter.next();
                let (_, saturday) = iter.next().unwrap();

                (weekday_count >= 2 && saturday.has(person))
                    .then_some(Conflict::WorkCount(person, week))
            })
        })
    }
}

use std::array;

use chrono::Weekday;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

use crate::{
    defs::*,
    types::{HourAssignments, PersonIdx, Role, ScheduleView, Slot, Weights},
};

pub struct ScheduleEvaluator<'a> {
    holiday_weeks: [[bool; N_WEEKS]; MAX_PEOPLE],
    hour_assignments: HourAssignments,
    n_people: usize,
    target_hours: [f32; MAX_PEOPLE],
    weights: &'a Weights,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleFitness {
    pub annual_hours: f32,
    pub consecutive_days: f32,
    pub consecutive_weekends: f32,
    pub weekend_alternation: f32,
    pub weekend_regularity: f32,
    pub weekly_hours: f32,
    pub blank_weeks: f32,
}

impl ScheduleEvaluator<'_> {
    pub fn weights(&self) -> &Weights {
        self.weights
    }

    pub fn new<'a>(problem: &ProblemInput, weights: &'a Weights) -> ScheduleEvaluator<'a> {
        let n_people = problem.people.len();
        let holiday_weeks = array::from_fn(|person_idx| {
            let mut weeks = [false; N_WEEKS];

            if person_idx < n_people {
                for &holiday_week in &problem.people[person_idx].holidays {
                    if (holiday_week as usize) < N_WEEKS {
                        weeks[holiday_week as usize] = true;
                    }
                }
            }

            weeks
        });

        ScheduleEvaluator {
            hour_assignments: HourAssignments::new(
                problem.start_date,
                &problem.overrides,
                &problem.weekday_hours,
                problem.skip_last_shifts,
            ),

            target_hours: array::from_fn(|i| {
                if i < n_people {
                    problem.people[i].rate.weekly_hours()
                } else {
                    0.0
                }
            }),
            holiday_weeks,
            weights,
            n_people,
        }
    }

    /// Returns the absolute annual drift per person.
    pub fn calculate_annual_deltas<S: ScheduleView>(&self, schedule: &S) -> [f32; MAX_PEOPLE] {
        let mut annual_delta = [0.; MAX_PEOPLE];

        for (week_idx, chunk) in schedule
            .iter_slots()
            .zip(self.hour_assignments.iter())
            .chunks(N_WEEKDAYS)
            .into_iter()
            .enumerate()
        {
            let mut hours_this_week = [0.; MAX_PEOPLE];

            for (slot, role_hours) in chunk {
                for (role, hours) in Role::iter().zip(role_hours) {
                    if let Some(person) = slot.get(role) {
                        hours_this_week[person] += hours;
                    }
                }
            }

            for person in PersonIdx::iter(self.n_people) {
                if !self.holiday_weeks[person][week_idx] {
                    annual_delta[person] += hours_this_week[person] - self.target_hours[person];
                }
            }
        }

        annual_delta
    }

    pub fn evaluate<S: ScheduleView>(&self, schedule: &S) -> ScheduleFitness {
        let consecutive_days = Self::consecutive_days(schedule);
        let consecutive_weekends = Self::consecutive_weekends(schedule);
        let (weekly, annual, blank_weeks) = self.calculate_hour_metrics(schedule);
        let (spacing, alternation) = self.weekend_metrics(schedule);

        ScheduleFitness {
            consecutive_days: self.weights.consecutive_days * consecutive_days as f32,
            consecutive_weekends: self.weights.consecutive_weekends * consecutive_weekends as f32,
            annual_hours: self.weights.annual_hours * annual,
            weekend_alternation: self.weights.weekend_alternation * alternation,
            weekend_regularity: self.weights.weekend_regularity * spacing,
            weekly_hours: self.weights.weekly_hours * weekly,
            blank_weeks: self.weights.blank_weeks * blank_weeks as f32,
        }
    }

    /// Returns (weekly_fitness, annual_fitness, blank_weeks_count)
    fn calculate_hour_metrics<S: ScheduleView>(&self, schedule: &S) -> (f32, f32, usize) {
        let mut sse = [0.; MAX_PEOPLE];
        let mut annual_delta = [0.; MAX_PEOPLE];
        let mut blank_weeks = 0;

        for (week_idx, chunk) in schedule
            .iter_slots()
            .zip(self.hour_assignments.iter())
            .chunks(N_WEEKDAYS)
            .into_iter()
            .enumerate()
        {
            let mut hours_this_week = [0.; MAX_PEOPLE];

            for (slot, role_hours) in chunk {
                for (role, hours) in Role::iter().zip(role_hours) {
                    if let Some(person) = slot.get(role) {
                        hours_this_week[person] += hours;
                    }
                }
            }

            for person in PersonIdx::iter(self.n_people) {
                if hours_this_week[person] == 0.
                    && !self.holiday_weeks[person][week_idx]
                    && self.target_hours[person] > 0.
                {
                    blank_weeks += 1;
                }

                let mut delta = hours_this_week[person] - self.target_hours[person];

                if self.holiday_weeks[person][week_idx] {
                    delta = 0.;
                }

                sse[person] += delta * delta;
                annual_delta[person] += delta;
            }
        }

        // RMSE across all people and weeks
        let weekly = sse
            .map(|x| (1. / N_WEEKS as f32 * x).sqrt())
            .into_iter()
            .sum();

        let annual = annual_delta.iter().map(|d| d.abs()).sum::<f32>();
        (weekly, annual, blank_weeks)
    }

    fn consecutive_days<S: ScheduleView>(schedule: &S) -> usize {
        schedule
            .iter_slots()
            .tuple_windows()
            .filter(|(a, b)| a.overlaps(*b))
            .count()
    }

    fn consecutive_weekends<S: ScheduleView>(schedule: &S) -> usize {
        schedule
            .iter_slots_weekday(Weekday::Sat)
            .tuple_windows()
            .filter(|&(a, b)| a.overlaps(b))
            .count()
    }

    fn weekend_metrics<S: ScheduleView>(&self, schedule: &S) -> (f32, f32) {
        Self::evaluate_weekend_metrics(schedule.iter_slots_weekday(Weekday::Sat), self.n_people)
    }

    /// Evaluates both spacing and alternation in one optimal, zero-allocation pass.
    /// Returns (spacing_penalty, alternation_penalty)
    pub fn evaluate_weekend_metrics(
        saturdays: impl Iterator<Item = Slot>,
        n_people: usize,
    ) -> (f32, f32) {
        let mut sum_sq_gaps = [0.0; MAX_PEOPLE];
        let mut counts = [0_u32; MAX_PEOPLE];
        let mut first_weekend = [None; MAX_PEOPLE]; // Stores (week_idx, role)
        let mut last_weekend = [None; MAX_PEOPLE]; // Stores (week_idx, role)

        let mut alternation_penalty = 0.0;

        for (week_idx, slot) in saturdays.enumerate() {
            for role in Role::iter() {
                if let Some(person) = slot.get(role) {
                    counts[person] += 1;

                    if let Some((last_idx, last_role)) = last_weekend[person] {
                        let gap = (week_idx - last_idx) as f32;
                        sum_sq_gaps[person] += gap * gap;

                        if last_role == role {
                            alternation_penalty += 1.0;
                        }
                    } else {
                        // First time seeing this person
                        first_weekend[person] = Some((week_idx, role));
                    }

                    last_weekend[person] = Some((week_idx, role));
                }
            }
        }

        let mut spacing_penalty = 0.0;

        for person in PersonIdx::iter(n_people) {
            if counts[person] < 2 {
                continue;
            }

            // Apply wrap-around gap from the end of the year back to the start
            if let (Some((first_idx, first_role)), Some((last_idx, last_role))) =
                (first_weekend[person], last_weekend[person])
            {
                let gap = ((first_idx + N_WEEKS) - last_idx) as f32;
                sum_sq_gaps[person] += gap * gap;

                if first_role == last_role {
                    alternation_penalty += 1.0;
                }

                // Mathematical variance simplification: sum(gap^2) - (N^2 / count)
                let expected_sum_sq = (N_WEEKS as f32 * N_WEEKS as f32) / (counts[person] as f32);

                // .max(0.0) protects against microscopic floating-point rounding negative errors
                spacing_penalty += (sum_sq_gaps[person] - expected_sum_sq).max(0.0);
            }
        }

        (spacing_penalty, alternation_penalty)
    }
}

impl ScheduleFitness {
    pub fn total(&self) -> f32 {
        self.annual_hours
            + self.consecutive_days
            + self.consecutive_weekends
            + self.weekend_alternation
            + self.weekend_regularity
            + self.weekly_hours
            + self.blank_weeks
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Slot, Solution, WeekIdx};
    use crate::utils::sample_people;
    use chrono::NaiveDate;

    fn make_saturday(person: PersonIdx, role: Role) -> Slot {
        Slot::default().with(role, person)
    }

    #[test]
    fn weekend_metrics_all_unassigned() {
        let slots = [Slot::default(); N_WEEKS];
        let (spacing, alternation) =
            ScheduleEvaluator::evaluate_weekend_metrics(slots.iter().copied(), MAX_PEOPLE);

        assert_eq!(spacing, 0.0);
        assert_eq!(alternation, 0.0);
    }

    #[test]
    fn weekend_metrics_perfectly_spaced_alternating() {
        // Person 0 works weeks 0, 12, 24, 36 (4 weekends, gap=12 each)
        // Roles: Lead, Support, Lead, Support → no adjacent role repeats
        let person = PersonIdx::new(0).unwrap();
        let mut slots = [Slot::default(); N_WEEKS];

        slots[0] = make_saturday(person, Role::Lead);
        slots[12] = make_saturday(person, Role::Support);
        slots[24] = make_saturday(person, Role::Lead);
        slots[36] = make_saturday(person, Role::Support);

        let (spacing, alternation) =
            ScheduleEvaluator::evaluate_weekend_metrics(slots.iter().copied(), MAX_PEOPLE);

        // Gaps: 12, 12, 12, wrap: 0+48-36=12. sum_sq = 4*144 = 576. expected = 48²/4 = 576.
        assert!((spacing - 0.0).abs() < 0.001);
        // L→S, S→L, L→S, wrap S→L → no same-role transitions
        assert_eq!(alternation, 0.0);
    }

    #[test]
    fn weekend_metrics_perfectly_spaced_same_role() {
        // Person 0 works weeks 0, 12, 24, 36, all as Lead
        let person = PersonIdx::new(0).unwrap();
        let mut slots = [Slot::default(); N_WEEKS];

        for &week in &[0, 12, 24, 36] {
            slots[week] = make_saturday(person, Role::Lead);
        }

        let (spacing, alternation) =
            ScheduleEvaluator::evaluate_weekend_metrics(slots.iter().copied(), MAX_PEOPLE);

        // Same gaps → spacing = 0
        assert!((spacing - 0.0).abs() < 0.001);
        // 3 internal same-role transitions + 1 wrap-around = 4
        assert_eq!(alternation, 4.0);
    }

    #[test]
    fn weekend_metrics_clustered() {
        // Person 0 works weeks 0, 1, 2, 3 (first 4 weeks, then nothing)
        let person = PersonIdx::new(0).unwrap();
        let mut slots = [Slot::default(); N_WEEKS];

        for week in 0..4 {
            slots[week] = make_saturday(person, Role::Lead);
        }

        let (spacing, alternation) =
            ScheduleEvaluator::evaluate_weekend_metrics(slots.iter().copied(), MAX_PEOPLE);

        // Gaps: 1, 1, 1, wrap: 0+48-3=45. sum_sq = 3*1 + 2025 = 2028.
        // expected = 48²/4 = 576. spacing = 2028 - 576 = 1452.
        assert!((spacing - 1452.0).abs() < 0.1);
        // All same role → 3 internal + 1 wrap = 4
        assert_eq!(alternation, 4.0);
    }

    #[test]
    fn weekend_metrics_two_people_alternating() {
        // Person 0: weeks 0,1 (Lead, Lead) - same role adjacent
        // Person 1: weeks 0,1 (Support, Support) - same role adjacent
        // Person 0 has 2 weekends, Person 1 has 2 weekends
        let p0 = PersonIdx::new(0).unwrap();
        let p1 = PersonIdx::new(1).unwrap();
        let mut slots = [Slot::default(); N_WEEKS];

        slots[0] = Slot::new(Some(p0), Some(p1)); // p0=Lead, p1=Support
        slots[1] = Slot::new(Some(p0), Some(p1)); // p0=Lead, p1=Support

        let (spacing, alternation) =
            ScheduleEvaluator::evaluate_weekend_metrics(slots.iter().copied(), MAX_PEOPLE);

        // Each person: gap=1 (internal) + wrap=47. sum_sq = 1 + 2209 = 2210.
        // expected = 48²/2 = 1152. spacing per person = 2210 - 1152 = 1058.
        // Total spacing = 2 * 1058 = 2116.
        assert!((spacing - 2116.0).abs() < 0.1);
        // Each person: 1 internal same-role transition + 1 wrap-around = 2. Total = 4.
        assert_eq!(alternation, 4.0);
    }

    #[test]
    fn test_consecutive_days() {
        let p0 = PersonIdx::new(0).unwrap();
        let p1 = PersonIdx::new(1).unwrap();

        // Day 0: p0=Lead, p1=Support; Day 1: p0=Support → Lead→Support overlap
        let mut slots = [Slot::default(); N_DAYS];
        slots[0] = Slot::new(Some(p0), Some(p1));
        slots[1] = Slot::new(Some(p1), Some(p0)); // p0 was Lead day 0, now Support

        let solution = Solution::from_boxed_array(Box::new(slots));
        assert_eq!(ScheduleEvaluator::consecutive_days(&solution), 1);

        // No overlap: completely distinct people
        let mut slots = [Slot::default(); N_DAYS];
        let p2 = PersonIdx::new(2).unwrap();
        let p3 = PersonIdx::new(3).unwrap();
        slots[0] = Slot::new(Some(p0), Some(p1));
        slots[1] = Slot::new(Some(p2), Some(p3));

        let solution = Solution::from_boxed_array(Box::new(slots));
        assert_eq!(ScheduleEvaluator::consecutive_days(&solution), 0);
    }

    #[test]
    fn test_consecutive_weekends() {
        let p0 = PersonIdx::new(0).unwrap();

        // Same person on consecutive Saturdays (all 48 weeks)
        let mut slots = [Slot::default(); N_DAYS];
        for week in WeekIdx::iter() {
            slots[week.weekday(Weekday::Sat)] = make_saturday(p0, Role::Lead);
        }

        let solution = Solution::from_boxed_array(Box::new(slots));
        // 47 consecutive weekend windows
        assert_eq!(
            ScheduleEvaluator::consecutive_weekends(&solution),
            N_WEEKS - 1
        );
    }

    #[test]
    fn test_annual_hours_calculation() {
        let start_date = NaiveDate::from_ymd_opt(2024, 8, 19).unwrap();
        let weekday_hours = [
            [13.5, 7.5], // Mon
            [14.5, 5.5], // Tue
            [14.5, 7.5], // Wed
            [14.5, 7.5], // Thu
            [14.5, 7.5], // Fri
            [20.0, 7.5], // Sat
            [14.5, 5.0], // Sun
        ];
        let problem = ProblemInput::try_new(
            start_date,
            sample_people().to_vec(),
            ProblemOverrides::default(),
            weekday_hours,
            3,
        )
        .unwrap();
        let evaluator = ScheduleEvaluator::new(&problem, &Weights::STANDARD);

        let slots = [Slot::default(); N_DAYS];
        let solution = Solution::from_boxed_array(Box::new(slots));

        let deltas = evaluator.calculate_annual_deltas(&solution);

        for person in PersonIdx::iter(problem.people.len()) {
            // Target hours are positive, so 0 actual hours results in negative drift
            assert!(deltas[person] < 0.0);
        }
    }
}

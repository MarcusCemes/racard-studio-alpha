use std::array;

use chrono::Weekday;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use strum::{EnumCount, IntoEnumIterator};

use crate::{
    defs::*,
    fitness::{ScheduleEvaluator, ScheduleFitness},
    misc::BoxedArray,
    types::{HourAssignments, PersonIdx, Role, Slot, SlotArrayRef, WeekIdx, Weights},
};

/// Complete statistics for a schedule
#[derive(Debug, Serialize)]
pub struct ScheduleStatistics {
    pub weekly_breakdown: WeeklyBreakdown,
    pub weekly_heatmap: WeeklyHeatmap,
    pub final_statistics: FinalStatistics,
    pub fitness: ScheduleFitness,
}

/// Per-week hours breakdown by person and role
#[serde_as]
#[derive(Debug, Serialize)]
pub struct WeeklyBreakdown {
    /// Hours worked per person, per role, per week
    /// Indexed as: [person][role][week]
    #[serde_as(as = "Box<[[[_; N_WEEKS]; Role::COUNT]; MAX_PEOPLE]>")]
    pub hours_by_role: Box<[[[f32; N_WEEKS]; Role::COUNT]; MAX_PEOPLE]>,

    /// Cumulative hours per person through each week.
    /// Indexed as: [person][week]
    #[serde_as(as = "Box<[[_; N_WEEKS]; MAX_PEOPLE]>")]
    pub cumulative_hours: Box<[[f32; N_WEEKS]; MAX_PEOPLE]>,
}

impl Default for WeeklyBreakdown {
    fn default() -> Self {
        Self {
            hours_by_role: BoxedArray::from_zeroed(),
            cumulative_hours: BoxedArray::from_zeroed(),
        }
    }
}

/// Heatmap showing work intensity
#[serde_as]
#[derive(Debug, Serialize)]
pub struct WeeklyHeatmap {
    /// Number of working days per person per week
    /// Indexed as: [person][week]
    #[serde_as(as = "Box<[[_; N_WEEKS]; MAX_PEOPLE]>")]
    pub slots_per_week: Box<[[u8; N_WEEKS]; MAX_PEOPLE]>,
}

impl Default for WeeklyHeatmap {
    fn default() -> Self {
        Self {
            slots_per_week: BoxedArray::from_zeroed(),
        }
    }
}

/// Overall summary statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct FinalStatistics {
    /// Total hours worked by each person
    pub total_hours_worked: [f32; MAX_PEOPLE],

    /// Expected hours for each person (rate * FULL_TIME * N_WEEKS)
    pub expected_hours: [f32; MAX_PEOPLE],

    /// Total hours available across all slots in the schedule
    pub total_available_hours: f32,

    /// Theoretical total based on sum of all rates
    pub theoretical_hours: f32,

    /// Number of fridays worked as lead
    pub lead_fridays: [u8; MAX_PEOPLE],

    /// Number of fridays worked as support
    pub support_fridays: [u8; MAX_PEOPLE],

    /// Number of long weekends worked (Saturday lead)
    pub long_weekends: [u8; MAX_PEOPLE],

    /// Number of short weekends worked (Saturday support)
    pub short_weekends: [u8; MAX_PEOPLE],
}

impl ScheduleStatistics {
    /// Compute all statistics for a completed schedule
    pub fn compute(problem: &ProblemInput, schedule: &[Slot; N_DAYS], weights: &Weights) -> Self {
        let skip_last_shifts = problem.skip_last_shifts;

        let hour_assignments = HourAssignments::new(
            problem.start_date,
            &problem.overrides,
            &problem.weekday_hours,
            skip_last_shifts,
        );

        let weekly_breakdown = Self::compute_weekly_breakdown(problem, schedule, &hour_assignments);

        let weekly_heatmap =
            Self::compute_weekly_heatmap(schedule, skip_last_shifts, problem.people.len());

        let final_statistics = Self::compute_final_statistics(
            problem,
            schedule,
            &hour_assignments,
            &weekly_breakdown,
            skip_last_shifts,
            problem.people.len(),
        );

        let evaluator = ScheduleEvaluator::new(problem, weights);
        let fitness = evaluator.evaluate(&SlotArrayRef(schedule));

        ScheduleStatistics {
            weekly_breakdown,
            weekly_heatmap,
            final_statistics,
            fitness,
        }
    }

    /// Returns the indices of phantom shifts (day, role) that should be
    /// excluded from statistics. Phantom shifts are the last `skip_last_shifts`
    /// shifts, iterating backwards from the end of the schedule with Support checked
    /// before Lead per day (matching `HourAssignments` zeroing order).
    fn phantom_shifts(skip_last_shifts: u8) -> Vec<(usize, Role)> {
        let mut remaining = skip_last_shifts as usize;
        let mut phantoms = Vec::new();

        for day_idx in (0..N_DAYS).rev() {
            if remaining > 0 {
                phantoms.push((day_idx, Role::Support));
                remaining -= 1;
            }

            if remaining > 0 {
                phantoms.push((day_idx, Role::Lead));
                remaining -= 1;
            } else {
                break;
            }
        }

        phantoms
    }

    /// Check if a specific (day, role) is a phantom shift.
    fn is_phantom(day_idx: usize, role: Role, phantom_shifts: &[(usize, Role)]) -> bool {
        phantom_shifts
            .iter()
            .any(|(d, r)| *d == day_idx && *r == role)
    }

    fn compute_weekly_breakdown(
        problem: &ProblemInput,
        schedule: &[Slot; N_DAYS],
        hour_assignments: &[[f32; Role::COUNT]; N_DAYS],
    ) -> WeeklyBreakdown {
        let n_people = problem.people.len();
        let mut weekly_breakdown = WeeklyBreakdown::default();

        // Build holiday weeks lookup
        let holiday_weeks: [[bool; N_WEEKS]; MAX_PEOPLE] = array::from_fn(|person_idx| {
            let mut weeks = [false; N_WEEKS];

            if person_idx < problem.people.len() {
                for &holiday_week in &problem.people[person_idx].holidays {
                    if (holiday_week as usize) < N_WEEKS {
                        weeks[holiday_week as usize] = true;
                    }
                }
            }

            weeks
        });

        // Track cumulative totals per person
        let mut running_totals = [0.; MAX_PEOPLE];

        for week in WeekIdx::iter() {
            let week_idx = week.get() as usize;

            // Iterate through all days in this week
            for day_offset in 0..N_WEEKDAYS {
                let day_idx = week.get() as usize * N_WEEKDAYS + day_offset;
                let slot = schedule[day_idx];

                // Add hours for each role
                for role in Role::iter() {
                    if let Some(person) = slot.get(role) {
                        let hours = hour_assignments[day_idx][role];

                        weekly_breakdown.hours_by_role[person][role][week] += hours;
                        running_totals[person] += hours;
                    }
                }
            }

            // Add holiday credit and store cumulative totals for this week
            for person in PersonIdx::iter(n_people) {
                if holiday_weeks[person][week_idx] {
                    // Credit holiday: add their weekly hours
                    let idx = person.get() as usize;

                    if idx < problem.people.len() {
                        running_totals[person] += problem.people[idx].rate.weekly_hours();
                    }
                }
                weekly_breakdown.cumulative_hours[person][week] = running_totals[person];
            }
        }

        weekly_breakdown
    }

    fn compute_weekly_heatmap(
        schedule: &[Slot; N_DAYS],
        skip_last_shifts: u8,
        n_people: usize,
    ) -> WeeklyHeatmap {
        let mut weekly_heatmap = WeeklyHeatmap::default();
        let phantom_shifts = Self::phantom_shifts(skip_last_shifts);

        for week in WeekIdx::iter() {
            // Count working days per person for this week (excluding phantom shifts)
            let mut day_counts = [0_u8; MAX_PEOPLE];

            for day_offset in 0..N_WEEKDAYS {
                let day_idx = week.get() as usize * N_WEEKDAYS + day_offset;
                let slot = schedule[day_idx];

                // Check if each person is working this day, excluding phantom shifts
                for role in Role::iter() {
                    if Self::is_phantom(day_idx, role, &phantom_shifts) {
                        continue;
                    }
                    if let Some(person) = slot.get(role) {
                        day_counts[person] += 1;
                    }
                }
            }

            // Store the counts for this week
            for person in PersonIdx::iter(n_people) {
                weekly_heatmap.slots_per_week[person][week] = day_counts[person];
            }
        }

        weekly_heatmap
    }

    fn compute_final_statistics(
        problem: &ProblemInput,
        schedule: &[Slot; N_DAYS],
        hour_assignments: &HourAssignments,
        weekly_breakdown: &WeeklyBreakdown,
        skip_last_shifts: u8,
        n_people: usize,
    ) -> FinalStatistics {
        // Calculate total hours worked by each person (from cumulative at last week)
        let mut total_hours_worked = [0.; MAX_PEOPLE];

        // The last week's cumulative is the total
        for person in PersonIdx::iter(n_people) {
            total_hours_worked[person] = *weekly_breakdown.cumulative_hours[person].last().unwrap();
        }

        let expected_hours = array::from_fn(|i| {
            if i < problem.people.len() {
                problem.people[i].rate.weekly_hours() * N_WEEKS as f32
            } else {
                0.0
            }
        });

        // Compute fridays and weekends from schedule (excluding phantom shifts)
        let mut lead_fridays = [0u8; MAX_PEOPLE];
        let mut support_fridays = [0u8; MAX_PEOPLE];
        let phantom_shifts = Self::phantom_shifts(skip_last_shifts);

        let mut long_weekends = [0u8; MAX_PEOPLE];
        let mut short_weekends = [0u8; MAX_PEOPLE];

        for week in WeekIdx::iter() {
            let mut counted_weekend = [false; MAX_PEOPLE];

            for weekday in [Weekday::Fri, Weekday::Sat] {
                let day_offset = weekday.num_days_from_monday() as usize;
                let day_idx = week.get() as usize * N_WEEKDAYS + day_offset;
                let slot = schedule[day_idx];

                if weekday == Weekday::Fri {
                    if !Self::is_phantom(day_idx, Role::Lead, &phantom_shifts)
                        && let Some(lead) = slot.get(Role::Lead)
                    {
                        lead_fridays[lead] += 1;
                    }

                    if !Self::is_phantom(day_idx, Role::Support, &phantom_shifts)
                        && let Some(support) = slot.get(Role::Support)
                    {
                        support_fridays[support] += 1;
                    }
                }

                if weekday == Weekday::Sat {
                    if !Self::is_phantom(day_idx, Role::Lead, &phantom_shifts)
                        && let Some(lead) = slot.get(Role::Lead)
                        && !counted_weekend[lead]
                    {
                        long_weekends[lead] += 1;
                        counted_weekend[lead] = true;
                    }

                    if !Self::is_phantom(day_idx, Role::Support, &phantom_shifts)
                        && let Some(support) = slot.get(Role::Support)
                        && !counted_weekend[support]
                    {
                        short_weekends[support] += 1;
                        counted_weekend[support] = true;
                    }
                }
            }
        }

        FinalStatistics {
            expected_hours,
            theoretical_hours: expected_hours.iter().sum(),
            total_available_hours: hour_assignments.total(),
            total_hours_worked,
            lead_fridays,
            support_fridays,
            long_weekends,
            short_weekends,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::sample_people;
    use chrono::NaiveDate;

    #[test]
    fn test_compute_does_not_panic() {
        let start_date = NaiveDate::from_ymd_opt(2024, 8, 19).unwrap();
        let weekday_hours = [[14.5, 7.5]; N_WEEKDAYS];

        let problem = ProblemInput::try_new(
            start_date,
            sample_people().to_vec(),
            ProblemOverrides::default(),
            weekday_hours,
            3,
        )
        .unwrap();

        let slots = [Slot::default(); N_DAYS];
        let stats = ScheduleStatistics::compute(&problem, &slots, &Weights::STANDARD);

        // Smoke test: just verify compute() doesn't panic.
        // Total hours worked includes holiday credit from sample_people()'s holidays.
        assert!(stats.final_statistics.total_available_hours > 0.0);

        // Every array should be sized correctly
        assert_eq!(stats.final_statistics.total_hours_worked.len(), MAX_PEOPLE);

        // Expected hours should match problem people
        let expected: f32 = stats.final_statistics.expected_hours.iter().sum();
        assert!(expected > 0.0);
    }
}

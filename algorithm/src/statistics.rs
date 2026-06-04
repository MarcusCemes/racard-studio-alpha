use chrono::Weekday;
use serde::{Deserialize, Serialize};
use strum::{EnumCount, IntoEnumIterator};

use crate::{
    defs::*,
    fitness::{ScheduleEvaluator, ScheduleFitness},
    types::{HourAssignments, Role, Slot, SlotArrayRef, WeekIdx, Weights},
};

/// Complete statistics for a schedule
#[derive(Debug, Serialize)]
pub struct ScheduleStatistics {
    /// Daily hours for both roles (N_ROLES * N_DAYS)
    pub hours: HourAssignments,
    /// Statistics for each person in the problem (in order)
    pub people: Vec<PersonStatistics>,
    /// Global metrics
    pub summary: GlobalStatistics,
    /// Detailed fitness breakdown
    pub fitness: ScheduleFitness,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PersonStatistics {
    pub weeks: Vec<WeeklyPersonStats>,
    pub totals: FinalPersonStats,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct WeeklyPersonStats {
    /// Hours worked in each role this week
    pub hours_by_role: [f32; Role::COUNT],
    /// Total hours worked up to and including this week (includes holiday credit)
    pub cumulative_hours: f32,
    /// Number of times each role was worked this week
    pub role_counts: [u8; Role::COUNT],
    /// Number of working slots this week (excluding phantom shifts)
    pub slots_count: u8,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct FinalPersonStats {
    pub total_hours_worked: f32,
    pub expected_hours: f32,
    pub lead_fridays: u8,
    pub support_fridays: u8,
    pub long_weekends: u8,  // Saturday Lead
    pub short_weekends: u8, // Saturday Support
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalStatistics {
    pub total_available_hours: f32,
    pub theoretical_hours: f32,
}

impl ScheduleStatistics {
    /// Compute all statistics for a completed schedule
    pub fn compute(problem: &ProblemInput, schedule: &[Slot; N_DAYS], weights: &Weights) -> Self {
        let hour_assignments = HourAssignments::new(
            problem.start_date,
            &problem.overrides,
            &problem.weekday_hours,
            problem.skip_last_shifts,
        );

        let n_people = problem.people.len();
        let phantom_shifts = Self::phantom_shifts(problem.skip_last_shifts);

        // Pre-calculate holiday weeks for each person
        let holiday_weeks: Vec<[bool; N_WEEKS]> = problem
            .people
            .iter()
            .map(|p| {
                let mut weeks = [false; N_WEEKS];
                for &hw in &p.holidays {
                    if (hw as usize) < N_WEEKS {
                        weeks[hw as usize] = true;
                    }
                }
                weeks
            })
            .collect();

        let mut people_stats: Vec<PersonStatistics> = problem
            .people
            .iter()
            .map(|p| PersonStatistics {
                weeks: Vec::with_capacity(N_WEEKS),
                totals: FinalPersonStats {
                    expected_hours: p.rate.weekly_hours() * N_WEEKS as f32,
                    ..Default::default()
                },
            })
            .collect();

        let mut running_totals = vec![0.0f32; n_people];
        let fri_offset = Weekday::Fri.num_days_from_monday() as usize;
        let sat_offset = Weekday::Sat.num_days_from_monday() as usize;

        for week in WeekIdx::iter() {
            let week_idx = week.get() as usize;
            let mut weekly_hours = vec![[0.0f32; Role::COUNT]; n_people];
            let mut weekly_role_counts = vec![[0u8; Role::COUNT]; n_people];
            let mut weekly_slots = vec![0u8; n_people];

            for day_offset in 0..N_WEEKDAYS {
                let day_idx = week_idx * N_WEEKDAYS + day_offset;
                let slot = schedule[day_idx];

                for role in Role::iter() {
                    if let Some(person_idx) = slot.get(role) {
                        let p_idx = person_idx.get() as usize;
                        if p_idx < n_people {
                            let hours = hour_assignments[day_idx][role];
                            weekly_hours[p_idx][role] += hours;
                            running_totals[p_idx] += hours;

                            if !Self::is_phantom(day_idx, role, &phantom_shifts) {
                                weekly_role_counts[p_idx][role] += 1;
                                weekly_slots[p_idx] += 1;

                                if day_offset == fri_offset {
                                    match role {
                                        Role::Lead => people_stats[p_idx].totals.lead_fridays += 1,
                                        Role::Support => {
                                            people_stats[p_idx].totals.support_fridays += 1
                                        }
                                    }
                                } else if day_offset == sat_offset {
                                    match role {
                                        Role::Lead => people_stats[p_idx].totals.long_weekends += 1,
                                        Role::Support => {
                                            people_stats[p_idx].totals.short_weekends += 1
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Apply holiday credits and store weekly stats
            for p_idx in 0..n_people {
                if holiday_weeks[p_idx][week_idx] {
                    running_totals[p_idx] += problem.people[p_idx].rate.weekly_hours();
                }

                people_stats[p_idx].weeks.push(WeeklyPersonStats {
                    hours_by_role: weekly_hours[p_idx],
                    cumulative_hours: running_totals[p_idx],
                    role_counts: weekly_role_counts[p_idx],
                    slots_count: weekly_slots[p_idx],
                });
            }
        }

        // Finalize totals
        for p_idx in 0..n_people {
            people_stats[p_idx].totals.total_hours_worked = running_totals[p_idx];
        }

        let summary = GlobalStatistics {
            total_available_hours: hour_assignments.total(),
            theoretical_hours: people_stats.iter().map(|p| p.totals.expected_hours).sum(),
        };

        let evaluator = ScheduleEvaluator::new(problem, weights);
        let fitness = evaluator.evaluate(&SlotArrayRef(schedule));

        ScheduleStatistics {
            hours: hour_assignments,
            people: people_stats,
            summary,
            fitness,
        }
    }

    /// Returns the indices of phantom shifts (day, role) that should be
    /// excluded from statistics.
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PersonIdx;
    use crate::utils::sample_people;
    use chrono::NaiveDate;

    #[test]
    fn test_compute_does_not_panic() {
        let start_date = NaiveDate::from_ymd_opt(2024, 8, 19).unwrap();
        let weekday_hours = [[14.5, 7.5]; N_WEEKDAYS];

        let people = sample_people();
        let problem = ProblemInput::try_new(
            start_date,
            people.to_vec(),
            ProblemOverrides::default(),
            weekday_hours,
            3,
        )
        .unwrap();

        // Assign one Lead and one Support to person 0 in week 0 (Mon, Tue)
        let p0 = PersonIdx::new(0).unwrap();
        let mut slots = [Slot::default(); N_DAYS];
        slots[0] = slots[0].with(Role::Lead, p0);
        slots[1] = slots[1].with(Role::Support, p0);

        let stats = ScheduleStatistics::compute(&problem, &slots, &Weights::STANDARD);

        // Smoke test: just verify compute() doesn't panic.
        assert!(stats.summary.total_available_hours > 0.0);

        // Every person should have stats
        assert_eq!(stats.people.len(), problem.people.len());

        // Expected hours should match problem people
        assert!(stats.summary.theoretical_hours > 0.0);

        assert_eq!(stats.hours[0][Role::Lead], 14.5);

        // role_counts: person 0 should have 1 Lead + 1 Support in week 0
        let w0 = &stats.people[0].weeks[0];
        assert_eq!(w0.role_counts[Role::Lead], 1);
        assert_eq!(w0.role_counts[Role::Support], 1);
        assert_eq!(w0.slots_count, 2);
    }
}

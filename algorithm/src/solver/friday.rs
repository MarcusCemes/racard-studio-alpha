use chrono::Weekday;
use rand::{Rng, RngExt, seq::SliceRandom};

use crate::{
    defs::N_WEEKS,
    solver::{
        context::{Context, PlacementContext, SingleAssignment},
        defs::WeekdayParameters,
        types::WeeklyRoleMask,
    },
    types::{PersonIdx, Slot, WeekIdx},
    utils::AtomicProgress,
};

pub struct FridaySolver<'a> {
    context: &'a Context,
    parameters: &'a WeekdayParameters,
    state: FridayState,
}

#[derive(Default)]
struct FridayState {
    assignment: SingleAssignment,
    cursor: u64,
    weekend_mask: WeeklyRoleMask,
}

impl FridaySolver<'_> {
    pub fn new<'a>(parameters: &'a WeekdayParameters, context: &'a Context) -> FridaySolver<'a> {
        FridaySolver {
            context,
            parameters,
            state: FridayState::default(),
        }
    }

    pub fn prime(&mut self, saturdays: &SingleAssignment) {
        self.state.cursor = 0;
        self.state.weekend_mask = WeeklyRoleMask::from_assignment(saturdays, self.context.n_people);
    }

    pub fn generate<'a, R: Rng + ?Sized>(
        &'a mut self,
        progress: &AtomicProgress,
        rng: &mut R,
    ) -> Option<&'a SingleAssignment> {
        while self.state.cursor < self.parameters.number_permutations {
            self.state.cursor += 1;
            self.fill_new_slots(rng);

            if self.resolve_conflicts(rng) {
                progress.increment_accepted();
                return Some(&self.state.assignment);
            }

            progress.increment_rejected();
        }

        None
    }

    fn fill_new_slots<R: Rng + ?Sized>(&mut self, rng: &mut R) {
        let &Context { n_people, .. } = self.context;

        let people = &mut PersonIdx::array()[..n_people];

        people.shuffle(rng);

        for (slot, (lead, support)) in self.state.assignment.iter_mut().zip(
            (1..n_people as i8)
                .flat_map(|offset| {
                    people.iter().map(move |&lead| {
                        let support_idx =
                            (lead.get() as i8 + offset).rem_euclid(n_people as i8) as u8;

                        let support = unsafe { PersonIdx::new_unchecked(support_idx) };

                        (lead, support)
                    })
                })
                .cycle()
                .take(N_WEEKS),
        ) {
            *slot = Slot::new(Some(lead), Some(support));
        }
    }

    fn resolve_conflicts<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let context = PlacementContext {
            holidays: &self.context.holidays,
            mask: &self.state.weekend_mask,
            weekday: Weekday::Fri,
        };

        for week in WeekIdx::iter() {
            if self.state.assignment.valid_at(week, &context) {
                continue;
            }

            let valid_swaps = self.state.assignment.number_valid_swaps(week, &context);

            if valid_swaps == 0 {
                return false;
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

        true
    }
}

#![allow(dead_code)]

use rand::{Rng, seq::SliceRandom};

use crate::types::{PersonIdx, Slot};

/// Infinite random slot generator, cycled every `n_people` slots to ensure
/// an even distribution of staff and double roles. Guarantees that no staff
/// member is assigned to the same double role.
#[derive(Default)]
pub struct ShuffledSlotStream {
    index: usize,
    n_people: usize,
    staff: Vec<PersonIdx>,
    double: Vec<PersonIdx>,
}

impl ShuffledSlotStream {
    pub fn new(n_people: usize) -> Self {
        Self {
            index: 0,
            n_people,
            staff: (0..n_people as u8)
                .map(|i| unsafe { PersonIdx::new_unchecked(i) })
                .collect(),
            double: (0..n_people as u8)
                .map(|i| unsafe { PersonIdx::new_unchecked(i) })
                .collect(),
        }
    }

    pub fn next<R: Rng + ?Sized>(&mut self, rng: &mut R) -> Slot {
        debug_assert_ne!(self.n_people, 0);

        if self.index == 0 {
            self.staff.shuffle(rng);
            self.double.shuffle(rng);

            // Ensure no person has the same role in the same slot
            while self.any_duplicate_slots() {
                self.double.shuffle(rng);
            }
        }

        let staff = self.staff[self.index];
        let double = self.double[self.index];

        self.index = (self.index + 1) % self.n_people;
        Slot::new(Some(staff), Some(double))
    }

    fn any_duplicate_slots(&self) -> bool {
        self.staff
            .iter()
            .zip(self.double.iter())
            .any(|(&s, &d)| s == d)
    }
}

#[cfg(test)]
mod tests {
    use std::iter;

    use itertools::Itertools;
    use rand::SeedableRng;

    use crate::{defs::AppRng, types::Role};

    use super::*;

    #[test]
    fn test_shuffled_slot_stream() {
        const TEST_SIZE: usize = 64;
        let n_people = 8;

        let mut rng = AppRng::seed_from_u64(42);
        let mut slot_gen = ShuffledSlotStream::new(n_people);
        let mut slots = Vec::with_capacity(TEST_SIZE);

        for _ in 0..TEST_SIZE {
            slots.clear();
            slots.extend(iter::from_fn(|| Some(slot_gen.next(&mut rng))).take(n_people));

            assert!(slots.iter().copied().all(Slot::is_valid));
            assert!(slots.iter().map(|s| s.get(Role::Lead)).all_unique());
            assert!(slots.iter().map(|s| s.get(Role::Support)).all_unique());
        }
    }
}

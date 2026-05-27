#![allow(dead_code)]

use itertools::Itertools;

use crate::{
    defs::N_DAYS,
    solver::defs::DraftSchedule,
    types::{ScheduleView, Slot},
};

pub struct SolutionStorage {
    index: Vec<(f32, u32)>,
    slots: Box<[[Slot; N_DAYS]]>,
}

impl SolutionStorage {
    pub fn with_capacity(capacity: usize) -> Self {
        assert!(capacity > 0);

        Self {
            index: Vec::with_capacity(capacity),
            slots: unsafe { Box::new_uninit_slice(capacity).assume_init() },
        }
    }

    pub fn add(&mut self, fitness: f32, schedule: DraftSchedule) -> bool {
        self.insert(fitness, schedule.iter_slots())
    }

    /// Merge another SolutionStorage into this one, keeping the best solutions
    pub fn merge(&mut self, other: &SolutionStorage) {
        for &(fitness, slot_idx) in other.index.iter() {
            let slots = other.slots[slot_idx as usize].iter().copied();

            if !self.insert(fitness, slots) {
                break;
            }
        }
    }

    /// Insert a solution with the given fitness and slot iterator
    fn insert(&mut self, fitness: f32, slots: impl Iterator<Item = Slot>) -> bool {
        let capacity = self.index.capacity();
        let length = self.index.len();

        // Find insertion point (keeping sorted order)
        let Some(index) = (self.index.iter().enumerate().rev())
            .take_while(|&(_, (old_fitness, _))| *old_fitness > fitness)
            .map(|(i, _)| i)
            .last()
            .or((length < capacity).then_some(length))
        else {
            return false;
        };

        // Get slot storage index (reuse evicted slot or use new one)
        let slot_index = (length == capacity)
            .then(|| self.index.pop().map(|(_, idx)| idx as usize))
            .flatten()
            .unwrap_or(length);

        self.index.insert(index, (fitness, slot_index as u32));

        for (a, b) in slots.zip_eq(self.slots[slot_index].iter_mut()) {
            *b = a;
        }

        true
    }

    pub fn read(&self) -> impl Iterator<Item = (f32, &[Slot; N_DAYS])> + '_ {
        self.index
            .iter()
            .map(move |&(fitness, idx)| (fitness, &self.slots[idx as usize]))
    }
}

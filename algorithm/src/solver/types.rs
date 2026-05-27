/* -- Masks -- */

use std::{
    array,
    ops::{BitOr, BitOrAssign, Deref, DerefMut, Index, IndexMut},
};

use serde::Serialize;
use strum::{EnumCount, EnumIter, IntoEnumIterator};

use crate::{
    defs::N_WEEKS,
    solver::context::SingleAssignment,
    types::{PersonIdx, Role, WeekIdx},
    utils::AtomicProgress,
};

#[derive(Copy, Clone, Default)]
pub struct PersonMask(pub u16);

impl PersonMask {
    pub fn get(&self, person: PersonIdx) -> bool {
        self.0 & (1 << person.get()) != 0
    }

    pub fn set(&mut self, person: PersonIdx) {
        self.0 |= 1 << person.get();
    }
}

impl BitOr for PersonMask {
    type Output = PersonMask;

    fn bitor(self, rhs: Self) -> Self::Output {
        PersonMask(self.0 | rhs.0)
    }
}

impl BitOrAssign<PersonMask> for PersonMask {
    fn bitor_assign(&mut self, rhs: PersonMask) {
        self.0 |= rhs.0;
    }
}

#[derive(Clone)]
pub struct WeeklyMask([u16; N_WEEKS]);

impl WeeklyMask {
    pub fn get(&self, week: WeekIdx, person: PersonIdx) -> bool {
        self[week] & (1 << person.get()) != 0
    }

    pub fn set(&mut self, week: WeekIdx, person: PersonIdx) {
        self[week] |= 1 << person.get();
    }

    pub fn mask(&self, week: WeekIdx) -> PersonMask {
        PersonMask(self.0[week])
    }
}

impl Deref for WeeklyMask {
    type Target = [u16; N_WEEKS];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for WeeklyMask {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl BitOr for &WeeklyMask {
    type Output = WeeklyMask;

    fn bitor(self, rhs: Self) -> Self::Output {
        WeeklyMask(array::from_fn(|week| self[week] | rhs[week]))
    }
}

impl Default for WeeklyMask {
    fn default() -> Self {
        Self([0; N_WEEKS])
    }
}

#[derive(Default, Clone)]
pub struct WeeklyRoleMask([WeeklyMask; Role::COUNT]);

impl WeeklyRoleMask {
    pub fn get_person(&self, week: WeekIdx, person: PersonIdx) -> bool {
        Role::iter().any(|role| self.get_role(week, person, role))
    }

    pub fn get_role(&self, week: WeekIdx, person: PersonIdx, role: Role) -> bool {
        self.0[role].get(week, person)
    }

    pub fn get_mask_for_role(&self, week: WeekIdx, role: Role) -> PersonMask {
        PersonMask(self.0[role].0[week])
    }

    pub fn set(&mut self, week: WeekIdx, role: Role, person: PersonIdx) {
        self.0[role].set(week, person);
    }

    pub fn from_role_assignment(role_assignment: &SingleAssignment) -> Self {
        let mut mask = Self::default();

        for week in WeekIdx::iter() {
            let slot = role_assignment[week];

            for role in Role::iter() {
                if let Some(person) = slot.get(role) {
                    mask.set(week, role, person);
                }
            }
        }

        mask
    }

    pub fn from_assignment(assignment: &SingleAssignment, n_people: usize) -> Self {
        let mut mask = Self::default();

        for week in WeekIdx::iter() {
            let slot = assignment[week];

            for person in PersonIdx::iter(n_people) {
                if slot.has(person) {
                    for role in Role::iter() {
                        mask.set(week, role, person);
                    }
                }
            }
        }

        mask
    }

    pub fn from_holidays(holidays: &WeeklyMask, n_people: usize) -> Self {
        let mut mask = Self::default();

        for week in WeekIdx::iter() {
            for person in PersonIdx::iter(n_people) {
                if holidays.get(week, person) {
                    for role in Role::iter() {
                        mask.set(week, role, person);
                    }
                }
            }
        }

        mask
    }

    pub fn flood_left(&mut self) {
        for mask in &mut self.0 {
            for i in 1..mask.len() {
                mask.0[i - 1] |= mask.0[i];
            }
        }
    }

    /// Degree of freedom: how many (person, role) assignments remain available
    /// for this week. Lower = more constrained = assign first.
    ///
    /// Each bit in the per-role u16 is 1 if that person is *unavailable*:
    ///   - Bits 0..n_people-1: can be 0 (available) or 1 (holiday / exhausted count)
    ///   - Bits n_people..14: always 1 (dummy people have count=0 → pick_role sets bit)
    ///   - Bit 15: forced to 1 via `| 0x8000` (no corresponding person)
    ///
    /// count_zeros() therefore counts only actually-available real people.
    pub fn dof(&self, week: WeekIdx) -> u32 {
        Role::iter()
            .map(|role| (self.0[role].0[week] | 0x8000).count_zeros())
            .sum()
    }
}

impl BitOr<&WeeklyMask> for &WeeklyRoleMask {
    type Output = WeeklyRoleMask;

    fn bitor(self, rhs: &WeeklyMask) -> Self::Output {
        WeeklyRoleMask(array::from_fn(|role| &self.0[role] | rhs))
    }
}

impl BitOr for &WeeklyRoleMask {
    type Output = WeeklyRoleMask;

    fn bitor(self, rhs: Self) -> Self::Output {
        WeeklyRoleMask(array::from_fn(|role| &self.0[role] | &rhs.0[role]))
    }
}

/* -- SolverProgress -- */

#[derive(Clone, Debug, Default, Serialize)]
pub struct SolverProgress([AtomicProgress; SolverStage::COUNT]);

#[derive(Copy, Clone, Debug, EnumCount, EnumIter)]
pub enum SolverStage {
    Weekend,
    Friday,
    Weekday,
}

impl Index<SolverStage> for SolverProgress {
    type Output = AtomicProgress;

    fn index(&self, index: SolverStage) -> &Self::Output {
        unsafe { self.0.get_unchecked(index as usize) }
    }
}

impl IndexMut<SolverStage> for SolverProgress {
    fn index_mut(&mut self, index: SolverStage) -> &mut Self::Output {
        unsafe { self.0.get_unchecked_mut(index as usize) }
    }
}

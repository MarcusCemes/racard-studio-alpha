use std::{
    array,
    fmt::{self, Debug},
    iter,
    ops::{Deref, DerefMut, Index, IndexMut, Sub},
};

use chrono::{Datelike, NaiveDate, Weekday};
use nonmax::{NonMaxU8, NonMaxU16};
use rand::RngExt;
use rand_distr::{Distribution, StandardUniform};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use strum::{EnumCount, EnumIter, IntoEnumIterator};
use thiserror::Error;

use crate::{
    defs::{MAX_PEOPLE, N_DAYS, N_WEEKDAYS, N_WEEKS, NULL_ID, ProblemOverrides},
    misc::BoxedArray,
};

/* === Solution === */

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Solution(Box<SlotArray>);

#[serde_as]
#[derive(Clone, Debug, Serialize, Deserialize)]
#[repr(transparent)]
pub struct SlotArray(#[serde_as(as = "[_; N_DAYS]")] [Slot; N_DAYS]);

impl Solution {
    pub fn from_boxed_array(boxed_array: Box<[Slot; N_DAYS]>) -> Self {
        let ptr = Box::into_raw(boxed_array) as *mut SlotArray;

        // SAFETY: SlotArray is #[repr(transparent)], so its layout is identical
        // to the array it contains. We consume the old box and produce a new one.
        Self(unsafe { Box::from_raw(ptr) })
    }
}

impl SlotArray {
    pub const fn new(slots: [Slot; N_DAYS]) -> Self {
        SlotArray(slots)
    }
}

impl Deref for Solution {
    type Target = SlotArray;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Solution {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Deref for SlotArray {
    type Target = [Slot; N_DAYS];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for SlotArray {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// A simple wrapper around `[Slot; N_DAYS]` that implements `ScheduleView`.
pub struct SlotArrayRef<'a>(pub &'a [Slot; N_DAYS]);

impl ScheduleView for SlotArrayRef<'_> {
    fn iter_slots(&self) -> impl Iterator<Item = Slot> + '_ {
        self.0.iter().copied()
    }

    fn iter_slots_weekday(&self, weekday: Weekday) -> impl Iterator<Item = Slot> + '_ {
        let offset = weekday as usize * N_WEEKDAYS;
        self.0.iter().skip(offset).step_by(N_WEEKDAYS).copied()
    }
}

/* -- Indexing -- */

impl Index<WeekIdx> for [Slot; N_DAYS] {
    type Output = [Slot; N_WEEKDAYS];

    fn index(&self, index: WeekIdx) -> &Self::Output {
        let offset = index.get() as usize * N_WEEKDAYS;

        // SAFETY: The array length is a multiple of N_WEEKDAYS
        unsafe {
            let ptr = self.as_ptr().add(offset);
            &*(ptr as *const [Slot; N_WEEKDAYS])
        }
    }
}

impl IndexMut<WeekIdx> for [Slot; N_DAYS] {
    fn index_mut(&mut self, index: WeekIdx) -> &mut Self::Output {
        let offset = index.get() as usize * N_WEEKDAYS;

        unsafe {
            let ptr = self.as_mut_ptr().add(offset);
            &mut *(ptr as *mut [Slot; N_WEEKDAYS])
        }
    }
}

impl<T> Index<WeekdayIdx> for [T; N_WEEKDAYS] {
    type Output = T;

    fn index(&self, index: WeekdayIdx) -> &Self::Output {
        // SAFETY: WeekdayIdx is guaranteed to be in range 0..N_WEEKDAYS,
        // and `num_days_from_monday` returns a value in that range.
        unsafe { self.get_unchecked(index.0.num_days_from_monday() as usize) }
    }
}

impl<T> Index<DayIdx> for [T; N_DAYS] {
    type Output = T;

    fn index(&self, index: DayIdx) -> &Self::Output {
        // SAFETY: DayIdx is guaranteed to be in range 0..N_DAYS
        unsafe { self.get_unchecked(index.get() as usize) }
    }
}

impl<T> IndexMut<DayIdx> for [T; N_DAYS] {
    fn index_mut(&mut self, index: DayIdx) -> &mut Self::Output {
        // SAFETY: DayIdx is guaranteed to be in range 0..N_DAYS
        unsafe { self.get_unchecked_mut(index.get() as usize) }
    }
}

impl<T> Index<WeekIdx> for [T; N_WEEKS] {
    type Output = T;

    fn index(&self, index: WeekIdx) -> &Self::Output {
        // SAFETY: WeekIdx is guaranteed to be in range 0..N_WEEKS
        unsafe { self.get_unchecked(index.get() as usize) }
    }
}

impl<T> IndexMut<WeekIdx> for [T; N_WEEKS] {
    fn index_mut(&mut self, index: WeekIdx) -> &mut Self::Output {
        // SAFETY: WeekIdx is guaranteed to be in range 0..N_WEEKS
        unsafe { self.get_unchecked_mut(index.get() as usize) }
    }
}

/* -- DayIdx -- */

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct DayIdx(NonMaxU16);

impl DayIdx {
    pub fn try_new(value: u16) -> Option<Self> {
        (value < N_DAYS as u16).then_some(unsafe { DayIdx::new_unchecked(value) })
    }

    /// # Safety
    /// `value` must be less than `N_DAYS`.
    pub unsafe fn new_unchecked(value: u16) -> Self {
        DayIdx(unsafe { NonMaxU16::new_unchecked(value) })
    }

    pub fn get(self) -> u16 {
        self.0.get()
    }

    pub fn checked_add_signed(self, rhs: i16) -> Option<Self> {
        Self::try_new(self.0.get().checked_add_signed(rhs)?)
    }

    pub fn prev(self) -> Option<Self> {
        self.checked_add_signed(-1)
    }

    pub fn next(self) -> Option<Self> {
        self.checked_add_signed(1)
    }

    pub fn iter() -> impl ExactSizeIterator<Item = Self> {
        (0..N_DAYS as u16).map(|d| unsafe { Self::new_unchecked(d) })
    }

    pub fn iter_to_inc(self, bound: DayIdx) -> impl ExactSizeIterator<Item = Self> {
        let start = self.0.get();
        let end = bound.0.get();

        (start..=end).map(|i| unsafe { Self::new_unchecked(i) })
    }

    pub fn week(self) -> WeekIdx {
        unsafe { WeekIdx::new_unchecked((self.0.get() / N_WEEKDAYS as u16) as u8) }
    }

    pub fn weekday(self) -> Weekday {
        // SAFETY: value is always in range 0..6
        unsafe { Weekday::try_from((self.0.get() % N_WEEKDAYS as u16) as u8).unwrap_unchecked() }
    }
}

impl Sub<i16> for DayIdx {
    type Output = DayIdx;

    fn sub(self, rhs: i16) -> Self::Output {
        self.checked_add_signed(-rhs).expect("Overflow")
    }
}

impl Distribution<DayIdx> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> DayIdx {
        let idx = rng.random_range(0..N_DAYS as u16);
        unsafe { DayIdx::new_unchecked(idx) }
    }
}

/* -- WeekIdx -- */

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct WeekIdx(NonMaxU8);

impl WeekIdx {
    pub fn try_new(value: u8) -> Option<Self> {
        match NonMaxU8::new(value) {
            Some(v) if v.get() < N_WEEKS as u8 => Some(WeekIdx(v)),
            _ => None,
        }
    }

    /// # Safety
    /// `value` must be less than `N_WEEKS`.
    pub unsafe fn new_unchecked(value: u8) -> Self {
        WeekIdx(unsafe { NonMaxU8::new_unchecked(value) })
    }

    pub fn get(self) -> u8 {
        self.0.get()
    }

    pub fn array() -> [Self; N_WEEKS] {
        array::from_fn(|i| unsafe { Self::new_unchecked(i as u8) })
    }

    pub fn iter() -> impl ExactSizeIterator<Item = Self> {
        (0..N_WEEKS as u8).map(|w| unsafe { Self::new_unchecked(w) })
    }

    pub fn prev(self) -> Option<Self> {
        Some(unsafe { Self::new_unchecked(self.0.get().checked_sub(1)?) })
    }

    pub fn next(self) -> Option<Self> {
        Self::try_new(self.0.get() + 1)
    }

    pub fn weekday(self, weekday: Weekday) -> DayIdx {
        let day = self.0.get() as u16 * N_WEEKDAYS as u16 + weekday.num_days_from_monday() as u16;
        unsafe { DayIdx::new_unchecked(day) }
    }

    pub fn weekdays(self) -> [DayIdx; 4] {
        let monday = self.0.get() as u16 * N_WEEKDAYS as u16;
        [0, 1, 2, 3].map(|offset| unsafe { DayIdx::new_unchecked(monday + offset) })
    }
}

impl Distribution<WeekIdx> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> WeekIdx {
        let idx = rng.random_range(0..N_WEEKS as u8);
        unsafe { WeekIdx::new_unchecked(idx) }
    }
}

/* -- WeekdayIdx -- */

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct WeekdayIdx(pub Weekday);

/* -- PersonIdx -- */

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PersonIdx(NonMaxU8);

impl PersonIdx {
    pub const fn new(value: u8) -> Option<Self> {
        match value as usize {
            0..MAX_PEOPLE => unsafe { Some(Self::new_unchecked(value)) },
            _ => None,
        }
    }

    /// # Safety
    /// `value` must be less than `MAX_PEOPLE`.
    pub const unsafe fn new_unchecked(value: u8) -> Self {
        PersonIdx(unsafe { NonMaxU8::new_unchecked(value) })
    }

    pub fn array() -> [Self; MAX_PEOPLE] {
        array::from_fn(|i| unsafe { Self::new_unchecked(i as u8) })
    }

    pub const fn get(&self) -> u8 {
        self.0.get()
    }

    pub fn iter(n_people: usize) -> impl ExactSizeIterator<Item = Self> {
        debug_assert!(
            n_people <= MAX_PEOPLE,
            "n_people {n_people} exceeds MAX_PEOPLE"
        );
        (0..n_people as u8).map(|p| unsafe { Self::new_unchecked(p) })
    }
}

#[derive(Debug, Error)]
#[error("Person index {0} is out of range [0, {MAX_PEOPLE})")]
pub struct InvalidPerson(u8);

impl TryFrom<u8> for PersonIdx {
    type Error = InvalidPerson;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (value as usize) < MAX_PEOPLE {
            Ok(unsafe { PersonIdx::new_unchecked(value) })
        } else {
            Err(InvalidPerson(value))
        }
    }
}

impl<T> Index<PersonIdx> for [T; MAX_PEOPLE] {
    type Output = T;

    fn index(&self, index: PersonIdx) -> &Self::Output {
        unsafe { self.get_unchecked(index.0.get() as usize) }
    }
}

impl<T> IndexMut<PersonIdx> for [T; MAX_PEOPLE] {
    fn index_mut(&mut self, index: PersonIdx) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index.0.get() as usize) }
    }
}

/* -- ScheduleView -- */

impl ScheduleView for Solution {
    fn iter_slots(&self) -> impl Iterator<Item = Slot> + '_ {
        self.0.iter().copied()
    }

    fn iter_slots_weekday(&self, weekday: Weekday) -> impl Iterator<Item = Slot> + '_ {
        let offset = weekday.num_days_from_monday() as usize;
        self.iter_slots().skip(offset).step_by(N_WEEKDAYS)
    }

    fn iter_week(&self, week: WeekIdx) -> impl Iterator<Item = (DayIdx, Slot)> + '_ {
        let start = week.weekday(Weekday::Mon).get();

        (0..N_WEEKDAYS).map(move |i| {
            let day = unsafe { DayIdx::new_unchecked(start.wrapping_add(i as u16)) };
            (day, self.0[day])
        })
    }
}

/* === Slot === */

#[derive(Copy, Clone, Deserialize, Serialize)]
pub struct Slot(pub u8);

impl Slot {
    pub const NULL: Self = Self(NULL_ID << 4 | NULL_ID);

    pub fn new(lead: Option<PersonIdx>, support: Option<PersonIdx>) -> Self {
        let lead = (lead.map_or(NULL_ID, |l| l.get()) & 0xF) << 4;
        let support = support.map_or(NULL_ID, |s| s.get() & 0xF);
        Slot(lead | support)
    }

    pub fn get(self, field: Role) -> Option<PersonIdx> {
        let value = match field {
            Role::Lead => self.0 >> 4,
            Role::Support => self.0 & 0xF,
        };

        PersonIdx::new(value)
    }

    pub fn set(&mut self, field: Role, person: Option<PersonIdx>) {
        let id = person.map_or(NULL_ID, |w| w.get() & 0xF);

        match field {
            Role::Lead => self.0 = (self.0 & 0x0F) | (id << 4),
            Role::Support => self.0 = (self.0 & 0xF0) | id,
        }
    }

    pub fn replace(&mut self, field: Role, person: Option<PersonIdx>) -> Option<PersonIdx> {
        let old = self.get(field);
        self.set(field, person);
        old
    }

    pub fn with(mut self, field: Role, value: PersonIdx) -> Self {
        self.set(field, Some(value));
        self
    }

    pub fn has(self, person: PersonIdx) -> bool {
        let p = person.get();
        (self.0 >> 4) & 0xF == p || self.0 & 0xF == p
    }

    pub fn overlaps(self, other: Slot) -> bool {
        Role::iter().any(|role| self.get(role).map(|p| other.has(p)).unwrap_or_default())
    }

    pub fn is_assigned(self) -> bool {
        Role::iter().all(|role| self.get(role).is_some())
    }

    pub fn is_valid(self) -> bool {
        match (self.get(Role::Lead), self.get(Role::Support)) {
            (Some(lead), Some(support)) => lead != support,
            _ => false,
        }
    }

    pub const fn swapped(self) -> Self {
        Slot(self.0.rotate_right(4))
    }

    /// Mix this slot with another, returning two new slots with swapped roles.
    pub fn mix_with(self, slot: Slot) -> (Slot, Slot) {
        (
            Slot(self.0 & 0xF0 | slot.0 & 0x0F),
            Slot(self.0 & 0x0F | slot.0 & 0xF0),
        )
    }
}

impl Debug for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Slot")
            .field(&self.get(Role::Lead))
            .field(&self.get(Role::Support))
            .finish()
    }
}

impl Default for Slot {
    fn default() -> Self {
        Self::NULL
    }
}

/* -- Role -- */

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumCount, EnumIter)]
pub enum Role {
    // do not modify order
    Lead,
    Support,
}

impl<T> Index<Role> for [T; Role::COUNT] {
    type Output = T;

    fn index(&self, index: Role) -> &Self::Output {
        unsafe { self.get_unchecked(index as usize) }
    }
}

impl<T> IndexMut<Role> for [T; Role::COUNT] {
    fn index_mut(&mut self, index: Role) -> &mut Self::Output {
        unsafe { self.get_unchecked_mut(index as usize) }
    }
}

impl Distribution<Role> for StandardUniform {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> Role {
        match rng.random() {
            true => Role::Lead,
            false => Role::Support,
        }
    }
}

/* -- AssignableHours -- */

pub struct HourAssignments(Box<[[f32; Role::COUNT]; N_DAYS]>);

impl HourAssignments {
    pub fn new(
        start_date: NaiveDate,
        overrides: &ProblemOverrides,
        weekday_hours: &[[f32; Role::COUNT]; N_WEEKDAYS],
        skip_last_shifts: u8,
    ) -> Self {
        let mut weekday = start_date.weekday();

        let mut hours = BoxedArray::from_iter((0..N_DAYS).map(|_| {
            let day_hours = weekday_hours[WeekdayIdx(weekday)];
            weekday = weekday.succ();
            day_hours
        }));

        for (date, hours_val) in overrides.lead.as_ref() {
            let index = (*date - start_date).num_days();

            if (0..N_DAYS as i64).contains(&index) {
                hours[index as usize][Role::Lead] = *hours_val;
            }
        }

        for (date, hours_val) in overrides.support.as_ref() {
            let index = (*date - start_date).num_days();

            if (0..N_DAYS as i64).contains(&index) {
                hours[index as usize][Role::Support] = *hours_val;
            }
        }

        // Zero-out the last `skip_last_shifts` slot hours
        (hours.iter_mut().rev())
            .flat_map(|day_roles| day_roles.iter_mut().rev())
            .take(skip_last_shifts as usize)
            .for_each(|h| *h = 0.);

        Self(hours)
    }

    pub fn total(&self) -> f32 {
        self.iter().map(|day| day.iter().sum::<f32>()).sum()
    }
}

impl Deref for HourAssignments {
    type Target = [[f32; Role::COUNT]; N_DAYS];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl AsRef<[[f32; Role::COUNT]; N_DAYS]> for HourAssignments {
    fn as_ref(&self) -> &[[f32; Role::COUNT]; N_DAYS] {
        &self.0
    }
}

/* -- FitnessWeights -- */

#[derive(Copy, Clone, Debug, Deserialize, Serialize)]
pub struct Weights {
    pub annual_hours: f32,
    pub consecutive_days: f32,
    pub consecutive_weekends: f32,
    pub weekend_alternation: f32,
    pub weekend_regularity: f32,
    pub weekly_hours: f32,
    pub blank_weeks: f32,
}

impl Weights {
    pub const STANDARD: Self = Self {
        annual_hours: 5.,
        consecutive_days: 20.,
        consecutive_weekends: 10.,
        weekend_alternation: 1.,
        weekend_regularity: 1.,
        weekly_hours: 1.,
        blank_weeks: 50.,
    };
}

/* -- ScheduleView --- */

pub trait ScheduleView {
    fn iter_slots(&self) -> impl Iterator<Item = Slot> + '_;
    fn iter_slots_weekday(&self, weekday: Weekday) -> impl Iterator<Item = Slot> + '_;

    fn iter_week(&self, week: WeekIdx) -> impl Iterator<Item = (DayIdx, Slot)> + '_ {
        let start = week.weekday(Weekday::Mon);

        iter::from_fn(move || start.next())
            .take(N_WEEKDAYS)
            .zip(self.iter_slots().skip(start.get() as usize))
    }
}

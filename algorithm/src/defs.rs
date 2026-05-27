use std::fmt::Debug;

use chrono::{Datelike, NaiveDate, Weekday};
use serde::{Deserialize, Serialize};
use strum::EnumCount;
use thiserror::Error;

use crate::types::Role;

/* === Constants === */

pub const N_DAYS: usize = N_WEEKS * N_WEEKDAYS;
pub const MAX_PEOPLE: usize = 15;
pub const N_WEEKDAYS: usize = 7;
pub const N_WEEKS: usize = 48;

pub const NULL_ID: u8 = 0xF;

/* === Types === */

pub type AppRng = rand_pcg::Mcg128Xsl64;

/* -- Person -- */

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Person {
    pub name: String,
    pub holidays: Vec<u8>,
    pub rate: Rate,
}

/* -- ProblemInput -- */

#[derive(Clone, Debug, Deserialize)]
pub struct ProblemInput {
    pub overrides: ProblemOverrides,
    pub people: Vec<Person>,
    pub start_date: NaiveDate,
    pub skip_last_shifts: u8,
    pub weekday_hours: [[f32; Role::COUNT]; N_WEEKDAYS],
}

#[derive(Clone, Debug, Error, Serialize)]
pub enum ProblemInputError {
    #[error("Invalid start date")]
    InvalidStartDate,

    #[error("Too many people (max {MAX_PEOPLE})")]
    TooManyPeople,

    #[error("At least two people are required")]
    TooFewPeople,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ProblemOverrides {
    pub lead: Box<[(NaiveDate, f32)]>,
    pub support: Box<[(NaiveDate, f32)]>,
}

impl ProblemInput {
    pub fn try_new(
        start_date: NaiveDate,
        people: Vec<Person>,
        overrides: ProblemOverrides,
        weekday_hours: [[f32; Role::COUNT]; N_WEEKDAYS],
        skip_last_shifts: u8,
    ) -> Result<Self, ProblemInputError> {
        if people.len() < 2 {
            return Err(ProblemInputError::TooFewPeople);
        }

        if people.len() > MAX_PEOPLE {
            return Err(ProblemInputError::TooManyPeople);
        }

        if start_date.weekday() != Weekday::Mon {
            return Err(ProblemInputError::InvalidStartDate);
        }

        Ok(Self {
            start_date,
            people,
            overrides,
            skip_last_shifts,
            weekday_hours,
        })
    }
}

/* -- Rate -- */

#[derive(Copy, Clone, Debug, PartialEq, Eq)] // do not (de)serialize directly
pub struct Rate(u8);

#[derive(Debug, Error)]
pub enum RateError {
    #[error("Rate {0} is out of range [5, 100]")]
    OutOfRange(u8),

    #[error("Rate {0} is not divisible by 5")]
    DivisionError(u8),
}

impl Rate {
    pub const FULL_TIME: f32 = 33.;

    pub const fn try_new(value: u8) -> Result<Self, RateError> {
        match value {
            5..=100 if value.is_multiple_of(5) => Ok(Rate(value / 5)),
            v @ 0..=4 | v @ 101.. => Err(RateError::OutOfRange(v)),
            v => Err(RateError::DivisionError(v)),
        }
    }

    pub fn factor(&self) -> u8 {
        self.0
    }

    pub fn weekly_hours(&self) -> f32 {
        Self::FULL_TIME * (self.0 as f32 / 20.)
    }
}

impl<'de> Deserialize<'de> for Rate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = u8::deserialize(deserializer)?;
        Rate::try_new(value).map_err(serde::de::Error::custom)
    }
}

impl Serialize for Rate {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_u8(self.0 * 5)
    }
}

/* -- Holiday Helpers -- */

impl Person {
    pub fn is_on_holiday(&self, week: crate::types::WeekIdx) -> bool {
        self.holidays.contains(&week.get())
    }
}

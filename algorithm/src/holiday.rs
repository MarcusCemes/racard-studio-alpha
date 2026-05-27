use std::{iter, ops::RangeInclusive};

use chrono::{Datelike, Month, NaiveDate, TimeDelta};
use itertools::Itertools;
use serde::Serialize;
use strum::{EnumIter, IntoEnumIterator, IntoStaticStr};

use crate::defs::N_WEEKDAYS;

#[derive(Copy, Clone, Debug, PartialEq, Eq, EnumIter, IntoStaticStr, Serialize)]
pub enum Holiday {
    NearYear,
    EasterFriday,
    EasterMonday,
    AscensionThursday,
    WhitMonday,
    NationalDay,
    JeuneGenevois,
    Christmas,
    PublicRestoration,
}

// Returns an iterator for all bank holidays in the given date range.
pub fn geneva_bank_holidays(
    dates: RangeInclusive<NaiveDate>,
) -> impl Iterator<Item = (NaiveDate, Holiday)> {
    let start_date = *dates.start();
    let end_date = *dates.end();

    (year(&start_date)..=year(&end_date))
        .flat_map(year_bank_holidays)
        .skip_while(move |(date, _)| *date < start_date)
        .take_while(move |(date, _)| *date <= end_date)
}

/// Returns an iterator for all bank holidays in the year.
pub fn year_bank_holidays(year: i32) -> impl Iterator<Item = (NaiveDate, Holiday)> {
    iter::once(new_year(year))
        .chain(easter_holidays(year))
        .chain(iter::once(national_day(year)))
        .chain(iter::once(jeune_genevois(year)))
        .chain(iter::once(christmas(year)))
        .chain(iter::once(public_restoration(year)))
        .zip_eq(Holiday::iter())
}

/// Returns the date of the New Year (1st of January).
fn new_year(year: i32) -> NaiveDate {
    date(year, Month::January, 1)
}

/// Returns dates for Easter Friday, Easter Monday, Ascension Thursday and Whit Monday.
fn easter_holidays(year: i32) -> impl Iterator<Item = NaiveDate> {
    let easter = easter_date(year).unwrap();
    [-2, 1, 39, 50]
        .map(move |delta| easter + TimeDelta::try_days(delta).unwrap())
        .into_iter()
}

// Returns the date of the Swiss National Day (1st of August).
fn national_day(year: i32) -> NaiveDate {
    date(year, Month::August, 1)
}

/// Returns the date of the first Thursday after the first Sunday of September.
fn jeune_genevois(year: i32) -> NaiveDate {
    let month_start = date(year, Month::September, 1);
    let days_to_sunday = N_WEEKDAYS as u32 - month_start.weekday().number_from_monday();
    date(year, Month::September, days_to_sunday + 5)
}

/// Returns the date of Christmas (25th of December).
fn christmas(year: i32) -> NaiveDate {
    date(year, Month::December, 25)
}

/// Returns the date of the public restoration (31st of December).
fn public_restoration(year: i32) -> NaiveDate {
    date(year, Month::December, 31)
}

/* == Easter == */

/// Returns the date of Easter Sunday. Returns None if the year
/// is before 1582 (the Gregorian calendar reform).
fn easter_date(year: i32) -> Option<NaiveDate> {
    easter_num_days_from_ce(year).and_then(NaiveDate::from_num_days_from_ce_opt)
}

/// Returns the number of days from this year's Easter to January 1 from the year 1
/// in the proleptic Gregorian calendar using the computus algorithm.
fn easter_num_days_from_ce(y: i32) -> Option<i32> {
    if y < 1582 {
        return None;
    }

    // century
    let c = (y / 100) + 1;

    // shifted Epact
    let mut se = (14 + 11 * (y % 19) - 3 * c / 4 + (5 + 8 * c) / 25) % 30;

    // adjust Epact
    if (se == 0) || ((se == 1) && (10 < (y % 19))) {
        se += 1;
    }

    // Paschal Moon
    let p = NaiveDate::from_ymd_opt(y, 4, 19)?.num_days_from_ce() - se;

    // local the Sunday after the Paschal Moon
    Some(p + 7 - (p % 7))
}

/* == Utils == */

/// Helper function to get the date of a specific month and day.
fn date(year: i32, month: Month, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month.number_from_month(), day).unwrap()
}

fn year(date: &NaiveDate) -> i32 {
    date.year_ce().1 as i32
}

/* == Tests == */

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bank_holidays() {
        let date_range = d(2023, 6, 29)..=d(2024, 5, 30);
        let holidays: Vec<_> = geneva_bank_holidays(date_range).collect();

        assert_eq!(
            holidays,
            [
                (d(2023, 8, 1), Holiday::NationalDay),
                (d(2023, 9, 7), Holiday::JeuneGenevois),
                (d(2023, 12, 25), Holiday::Christmas),
                (d(2023, 12, 31), Holiday::PublicRestoration),
                (d(2024, 1, 1), Holiday::NearYear),
                (d(2024, 3, 29), Holiday::EasterFriday),
                (d(2024, 4, 1), Holiday::EasterMonday),
                (d(2024, 5, 9), Holiday::AscensionThursday),
                (d(2024, 5, 20), Holiday::WhitMonday)
            ]
        );
    }

    fn d(year: i32, month: u32, day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, day).unwrap()
    }
}

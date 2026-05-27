use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;
use serde_with::serde_as;

use crate::{
    defs::{N_WEEKS, Person, Rate},
    misc::BoxedArray,
    types::WeekIdx,
};

#[serde_as]
#[derive(Debug, Serialize)]
pub struct AtomicProgress {
    accepted: AtomicU64,
    rejected: AtomicU64,

    #[serde_as(as = "Box<[_; N_WEEKS]>")]
    week_failures: Box<[AtomicU64; N_WEEKS]>,
}

impl AtomicProgress {
    pub fn new() -> Self {
        Self::default()
    }
}

impl AtomicProgress {
    pub fn increment_accepted(&self) {
        self.accepted.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_rejected(&self) {
        self.rejected.fetch_add(1, Ordering::Relaxed);
    }

    pub fn increment_week_failure(&self, week: WeekIdx) {
        self.week_failures[week].fetch_add(1, Ordering::Relaxed);
    }

    pub fn accepted(&self) -> u64 {
        self.accepted.load(Ordering::Relaxed)
    }

    pub fn rejected(&self) -> u64 {
        self.rejected.load(Ordering::Relaxed)
    }
}

impl Default for AtomicProgress {
    fn default() -> Self {
        Self {
            accepted: AtomicU64::new(0),
            rejected: AtomicU64::new(0),
            week_failures: BoxedArray::from_default(),
        }
    }
}

impl Clone for AtomicProgress {
    fn clone(&self) -> Self {
        Self {
            accepted: AtomicU64::new(self.accepted.load(Ordering::Relaxed)),
            rejected: AtomicU64::new(self.rejected.load(Ordering::Relaxed)),

            week_failures: BoxedArray::from_iter(
                self.week_failures
                    .iter()
                    .map(|x| AtomicU64::new(x.load(Ordering::Relaxed))),
            ),
        }
    }
}

/// Default 8-person sample data for CLI demos and tests.
/// Not parameterized — callers that need variable counts should construct their own.
pub fn sample_people() -> [Person; 8] {
    [
        ("Alain", [10, 18, 25, 33, 43, 44].as_slice(), 75),
        ("Berthe", [0, 11, 15, 28, 29].as_slice(), 70),
        ("Clementine", [2, 10, 21, 36, 43, 44].as_slice(), 80),
        ("Dylan", [9, 18, 27, 36, 37].as_slice(), 80),
        ("Emery", [2, 9, 19, 28, 35].as_slice(), 60),
        ("Fabian", [6, 20, 32, 37, 38].as_slice(), 60),
        ("Giulia", [3, 19, 34, 35, 41].as_slice(), 55),
        ("Hagrid", [4, 20, 25, 34, 41].as_slice(), 40),
    ]
    .map(create_person)
}

fn create_person(data: (&str, &[u8], u8)) -> Person {
    Person {
        name: data.0.to_string(),
        holidays: data.1.to_vec(),
        rate: Rate::try_new(data.2).unwrap(),
    }
}

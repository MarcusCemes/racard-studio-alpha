use std::{
    sync::atomic::{AtomicU64, Ordering},
    thread,
};

use rand::{prelude::*, rngs::SysRng};
use serde::Serialize;
use thiserror::Error;

use crate::{
    defs::*,
    fitness::ScheduleEvaluator,
    solver::{
        context::Context,
        defs::{DraftSchedule, SolverParameters},
        friday::FridaySolver,
        types::{SolverProgress, SolverStage},
        weekday::WeekdaySolver,
        weekend::WeekendSolver,
    },
    tools::{
        controller::{ExecutionController, InterruptRequest},
        solution_storage::SolutionStorage,
    },
    types::{Slot, Solution, WeekIdx, Weights},
};

pub mod context;
pub mod defs;
pub mod friday;
pub mod types;
pub mod weekday;
pub mod weekend;

pub struct Solver<'a> {
    context: Context,
    pub evaluator: ScheduleEvaluator<'a>,
}

#[derive(Serialize)]
pub struct SolverSolution {
    pub fitness: f32,
    pub progress: SolverProgress,
    pub solution: Solution,
}

#[derive(Debug, Error)]
pub enum SolverError {
    #[error("{0}")]
    Interrupted(#[from] InterruptRequest),

    #[error("No solution found")]
    NoSolutionFound,
}

impl Solver<'_> {
    pub fn new<'a>(problem: &ProblemInput, weights: &'a Weights) -> Solver<'a> {
        Solver {
            context: Context::new(problem),
            evaluator: ScheduleEvaluator::new(problem, weights),
        }
    }

    pub fn execute(
        &mut self,
        parameters: SolverParameters,
        threads: Option<u16>,
        controller: &ExecutionController,
        progress: &SolverProgress,
    ) -> Result<SolverSolution, SolverError> {
        let mut best = self.execute_top_k(1, parameters, threads, controller, progress)?;
        best.pop().ok_or(SolverError::NoSolutionFound)
    }

    /// Run only the weekend solver, leaving Fridays and weekdays blank (Slot::NULL).
    ///
    /// Fitness is computed from weekend regularity and alternation only.
    /// This is useful for finding a very flat, regular weekend distribution
    /// without the constraints imposed by Friday/weekday placement.
    pub fn execute_weekends(
        &mut self,
        parameters: SolverParameters,
        threads: Option<u16>,
        controller: &ExecutionController,
        progress: &SolverProgress,
    ) -> Result<SolverSolution, SolverError> {
        let threads = threads.unwrap_or(num_cpus::get() as u16);
        let extra_threads = threads.saturating_sub(1) as usize;

        let root_counter = AtomicU64::new(parameters.weekend.number_permutations);

        let worker = Worker {
            top_k: 1,
            parameters: &parameters,
            progress,
            controller,
            counter: &root_counter,
            solver: self,
        };

        let mut storage = SolutionStorage::with_capacity(1);

        thread::scope(|scope| -> Result<(), SolverError> {
            let extra_workers = (0..extra_threads)
                .map(|_| scope.spawn(|| worker.spin_weekends_only()))
                .collect::<Vec<_>>();

            let local = worker.spin_weekends_only()?;
            storage.merge(&local);

            for handle in extra_workers {
                let result = handle.join().expect("thread panicked")?;
                storage.merge(&result);
            }

            Ok(())
        })?;

        storage
            .read()
            .next()
            .map(|(fitness, slots)| SolverSolution {
                fitness,
                progress: progress.clone(),
                solution: Solution::from_slot_array(slots),
            })
            .ok_or(SolverError::NoSolutionFound)
    }

    pub fn execute_top_k(
        &mut self,
        top_k: u32,
        parameters: SolverParameters,
        threads: Option<u16>,
        controller: &ExecutionController,
        progress: &SolverProgress,
    ) -> Result<Vec<SolverSolution>, SolverError> {
        let threads = threads.unwrap_or(num_cpus::get() as u16);
        let extra_threads = threads.saturating_sub(1) as usize;

        let root_counter = AtomicU64::new(parameters.weekend.number_permutations);

        let worker = Worker {
            top_k,
            parameters: &parameters,
            progress,
            controller,
            counter: &root_counter,
            solver: self,
        };

        let mut storage = SolutionStorage::with_capacity(top_k as usize);

        thread::scope(|scope| -> Result<(), SolverError> {
            let extra_workers = (0..extra_threads)
                .map(|_| scope.spawn(|| worker.spin()))
                .collect::<Vec<_>>();

            let local = worker.spin()?;
            storage.merge(&local);

            for handle in extra_workers {
                let result = handle.join().expect("thread panicked")?;
                storage.merge(&result);
            }

            Ok(())
        })?;

        let solutions: Vec<_> = storage
            .read()
            .map(|(fitness, slots)| SolverSolution {
                fitness,
                progress: progress.clone(),
                solution: Solution::from_slot_array(slots),
            })
            .collect();

        Ok(solutions)
    }
}

struct Worker<'a> {
    controller: &'a ExecutionController,
    counter: &'a AtomicU64,
    parameters: &'a SolverParameters,
    progress: &'a SolverProgress,
    solver: &'a Solver<'a>,
    top_k: u32,
}

impl Worker<'_> {
    fn spin(&self) -> Result<SolutionStorage, SolverError> {
        let rng = &mut AppRng::try_from_rng(&mut SysRng).unwrap();

        let mut friday_solver = FridaySolver::new(&self.parameters.friday, &self.solver.context);
        let mut weekend_solver = WeekendSolver::new(&self.parameters.weekend, &self.solver.context);
        let mut weekday_solver = WeekdaySolver::new(&self.parameters.weekday, &self.solver.context);
        let mut storage = SolutionStorage::with_capacity(self.top_k as usize);

        loop {
            let current = self.counter.load(Ordering::Relaxed);

            if current == 0 {
                break;
            }

            if self
                .counter
                .compare_exchange(current, current - 1, Ordering::Relaxed, Ordering::Relaxed)
                .is_err()
            {
                continue;
            }

            self.controller.assert()?;

            let Some(saturdays) = weekend_solver.generate(
                self.solver.evaluator.weights(),
                &self.progress[SolverStage::Weekend],
                rng,
            ) else {
                continue;
            };

            friday_solver.prime(saturdays);

            while let Some(fridays) =
                friday_solver.generate(&self.progress[SolverStage::Friday], rng)
            {
                self.controller.assert()?;
                weekday_solver.prime(fridays, saturdays);

                while let Some(weekdays) =
                    weekday_solver.generate(saturdays, &self.progress[SolverStage::Weekday], rng)
                {
                    self.controller.assert()?;

                    let draft = DraftSchedule {
                        fridays,
                        saturdays,
                        weekdays,
                    };

                    let fitness = self.solver.evaluator.evaluate(&draft).total();
                    storage.add_draft(fitness, draft);
                }
            }
        }

        Ok(storage)
    }

    fn spin_weekends_only(&self) -> Result<SolutionStorage, SolverError> {
        let rng = &mut AppRng::try_from_rng(&mut SysRng).unwrap();

        let mut weekend_solver = WeekendSolver::new(&self.parameters.weekend, &self.solver.context);
        let weights = self.solver.evaluator.weights();
        let n_people = self.solver.context.n_people;

        let mut storage = SolutionStorage::with_capacity(self.top_k as usize);

        loop {
            let current = self.counter.load(Ordering::Relaxed);

            if current == 0 {
                break;
            }

            if self
                .counter
                .compare_exchange(current, current - 1, Ordering::Relaxed, Ordering::Relaxed)
                .is_err()
            {
                continue;
            }

            self.controller.assert()?;

            let Some(saturdays) =
                weekend_solver.generate(weights, &self.progress[SolverStage::Weekend], rng)
            else {
                continue;
            };

            // Compute weekend-only fitness (regularity + alternation)
            let (spacing, alt) =
                ScheduleEvaluator::evaluate_weekend_metrics(saturdays.iter().copied(), n_people);

            let fitness = spacing * weights.weekend_regularity + alt * weights.weekend_alternation;

            // Build full slot array: weekends filled, everything else Slot::NULL
            let mut slots = [Slot::NULL; N_DAYS];

            for week in WeekIdx::iter() {
                let saturday_idx = (week.get() as usize) * N_WEEKDAYS + 5;
                let sunday_idx = saturday_idx + 1;

                // SAFETY: saturday_idx and sunday_idx are always in bounds (max week 47 → idx 335)
                slots[saturday_idx] = saturdays[week];
                slots[sunday_idx] = saturdays[week].swapped();
            }

            storage.add_slots(fitness, slots.iter().copied());
        }

        Ok(storage)
    }
}

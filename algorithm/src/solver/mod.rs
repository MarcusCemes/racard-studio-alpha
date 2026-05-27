use std::{
    iter,
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
    tools::controller::{ExecutionController, InterruptRequest},
    types::{Solution, Weights},
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
        let threads = threads.unwrap_or(num_cpus::get() as u16);
        let extra_threads = threads.saturating_sub(1) as usize;

        let root_counter = AtomicU64::new(parameters.weekend_parameters.number_permutations);

        let worker = Worker {
            parameters: &parameters,
            progress,
            controller,
            counter: &root_counter,
            solver: self,
        };

        let best_solution =
            thread::scope(|scope| -> Result<Option<(f32, Solution)>, SolverError> {
                // Spawn extra workers
                let extra_workers = (0..extra_threads)
                    .map(|_| scope.spawn(|| worker.spin()))
                    .collect::<Vec<_>>();

                // Use the current thread to spin one of the workers
                let local_solution = worker.spin();

                // Gather all solutions from the threads
                let mut all_solutions = iter::once(local_solution).chain(
                    extra_workers
                        .into_iter()
                        .map(|t| t.join().expect("thread panicked")),
                );

                // Find the best solution (smallest fitness)
                all_solutions.try_fold(None, |acc, solution| match solution {
                    Ok(solution) => {
                        let next = match acc {
                            None => solution,
                            Some((old_fitness, _)) if old_fitness > solution.0 => solution,
                            Some(acc) => acc,
                        };

                        Ok(Some(next))
                    }

                    Err(SolverError::NoSolutionFound) => Ok(acc),
                    Err(SolverError::Interrupted(e)) => Err(SolverError::Interrupted(e)),
                })
            });

        let Some((fitness, solution)) = best_solution? else {
            return Err(SolverError::NoSolutionFound);
        };

        Ok(SolverSolution {
            fitness,
            progress: progress.clone(),
            solution,
        })
    }
}

struct Worker<'a> {
    controller: &'a ExecutionController,
    counter: &'a AtomicU64,
    parameters: &'a SolverParameters,
    progress: &'a SolverProgress,
    solver: &'a Solver<'a>,
}

impl Worker<'_> {
    fn spin(&self) -> Result<(f32, Solution), SolverError> {
        let rng = &mut AppRng::try_from_rng(&mut SysRng).unwrap();

        let mut friday_solver =
            FridaySolver::new(&self.parameters.friday_parameters, &self.solver.context);

        let mut weekend_solver =
            WeekendSolver::new(&self.parameters.weekend_parameters, &self.solver.context);

        let mut weekday_solver =
            WeekdaySolver::new(&self.parameters.weekday_parameters, &self.solver.context);

        let mut best_solution = None;

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

                    let draft = &DraftSchedule {
                        fridays,
                        saturdays,
                        weekdays,
                    };

                    let fitness = self.solver.evaluator.evaluate(draft).total();

                    match &best_solution {
                        None => best_solution = Some((fitness, Solution::from(draft))),

                        Some((best_fitness, _)) if *best_fitness > fitness => {
                            best_solution = Some((fitness, Solution::from(draft)));
                        }

                        _ => {}
                    }
                }
            }
        }

        best_solution.ok_or(SolverError::NoSolutionFound)
    }
}

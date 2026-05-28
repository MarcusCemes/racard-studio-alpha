use std::{
    sync::atomic::{AtomicU8, AtomicU32, AtomicU64, Ordering},
    thread,
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{
    Solver,
    defs::ProblemInput,
    refiner::{RefinementParameters, Refiner, RefinerProgress},
    solver::{SolverError, defs::SolverParameters, types::SolverProgress},
    tools::controller::{ExecutionController, InterruptRequest},
    types::{Solution, Weights},
};

/* -- Parameters -- */

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OrchestrationParameters {
    pub top_k: u32,
    pub solver: SolverParameters,
    pub refiner: RefinementParameters,
}

/* -- Solution -- */

#[derive(Serialize)]
pub struct OrchestrationSolution {
    pub fitness: f32,
    pub solution: Solution,
    pub progress: OrchestrationProgress,
}

/* -- Progress -- */

pub struct OrchestrationProgress {
    pub phase: AtomicU8,
    pub solver: SolverProgress,
    pub refiner: RefinerProgress,
    pub refined: AtomicU32,
    pub total: AtomicU32,
    pub best_fitness: AtomicU64,
}

impl Default for OrchestrationProgress {
    fn default() -> Self {
        Self {
            phase: AtomicU8::new(Phase::Solving as u8),
            solver: SolverProgress::default(),
            refiner: RefinerProgress::default(),
            refined: AtomicU32::new(0),
            total: AtomicU32::new(0),
            best_fitness: AtomicU64::new(f32::MAX.to_bits() as u64),
        }
    }
}

impl Serialize for OrchestrationProgress {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use serde::ser::SerializeStruct;

        let phase = self.phase.load(Ordering::Relaxed);
        let refined = self.refined.load(Ordering::Relaxed);
        let total = self.total.load(Ordering::Relaxed);
        let best_fitness = f32::from_bits(self.best_fitness.load(Ordering::Relaxed) as u32);

        let mut s = serializer.serialize_struct("OrchestrationProgress", 6)?;
        s.serialize_field("phase", &phase)?;
        s.serialize_field("solver", &self.solver)?;
        s.serialize_field("refiner", &self.refiner)?;
        s.serialize_field("refined", &refined)?;
        s.serialize_field("total", &total)?;
        s.serialize_field("best_fitness", &best_fitness)?;
        s.end()
    }
}

impl OrchestrationProgress {
    fn set_phase(&self, phase: Phase) {
        self.phase.store(phase as u8, Ordering::Relaxed);
    }

    fn report_best(&self, fitness: f32) {
        let bits = fitness.to_bits() as u64;
        let mut current = self.best_fitness.load(Ordering::Relaxed);
        loop {
            if fitness >= f32::from_bits(current as u32) {
                break;
            }
            match self.best_fitness.compare_exchange_weak(
                current,
                bits,
                Ordering::Relaxed,
                Ordering::Relaxed,
            ) {
                Ok(_) => break,
                Err(actual) => current = actual,
            }
        }
    }
}

#[repr(u8)]
enum Phase {
    Solving = 0,
    Refining = 1,
}

/* -- Error -- */

#[derive(Debug, Error)]
pub enum OrchestrationError {
    #[error("Solver error: {0}")]
    Solver(#[from] SolverError),

    #[error("{0}")]
    Interrupted(#[from] InterruptRequest),
}

/* -- Orchestrator -- */

pub struct Orchestrator;

impl Orchestrator {
    pub fn execute(
        problem: &ProblemInput,
        weights: &Weights,
        parameters: &OrchestrationParameters,
        controller: &ExecutionController,
        progress: &OrchestrationProgress,
    ) -> Result<OrchestrationSolution, OrchestrationError> {
        /* -- Phase 1: Solve top-K -- */
        progress.set_phase(Phase::Solving);

        let top_solutions = {
            let mut solver = Solver::new(problem, weights);
            solver.execute_top_k(
                parameters.top_k,
                parameters.solver.clone(),
                None,
                controller,
                &progress.solver,
            )?
        };

        /* -- Phase 2: Refine each -- */
        progress.set_phase(Phase::Refining);
        progress
            .total
            .store(top_solutions.len() as u32, Ordering::Relaxed);

        if top_solutions.is_empty() {
            return Err(SolverError::NoSolutionFound.into());
        }

        let mut best: Option<(f32, Solution)> = None;

        let n = top_solutions.len();
        let n_threads = num_cpus::get().min(n.max(1));
        let chunk_size = (n + n_threads - 1) / n_threads;

        let refine_result = thread::scope(|scope| {
            let handles: Vec<_> = top_solutions
                .chunks(chunk_size)
                .map(|chunk| {
                    let refiner_params = &parameters.refiner;
                    scope.spawn(
                        move || -> Result<Option<(f32, Solution)>, OrchestrationError> {
                            let mut local_best: Option<(f32, Solution)> = None;
                            for sol in chunk {
                                controller.assert()?;

                                let refiner = Refiner::new(problem, weights);
                                let result = refiner.execute(
                                    &sol.solution,
                                    refiner_params,
                                    Some(1),
                                    controller,
                                    &progress.refiner,
                                )?;

                                let fitness =
                                    result.as_ref().map(|(f, _)| *f).unwrap_or(sol.fitness);

                                let solution = result
                                    .map(|(_, s)| s)
                                    .unwrap_or_else(|| sol.solution.clone());
                                let mut solution = solution;

                                if refiner_params.polish {
                                    refiner.polish(&mut solution);
                                    let polished_fitness =
                                        refiner.evaluator.evaluate(&solution).total();
                                    progress.report_best(polished_fitness);

                                    if local_best
                                        .as_ref()
                                        .is_none_or(|(b, _)| polished_fitness < *b)
                                    {
                                        local_best = Some((polished_fitness, solution));
                                    }

                                    progress.refined.fetch_add(1, Ordering::Relaxed);
                                    continue;
                                }

                                if local_best.as_ref().is_none_or(|(b, _)| fitness < *b) {
                                    local_best = Some((fitness, solution));
                                }

                                progress.refined.fetch_add(1, Ordering::Relaxed);
                                progress.report_best(fitness);
                            }
                            Ok(local_best)
                        },
                    )
                })
                .collect();

            for handle in handles {
                match handle.join().expect("refiner thread panicked") {
                    Ok(Some((f, s))) => {
                        if best.as_ref().is_none_or(|(b, _)| f < *b) {
                            best = Some((f, s));
                        }
                    }
                    Ok(None) => {}
                    Err(error) => return Err(error),
                }
            }
            Ok(())
        });
        refine_result?;

        let (fitness, solution) = best.ok_or(SolverError::NoSolutionFound)?;

        Ok(OrchestrationSolution {
            fitness,
            solution,
            progress: OrchestrationProgress {
                phase: AtomicU8::new(progress.phase.load(Ordering::Relaxed)),
                solver: progress.solver.clone(),
                refiner: progress.refiner.clone(),
                refined: AtomicU32::new(progress.refined.load(Ordering::Relaxed)),
                total: AtomicU32::new(progress.total.load(Ordering::Relaxed)),
                best_fitness: AtomicU64::new(progress.best_fitness.load(Ordering::Relaxed)),
            },
        })
    }
}

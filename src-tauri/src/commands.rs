use std::{
    sync::{Arc, Weak},
    time::Duration,
};

use chrono::{Days, NaiveDate};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, async_runtime};
use thiserror::Error;
use tokio::time::interval;

use algorithm::{
    Conflict, Context, ExecutionController, Holiday, InterruptRequest, N_DAYS, OrchestrationError,
    OrchestrationParameters, OrchestrationProgress, OrchestrationSolution, ProblemConfig,
    ProblemInput, ProblemInputError, RefinementParameters, Refiner, RefinerProgress,
    ScheduleStatistics, ScheduleValidator, Solution, Solver, SolverError, SolverParameters,
    SolverProgress, SolverSolution, Weights,
};

use crate::types::Handle;

const REPORT_PERIOD: Duration = Duration::from_millis(20);
const SOLVER_PROGRESS_KEY: &str = "solver-progress";
const REFINER_PROGRESS_KEY: &str = "refiner-progress";
const ORCHESTRATE_PROGRESS_KEY: &str = "orchestrate-progress";

/* === Solver === */

#[derive(Debug, Error)]
pub enum SolveError {
    #[error("Problem input error: {0}")]
    ProblemInput(#[from] ProblemInputError),

    #[error("Solver error: {0}")]
    Solver(#[from] SolverError),
}

impl Serialize for SolveError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn solve(
    app: AppHandle,
    problem: ProblemConfig,
    solver_parameters: SolverParameters,
    weights: Weights,
) -> Result<SolverSolution, SolveError> {
    let problem = ProblemInput::try_from(problem)?;

    let controller = ExecutionController::default();
    let progress = Arc::new(SolverProgress::default());

    {
        let handle = app.state::<Handle<SolverProgress>>();
        let mut lock = handle.0.lock().await;
        *lock = Some((progress.clone(), controller.clone()));
    }

    async_runtime::spawn(progress_reporter(
        app.clone(),
        Arc::downgrade(&progress),
        SOLVER_PROGRESS_KEY,
    ));

    let result = async_runtime::spawn_blocking(move || {
        Solver::new(&problem, &weights).execute(solver_parameters, None, &controller, &progress)
    })
    .await
    .unwrap();

    // Stop the progress reporter
    interrupt_handle::<SolverProgress>(&app).await;

    Ok(result?) // coerce the error
}

/* === Refine === */

#[derive(Debug, Error)]
pub enum RefineError {
    #[error("Problem input error: {0}")]
    ProblemInput(#[from] ProblemInputError),

    #[error("{0}")]
    Interrupted(#[from] InterruptRequest),
}

impl Serialize for RefineError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn refine(
    app: AppHandle,
    problem: ProblemConfig,
    parameters: RefinementParameters,
    solution: Solution,
    weights: Weights,
) -> Result<Option<(f32, Solution)>, RefineError> {
    let problem = ProblemInput::try_from(problem)?;

    let controller = ExecutionController::default();
    let progress = Arc::new(RefinerProgress::new());

    {
        let handle = app.state::<Handle<RefinerProgress>>();
        let mut lock = handle.0.lock().await;
        *lock = Some((progress.clone(), controller.clone()));
    }

    async_runtime::spawn(progress_reporter(
        app.clone(),
        Arc::downgrade(&progress),
        REFINER_PROGRESS_KEY,
    ));

    let result = async_runtime::spawn_blocking(move || {
        let refiner = Refiner::new(&problem, &weights);

        let mut result = refiner.execute(&solution, &parameters, None, &controller, &progress);

        if let Ok(Some((fitness, solution))) = &mut result
            && parameters.polish
        {
            refiner.polish(solution);
            *fitness = refiner.evaluator.evaluate(solution).total();
        };

        result
    })
    .await
    .unwrap();

    // Stop the progress reporter
    interrupt_handle::<RefinerProgress>(&app).await;

    Ok(result?)
}

/* === Statistics === */

#[tauri::command(rename_all = "snake_case")]
pub async fn statistics(
    problem: ProblemConfig,
    solution: Solution,
    weights: Weights,
) -> Result<ScheduleStatistics, ProblemInputError> {
    let problem = ProblemInput::try_from(problem)?;

    Ok(ScheduleStatistics::compute(&problem, &solution, &weights))
}

/* === Orchestrate === */

#[derive(Debug, Error)]
pub enum OrchestrateError {
    #[error("Problem input error: {0}")]
    ProblemInput(#[from] ProblemInputError),

    #[error("{0}")]
    Orchestration(#[from] OrchestrationError),
}

impl Serialize for OrchestrateError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn orchestrate(
    app: AppHandle,
    problem: ProblemConfig,
    parameters: OrchestrationParameters,
    weights: Weights,
) -> Result<OrchestrationSolution, OrchestrateError> {
    let problem = ProblemInput::try_from(problem)?;

    let controller = ExecutionController::default();
    let progress = Arc::new(OrchestrationProgress::default());

    {
        let handle = app.state::<Handle<OrchestrationProgress>>();
        let mut lock = handle.0.lock().await;
        *lock = Some((progress.clone(), controller.clone()));
    }

    async_runtime::spawn(progress_reporter(
        app.clone(),
        Arc::downgrade(&progress),
        ORCHESTRATE_PROGRESS_KEY,
    ));

    let result = async_runtime::spawn_blocking(move || {
        algorithm::Orchestrator::execute(&problem, &weights, &parameters, &controller, &progress)
    })
    .await
    .unwrap();

    // Stop the progress reporter
    interrupt_handle::<OrchestrationProgress>(&app).await;

    Ok(result?)
}

/* === Progress reporter === */

async fn progress_reporter<P: Serialize + Send + Sync + 'static>(
    app: AppHandle,
    progress: Weak<P>,
    event: &'static str,
) {
    let mut timer = interval(REPORT_PERIOD);

    loop {
        timer.tick().await;

        let Some(progress) = progress.upgrade() else {
            break;
        };

        if app.emit(event, &*progress).is_err() {
            break;
        }
    }
}

/* === Interrupt === */

#[tauri::command(rename_all = "snake_case")]
pub async fn interrupt(app: AppHandle) {
    interrupt_handle::<SolverProgress>(&app).await;
    interrupt_handle::<RefinerProgress>(&app).await;
    interrupt_handle::<OrchestrationProgress>(&app).await;
}

async fn interrupt_handle<P: Send + Sync + 'static>(app: &AppHandle) {
    let handle = app.state::<Handle<P>>();

    if let Some((_, controller)) = handle.0.lock().await.take() {
        controller.request_stop();
    }
}

/* === Bank holidays === */

#[tauri::command(rename_all = "snake_case")]
pub fn geneva_bank_holidays(start_date: NaiveDate) -> Vec<(NaiveDate, Holiday)> {
    let end_date = start_date + Days::new(N_DAYS as u64 - 1);
    algorithm::geneva_bank_holidays(start_date..=end_date).collect()
}

/* === Validate === */

#[tauri::command(rename_all = "snake_case")]
pub fn validate(
    problem: ProblemConfig,
    solution: Solution,
) -> Result<Vec<Conflict>, ProblemInputError> {
    let problem = ProblemInput::try_from(problem)?;

    let context = Context::new(&problem);

    let conflicts = ScheduleValidator::new(&context, &solution)
        .validate()
        .collect();

    Ok(conflicts)
}

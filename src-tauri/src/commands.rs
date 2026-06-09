use std::{
    sync::{Arc, Weak, atomic::Ordering},
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

use crate::types::{ActiveOperation, ActiveOperationState};

const REPORT_PERIOD: Duration = Duration::from_millis(20);
const OPERATION_EVENT_KEY: &str = "operation-event";

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OperationKind {
    Solve,
    WeekendSolve,
    Refine,
    Orchestrate,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum OperationStatus {
    Started,
    Running,
    Finished,
    Failed,
    Interrupted,
}

#[derive(Clone, Copy, Serialize)]
#[serde(rename_all = "snake_case")]
enum OperationPhase {
    Solving,
    Refining,
}

#[derive(Clone, Serialize)]
struct OperationEvent<'a> {
    operation: Option<OperationKind>,
    status: OperationStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    phase: Option<OperationPhase>,
    progress: OperationProgress<'a>,
}

#[derive(Clone, Default, Serialize)]
struct OperationProgress<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    solver: Option<&'a SolverProgress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    refiner: Option<&'a RefinerProgress>,
    #[serde(skip_serializing_if = "Option::is_none")]
    orchestration: Option<OrchestrationProgressSummary>,
}

#[derive(Clone, Serialize)]
struct OrchestrationProgressSummary {
    refined: u32,
    total: u32,
    best_fitness: Option<f32>,
}

/* === Solver === */

#[derive(Debug, Error)]
pub enum SolveError {
    #[error("Another operation is already running")]
    Busy,

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
    let controller = begin_operation(&app, OperationKind::Solve)
        .await
        .map_err(|_| SolveError::Busy)?;
    let progress = Arc::new(SolverProgress::default());

    async_runtime::spawn(solver_progress_reporter(
        app.clone(),
        Arc::downgrade(&progress),
        OperationKind::Solve,
    ));

    let result = async_runtime::spawn_blocking(move || {
        Solver::new(&problem, &weights).execute(solver_parameters, None, &controller, &progress)
    })
    .await
    .unwrap();

    let status = match &result {
        Ok(_) => OperationStatus::Finished,
        Err(SolverError::Interrupted(_)) => OperationStatus::Interrupted,
        Err(_) => OperationStatus::Failed,
    };
    end_operation(&app, OperationKind::Solve, status).await;

    Ok(result?)
}

/* === Weekend Solve === */

#[derive(Debug, Error)]
pub enum WeekendSolveError {
    #[error("Another operation is already running")]
    Busy,

    #[error("Problem input error: {0}")]
    ProblemInput(#[from] ProblemInputError),

    #[error("Solver error: {0}")]
    Solver(#[from] SolverError),
}

impl Serialize for WeekendSolveError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[tauri::command(rename_all = "snake_case")]
pub async fn weekend_solve(
    app: AppHandle,
    problem: ProblemConfig,
    solver_parameters: SolverParameters,
    weights: Weights,
) -> Result<SolverSolution, WeekendSolveError> {
    let problem = ProblemInput::try_from(problem)?;
    let controller = begin_operation(&app, OperationKind::WeekendSolve)
        .await
        .map_err(|_| WeekendSolveError::Busy)?;
    let progress = Arc::new(SolverProgress::default());

    async_runtime::spawn(solver_progress_reporter(
        app.clone(),
        Arc::downgrade(&progress),
        OperationKind::WeekendSolve,
    ));

    let result = async_runtime::spawn_blocking(move || {
        Solver::new(&problem, &weights).execute_weekends(
            solver_parameters,
            None,
            &controller,
            &progress,
        )
    })
    .await
    .unwrap();

    let status = match &result {
        Ok(_) => OperationStatus::Finished,
        Err(SolverError::Interrupted(_)) => OperationStatus::Interrupted,
        Err(_) => OperationStatus::Failed,
    };
    end_operation(&app, OperationKind::WeekendSolve, status).await;

    Ok(result?)
}

/* === Refine === */

#[derive(Debug, Error)]
pub enum RefineError {
    #[error("Another operation is already running")]
    Busy,

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
) -> Result<(f32, Solution), RefineError> {
    let problem = ProblemInput::try_from(problem)?;
    let controller = begin_operation(&app, OperationKind::Refine)
        .await
        .map_err(|_| RefineError::Busy)?;
    let progress = Arc::new(RefinerProgress::new());

    async_runtime::spawn(refiner_progress_reporter(
        app.clone(),
        Arc::downgrade(&progress),
    ));

    let result = async_runtime::spawn_blocking(move || {
        let refiner = Refiner::new(&problem, &weights);

        let result = refiner.execute(&solution, &parameters, None, &controller, &progress);

        let (mut fitness, mut solution) = match result {
            Ok(Some(result)) => result,
            Ok(None) => {
                let fitness = refiner.evaluator.evaluate(&solution).total();
                (fitness, solution)
            }
            Err(error) => return Err(error),
        };

        if parameters.polish {
            refiner.polish(&mut solution);
            refiner.cleanup_consecutive(&mut solution);
            fitness = refiner.evaluator.evaluate(&solution).total();
        };

        Ok((fitness, solution))
    })
    .await
    .unwrap();

    let status = match &result {
        Ok(_) => OperationStatus::Finished,
        Err(_) => OperationStatus::Interrupted,
    };
    end_operation(&app, OperationKind::Refine, status).await;

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
    #[error("Another operation is already running")]
    Busy,

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
    let controller = begin_operation(&app, OperationKind::Orchestrate)
        .await
        .map_err(|_| OrchestrateError::Busy)?;
    let progress = Arc::new(OrchestrationProgress::default());

    async_runtime::spawn(orchestration_progress_reporter(
        app.clone(),
        Arc::downgrade(&progress),
    ));

    let result = async_runtime::spawn_blocking(move || {
        algorithm::Orchestrator::execute(&problem, &weights, &parameters, &controller, &progress)
    })
    .await
    .unwrap();

    let status = match &result {
        Ok(_) => OperationStatus::Finished,
        Err(OrchestrationError::Interrupted(_)) => OperationStatus::Interrupted,
        Err(_) => OperationStatus::Failed,
    };
    end_operation(&app, OperationKind::Orchestrate, status).await;

    Ok(result?)
}

/* === Operation lifecycle === */

async fn begin_operation(app: &AppHandle, kind: OperationKind) -> Result<ExecutionController, ()> {
    let state = app.state::<ActiveOperationState>();
    let mut active = state.0.lock().await;
    if active.is_some() {
        return Err(());
    }

    let controller = ExecutionController::default();
    *active = Some(ActiveOperation {
        controller: controller.clone(),
    });

    emit_operation_event(
        app,
        kind,
        OperationStatus::Started,
        None,
        OperationProgress::default(),
    );

    Ok(controller)
}

async fn end_operation(app: &AppHandle, kind: OperationKind, status: OperationStatus) {
    let state = app.state::<ActiveOperationState>();
    let mut active = state.0.lock().await;
    *active = None;
    drop(active);

    emit_operation_event(app, kind, status, None, OperationProgress::default());
}

fn emit_operation_event(
    app: &AppHandle,
    kind: OperationKind,
    status: OperationStatus,
    phase: Option<OperationPhase>,
    progress: OperationProgress<'_>,
) {
    let _ = app.emit(
        OPERATION_EVENT_KEY,
        OperationEvent {
            operation: Some(kind),
            status,
            phase,
            progress,
        },
    );
}

/* === Progress reporters === */

async fn solver_progress_reporter(
    app: AppHandle,
    progress: Weak<SolverProgress>,
    kind: OperationKind,
) {
    let mut timer = interval(REPORT_PERIOD);

    loop {
        timer.tick().await;

        let Some(progress) = progress.upgrade() else {
            break;
        };

        emit_operation_event(
            &app,
            kind,
            OperationStatus::Running,
            Some(OperationPhase::Solving),
            OperationProgress {
                solver: Some(progress.as_ref()),
                ..OperationProgress::default()
            },
        );
    }
}

async fn refiner_progress_reporter(app: AppHandle, progress: Weak<RefinerProgress>) {
    let mut timer = interval(REPORT_PERIOD);

    loop {
        timer.tick().await;

        let Some(progress) = progress.upgrade() else {
            break;
        };

        emit_operation_event(
            &app,
            OperationKind::Refine,
            OperationStatus::Running,
            Some(OperationPhase::Refining),
            OperationProgress {
                refiner: Some(progress.as_ref()),
                ..OperationProgress::default()
            },
        );
    }
}

async fn orchestration_progress_reporter(app: AppHandle, progress: Weak<OrchestrationProgress>) {
    let mut timer = interval(REPORT_PERIOD);

    loop {
        timer.tick().await;

        let Some(progress) = progress.upgrade() else {
            break;
        };

        let phase = match progress.phase.load(Ordering::Relaxed) {
            0 => OperationPhase::Solving,
            _ => OperationPhase::Refining,
        };
        let best_fitness = f32::from_bits(progress.best_fitness.load(Ordering::Relaxed) as u32);
        let best_fitness = (best_fitness != f32::MAX).then_some(best_fitness);

        emit_operation_event(
            &app,
            OperationKind::Orchestrate,
            OperationStatus::Running,
            Some(phase),
            OperationProgress {
                solver: Some(&progress.solver),
                refiner: Some(&progress.refiner),
                orchestration: Some(OrchestrationProgressSummary {
                    refined: progress.refined.load(Ordering::Relaxed),
                    total: progress.total.load(Ordering::Relaxed),
                    best_fitness,
                }),
                ..OperationProgress::default()
            },
        );
    }
}

/* === Interrupt === */

#[tauri::command(rename_all = "snake_case")]
pub async fn interrupt(app: AppHandle) {
    let state = app.state::<ActiveOperationState>();
    let active = state.0.lock().await;

    if let Some(active) = active.as_ref() {
        active.controller.request_stop();
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

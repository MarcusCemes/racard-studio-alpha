mod defs;
mod fitness;
mod holiday;
mod misc;
mod orchestrator;
mod refiner;
mod solver;
mod statistics;
mod tools;
mod types;
mod utils;
mod validate;

pub use {
    defs::{
        MAX_PEOPLE, N_DAYS, N_WEEKDAYS, N_WEEKS, NULL_ID, Person, ProblemConfig, ProblemInput,
        ProblemInputError, ProblemOverrides, Rate, RateError,
    },
    fitness::{ScheduleEvaluator, ScheduleFitness},
    holiday::{Holiday, geneva_bank_holidays},
    orchestrator::{
        OrchestrationError, OrchestrationParameters, OrchestrationProgress, OrchestrationSolution,
        Orchestrator,
    },
    refiner::{RefinementParameters, Refiner, RefinerProgress},
    solver::{
        Solver, SolverError, SolverSolution,
        context::Context,
        defs::{SolverParameters, WeekdayParameters, WeekendParameters},
        types::{SolverProgress, SolverStage},
    },
    statistics::{
        FinalPersonStats, GlobalStatistics, PersonStatistics, ScheduleStatistics, WeeklyPersonStats,
    },
    tools::controller::{ExecutionController, InterruptRequest},
    types::{DayIdx, PersonIdx, Role, ScheduleView, Slot, SlotArray, Solution, WeekIdx, Weights},
    utils::sample_people,
    validate::{Conflict, ScheduleValidator},
};

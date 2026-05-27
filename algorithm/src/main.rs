use std::{
    env::args,
    fmt::{self, Display},
    fs::File,
    io::BufReader,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread,
    time::Duration,
};

use algorithm::{
    Context, DayIdx, ExecutionController, N_WEEKS, Person, PersonIdx, ProblemInput,
    ProblemOverrides, RefinementParameters, Refiner, RefinerProgress, Role, ScheduleEvaluator,
    ScheduleValidator, Solver, SolverParameters, SolverProgress, SolverStage, WeekIdx, Weights,
    sample_people,
};
use chrono::NaiveDate;
use serde::Deserialize;
use strum::IntoEnumIterator;
use tracing_subscriber::EnvFilter;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

const TARGET_ARCH: &str = env!("CARGO_CFG_TARGET_ARCH");
const TARGET_ENV: &str = env!("CARGO_CFG_TARGET_ENV");
const TARGET_OS: &str = env!("CARGO_CFG_TARGET_OS");
const TARGET_VENDOR: &str = env!("CARGO_CFG_TARGET_VENDOR");

#[derive(Deserialize)]
struct Config {
    #[serde(rename = "problemParameters")]
    problem: ProblemInput,
    #[serde(rename = "refineParameters")]
    refinement: RefinementParameters,
    #[serde(rename = "solveParameters")]
    solver: SolverParameters,
}

pub fn main() {
    init();

    let args: Vec<String> = args().collect();
    let config_path = args
        .iter()
        .find(|arg| !arg.starts_with("--") && arg.ends_with(".json"));

    let (problem, solver_params, refinement_params) = if let Some(path) = config_path {
        println!("Loading configuration from: {path}");
        let file = File::open(path).expect("Failed to open config file");
        let reader = BufReader::new(file);
        let config: Config = serde_json::from_reader(reader).expect("Failed to parse config file");

        (config.problem, config.solver, config.refinement)
    } else {
        println!("Using default sample configuration");
        let start_date = NaiveDate::from_ymd_opt(2024, 8, 19).unwrap();
        let people = sample_people().to_vec();

        let mut params = if args.iter().any(|arg| arg == "--slow") {
            SolverParameters::SLOW
        } else {
            SolverParameters::FAST
        };
        params.max_solutions = 1;

        let weekday_hours = [
            [13.5, 7.5], // Mon
            [14.5, 5.5], // Tue
            [14.5, 7.5], // Wed
            [14.5, 7.5], // Thu
            [14.5, 7.5], // Fri
            [20.0, 7.5], // Sat
            [14.5, 5.0], // Sun
        ];

        let problem = ProblemInput::try_new(
            start_date,
            people,
            ProblemOverrides::default(),
            weekday_hours,
            3,
        )
        .unwrap();

        let refinement = if args.iter().any(|arg| arg == "--slow") {
            RefinementParameters::SLOW
        } else {
            RefinementParameters::FAST
        };

        (problem, params, refinement)
    };

    println!("Start date: {}", problem.start_date);
    println!("\n{solver_params:#?}");

    let progress = Arc::new(SolverProgress::default());
    let controller = ExecutionController::default();
    let people_clone = problem.people.clone();
    let stop = Arc::new(AtomicBool::new(false));

    let progress_clone = progress.clone();
    let stop_clone = stop.clone();

    thread::spawn(move || progress_reporter(progress_clone, stop_clone));

    let mut solver = Solver::new(&problem, &Weights::STANDARD);
    let result = solver.execute(solver_params, None, &controller, &progress);
    stop.store(true, Ordering::SeqCst);

    print_progress(&progress);
    println!();

    let solver_solution = match result {
        Ok(solution) => solution,

        Err(error) => {
            println!("{RED}Solver error: {error:?}{RESET}");
            return;
        }
    };

    let context = Context::new(&problem);

    let errors = ScheduleValidator::new(&context, &solver_solution.solution)
        .validate()
        .collect::<Vec<_>>();

    if let count @ 1.. = errors.len() {
        println!("{RED}{count} errors found:{RESET}");

        for error in errors {
            println!(" - {error:?}");
        }
    } else {
        println!("{GREEN}No errors found in initial solution!{RESET}");
    }

    if args.iter().any(|arg| arg == "--schedule") {
        print_full_schedule(&solver_solution.solution, &people_clone);
    }

    print_cost_breakdown(
        &solver_solution.solution,
        &people_clone,
        solver_solution.fitness,
        &problem,
    );

    section("REFINEMENT");

    let refiner = Refiner::new(&problem, &Weights::STANDARD);
    let refine_controller = ExecutionController::default();
    let refine_progress = RefinerProgress::new();
    let refined = refiner.execute(
        &solver_solution.solution,
        &refinement_params,
        None,
        &refine_controller,
        &refine_progress,
    );

    let mut refined_solution = match refined {
        Ok(Some((_, solution))) => solution,
        Ok(None) => {
            println!("{YELLOW}Refinement found no improvement{RESET}");
            solver_solution.solution.clone()
        }
        Err(e) => {
            println!("{YELLOW}Refinement interrupted: {e}{RESET}");
            solver_solution.solution.clone()
        }
    };

    println!("✨ Performing targeted polish pass...");
    refiner.polish(&mut refined_solution);

    let refined_fitness = refiner.evaluator.evaluate(&refined_solution);

    let errors = ScheduleValidator::new(&context, &refined_solution)
        .validate()
        .collect::<Vec<_>>();

    if let count @ 1.. = errors.len() {
        println!("{RED}{count} errors found:{RESET}");

        for error in errors {
            println!(" - {error:?}");
        }
    } else {
        println!("{GREEN}No errors found after refinement and polish!{RESET}");
    }

    if args.iter().any(|arg| arg == "--schedule") {
        print_full_schedule(&refined_solution, &people_clone);
    }

    print_cost_breakdown(
        &refined_solution,
        &people_clone,
        refined_fitness.total(),
        &problem,
    );
}

fn init() {
    eprintln!("{NAME} {VERSION} ({TARGET_ARCH}-{TARGET_VENDOR}-{TARGET_OS}-{TARGET_ENV})");
    eprintln!("Designed by Marcus Cemes\n");

    let filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("debug"))
        .unwrap();

    tracing_subscriber::fmt().with_env_filter(filter).init();
}

fn progress_reporter(progress: Arc<SolverProgress>, stop: Arc<AtomicBool>) {
    loop {
        thread::sleep(Duration::from_millis(500));

        if stop.load(Ordering::SeqCst) {
            break;
        }

        print_progress(&progress);
    }
}

fn print_progress(progress: &SolverProgress) {
    for stage in SolverStage::iter() {
        let stats = &progress[stage];
        let accepted = stats.accepted();
        let rejected = stats.rejected();
        let total = accepted + rejected;
        let mut success_rate = (accepted as f32 / total as f32) * 100.;

        if success_rate.is_nan() {
            success_rate = 100.;
        }

        print!("{stage:?} {GREEN}{accepted}{RESET}/{RED}{rejected}{RESET} ({success_rate:.0}%)  ");
    }

    println!();
}

fn print_full_schedule(solution: &algorithm::Solution, people: &[Person]) {
    section("FULL SCHEDULE");

    for (i, person) in people.iter().enumerate() {
        print!("{}{i}:{RESET} {}", PersonColour(i as u8), person.name);
    }

    println!(
        "\n{:<4} {:<3} {:<9} {:<9} {:<9} {:<9} {:<9} {:<9} {:<9}",
        "Week", "Day", "Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"
    );
    println!("{:-^100}", "");

    for week in WeekIdx::iter() {
        let week_num = week.get() + 1;

        // Lead row
        print!("{:>4} {:>3} ", week_num, "L");
        for day_offset in 0..7 {
            let day_idx = DayIdx::try_new(week.get() as u16 * 7 + day_offset as u16);
            if let Some(day_idx) = day_idx {
                let slot = solution[day_idx];
                if let Some(lead_person) = slot.get(Role::Lead) {
                    print!(
                        "{}  {:>2}     {RESET}",
                        PersonColour(lead_person.get()),
                        lead_person.get(),
                    );
                } else {
                    print!("   ?     ");
                }
            } else {
                print!("   -     ");
            }
        }
        println!();

        // Support row
        print!("     {:>3} ", "S");
        for day_offset in 0..7 {
            let day_idx = DayIdx::try_new(week.get() as u16 * 7 + day_offset as u16);
            if let Some(day_idx) = day_idx {
                let slot = solution[day_idx];
                if let Some(support_person) = slot.get(Role::Support) {
                    print!(
                        "{}  {:>2}     {RESET}",
                        PersonColour(support_person.get()),
                        support_person.get(),
                    );
                } else {
                    print!("   ?     ");
                }
            } else {
                print!("   -     ");
            }
        }
        println!();

        // Add spacing every 4 weeks for readability
        if week_num % 4 == 0 && week.get() < N_WEEKS as u8 - 1 {
            println!("{:-^100}", "");
        }
    }
}

fn print_cost_breakdown(
    solution: &algorithm::Solution,
    people: &[Person],
    total_fitness: f32,
    problem: &ProblemInput,
) {
    section("COST BREAKDOWN");

    let evaluator = ScheduleEvaluator::new(problem, &Weights::STANDARD);
    let deltas = evaluator.calculate_annual_deltas(solution);

    println!("📊 Cost Components:");
    println!("   🎯 TOTAL FITNESS       : {:.1}", total_fitness);

    println!("\n👥 Person Assignment Summary:");
    println!(
        "{:<12} {:>8} {:>8} {:>10}",
        "Person", "Lead", "Support", "Drift (h)"
    );
    println!("{:-^42}", "");

    for person in PersonIdx::iter(people.len()) {
        let mut lead_count = 0;
        let mut support_count = 0;

        for day_idx in DayIdx::iter() {
            let slot = solution[day_idx];

            if let Some(lead) = slot.get(Role::Lead)
                && lead == person
            {
                lead_count += 1;
            }

            if let Some(support) = slot.get(Role::Support)
                && support == person
            {
                support_count += 1;
            }
        }

        let drift = deltas[person.get() as usize];
        let drift_colour = if drift.abs() > 15.0 { RED } else { GREEN };

        println!(
            "{}{:<12}{RESET} {lead_count:>8} {support_count:>8} {drift_colour}{drift:>10.1}{RESET}",
            PersonColour(person.get()),
            people[person.get() as usize].name
        );
    }
}

/* -- Misc -- */

const RED: &str = "\x1b[91m";
const GREEN: &str = "\x1b[92m";
const YELLOW: &str = "\x1b[93m";
const BLUE: &str = "\x1b[94m";
const MAGENTA: &str = "\x1b[95m";
const CYAN: &str = "\x1b[96m";
const WHITE: &str = "\x1b[97m";
const GRAY: &str = "\x1b[90m";
const RESET: &str = "\x1b[0m";

struct PersonColour(u8);

impl Display for PersonColour {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        const COLOURS: [&str; 8] = [RED, GREEN, YELLOW, BLUE, MAGENTA, CYAN, WHITE, GRAY];
        let colour = COLOURS[(self.0 % 8) as usize];

        write!(f, "{colour}")
    }
}

fn section(name: &str) {
    println!("\n{:=^100}\n", format!(" {name} "));
}

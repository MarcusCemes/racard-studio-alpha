import { invoke } from "@tauri-apps/api/core";
import { addDays, format, getISODay } from "date-fns";

import { Role } from "$lib/defs.js";
import type {
    BankHoliday,
    Conflict,
    CustomOverride,
    FitnessWeights,
    Holiday,
    OrchestrationParameters,
    OrchestrationSolution,
    Person,
    ProblemConfig,
    ProblemOverrides,
    RefinementParameters,
    Schedule,
    ScheduleStatistics,
    Solution,
    SolverParameters,
    SolverSolution,
} from "$lib/schemas.js";

function buildOverrides(
    bank_holidays: BankHoliday[],
    custom_overrides: CustomOverride[],
    bank_holiday_default_hours: [number, number][],
): ProblemOverrides {
    const leadMap = new Map<string, number>();
    const supportMap = new Map<string, number>();

    // 1. Process bank holidays (Lead: day before, Support: day of)
    for (const bh of bank_holidays) {
        if (!bh.enabled) continue;

        const bhDayIdx = getISODay(bh.date) - 1; // getISODay() returns 1-based index (Monday = 1)
        const defaultsForThisBH = bank_holiday_default_hours[bhDayIdx];

        if (!defaultsForThisBH) {
            throw new Error(`Missing bank holiday default hours for weekday index ${bhDayIdx}`);
        }

        const leadDate = format(addDays(bh.date, 1), "yyyy-MM-dd");
        const leadHours = bh.lead_hours !== null ? bh.lead_hours : defaultsForThisBH[Role.Lead];
        leadMap.set(leadDate, leadHours);

        const supportHours =
            bh.support_hours !== null ? bh.support_hours : defaultsForThisBH[Role.Support];
        supportMap.set(bh.date, supportHours);
    }

    // 2. Process custom overrides (which override bank holiday hours)
    for (const co of custom_overrides) {
        if (co.role === "Lead") {
            leadMap.set(co.date, co.hours);
        } else {
            supportMap.set(co.date, co.hours);
        }
    }

    return {
        lead: Array.from(leadMap.entries()),
        support: Array.from(supportMap.entries()),
    };
}

function buildProblemConfig(input: {
    bank_holidays: BankHoliday[];
    custom_overrides: CustomOverride[];
    bank_holiday_default_hours: [number, number][];
    weekday_hours: [number, number][];
    people: Person[];
    start_date: string;
    skip_last_shifts: number;
}): ProblemConfig {
    return {
        start_date: input.start_date,
        people: input.people,
        weekday_hours: input.weekday_hours,
        overrides: buildOverrides(
            input.bank_holidays,
            input.custom_overrides,
            input.bank_holiday_default_hours,
        ),
        skip_last_shifts: input.skip_last_shifts,
    };
}

export async function apiSolve(input: {
    bank_holidays: BankHoliday[];
    custom_overrides: CustomOverride[];
    bank_holiday_default_hours: [number, number][];
    weekday_hours: [number, number][];
    people: Person[];
    start_date: string;
    skip_last_shifts: number;
    solver_parameters: SolverParameters;
    weights: FitnessWeights;
}) {
    const problem = buildProblemConfig(input);
    return await invoke<SolverSolution>("solve", {
        problem,
        solver_parameters: input.solver_parameters,
        weights: input.weights,
    });
}

export async function apiRefine(input: {
    bank_holidays: BankHoliday[];
    custom_overrides: CustomOverride[];
    bank_holiday_default_hours: [number, number][];
    weekday_hours: [number, number][];
    parameters: RefinementParameters;
    people: Person[];
    solution: Solution;
    start_date: string;
    skip_last_shifts: number;
    weights: FitnessWeights;
}) {
    const problem = buildProblemConfig(input);
    return await invoke<[number, Solution] | null>("refine", {
        problem,
        parameters: input.parameters,
        solution: input.solution,
        weights: input.weights,
    });
}

export async function apiOrchestrate(input: {
    bank_holidays: BankHoliday[];
    custom_overrides: CustomOverride[];
    bank_holiday_default_hours: [number, number][];
    weekday_hours: [number, number][];
    parameters: OrchestrationParameters;
    people: Person[];
    start_date: string;
    skip_last_shifts: number;
    weights: FitnessWeights;
}) {
    const problem = buildProblemConfig(input);
    return await invoke<OrchestrationSolution>("orchestrate", {
        problem,
        parameters: input.parameters,
        weights: input.weights,
    });
}

export async function apiStatistics(input: {
    bank_holidays: BankHoliday[];
    custom_overrides: CustomOverride[];
    bank_holiday_default_hours: [number, number][];
    weekday_hours: [number, number][];
    people: Person[];
    solution: Solution;
    start_date: string;
    skip_last_shifts: number;
    weights: FitnessWeights;
}) {
    const problem = buildProblemConfig(input);
    return await invoke<ScheduleStatistics>("statistics", {
        problem,
        solution: input.solution,
        weights: input.weights,
    });
}

export async function apiInterrupt() {
    await invoke("interrupt");
}

export async function apiBankHolidays(start_date: string) {
    return await invoke<[string, Holiday][]>("geneva_bank_holidays", { start_date });
}

export async function apiValidate(input: {
    bank_holidays: BankHoliday[];
    custom_overrides: CustomOverride[];
    bank_holiday_default_hours: [number, number][];
    weekday_hours: [number, number][];
    people: Person[];
    solution: Solution;
    start_date: string;
    skip_last_shifts: number;
}): Promise<Conflict[]> {
    const problem = buildProblemConfig(input);
    return await invoke("validate", {
        problem,
        solution: input.solution,
    });
}

export async function exportSchedule(schedule: Schedule) {
    await invoke("export_schedule", { schedule });
}

export async function showMainWindow() {
    await invoke("show_main_window");
}

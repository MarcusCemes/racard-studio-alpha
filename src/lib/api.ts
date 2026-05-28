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
    OrchestrationRequest,
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
    bankHolidays: BankHoliday[],
    customOverrides: CustomOverride[],
    bankHolidayDefaultHours: [number, number][],
): ProblemOverrides {
    const leadMap = new Map<string, number>();
    const supportMap = new Map<string, number>();

    for (const bh of bankHolidays) {
        if (!bh.enabled) continue;
        const bhDayIdx = getISODay(bh.date) - 1;
        const defaults = bankHolidayDefaultHours[bhDayIdx];
        if (!defaults) {
            throw new Error(`Missing bank holiday default hours for weekday index ${bhDayIdx}`);
        }
        const leadDate = format(addDays(bh.date, 1), "yyyy-MM-dd");
        const leadHours = bh.lead_hours ?? defaults[Role.Lead];
        leadMap.set(leadDate, leadHours);
        const supportHours = bh.support_hours ?? defaults[Role.Support];
        supportMap.set(bh.date, supportHours);
    }

    for (const co of customOverrides) {
        if (co.role === "Lead") leadMap.set(co.date, co.hours);
        else supportMap.set(co.date, co.hours);
    }

    return {
        lead: Array.from(leadMap.entries()),
        support: Array.from(supportMap.entries()),
    };
}

export function buildProblemConfig(input: {
    startDate: string;
    people: Person[];
    weekdayHours: [number, number][];
    bankHolidayDefaultHours: [number, number][];
    bankHolidays: BankHoliday[];
    customOverrides: CustomOverride[];
    skipLastShifts: number;
}): ProblemConfig {
    return {
        start_date: input.startDate,
        people: input.people,
        weekday_hours: input.weekdayHours,
        overrides: buildOverrides(
            input.bankHolidays,
            input.customOverrides,
            input.bankHolidayDefaultHours,
        ),
        skip_last_shifts: input.skipLastShifts,
    };
}

export async function apiSolve(
    problem: {
        startDate: string;
        people: Person[];
        weekdayHours: [number, number][];
        bankHolidayDefaultHours: [number, number][];
        bankHolidays: BankHoliday[];
        customOverrides: CustomOverride[];
        skipLastShifts: number;
    },
    solverParams: SolverParameters,
    weights: FitnessWeights,
) {
    return await invoke<SolverSolution>("solve", {
        problem: buildProblemConfig(problem),
        solver_parameters: solverParams,
        weights,
    });
}

export async function apiRefine(
    problem: {
        startDate: string;
        people: Person[];
        weekdayHours: [number, number][];
        bankHolidayDefaultHours: [number, number][];
        bankHolidays: BankHoliday[];
        customOverrides: CustomOverride[];
        skipLastShifts: number;
    },
    refinerParams: RefinementParameters,
    solution: Solution,
    weights: FitnessWeights,
) {
    return await invoke<[number, Solution]>("refine", {
        problem: buildProblemConfig(problem),
        parameters: refinerParams,
        solution,
        weights,
    });
}

export async function apiOrchestrate(
    problem: {
        startDate: string;
        people: Person[];
        weekdayHours: [number, number][];
        bankHolidayDefaultHours: [number, number][];
        bankHolidays: BankHoliday[];
        customOverrides: CustomOverride[];
        skipLastShifts: number;
    },
    orchestrateParams: OrchestrationParameters,
    solverParams: SolverParameters,
    refinerParams: RefinementParameters,
    weights: FitnessWeights,
) {
    const parameters: OrchestrationRequest = {
        top_k: orchestrateParams.top_k,
        solver: solverParams,
        refiner: refinerParams,
    };

    return await invoke<OrchestrationSolution>("orchestrate", {
        problem: buildProblemConfig(problem),
        parameters,
        weights,
    });
}

export async function apiStatistics(
    problem: {
        startDate: string;
        people: Person[];
        weekdayHours: [number, number][];
        bankHolidayDefaultHours: [number, number][];
        bankHolidays: BankHoliday[];
        customOverrides: CustomOverride[];
        skipLastShifts: number;
    },
    solution: Solution,
    weights: FitnessWeights,
) {
    return await invoke<ScheduleStatistics>("statistics", {
        problem: buildProblemConfig(problem),
        solution,
        weights,
    });
}

export async function apiInterrupt() {
    await invoke("interrupt");
}

export async function apiBankHolidays(start_date: string) {
    return await invoke<[string, Holiday][]>("geneva_bank_holidays", { start_date });
}

export async function apiValidate(
    problem: {
        startDate: string;
        people: Person[];
        weekdayHours: [number, number][];
        bankHolidayDefaultHours: [number, number][];
        bankHolidays: BankHoliday[];
        customOverrides: CustomOverride[];
        skipLastShifts: number;
    },
    solution: Solution,
): Promise<Conflict[]> {
    return await invoke("validate", {
        problem: buildProblemConfig(problem),
        solution,
    });
}

export async function exportSchedule(schedule: Schedule) {
    await invoke("export_schedule", { schedule });
}

export async function showMainWindow() {
    await invoke("show_main_window");
}

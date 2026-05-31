import { invoke } from "@tauri-apps/api/core";
import { addDays, format, getISODay } from "date-fns";

import { Role } from "$lib/defs.js";
import type * as T from "$lib/schemas.js";

function buildOverrides(
    bankHolidays: T.BankHoliday[],
    customOverrides: T.CustomOverride[],
    bankHolidayDefaultHours: [number, number][],
): T.ProblemOverrides {
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
        const leadHours = bh.lead_hours ?? defaults[0];
        leadMap.set(leadDate, leadHours);
        const supportHours = bh.support_hours ?? defaults[1];
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
    people: T.Person[];
    weekdayHours: [number, number][];
    bankHolidayDefaultHours: [number, number][];
    bankHolidays: T.BankHoliday[];
    customOverrides: T.CustomOverride[];
    skipLastShifts: number;
}): T.ProblemConfig {
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
        people: T.Person[];
        weekdayHours: [number, number][];
        bankHolidayDefaultHours: [number, number][];
        bankHolidays: T.BankHoliday[];
        customOverrides: T.CustomOverride[];
        skipLastShifts: number;
    },
    solverParams: T.SolverParameters,
    weights: T.FitnessWeights,
) {
    return await invoke<T.SolverSolution>("solve", {
        problem: buildProblemConfig(problem),
        solver_parameters: solverParams,
        weights,
    });
}

export async function apiRefine(
    problem: {
        startDate: string;
        people: T.Person[];
        weekdayHours: [number, number][];
        bankHolidayDefaultHours: [number, number][];
        bankHolidays: T.BankHoliday[];
        customOverrides: T.CustomOverride[];
        skipLastShifts: number;
    },
    refinerParams: T.RefinementParameters,
    solution: T.Solution,
    weights: T.FitnessWeights,
) {
    return await invoke<[number, T.Solution]>("refine", {
        problem: buildProblemConfig(problem),
        parameters: refinerParams,
        solution,
        weights,
    });
}

export async function apiOrchestrate(
    problem: {
        startDate: string;
        people: T.Person[];
        weekdayHours: [number, number][];
        bankHolidayDefaultHours: [number, number][];
        bankHolidays: T.BankHoliday[];
        customOverrides: T.CustomOverride[];
        skipLastShifts: number;
    },
    orchestrateParams: T.OrchestrationParameters,
    solverParams: T.SolverParameters,
    refinerParams: T.RefinementParameters,
    weights: T.FitnessWeights,
) {
    const parameters: T.OrchestrationRequest = {
        top_k: orchestrateParams.top_k,
        solver: solverParams,
        refiner: refinerParams,
    };

    return await invoke<T.OrchestrationSolution>("orchestrate", {
        problem: buildProblemConfig(problem),
        parameters,
        weights,
    });
}

export async function apiStatistics(
    problem: {
        startDate: string;
        people: T.Person[];
        weekdayHours: [number, number][];
        bankHolidayDefaultHours: [number, number][];
        bankHolidays: T.BankHoliday[];
        customOverrides: T.CustomOverride[];
        skipLastShifts: number;
    },
    solution: T.Solution,
    weights: T.FitnessWeights,
) {
    return await invoke<T.ScheduleStatistics>("statistics", {
        problem: buildProblemConfig(problem),
        solution,
        weights,
    });
}

export async function apiInterrupt() {
    await invoke("interrupt");
}

export async function apiBankHolidays(start_date: string) {
    return await invoke<[string, T.Holiday][]>("geneva_bank_holidays", { start_date });
}

export async function apiValidate(
    problem: {
        startDate: string;
        people: T.Person[];
        weekdayHours: [number, number][];
        bankHolidayDefaultHours: [number, number][];
        bankHolidays: T.BankHoliday[];
        customOverrides: T.CustomOverride[];
        skipLastShifts: number;
    },
    solution: T.Solution,
): Promise<T.Conflict[]> {
    return await invoke("validate", {
        problem: buildProblemConfig(problem),
        solution,
    });
}

export async function exportSchedule(schedule: T.Schedule) {
    await invoke("export_schedule", { schedule });
}

export async function showMainWindow() {
    await invoke("show_main_window");
}

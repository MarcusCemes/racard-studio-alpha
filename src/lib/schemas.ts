import { z } from "zod";

import {
    DEFAULT_BANK_HOLIDAY_HOURS,
    DEFAULT_WEEKDAY_HOURS,
    MAX_PEOPLE,
    N_DAYS,
    N_WEEKS,
} from "$lib/defs.js";

const dateSchema = z.string().regex(/^\d{4}-\d{2}-\d{2}$/);
const slotSchema = z.number().brand<"Slot">();
const slotsSchema = z.array(slotSchema).length(N_DAYS);

export type Slot = z.infer<typeof slotSchema>;
export type Slots = z.infer<typeof slotsSchema>;

/* === Core Types === */

export type Person = z.infer<typeof personSchema>;
const personSchema = z.object({
    name: z.string(),
    holidays: z.array(z.number()),
    rate: z.number().int().min(0).max(100),
});

export type BankHoliday = z.infer<typeof bankHolidaySchema>;
export const bankHolidaySchema = z.object({
    date: dateSchema,
    name: z.string(),
    enabled: z.boolean(),
    lead_hours: z.number().nullable(),
    support_hours: z.number().nullable(),
});

export type CustomOverride = z.infer<typeof customOverrideSchema>;
export const customOverrideSchema = z.object({
    date: dateSchema,
    role: z.enum(["Lead", "Support"]),
    hours: z.number(),
});

export type ProblemOverrides = z.infer<typeof problemOverridesSchema>;
export const problemOverridesSchema = z.object({
    lead: z.array(z.tuple([dateSchema, z.number()])),
    support: z.array(z.tuple([dateSchema, z.number()])),
});

export type ProblemConfig = z.infer<typeof problemConfigSchema>;
export const problemConfigSchema = z.object({
    start_date: dateSchema,
    people: z.array(personSchema),
    weekday_hours: z.array(z.tuple([z.number(), z.number()])).length(7),
    overrides: problemOverridesSchema,
    skip_last_shifts: z.number().int().min(0),
});

export type ProblemParameters = z.infer<typeof parametersSchema>;
export const parametersSchema = z.object({
    weekday_hours: z
        .array(z.tuple([z.number(), z.number()]))
        .length(7)
        .default(DEFAULT_WEEKDAY_HOURS),
    bank_holiday_default_hours: z
        .array(z.tuple([z.number(), z.number()]))
        .length(7)
        .default(DEFAULT_BANK_HOLIDAY_HOURS),
    bank_holidays: z.array(bankHolidaySchema).default([]),
    custom_overrides: z.array(customOverrideSchema).default([]),
    people: z.array(personSchema).min(1).max(MAX_PEOPLE),
    start_date: dateSchema,
});

export type Solution = z.infer<typeof solutionSchema>;
export const solutionSchema = slotsSchema;

export type Schedule = z.infer<typeof scheduleSchema>;
export const scheduleSchema = z.object({
    parameters: parametersSchema,
    slots: slotsSchema,
});

/* === Parameters === */

export type PhaseParameters = z.infer<typeof phaseParametersSchema>;
export const phaseParametersSchema = z.object({
    number_permutations: z.number().int().min(1),
    max_resolve_attempts: z.number().int().min(1),
});

export type SolverParameters = z.infer<typeof solverParametersSchema>;
export const solverParametersSchema = z.object({
    weekend: phaseParametersSchema,
    friday: phaseParametersSchema,
    weekday: phaseParametersSchema,
});

export type RefinementParameters = z.infer<typeof refinementParametersSchema>;
export const refinementParametersSchema = z.object({
    cooling_rate: z.number().gt(0).lt(1),
    initial_temperature: z.number().gt(0),
    num_iterations: z.number().int().min(0),
    polish: z.boolean(),
    searches: z.number().int().min(1),
});

export type OrchestrationParameters = z.infer<typeof orchestrationParametersSchema>;
export const orchestrationParametersSchema = z.object({
    top_k: z.number().int().min(1),
});

export type OrchestrationRequest = z.infer<typeof orchestrationRequestSchema>;
export const orchestrationRequestSchema = orchestrationParametersSchema.extend({
    solver: solverParametersSchema,
    refiner: refinementParametersSchema,
});

export type FitnessWeights = z.infer<typeof fitnessWeightsSchema>;
export const fitnessWeightsSchema = z.object({
    annual_hours: z.number().min(0),
    consecutive_days: z.number().min(0),
    consecutive_weekends: z.number().min(0),
    weekend_alternation: z.number().min(0),
    weekend_regularity: z.number().min(0),
    weekly_hours: z.number().min(0),
    blank_weeks: z.number().min(0),
});

/* === Progress & Solution === */

export type StageProgress = z.infer<typeof stageProgressSchema>;
export const stageProgressSchema = z.object({
    accepted: z.number().int().min(0),
    rejected: z.number().int().min(0),
});

export type SolverProgress = z.infer<typeof solverProgressSchema>;
export const solverProgressSchema = z.tuple([
    stageProgressSchema,
    stageProgressSchema,
    stageProgressSchema,
]);

/* === RefinerProgress === */

export type RefinerProgress = z.infer<typeof refinerProgressSchema>;
export const refinerProgressSchema = z.object({
    accepted: z.number().int().min(0),
    rejected: z.number().int().min(0),
    current_fitness: z.number(),
    best_fitness: z.number(),
    temperature: z.number(),
    iteration: z.number().int().min(0),
    search: z.number().int().min(0),
});

export type SolverSolution = z.infer<typeof solverSolutionSchema>;
export const solverSolutionSchema = z.object({
    fitness: z.number(),
    progress: solverProgressSchema,
    solution: slotsSchema,
});

export type OrchestrationProgress = z.infer<typeof orchestrationProgressSchema>;
export const orchestrationProgressSchema = z.object({
    phase: z.number().int().min(0).max(1),
    solver: solverProgressSchema,
    refiner: refinerProgressSchema,
    refined: z.number().int().min(0),
    total: z.number().int().min(0),
    best_fitness: z.number(),
});

export type OrchestrationSolution = z.infer<typeof orchestrationSolutionSchema>;
export const orchestrationSolutionSchema = z.object({
    fitness: z.number(),
    solution: slotsSchema,
    progress: orchestrationProgressSchema,
});

export const operationKindSchema = z.enum(["solve", "refine", "orchestrate"]);
export type OperationKind = z.infer<typeof operationKindSchema>;

export const operationStatusSchema = z.enum([
    "started",
    "running",
    "finished",
    "failed",
    "interrupted",
]);
export type OperationStatus = z.infer<typeof operationStatusSchema>;

export const operationPhaseSchema = z.enum(["solving", "refining"]);
export type OperationPhase = z.infer<typeof operationPhaseSchema>;

export type OperationProgressSummary = z.infer<typeof operationProgressSummarySchema>;
export const operationProgressSummarySchema = z.object({
    refined: z.number().int().min(0),
    total: z.number().int().min(0),
    best_fitness: z.number().nullable().optional(),
});

export type OperationProgress = z.infer<typeof operationProgressSchema>;
export const operationProgressSchema = z.object({
    solver: solverProgressSchema.optional(),
    refiner: refinerProgressSchema.optional(),
    orchestration: operationProgressSummarySchema.optional(),
});

export type OperationEvent = z.infer<typeof operationEventSchema>;
export const operationEventSchema = z.object({
    operation: operationKindSchema.nullable(),
    status: operationStatusSchema,
    phase: operationPhaseSchema.optional(),
    progress: operationProgressSchema,
});

/* === Errors === */

export type TaskInputError = z.infer<typeof taskInputErrorSchema>;
export const taskInputErrorSchema = z.enum(["StartDate", "PeopleCount"]);

export type SolverError = z.infer<typeof solverErrorSchema>;
export const solverErrorSchema = z.union([
    z.literal("Interrupted"),
    z.object({ Task: z.union([z.literal("StartDate"), z.literal("PeopleCount")]) }),
]);

/* === Statistics === */

export type WeeklyPersonStats = z.infer<typeof weeklyPersonStatsSchema>;
export const weeklyPersonStatsSchema = z.object({
    hours_by_role: z.tuple([z.number(), z.number()]),
    cumulative_hours: z.number(),
    slots_count: z.number().int(),
});

export type FinalPersonStats = z.infer<typeof finalPersonStatsSchema>;
export const finalPersonStatsSchema = z.object({
    total_hours_worked: z.number(),
    expected_hours: z.number(),
    lead_fridays: z.number().int(),
    support_fridays: z.number().int(),
    long_weekends: z.number().int(),
    short_weekends: z.number().int(),
});

export type PersonStatistics = z.infer<typeof personStatisticsSchema>;
export const personStatisticsSchema = z.object({
    name: z.string(),
    weeks: z.array(weeklyPersonStatsSchema).length(N_WEEKS),
    totals: finalPersonStatsSchema,
});

export type GlobalStatistics = z.infer<typeof globalStatisticsSchema>;
export const globalStatisticsSchema = z.object({
    total_available_hours: z.number(),
    theoretical_hours: z.number(),
});

export type ScheduleFitness = z.infer<typeof scheduleFitnessSchema>;
export const scheduleFitnessSchema = z.object({
    annual_hours: z.number(),
    consecutive_days: z.number(),
    consecutive_weekends: z.number(),
    weekend_alternation: z.number(),
    weekend_regularity: z.number(),
    weekly_hours: z.number(),
    blank_weeks: z.number(),
});

export type ScheduleStatistics = z.infer<typeof scheduleStatisticsSchema>;
export const scheduleStatisticsSchema = z.object({
    people: z.array(personStatisticsSchema),
    summary: globalStatisticsSchema,
    fitness: scheduleFitnessSchema,
});

/* === Enums === */

export type Holiday = z.infer<typeof holidaySchema>;
export const holidaySchema = z.enum([
    "NearYear",
    "EasterFriday",
    "EasterMonday",
    "AscensionThursday",
    "WhitMonday",
    "NationalDay",
    "JeuneGenevois",
    "Christmas",
    "PublicRestoration",
]);

export type Conflict =
    | { ConsecutiveDay: [number, number, number] }
    | { Holiday: [number, number] }
    | { Role: [number, number] }
    | { WorkCount: [number, number] };

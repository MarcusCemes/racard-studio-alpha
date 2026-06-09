import { format, startOfISOWeek } from "date-fns";

import type { Slots } from "$lib/schemas.js";
import { NULL_SLOT, equalSlots } from "$lib/slot.js";

import {
    DEFAULT_BANK_HOLIDAY_HOURS,
    DEFAULT_WEEKDAY_HOURS,
    N_DAYS,
    N_WEEKDAYS,
    Role,
} from "./defs.js";
import { type ParsedConflict, iterParsedConflict } from "./misc.js";
import type {
    BankHoliday,
    Conflict,
    CustomOverride,
    FitnessWeights,
    OperationKind,
    OperationPhase,
    OperationProgressSummary,
    OperationStatus,
    Person,
    RefinementParameters,
    RefinerProgress,
    ScheduleStatistics,
    SolverParameters,
    SolverProgress,
} from "./schemas.js";

class AppState {
    // --- Problem inputs (raw, editable) ---
    startDate = $state(newStartDate());
    people = $state<Person[]>(newPeople());
    weekdayHours = $state<[number, number][]>(DEFAULT_WEEKDAY_HOURS);
    bankHolidayDefaultHours = $state<[number, number][]>(DEFAULT_BANK_HOLIDAY_HOURS);
    bankHolidays = $state<BankHoliday[]>([]);
    customOverrides = $state<CustomOverride[]>([]);
    skipLastShifts = $state(0);

    // --- Schedule ---
    slots = $state<Slots>(Array.from({ length: N_DAYS }, () => NULL_SLOT));

    // --- Parameters ---
    solverParams = $state<SolverParameters>({
        weekend: {
            number_permutations: 50,
            max_resolve_attempts: 50,
            hill_climb_iterations: 10_000,
        },
        friday: { number_permutations: 1000, max_resolve_attempts: 50 },
        weekday: { number_permutations: 20, max_resolve_attempts: 50 },
    });

    refinerParams = $state<RefinementParameters>({
        cooling_rate: 0.995,
        initial_temperature: 1,
        num_iterations: 10000,
        polish: true,
        searches: 1000,
    });

    weights = $state<FitnessWeights>({
        annual_hours: 5,
        consecutive_days: 20,
        consecutive_weekends: 10,
        weekend_alternation: 1,
        weekend_regularity: 1,
        weekly_hours: 1,
        blank_weeks: 50,
    });

    topK = $state(5);

    // --- Interaction states ---
    activeMode = $state<ActiveMode>("select");

    // --- Operation state (from events only) ---
    activeOp = $state<OperationKind | null>(null);
    operationStatus = $state<OperationStatus | null>(null);
    operationPhase = $state<OperationPhase | null>(null);
    solverProgress = $state<SolverProgress>();
    refinerProgress = $state<RefinerProgress>();
    orchestrationProgress = $state<OperationProgressSummary>();

    // --- Computed results ---
    statistics = $state<ScheduleStatistics>();
    conflicts = $state<Conflict[]>([]);

    // --- History ---
    checkpoints = $state<Checkpoint[]>([]);
    history = new History();

    formattedNames = $derived.by(() =>
        this.people.map((p) => {
            const spaceIdx = p.name.indexOf(" ");

            return spaceIdx === -1
                ? p.name
                : `${p.name.slice(0, spaceIdx)} ${p.name[spaceIdx + 1]}.`;
        }),
    );

    get totalFitness(): number | null {
        return this.statistics?.fitness
            ? Object.values(this.statistics.fitness).reduce((a, b) => a + b, 0)
            : null;
    }

    conflictMap = $derived.by(() => {
        const map: Map<number, ParsedConflict[]> = new Map();

        for (const conflict of this.conflicts) {
            for (const parsedConflict of iterParsedConflict(conflict)) {
                if (map.has(parsedConflict.dayIdx)) {
                    map.get(parsedConflict.dayIdx)!.push(parsedConflict);
                } else {
                    map.set(parsedConflict.dayIdx, [parsedConflict]);
                }
            }
        }

        return map;
    });

    loadSlots = (slots: Slots) => {
        for (let i = 0; i < slots.length; i++) {
            this.slots[i] = slots[i];
        }
    };

    restoreCheckpoint = (checkpoint: Checkpoint) => {
        const { slots } = checkpoint;

        for (let i = 0; i < slots.length; i++) {
            this.slots[i] = slots[i];
        }
    };
}

export class History {
    #past = $state<Slots[]>([]);
    #future = $state<Slots[]>([]);

    // Called right BEFORE mutating the external state
    push(current: Slots) {
        // Optional guard: prevent pushing if no changes were made since last push
        if (this.#past.length > 0 && equalSlots(current, this.#past[this.#past.length - 1])) {
            return;
        }

        this.#past.push([...current]);
        this.#future = []; // A new action invalidates the redo timeline
    }

    undo(current: Slots): Slots | undefined {
        if (!this.canUndo) return;

        // 1. Save the unrecorded present into 'future' so we can redo back to it
        this.#future.push([...current]);

        // 2. Return the most recent past state to apply to your app
        return this.#past.pop();
    }

    redo(current: Slots): Slots | undefined {
        if (!this.canRedo) return;

        // 1. Save the current state into 'past' so we can undo back to it
        this.#past.push([...current]);

        // 2. Return the future state to apply to your app
        return this.#future.pop();
    }

    get canUndo(): boolean {
        return this.#past.length > 0;
    }

    get canRedo(): boolean {
        return this.#future.length > 0;
    }

    get length(): number {
        return this.#past.length;
    }
}

export class Selection {
    day = $state<number>();
    person = $state<number>();
    role = $state<Role>();

    selectPerson = (person?: number) => {
        this.person = person;
    };

    selectSlot = (day?: number, role?: Role) => {
        this.day = day;
        this.role = role;
    };

    get hasSelection(): boolean {
        return this.day !== undefined;
    }

    get selectedWeek(): number | undefined {
        if (this.day === undefined) {
            return;
        }

        return Math.floor(this.day / N_WEEKDAYS);
    }

    get selectedDayOfWeek(): number | undefined {
        if (this.day === undefined) {
            return;
        }

        return this.day % N_WEEKDAYS;
    }
}

export class View {
    gridMode = $state(GridMode.Filled);
    mode = $state(ViewMode.Calendar);
    showConflicts = $state(true);
    showHolidays = $state(true);
    zoom = $state(ZoomLevel.Comfy);
}

let currentApp = $state(new AppState());

export const selection = $state(new Selection());
export const view = $state(new View());

export const app = new Proxy({} as AppState, {
    get(_, prop) {
        const value = Reflect.get(currentApp, prop);

        if (typeof value === "function") {
            return value.bind(currentApp);
        }

        return value;
    },

    set(_, prop, value) {
        return Reflect.set(currentApp, prop, value);
    },
});

export function resetApp() {
    currentApp = new AppState();
}

export interface Checkpoint {
    name: string;
    slots: Slots;
    timestamp: number;
}

export type ActiveMode = "select" | "set" | "swap_role" | "swap_day" | "erase";

export enum GridMode {
    Filled = "filled",
    Bars = "bars",
}

export enum ViewMode {
    Calendar = "standard",
    Weekly = "weekly",
}

export enum ZoomLevel {
    Normal = "normal",
    Comfy = "comfy",
}

function newStartDate(): string {
    return format(startOfISOWeek(new Date()), "yyyy-MM-dd");
}

function newPeople(): Person[] {
    return [
        { holidays: [], name: "Alice", rate: 50 },
        { holidays: [], name: "Bob", rate: 50 },
    ];
}

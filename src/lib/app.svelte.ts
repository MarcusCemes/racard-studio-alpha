import { format, startOfISOWeek } from "date-fns";

import { DEFAULT_BANK_HOLIDAY_HOURS, DEFAULT_WEEKDAY_HOURS, N_DAYS, N_WEEKDAYS } from "./defs.js";
import { parseConflict } from "./misc.js";
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
import { NULL_SLOT, getLead, getSupport, setLead, setSupport } from "./slot.js";

class AppState {
    // --- Problem inputs (raw, editable) ---
    startDate = newStartDate();
    people = $state<Person[]>(newPeople());
    weekdayHours = $state<[number, number][]>(DEFAULT_WEEKDAY_HOURS);
    bankHolidayDefaultHours = $state<[number, number][]>(DEFAULT_BANK_HOLIDAY_HOURS);
    bankHolidays = $state<BankHoliday[]>([]);
    customOverrides = $state<CustomOverride[]>([]);
    skipLastShifts = $state(0);

    // --- Schedule ---
    slots = $state<number[]>(Array.from({ length: N_DAYS }, () => NULL_SLOT));

    // --- Parameters ---
    solverParams = $state<SolverParameters>({
        weekend: { number_permutations: 50, max_resolve_attempts: 50 },
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
    selection = $state<SelectionTarget>({ type: "none" });
    swapSource = $state<SelectionTarget>({ type: "none" });
    activeBrush = $state<number>();
    activeMode = $state<ActiveMode>("select");
    zoomLevel = $state<ZoomLevel>("standard");

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
    history = $state<number[][]>([]);
    historyCursor = $state(0);
    checkpoints = $state<Checkpoint[]>([]);

    formattedNames = $derived.by(() =>
        this.people.map((p) => {
            const spaceIdx = p.name.indexOf(" ");

            return spaceIdx === -1
                ? p.name
                : `${p.name.slice(0, spaceIdx)} ${p.name[spaceIdx + 1]}.`;
        }),
    );

    // --- Derived: fitness ---
    get fitness(): number {
        return this.statistics?.fitness
            ? Object.values(this.statistics.fitness).reduce((a, b) => a + b, 0)
            : 0;
    }

    // --- Derived: holiday lookup map ---
    holidayMap = $derived.by<Record<string, string>>(() => {
        const map: Record<string, string> = {};
        for (const h of this.bankHolidays) {
            if (h.enabled) map[h.date] = h.name;
        }
        return map;
    });

    // --- Derived: conflict lookup by day ---
    conflictMap = $derived.by<Record<number, string[]>>(() => {
        const map: Record<number, string[]> = {};

        for (const conflict of this.conflicts) {
            const parsedConflict = parseConflict(conflict);

            if (parsedConflict.type === "day") {
                if (!map[parsedConflict.index]) map[parsedConflict.index] = [];
                map[parsedConflict.index].push(parsedConflict.message);
            }
        }

        return map;
    });

    // --- Clean mutations for the grid ---
    setRole(dayIndex: number, role: "lead" | "support", personId: number | undefined) {
        const current = this.slots[dayIndex];
        if (role === "lead") {
            this.slots[dayIndex] = setLead(current, personId);
        } else {
            this.slots[dayIndex] = setSupport(current, personId);
        }
    }

    swapDays(idxA: number, idxB: number) {
        const temp = this.slots[idxA];
        this.slots[idxA] = this.slots[idxB];
        this.slots[idxB] = temp;
    }

    swapRoles(idxA: number, roleA: "lead" | "support", idxB: number, roleB: "lead" | "support") {
        const valA = this.getRoleValue(idxA, roleA);
        const valB = this.getRoleValue(idxB, roleB);

        this.setRole(idxA, roleA, valB);
        this.setRole(idxB, roleB, valA);
    }

    private getRoleValue(dayIndex: number, role: "lead" | "support"): number | undefined {
        const current = this.slots[dayIndex];
        return role === "lead" ? getLead(current) : getSupport(current);
    }

    isSelected(dayIndex: number, role?: "lead" | "support"): boolean {
        const sel = this.selection;
        if (sel.type === "none") return false;
        if (role) {
            return sel.type === "role" && sel.dayIndex === dayIndex && sel.role === role;
        }
        return sel.type === "day" && sel.dayIndex === dayIndex;
    }

    isSwapSource(dayIndex: number, role?: "lead" | "support"): boolean {
        const src = this.swapSource;
        if (src.type === "none") return false;
        if (role) {
            return src.type === "role" && src.dayIndex === dayIndex && src.role === role;
        }
        return src.type === "day" && src.dayIndex === dayIndex;
    }

    get selectedWeek(): number | undefined {
        if (this.selection.type === "none") return undefined;
        return Math.floor(this.selection.dayIndex / N_WEEKDAYS);
    }

    get selectedDayOfWeek(): number | undefined {
        if (this.selection.type === "none") return undefined;
        return this.selection.dayIndex % N_WEEKDAYS;
    }

    restoreCheckpoint = (cp: Checkpoint) => {
        this.slots = cp.slots;
    };
}

let currentApp = $state(new AppState());

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
    slots: number[];
    timestamp: number;
}

export type ActiveMode = "select" | "set" | "swap_role" | "swap_day" | "erase";
export type ZoomLevel = "micro" | "standard" | "detail";

export type SelectionTarget =
    | { type: "none" }
    | { type: "day"; dayIndex: number }
    | { type: "role"; dayIndex: number; role: "lead" | "support" };

function newStartDate(): string {
    return format(startOfISOWeek(new Date()), "yyyy-MM-dd");
}

function newPeople(): Person[] {
    return [
        { holidays: [], name: "Alice", rate: 50 },
        { holidays: [], name: "Bob", rate: 50 },
    ];
}

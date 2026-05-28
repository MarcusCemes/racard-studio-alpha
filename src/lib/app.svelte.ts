import { format, startOfISOWeek } from "date-fns";

import { NULL_ID, NULL_SLOT, N_DAYS, N_WEEKDAYS } from "./defs.js";
import { parseConflict } from "./misc.js";
import type {
    Conflict,
    OrchestrationProgress,
    Person,
    RefinerProgress,
    SolverProgress,
} from "./schemas.js";

class AppState {
    startDate = newStartDate();
    people = $state(newPeople());
    slots = $state<number[]>(sampleSlots());

    // Cleaned Interaction states
    selection = $state<SelectionTarget>({ type: "none" });
    swapSource = $state<SelectionTarget>({ type: "none" });
    activeBrush = $state<number>();

    // Sparse raw collections
    holidays = $state<{ date: string; name: string }[]>([]); // Sparse holiday list
    conflicts = $state<Conflict[]>([]); // Sparse conflict list

    activeMode = $state<ActiveMode>("select");
    zoomLevel = $state<ZoomLevel>("standard");

    // History & Solvers (kept intact)
    history = $state<number[][]>([]);
    historyCursor = $state(0);
    checkpoints = $state<Checkpoint[]>(sampleCheckpoints());
    solverActive = $state(false);
    solverPopulation = $state(500);
    solverProgress = $state<SolverProgress>();
    refinerActive = $state(false);
    refinerProgress = $state<RefinerProgress>();
    refinerRounds = $state(500);
    orchestratorActive = $state(false);
    orchestratorProgress = $state<OrchestrationProgress>();

    // 2. High-Performance Derived O(1) Lookups
    // Map of date string -> holiday name
    holidayMap = $derived.by<Record<string, string>>(() => {
        const map: Record<string, string> = {};
        for (const h of this.holidays) {
            map[h.date] = h.name;
        }
        return map;
    });

    // Map of dayIndex -> Conflicts list
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

    // Clean mutations for the grid to call directly
    setRole(dayIndex: number, role: "lead" | "support", personId: number | undefined) {
        const current = this.slots[dayIndex];
        let lead = current & 0xf;
        let supp = current >> 4;

        const val = personId ?? NULL_SLOT;
        if (role === "lead") lead = val;
        else supp = val;

        this.slots[dayIndex] = (supp << 4) | lead;
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
        const val = role === "lead" ? current & 0xf : current >> 4;
        return val !== NULL_ID ? val : undefined;
    }

    // Helpers to quickly evaluate selection state
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

    get solverCompletion(): number {
        const { accepted, rejected } = this.solverProgress?.[0] ?? { accepted: 0, rejected: 0 };
        return (accepted + rejected) / this.solverPopulation;
    }
}

export const app: AppState = new AppState();

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

function sampleSlots(): number[] {
    return Array.from({ length: N_DAYS }, (_, i) => i % 256);
}

function sampleCheckpoints(): Checkpoint[] {
    return [
        {
            name: "Checkpoint 1",
            slots: Array.from({ length: N_DAYS }, () => NULL_SLOT),
            timestamp: Date.now(),
        },
    ];
}

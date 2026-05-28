import { format, startOfISOWeek } from "date-fns";

import { NULL_SLOT, N_DAYS, N_WEEKDAYS, PERSON_COLORS, PERSON_TEXT_COLORS } from "./defs.js";
import type { Conflict, Person, RefinerProgress, SolverProgress } from "./schemas.js";

class AppState {
    startDate = newStartDate();
    people = $state(newPeople());
    slots = $state<number[]>(sampleSlots());

    selectedDay = $state<number>();
    selectedPersonIndex = $state<number>();

    history = $state<number[][]>([]);
    historyCursor = $state(0);

    checkpoints = $state<Checkpoint[]>(sampleCheckpoints());
    conflicts = $state<Conflict[]>([]);

    activeMode = $state<ActiveMode>("select");
    zoomLevel = $state<ZoomLevel>("standard");

    solverActive = $state(false);
    solverPopulation = $state(500);
    solverProgress = $state<SolverProgress>();

    refinerActive = $state(false);
    refinerProgress = $state<RefinerProgress>();
    refinerRounds = $state(500);

    restoreCheckpoint = (cp: Checkpoint) => {
        this.slots = cp.slots;
    };

    get selectedPerson(): [Person, string, string, string] | undefined {
        let personIdx = this.selectedPersonIndex ?? -1;
        const person = this.people[personIdx];
        if (!person) return undefined;

        let wrappedIdx = personIdx % PERSON_COLORS.length;
        const [bg0, bg1] = PERSON_COLORS[wrappedIdx]!;
        const text = PERSON_TEXT_COLORS[wrappedIdx]!;

        return [person, bg0, bg1, text];
    }

    get selectedWeek(): number | undefined {
        return this.selectedDay && Math.floor(this.selectedDay / N_WEEKDAYS);
    }

    get selectedDayOfWeek(): number | undefined {
        return this.selectedDay && this.selectedDay % N_WEEKDAYS;
    }

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

/* === Miscellaneous === */

function newStartDate(): string {
    return format(startOfISOWeek(new Date()), "yyyy-MM-dd");
}

function newPeople(): Person[] {
    return [
        { holidays: [], name: "Alice", rate: 50 },
        { holidays: [], name: "Bob", rate: 50 },
    ];
}

function newSlots(): number[] {
    return Array.from({ length: N_DAYS }, () => NULL_SLOT);
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
        {
            name: "Checkpoint 2",
            slots: Array.from({ length: N_DAYS }, () => 0),
            timestamp: Date.now(),
        },
    ];
}

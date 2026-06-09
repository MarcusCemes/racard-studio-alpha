import { N_WEEKDAYS } from "./defs.js";
import type { Conflict } from "./schemas.js";

export interface ParsedConflict {
    dayIdx: number;
    kind: ConflictKind;
    otherDayIdx?: number;
    personIdx: number;
}

export enum ConflictKind {
    ConsecutiveDay,
    Holiday,
    Role,
    WorkCount,
}

export function* iterParsedConflict(conflict: Conflict): Generator<ParsedConflict> {
    if ("ConsecutiveDay" in conflict) {
        const [personIdx, dayIdx] = conflict.ConsecutiveDay;

        for (let i = 0; i < 2; ++i) {
            yield {
                dayIdx: dayIdx + i,
                kind: ConflictKind.ConsecutiveDay,
                otherDayIdx: dayIdx + (i === 0 ? 1 : -1),
                personIdx,
            };
        }
    } else if ("Holiday" in conflict) {
        const [personIdx, dayIdx] = conflict.Holiday;

        yield {
            dayIdx,
            kind: ConflictKind.Holiday,
            personIdx,
        };
    } else if ("Role" in conflict) {
        const [personIdx, dayIdx] = conflict.Role;

        yield {
            dayIdx,
            kind: ConflictKind.Role,
            personIdx,
        };
    } else if ("WorkCount" in conflict) {
        const [personIdx, weekIdx] = conflict.WorkCount;

        for (let i = 0; i < 4; ++i) {
            yield {
                dayIdx: weekIdx * N_WEEKDAYS + i,
                kind: ConflictKind.WorkCount,
                personIdx,
            };
        }
    } else {
        throw new Error(`Unknown conflict type: ${JSON.stringify(conflict)}`);
    }
}

function conflictMessage(conflict: ParsedConflict): string {
    const person =
        conflict.personIdx !== undefined ? app.people[conflict.personIdx].name : undefined;
    const other =
        conflict.otherDayIdx && format(addDays(app.startDate, conflict.otherDayIdx), "EEEE");

    switch (conflict.kind) {
        case ConflictKind.ConsecutiveDay:
            return `Consecutive workday with ${other}`;

        case ConflictKind.Holiday:
            return `${person} is on holiday`;

        case ConflictKind.Role:
            return `${person} works both roles`;

        case ConflictKind.WorkCount:
            return `${person} works too many days`;
    }
}

export function plural(n: number, singular: string, plural?: string): string {
    return n === 1 ? singular : (plural ?? `${singular}s`);
}

export function isModifierKeyPressed(event: KeyboardEvent) {
    return ["Shift", "Alt", "AltGraph", "Control", "Meta", "CapsLock", "Fn", "FnLock"].some((key) =>
        event.getModifierState(key),
    );
}

export function timestamp(): number {
    return Math.floor(new Date().getTime() / 1000);
}

export function dateFromTimestamp(timestamp: number): Date {
    return new Date(timestamp * 1000);
}

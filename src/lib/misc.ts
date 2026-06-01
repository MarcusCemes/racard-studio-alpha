import { N_WEEKDAYS, WEEKDAYS } from "./defs.js";
import type { Conflict } from "./schemas.js";

export interface ParsedConflict {
    type: "ConsecutiveDay" | "Holiday" | "Role" | "WorkCount";
    personIdx: number;
    description: string;
    scope: "day" | "week";
    scopeIndex: number;
}

export function parseConflict(
    conflict: Conflict,
    formatDay: (dayIdx: number) => string,
): ParsedConflict {
    if ("ConsecutiveDay" in conflict) {
        const [person, dayA, dayB] = conflict.ConsecutiveDay;
        return {
            type: "ConsecutiveDay",
            personIdx: person,
            description: `${formatDay(dayA)} → ${formatDay(dayB)}`,
            scope: "day",
            scopeIndex: dayA,
        };
    } else if ("Holiday" in conflict) {
        const [person, day] = conflict.Holiday;
        return {
            type: "Holiday",
            personIdx: person,
            description: formatDay(day),
            scope: "week",
            scopeIndex: Math.floor(day / N_WEEKDAYS),
        };
    } else if ("Role" in conflict) {
        const [person, day] = conflict.Role;
        return {
            type: "Role",
            personIdx: person,
            description: formatDay(day),
            scope: "day",
            scopeIndex: day,
        };
    } else if ("WorkCount" in conflict) {
        const [person, week] = conflict.WorkCount;
        return {
            type: "WorkCount",
            personIdx: person,
            description: `Week ${week + 1}`,
            scope: "week",
            scopeIndex: week,
        };
    }

    throw new Error("Unknown conflict type");
}

export function formatDayIdx(dayIdx: number): string {
    const week = Math.floor(dayIdx / N_WEEKDAYS) + 1;
    const day = WEEKDAYS[dayIdx % N_WEEKDAYS];
    return `${day} Wk ${week}`;
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

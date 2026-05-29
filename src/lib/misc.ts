import type { Conflict } from "./schemas.js";

export function parseConflict(conflict: Conflict): {
    type: "day" | "week";
    index: number;
    message: string;
} {
    if ("ConsecutiveDay" in conflict) {
        return {
            type: "day",
            index: conflict.ConsecutiveDay[0],
            message: `Consecutive day conflict: ${conflict.ConsecutiveDay[0]} - ${conflict.ConsecutiveDay[1]} (${conflict.ConsecutiveDay[2]} hours)`,
        };
    } else if ("Holiday" in conflict) {
        return {
            type: "day",
            index: conflict.Holiday[0],
            message: `Holiday conflict: ${conflict.Holiday[0]} - ${conflict.Holiday[1]}`,
        };
    } else if ("Role" in conflict) {
        return {
            type: "day",
            index: conflict.Role[0],
            message: `Role conflict: ${conflict.Role[0]} - ${conflict.Role[1]}`,
        };
    } else if ("WorkCount" in conflict) {
        return {
            type: "week",
            index: conflict.WorkCount[0],
            message: `Work count conflict: ${conflict.WorkCount[0]} - ${conflict.WorkCount[1]}`,
        };
    }

    throw new Error("Unknown conflict type");
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

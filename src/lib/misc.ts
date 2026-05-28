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

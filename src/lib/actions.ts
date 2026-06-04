import { app } from "$lib/app.svelte.js";

import type { Role } from "./defs.js";
import { getRole, setRole } from "./slot.js";

export function swapDays(a: number, b: number) {
    app.history.push(app.slots);

    const tmp = app.slots[a];
    app.slots[a] = app.slots[b];
    app.slots[b] = tmp;
}

export function swapRoles(dayA: number, dayB: number, roleA: Role, roleB: Role) {
    app.history.push(app.slots);

    const slotA = app.slots[dayA];
    const slotB = app.slots[dayB];

    const userA = getRole(slotA, roleA);
    const userB = getRole(slotB, roleB);

    app.slots[dayA] = setRole(slotA, roleA, userB);
    app.slots[dayB] = setRole(slotB, roleB, userA);
}

export function setPerson(day: number, role: Role, person?: number) {
    app.history.push(app.slots);
    app.slots[day] = setRole(app.slots[day], role, person);
}

/* === History === */

export function undo() {
    if (!app.history.canUndo) return;

    const slots = app.history.undo(app.slots);

    if (slots) {
        app.loadSlots(slots);
    }
}

export function redo() {
    if (!app.history.canRedo) return;

    const slots = app.history.redo(app.slots);

    if (slots) {
        app.loadSlots(slots);
    }
}

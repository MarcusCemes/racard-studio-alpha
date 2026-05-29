/**
 * Centralized slot byte encoding.
 *
 * Layout (matching Rust's `algorithm::types::Slot`):
 *   Bits 7-4 (high nibble) = lead person ID
 *   Bits 3-0 (low nibble)  = support person ID
 *
 * Sentinel values:
 *   NULL_ID   = 0xF — "no person" for a single role nibble
 *   NULL_SLOT = 0xFF — "no person" for the entire slot (both roles empty)
 */
import { N_DAYS, Role } from "./defs.js";
import type { Slot } from "./schemas.js";

export const NULL_ID = 0xf as Slot;
export const NULL_SLOT = 0xff as Slot;

/** Extract the lead person ID from a packed slot. Returns undefined if empty. */
export function getLead(slot: number): number | undefined {
    const val = slot >> 4;
    return val !== NULL_ID ? val : undefined;
}

/** Extract the support person ID from a packed slot. Returns undefined if empty. */
export function getSupport(slot: number): number | undefined {
    const val = slot & 0xf;
    return val !== NULL_ID ? val : undefined;
}

export function getRole(slot: number, role: Role): number | undefined {
    return role === Role.Lead ? getLead(slot) : getSupport(slot);
}

/** Return a new slot with the lead role set. Pass undefined to clear. */
export function setLead(slot: Slot, personId: number | undefined): Slot {
    const id = personId ?? NULL_ID;
    return ((slot & 0xf) | (id << 4)) as Slot;
}

/** Return a new slot with the support role set. Pass undefined to clear. */
export function setSupport(slot: Slot, personId: number | undefined): Slot {
    const id = personId ?? NULL_ID;
    return ((slot & 0xf0) | id) as Slot;
}

export function setRole(slot: Slot, role: Role, personId: number | undefined): Slot {
    return role === Role.Lead ? setLead(slot, personId) : setSupport(slot, personId);
}

/** Create a packed slot from optional lead and support person IDs. */
export function makeSlot(lead: number | undefined, support: number | undefined): Slot {
    return setSupport(setLead(NULL_SLOT, lead), support);
}

export function equalSlots(slotsA: Slots, slotsB: Slots): boolean {
    for (let i = 0; i < N_DAYS; i++) {
        if (slotsA[i] !== slotsB[i]) {
            return false;
        }
    }

    return true;
}

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

export const NULL_ID = 0xf;
export const NULL_SLOT = 0xff;

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

/** Return a new slot with the lead role set. Pass undefined to clear. */
export function setLead(slot: number, personId: number | undefined): number {
    const id = personId ?? NULL_ID;
    return (slot & 0x0f) | (id << 4);
}

/** Return a new slot with the support role set. Pass undefined to clear. */
export function setSupport(slot: number, personId: number | undefined): number {
    const id = personId ?? NULL_ID;
    return (slot & 0xf0) | id;
}

/** Create a packed slot from optional lead and support person IDs. */
export function makeSlot(lead: number | undefined, support: number | undefined): number {
    return setSupport(setLead(0, lead), support);
}

/** True if the entire slot is empty (both roles unset). */
export function isNullSlot(slot: number): boolean {
    return slot === NULL_SLOT;
}

# 🏥 Shift Scheduling Algorithm - Agent Context

## 1. Domain Overview

Generates a 48-week (336 day) schedule for 2–15 workers, 2 shifts per day (Lead/Support). Each worker has a pre-defined rate (5–100%, multiples of 5) and 4 weeks of holiday blackouts.

**Terminology:**

- **Lead** (formerly "Staff"): Primary/night shift (14.5h std, 13.5h Mon, 20h Sat)
- **Support** (formerly "Double"): Secondary/day shift (7.5h std, 5.5h Tue, 5h Sun)
- **Phantom Shifts**: Last 3 shifts (Sun Support, Sun Lead, Sat Support) are assigned by solver but masked to 0 hours in `ScheduleEvaluator`

## 2. Hard Constraints (Invalid if Violated)

1. **Single Shift per Day**: Cannot work both Lead and Support on same day
2. **Holiday Blackout**: No work during holiday week (Mon-Sun)
3. **Holiday Margins**: Cannot work weekend before holiday, nor Lead on preceding Friday
4. **Rest Period**: Lead → Support forbidden next day (EXCEPT Fri→Sat, Sat→Sun)
5. **Weekend Pairs**: Saturday Lead/Support swap roles on Sunday (Person A Lead Sat + Person B Support Sat → Person A Support Sun + Person B Lead Sun)
6. **No Friday Before Weekend**: Weekend workers cannot work preceding Friday (any role)
7. **Weekend Exhaustion**: Max 1 weekday shift (Mon-Thu) if working that weekend

## 3. ⚠️ AI Agent Directives

- **Do NOT alter Generation Order**: Weekends → Fridays → Weekdays (see `ARCHITECTURE.md`)
- **Perturbations must be self-inverse**: Allow quick rollback in Refiner
- **Sunday is derived**: Always `saturday.swapped()` — never modify directly
- **Phantom Shifts**: Keep in core data structures; evaluator masks them to 0 hours
- **Bank Holidays**: Handled via `ProblemOverrides` (date → hour overrides), NOT in solver logic
- **Slot bit-packing**: `u8` format: high nibble = Lead, low nibble = Support. Nibble sentinel `0xF` = unassigned person; full-byte `0xFF` = unassigned slot (`Slot::NULL`)

## 4. Repository Structure

```
algorithm/          → Rust crate (edition 2024)
  src/
    solver/         → Constructive heuristic (weekend → friday → weekday)
      weekend.rs    → GRASP + mini hill-climb
      friday.rs     → Build around locked weekends
      weekday.rs   → Fill Mon-Thu
    refiner.rs      → Simulated Annealing (4 perturbation types)
    fitness.rs      → ScheduleEvaluator (7 weighted components)
    validate.rs     → ScheduleValidator (4 conflict types)
    holiday.rs      → Geneva bank holiday calculation
    types.rs        → Slot, DayIdx, WeekIdx, PersonIdx, Solution
    defs.rs         → Constants (N_DAYS=336, MAX_PEOPLE=15, N_WEEKS=48)

src/                → SvelteKit frontend (viewer, settings components)
src-tauri/          → Tauri backend (commands.rs, PDF export, tauri.conf.json)
```

## 5. Fitness Weights (Lower = Better)

```rust
STANDARD: {
    annual_hours: 5.0,
    consecutive_days: 20.0,
    consecutive_weekends: 10.0,
    weekend_alternation: 1.0,
    weekend_regularity: 1.0,
    weekly_hours: 1.0,
    blank_weeks: 50.0,
}
```

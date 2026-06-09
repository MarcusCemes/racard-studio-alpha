# 🏥 Shift Scheduling Algorithm - Architecture

## 1. System Overview

The scheduler uses a **hybrid Constructive Heuristic + Metaheuristic Refiner** approach:

```
ProblemInput (people, start_date, holidays, overrides)
    ↓
┌──────────────────────────────────────────────┐
│  Constructive Heuristic (Solver)             │
│  1. WeekendSolver (GRASP + hill-climb)     │
│  2. FridaySolver (build around weekends)     │
│  3. WeekdaySolver (fill Mon-Thu)           │
│                                              │
│  → Produces valid DraftSchedule             │
└──────────────────────────────────────────────┘
    ↓
┌──────────────────────────────────────────────┐
│  Refiner (Simulated Annealing)               │
│  - 4 perturbation types (self-inverse)      │
│  - Polish pass (greedy hour balancing)      │
│                                              │
│  → Optimized Solution (lower fitness = better)│
└──────────────────────────────────────────────┘
```

**Why this order?** Friday constraints heavily restrict weekends. By locking weekends first, we reduce the search space for subsequent stages.

---

## 2. Data Structures

### 2.1 Slot (Bit-Packed `u8`)

```rust
struct Slot(u8);
// High nibble (bits 7-4): Lead person index (0-14, or 0xF = unassigned)
// Low nibble (bits 3-0): Support person index (0-14, or 0xF = unassigned)
// Full byte: 0xFF = Slot::NULL (entire slot unassigned)
```

Key operations:

- `Slot::new(lead, support)` → packs into `u8`
- `slot.get(Role::Lead)` → extracts high nibble
- `slot.swapped()` → `self.0.rotate_right(4)` (swap nibbles)
- `slot.is_valid()` → true if Lead ≠ Support and both assigned
- `Slot::NULL` → constant `0xFF` (unassigned slot)
- `slot.overlaps(other)` → true if any person appears in both slots

### 2.2 Index Types (Type-Safe Wrappers)

```rust
struct DayIdx(NonMaxU16);   // 0..336 (N_DAYS)
struct WeekIdx(NonMaxU8);   // 0..48  (N_WEEKS)
struct PersonIdx(NonMaxU8); // 0..15  (MAX_PEOPLE, variable count)
```

All use `NonMax*` to ensure indices stay within valid ranges. Invalid values (0xFF for `u8`) represent "unassigned".

### 2.3 Solution Layout

```rust
struct Solution(Box<SlotArray>);
// SlotArray = [Slot; 336] where index = day_index
// Days 0-6 = Week 0 (Mon-Sun), 7-13 = Week 1, etc.
```

### 2.4 Hour Assignments

```rust
struct HourAssignments(Box<[[f32; 2]; N_DAYS]>);
// [f32; 2] = [lead_hours, support_hours] for each day
// Generated from start_date, with overrides applied
// Last N shifts zeroed for phantom shifts
```

---

## 3. Solver Architecture

### 3.1 WeekendSolver (GRASP Algorithm)

**Goal**: Assign 48 Saturday-Sunday pairs (96 slots, but 3 are phantom).

**GRASP Steps**:

1. **Shuffle** week order randomly
2. **Sort by DOF** (Degree of Freedom - most constrained first, stable sort preserves shuffle for ties)
3. **Generate distribution** proportionate to worker rates:
    ```rust
    // Example with rates [80, 80, 75, 70, 60, 60, 55, 40]:
    // Lead counts:   [8, 8, 7, 7, 5, 5, 5, 3]  (sum=48)
    // Support counts: [7, 7, 7, 6, 6, 6, 5, 4]  (sum=48)
    ```
4. **Fill slots** using distribution counts, checking holiday masks
5. **Resolve conflicts** (swaps) with max attempts
6. **Mini Hill-Climb** (configurable via `hill_climb_iterations`, default 10k):
    - Randomly pick two weeks and a role
    - Swap assignments if it improves combined `weekend_regularity` + `weekend_alternation` score
    - Accept equal scores (walk across plateaus)
    - Higher values produce flatter, more regular distributions at the cost of runtime

**Key**: Sunday is ALWAYS `saturday.swapped()` — never directly assigned.

### 3.2 FridaySolver

**Goal**: Assign 48 Fridays after weekends are locked.

**Strategy**:

1. Generate permutation based on weekend mask (people already working Saturday can't work Friday)
2. Fill Fridays ensuring:
    - No overlap with next day's Saturday workers
    - No Lead on Friday before someone's holiday week
3. Resolve conflicts via swaps

### 3.3 WeekdaySolver

**Goal**: Fill Monday-Thursday slots (4 days × 48 weeks = 192 slots).

**Strategy**:

- Build around locked weekends + Fridays
- Respect "max 1 weekday shift if working weekend" constraint
- Generate valid permutations

### 3.4 Solver Parameters

Separate parameter types per phase — `WeekendParameters` includes the hill-climb iteration count; `WeekdayParameters` (used for both Friday and weekday phases) does not:

```rust
pub struct WeekendParameters {
    pub number_permutations: u64,
    pub max_resolve_attempts: u64,
    pub hill_climb_iterations: u64,
}

pub struct WeekdayParameters {
    pub number_permutations: u64,
    pub max_resolve_attempts: u64,
}

pub struct SolverParameters {
    pub weekend: WeekendParameters,
    pub friday: WeekdayParameters,
    pub weekday: WeekdayParameters,
}
```

Preset profiles:

| Profile | Weekend perms | Hill climb | Friday perms | Weekday perms |
| ------- | ------------- | ---------- | ------------ | ------------- |
| FAST    | 50            | 10,000     | 100          | 50            |
| SLOW    | 100           | 50,000     | 10,000       | 100           |

### 3.5 Parallel Execution

```rust
// Solver::execute() spawns threads
let threads = num_cpus::get();
// Each thread runs Worker::spin() which:
//   1. Generates weekend permutations (up to number_permutations)
//   2. For each weekend, generates Friday permutations
//   3. For each Friday, generates weekday assignments
//   4. Evaluates fitness, keeps best solution
```

Uses `AtomicU64` counter for work distribution across threads.

### 3.6 Weekends-Only Mode

`Solver::execute_weekends()` runs only the weekend solver, leaving Fridays and weekdays as `Slot::NULL`. This is useful for finding a flat, regular weekend distribution without the constraints of Friday/weekday placement.

- Uses `spin_weekends_only()` which skips Friday/Weekday solvers entirely
- Fitness = `weekend_regularity × weight` + `weekend_alternation × weight` only
- Returns a full `Solution` with weekends filled, all other slots zeroed
- Exposed as a separate Tauri command (`weekend_solve`) and UI trigger ("Solve (weekends)")
- Performance is orders of magnitude faster than the full solver (no multiplicative nesting), allowing much higher `hill_climb_iterations` values

---

## 4. Refiner (Simulated Annealing)

### 4.1 Algorithm

```rust
for _ in 0..searches {
    temperature = initial_temperature;
    for _ in 0..num_iterations {
        perturbation = generate_valid_perturbation();
        new_fitness = evaluate(solution);
        delta = current_fitness - new_fitness;

        if delta > 0 || rng.random() < (delta / temperature).exp() {
            accept(perturbation);  // Update best if improved
        } else {
            perturbation.revert();  // Self-inverse: reapply to undo
        }

        temperature *= cooling_rate;
    }
}
```

**Parameters**:

- `FAST`: temp=5.0, cooling=0.995, iterations=10k, searches=100
- `SLOW`: temp=20.0, cooling=0.9995, iterations=20k, searches=200

### 4.2 Perturbation Types (All Self-Inverse)

| Type                                   | Description                        | Inverse                         |
| -------------------------------------- | ---------------------------------- | ------------------------------- |
| `ReplacePerson(day, role, old_person)` | Replace person in role             | Re-apply with stored old_person |
| `SwapDays(a, b)`                       | Exchange complete slot assignments | Same operation (swap again)     |
| `SwapRoles(day)`                       | Swap Lead↔Support in slot          | Same operation                  |
| `SwapRoleBetweenDays(a, b, role)`      | Swap one role between two days     | Same operation                  |

**Critical**: Sunday is derived from Saturday. All perturbations affecting Saturday auto-update Sunday via `saturday_with_next()`.

### 4.3 Polish Pass (Greedy)

After SA, runs targeted optimization:

1. Calculate annual hour deltas per person
2. Sort by overworked (highest positive delta first)
3. For each overworked person, find underworked person
4. Try to move a shift (Mon-Thu only, excludes weekends/Fridays)
5. Accept if it reduces absolute drift sum
6. Repeat until no improvement found

---

## 5. Fitness Function

```rust
struct ScheduleFitness {
    annual_hours: f32,         // |Σ(actual - target)| across all people
    consecutive_days: f32,     // Count of Lead→Support sequences
    consecutive_weekends: f32, // Count of Sat→Sat person overlaps
    weekend_alternation: f32,  // Count of same-role-repeated weekends (with wrap-around)
    weekend_regularity: f32,   // Variance of gap spacing between weekends per person
    weekly_hours: f32,         // RMSE of weekly hours vs target
    blank_weeks: f32,          // Count of weeks with 0 shifts (non-holiday)
}

total = Σ(weight * component)
```

**Details**:

- **Annual Hours**: Absolute drift summed across all people
- **Weekly Hours**: RMSE = sqrt(Σ(weekly_delta²) / 48) summed across people
- **Weekend Regularity**: For each person, calculate sum(gap²) - (48² / count). Minimum is zero when weekends are perfectly evenly spaced.
- **Weekend Alternation**: Counts consecutive weekends where the same person works the same role (Lead-Lead or Support-Support), including the wrap-around from year-end to year-start. Lower is better.
- **Blank Weeks**: Penalizes weeks where person has 0 shifts (but not on holiday)

---

## 6. Validation (4 Conflict Types)

```rust
enum Conflict {
    ConsecutiveDay(PersonIdx, DayIdx),  // Lead day X → Support day X+1
    Holiday(PersonIdx, DayIdx),                 // Working during holiday week
    Role(PersonIdx, DayIdx),                    // Lead == Support in same slot
    WorkCount(PersonIdx, WeekIdx),              // >1 weekday shift + weekend work
}
```

**Validator checks**:

1. **Consecutive Days**: Lead on day X + Support on day X+1 (excludes Sat→Sun)
2. **Holidays**: Work during holiday week, or Lead on Friday before holiday
3. **Role Conflict**: Same person in both Lead and Support
4. **Work Count**: >1 weekday shift (Mon-Thu) when working that weekend

---

## 7. Hour Calculation

### 7.1 Standard Hours

| Day | Lead Hours | Support Hours |
| --- | ---------- | ------------- |
| Mon | 13.5h      | 7.5h          |
| Tue | 14.5h      | 5.5h          |
| Wed | 14.5h      | 7.5h          |
| Thu | 14.5h      | 7.5h          |
| Fri | 14.5h      | 7.5h          |
| Sat | 20h        | 7.5h          |
| Sun | 14.5h      | 5h            |

### 7.2 Override Mechanism

```rust
struct ProblemOverrides(Box<[(NaiveDate, [f32; 2])]>);
// Maps specific dates to custom [lead_hours, support_hours]
// Applied on top of standard hours in HourAssignments::from_date_overrides()
```

**Phantom Shifts**: Last 3 shifts (Sun Support, Sun Lead, Sat Support) have hours zeroed in `HourAssignments`. The `Solution` still contains assignments, but they contribute 0 hours in fitness calculations.

### 7.3 Target Hours

```rust
// 100% rate = 33 hours/week
weekly_hours = 33 * (rate / 100)
// Examples: 80% → 26.4h, 40% → 13.2h
```

---

## 8. Holiday Handling

### 8.1 Input Structure

```rust
struct ProblemInput {
    start_date: NaiveDate,           // Must be a Monday
    people: Vec<Person>,             // Each has .holidays: Vec<u8> (week indices)
    overrides: ProblemOverrides,      // Date → hour overrides
    skip_last_shifts: u8,            // Number of phantom shifts (usually 3)
}
```

### 8.2 Context Generation

```rust
struct Context {
    n_people: usize,                       // Actual worker count (2..MAX_PEOPLE)
    people: [ContextPerson; MAX_PEOPLE],  // Includes work_share; dummy entries beyond n_people
    holidays: WeeklyMask,                  // [u16; 48] bitmask per week, 1 bit per person
}

// WeeklyMask: array of u16 bitmasks, one per week
// Each bit N = 1 means person N is on holiday that week
// Used by solver to check: holidays.get(week, person)
```

### 8.3 Holiday Constraints

1. **Blackout**: No work Mon-Sun of holiday week
2. **Weekend before**: No work on Sat/Sun preceding holiday week
3. **Friday margin**: No Lead on Friday before holiday week (Support allowed)

---

## 9. Bank Holidays (Geneva)

Bank holidays are **NOT** special-cased in the solver. Instead, they're handled via `ProblemOverrides`:

```rust
// algorithm/src/holiday.rs
fn geneva_bank_holidays(dates: Range<NaiveDate>) -> impl Iterator<Item = (NaiveDate, Holiday)> {
    // Returns: New Year, Easter Friday, Easter Monday, Ascension Thursday,
    //          Whit Monday, National Day (Aug 1), Jeune Genevois,
    //          Christmas, Public Restoration (Dec 31)
}
```

These dates are converted to `ProblemOverrides` by the UI/input layer, setting custom hours for those dates. The solver sees them as just another date override.

---

## 10. Key Files Reference

| File                              | Purpose                                                    |
| --------------------------------- | ---------------------------------------------------------- |
| `algorithm/src/types.rs`          | Slot, DayIdx, WeekIdx, PersonIdx, Solution, FitnessWeights |
| `algorithm/src/defs.rs`           | Constants, Person, ProblemInput, ProblemOverrides, Rate    |
| `algorithm/src/solver/mod.rs`     | Solver orchestration, parallel execution                   |
| `algorithm/src/solver/weekend.rs` | GRASP + hill-climb for weekends                            |
| `algorithm/src/solver/friday.rs`  | Friday assignment around weekends                          |
| `algorithm/src/solver/weekday.rs` | Mon-Thu filling                                            |
| `algorithm/src/refiner.rs`        | Simulated Annealing, perturbations, polish                 |
| `algorithm/src/fitness.rs`        | ScheduleEvaluator, 7 fitness components                    |
| `algorithm/src/validate.rs`       | ScheduleValidator, 4 conflict types                        |
| `algorithm/src/holiday.rs`        | Geneva bank holiday computation                            |
| `algorithm/src/lib.rs`            | Public API exports                                         |

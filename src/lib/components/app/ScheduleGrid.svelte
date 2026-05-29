<script lang="ts">
    import { addDays, addWeeks, format, getDate, parseISO, startOfISOWeek } from "date-fns";

    import { app } from "$lib/app.svelte.js";
    import { N_DAYS, N_WEEKDAYS, N_WEEKS, PERSON_COLORS, WEEKDAYS } from "$lib/defs.js";
    import { getLead, getSupport } from "$lib/slot.js";
    import { cn } from "$lib/utils.js";

    // Structural Geometry - Only recalculated if startDate shifts
    interface DayTopology {
        index: number;
        date: Date;
        dateString: string;
        isWeekend: boolean;
    }

    interface WeekTopology {
        weekIndex: number;
        weekNumber: number;
        mondayDate: Date;
        days: DayTopology[];
    }

    const weeks = $derived.by<WeekTopology[]>(() => {
        const baseDate = startOfISOWeek(parseISO(app.startDate));
        const result: WeekTopology[] = [];

        for (let w = 0; w < N_WEEKS; w++) {
            const monday = addWeeks(baseDate, w);
            const days: DayTopology[] = [];

            for (let d = 0; d < N_WEEKDAYS; d++) {
                const dayIndex = w * N_WEEKDAYS + d;
                const date = addDays(monday, d);
                days.push({
                    index: dayIndex,
                    date,
                    dateString: format(date, "yyyy-MM-dd"),
                    isWeekend: d >= 5,
                });
            }

            result.push({ weekIndex: w, weekNumber: w + 1, mondayDate: monday, days });
        }

        return result;
    });

    // Clean interaction click router
    function handleCellClick(dayIndex: number, role?: "lead" | "support") {
        const mode = app.activeMode;

        if (mode === "select") {
            if (role) {
                app.selection = { type: "role", dayIndex, role };
            } else {
                app.selection = { type: "day", dayIndex };
            }
        } else if (mode === "set") {
            if (!role) return;
            app.setRole(dayIndex, role, app.activeBrush);
        } else if (mode === "erase") {
            if (!role) return;
            app.setRole(dayIndex, role, undefined);
        } else if (mode === "swap_day") {
            const src = app.swapSource;
            if (src.type === "day") {
                app.swapDays(src.dayIndex, dayIndex);
                app.swapSource = { type: "none" };
            } else {
                app.swapSource = { type: "day", dayIndex };
            }
        } else if (mode === "swap_role") {
            if (!role) return;
            const src = app.swapSource;
            if (src.type === "role") {
                app.swapRoles(src.dayIndex, src.role, dayIndex, role);
                app.swapSource = { type: "none" };
            } else {
                app.swapSource = { type: "role", dayIndex, role };
            }
        }
    }

    // Resets pending actions when switching modes
    $effect(() => {
        const _ = app.activeMode;
        app.selection = { type: "none" };
        app.swapSource = { type: "none" };
    });

    function weekMonthLabel(week: WeekTopology, weekIdx: number): string | null {
        const m = week.mondayDate.getMonth();
        if (weekIdx === 0) {
            return week.mondayDate.toLocaleDateString("en-GB", { month: "long", year: "numeric" });
        }
        const prev = weeks[weekIdx - 1];
        if (!prev || prev.mondayDate.getMonth() !== m) {
            return week.mondayDate.toLocaleDateString("en-GB", { month: "long", year: "numeric" });
        }
        return null;
    }

    function handleKeyDown(event: KeyboardEvent) {
        if (app.activeMode !== "select") return;

        let day = app.selection.type !== "none" ? app.selection.dayIndex : undefined;
        if (day === undefined) return;

        let newDay = day;
        if (event.key === "ArrowUp") newDay -= N_WEEKDAYS;
        else if (event.key === "ArrowDown") newDay += N_WEEKDAYS;
        else if (event.key === "ArrowLeft") newDay -= 1;
        else if (event.key === "ArrowRight") newDay += 1;
        else return;

        if (newDay >= 0 && newDay < N_DAYS) {
            if (app.selection.type === "role") {
                app.selection = { type: "role", dayIndex: newDay, role: app.selection.role };
            } else {
                app.selection = { type: "day", dayIndex: newDay };
            }
            event.preventDefault();
        }
    }
</script>

<svelte:window onkeydown={handleKeyDown} />

<div
    class="grid-container flex-1 min-w-0 flex flex-col bg-background relative overflow-hidden zoom-{app.zoomLevel}"
>
    <div
        class={cn(
            "flex-1 overflow-auto pt-2 pb-4 px-3",
            app.zoomLevel === "detail" && "overflow-x-auto",
        )}
    >
        <!-- Header row -->
        <div
            class="flex items-center sticky top-0 z-4 bg-background border-b border-border pb-1 mb-1"
        >
            <div class="w-17 shrink-0"></div>
            {#each WEEKDAYS as day, i}
                <div
                    class={cn(
                        "flex-1 min-w-0 text-center text-[10.5px] font-semibold uppercase tracking-wider py-1",
                        i >= 5 ? "text-muted-foreground/60" : "text-muted-foreground",
                    )}
                >
                    {day}
                </div>
            {/each}
        </div>

        <!-- Week rows -->
        {#each weeks as week, wi}
            {@const isWeekSelected = app.selectedWeek === week.weekIndex}
            {@const monthLabel = weekMonthLabel(week, wi)}
            {@const holidayPeople = app.people.filter((p) => p.holidays.includes(week.weekIndex))}
            {@const initialsList = holidayPeople
                .map((p) =>
                    p.name
                        .split(/\s+/)
                        .map((n) => n[0] || "")
                        .join("")
                        .toUpperCase(),
                )
                .join(", ")}
            {@const fullNamesList = holidayPeople.map((p) => p.name).join(", ")}

            {#if monthLabel}
                <div class="flex items-center gap-2 pt-2.5 pb-1">
                    <span class="text-[11px] font-semibold text-muted-foreground tracking-[0.04em]"
                        >{monthLabel}</span
                    >
                    <div class="flex-1 h-px bg-border"></div>
                </div>
            {/if}

            <div
                class={cn(
                    "flex items-stretch gap-0.5 mb-0.5 rounded-[6px] px-0.5 transition-colors",
                    isWeekSelected && "bg-blue-500/5 border border-dashed border-blue-500/20",
                )}
            >
                <!-- Week label -->
                <div class="w-17 shrink-0 flex flex-col justify-center pr-1.5 gap-px">
                    <span class="font-mono text-[10px] font-bold text-muted-foreground"
                        >W{week.weekNumber}</span
                    >
                    <span class="text-[9.5px] text-muted-foreground/70 hide-micro">
                        {week.mondayDate.toLocaleDateString("en-GB", {
                            day: "numeric",
                            month: "short",
                        })}
                    </span>
                    {#if holidayPeople.length > 0}
                        <span
                            class="text-[8.5px] text-muted-foreground/50 italic truncate hide-micro select-none"
                            title="Holidays: {fullNamesList}"
                        >
                            Off: {initialsList}
                        </span>
                    {/if}
                </div>

                <!-- Day cells -->
                {#each week.days as day}
                    {@const isDaySelected = app.isSelected(day.index)}
                    {@const isDaySwapSource = app.isSwapSource(day.index)}

                    {@const daySlots = app.slots[day.index]}
                    {@const leadSlot = getLead(daySlots)}
                    {@const suppSlot = getSupport(daySlots)}

                    {@const holiday = app.holidayMap[day.dateString]}
                    {@const conflicts = app.conflictMap[day.index]}

                    <!-- Determine target click behavior based on mode -->
                    {@const hitsWholeDay =
                        app.activeMode === "select" || app.activeMode === "swap_day"}

                    <div
                        class={cn(
                            "day-cell flex-1 min-w-0 flex flex-col border-[1.5px] relative overflow-hidden outline-none bg-card select-none",
                            day.isWeekend && "bg-muted/20",
                            holiday && "bg-amber-500/5 border-amber-500/20",
                            hitsWholeDay ? "hitbox-day" : "hitbox-role",
                            isDaySelected &&
                                "border-blue-500! shadow-[0_0_0_1px_var(--color-blue-500)] z-[3]",
                            isDaySwapSource &&
                                "border-dashed border-orange-500! animate-pulse z-[3]",
                        )}
                    >
                        <!-- Click trigger overlay for whole-day tools -->
                        {#if hitsWholeDay}
                            <button
                                class="absolute inset-0 w-full h-full cursor-pointer z-[4]"
                                onclick={() => handleCellClick(day.index)}
                                aria-label="Select Day"
                            ></button>
                        {/if}

                        <!-- Date -->
                        <span
                            class="absolute top-0.5 left-0.75 font-mono text-[8.5px] text-muted-foreground/60 leading-none z-[2] hide-micro"
                        >
                            {getDate(day.date)}
                        </span>

                        <!-- Indicators Overlay -->
                        <div class="absolute top-0.5 right-0.75 flex items-center gap-1 z-[5]">
                            {#if holiday}
                                <span
                                    class="text-[7.5px] text-amber-600 bg-amber-500/10 px-1 py-0.2 rounded font-semibold uppercase tracking-wider hide-micro"
                                >
                                    {holiday}
                                </span>
                            {/if}

                            <!-- Performant CSS-driven Conflict Tooltip -->
                            {#if conflicts && conflicts.length > 0}
                                <div class="conflict-wrapper relative group">
                                    <span
                                        class="size-2 rounded-full bg-destructive block animate-pulse cursor-help"
                                    ></span>
                                    <div
                                        class="conflict-tooltip absolute bottom-full right-0 mb-1 hidden group-hover:block bg-popover text-popover-foreground border border-destructive/20 shadow-lg rounded p-2 text-[10.5px] min-w-44 pointer-events-none"
                                    >
                                        <p
                                            class="font-bold border-b border-border pb-1 mb-1 text-destructive"
                                        >
                                            Scheduling Conflicts
                                        </p>
                                        <ul
                                            class="space-y-1 text-[10px] list-disc pl-3 font-medium"
                                        >
                                            {#each conflicts as conflict}
                                                <li>{conflict}</li>
                                            {/each}
                                        </ul>
                                    </div>
                                </div>
                            {/if}
                        </div>

                        <!-- Dynamic Snippet Zoom Dispatcher -->
                        {#if app.zoomLevel === "micro"}
                            {@render microCell(day, leadSlot, suppSlot)}
                        {:else if app.zoomLevel === "standard"}
                            {@render standardCell(day, leadSlot, suppSlot)}
                        {:else}
                            {@render detailCell(day, leadSlot, suppSlot)}
                        {/if}
                    </div>
                {/each}
            </div>
        {/each}
    </div>
</div>

<!-- == ZOOM MODE SNIPPETS == -->

{#snippet microCell(day: DayTopology, lead: number | undefined, support: number | undefined)}
    {@render roleHalf(day.index, lead, false, true)}
    {@render roleHalf(day.index, support, true, true)}
{/snippet}

{#snippet standardCell(day: DayTopology, lead: number | undefined, support: number | undefined)}
    {@render roleHalf(day.index, lead, false, false)}
    {@render roleHalf(day.index, support, true, false)}
{/snippet}

{#snippet detailCell(day: DayTopology, lead: number | undefined, support: number | undefined)}
    {@render roleHalf(day.index, lead, false, false)}
    {@render roleHalf(day.index, support, true, false)}
{/snippet}

<!-- == CORE ROLE RENDERER == -->
{#snippet roleHalf(
    dayIndex: number,
    personIndex: number | undefined,
    isSupport: boolean,
    isMicro: boolean,
)}
    {@const name = app.formattedNames[personIndex ?? -1]}
    {@const swatch =
        personIndex != null ? PERSON_COLORS[personIndex % PERSON_COLORS.length][1] : null}
    {@const roleType = isSupport ? "support" : "lead"}

    {@const isRoleSelected = app.isSelected(dayIndex, roleType)}
    {@const isRoleSwapSource = app.isSwapSource(dayIndex, roleType)}

    <div
        class={cn(
            "role-half flex-1 flex items-center gap-1 relative overflow-hidden justify-center sm:justify-start",
            !isSupport && "border-b border-border",
            isRoleSelected && "bg-blue-500/15 ring-1 ring-blue-500/30 z-[2]",
            isRoleSwapSource && "bg-orange-500/15 ring-1 ring-orange-500/30 border-dashed z-[2]",
        )}
    >
        <!-- Role click triggers when not in whole-day selection modes -->
        <button
            class="absolute inset-0 w-full h-full cursor-pointer z-10 focus:outline-none"
            onclick={() => handleCellClick(dayIndex, roleType)}
            tabindex="-1"
            aria-label="Select Role"
        ></button>

        {#if name}
            <span class={cn("swatch shrink-0", swatch)}></span>
            {#if !isMicro}
                <span class="text-[10.5px] font-semibold leading-none truncate">{name}</span>
            {/if}
        {:else if !isMicro}
            <span class="text-[10px] text-muted-foreground/40 italic">&mdash;</span>
        {/if}
    </div>
{/snippet}

<style>
    /* Scope dynamic metrics to parent class configurations */
    .grid-container {
        --cell-min-h: 3.375rem;
        --cell-p: 0.1875rem 0.375rem;
        --cell-radius: 5px;
        --swatch-size: 0.5625rem;
        --swatch-rounded: 2px;
    }

    .grid-container.zoom-micro {
        --cell-min-h: 2.25rem;
        --cell-p: 0;
        --cell-radius: 4px;
        --swatch-size: 100%;
        --swatch-rounded: 0;
    }

    .grid-container.zoom-detail {
        --cell-min-h: 4.5rem;
        --cell-p: 0.25rem 0.375rem;
        --cell-radius: 6px;
        --swatch-size: 0.625rem;
        --swatch-rounded: 2px;
    }

    /* Set up target mode rules for clicking and highlighting */
    .hitbox-day {
        cursor: pointer;
    }
    .hitbox-day .role-half button {
        pointer-events: none !important;
    }

    .hitbox-role .day-cell button {
        pointer-events: none !important;
    }

    /* Elements to dismiss immediately in micro zoom without DOM mutation */
    .grid-container.zoom-micro .hide-micro {
        display: none !important;
    }

    /* Apply geometry vars */
    .day-cell {
        min-height: var(--cell-min-h);
        border-radius: var(--cell-radius);
    }

    .role-half {
        padding: var(--cell-p);
    }

    .grid-container.zoom-detail .role-half {
        flex-wrap: wrap;
    }

    .swatch {
        width: var(--swatch-size);
        height: var(--swatch-size);
        border-radius: var(--swatch-rounded);
    }
</style>

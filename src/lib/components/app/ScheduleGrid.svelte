<script lang="ts">
    import { ZoomIn, ZoomOut } from "@lucide/svelte";
    import { addDays, addWeeks, getDate, parseISO, startOfISOWeek } from "date-fns";

    import { app } from "$lib/app.svelte";
    import { NULL_SLOT, N_DAYS, N_WEEKDAYS, N_WEEKS, PERSON_COLORS, WEEKDAYS } from "$lib/defs.js";
    import { cn } from "$lib/utils.js";

    interface DayView {
        lead: number | null;
        support: number | null;
        date: Date;
        hasViolation: boolean;
    }

    interface WeekView {
        weekNumber: number;
        mondayDate: Date;
        days: DayView[];
    }

    let selectable = $derived(app.activeMode === "select");

    let dayIndex = $derived(selectable ? app.selectedDayOfWeek : undefined);
    let weekIndex = $derived(selectable ? app.selectedWeek : undefined);

    const weeks = $derived.by<WeekView[]>(() => {
        const baseDate = startOfISOWeek(parseISO(app.startDate));
        const result: WeekView[] = [];

        for (let w = 0; w < N_WEEKS; w++) {
            const monday = addWeeks(baseDate, w);
            const days: DayView[] = [];

            for (let d = 0; d < N_WEEKDAYS; d++) {
                const daySlot = w * N_WEEKDAYS + d;
                const leadSlot = app.slots[daySlot] & 0xf;
                const suppSlot = app.slots[daySlot] >> 4;

                days.push({
                    lead: leadSlot !== NULL_SLOT ? leadSlot : null,
                    support: suppSlot !== NULL_SLOT ? suppSlot : null,
                    date: addDays(monday, d),
                    hasViolation: false,
                });
            }

            result.push({ weekNumber: w + 1, mondayDate: monday, days });
        }

        return result;
    });

    function selectDay(w: number, d: number) {
        if (weekIndex === w && dayIndex === d) {
            app.selectedDay = undefined;
        } else {
            app.selectedDay = w * N_WEEKDAYS + d;
        }
    }

    function isSelected(w: number, d: number): boolean {
        return weekIndex === w && dayIndex === d;
    }

    function weekMonthLabel(week: WeekView, weekIdx: number): string | null {
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

    // Arrow key navigation
    function handleKeyDown(event: KeyboardEvent) {
        const day = app.selectedDay;
        if (day === undefined) return;

        if (event.key === "ArrowUp") {
            if (day > N_WEEKDAYS) {
                app.selectedDay = day - N_WEEKDAYS;
            }
        } else if (event.key === "ArrowDown") {
            if (day < N_DAYS - N_WEEKDAYS) {
                app.selectedDay = day + N_WEEKDAYS;
            }
        } else if (event.key === "ArrowLeft") {
            if (day > 0) {
                app.selectedDay = day - 1;
            }
        } else if (event.key === "ArrowRight") {
            if (day < N_DAYS - 1) {
                app.selectedDay = day + 1;
            }
        } else {
            return;
        }

        event.preventDefault();
    }
</script>

<svelte:window onkeydown={handleKeyDown} />

<div class="flex-1 min-w-0 flex flex-col bg-background relative overflow-hidden">
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
            {@const monthLabel = weekMonthLabel(week, wi)}
            {#if monthLabel}
                <div class="flex items-center gap-2 pt-2.5 pb-1">
                    <span class="text-[11px] font-semibold text-muted-foreground tracking-[0.04em]"
                        >{monthLabel}</span
                    >
                    <div class="flex-1 h-px bg-border"></div>
                </div>
            {/if}

            <div class="flex items-stretch gap-0.5 mb-0.5">
                <!-- Week label -->
                <div class="w-17 shrink-0 flex flex-col justify-center pr-1.5 gap-px">
                    <span class="font-mono text-[10px] font-bold text-muted-foreground"
                        >W{week.weekNumber}</span
                    >
                    {#if app.zoomLevel !== "micro"}
                        <span class="text-[9.5px] text-muted-foreground/70">
                            {week.mondayDate.toLocaleDateString("en-GB", {
                                day: "numeric",
                                month: "short",
                            })}
                        </span>
                    {/if}
                </div>

                <!-- Day cells -->
                {#each week.days as day, di}
                    {@const isWeekend = di >= 5}
                    {@const leadPerson = day.lead !== null ? app.people[day.lead] : null}
                    {@const suppPerson = day.support !== null ? app.people[day.support] : null}

                    {@const leadSwatch =
                        day.lead !== null
                            ? PERSON_COLORS[day.lead % PERSON_COLORS.length][1]
                            : null}

                    {@const suppSwatch =
                        day.support !== null
                            ? PERSON_COLORS[day.support % PERSON_COLORS.length][1]
                            : null}

                    <button
                        class={cn(
                            "flex-1 min-w-0 flex flex-col rounded-[5px] border-[1.5px] cursor-pointer relative overflow-hidden outline-none",
                            isWeekend ? "bg-muted/30" : "bg-card",
                            isSelected(wi, di) &&
                                "border-blue-500! shadow-[0_0_0_1px_var(--color-blue-500)]",
                            !isSelected(wi, di) && "border-transparent hover:border-border",
                            cn(
                                app.zoomLevel === "micro" && "min-h-9",
                                app.zoomLevel === "standard" && "min-h-13.5",
                                app.zoomLevel === "detail" && "min-h-18",
                            ),
                        )}
                        onclick={() => selectDay(wi, di)}
                    >
                        <!-- Date -->
                        {#if app.zoomLevel !== "micro"}
                            <span
                                class="absolute top-0.5 left-0.75 font-mono text-[8.5px] text-muted-foreground/60 leading-none z-[2]"
                            >
                                {getDate(day.date)}
                            </span>
                        {/if}

                        <!-- Lead half -->
                        <div
                            class={cn(
                                "flex-1 flex items-center gap-1 relative",
                                app.zoomLevel === "micro" && "p-0",
                                app.zoomLevel === "standard" && "px-1.5 py-0.75",
                                app.zoomLevel === "detail" &&
                                    "px-1.5 py-1 flex-row items-center flex-wrap",
                                "border-b border-border",
                            )}
                        >
                            {#if leadPerson}
                                <span
                                    class={cn(
                                        "rounded-xs shrink-0",
                                        cn(
                                            app.zoomLevel === "micro" &&
                                                "w-full h-full! rounded-none",
                                            app.zoomLevel === "standard" && "size-2.25",
                                            app.zoomLevel === "detail" && "size-2.5",
                                            leadSwatch,
                                        ),
                                    )}
                                ></span>
                                {#if app.zoomLevel !== "micro"}
                                    <span class="text-[10.5px] font-semibold leading-none truncate"
                                        >{leadPerson.name}</span
                                    >
                                {/if}
                            {:else if app.zoomLevel !== "micro"}
                                <span class="text-[10px] text-muted-foreground/50 italic"
                                    >&mdash;</span
                                >
                            {/if}
                        </div>

                        <!-- Support half -->
                        <div
                            class={cn(
                                "flex-1 flex items-center gap-1 relative",
                                app.zoomLevel === "micro" && "p-0",
                                app.zoomLevel === "standard" && "px-1.5 py-0.75",
                                app.zoomLevel === "detail" &&
                                    "px-1.5 py-1 flex-row items-center flex-wrap",
                            )}
                        >
                            {#if suppPerson}
                                <span
                                    class={cn(
                                        "rounded-xs shrink-0",
                                        cn(
                                            app.zoomLevel === "micro" &&
                                                "w-full h-full! rounded-none",
                                            app.zoomLevel === "standard" && "size-2.25",
                                            app.zoomLevel === "detail" && "size-2.5",
                                            suppSwatch,
                                        ),
                                    )}
                                ></span>
                                {#if app.zoomLevel !== "micro"}
                                    <span class="text-[10.5px] font-semibold leading-none truncate"
                                        >{suppPerson.name}</span
                                    >
                                {/if}
                            {:else if app.zoomLevel !== "micro"}
                                <span class="text-[10px] text-muted-foreground/50 italic"
                                    >&mdash;</span
                                >
                            {/if}
                        </div>
                    </button>
                {/each}
            </div>
        {/each}
    </div>
</div>

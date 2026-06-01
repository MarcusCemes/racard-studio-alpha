<script lang="ts">
    import { app } from "$lib/app.svelte.js";
    import { N_DAYS, N_WEEKDAYS, N_WEEKS, PERSON_COLORS } from "$lib/defs.js";
    import { getLead, getSupport } from "$lib/slot.js";
    import { cn } from "$lib/utils.js";

    interface WeekDayBreakdown {
        weekdays: number; // Mon–Thu shifts
        fridays: number; // Fri shifts
        satLead: number; // Saturday Lead
        satSupport: number; // Saturday Support
    }

    // [week][person] → WeekDayBreakdown
    const distribution = $derived.by<WeekDayBreakdown[][]>(() => {
        const grid: WeekDayBreakdown[][] = Array.from({ length: N_WEEKS }, () =>
            app.people.map(() => ({
                weekdays: 0,
                fridays: 0,
                satLead: 0,
                satSupport: 0,
            })),
        );

        for (let dayIdx = 0; dayIdx < N_DAYS; dayIdx++) {
            const slot = app.slots[dayIdx];
            const week = Math.floor(dayIdx / N_WEEKDAYS);
            const dayOfWeek = dayIdx % N_WEEKDAYS;

            const lead = getLead(slot);
            const support = getSupport(slot);

            if (lead !== undefined) {
                const cell = grid[week][lead];
                if (dayOfWeek < 4) cell.weekdays++;
                else if (dayOfWeek === 4) cell.fridays++;
                else if (dayOfWeek === 5) cell.satLead++;
            }

            if (support !== undefined) {
                const cell = grid[week][support];
                if (dayOfWeek < 4) cell.weekdays++;
                else if (dayOfWeek === 4) cell.fridays++;
                else if (dayOfWeek === 5) cell.satSupport++;
            }
        }

        return grid;
    });

    function isHolidayWeek(weekIdx: number): boolean {
        return app.people.some((p) => p.holidays.includes(weekIdx));
    }

    function isPersonHoliday(personIdx: number, weekIdx: number): boolean {
        return app.people[personIdx].holidays.includes(weekIdx);
    }
</script>

<div class="text-[11px] text-muted-foreground mb-4 space-y-1">
    <p class="font-semibold">Legend:</p>
    <div class="flex flex-wrap gap-x-4 gap-y-1">
        <span
            ><span class="inline-block w-2 h-2 rounded-full bg-blue-500"></span> Weekday (Mon–Thu)</span
        >
        <span><span class="inline-block w-2 h-2 rounded-full bg-amber-500"></span> Friday</span>
        <span
            ><span class="inline-block w-2.5 h-2.5 bg-green-600 inline-block align-middle"></span> Saturday
            Lead</span
        >
        <span
            ><span
                class="inline-block w-2.5 h-2.5 border border-green-600 inline-block align-middle"
            ></span> Saturday Support</span
        >
    </div>
</div>

<div class="inline-block min-w-full">
    <!-- Header row: person names -->
    <div class="flex gap-0.5 mb-1 sticky top-0 bg-background z-10">
        <div class="w-12 shrink-0"></div>
        {#each app.people as person, i}
            {@const [, swatch] = PERSON_COLORS[i]}
            <div
                class="flex-1 min-w-[48px] text-[9px] font-semibold text-center truncate px-0.5"
                title={person.name}
            >
                <span
                    class="inline-block w-1.5 h-1.5 rounded-sm mr-0.5 align-middle"
                    style="background:{swatch}"
                ></span>
                {app.formattedNames[i]}
            </div>
        {/each}
    </div>

    <!-- Week rows -->
    {#each distribution as weekData, weekIdx}
        {@const isHol = isHolidayWeek(weekIdx)}
        <div class={cn("flex gap-0.5 py-0.5", isHol && "bg-amber-500/10")}>
            <div
                class="w-12 shrink-0 text-[9px] text-muted-foreground font-mono leading-[18px] text-right pr-1"
            >
                Wk {weekIdx + 1}
            </div>
            {#each weekData as cell, personIdx}
                {@const personHol = isPersonHoliday(personIdx, weekIdx)}
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div
                    class={cn(
                        "flex-1 min-w-[48px] h-[18px] flex items-center justify-center gap-px text-[8px] font-mono rounded-sm overflow-hidden",
                        personHol && "opacity-40",
                    )}
                    title="{app.people[personIdx].name}: Wk {weekIdx + 1}"
                >
                    {#if cell.weekdays > 0}
                        <span class="text-blue-500">{"●".repeat(Math.min(cell.weekdays, 4))}</span>
                    {/if}
                    {#if cell.fridays > 0}
                        <span class="text-amber-500">{"◐".repeat(Math.min(cell.fridays, 1))}</span>
                    {/if}
                    {#if cell.satLead > 0}
                        <span class="text-green-600">■</span>
                    {/if}
                    {#if cell.satSupport > 0}
                        <span
                            class="text-green-600 border border-green-600 inline-block w-1.5 h-1.5"
                        ></span>
                    {/if}
                </div>
            {/each}
        </div>
    {/each}
</div>

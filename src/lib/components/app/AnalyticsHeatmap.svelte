<script lang="ts">
    import { app } from "$lib/app.svelte.js";
    import { N_DAYS, N_WEEKDAYS, N_WEEKS, PERSON_COLORS } from "$lib/defs.js";
    import { getLead, getSupport } from "$lib/slot.js";

    interface HeatCell {
        personIdx: number;
        weekIdx: number;
        leadHours: number;
        supportHours: number;
        totalHours: number;
        onHoliday: boolean;
    }

    // [personIdx][weekIdx] → HeatCell
    const heatmapData = $derived.by<HeatCell[][]>(() => {
        const grid: HeatCell[][] = app.people.map((person, personIdx) =>
            Array.from({ length: N_WEEKS }, (_, weekIdx) => {
                const onHoliday = person.holidays.includes(weekIdx);
                return {
                    personIdx,
                    weekIdx,
                    leadHours: 0,
                    supportHours: 0,
                    totalHours: 0,
                    onHoliday,
                };
            }),
        );

        // Scan all slots and accumulate hours
        for (let dayIdx = 0; dayIdx < N_DAYS; dayIdx++) {
            const slot = app.slots[dayIdx];
            if (slot === 0xff) continue; // NULL_SLOT

            const week = Math.floor(dayIdx / N_WEEKDAYS);
            const dayOfWeek = dayIdx % N_WEEKDAYS;
            const [leadHours, supportHours] = app.weekdayHours[dayOfWeek];

            const lead = getLead(slot);
            const support = getSupport(slot);

            if (lead !== undefined) {
                grid[lead][week].leadHours += leadHours;
                grid[lead][week].totalHours += leadHours;
            }

            if (support !== undefined) {
                grid[support][week].supportHours += supportHours;
                grid[support][week].totalHours += supportHours;
            }
        }

        return grid;
    });

    // Compute max hours for color scaling
    const maxHours = $derived.by(() => {
        let max = 0;
        for (const row of heatmapData) {
            for (const cell of row) {
                if (!cell.onHoliday && cell.totalHours > max) {
                    max = cell.totalHours;
                }
            }
        }
        return max > 0 ? max : 1;
    });

    function getCellColor(cell: HeatCell): string {
        if (cell.onHoliday) return "rgba(251, 191, 36, 0.3)"; // amber-400/30
        if (cell.totalHours === 0) return "rgba(148, 163, 184, 0.1)"; // slate-400/10

        // Scale intensity based on hours worked
        const intensity = Math.min(cell.totalHours / maxHours, 1);
        // Use green for balanced load, red if overloaded (>40h/week is high)
        if (cell.totalHours > 40) {
            return `rgba(239, 68, 68, ${0.3 + intensity * 0.5})`; // red-500
        }
        return `rgba(34, 197, 94, ${0.2 + intensity * 0.6})`; // green-500
    }

    function getCellTitle(cell: HeatCell): string {
        const personName = app.people[cell.personIdx].name;
        if (cell.onHoliday) {
            return `${personName} — Wk ${cell.weekIdx + 1}: Holiday`;
        }
        const parts = [];
        if (cell.leadHours > 0) parts.push(`Lead: ${cell.leadHours.toFixed(1)}h`);
        if (cell.supportHours > 0) parts.push(`Support: ${cell.supportHours.toFixed(1)}h`);
        if (parts.length === 0) {
            return `${personName} — Wk ${cell.weekIdx + 1}: No shifts`;
        }
        return `${personName} — Wk ${cell.weekIdx + 1}\n${parts.join(", ")}\nTotal: ${cell.totalHours.toFixed(1)}h`;
    }
</script>

<div class="text-[11px] text-muted-foreground mb-4 space-y-1">
    <p class="font-semibold">Legend:</p>
    <div class="flex flex-wrap gap-x-4 gap-y-1">
        <span
            ><span class="inline-block w-3 h-3 rounded-sm" style="background:rgba(34, 197, 94, 0.5)"
            ></span> Balanced (≤40h)</span
        >
        <span
            ><span class="inline-block w-3 h-3 rounded-sm" style="background:rgba(239, 68, 68, 0.5)"
            ></span> Overloaded (>40h)</span
        >
        <span
            ><span
                class="inline-block w-3 h-3 rounded-sm"
                style="background:rgba(251, 191, 36, 0.3)"
            ></span> Holiday</span
        >
        <span
            ><span
                class="inline-block w-3 h-3 rounded-sm"
                style="background:rgba(148, 163, 184, 0.1)"
            ></span> No shifts</span
        >
    </div>
</div>

<div class="inline-block min-w-full">
    <!-- Week number header -->
    <div class="flex gap-0.5 mb-2">
        <div class="w-24 shrink-0"></div>
        {#each Array(N_WEEKS) as _, weekIdx}
            <div
                class="flex-1 min-w-3 text-[8px] text-muted-foreground text-center font-mono"
                class:hidden={weekIdx % 4 !== 0 && weekIdx !== N_WEEKS - 1}
            >
                {weekIdx + 1}
            </div>
        {/each}
    </div>

    <!-- Person rows -->
    {#each heatmapData as personRow, personIdx}
        {@const name = app.formattedNames[personIdx]}
        {@const [swatch] = PERSON_COLORS[personIdx]}

        <div class="flex gap-0.5 mb-0.5 items-center">
            <div class="w-24 shrink-0 flex items-center gap-1">
                <span
                    class="inline-block w-2 h-2 rounded-sm"
                    style="background:{swatch
                        .replace('bg-', '')
                        .replace('-500', '')
                        .replace('-400', '')}"
                ></span>
                <span class="text-[11px] font-medium truncate">{name}</span>
            </div>

            {#each personRow as cell}
                <div
                    class="flex-1 min-w-3 h-6 rounded-sm transition-colors"
                    style="background:{getCellColor(cell)}"
                    title={getCellTitle(cell)}
                ></div>
            {/each}
        </div>
    {/each}
</div>

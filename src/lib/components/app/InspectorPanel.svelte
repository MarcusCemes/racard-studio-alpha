<script lang="ts">
    import { addDays, addWeeks, parseISO, startOfISOWeek } from "date-fns";
    import { LineChart } from "layerchart";

    import { app } from "$lib/app.svelte.js";
    import { type ChartConfig, ChartContainer } from "$lib/components/ui/chart/index.js";
    import { Separator } from "$lib/components/ui/separator/index.js";
    import { DEFAULT_WEEKDAY_HOURS, N_WEEKDAYS, N_WEEKS, PERSON_COLORS } from "$lib/defs.js";
    import { getLead, getSupport } from "$lib/slot.js";
    import { cn } from "$lib/utils.js";

    let dayIndex = $derived(app.selectedDayOfWeek);
    let weekIndex = $derived(app.selectedWeek);
    let selectedPerson = $derived(app.people[app.activeBrush ?? -1]);

    // Derive day data from selection
    const dayData = $derived.by(() => {
        if (dayIndex == null || weekIndex == null) return null;

        const baseDate = startOfISOWeek(parseISO(app.startDate));
        const monday = addWeeks(baseDate, weekIndex);
        const date = addDays(monday, dayIndex);
        const daySlot = weekIndex * N_WEEKDAYS + dayIndex;

        const leadSlot = getLead(app.slots[daySlot]);
        const suppSlot = getSupport(app.slots[daySlot]);

        return {
            date,
            weekNumber: weekIndex + 1,
            lead: leadSlot != null ? leadSlot : null,
            support: suppSlot != null ? suppSlot : null,
        };
    });

    // Derive employee stats from slots
    const empStats = $derived.by(() => {
        const stats = app.people.map(() => ({
            idx: 0,
            totalShifts: 0,
            leadShifts: 0,
            supportShifts: 0,
            violations: 0,
        }));

        for (let d = 0; d < N_WEEKS * N_WEEKDAYS; d++) {
            const lead = getLead(app.slots[d]);
            const supp = getSupport(app.slots[d]);

            if (lead != null && lead < stats.length) {
                stats[lead].totalShifts++;
                stats[lead].leadShifts++;
            }

            if (supp != null && supp < stats.length) {
                stats[supp].totalShifts++;
                stats[supp].supportShifts++;
            }
        }

        for (const c of app.conflicts) {
            if ("ConsecutiveDay" in c) stats[c.ConsecutiveDay[0]].violations++;
            if ("Holiday" in c) stats[c.Holiday[0]].violations++;
            if ("Role" in c) stats[c.Role[0]].violations++;
            if ("WorkCount" in c) stats[c.WorkCount[0]].violations++;
        }

        return stats;
    });

    const PERSON_COLOR_HEX = [
        "#3b82f6",
        "#22c55e",
        "#ef4444",
        "#eab308",
        "#a855f7",
        "#ec4899",
        "#6366f1",
        "#14b8a6",
        "#60a5fa",
        "#4ade80",
        "#f87171",
        "#facc15",
        "#c084fc",
        "#f472b6",
        "#818cf8",
    ];

    const weeklyHours = $derived.by(() => {
        const idx = app.activeBrush;
        if (idx === undefined) return [];

        const data: { week: number; hours: number }[] = [];
        for (let w = 0; w < N_WEEKS; w++) {
            let totalHours = 0;
            for (let d = 0; d < N_WEEKDAYS; d++) {
                const slot = app.slots[w * N_WEEKDAYS + d];
                const lead = getLead(slot);
                const supp = getSupport(slot);

                if (lead === idx) totalHours += DEFAULT_WEEKDAY_HOURS[d][0];
                if (supp === idx) totalHours += DEFAULT_WEEKDAY_HOURS[d][1];
            }
            data.push({ week: w + 1, hours: totalHours });
        }
        return data;
    });

    const weeklyChartConfig = $derived.by(() => {
        const idx = app.activeBrush;

        const config: ChartConfig = {
            hours: {
                label: "Hours",
                color:
                    idx !== undefined ? PERSON_COLOR_HEX[idx % PERSON_COLOR_HEX.length] : "#3b82f6",
            },
        };
        return config;
    });

    const panelState = $derived(
        dayIndex != null && weekIndex != null ? "day" : selectedPerson ? "employee" : "idle",
    );
</script>

<aside class="w-72 shrink-0 flex flex-col border-l border-border bg-card overflow-hidden">
    <!-- Header -->
    <div class="px-3.5 py-3 border-b border-border shrink-0">
        {#if panelState === "idle"}
            <span class="text-[13px] font-semibold">Inspector</span>
        {:else if panelState === "day" && dayData}
            <div class="flex flex-col gap-0.5">
                <span class="text-[13px] font-semibold">
                    {dayData.date.toLocaleDateString("en-GB", { weekday: "long" })}
                </span>
                <span class="text-[11.5px] text-muted-foreground">
                    {dayData.date.toLocaleDateString("en-GB", {
                        day: "numeric",
                        month: "long",
                        year: "numeric",
                    })}
                </span>
            </div>
        {:else if panelState === "employee" && app.activeBrush !== undefined}
            {@const { name, rate } = app.people[app.activeBrush]}
            {@const swatch = PERSON_COLORS[app.activeBrush % PERSON_COLORS.length][1]}

            <div class="flex items-center gap-2.5">
                <span class="w-1.5 h-9 rounded-[3px] shrink-0" style="background:{swatch}"></span>
                <div>
                    <span class="text-[13px] font-semibold block">{name}</span>
                    <span class="text-[11px] text-muted-foreground">{rate}%</span>
                </div>
            </div>
        {/if}
    </div>

    <div class="flex-1 overflow-y-auto py-2.5">
        <!-- ── IDLE STATE ── -->
        {#if panelState === "idle"}
            <div class="px-3.5 py-6 flex flex-col items-center justify-center text-center gap-2">
                <span class="text-muted-foreground text-sm">Select a day or employee</span>
                <span class="text-muted-foreground text-xs"
                    >Press <kbd class="px-1 py-0.5 rounded bg-muted font-mono text-[10px]">c</kbd> for
                    settings</span
                >
            </div>

            <!-- ── DAY STATE ── -->
        {:else if panelState === "day" && dayData}
            <div class="mx-3.5 mb-3 rounded-lg border border-border overflow-hidden bg-background">
                <div class="flex items-center gap-1.5 px-3 py-2.5 min-h-[42px]">
                    <span
                        class="text-[10px] font-bold uppercase tracking-[0.07em] text-muted-foreground w-[44px] shrink-0"
                        >Lead</span
                    >
                    {#if dayData.lead !== null}
                        {@const person = app.people[dayData.lead]}
                        {@const swatch = PERSON_COLORS[dayData.lead % PERSON_COLORS.length][1]}
                        <span class="w-2.5 h-2.5 rounded-[3px] shrink-0" style="background:{swatch}"
                        ></span>
                        <span class="flex-1 text-[12.5px] font-medium truncate">{person.name}</span>
                    {:else}
                        <span class="text-xs text-muted-foreground italic">Unassigned</span>
                    {/if}
                </div>
                <Separator />
                <div class="flex items-center gap-1.5 px-3 py-2.5 min-h-[42px]">
                    <span
                        class="text-[10px] font-bold uppercase tracking-[0.07em] text-muted-foreground w-[44px] shrink-0"
                        >Support</span
                    >
                    {#if dayData.support !== null}
                        {@const person = app.people[dayData.support]}
                        {@const swatch = PERSON_COLORS[dayData.support % PERSON_COLORS.length][1]}
                        <span class="w-2.5 h-2.5 rounded-[3px] shrink-0" style="background:{swatch}"
                        ></span>
                        <span class="flex-1 text-[12.5px] font-medium truncate">{person.name}</span>
                    {:else}
                        <span class="text-xs text-muted-foreground italic">Unassigned</span>
                    {/if}
                </div>
            </div>

            <div class="mx-3.5 mb-2.5 text-[11.5px] text-muted-foreground italic">
                No violations
            </div>

            <div class="mx-3.5 pt-2.5 border-t border-border">
                <span
                    class="text-[10px] font-semibold uppercase tracking-[0.07em] text-muted-foreground"
                    >Week {dayData.weekNumber}</span
                >
            </div>

            <!-- ── EMPLOYEE STATE ── -->
        {:else if panelState === "employee" && selectedPerson}
            {@const s = empStats[app.activeBrush!]}

            <div
                class="grid grid-cols-2 gap-px mx-3.5 mb-3.5 rounded-lg overflow-hidden border border-border bg-border"
            >
                {#each [["Total shifts", `${s.totalShifts}`], ["Lead shifts", `${s.leadShifts}`], ["Support shifts", `${s.supportShifts}`], ["Violations", `${s.violations}`]] as [label, val]}
                    <div
                        class={cn(
                            "flex flex-col gap-0.5 px-2.5 py-2 bg-background",
                            label === "Violations" && s.violations > 0 && "col-span-2",
                        )}
                    >
                        <span
                            class="text-[9.5px] uppercase tracking-wider text-muted-foreground font-semibold"
                            >{label}</span
                        >
                        <span
                            class={cn(
                                "text-sm font-semibold font-mono",
                                label === "Violations" && s.violations > 0 && "text-red-500",
                            )}
                        >
                            {val}
                        </span>
                    </div>
                {/each}
            </div>

            <!-- Weekly hours chart -->
            <div class="mx-3.5">
                <span
                    class="text-[10px] font-semibold uppercase tracking-[0.06em] text-muted-foreground mb-1.5 block"
                    >Weekly hours</span
                >
                <ChartContainer
                    config={weeklyChartConfig}
                    class="h-10! aspect-auto! justify-start!"
                >
                    {#if weeklyHours.length > 1}
                        <LineChart
                            data={weeklyHours}
                            x="week"
                            y="hours"
                            padding={{ top: 2, right: 2, bottom: 8, left: 2 }}
                            yDomain={[0, null]}
                            series={[{ key: "hours", color: "var(--color-hours)" }]}
                        />
                    {/if}
                </ChartContainer>
                <div class="h-px bg-border mt-0.5"></div>
            </div>
        {/if}
    </div>
</aside>

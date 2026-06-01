<script lang="ts">
    import { BarChart, Rule } from "layerchart";

    import { app } from "$lib/app.svelte.js";
    import * as Chart from "$lib/components/ui/chart/index.js";
    import { N_WEEKS } from "$lib/defs.js";
    import { cn } from "$lib/utils.js";

    interface Props {
        person: number;
    }

    let { person }: Props = $props();

    let statistics = $derived(app.statistics?.people[person]);

    let weeklyHours = $derived(
        statistics?.weeks.map(({ hours_by_role: [a, b] }, i) => ({
            week: `Week ${i + 1}`,
            hours: a + b,
        })),
    );

    // Role split totals
    let roleSplit = $derived.by(() => {
        if (!statistics) return null;
        const lead = statistics.weeks.reduce((sum, w) => sum + w.hours_by_role[0], 0);
        const support = statistics.weeks.reduce((sum, w) => sum + w.hours_by_role[1], 0);
        const total = lead + support;
        return { lead, support, total, leadPct: total > 0 ? (lead / total) * 100 : 0 };
    });

    // Blank weeks (no shifts, not on holiday)
    let blankWeeks = $derived.by(() => {
        if (!statistics) return null;
        const personData = app.people[person];
        const blanks = statistics.weeks.filter((w, i) => {
            if (personData.holidays.includes(i)) return false;
            return w.slots_count === 0;
        }).length;
        return blanks;
    });

    // Hour delta
    let hourDelta = $derived.by(() => {
        if (!statistics) return null;
        const { totals } = statistics;
        const delta = totals.total_hours_worked - totals.expected_hours;
        const pct = totals.expected_hours > 0 ? (delta / totals.expected_hours) * 100 : 0;
        return { delta, pct, absPct: Math.abs(pct) };
    });

    const stats = $derived.by(() => {
        if (!statistics) return null;

        const { totals } = statistics;

        return [
            ["Expected hours", totals.expected_hours.toFixed(2)],
            ["Total worked", totals.total_hours_worked.toFixed(2)],
            ["Long weekends", totals.long_weekends.toString()],
            ["Short weekends", totals.short_weekends.toString()],
            ["Lead fridays", totals.lead_fridays.toString()],
            ["Support fridays", totals.support_fridays.toString()],
        ] as [string, string][];
    });

    const config = {
        hours: {
            label: "Hours",
            color: "#2563eb",
        },
    } satisfies Chart.ChartConfig;
</script>

{#if statistics}
    <div
        class="grid grid-cols-2 gap-px mx-3.5 mb-3.5 rounded-lg overflow-hidden border border-border bg-border"
    >
        {#each stats as [label, val]}
            <div class="flex flex-col gap-0.5 px-2.5 py-2 bg-background">
                <span
                    class="text-[9.5px] uppercase tracking-wider text-muted-foreground font-semibold"
                >
                    {label}
                </span>

                <span class="text-sm font-semibold font-mono">
                    {val}
                </span>
            </div>
        {/each}

        <!-- Hour delta -->
        {#if hourDelta}
            <div class="col-span-2 flex items-center justify-between px-2.5 py-2 bg-background">
                <span
                    class="text-[9.5px] uppercase tracking-wider text-muted-foreground font-semibold"
                >
                    Hour delta
                </span>
                <span
                    class={cn(
                        "text-sm font-semibold font-mono",
                        hourDelta.absPct < 2
                            ? "text-green-600 dark:text-green-500"
                            : hourDelta.pct > 0
                              ? "text-red-600 dark:text-red-500"
                              : "text-blue-600 dark:text-blue-500",
                    )}
                >
                    {hourDelta.delta > 0 ? "+" : ""}
                    {hourDelta.delta.toFixed(1)}h ({hourDelta.pct > 0 ? "+" : ""}
                    {hourDelta.pct.toFixed(1)}%)
                </span>
            </div>
        {/if}

        <!-- Blank weeks -->
        {#if blankWeeks !== null}
            <div class="col-span-2 flex items-center justify-between px-2.5 py-2 bg-background">
                <span
                    class="text-[9.5px] uppercase tracking-wider text-muted-foreground font-semibold"
                >
                    Blank weeks
                </span>
                <span class="text-sm font-semibold font-mono">
                    {blankWeeks} of {N_WEEKS - app.people[person].holidays.length}
                </span>
            </div>
        {/if}
    </div>

    <!-- Role split visualization -->
    {#if roleSplit && roleSplit.total > 0}
        <div class="mx-3.5 mb-3.5">
            <div
                class="flex items-center justify-between text-[9.5px] uppercase tracking-wider text-muted-foreground font-semibold mb-1.5"
            >
                <span>Role split</span>
                <span class="font-mono normal-case tracking-normal">
                    {roleSplit.leadPct.toFixed(0)}% Lead
                </span>
            </div>
            <div class="h-2 rounded overflow-hidden bg-border flex">
                <div class="bg-blue-500 transition-all" style:width="{roleSplit.leadPct}%"></div>
            </div>
            <div class="flex justify-between text-[10px] text-muted-foreground mt-0.5 font-mono">
                <span>Lead {roleSplit.lead.toFixed(1)}h</span>
                <span>Support {roleSplit.support.toFixed(1)}h</span>
            </div>
        </div>
    {/if}

    <!-- Weekly hours chart -->
    <div class="mx-3.5">
        <span
            class="text-[10px] font-semibold uppercase tracking-[0.06em] text-muted-foreground mb-1.5 block"
        >
            Weekly hours
        </span>

        <Chart.Container {config} class="h-10! aspect-auto! justify-start!">
            {#if weeklyHours}
                <BarChart
                    data={weeklyHours}
                    x="week"
                    y="hours"
                    axis={false}
                    grid={false}
                    series={[{ key: "hours", color: "var(--color-hours)" }]}
                    props={{
                        bars: {
                            stroke: "none",
                            rounded: "none",
                        },
                    }}
                />
            {/if}
        </Chart.Container>
    </div>
{/if}

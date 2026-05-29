<script lang="ts">
    import { BarChart, Rule } from "layerchart";

    import { app } from "$lib/app.svelte.js";
    import * as Chart from "$lib/components/ui/chart/index.js";

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
    </div>

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

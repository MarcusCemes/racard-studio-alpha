<script lang="ts">
    import { LineChart, ScatterChart } from "layerchart";

    import { app } from "$lib/app.svelte.js";
    import * as Chart from "$lib/components/ui/chart/index.js";
    import { N_DAYS, N_WEEKDAYS, N_WEEKS, PERSON_HEX_COLORS } from "$lib/defs.js";
    import { getLead, getSupport } from "$lib/slot.js";

    // --- D1: Cumulative Hours ---
    const cumulativeData = $derived.by(() => {
        if (!app.statistics) return [];
        const data: { week: number; person: string; cumulative: number }[] = [];

        for (let i = 0; i < app.people.length; i++) {
            const personStats = app.statistics.people[i];
            const name = app.formattedNames[i];
            for (let w = 0; w < N_WEEKS; w++) {
                data.push({
                    week: w + 1,
                    person: name,
                    cumulative: personStats.weeks[w].cumulative_hours,
                });
            }
        }
        return data;
    });

    // --- D2: Weekend Scatter ---
    interface WeekendDot {
        week: number;
        person: string;
        role: "lead" | "support";
    }

    const weekendData = $derived.by<WeekendDot[]>(() => {
        const dots: WeekendDot[] = [];

        for (let dayIdx = 0; dayIdx < N_DAYS; dayIdx++) {
            const slot = app.slots[dayIdx];
            if (slot === 0xff) continue;

            const week = Math.floor(dayIdx / N_WEEKDAYS);
            const dayOfWeek = dayIdx % N_WEEKDAYS;

            // Only Saturday (day 5 = Sat)
            if (dayOfWeek !== 5) continue;

            const lead = getLead(slot);
            const support = getSupport(slot);

            if (lead !== undefined) {
                dots.push({
                    week: week + 1,
                    person: app.formattedNames[lead],
                    role: "lead",
                });
            }

            if (support !== undefined) {
                dots.push({
                    week: week + 1,
                    person: app.formattedNames[support],
                    role: "support",
                });
            }
        }

        return dots;
    });

    // --- D3: Fitness Breakdown ---
    interface FitnessBar {
        component: string;
        value: number;
        label: string;
    }

    const fitnessData = $derived.by<FitnessBar[]>(() => {
        if (!app.statistics) return [];

        const labels: Record<string, string> = {
            annual_hours: "Annual Hours",
            consecutive_days: "Consecutive Days",
            consecutive_weekends: "Consecutive Weekends",
            weekend_alternation: "Weekend Balance",
            weekend_regularity: "Weekend Rythm",
            weekly_hours: "Weekly Hours",
            blank_weeks: "Blank Weeks",
        };

        return Object.entries(app.statistics.fitness)
            .map(([key, value]) => ({
                component: key,
                value: value as number,
                label: labels[key] ?? key,
            }))
            .sort((a, b) => b.value - a.value);
    });

    const totalFitness = $derived(fitnessData.reduce((sum, d) => sum + d.value, 0));
</script>

{#if app.statistics}
    <!-- ═══ D1: Cumulative Hours ═══ -->
    {#if cumulativeData.length > 0}
        <section>
            <h3
                class="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground mb-2"
            >
                Cumulative Hours
            </h3>
            <p class="text-[11px] text-muted-foreground mb-4">
                Each line tracks cumulative hours over the 48-week period. Lines converging toward
                their target indicates fairness.
            </p>

            <Chart.Container config={{}}>
                <LineChart
                    data={cumulativeData}
                    x="week"
                    axis={true}
                    series={app.people.map((_, i) => ({
                        key: app.formattedNames[i],
                        data: cumulativeData.filter((d) => d.person === app.formattedNames[i]),
                        value: "cumulative",
                        color: PERSON_HEX_COLORS[i],
                    }))}
                    props={{
                        spline: { strokeWidth: 1.5 },
                        xAxis: { ticks: 12 },
                        yAxis: { ticks: 6 },
                    }}
                />
            </Chart.Container>
        </section>
    {/if}

    <!-- ═══ D2: Weekend Scatter ═══ -->
    {#if weekendData.length > 0}
        <section>
            <h3
                class="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground mb-2"
            >
                Weekend Distribution
            </h3>
            <p class="text-[11px] text-muted-foreground mb-4">
                Each dot represents a Saturday shift. Evenly spaced dots indicate fair distribution.
            </p>

            <Chart.Container config={{}}>
                <ScatterChart
                    data={weekendData}
                    x="week"
                    y="person"
                    axis={true}
                    series={app.people.map((_, i) => ({
                        key: app.formattedNames[i],
                        data: weekendData.filter((d) => d.person === app.formattedNames[i]),
                        color: PERSON_HEX_COLORS[i],
                    }))}
                    props={{
                        xAxis: { ticks: 12 },
                    }}
                />
            </Chart.Container>
        </section>
    {/if}

    <!-- ═══ D3: Fitness Breakdown ═══ -->
    {#if fitnessData.length > 0}
        <section>
            <h3
                class="text-[11px] font-semibold uppercase tracking-wider text-muted-foreground mb-2"
            >
                Fitness Breakdown
            </h3>
            <p class="text-[11px] text-muted-foreground mb-4">
                Lower is better. Total fitness: <span class="font-mono font-semibold"
                    >{totalFitness.toFixed(1)}</span
                >
            </p>

            <div class="space-y-2">
                {#each fitnessData as d}
                    {@const pct = totalFitness > 0 ? (d.value / totalFitness) * 100 : 0}
                    <div>
                        <div class="flex justify-between items-baseline mb-0.5">
                            <span class="text-xs">{d.label}</span>
                            <span class="text-[10px] text-muted-foreground font-mono">
                                {d.value.toFixed(1)}
                                <span class="opacity-50 ml-1">({pct.toFixed(0)}%)</span>
                            </span>
                        </div>
                        <div class="h-4 bg-muted rounded-sm overflow-hidden relative">
                            <div
                                class="h-full bg-primary rounded-sm transition-all"
                                style:width={`${pct}%`}
                            ></div>
                        </div>
                    </div>
                {/each}
            </div>
        </section>
    {/if}
{:else}
    <div class="text-center text-sm text-muted-foreground py-16">
        Run the solver or refiner to generate statistics.
    </div>
{/if}

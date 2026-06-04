<script lang="ts">
    import { ArrowDownAZ, ArrowUpAZ, ArrowUpDown } from "@lucide/svelte";

    import { app } from "$lib/app.svelte.js";
    import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";
    import * as Table from "$lib/components/ui/table/index.js";
    import { N_DAYS, N_WEEKDAYS, PERSON_TEXT_COLORS } from "$lib/defs.js";
    import { getLead, getSupport } from "$lib/slot.js";
    import { cn } from "$lib/utils.js";

    interface PersonRow {
        index: number;
        name: string;
        rate: number;
        total: number;
        expected: number;
        delta: number;
        weekdayHours: number;
        fridayHours: number;
        longWeekends: number;
        shortWeekends: number;
        leadFridays: number;
        supportFridays: number;
        holidayWeeks: number;
    }

    type ColumnKey =
        | "name"
        | "rate"
        | "total"
        | "expected"
        | "delta"
        | "weekday"
        | "friday"
        | "longWknd"
        | "shortWknd"
        | "leadFri"
        | "suppFri"
        | "holiday";

    const rows = $derived.by<PersonRow[]>(() => {
        if (!app.statistics) return [];

        return app.people.map((person, i) => {
            const personStats = app.statistics?.people[i];
            if (!personStats) {
                return {
                    index: i,
                    name: person.name,
                    rate: person.rate,
                    total: 0,
                    expected: 0,
                    delta: 0,
                    weekdayHours: 0,
                    fridayHours: 0,
                    longWeekends: 0,
                    shortWeekends: 0,
                    leadFridays: 0,
                    supportFridays: 0,
                    holidayWeeks: person.holidays.length,
                };
            }

            const { totals } = personStats;

            // Compute Mon–Thu vs Fri hours from slots
            let weekdayHours = 0;
            let fridayHours = 0;

            for (let day = 0; day < N_DAYS; day++) {
                const slot = app.slots[day];
                const dayOfWeek = day % N_WEEKDAYS;
                const hours = app.weekdayHours[dayOfWeek];

                const lead = getLead(slot);
                const support = getSupport(slot);

                if (lead === i) {
                    if (dayOfWeek < 4) weekdayHours += hours[0];
                    else if (dayOfWeek === 4) fridayHours += hours[0];
                }
                if (support === i) {
                    if (dayOfWeek < 4) weekdayHours += hours[1];
                    else if (dayOfWeek === 4) fridayHours += hours[1];
                }
            }

            return {
                index: i,
                name: person.name,
                rate: person.rate,
                total: totals.total_hours_worked,
                expected: totals.expected_hours,
                delta: totals.total_hours_worked - totals.expected_hours,
                weekdayHours,
                fridayHours,
                longWeekends: totals.long_weekends,
                shortWeekends: totals.short_weekends,
                leadFridays: totals.lead_fridays,
                supportFridays: totals.support_fridays,
                holidayWeeks: person.holidays.length,
            };
        });
    });

    const totals = $derived.by(() => {
        const t = {
            total: 0,
            expected: 0,
            delta: 0,
            weekday: 0,
            friday: 0,
            longWknd: 0,
            shortWknd: 0,
            leadFri: 0,
            suppFri: 0,
            holiday: 0,
        };
        for (const r of rows) {
            t.total += r.total;
            t.expected += r.expected;
            t.delta += r.delta;
            t.weekday += r.weekdayHours;
            t.friday += r.fridayHours;
            t.longWknd += r.longWeekends;
            t.shortWknd += r.shortWeekends;
            t.leadFri += r.leadFridays;
            t.suppFri += r.supportFridays;
            t.holiday += r.holidayWeeks;
        }
        return t;
    });

    function deltaClass(delta: number, expected: number): string {
        if (expected === 0) return "text-muted-foreground";
        const pct = (delta / expected) * 100;
        if (Math.abs(pct) < 2) return "text-green-600 dark:text-green-500";
        return pct > 0 ? "text-red-600 dark:text-red-500" : "text-blue-600 dark:text-blue-500";
    }
</script>

{#snippet header(col: ColumnKey, label: string, className?: string)}
    <Table.Head class={className}>
        {label}
    </Table.Head>
{/snippet}

<Table.Root class="text-xs">
    <Table.Header>
        <Table.Row class="border-b-2">
            {@render header("name", "Person")}
            {@render header("rate", "Rate", "text-end")}
            {@render header("total", "Total", "text-end")}
            {@render header("expected", "Expected", "text-end")}
            {@render header("delta", "Δ", "text-end")}
            {@render header("weekday", "Mon–Thu", "text-end")}
            {@render header("friday", "Fri", "text-end")}
            {@render header("longWknd", "Long", "text-end")}
            {@render header("shortWknd", "Short", "text-end")}
            {@render header("leadFri", "L-Fri", "text-end")}
            {@render header("suppFri", "S-Fri", "text-end")}
            {@render header("holiday", "Hol.", "text-end")}
        </Table.Row>
    </Table.Header>

    <Table.Body>
        {#each rows as row}
            {@const color = PERSON_TEXT_COLORS[row.index % PERSON_TEXT_COLORS.length]}

            <Table.Row class="border-b">
                <Table.Cell class="font-medium">
                    <span class={color}>{row.name}</span>
                </Table.Cell>
                <Table.Cell class="text-end text-muted-foreground">{row.rate}%</Table.Cell>
                <Table.Cell class="text-end font-mono">{row.total.toFixed(1)}</Table.Cell>
                <Table.Cell class="text-end font-mono text-muted-foreground">
                    {row.expected.toFixed(1)}
                </Table.Cell>
                <Table.Cell
                    class={cn(
                        "text-end font-mono font-semibold",
                        deltaClass(row.delta, row.expected),
                    )}
                >
                    {row.delta > 0 ? "+" : ""}{row.delta.toFixed(1)}
                </Table.Cell>
                <Table.Cell class="text-end font-mono">{row.weekdayHours.toFixed(1)}</Table.Cell>
                <Table.Cell class="text-end font-mono">{row.fridayHours.toFixed(1)}</Table.Cell>
                <Table.Cell class="text-end font-mono">{row.longWeekends}</Table.Cell>
                <Table.Cell class="text-end font-mono">{row.shortWeekends}</Table.Cell>
                <Table.Cell class="text-end font-mono">{row.leadFridays}</Table.Cell>
                <Table.Cell class="text-end font-mono">{row.supportFridays}</Table.Cell>
                <Table.Cell class="text-end font-mono">{row.holidayWeeks}</Table.Cell>
            </Table.Row>
        {/each}

        <!-- Totals row -->
        <Table.Row class="border-t-2 bg-muted/30 font-semibold">
            <Table.Cell class="text-muted-foreground">Total</Table.Cell>
            <Table.Cell class="text-end"></Table.Cell>
            <Table.Cell class="text-end font-mono">{totals.total.toFixed(1)}</Table.Cell>
            <Table.Cell class="text-end font-mono">{totals.expected.toFixed(1)}</Table.Cell>
            <Table.Cell
                class={cn(
                    "text-end font-mono font-semibold",
                    totals.expected > 0 && Math.abs((totals.delta / totals.expected) * 100) >= 2
                        ? "text-red-600 dark:text-red-500"
                        : "text-muted-foreground",
                )}
            >
                {totals.delta > 0 ? "+" : ""}{totals.delta.toFixed(1)}
            </Table.Cell>
            <Table.Cell class="text-end font-mono">{totals.weekday.toFixed(1)}</Table.Cell>
            <Table.Cell class="text-end font-mono">{totals.friday.toFixed(1)}</Table.Cell>
            <Table.Cell class="text-end font-mono">{totals.longWknd}</Table.Cell>
            <Table.Cell class="text-end font-mono">{totals.shortWknd}</Table.Cell>
            <Table.Cell class="text-end font-mono">{totals.leadFri}</Table.Cell>
            <Table.Cell class="text-end font-mono">{totals.suppFri}</Table.Cell>
            <Table.Cell class="text-end font-mono">{totals.holiday}</Table.Cell>
        </Table.Row>
    </Table.Body>
</Table.Root>

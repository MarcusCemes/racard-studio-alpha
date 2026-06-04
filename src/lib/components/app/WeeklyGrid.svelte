<script lang="ts">
    import { ArrowRight } from "@lucide/svelte";
    import { addDays, format, isEqual } from "date-fns";

    import { app } from "$lib/app.svelte.js";
    import * as Select from "$lib/components/ui/select/index.js";
    import Slider from "$lib/components/ui/slider/slider.svelte";
    import * as Table from "$lib/components/ui/table/index.js";
    import { N_WEEKDAYS, N_WEEKS, Role } from "$lib/defs.js";
    import { useHotKeys } from "$lib/hooks/useHotkey.svelte.js";
    import { getLead, getSupport, setLead, setSupport } from "$lib/slot";

    import AnalyticsSummary from "./AnalyticsSummary.svelte";

    const FULL_TIME_HOURS = 33;

    useHotKeys(null, handleKey);

    let page = $state(1);

    let week = $derived(page - 1);
    let index = $derived(week * N_WEEKDAYS);

    let weekStart = $derived(addDays(app.startDate, week * N_WEEKDAYS));
    let weekEnd = $derived(addDays(weekStart, N_WEEKDAYS - 1));

    function handleKey(event: KeyboardEvent) {
        if (event.key === "ArrowUp") {
            page = Math.max(1, page - 1);
        } else if (event.key === "ArrowDown") {
            page = Math.min(N_WEEKS, page + 1);
        } else {
            return;
        }

        event.preventDefault();
    }

    function* generateDays(index: number) {
        for (let i = 0; i < N_WEEKDAYS; i++) {
            const date = addDays(app.startDate, index + i);
            const slot = app.slots[index + i];

            const bankHoliday = app.bankHolidays.find(
                (holiday) => holiday.enabled && isEqual(holiday.date, date),
            );

            const hours = app.statistics?.hours[index + i];

            yield {
                bankHoliday,
                day: format(date, "EEEE"),
                date: format(date, "MMMM dd"),
                hours,
                index: index + i,
                lead: getLead(slot),
                support: getSupport(slot),
            };
        }
    }

    function* generatePeople() {
        for (let i = 0; i < app.people.length; i++) {
            const { holidays, rate } = app.people[i];
            const stats = app.statistics?.people[i];
            const weekStats = stats?.weeks[week];
            const onHoliday = holidays.includes(week);

            const ratedWorked = weekStats?.hours_by_role.reduce((a, b) => a + b, 0) ?? 0;
            const rateHours = (FULL_TIME_HOURS * rate) / 100;
            const holidayWorked = onHoliday ? rateHours : 0;
            const totalWorked = ratedWorked + holidayWorked;

            const diff = weekStats && (week + 1) * rateHours - weekStats.cumulative_hours;

            yield {
                diff,
                name: app.formattedNames[i],
                holidayWorked,
                roles: weekStats?.role_counts,
                totalWorked,
            };
        }
    }

    function fmtNum(value?: number) {
        return value?.toFixed(2) ?? "-";
    }

    function fmtDate(date: Date) {
        return format(date, "MMM dd");
    }
</script>

<div class="flex-1 flex p-4 flex-col gap-4 overflow-auto bg-neutral-100">
    <div class="flex gap-4">
        <div class="w-48 flex items-center overflow-clip">
            <div class="w-10 whitespace-nowrap font-bold">W{page}</div>

            <div
                class="flex-1 flex justify-center items-center text-muted-foreground text-sm whitespace-nowrap gap-1"
            >
                <div class="flex-1 text-right">{fmtDate(weekStart)}</div>
                <div><ArrowRight class="size-3" /></div>
                <div class="flex-1">{fmtDate(weekEnd)}</div>
            </div>
        </div>

        <div class="flex-1 self-center px-2 py-3 bg-white border rounded-full">
            <Slider type="single" bind:value={page} max={N_WEEKS} min={1} step={1} />
        </div>
    </div>

    <div class="flex-1 flex gap-4">
        <div class="flex-1 border bg-white">
            <Table.Root class="text-xs">
                <Table.Body>
                    {#each generateDays(index) as day, i (i)}
                        <Table.Row>
                            <Table.Cell>
                                <div class="font-medium">{day.day}</div>
                                <div class="text-xs text-muted-foreground">{day.date}</div>
                            </Table.Cell>

                            <Table.Cell>
                                {@render selector(day.lead, day.index, Role.Lead)}
                            </Table.Cell>

                            <Table.Cell class="font-mono">
                                {day.hours?.[0].toFixed(2)}
                            </Table.Cell>

                            <Table.Cell>
                                {@render selector(day.support, day.index, Role.Support)}
                            </Table.Cell>

                            <Table.Cell class="font-mono">
                                {day.hours?.[1].toFixed(2)}
                            </Table.Cell>
                        </Table.Row>
                    {/each}
                </Table.Body>
            </Table.Root>
        </div>

        <div class="flex-1 border bg-white">
            <Table.Root class="text-xs">
                <Table.Header>
                    <Table.Row>
                        <Table.Head>Person</Table.Head>
                        <Table.Head>Holiday</Table.Head>
                        <Table.Head>Total</Table.Head>
                        <Table.Head>Diff</Table.Head>
                        <Table.Head>Lead</Table.Head>
                        <Table.Head>Support</Table.Head>
                    </Table.Row>
                </Table.Header>

                <Table.Body>
                    {#each generatePeople() as person, i (i)}
                        <Table.Row>
                            <Table.Cell class="font-medium">{person.name}</Table.Cell>
                            <Table.Cell class="font-mono">{fmtNum(person.holidayWorked)}</Table.Cell
                            >
                            <Table.Cell class="font-mono">{fmtNum(person.totalWorked)}</Table.Cell>
                            <Table.Cell class="font-mono">{fmtNum(person.diff)}</Table.Cell>
                            <Table.Cell class="font-mono">{person.roles?.[0] ?? "-"}</Table.Cell>
                            <Table.Cell class="font-mono">{person.roles?.[1] ?? "-"}</Table.Cell>
                        </Table.Row>
                    {/each}
                </Table.Body>
            </Table.Root>
        </div>
    </div>

    <div class="border bg-white">
        <AnalyticsSummary />
    </div>
</div>

{#snippet selector(personIdx: number | undefined, dayIdx: number, role: Role)}
    <Select.Root
        type="single"
        bind:value={
            () => (personIdx === undefined ? "-" : app.formattedNames[personIdx]),
            (value) => {
                const newValue = value === "-" ? undefined : parseInt(value);
                if (role === Role.Lead) {
                    app.slots[dayIdx] = setLead(app.slots[dayIdx], newValue);
                } else if (role === Role.Support) {
                    app.slots[dayIdx] = setSupport(app.slots[dayIdx], newValue);
                }
            }
        }
    >
        <Select.Trigger class="w-32 text-xs">
            {personIdx !== undefined ? app.formattedNames[personIdx] : "-"}
        </Select.Trigger>

        <Select.Content>
            <Select.Item value="-" label="-" />

            {#each app.formattedNames as name, i (i)}
                <Select.Item value={i.toString()} label={name}>
                    {name}
                </Select.Item>
            {/each}
        </Select.Content>
    </Select.Root>
{/snippet}

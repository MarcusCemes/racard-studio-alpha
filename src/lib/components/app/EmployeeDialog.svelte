<script lang="ts">
    import { Minus, Plus, Settings } from "@lucide/svelte";
    import { addDays, addWeeks, format, parseISO, startOfISOWeek } from "date-fns";

    import { app } from "$lib/app.svelte.js";
    import Badge from "$lib/components/ui/badge/badge.svelte";
    import { Button, buttonVariants } from "$lib/components/ui/button/index.js";
    import * as Card from "$lib/components/ui/card/index.js";
    import * as Dialog from "$lib/components/ui/dialog/index.js";
    import * as Field from "$lib/components/ui/field/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Separator } from "$lib/components/ui/separator/index.js";
    import { Slider } from "$lib/components/ui/slider/index.js";
    import { Toggle } from "$lib/components/ui/toggle/index.js";
    import * as Tooltip from "$lib/components/ui/tooltip/index.js";
    import { MAX_PEOPLE, MIN_PEOPLE, N_WEEKS, PERSON_COLORS } from "$lib/defs.js";
    import { plural } from "$lib/misc";
    import type { Person } from "$lib/schemas.js";

    let { open = $bindable(false) } = $props();

    // Pre-calculate week date ranges for the 48 weeks based on app.startDate
    const weekRanges = $derived.by(() => {
        try {
            const baseDate = startOfISOWeek(parseISO(app.startDate));
            const ranges = [];
            for (let w = 0; w < N_WEEKS; w++) {
                const monday = addWeeks(baseDate, w);
                const sunday = addDays(monday, 6);
                ranges.push({
                    index: w,
                    dateRange: `${format(monday, "d MMM")} – ${format(sunday, "d MMM yyyy")}`,
                });
            }
            return ranges;
        } catch (e) {
            console.error("Error calculating week ranges in EmployeeDialog:", e);
            return [];
        }
    });

    function onadd() {
        const length = app.people.length;
        if (length >= MAX_PEOPLE) return;
        app.people.push({ holidays: [], name: `Employee ${length + 1}`, rate: 80 });
    }

    function onremove() {
        if (app.people.length <= MIN_PEOPLE) return;
        app.people.pop();
    }

    function toggleHoliday(person: Person, weekIndex: number) {
        if (person.holidays.includes(weekIndex)) {
            person.holidays = person.holidays.filter((w) => w !== weekIndex);
        } else {
            person.holidays = [...person.holidays, weekIndex].sort((a, b) => a - b);
        }
    }
</script>

<Dialog.Root>
    <Dialog.Trigger class={buttonVariants({ variant: "secondary", size: "icon" })}>
        <Settings class="size-4" />
    </Dialog.Trigger>

    <Dialog.Content style="max-width: 62rem;" class="flex flex-col max-h-[85vh]">
        <Dialog.Header class="pb-2 border-b border-border shrink-0">
            <Dialog.Title class="text-base font-semibold">Manage Employees</Dialog.Title>
        </Dialog.Header>

        <div class="flex-1 overflow-y-auto space-y-4 p-2">
            {#each app.people as person, i}
                {@const swatch = PERSON_COLORS[i % PERSON_COLORS.length][1]}

                <Card.Root>
                    <Card.Content class="flex">
                        <!-- Left Section: Details (Name, Rate, Delete) -->
                        <div class="flex flex-col gap-2.5">
                            <div class="flex items-center gap-2.5">
                                <!-- Swatch indicator -->
                                <span class="self-center size-4 rounded {swatch}"></span>

                                <!-- Name input -->
                                <Input
                                    type="text"
                                    bind:value={person.name}
                                    placeholder="Full name"
                                    class="flex-1 h-9 text-[13px] font-medium"
                                />
                            </div>

                            <!-- Rate controls -->
                            <div class="flex items-center gap-2 px-2.5 py-1">
                                <Field.Label>Rate</Field.Label>

                                <Slider
                                    bind:value={person.rate}
                                    type="single"
                                    min={5}
                                    max={100}
                                    step={5}
                                />

                                <span class="font-medium text-muted-foreground text-xs"
                                    >{person.rate}%</span
                                >
                            </div>
                        </div>

                        <div class="self-stretch mx-4 my-2">
                            <Separator orientation="vertical" />
                        </div>

                        <!-- Right Section: 48-Week Holiday Grid -->
                        <div class="flex-1">
                            <div class="flex items-center justify-between px-0.5 mb-1.5">
                                <Field.Label>Holidays</Field.Label>

                                {#if person.holidays.length > 0}
                                    <Badge variant="secondary">
                                        {person.holidays.length}
                                        {plural(person.holidays.length, "week")} off
                                    </Badge>
                                {/if}
                            </div>

                            <div class="grid grid-cols-24">
                                {#each weekRanges as week}
                                    {@const isHoliday = person.holidays.includes(week.index)}

                                    <Tooltip.Root>
                                        <Tooltip.Trigger>
                                            {#snippet child({ props })}
                                                <Toggle
                                                    {...props}
                                                    bind:pressed={
                                                        () => isHoliday,
                                                        () => toggleHoliday(person, week.index)
                                                    }
                                                    size="sm"
                                                    class="text-xs"
                                                >
                                                    {week.index + 1}
                                                </Toggle>
                                            {/snippet}
                                        </Tooltip.Trigger>

                                        <Tooltip.Content>
                                            {week.dateRange}
                                        </Tooltip.Content>
                                    </Tooltip.Root>
                                {/each}
                            </div>
                        </div>
                    </Card.Content>
                </Card.Root>
            {/each}
        </div>

        <Dialog.Footer>
            <Button variant="outline" onclick={onremove} disabled={app.people.length <= MIN_PEOPLE}>
                <Minus class="size-4" />
                Remove employee
            </Button>

            <Button variant="outline" onclick={onadd} disabled={app.people.length >= MAX_PEOPLE}>
                <Plus class="size-4" />
                Add Employee
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>

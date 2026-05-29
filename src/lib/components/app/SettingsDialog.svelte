<script lang="ts">
    import {
        Clock,
        Cog,
        Cpu,
        Crown,
        FileText,
        Flame,
        Minus,
        Pencil,
        Plus,
        RefreshCw,
        Scale,
        Settings,
        Shield,
        Users,
        X,
    } from "@lucide/svelte";
    import { addDays, addWeeks, format, getISODay, parseISO, startOfISOWeek } from "date-fns";

    import { apiBankHolidays } from "$lib/api.js";
    import { app } from "$lib/app.svelte.js";
    import Badge from "$lib/components/ui/badge/badge.svelte";
    import { Button } from "$lib/components/ui/button/index.js";
    import * as Dialog from "$lib/components/ui/dialog/index.js";
    import * as Field from "$lib/components/ui/field/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";
    import * as Separator from "$lib/components/ui/separator/index.js";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import { Slider } from "$lib/components/ui/slider/index.js";
    import { Switch } from "$lib/components/ui/switch/index.js";
    import * as Toggle from "$lib/components/ui/toggle/index.js";
    import * as Tooltip from "$lib/components/ui/tooltip/index.js";
    import {
        HOLIDAY_NAMES,
        MAX_PEOPLE,
        MIN_PEOPLE,
        N_WEEKS,
        PERSON_COLORS,
        WEEKDAYS,
    } from "$lib/defs.js";
    import { useHotKey } from "$lib/hooks/useHotkey.svelte";
    import { plural } from "$lib/misc";
    import type { Person } from "$lib/schemas.js";

    import Kbd from "../ui/kbd/kbd.svelte";

    /* === Nav === */
    const nav = [
        { name: "Details", icon: FileText },
        { name: "Employees", icon: Users },
        { name: "Hours", icon: Clock },
        { name: "Overrides", icon: Pencil },
        { name: "Solver", icon: Cpu },
        { name: "Refiner", icon: Flame },
        { name: "Weights", icon: Scale },
        { name: "Advanced", icon: Cog },
    ];

    let open = $state(false);
    let activeTab = $state("Details");

    useHotKey("c", () => (open = !open));

    /* === Employees === */
    const weekRanges = $derived.by(() => {
        try {
            const baseDate = startOfISOWeek(parseISO(app.startDate));
            const ranges = [];
            for (let w = 0; w < N_WEEKS; w++) {
                const monday = addWeeks(baseDate, w);
                const sunday = addDays(monday, 6);
                ranges.push({
                    index: w,
                    dateRange: `${format(monday, "d MMM")} \u2013 ${format(sunday, "d MMM yyyy")}`,
                });
            }
            return ranges;
        } catch (e) {
            console.error("Error calculating week ranges:", e);
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

    /* === Bank holidays === */
    async function refetchBankHolidays() {
        const raw = await apiBankHolidays(app.startDate);
        app.bankHolidays = raw.map(([date, holiday]) => ({
            date,
            name: HOLIDAY_NAMES[holiday],
            enabled: true,
            lead_hours: null,
            support_hours: null,
        }));
    }

    function bankHolidayDefaults(date: string): [number, number] {
        const weekday = getISODay(parseISO(date)) - 1;
        return app.bankHolidayDefaultHours[weekday] ?? [0, 0];
    }

    function removeBankHoliday(index: number) {
        app.bankHolidays.splice(index, 1);
    }

    /* === Custom overrides === */
    function addOverride() {
        app.customOverrides.push({ date: "", role: "Lead", hours: 0 });
    }

    function removeOverride(index: number) {
        app.customOverrides.splice(index, 1);
    }

    /* === Solver phases === */
    const solverPhases = [
        { key: "weekend", label: "Weekend" },
        { key: "friday", label: "Friday" },
        { key: "weekday", label: "Weekday" },
    ] as const;

    /* === Weight definitions === */
    const weightsDefs = [
        {
            key: "annual_hours",
            label: "Annual Hours",
            desc: "Penalty for deviating from expected annual total.",
        },
        {
            key: "consecutive_days",
            label: "Consecutive Days",
            desc: "Penalty for long work stretches without a break.",
        },
        {
            key: "consecutive_weekends",
            label: "Weekend Runs",
            desc: "Penalty for working too many weekends in a row.",
        },
        {
            key: "weekend_alternation",
            label: "Weekend Balance",
            desc: "Encourages fair weekend rotation.",
        },
        {
            key: "weekend_regularity",
            label: "Weekend Rhythm",
            desc: "Penalty for irregular weekend patterns.",
        },
        {
            key: "weekly_hours",
            label: "Weekly Hours",
            desc: "Penalty when weekly hours stray from the target.",
        },
        {
            key: "blank_weeks",
            label: "Blank Weeks",
            desc: "Heavy penalty for unassigned weeks.",
        },
    ] as const;
</script>

<Dialog.Root bind:open>
    <Dialog.Trigger>
        {#snippet child({ props })}
            <Tooltip.Root>
                <Tooltip.Trigger>
                    <Button {...props} variant="ghost" size="icon">
                        <Settings />
                    </Button>
                </Tooltip.Trigger>

                <Tooltip.Content>
                    Open settings <Kbd>c</Kbd>
                </Tooltip.Content>
            </Tooltip.Root>
        {/snippet}
    </Dialog.Trigger>

    <Dialog.Content
        class="overflow-hidden p-0 sm:max-h-[85vh] sm:h-[85vh] sm:max-w-4xl flex flex-col gap-0"
    >
        <Dialog.Title class="sr-only">Settings</Dialog.Title>
        <Dialog.Description class="sr-only">Configure schedule parameters.</Dialog.Description>

        <Sidebar.Provider class="overflow-hidden flex-1">
            <Sidebar.Root collapsible="none" class="hidden md:flex border-r">
                <Sidebar.Content>
                    <Sidebar.Group>
                        <Sidebar.GroupLabel class="mb-4">Project configuration</Sidebar.GroupLabel>

                        <Sidebar.GroupContent>
                            <Sidebar.Menu>
                                {#each nav as item}
                                    <Sidebar.MenuItem>
                                        <Sidebar.MenuButton
                                            isActive={activeTab === item.name}
                                            onclick={() => (activeTab = item.name)}
                                        >
                                            <item.icon />
                                            <span>{item.name}</span>
                                        </Sidebar.MenuButton>
                                    </Sidebar.MenuItem>
                                {/each}
                            </Sidebar.Menu>
                        </Sidebar.GroupContent>
                    </Sidebar.Group>
                </Sidebar.Content>
            </Sidebar.Root>

            <main class="flex-1 flex">
                <ScrollArea class="flex-1 h-[80vh]">
                    <div class="p-6">
                        <!-- ═══ Details ═══ -->
                        {#if activeTab === "Details"}
                            <Field.FieldSet>
                                <Field.Legend>Problem details</Field.Legend>
                                <Field.Description>
                                    Configure the core problem parameters for schedule generation.
                                </Field.Description>

                                <div class="overflow-hidden rounded-lg border bg-card">
                                    <Field.Field
                                        orientation="horizontal"
                                        class="border-b px-6 py-4"
                                    >
                                        <Field.Content>
                                            <Field.Label>Start date</Field.Label>
                                            <Field.Description>
                                                First Monday of the schedule period.
                                            </Field.Description>
                                        </Field.Content>

                                        <div>
                                            <Input
                                                type="date"
                                                bind:value={app.startDate}
                                                class="font-mono"
                                            />
                                        </div>
                                    </Field.Field>

                                    <Field.Field
                                        orientation="horizontal"
                                        class="border-b px-6 py-4"
                                    >
                                        <Field.Content>
                                            <Field.Label>Employees</Field.Label>
                                            <Field.Description>
                                                See the Employees tab to configure names, rates, and
                                                holidays.
                                            </Field.Description>
                                        </Field.Content>
                                        <span class="text-sm font-mono tabular-nums">
                                            {app.people.length} configured
                                        </span>
                                    </Field.Field>

                                    <Field.Field orientation="horizontal" class="px-6 py-4">
                                        <Field.Content>
                                            <Field.Label>Skip last shifts</Field.Label>
                                            <Field.Description>
                                                Exclude the final N workdays from scheduling.
                                            </Field.Description>
                                        </Field.Content>
                                        <Input
                                            type="number"
                                            bind:value={app.skipLastShifts}
                                            min="0"
                                            class="w-24 font-mono"
                                        />
                                    </Field.Field>
                                </div>
                            </Field.FieldSet>
                        {/if}

                        <!-- ═══ Employees ═══ -->
                        {#if activeTab === "Employees"}
                            <Field.FieldSet>
                                <Field.Legend>Manage employees</Field.Legend>
                                <Field.Description>
                                    Configure names, rates, and scheduled holidays.
                                </Field.Description>

                                <div class="flex flex-col gap-3">
                                    {#each app.people as person, i}
                                        {@const swatch = PERSON_COLORS[i % PERSON_COLORS.length][1]}

                                        <div class="rounded-lg border bg-card p-4">
                                            <div class="flex gap-4">
                                                <!-- Name & Rate -->
                                                <div
                                                    class="flex flex-col gap-3 min-w-0 w-48 shrink-0"
                                                >
                                                    <div class="flex items-center gap-2.5">
                                                        <span
                                                            class="size-3.5 rounded {swatch} shrink-0"
                                                        ></span>
                                                        <Input
                                                            type="text"
                                                            bind:value={person.name}
                                                            placeholder="Full name"
                                                            class="flex-1 h-9 text-[13px] font-medium"
                                                        />
                                                    </div>

                                                    <div class="flex items-center gap-2 px-1">
                                                        <Field.Label class="text-xs shrink-0"
                                                            >Rate</Field.Label
                                                        >
                                                        <Slider
                                                            bind:value={person.rate}
                                                            type="single"
                                                            min={5}
                                                            max={100}
                                                            step={5}
                                                        />
                                                        <span
                                                            class="font-mono text-xs text-muted-foreground w-8 text-right shrink-0"
                                                            >{person.rate}%</span
                                                        >
                                                    </div>
                                                </div>

                                                <Separator.Root
                                                    orientation="vertical"
                                                    class="mx-2"
                                                />

                                                <!-- Holiday grid -->
                                                <div class="flex-1 min-w-0">
                                                    <div
                                                        class="flex items-center justify-between mb-1.5"
                                                    >
                                                        <Field.Label class="text-xs"
                                                            >Holidays</Field.Label
                                                        >
                                                        {#if person.holidays.length > 0}
                                                            <Badge
                                                                variant="secondary"
                                                                class="text-[10px]"
                                                            >
                                                                {person.holidays.length}
                                                                {plural(
                                                                    person.holidays.length,
                                                                    "week",
                                                                )} off
                                                            </Badge>
                                                        {/if}
                                                    </div>

                                                    <div
                                                        class="grid gap-0.5"
                                                        style="grid-template-columns: repeat(12, minmax(0, 1fr))"
                                                    >
                                                        {#each weekRanges as week}
                                                            {@const isHoliday =
                                                                person.holidays.includes(
                                                                    week.index,
                                                                )}

                                                            <Tooltip.Root>
                                                                <Tooltip.Trigger>
                                                                    {#snippet child({ props })}
                                                                        <Toggle.Root
                                                                            {...props}
                                                                            bind:pressed={
                                                                                () => isHoliday,
                                                                                () =>
                                                                                    toggleHoliday(
                                                                                        person,
                                                                                        week.index,
                                                                                    )
                                                                            }
                                                                            size="sm"
                                                                            class="text-[10px]"
                                                                        >
                                                                            {week.index + 1}
                                                                        </Toggle.Root>
                                                                    {/snippet}
                                                                </Tooltip.Trigger>

                                                                <Tooltip.Content>
                                                                    {week.dateRange}
                                                                </Tooltip.Content>
                                                            </Tooltip.Root>
                                                        {/each}
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    {/each}
                                </div>

                                <div class="flex justify-center gap-2">
                                    <Button
                                        variant="outline"
                                        onclick={onremove}
                                        disabled={app.people.length <= MIN_PEOPLE}
                                    >
                                        <Minus class="size-4" />
                                        Remove employee
                                    </Button>

                                    <Button
                                        variant="outline"
                                        onclick={onadd}
                                        disabled={app.people.length >= MAX_PEOPLE}
                                    >
                                        <Plus class="size-4" />
                                        Add Employee
                                    </Button>
                                </div>
                            </Field.FieldSet>
                        {/if}

                        <!-- ═══ Hours ═══ -->
                        {#if activeTab === "Hours"}
                            <Field.FieldGroup class="gap-8">
                                <!-- Standard weekday hours — horizontal grid -->
                                <Field.FieldSet>
                                    <Field.Legend>Standard hours</Field.Legend>
                                    <Field.Description>
                                        Set weekday hour expectations for lead and support roles.
                                    </Field.Description>
                                    <div class="grid grid-cols-7 gap-2">
                                        {#each WEEKDAYS as day, i}
                                            <div class="flex flex-col items-center gap-1">
                                                <span
                                                    class="text-xs font-semibold text-muted-foreground uppercase tracking-wider"
                                                    >{day}</span
                                                >
                                                <span class="text-[10px] text-muted-foreground"
                                                    >Lead</span
                                                >
                                                <Input
                                                    type="number"
                                                    bind:value={app.weekdayHours[i][0]}
                                                    class="w-full! font-mono! text-center!"
                                                    min="0"
                                                />
                                                <span class="text-[10px] text-muted-foreground"
                                                    >Supp</span
                                                >
                                                <Input
                                                    type="number"
                                                    bind:value={app.weekdayHours[i][1]}
                                                    class="w-full! font-mono! text-center!"
                                                    min="0"
                                                />
                                            </div>
                                        {/each}
                                    </div>
                                </Field.FieldSet>

                                <!-- Bank holiday default hours — horizontal grid -->
                                <Field.FieldSet>
                                    <Field.Legend>Bank holiday defaults</Field.Legend>
                                    <Field.Description>
                                        Default hours applied to bank holiday dates, keyed by the
                                        weekday the holiday falls on.
                                    </Field.Description>
                                    <div class="grid grid-cols-7 gap-2">
                                        {#each WEEKDAYS as day, i}
                                            <div class="flex flex-col items-center gap-1">
                                                <span
                                                    class="text-xs font-semibold text-muted-foreground uppercase tracking-wider"
                                                    >{day}</span
                                                >
                                                <span class="text-[10px] text-muted-foreground"
                                                    >Lead</span
                                                >
                                                <Input
                                                    type="number"
                                                    bind:value={app.bankHolidayDefaultHours[i][0]}
                                                    class="w-full! font-mono! text-center!"
                                                    min="0"
                                                />
                                                <span class="text-[10px] text-muted-foreground"
                                                    >Supp</span
                                                >
                                                <Input
                                                    type="number"
                                                    bind:value={app.bankHolidayDefaultHours[i][1]}
                                                    class="w-full! font-mono! text-center!"
                                                    min="0"
                                                />
                                            </div>
                                        {/each}
                                    </div>
                                </Field.FieldSet>

                                <!-- Bank holidays -->
                                <Separator.Root />
                                <Field.FieldSet>
                                    <Field.Legend>Bank holidays</Field.Legend>
                                    <Field.Description>
                                        Override hours for specific bank holidays. Empty fields fall
                                        back to the bank holiday defaults above.
                                    </Field.Description>
                                    <div class="flex items-center justify-between">
                                        <span></span>
                                        <button
                                            onclick={refetchBankHolidays}
                                            class="flex items-center gap-1.5 text-sm text-blue-500 hover:text-blue-400 cursor-pointer bg-transparent border-none py-1 px-2 rounded-md hover:bg-accent"
                                        >
                                            <RefreshCw size={13} />
                                            Refetch
                                        </button>
                                    </div>

                                    {#if app.bankHolidays.length === 0}
                                        <span class="text-sm text-muted-foreground italic"
                                            >No bank holidays loaded. Click Refetch.</span
                                        >
                                    {:else}
                                        <div class="flex flex-col gap-2">
                                            {#each app.bankHolidays as bh, i (bh.date)}
                                                {@const defaults = bankHolidayDefaults(bh.date)}
                                                <div
                                                    class="rounded-lg border border-border bg-background p-3 flex flex-col gap-2"
                                                    class:opacity-50={!bh.enabled}
                                                >
                                                    <div class="flex items-center gap-2">
                                                        <button
                                                            class="cursor-pointer bg-transparent border-none p-0 flex items-center"
                                                            onclick={() =>
                                                                (bh.enabled = !bh.enabled)}
                                                        >
                                                            {#if bh.enabled}
                                                                <span
                                                                    class="w-4 h-4 rounded border-2 border-blue-500 bg-blue-500 flex items-center justify-center"
                                                                >
                                                                    <svg
                                                                        viewBox="0 0 10 10"
                                                                        class="w-2.5 h-2.5 text-white"
                                                                        ><path
                                                                            d="M2 5l2 2 4-4"
                                                                            fill="none"
                                                                            stroke="currentColor"
                                                                            stroke-width="1.5"
                                                                            stroke-linecap="round"
                                                                            stroke-linejoin="round"
                                                                        /></svg
                                                                    >
                                                                </span>
                                                            {:else}
                                                                <span
                                                                    class="w-4 h-4 rounded border-2 border-muted-foreground/40"
                                                                ></span>
                                                            {/if}
                                                        </button>
                                                        <span
                                                            class="flex-1 text-sm font-medium truncate"
                                                            >{bh.name}</span
                                                        >
                                                        <span
                                                            class="text-xs text-muted-foreground font-mono"
                                                        >
                                                            {format(
                                                                parseISO(bh.date),
                                                                "d MMM yyyy",
                                                            )}
                                                        </span>
                                                        <button
                                                            onclick={() => removeBankHoliday(i)}
                                                            class="text-muted-foreground hover:text-destructive cursor-pointer bg-transparent border-none p-0"
                                                        >
                                                            <X size={14} />
                                                        </button>
                                                    </div>

                                                    <div class="flex items-center gap-3">
                                                        <Crown
                                                            size={14}
                                                            class="text-muted-foreground shrink-0"
                                                        />
                                                        <Input
                                                            type="number"
                                                            bind:value={bh.lead_hours as any}
                                                            placeholder={defaults[0].toString()}
                                                            class="w-20! font-mono! text-sm!"
                                                            min="0"
                                                        />
                                                        <Shield
                                                            size={14}
                                                            class="text-muted-foreground shrink-0"
                                                        />
                                                        <Input
                                                            type="number"
                                                            bind:value={bh.support_hours as any}
                                                            placeholder={defaults[1].toString()}
                                                            class="w-20! font-mono! text-sm!"
                                                            min="0"
                                                        />
                                                    </div>
                                                </div>
                                            {/each}
                                        </div>
                                    {/if}
                                </Field.FieldSet>
                            </Field.FieldGroup>
                        {/if}

                        <!-- ═══ Overrides ═══ -->
                        {#if activeTab === "Overrides"}
                            <Field.FieldSet>
                                <Field.Legend>Custom date overrides</Field.Legend>
                                <Field.Description>
                                    Manually set hours for specific dates and roles.
                                </Field.Description>

                                <div class="overflow-hidden rounded-lg border bg-card">
                                    {#each app.customOverrides as ov, i}
                                        <Field.Field
                                            orientation="horizontal"
                                            class="border-b px-6 py-4"
                                        >
                                            <Field.Content>
                                                <Input
                                                    type="date"
                                                    bind:value={ov.date}
                                                    class="font-mono"
                                                />
                                            </Field.Content>
                                            <div class="flex items-center gap-2">
                                                <Toggle.Root
                                                    pressed={ov.role === "Lead"}
                                                    onPressedChange={() =>
                                                        (ov.role =
                                                            ov.role === "Lead"
                                                                ? "Support"
                                                                : "Lead")}
                                                    class="h-9! w-9! p-0! rounded-lg! border! border-border!"
                                                >
                                                    {#if ov.role === "Lead"}
                                                        <Crown size={14} />
                                                    {:else}
                                                        <Shield size={14} />
                                                    {/if}
                                                </Toggle.Root>
                                                <Input
                                                    type="number"
                                                    bind:value={ov.hours}
                                                    class="w-20! font-mono!"
                                                    min="0"
                                                />
                                                <button
                                                    onclick={() => removeOverride(i)}
                                                    class="text-muted-foreground hover:text-destructive cursor-pointer bg-transparent border-none p-0"
                                                >
                                                    <X size={14} />
                                                </button>
                                            </div>
                                        </Field.Field>
                                    {:else}
                                        <div class="px-6 py-4">
                                            <span class="text-sm text-muted-foreground">
                                                No overrides yet.
                                            </span>
                                        </div>
                                    {/each}

                                    <div class="px-6 py-3">
                                        <button
                                            onclick={addOverride}
                                            class="flex items-center gap-1.5 text-sm text-blue-500 hover:text-blue-400 cursor-pointer bg-transparent border-none"
                                        >
                                            <Plus size={14} />
                                            Add Override
                                        </button>
                                    </div>
                                </div>
                            </Field.FieldSet>
                        {/if}

                        <!-- ═══ Solver ═══ -->
                        {#if activeTab === "Solver"}
                            <Field.FieldGroup class="gap-8">
                                {#each solverPhases as phase}
                                    <Field.FieldSet>
                                        <Field.Legend>{phase.label}</Field.Legend>
                                        <Field.Description>
                                            Configure how the solver generates candidates for {phase.label.toLowerCase()}
                                            assignments.
                                        </Field.Description>

                                        <div class="overflow-hidden rounded-lg border bg-card">
                                            <Field.Field
                                                orientation="horizontal"
                                                class="border-b px-6 py-4"
                                            >
                                                <Field.Content>
                                                    <Field.Label>Permutations</Field.Label>
                                                    <Field.Description>
                                                        Number of candidate schedules to generate.
                                                    </Field.Description>
                                                </Field.Content>
                                                <Input
                                                    type="number"
                                                    bind:value={
                                                        app.solverParams[phase.key]
                                                            .number_permutations
                                                    }
                                                    class="w-24 font-mono"
                                                    min="1"
                                                />
                                            </Field.Field>

                                            <Field.Field orientation="horizontal" class="px-6 py-4">
                                                <Field.Content>
                                                    <Field.Label>Resolve attempts</Field.Label>
                                                    <Field.Description>
                                                        Maximum attempts to resolve conflicts per
                                                        candidate.
                                                    </Field.Description>
                                                </Field.Content>
                                                <Input
                                                    type="number"
                                                    bind:value={
                                                        app.solverParams[phase.key]
                                                            .max_resolve_attempts
                                                    }
                                                    class="w-24 font-mono"
                                                    min="1"
                                                />
                                            </Field.Field>
                                        </div>
                                    </Field.FieldSet>
                                {/each}
                            </Field.FieldGroup>
                        {/if}

                        <!-- ═══ Refiner ═══ -->
                        {#if activeTab === "Refiner"}
                            <Field.FieldSet>
                                <Field.Legend>Simulated annealing</Field.Legend>
                                <Field.Description>
                                    Tune the refinement pass that improves solver results.
                                </Field.Description>

                                <div class="overflow-hidden rounded-lg border bg-card">
                                    <Field.Field
                                        orientation="horizontal"
                                        class="border-b px-6 py-4"
                                    >
                                        <Field.Content>
                                            <Field.Label>Iterations</Field.Label>
                                            <Field.Description>
                                                Total simulated annealing steps per search.
                                            </Field.Description>
                                        </Field.Content>
                                        <Input
                                            type="number"
                                            bind:value={app.refinerParams.num_iterations}
                                            class="w-24 font-mono"
                                            min="0"
                                        />
                                    </Field.Field>

                                    <Field.Field
                                        orientation="horizontal"
                                        class="border-b px-6 py-4"
                                    >
                                        <Field.Content>
                                            <Field.Label>Temperature</Field.Label>
                                            <Field.Description>
                                                Starting temperature for acceptance probability.
                                            </Field.Description>
                                        </Field.Content>
                                        <Input
                                            type="number"
                                            bind:value={app.refinerParams.initial_temperature}
                                            class="w-24 font-mono"
                                            step="0.1"
                                            min="0"
                                        />
                                    </Field.Field>

                                    <Field.Field
                                        orientation="horizontal"
                                        class="border-b px-6 py-4"
                                    >
                                        <Field.Content>
                                            <Field.Label>Cooling rate</Field.Label>
                                            <Field.Description>
                                                Multiplicative decay per iteration (0–1).
                                            </Field.Description>
                                        </Field.Content>
                                        <Input
                                            type="number"
                                            bind:value={app.refinerParams.cooling_rate}
                                            class="w-24 font-mono"
                                            step="0.001"
                                            min="0.001"
                                            max="0.999"
                                        />
                                    </Field.Field>

                                    <Field.Field
                                        orientation="horizontal"
                                        class="border-b px-6 py-4"
                                    >
                                        <Field.Content>
                                            <Field.Label>Searches</Field.Label>
                                            <Field.Description>
                                                Number of independent annealing runs.
                                            </Field.Description>
                                        </Field.Content>
                                        <Input
                                            type="number"
                                            bind:value={app.refinerParams.searches}
                                            class="w-24 font-mono"
                                            min="1"
                                        />
                                    </Field.Field>

                                    <Field.Field orientation="horizontal" class="px-6 py-4">
                                        <Field.Content>
                                            <Field.Label>Polish</Field.Label>
                                            <Field.Description>
                                                Run a greedy final pass to clean up the result.
                                            </Field.Description>
                                        </Field.Content>

                                        <Switch bind:checked={app.refinerParams.polish} />
                                    </Field.Field>
                                </div>
                            </Field.FieldSet>
                        {/if}

                        <!-- ═══ Weights ═══ -->
                        {#if activeTab === "Weights"}
                            <Field.FieldSet>
                                <Field.Legend>Fitness weights</Field.Legend>
                                <Field.Description>
                                    Adjust how the solver and refiner penalise different quality
                                    aspects.
                                </Field.Description>

                                <div class="overflow-hidden rounded-lg border bg-card">
                                    {#each weightsDefs as wd, idx}
                                        <Field.Field
                                            orientation="horizontal"
                                            class={idx < weightsDefs.length - 1
                                                ? "border-b px-6 py-4"
                                                : "px-6 py-4"}
                                        >
                                            <Field.Content>
                                                <Field.Label>{wd.label}</Field.Label>
                                                <Field.Description>
                                                    {wd.desc}
                                                </Field.Description>
                                            </Field.Content>
                                            <Input
                                                type="number"
                                                bind:value={app.weights[wd.key] as any}
                                                class="w-24 font-mono"
                                                min="0"
                                            />
                                        </Field.Field>
                                    {/each}
                                </div>
                            </Field.FieldSet>
                        {/if}

                        <!-- ═══ Advanced ═══ -->
                        {#if activeTab === "Advanced"}
                            <Field.FieldSet>
                                <Field.Legend>Pipeline</Field.Legend>
                                <Field.Description>
                                    Control how the orchestration pipeline operates.
                                </Field.Description>

                                <div class="overflow-hidden rounded-lg border bg-card">
                                    <Field.Field orientation="horizontal" class="px-6 py-4">
                                        <Field.Content>
                                            <Field.Label>Top‑K</Field.Label>
                                            <Field.Description>
                                                Number of best solver candidates forwarded to the
                                                refiner.
                                            </Field.Description>
                                        </Field.Content>
                                        <Input
                                            type="number"
                                            bind:value={app.topK}
                                            class="w-24 font-mono"
                                            min="1"
                                        />
                                    </Field.Field>
                                </div>
                            </Field.FieldSet>
                        {/if}
                    </div>
                </ScrollArea>
            </main>
        </Sidebar.Provider>
    </Dialog.Content>
</Dialog.Root>

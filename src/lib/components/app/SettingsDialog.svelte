<script lang="ts">
    import {
        Clock,
        Cog,
        Cpu,
        Crown,
        FileText,
        Flame,
        Pencil,
        Plus,
        RefreshCw,
        Scale,
        Settings,
        Shield,
        X,
    } from "@lucide/svelte";
    import { format, getISODay, parseISO } from "date-fns";

    import { apiBankHolidays } from "$lib/api.js";
    import { app } from "$lib/app.svelte.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import * as Dialog from "$lib/components/ui/dialog/index.js";
    import * as Field from "$lib/components/ui/field/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import ScrollArea from "$lib/components/ui/scroll-area/scroll-area.svelte";
    import * as Separator from "$lib/components/ui/separator/index.js";
    import * as Sidebar from "$lib/components/ui/sidebar/index.js";
    import * as Toggle from "$lib/components/ui/toggle/index.js";
    import * as Tooltip from "$lib/components/ui/tooltip/index.js";
    import { HOLIDAY_NAMES, WEEKDAYS } from "$lib/defs.js";
    import { useHotKey } from "$lib/hooks/useHotkey.svelte";

    import Kbd from "../ui/kbd/kbd.svelte";

    /* === Nav === */
    const nav = [
        { name: "Details", icon: FileText },
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

    <Dialog.Content class="overflow-hidden p-0 sm:max-h-[85vh] sm:max-w-4xl">
        <Dialog.Title class="sr-only">Settings</Dialog.Title>
        <Dialog.Description class="sr-only">Configure schedule parameters.</Dialog.Description>
        <Sidebar.Provider class="items-start">
            <Sidebar.Root collapsible="none" class="hidden md:flex border-r">
                <Sidebar.Content>
                    <Sidebar.Group>
                        <Sidebar.GroupLabel>Configuration</Sidebar.GroupLabel>
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

            <main class="flex-1 flex flex-col min-h-0">
                <ScrollArea class="flex-1">
                    <div class="p-6">
                        <!-- ═══ Details ═══ -->
                        {#if activeTab === "Details"}
                            <Field.FieldSet>
                                <Field.FieldTitle>Problem details</Field.FieldTitle>

                                <Field.Field orientation="horizontal">
                                    <Field.FieldLabel class="w-40 shrink-0 text-right"
                                        >Start date</Field.FieldLabel
                                    >
                                    <Field.FieldContent>
                                        <Input
                                            type="date"
                                            bind:value={app.startDate}
                                            class="font-mono!"
                                        />
                                        <Field.FieldDescription>
                                            First Monday of the schedule period.
                                        </Field.FieldDescription>
                                    </Field.FieldContent>
                                </Field.Field>

                                <Field.Field orientation="horizontal">
                                    <Field.FieldLabel class="w-40 shrink-0 text-right"
                                        >Employees</Field.FieldLabel
                                    >
                                    <Field.FieldContent>
                                        <span class="text-sm font-mono tabular-nums">
                                            {app.people.length} configured
                                        </span>
                                        <Field.FieldDescription>
                                            Manage employees in the Employees dialog.
                                        </Field.FieldDescription>
                                    </Field.FieldContent>
                                </Field.Field>

                                <Field.Field orientation="horizontal">
                                    <Field.FieldLabel class="w-40 shrink-0 text-right"
                                        >Skip last shifts</Field.FieldLabel
                                    >
                                    <Field.FieldContent>
                                        <Input
                                            type="number"
                                            bind:value={app.skipLastShifts}
                                            min="0"
                                            class="w-24! font-mono!"
                                        />
                                        <Field.FieldDescription>
                                            Exclude the final N workdays from scheduling.
                                        </Field.FieldDescription>
                                    </Field.FieldContent>
                                </Field.Field>
                            </Field.FieldSet>
                        {/if}

                        <!-- ═══ Hours ═══ -->
                        {#if activeTab === "Hours"}
                            <Field.FieldGroup class="gap-8">
                                <!-- Standard weekday hours — horizontal grid -->
                                <Field.FieldSet>
                                    <Field.FieldTitle>Standard hours</Field.FieldTitle>
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
                                    <Field.FieldTitle>Bank holiday defaults</Field.FieldTitle>
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
                                    <div class="flex items-center justify-between">
                                        <Field.FieldTitle class="mb-0!"
                                            >Bank holidays</Field.FieldTitle
                                        >
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
                                <Field.FieldTitle>Custom date overrides</Field.FieldTitle>

                                <div class="flex flex-col gap-2">
                                    {#each app.customOverrides as ov, i}
                                        <div class="flex items-center gap-2">
                                            <Input
                                                type="date"
                                                bind:value={ov.date}
                                                class="flex-1! font-mono!"
                                            />
                                            <Toggle.Root
                                                pressed={ov.role === "Lead"}
                                                onPressedChange={() =>
                                                    (ov.role =
                                                        ov.role === "Lead" ? "Support" : "Lead")}
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
                                    {/each}

                                    <button
                                        onclick={addOverride}
                                        class="flex items-center gap-1.5 text-sm text-blue-500 hover:text-blue-400 cursor-pointer bg-transparent border-none px-0 py-1"
                                    >
                                        <Plus size={14} />
                                        Add Override
                                    </button>
                                </div>
                            </Field.FieldSet>
                        {/if}

                        <!-- ═══ Solver ═══ -->
                        {#if activeTab === "Solver"}
                            <Field.FieldGroup class="gap-8">
                                {#each solverPhases as phase}
                                    <Field.FieldSet>
                                        <Field.FieldTitle>{phase.label}</Field.FieldTitle>

                                        <Field.Field orientation="horizontal">
                                            <Field.FieldLabel class="w-44 shrink-0 text-right"
                                                >Permutations</Field.FieldLabel
                                            >
                                            <Field.FieldContent>
                                                <Input
                                                    type="number"
                                                    bind:value={
                                                        app.solverParams[phase.key]
                                                            .number_permutations
                                                    }
                                                    class="w-24! font-mono!"
                                                    min="1"
                                                />
                                                <Field.FieldDescription>
                                                    Number of candidate schedules to generate.
                                                </Field.FieldDescription>
                                            </Field.FieldContent>
                                        </Field.Field>

                                        <Field.Field orientation="horizontal">
                                            <Field.FieldLabel class="w-44 shrink-0 text-right"
                                                >Resolve attempts</Field.FieldLabel
                                            >
                                            <Field.FieldContent>
                                                <Input
                                                    type="number"
                                                    bind:value={
                                                        app.solverParams[phase.key]
                                                            .max_resolve_attempts
                                                    }
                                                    class="w-24! font-mono!"
                                                    min="1"
                                                />
                                                <Field.FieldDescription>
                                                    Maximum attempts to resolve conflicts per
                                                    candidate.
                                                </Field.FieldDescription>
                                            </Field.FieldContent>
                                        </Field.Field>
                                    </Field.FieldSet>
                                {/each}
                            </Field.FieldGroup>
                        {/if}

                        <!-- ═══ Refiner ═══ -->
                        {#if activeTab === "Refiner"}
                            <Field.FieldSet>
                                <Field.FieldTitle>Simulated annealing</Field.FieldTitle>

                                <Field.Field orientation="horizontal">
                                    <Field.FieldLabel class="w-44 shrink-0 text-right"
                                        >Iterations</Field.FieldLabel
                                    >
                                    <Field.FieldContent>
                                        <Input
                                            type="number"
                                            bind:value={app.refinerParams.num_iterations}
                                            class="w-24! font-mono!"
                                            min="0"
                                        />
                                        <Field.FieldDescription>
                                            Total simulated annealing steps per search.
                                        </Field.FieldDescription>
                                    </Field.FieldContent>
                                </Field.Field>

                                <Field.Field orientation="horizontal">
                                    <Field.FieldLabel class="w-44 shrink-0 text-right"
                                        >Temperature</Field.FieldLabel
                                    >
                                    <Field.FieldContent>
                                        <Input
                                            type="number"
                                            bind:value={app.refinerParams.initial_temperature}
                                            class="w-24! font-mono!"
                                            step="0.1"
                                            min="0"
                                        />
                                        <Field.FieldDescription>
                                            Starting temperature for acceptance probability.
                                        </Field.FieldDescription>
                                    </Field.FieldContent>
                                </Field.Field>

                                <Field.Field orientation="horizontal">
                                    <Field.FieldLabel class="w-44 shrink-0 text-right"
                                        >Cooling rate</Field.FieldLabel
                                    >
                                    <Field.FieldContent>
                                        <Input
                                            type="number"
                                            bind:value={app.refinerParams.cooling_rate}
                                            class="w-24! font-mono!"
                                            step="0.001"
                                            min="0.001"
                                            max="0.999"
                                        />
                                        <Field.FieldDescription>
                                            Multiplicative decay per iteration (0–1).
                                        </Field.FieldDescription>
                                    </Field.FieldContent>
                                </Field.Field>

                                <Field.Field orientation="horizontal">
                                    <Field.FieldLabel class="w-44 shrink-0 text-right"
                                        >Searches</Field.FieldLabel
                                    >
                                    <Field.FieldContent>
                                        <Input
                                            type="number"
                                            bind:value={app.refinerParams.searches}
                                            class="w-24! font-mono!"
                                            min="1"
                                        />
                                        <Field.FieldDescription>
                                            Number of independent annealing runs.
                                        </Field.FieldDescription>
                                    </Field.FieldContent>
                                </Field.Field>

                                <Field.Field orientation="horizontal">
                                    <Field.FieldLabel class="w-44 shrink-0 text-right"
                                        >Polish</Field.FieldLabel
                                    >
                                    <Field.FieldContent>
                                        <button
                                            class="cursor-pointer bg-transparent border-none p-0 flex items-center"
                                            onclick={() =>
                                                (app.refinerParams.polish =
                                                    !app.refinerParams.polish)}
                                        >
                                            {#if app.refinerParams.polish}
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
                                        <Field.FieldDescription>
                                            Run a greedy final pass to clean up the result.
                                        </Field.FieldDescription>
                                    </Field.FieldContent>
                                </Field.Field>
                            </Field.FieldSet>
                        {/if}

                        <!-- ═══ Weights ═══ -->
                        {#if activeTab === "Weights"}
                            <Field.FieldSet>
                                <Field.FieldTitle>Fitness weights</Field.FieldTitle>

                                {#each weightsDefs as wd}
                                    <Field.Field orientation="horizontal">
                                        <Field.FieldLabel class="w-44 shrink-0 text-right"
                                            >{wd.label}</Field.FieldLabel
                                        >
                                        <Field.FieldContent>
                                            <Input
                                                type="number"
                                                bind:value={app.weights[wd.key] as any}
                                                class="w-24! font-mono!"
                                                min="0"
                                            />
                                            <Field.FieldDescription>
                                                {wd.desc}
                                            </Field.FieldDescription>
                                        </Field.FieldContent>
                                    </Field.Field>
                                {/each}
                            </Field.FieldSet>
                        {/if}

                        <!-- ═══ Advanced ═══ -->
                        {#if activeTab === "Advanced"}
                            <Field.FieldSet>
                                <Field.FieldTitle>Pipeline</Field.FieldTitle>

                                <Field.Field orientation="horizontal">
                                    <Field.FieldLabel class="w-44 shrink-0 text-right"
                                        >Top‑K</Field.FieldLabel
                                    >
                                    <Field.FieldContent>
                                        <Input
                                            type="number"
                                            bind:value={app.topK}
                                            class="w-24! font-mono!"
                                            min="1"
                                        />
                                        <Field.FieldDescription>
                                            Number of best solver candidates forwarded to the
                                            refiner.
                                        </Field.FieldDescription>
                                    </Field.FieldContent>
                                </Field.Field>
                            </Field.FieldSet>
                        {/if}
                    </div>
                </ScrollArea>
            </main>
        </Sidebar.Provider>
    </Dialog.Content>
</Dialog.Root>

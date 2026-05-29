<script lang="ts">
    import { ChevronDown, ChevronRight, Crown, Plus, RefreshCw, Shield, X } from "@lucide/svelte";
    import { format, getISODay, parseISO } from "date-fns";

    import { apiBankHolidays } from "$lib/api.js";
    import { app } from "$lib/app.svelte";
    import { Input } from "$lib/components/ui/input/index.js";
    import { Separator } from "$lib/components/ui/separator/index.js";
    import * as Tabs from "$lib/components/ui/tabs/index.js";
    import * as Toggle from "$lib/components/ui/toggle/index.js";
    import { HOLIDAY_NAMES } from "$lib/defs.js";

    import EmployeeDialog from "./EmployeeDialog.svelte";
    import HoursGrid from "./HoursGrid.svelte";

    /* === Section toggles === */
    let sections = $state({
        problem: true,
        hours: true,
        overrides: false,
        solver: false,
        refiner: false,
        costs: false,
        orchestration: false,
    });

    function toggleSection(key: keyof typeof sections) {
        sections[key] = !sections[key];
    }

    /* === Hours tab state === */
    let hoursTab = $state("standard");

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
    let overrideRole = $state<"Lead" | "Support">("Lead");

    function addOverride() {
        app.customOverrides.push({ date: "", role: overrideRole, hours: 0 });
    }

    function removeOverride(index: number) {
        app.customOverrides.splice(index, 1);
    }

    /* === Costs weights friendly labels === */
    const weightDefs: { key: keyof typeof app.weights; label: string; desc: string }[] = [
        { key: "annual_hours", label: "Annual Hours", desc: "Penalty for deviating from expected annual total" },
        { key: "consecutive_days", label: "Consecutive Days", desc: "Penalty for long work stretches without a break" },
        { key: "consecutive_weekends", label: "Weekend Runs", desc: "Penalty for working too many weekends in a row" },
        { key: "weekend_alternation", label: "Weekend Balance", desc: "Encourages fair weekend rotation" },
        { key: "weekend_regularity", label: "Weekend Rhythm", desc: "Penalty for irregular weekend patterns" },
        { key: "weekly_hours", label: "Weekly Hours", desc: "Penalty when weekly hours stray from the target" },
        { key: "blank_weeks", label: "Blank Weeks", desc: "Heavy penalty for unassigned weeks" },
    ];
</script>

<div class="flex flex-col">
    <!-- ═══ Problem ═══ -->
    <div class="border-b border-border pb-1 mb-1">
        <button
            class="flex items-center gap-1.5 w-full px-3.5 py-2 bg-transparent border-none text-[10px] font-semibold uppercase tracking-[0.02em] cursor-pointer text-left hover:bg-accent"
            onclick={() => toggleSection("problem")}
        >
            {#if sections.problem}<ChevronDown size={13} />{:else}<ChevronRight size={13} />{/if}
            <span>Problem</span>
        </button>

        {#if sections.problem}
            <div class="px-3.5 py-1 pb-2 flex flex-col gap-2">
                <div class="flex items-center gap-2">
                    <label
                        for="cfg-start"
                        class="text-[11.5px] text-muted-foreground w-27 shrink-0"
                        >Start date</label
                    >
                    <Input
                        id="cfg-start"
                        type="date"
                        bind:value={app.startDate}
                        class="flex-1! h-7! text-[11.5px]! font-mono!"
                    />
                </div>

                <div class="flex items-center gap-2">
                    <label class="text-[11.5px] text-muted-foreground w-27 shrink-0"
                        >Employees</label
                    >
                    <span class="text-[11.5px] font-mono">{app.people.length} configured</span>
                    <EmployeeDialog />
                </div>

                <div class="flex items-center gap-2">
                    <label class="text-[11.5px] text-muted-foreground w-27 shrink-0"
                        >Skip last shifts</label
                    >
                    <Input
                        type="number"
                        bind:value={app.skipLastShifts}
                        class="flex-1! h-7! text-[11.5px]! font-mono!"
                        min="0"
                    />
                </div>
            </div>
        {/if}
    </div>

    <!-- ═══ Hours ═══ -->
    <div class="border-b border-border pb-1 mb-1">
        <button
            class="flex items-center gap-1.5 w-full px-3.5 py-2 bg-transparent border-none text-[10px] font-semibold uppercase tracking-[0.02em] cursor-pointer text-left hover:bg-accent"
            onclick={() => toggleSection("hours")}
        >
            {#if sections.hours}<ChevronDown size={13} />{:else}<ChevronRight size={13} />{/if}
            <span>Hours</span>
        </button>

        {#if sections.hours}
            <div class="px-3.5 py-1 pb-2 flex flex-col gap-3">
                <!-- Tabs: Standard / Bank -->
                <Tabs.Tabs bind:value={hoursTab}>
                    <div class="flex items-center gap-2">
                        <Tabs.List>
                            <Tabs.Trigger value="standard">Standard</Tabs.Trigger>
                            <Tabs.Trigger value="bank">Bank</Tabs.Trigger>
                        </Tabs.List>
                    </div>

                    <Tabs.Content value="standard">
                        <div class="mt-2">
                            <HoursGrid hours={app.weekdayHours} />
                        </div>
                    </Tabs.Content>

                    <Tabs.Content value="bank">
                        <div class="mt-2">
                            <HoursGrid hours={app.bankHolidayDefaultHours} />
                        </div>
                    </Tabs.Content>
                </Tabs.Tabs>

                <!-- Bank holidays list -->
                <Separator />

                <div class="flex flex-col gap-2">
                    <div class="flex items-center justify-between">
                        <span class="text-[10px] font-semibold uppercase tracking-[0.05em] text-muted-foreground"
                            >Bank Holidays</span
                        >
                        <button
                            onclick={refetchBankHolidays}
                            class="flex items-center gap-1 text-[10.5px] text-blue-500 hover:text-blue-400 cursor-pointer bg-transparent border-none"
                        >
                            <RefreshCw size={11} />
                            Refetch
                        </button>
                    </div>

                    {#if app.bankHolidays.length === 0}
                        <span class="text-[11px] text-muted-foreground italic"
                            >No bank holidays loaded. Click Refetch.</span
                        >
                    {:else}
                        {#each app.bankHolidays as bh, i (bh.date)}
                            {@const defaults = bankHolidayDefaults(bh.date)}
                            <div
                                class="rounded-md border border-border bg-background p-2 flex flex-col gap-1.5"
                                class:opacity-50={!bh.enabled}
                            >
                                <div class="flex items-center gap-1.5">
                                    <!-- Enabled toggle -->
                                    <button
                                        class="cursor-pointer bg-transparent border-none p-0 flex items-center"
                                        onclick={() => (bh.enabled = !bh.enabled)}
                                    >
                                        {#if bh.enabled}
                                            <span class="w-3.5 h-3.5 rounded border-2 border-blue-500 bg-blue-500 flex items-center justify-center">
                                                <svg viewBox="0 0 10 10" class="w-2 h-2 text-white"><path d="M2 5l2 2 4-4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
                                            </span>
                                        {:else}
                                            <span class="w-3.5 h-3.5 rounded border-2 border-muted-foreground/40"></span>
                                        {/if}
                                    </button>
                                    <span class="flex-1 text-[11.5px] font-medium truncate"
                                        >{bh.name}</span
                                    >
                                    <button
                                        onclick={() => removeBankHoliday(i)}
                                        class="text-muted-foreground hover:text-foreground cursor-pointer bg-transparent border-none p-0"
                                    >
                                        <X size={11} />
                                    </button>
                                </div>

                                <div class="text-[10.5px] text-muted-foreground">
                                    {format(parseISO(bh.date), "d MMM yyyy")}
                                </div>

                                <!-- Lead / Support override inputs -->
                                <div class="flex items-center gap-2">
                                    <Crown size={12} class="text-muted-foreground shrink-0" />
                                    <Input
                                        type="number"
                                        bind:value={bh.lead_hours as any}
                                        placeholder={defaults[0].toString()}
                                        class="w-14! h-6.5! text-[11px]! font-mono! text-right!"
                                    />
                                    <Shield size={12} class="text-muted-foreground shrink-0 ml-1" />
                                    <Input
                                        type="number"
                                        bind:value={bh.support_hours as any}
                                        placeholder={defaults[1].toString()}
                                        class="w-14! h-6.5! text-[11px]! font-mono! text-right!"
                                    />
                                </div>
                            </div>
                        {/each}
                    {/if}
                </div>
            </div>
        {/if}
    </div>

    <!-- ═══ Overrides ═══ -->
    <div class="border-b border-border pb-1 mb-1">
        <button
            class="flex items-center gap-1.5 w-full px-3.5 py-2 bg-transparent border-none text-[10px] font-semibold uppercase tracking-[0.02em] cursor-pointer text-left hover:bg-accent"
            onclick={() => toggleSection("overrides")}
        >
            {#if sections.overrides}<ChevronDown size={13} />{:else}<ChevronRight size={13} />{/if}
            <span>Overrides</span>
        </button>

        {#if sections.overrides}
            <div class="px-3.5 py-1 pb-2 flex flex-col gap-2">
                {#each app.customOverrides as ov, i}
                    <div class="flex items-center gap-1.5">
                        <Input
                            type="date"
                            bind:value={ov.date}
                            class="flex-1! h-7! text-[11px]! font-mono!"
                        />
                        <Toggle.Root
                            pressed={ov.role === "Lead"}
                            onPressedChange={() => (ov.role = ov.role === "Lead" ? "Support" : "Lead")}
                            class="h-7! w-7! p-0! rounded-md! border! border-border!"
                        >
                            {#if ov.role === "Lead"}
                                <Crown size={12} />
                            {:else}
                                <Shield size={12} />
                            {/if}
                        </Toggle.Root>
                        <Input
                            type="number"
                            bind:value={ov.hours}
                            class="w-14! h-7! text-[11px]! font-mono! text-right!"
                        />
                        <button
                            onclick={() => removeOverride(i)}
                            class="text-muted-foreground hover:text-destructive cursor-pointer bg-transparent border-none p-0"
                        >
                            <X size={12} />
                        </button>
                    </div>
                {/each}

                <button
                    onclick={addOverride}
                    class="flex items-center gap-1.5 text-[11.5px] text-blue-500 hover:text-blue-400 cursor-pointer bg-transparent border-none px-0 py-1"
                >
                    <Plus size={12} />
                    Add Override
                </button>
            </div>
        {/if}
    </div>

    <!-- ═══ Solver ═══ -->
    <div class="border-b border-border pb-1 mb-1">
        <button
            class="flex items-center gap-1.5 w-full px-3.5 py-2 bg-transparent border-none text-[10px] font-semibold uppercase tracking-[0.02em] cursor-pointer text-left hover:bg-accent"
            onclick={() => toggleSection("solver")}
        >
            {#if sections.solver}<ChevronDown size={13} />{:else}<ChevronRight size={13} />{/if}
            <span>Solver</span>
        </button>

        {#if sections.solver}
            <div class="px-3.5 py-1 pb-2 flex flex-col gap-2.5">
                {#each [
                    { key: "weekend", label: "Weekend" },
                    { key: "friday", label: "Friday" },
                    { key: "weekday", label: "Weekday" },
                ] as phase}
                    <div class="flex flex-col gap-1">
                        <span class="text-[9.5px] font-semibold uppercase tracking-[0.05em] text-muted-foreground"
                            >{phase.label}</span
                        >
                        <div class="flex items-center gap-2">
                            <label class="text-[10.5px] text-muted-foreground w-16 shrink-0"
                                >Permutations</label
                            >
                            <Input
                                type="number"
                                bind:value={app.solverParams[phase.key as keyof typeof app.solverParams].number_permutations}
                                class="w-16! h-7! text-[11px]! font-mono! text-right!"
                                min="1"
                            />
                        </div>
                        <div class="flex items-center gap-2">
                            <label class="text-[10.5px] text-muted-foreground w-16 shrink-0"
                                >Resolve att.</label
                            >
                            <Input
                                type="number"
                                bind:value={app.solverParams[phase.key as keyof typeof app.solverParams].max_resolve_attempts}
                                class="w-16! h-7! text-[11px]! font-mono! text-right!"
                                min="1"
                            />
                        </div>
                    </div>
                {/each}
            </div>
        {/if}
    </div>

    <!-- ═══ Refiner ═══ -->
    <div class="border-b border-border pb-1 mb-1">
        <button
            class="flex items-center gap-1.5 w-full px-3.5 py-2 bg-transparent border-none text-[10px] font-semibold uppercase tracking-[0.02em] cursor-pointer text-left hover:bg-accent"
            onclick={() => toggleSection("refiner")}
        >
            {#if sections.refiner}<ChevronDown size={13} />{:else}<ChevronRight size={13} />{/if}
            <span>Refiner</span>
        </button>

        {#if sections.refiner}
            <div class="px-3.5 py-1 pb-2 flex flex-col gap-2">
                <div class="flex items-center gap-2">
                    <label class="text-[11px] text-muted-foreground w-24 shrink-0"
                        >Iterations</label
                    >
                    <Input
                        type="number"
                        bind:value={app.refinerParams.num_iterations}
                        class="w-20! h-7! text-[11px]! font-mono! text-right!"
                        min="0"
                    />
                </div>
                <div class="flex items-center gap-2">
                    <label class="text-[11px] text-muted-foreground w-24 shrink-0"
                        >Temperature</label
                    >
                    <Input
                        type="number"
                        bind:value={app.refinerParams.initial_temperature}
                        class="w-20! h-7! text-[11px]! font-mono! text-right!"
                        step="0.1"
                        min="0"
                    />
                </div>
                <div class="flex items-center gap-2">
                    <label class="text-[11px] text-muted-foreground w-24 shrink-0"
                        >Cooling rate</label
                    >
                    <Input
                        type="number"
                        bind:value={app.refinerParams.cooling_rate}
                        class="w-20! h-7! text-[11px]! font-mono! text-right!"
                        step="0.001"
                        min="0.001"
                        max="0.999"
                    />
                </div>
                <div class="flex items-center gap-2">
                    <label class="text-[11px] text-muted-foreground w-24 shrink-0"
                        >Searches</label
                    >
                    <Input
                        type="number"
                        bind:value={app.refinerParams.searches}
                        class="w-20! h-7! text-[11px]! font-mono! text-right!"
                        min="1"
                    />
                </div>
                <div class="flex items-center gap-2">
                    <label class="text-[11px] text-muted-foreground w-24 shrink-0">Polish</label>
                    <button
                        class="cursor-pointer bg-transparent border-none p-0 flex items-center"
                        onclick={() => (app.refinerParams.polish = !app.refinerParams.polish)}
                    >
                        {#if app.refinerParams.polish}
                            <span class="w-3.5 h-3.5 rounded border-2 border-blue-500 bg-blue-500 flex items-center justify-center">
                                <svg viewBox="0 0 10 10" class="w-2 h-2 text-white"><path d="M2 5l2 2 4-4" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round"/></svg>
                            </span>
                        {:else}
                            <span class="w-3.5 h-3.5 rounded border-2 border-muted-foreground/40"></span>
                        {/if}
                    </button>
                </div>
            </div>
        {/if}
    </div>

    <!-- ═══ Costs ═══ -->
    <div class="border-b border-border pb-1 mb-1">
        <button
            class="flex items-center gap-1.5 w-full px-3.5 py-2 bg-transparent border-none text-[10px] font-semibold uppercase tracking-[0.02em] cursor-pointer text-left hover:bg-accent"
            onclick={() => toggleSection("costs")}
        >
            {#if sections.costs}<ChevronDown size={13} />{:else}<ChevronRight size={13} />{/if}
            <span>Costs</span>
        </button>

        {#if sections.costs}
            <div class="px-3.5 py-1 pb-2 flex flex-col gap-2">
                {#each weightDefs as wd}
                    <div class="flex flex-col gap-0.5">
                        <div class="flex items-center gap-1.5">
                            <label class="text-[11px] text-muted-foreground flex-1 truncate"
                                title={wd.desc}>{wd.label}</label
                            >
                            <span class="text-[10.5px] font-mono text-muted-foreground w-8 text-right"
                                >{app.weights[wd.key]}</span
                            >
                        </div>
                        <input
                            type="range"
                            min="0"
                            max="100"
                            step="0.5"
                            value={app.weights[wd.key]}
                            oninput={(e) => (app.weights[wd.key] = Number(e.currentTarget.value))}
                            class="w-full accent-blue-500 h-1.5"
                        />
                    </div>
                {/each}
            </div>
        {/if}
    </div>

    <!-- ═══ Orchestration ═══ -->
    <div>
        <div class="flex items-center gap-2 px-3.5 py-1.5">
            <span class="text-[10px] font-semibold uppercase tracking-[0.06em] text-muted-foreground"
                >Top‑K</span
            >
            <Input
                type="number"
                bind:value={app.topK}
                class="w-14! h-7! text-[11.5px]! font-mono! text-right!"
                min="1"
            />
        </div>
    </div>
</div>

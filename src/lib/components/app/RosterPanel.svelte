<script lang="ts">
    import { Minus, Pencil, Plus } from "@lucide/svelte";

    import { app } from "$lib/app.svelte";
    import { Button } from "$lib/components/ui/button/index.js";
    import { Toggle } from "$lib/components/ui/toggle/index.js";
    import * as Tooltip from "$lib/components/ui/tooltip/index.js";
    import {
        MAX_PEOPLE,
        MIN_PEOPLE,
        NULL_SLOT,
        N_DAYS,
        N_ROLES,
        PERSON_COLORS,
        Role,
    } from "$lib/defs.js";
    import { cn } from "$lib/utils.js";

    interface EmpStats {
        index: number;
        totalShifts: number;
        leadShifts: number;
        supportShifts: number;
        hasViolations: boolean;
    }

    let edit = $state(false);

    const stats = $derived.by(() => {
        const map = new Map<number, EmpStats>();

        for (let i = 0; i < app.people.length; i++) {
            map.set(i, {
                index: i,
                totalShifts: 0,
                leadShifts: 0,
                supportShifts: 0,
                hasViolations: false,
            });
        }

        for (let day = 0; day < N_DAYS; day++) {
            for (let role = 0; role < N_ROLES; role++) {
                const slot = app.slots[day * N_ROLES + role];
                if (slot !== NULL_SLOT) {
                    const entry = map.get(slot);
                    if (entry) {
                        entry.totalShifts++;
                        if (role === Role.Lead) entry.leadShifts++;
                        else entry.supportShifts++;
                    }
                }
            }
        }

        // Mark violations from conflicts
        for (const conflict of app.conflicts) {
            if ("ConsecutiveDay" in conflict) {
                const idx = conflict.ConsecutiveDay[0];
                const entry = map.get(idx);
                if (entry) entry.hasViolations = true;
            }
            if ("Role" in conflict) {
                const idx = conflict.Role[0];
                const entry = map.get(idx);
                if (entry) entry.hasViolations = true;
            }
            if ("WorkCount" in conflict) {
                const idx = conflict.WorkCount[0];
                const entry = map.get(idx);
                if (entry) entry.hasViolations = true;
            }
        }

        return [...map.values()];
    });

    function toggle(index: number) {
        const sameIndex = app.selectedPersonIndex === index;
        app.selectedPersonIndex = sameIndex ? undefined : index;
    }

    function onadd() {
        const length = app.people.length;
        if (length >= MAX_PEOPLE) return;

        app.people.push({ holidays: [], name: `Employee ${length + 1}`, rate: 80 });
    }

    function onremove() {
        const length = app.people.length;
        if (length <= MIN_PEOPLE) return;

        app.people.pop();
    }
</script>

<aside class="w-48 shrink-0 flex flex-col border-r border-border bg-card">
    <div
        class="px-3 py-2.5 border-b border-border text-[10px] font-semibold uppercase tracking-[0.08em] text-muted-foreground"
    >
        Employees
    </div>

    <div class="flex flex-col p-1.5 gap-0.5 overflow-y-auto">
        {#each app.people as person, i}
            {@const s = stats[i]}
            {@const swatch = PERSON_COLORS[i % PERSON_COLORS.length][1]}

            <button
                class={cn(
                    "flex items-center rounded-md border border-transparent cursor-pointer text-left transition-colors overflow-hidden min-h-[44px]",
                    app.selectedPersonIndex === i
                        ? "bg-accent border-border"
                        : "bg-transparent hover:bg-accent/50",
                )}
                onclick={() => toggle(i)}
                aria-pressed={app.selectedPersonIndex === i}
                title={person.name}
            >
                <span class="w-1.25 self-stretch shrink-0 rounded-l-md {swatch}"></span>
                <div class="flex-1 min-w-0 py-1.5 px-2 flex flex-col gap-0.75">
                    <span class="text-[12.5px] font-medium text-foreground truncate"
                        >{person.name}</span
                    >
                    {#if s}
                        <div class="flex items-center gap-1.5 font-mono text-[10.5px]">
                            <span class="text-muted-foreground">{s.totalShifts}sh</span>
                            <span class="text-muted-foreground"
                                >L{s.leadShifts}/S{s.supportShifts}</span
                            >
                            {#if s.hasViolations}
                                <span
                                    class="bg-red-500 text-white text-[9.5px] font-bold rounded-full px-1.5 py-px font-mono leading-snug"
                                    >!</span
                                >
                            {/if}
                        </div>
                    {/if}
                </div>
                {#if person.rate < 100}
                    <span
                        class="text-[9.5px] font-semibold text-muted-foreground bg-secondary border border-border rounded px-1.5 py-px mr-1.5 shrink-0"
                    >
                        {person.rate}%
                    </span>
                {/if}
            </button>
        {/each}

        <div class="mt-4 flex justify-center gap-2">
            <Tooltip.Root>
                <Tooltip.Trigger>
                    <Button
                        class="flex-1"
                        variant="secondary"
                        onclick={onremove}
                        disabled={app.people.length <= MIN_PEOPLE}
                    >
                        <Minus class="size-4" />
                    </Button>
                </Tooltip.Trigger>
                <Tooltip.Content>Remove employee</Tooltip.Content>
            </Tooltip.Root>

            <Tooltip.Root>
                <Tooltip.Trigger>
                    <Button
                        class="flex-1"
                        variant="secondary"
                        onclick={onadd}
                        disabled={app.people.length >= MAX_PEOPLE}
                    >
                        <Plus class="size-4" />
                    </Button>
                </Tooltip.Trigger>
                <Tooltip.Content>Add employee</Tooltip.Content>
            </Tooltip.Root>
        </div>
    </div>
</aside>

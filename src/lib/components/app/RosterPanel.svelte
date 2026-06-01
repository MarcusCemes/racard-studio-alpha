<script lang="ts">
    import { Check } from "@lucide/svelte";

    import { app, selection } from "$lib/app.svelte.js";
    import { N_DAYS, PERSON_COLORS } from "$lib/defs.js";
    import { NULL_SLOT, getLead, getSupport } from "$lib/slot.js";
    import { cn } from "$lib/utils.js";

    interface EmpStats {
        index: number;
        totalShifts: number;
        leadShifts: number;
        supportShifts: number;
        hasViolations: boolean;
        delta?: number;
        pct?: number;
    }

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
            const slot = app.slots[day];
            if (slot === NULL_SLOT) continue;

            const lead = getLead(slot);
            const supp = getSupport(slot);
            if (lead != null) {
                const entry = map.get(lead);
                if (entry) {
                    entry.totalShifts++;
                    entry.leadShifts++;
                }
            }
            if (supp != null) {
                const entry = map.get(supp);
                if (entry) {
                    entry.totalShifts++;
                    entry.supportShifts++;
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

        // Add hour deltas from statistics
        if (app.statistics) {
            map.forEach((entry, i) => {
                const personStats = app.statistics?.people[i];
                if (personStats) {
                    const { totals } = personStats;
                    entry.delta = totals.total_hours_worked - totals.expected_hours;
                    entry.pct =
                        totals.expected_hours > 0 ? (entry.delta / totals.expected_hours) * 100 : 0;
                }
            });
        }

        return [...map.values()];
    });

    function toggle(index: number) {
        const sameIndex = selection.person === index;
        selection.person = sameIndex ? undefined : index;
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
                    selection.person === i
                        ? "bg-accent border-border"
                        : "bg-transparent hover:bg-accent/50",
                )}
                onclick={() => toggle(i)}
                aria-pressed={selection.person === i}
                title={person.name}
            >
                <span class="w-1.25 self-stretch shrink-0 rounded-l-md {swatch}"></span>
                <div class="flex-1 min-w-0 py-1.5 px-2 flex flex-col gap-0.75">
                    <span class="text-[12.5px] font-medium text-foreground truncate"
                        >{person.name}</span
                    >
                    {#if s}
                        {#if s.delta !== undefined && s.pct !== undefined}
                            <!-- Hour delta display -->
                            <div class="flex flex-col gap-0.5">
                                <div class="h-1 w-full rounded bg-border overflow-hidden">
                                    <div
                                        class={cn(
                                            "h-full transition-all",
                                            Math.abs(s.pct!) < 2
                                                ? "bg-green-500"
                                                : Math.abs(s.pct!) < 10
                                                  ? "bg-amber-500"
                                                  : "bg-red-500",
                                        )}
                                        style:width="{Math.min(Math.abs(s.pct!), 110)}%"
                                    ></div>
                                </div>
                                <div class="flex items-center gap-1 font-mono text-[10px]">
                                    <span
                                        class={cn(
                                            "min-w-[28px]",
                                            Math.abs(s.pct) < 2
                                                ? "text-green-600 dark:text-green-500"
                                                : Math.abs(s.pct) < 10
                                                  ? "text-amber-600 dark:text-amber-500"
                                                  : "text-red-600 dark:text-red-500",
                                        )}
                                    >
                                        {s.delta > 0 ? "+" : ""}{s.delta.toFixed(0)}h
                                    </span>
                                    {#if Math.abs(s.pct) < 2}
                                        <Check class="size-2.5 text-green-500" />
                                    {/if}
                                    {#if s.hasViolations}
                                        <span
                                            class="ml-auto bg-red-500 text-white text-[9px] font-bold rounded-full px-1 py-px leading-none"
                                            >!</span
                                        >
                                    {/if}
                                </div>
                            </div>
                        {:else}
                            <!-- Fallback: shift counts -->
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
    </div>
</aside>

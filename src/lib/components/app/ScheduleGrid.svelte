<script lang="ts">
    import { addDays, addWeeks, format, parseISO, startOfISOWeek } from "date-fns";
    import { untrack } from "svelte";

    import { setPerson, swapDays, swapRoles } from "$lib/actions";
    import { GridMode, app, selection, view } from "$lib/app.svelte.js";
    import { N_DAYS, N_WEEKDAYS, N_WEEKS, Role, WEEKDAYS } from "$lib/defs.js";
    import { useHotKey } from "$lib/hooks/useHotkey.svelte.js";
    import { getLead, getSupport } from "$lib/slot.js";

    // Per-person palette: [bar, bg, text]
    const PALETTE: [string, string, string][] = [
        ["#3b82f6", "#dbeafe", "#1e40af"], // blue
        ["#22c55e", "#dcfce7", "#166534"], // green
        ["#ef4444", "#fee2e2", "#991b1b"], // red
        ["#f59e0b", "#fef3c7", "#92400e"], // amber
        ["#a855f7", "#f3e8ff", "#6b21a8"], // purple
        ["#ec4899", "#fce7f3", "#9d174d"], // pink
        ["#6366f1", "#e0e7ff", "#3730a3"], // indigo
        ["#14b8a6", "#ccfbf1", "#115e59"], // teal
        ["#06b6d4", "#cffafe", "#155e75"], // cyan
        ["#84cc16", "#ecfccb", "#3f6212"], // lime
        ["#f97316", "#ffedd5", "#c2410c"], // orange
        ["#f43f5e", "#ffe4e6", "#9f1239"], // rose
        ["#8b5cf6", "#ede9fe", "#5b21b6"], // violet
        ["#0ea5e9", "#e0f2fe", "#075985"], // sky
        ["#10b981", "#d1fae5", "#065f46"], // emerald
    ];

    useHotKey("Escape", () => {
        if (selection.day !== undefined) {
            selection.selectSlot();
        } else if (selection.person !== undefined) {
            selection.selectPerson();
        }
    });

    let daySelectionMask = $state(Array.from({ length: N_DAYS }, () => 0));

    // Static structure — only recomputes when startDate changes
    const weeks = $derived.by(() => {
        const base = startOfISOWeek(parseISO(app.startDate));

        const out: { weekNum: number; days: { dateStr: string; idx: number }[]; month: string }[] =
            [];

        for (let w = 0; w < N_WEEKS; w++) {
            const mon = addWeeks(base, w);
            const days: { dateStr: string; idx: number }[] = [];

            for (let d = 0; d < N_WEEKDAYS; d++) {
                days.push({
                    dateStr: format(addDays(mon, d), "yyyy-MM-dd"),
                    idx: w * N_WEEKDAYS + d,
                });
            }

            out.push({ weekNum: w + 1, days, month: format(mon, "MMMM yyyy") });
        }

        return out;
    });

    $effect(() => {
        untrack(() => {
            for (let i = 0; i < N_DAYS; i++) {
                if (daySelectionMask[i] !== 0) {
                    daySelectionMask[i] = 0;
                }
            }
        });

        if (!selection.day) {
            return;
        }

        switch (app.activeMode) {
            case "select":
                daySelectionMask[selection.day] = 1;
                break;

            case "swap_day":
                daySelectionMask[selection.day] = 2;
                break;

            case "swap_role":
                daySelectionMask[selection.day] = selection.role === Role.Lead ? 3 : 4;
                break;
        }
    });

    function onclick(event: MouseEvent) {
        event.preventDefault();

        const target = event.target;

        if (!target) {
            return;
        }

        switch (app.activeMode) {
            case "select": {
                const day = findDay(target);

                if (day !== undefined) {
                    selection.selectSlot(day);
                }

                break;
            }

            case "set":
            case "erase": {
                const set = app.activeMode === "set";

                const day = findDay(target);
                const role = findRole(target);

                if (day === undefined || !role) {
                    return;
                }

                setPerson(day, role[0], set ? selection.person : undefined);

                break;
            }

            case "swap_day": {
                const day = findDay(target);

                if (day === undefined) {
                    return;
                }

                const otherDay = selection.day;

                if (otherDay === day) {
                    return;
                }

                if (otherDay === undefined) {
                    selection.selectSlot(day);
                } else {
                    swapDays(otherDay, day);
                    selection.selectSlot();
                }

                break;
            }

            case "swap_role": {
                const day = findDay(target);
                const role = findRole(target);

                const otherDay = selection.day;
                const otherRole = selection.role;

                if (day === undefined || !role) {
                    return;
                }

                if (otherDay === undefined || !otherRole) {
                    selection.selectSlot(day, role[0]);
                } else if (otherDay !== day || otherRole !== role[0]) {
                    swapRoles(day, otherDay, role[0], otherRole);
                    selection.selectSlot();
                }

                break;
            }
        }
    }

    function findRole(target: EventTarget): [Role, number?] | undefined {
        if (!(target instanceof HTMLElement)) {
            return;
        }

        let cursor: HTMLElement | null = target;

        while (cursor) {
            for (const role of [Role.Lead, Role.Support]) {
                if (role in cursor.dataset) {
                    const typedRole = role as Role;
                    const value = cursor.dataset[role];
                    return value ? [typedRole, parseInt(value!)] : [typedRole];
                }
            }

            cursor = cursor.parentElement;
        }
    }

    function findDay(target: EventTarget): number | undefined {
        if (!(target instanceof HTMLElement)) {
            return;
        }

        let cursor: HTMLElement | null = target;

        while (cursor) {
            if ("day" in cursor.dataset) {
                return parseInt(cursor.dataset.day!);
            }

            cursor = cursor.parentElement;
        }
    }
</script>

<!-- svelte-ignore a11y_click_events_have_key_events -->
<!-- svelte-ignore a11y_no_static_element_interactions -->
<div class="flex flex-col flex-1 min-w-0 overflow-hidden" {onclick}>
    <!-- ── Sticky weekday header ── -->
    <div class="shrink-0 flex border-b-2 border-border bg-card select-none">
        <div
            class="w-10 shrink-0 flex items-center justify-center text-[9px] font-bold tracking-wide text-muted-foreground/50"
        >
            W
        </div>

        {#each WEEKDAYS as day, d}
            <div
                class={[
                    "flex-1 min-w-0 text-center text-[10px] font-semibold py-1.5 border-l border-border/40",
                    d >= 5 && "bg-muted/30 text-muted-foreground/60",
                ]}
            >
                {day}
            </div>
        {/each}
    </div>

    <!-- ── Scrollable grid body ── -->
    <div class="flex-1 overflow-y-auto">
        {#each weeks as week, w (w)}
            <!-- Month divider (sticky) -->
            {#if w === 0 || week.month !== weeks[w - 1].month}
                <div
                    class="sticky top-0 z-5 bg-card border-b border-border px-3 py-1 text-[10px] font-semibold text-muted-foreground/80 uppercase tracking-wider select-none"
                >
                    {week.month}
                </div>
            {/if}

            <!-- Week row -->
            <div class="flex border-b border-border/40" style="min-height: 34px">
                <!-- Week number gutter -->
                <div
                    class="w-10 shrink-0 flex items-center justify-center text-[9px] font-mono select-none border-r border-border/40 {selection.selectedWeek ===
                    w
                        ? 'bg-secondary text-muted-foreground'
                        : 'bg-muted/15 text-muted-foreground/50'}"
                >
                    {String(week.weekNum).padStart(2, "0")}
                </div>

                <!-- Day cells -->
                {#each week.days as day, d (day.idx)}
                    {@const slot = app.slots[day.idx]}
                    {@const leadId = getLead(slot)}
                    {@const suppId = getSupport(slot)}
                    {@const isHoliday = day.dateStr in app.holidayMap}
                    {@const isWeekend = d >= 5}
                    {@const mask = daySelectionMask[day.idx]}

                    <div
                        data-day={day.idx}
                        class={[
                            "flex-1 min-w-0 flex flex-col border-l border-border/30",
                            isHoliday && "ring-1 ring-inset ring-amber-300/50 bg-amber-50/40",
                            !isHoliday && isWeekend && "bg-muted/25",
                            mask === 1 && "ring-blue-500 ring-4 ring-offset-2 z-10",
                            mask === 2 && "ring-orange-500 ring-4 ring-offset-2 z-10",
                        ]}
                    >
                        <!-- Lead half -->
                        {#if leadId !== undefined}
                            {@const c = PALETTE[leadId]}

                            <div
                                data-lead={leadId}
                                class={[
                                    "flex-1 min-h-0 flex items-center overflow-hidden",
                                    mask === 3 && "ring-pink-500 ring-4 ring-offset-2 z-10 ",
                                ]}
                                style="border-left:3px solid {c[0]};{view.gridMode ===
                                GridMode.Filled
                                    ? `background:${c[1]}`
                                    : ''}"
                            >
                                <span
                                    class="px-1 text-[10px] leading-none truncate"
                                    style="color:{view.gridMode === GridMode.Filled
                                        ? c[2]
                                        : 'var(--foreground)'}"
                                    >{app.formattedNames[leadId] ?? `#${leadId}`}</span
                                >
                            </div>
                        {:else}
                            <div
                                class="flex-1 min-h-0 border-l-[3px] border-l-transparent"
                                data-lead
                            ></div>
                        {/if}

                        <!-- Support half -->
                        {#if suppId !== undefined}
                            {@const c = PALETTE[suppId % 15]}

                            <div
                                data-support={suppId}
                                class={[
                                    "flex-1 min-h-0 flex items-center overflow-hidden border-t border-border/30",
                                    mask === 4 && "ring-purple-500 ring-4 ring-offset-2 z-10",
                                ]}
                                style="border-left:3px solid {c[0]};{view.gridMode ===
                                GridMode.Filled
                                    ? `background:${c[1]}`
                                    : ''}"
                            >
                                <span
                                    class="px-1 text-[10px] leading-none truncate"
                                    style="color:{view.gridMode === GridMode.Filled
                                        ? c[2]
                                        : 'var(--foreground)'}"
                                    >{app.formattedNames[suppId] ?? `#${suppId}`}</span
                                >
                            </div>
                        {:else}
                            <div
                                class="flex-1 min-h-0 border-l-[3px] border-l-transparent border-t border-border/30"
                                data-support
                            ></div>
                        {/if}
                    </div>
                {/each}
            </div>
        {/each}
    </div>
</div>

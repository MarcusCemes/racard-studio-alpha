<script lang="ts">
    import {
        addDays,
        addWeeks,
        format,
        formatISO,
        getDate,
        parseISO,
        startOfISOWeek,
    } from "date-fns";
    import { untrack } from "svelte";

    import { setPerson, swapDays, swapRoles } from "$lib/actions";
    import { GridMode, ZoomLevel, app, selection, view } from "$lib/app.svelte.js";
    import * as Tooltip from "$lib/components/ui/tooltip/index.js";
    import { N_DAYS, N_WEEKDAYS, N_WEEKS, Role, WEEKDAYS } from "$lib/defs.js";
    import { useGridNavigation } from "$lib/hooks/useGridNavigation";
    import { useHotKey } from "$lib/hooks/useHotkey.svelte.js";
    import { type ParsedConflict, conflictMessage } from "$lib/misc.js";
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
    useGridNavigation();

    let daySelectionMask = $state(Array.from({ length: N_DAYS }, () => 0));

    // Compute grid layout indices (on startDate change)
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

    function* iterWeeks() {
        const base = startOfISOWeek(parseISO(app.startDate));

        for (let weekIdx = 0; weekIdx < N_WEEKS; weekIdx++) {
            const monday = addWeeks(base, weekIdx);
            const sunday = addDays(monday, N_WEEKDAYS - 1);
            const selected = selection.selectedWeek === weekIdx;
            const weekLabel = (weekIdx + 1).toString().padStart(2, "0");

            const header =
                (weekIdx === 0 || getDate(sunday) <= N_WEEKDAYS) && format(sunday, "MMMM yyyy");

            yield {
                header,
                selected,
                weekIdx,
                weekLabel,
            };
        }
    }

    function* iterDays(weekIdx: number) {
        for (let i = 0; i < N_WEEKDAYS; i++) {
            const dayIdx = weekIdx * N_WEEKDAYS + i;
            const slot = app.slots[dayIdx];
            const lead = getLead(slot);
            const support = getSupport(slot);
            const mask = daySelectionMask[dayIdx];

            const date = addDays(app.startDate, dayIdx);
            const formattedDate = format(date, "dd/MM");
            const isoDate = formatISO(date, { representation: "date" });
            const conflicts = app.conflictMap.get(dayIdx);
            const bankHoliday = app.bankHolidays.find((h) => h.date === isoDate);

            yield { bankHoliday, conflicts, dayIdx, formattedDate, lead, mask, support };
        }
    }

    // Update daySelectionMask on selection change
    $effect(() => {
        untrack(() => {
            for (let i = 0; i < N_DAYS; i++) {
                if (daySelectionMask[i] !== 0) {
                    daySelectionMask[i] = 0;
                }
            }
        });

        if (selection.day === undefined) {
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
                    selection.selectSlot(day === selection.day ? undefined : day);
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

                if (otherDay === undefined) {
                    selection.selectSlot(day);
                } else {
                    if (otherDay !== day) {
                        swapDays(otherDay, day);
                    }

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
        {#each iterWeeks() as { header, selected, weekIdx, weekLabel } (weekIdx)}
            <!-- Month divider (sticky) -->
            {#if header}
                <div
                    class={[
                        "sticky top-0 z-5 bg-card border-b border-border px-3 py-1 text-[10px] font-semibold text-muted-foreground/80 uppercase tracking-wider",
                        weekIdx > 0 && "mt-4",
                    ]}
                >
                    {header}
                </div>
            {/if}

            <!-- Week row -->
            <div
                class={[
                    "flex border-b border-border/40",
                    view.zoom === ZoomLevel.Comfy ? "h-16" : "h-12",
                ]}
            >
                <!-- Week number gutter -->
                <div
                    class="w-10 shrink-0 flex items-center justify-center text-[9px] font-mono select-none border-r border-border/40 {selected
                        ? 'bg-secondary text-muted-foreground'
                        : 'bg-muted/15 text-muted-foreground/50'}"
                >
                    {weekLabel}
                </div>

                <!-- Day cells -->
                {#each iterDays(weekIdx) as { bankHoliday, conflicts, dayIdx, formattedDate, lead, mask, support } (dayIdx)}
                    <div
                        data-day={dayIdx}
                        class={[
                            "flex-1 min-w-0 flex flex-col border-l border-border/30 relative",
                            [1, 2].includes(mask) && "ring-4 ring-offset-2 z-10",
                            mask === 1 && "ring-blue-500",
                            mask === 2 && "ring-orange-500",
                            bankHoliday && "bg-yellow-50",
                        ]}
                    >
                        <!-- Day date -->
                        <div
                            class="flex mt-2 mb-1 px-2 text-[8px] text-neutral-500 font-medium gap-1"
                        >
                            <div class="flex-1">{formattedDate}</div>

                            {#if conflicts}
                                {#each conflicts as conflict}
                                    {@render conflictIndicator(conflict)}
                                {/each}
                            {/if}

                            {#if bankHoliday}
                                <Tooltip.Root>
                                    <Tooltip.Trigger class="ml-2 bg-yellow-600 size-2" />
                                    <Tooltip.Content>{bankHoliday.name}</Tooltip.Content>
                                </Tooltip.Root>
                            {/if}
                        </div>

                        <!-- Lead half -->
                        {#if lead !== undefined}
                            {@const [c0, c1, c2] = PALETTE[lead]}

                            <div
                                data-lead={lead}
                                class={[
                                    "flex-1 min-h-0 min-w-0 flex items-center",
                                    mask === 3 && "ring-pink-500 ring-4 ring-offset-2 z-10",
                                ]}
                                style="border-left:3px solid {c0};{view.gridMode === GridMode.Filled
                                    ? `background:${c1}`
                                    : ''}"
                            >
                                <span
                                    class="px-1 text-[10px] leading-none truncate"
                                    style="color:{view.gridMode === GridMode.Filled
                                        ? c2
                                        : 'var(--foreground)'}"
                                    >{app.formattedNames[lead] ?? `#${lead}`}</span
                                >
                            </div>
                        {:else}
                            <div
                                class="flex-1 min-h-0 min-w-0 border-l-[3px] border-l-transparent"
                                data-lead
                            ></div>
                        {/if}

                        <!-- Support half -->
                        {#if support !== undefined}
                            {@const [c0, c1, c2] = PALETTE[support % 15]}

                            <div
                                data-support={support}
                                class={[
                                    "flex-1 min-h-0 min-w-0 flex items-center border-t border-border/30",
                                    mask === 4 && "ring-purple-500 ring-4 ring-offset-2 z-10",
                                ]}
                                style="border-left:3px solid {c0};{view.gridMode === GridMode.Filled
                                    ? `background:${c1}`
                                    : ''}"
                            >
                                <span
                                    class="px-1 text-[10px] leading-none truncate"
                                    style="color:{view.gridMode === GridMode.Filled
                                        ? c2
                                        : 'var(--foreground)'}"
                                    >{app.formattedNames[support] ?? `#${support}`}</span
                                >
                            </div>
                        {:else}
                            <div
                                class="flex-1 min-h-0 min-w-0 border-l-[3px] border-l-transparent border-t border-border/30"
                                data-support
                            ></div>
                        {/if}
                    </div>
                {/each}
            </div>
        {/each}
    </div>
</div>

{#snippet conflictIndicator(conflict: ParsedConflict)}
    <Tooltip.Root>
        <Tooltip.Trigger class="bg-red-500 size-2 rounded-full" />

        <Tooltip.Content>
            {conflictMessage(conflict)}
        </Tooltip.Content>
    </Tooltip.Root>
{/snippet}

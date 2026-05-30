<script lang="ts">
    import { addDays, format, parseISO, startOfISOWeek } from "date-fns";

    import { setPerson, swapDays, swapRoles } from "$lib/actions.js";
    import { app } from "$lib/app.svelte.js";
    import { N_DAYS, N_WEEKDAYS, PERSON_COLORS, Role, WEEKDAYS } from "$lib/defs.js";
    import { useHotKeys } from "$lib/hooks/useHotkey.svelte.js";
    import { getLead, getSupport } from "$lib/slot.js";

    interface DayDescriptor {
        dayIndex: number;
        weekNumber: number;
        dateString: string;
        mondayLabel: string | null;
        isWeekend: boolean;
        isWeekStart: boolean;
        isNewMonth: boolean;
        monthLabel: string | null;
    }

    // ── Delegated click handler ──

    function handleGridClick(e: MouseEvent) {
        const target = e.target as HTMLElement;
        const roleEl = target.closest<HTMLElement>("[data-role]");
        const dayEl = target.closest<HTMLElement>("[data-day]");

        if (!dayEl) return;

        const dayIndex = Number(dayEl.dataset.day);
        const role = roleEl ? (Number(roleEl.dataset.role) as Role) : undefined;
        const mode = app.activeMode;

        switch (mode) {
            case "select":
                app.selection = { type: "day", dayIndex };
                break;

            case "set":
                if (role !== undefined && app.activeBrush !== undefined) {
                    setPerson(dayIndex, role, app.activeBrush);
                }
                break;

            case "erase":
                if (role !== undefined) {
                    setPerson(dayIndex, role);
                }
                break;

            case "swap_role": {
                if (role === undefined) return;
                const src = app.swapSource;
                if (src.type === "role") {
                    swapRoles(src.dayIndex, dayIndex, src.role, role);
                    app.swapSource = { type: "none" };
                } else {
                    app.swapSource = { type: "role", dayIndex, role };
                }
                break;
            }

            case "swap_day": {
                const src = app.swapSource;
                if (src.type === "day") {
                    swapDays(src.dayIndex, dayIndex);
                    app.swapSource = { type: "none" };
                } else {
                    app.swapSource = { type: "day", dayIndex };
                }
                break;
            }
        }
    }

    // ── Arrow key navigation (select mode only) ──

    function handleArrowKey(event: KeyboardEvent) {
        if (app.activeMode !== "select") return;
        if (app.selection.type !== "day") return;

        let { dayIndex } = app.selection;
        let newDay = dayIndex;

        if (event.key === "ArrowUp") newDay -= N_WEEKDAYS;
        else if (event.key === "ArrowDown") newDay += N_WEEKDAYS;
        else if (event.key === "ArrowLeft") newDay -= 1;
        else if (event.key === "ArrowRight") newDay += 1;
        else return;

        if (newDay >= 0 && newDay < N_DAYS) {
            app.selection = { type: "day", dayIndex: newDay };
            event.preventDefault();
        }
    }

    useHotKeys(["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"], handleArrowKey);

    // ── Clear selection on mode change ──

    $effect(() => {
        void app.activeMode;
        app.selection = { type: "none" };
        app.swapSource = { type: "none" };
    });

    // ── Schedule data generator ──

    function* scheduleDays(): Generator<DayDescriptor> {
        const baseDate = startOfISOWeek(parseISO(app.startDate));
        let lastMonth = -1;

        for (let dayIndex = 0; dayIndex < N_DAYS; dayIndex++) {
            const weekIdx = Math.floor(dayIndex / N_WEEKDAYS);
            const dayOfWeek = dayIndex % N_WEEKDAYS;
            const date = addDays(baseDate, dayIndex);

            let isNewMonth = false;
            let monthLabel: string | null = null;
            let mondayLabel: string | null = null;

            if (dayOfWeek === 0) {
                const m = date.getMonth();
                if (m !== lastMonth) {
                    isNewMonth = true;
                    monthLabel = date.toLocaleDateString("en-GB", {
                        month: "long",
                        year: "numeric",
                    });
                    lastMonth = m;
                }

                mondayLabel = date.toLocaleDateString("en-GB", {
                    day: "numeric",
                    month: "short",
                });
            }

            yield {
                dayIndex,
                weekNumber: weekIdx + 1,
                dateString: format(date, "yyyy-MM-dd"),
                mondayLabel,
                isWeekend: dayOfWeek >= 5,
                isWeekStart: dayOfWeek === 0,
                isNewMonth,
                monthLabel,
            };
        }
    }
</script>

{#snippet roleHalf(role: Role, personId: number | undefined, day: DayDescriptor)}
    {@const name = app.formattedNames[personId ?? -1]}
    {@const swatchClass =
        personId != null ? PERSON_COLORS[personId % PERSON_COLORS.length][1] : null}
    {@const isSelected = app.isSelected(day.dayIndex, role)}
    {@const isSwapSrc = app.isSwapSource(day.dayIndex, role)}

    <div
        class="role"
        class:lead={role === Role.Lead}
        class:support={role === Role.Support}
        class:selected={isSelected}
        class:swap-source={isSwapSrc}
        data-role={role}
    >
        <span class="swatch {swatchClass}" class:swatch-empty={!name}></span>
        {#if name}
            <span class="role-name hide-micro">{name}</span>
        {:else}
            <span class="role-empty hide-micro">&mdash;</span>
        {/if}
    </div>
{/snippet}

<div class="grid-container flex-1 min-w-0 flex flex-col bg-gray-50 zoom-{app.zoomLevel}">
    <!-- Sticky header row -->
    <div class="grid-header sticky top-0 z-10 bg-background border-b border-border">
        <div class="header-label"></div>
        {#each WEEKDAYS as day, i}
            <div class="header-day" class:header-day-weekend={i >= 5}>
                {day}
            </div>
        {/each}
    </div>

    <!-- Grid body -->
    <!-- svelte-ignore a11y_no_static_element_interactions, a11y_click_events_have_key_events -->
    <div
        class="grid-body flex-1 overflow-y-auto"
        data-mode={app.activeMode}
        onclick={handleGridClick}
    >
        {#each scheduleDays() as day (day.dayIndex)}
            {#if day.isNewMonth && day.monthLabel}
                <div class="month-divider">
                    <span>{day.monthLabel}</span>
                    <div></div>
                </div>
            {/if}

            {#if day.isWeekStart}
                <div class="week-label">
                    <span class="font-mono text-[10px] font-bold text-muted-foreground"
                        >W{day.weekNumber}</span
                    >
                    <span class="week-label-date hide-micro">{day.mondayLabel}</span>
                </div>
            {/if}

            {@const slot = app.slots[day.dayIndex]}
            {@const lead = getLead(slot)}
            {@const support = getSupport(slot)}
            {@const holiday = app.holidayMap[day.dateString]}
            {@const isDaySelected = app.isSelected(day.dayIndex)}
            {@const isDaySwapSrc = app.isSwapSource(day.dayIndex)}

            <div
                class="day-cell"
                class:weekend={day.isWeekend}
                class:holiday={!!holiday}
                class:selected={isDaySelected}
                class:swap-source={isDaySwapSrc}
                data-day={day.dayIndex}
            >
                {#if holiday}
                    <span class="holiday-badge hide-micro">{holiday}</span>
                {/if}

                {@render roleHalf(Role.Lead, lead, day)}
                {@render roleHalf(Role.Support, support, day)}
            </div>
        {/each}
    </div>
</div>

<style>
    /* ── Zoom-level CSS custom properties ── */
    .grid-container {
        --cell-min-h: 3.375rem;
        --cell-p-y: 0.1875rem;
        --cell-p-x: 0.375rem;
        --cell-radius: 5px;
        --swatch-size: 0.5625rem;
        --swatch-rounded: 2px;
    }

    .grid-container.zoom-micro {
        --cell-min-h: 2.25rem;
        --cell-p-y: 0;
        --cell-p-x: 0;
        --cell-radius: 4px;
        --swatch-size: 100%;
        --swatch-rounded: 0;
    }

    /* In micro zoom, the swatch fills the entire role area via flex stretch.
       height: 100% would resolve to 0 (parent computed height is auto),
       so we clear it and let align-self: stretch handle the cross-axis. */
    .grid-container.zoom-micro .swatch {
        height: auto;
        align-self: stretch;
    }

    .grid-container.zoom-detail {
        --cell-min-h: 4.5rem;
        --cell-p-y: 0.25rem;
        --cell-p-x: 0.375rem;
        --cell-radius: 6px;
        --swatch-size: 0.625rem;
        --swatch-rounded: 2px;
    }

    .grid-container.zoom-micro .hide-micro {
        display: none !important;
    }

    /* ── Header row ── */
    .grid-header {
        display: grid;
        grid-template-columns: 4.25rem repeat(7, 1fr);
        flex-shrink: 0;
    }

    .header-label {
        /* spacer — width controlled by grid-template-columns */
        display: block;
    }

    .header-day {
        text-align: center;
        font-size: 0.656rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.05em;
        padding-block: 0.25rem;
        color: var(--color-muted-foreground);
    }

    .header-day-weekend {
        opacity: 0.6;
    }

    /* ── Grid body ── */
    .grid-body {
        display: grid;
        grid-template-columns: 4.25rem repeat(7, 1fr);
        grid-auto-rows: auto;
        align-items: start;
        column-gap: 0.125rem;
        row-gap: 0.25rem;
        padding: 0 0.125rem;
    }

    /* ── Month divider (spans full row) ── */
    .month-divider {
        grid-column: 1 / -1;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding-block: 0.5rem 0.125rem;
    }

    .month-divider span {
        font-size: 0.688rem;
        font-weight: 600;
        color: var(--color-muted-foreground);
        letter-spacing: 0.04em;
    }

    .month-divider div {
        flex: 1;
        height: 1px;
        background: var(--color-border);
    }

    /* ── Week label ── */
    .week-label {
        display: flex;
        flex-direction: column;
        justify-content: center;
        padding-right: 0.375rem;
        gap: 1px;
        min-height: var(--cell-min-h);
    }

    .week-label-date {
        font-size: 0.594rem;
        color: var(--color-muted-foreground);
        opacity: 0.7;
    }

    /* ── Day cell ── */
    .day-cell {
        min-height: var(--cell-min-h);
        border-radius: var(--cell-radius);
        background: var(--color-card);
        border: 1.5px solid transparent;
        display: flex;
        flex-direction: column;
        position: relative;
    }

    .day-cell.weekend {
        background: oklch(0.963 0.002 197.1 / 0.2);
    }

    .day-cell.holiday {
        background: oklch(0.85 0.15 85 / 0.05);
        border-color: oklch(0.85 0.15 85 / 0.2);
    }

    /* ── Holiday badge ── */
    .holiday-badge {
        position: absolute;
        top: 0.125rem;
        right: 0.1875rem;
        font-size: 0.469rem;
        color: oklch(0.65 0.15 80);
        background: oklch(0.85 0.15 85 / 0.1);
        padding: 0 0.25rem;
        padding-block: 0.05rem;
        border-radius: 0.15rem;
        font-weight: 600;
        text-transform: uppercase;
        letter-spacing: 0.05em;
    }

    /* ── Role halves ── */
    .role {
        flex: 1;
        display: flex;
        align-items: center;
        gap: 0.25rem;
        padding-inline: var(--cell-p-x);
        padding-block: var(--cell-p-y);
        justify-content: center;
    }

    @media (min-width: 640px) {
        .role {
            justify-content: flex-start;
        }
    }

    .role.lead {
        border-bottom: 1px solid var(--color-border);
    }

    .grid-container.zoom-detail .role {
        flex-wrap: wrap;
    }

    /* ── Role content ── */
    .role-name {
        font-size: 0.656rem;
        font-weight: 600;
        line-height: 1;
        overflow: hidden;
        text-overflow: ellipsis;
        white-space: nowrap;
    }

    .role-empty {
        font-size: 0.656rem;
        color: var(--color-muted-foreground);
        opacity: 0.4;
        font-style: italic;
        line-height: 1;
    }

    /* ── Swatch ── */
    .swatch {
        flex-shrink: 0;
        width: var(--swatch-size);
        height: var(--swatch-size);
        border-radius: var(--swatch-rounded);
    }

    .swatch-empty {
        visibility: hidden;
    }

    /* ── Selection visual states ── */

    .day-cell.selected {
        border-color: oklch(0.55 0.22 260);
        box-shadow: 0 0 0 1px oklch(0.55 0.22 260 / 0.45);
    }

    .day-cell.swap-source {
        border-color: oklch(0.65 0.18 50);
        border-style: dashed;
        animation: swap-pulse 1.4s ease-in-out infinite;
    }

    .role.selected {
        background: oklch(0.55 0.22 260 / 0.12);
        box-shadow: inset 0 0 0 1px oklch(0.55 0.22 260 / 0.25);
    }

    .role.swap-source {
        background: oklch(0.65 0.18 50 / 0.12);
        box-shadow: inset 0 0 0 1px oklch(0.65 0.18 50 / 0.25);
    }

    @keyframes swap-pulse {
        0%,
        100% {
            opacity: 1;
        }
        50% {
            opacity: 0.7;
        }
    }

    /* ── Mode-driven cursors ── */

    .grid-body[data-mode="select"] .day-cell,
    .grid-body[data-mode="swap_day"] .day-cell {
        cursor: pointer;
    }

    .grid-body[data-mode="set"] .role,
    .grid-body[data-mode="erase"] .role,
    .grid-body[data-mode="swap_role"] .role {
        cursor: pointer;
    }
</style>

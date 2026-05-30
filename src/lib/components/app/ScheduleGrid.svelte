<script lang="ts">
    import { addDays, format, parseISO, startOfISOWeek } from "date-fns";

    import { app } from "$lib/app.svelte.js";
    import { N_DAYS, N_WEEKDAYS, PERSON_COLORS, WEEKDAYS } from "$lib/defs.js";
    import { getLead, getSupport } from "$lib/slot.js";

    interface DayDescriptor {
        dayIndex: number;
        weekNumber: number;
        dateString: string;
        dayOfMonth: number;
        mondayLabel: string | null;
        isWeekend: boolean;
        isWeekStart: boolean;
        isNewMonth: boolean;
        monthLabel: string | null;
    }

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
                dayOfMonth: date.getDate(),
                mondayLabel,
                isWeekend: dayOfWeek >= 5,
                isWeekStart: dayOfWeek === 0,
                isNewMonth,
                monthLabel,
            };
        }
    }
</script>

<div
    class="grid-container flex-1 min-w-0 flex flex-col bg-gray-50 zoom-{app.zoomLevel}"
>
    <!-- Sticky header row -->
    <div class="grid-header sticky top-0 z-10 bg-background border-b border-border">
        <div class="header-label"></div>
        {#each WEEKDAYS as day, i}
            <div
                class="header-day"
                class:header-day-weekend={i >= 5}
            >
                {day}
            </div>
        {/each}
    </div>

    <!-- Scrollable grid body -->
    <div class="grid-body flex-1 overflow-y-auto">
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
            {@const leadName = app.formattedNames[lead ?? -1]}
            {@const leadSwatch = lead != null ? PERSON_COLORS[lead % PERSON_COLORS.length][1] : null}
            {@const supportName = app.formattedNames[support ?? -1]}
            {@const supportSwatch = support != null ? PERSON_COLORS[support % PERSON_COLORS.length][1] : null}
            {@const holiday = app.holidayMap[day.dateString]}

            <div
                class="day-cell"
                class:weekend={day.isWeekend}
                class:holiday={!!holiday}
            >
                <span class="day-date hide-micro">{day.dayOfMonth}</span>

                {#if holiday}
                    <span class="holiday-badge hide-micro">{holiday}</span>
                {/if}

                <!-- Lead role -->
                <div class="role lead">
                    {#if leadName}
                        <span class="swatch {leadSwatch}"></span>
                        <span class="role-name hide-micro">{leadName}</span>
                    {:else}
                        <span class="role-empty hide-micro">&mdash;</span>
                    {/if}
                </div>

                <!-- Support role -->
                <div class="role support">
                    {#if supportName}
                        <span class="swatch {supportSwatch}"></span>
                        <span class="role-name hide-micro">{supportName}</span>
                    {:else}
                        <span class="role-empty hide-micro">&mdash;</span>
                    {/if}
                </div>
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
        gap: 0.125rem;
        padding: 0 0.125rem;
    }

    /* ── Month divider (spans full row) ── */
    .month-divider {
        grid-column: 1 / -1;
        display: flex;
        align-items: center;
        gap: 0.5rem;
        padding-block: 0.625rem 0.25rem;
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

    /* ── Date badge ── */
    .day-date {
        position: absolute;
        top: 0.125rem;
        left: 0.1875rem;
        font-family: var(--font-mono, monospace);
        font-size: 0.531rem;
        color: var(--color-muted-foreground);
        opacity: 0.6;
        line-height: 1;
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
        font-size: 0.625rem;
        color: var(--color-muted-foreground);
        opacity: 0.4;
        font-style: italic;
    }

    /* ── Swatch ── */
    .swatch {
        flex-shrink: 0;
        width: var(--swatch-size);
        height: var(--swatch-size);
        border-radius: var(--swatch-rounded);
    }
</style>

<script lang="ts">
    import { SquareDashedMousePointer } from "@lucide/svelte";
    import { addDays, addWeeks, parseISO, startOfISOWeek } from "date-fns";

    import { app, selection } from "$lib/app.svelte.js";
    import * as Empty from "$lib/components/ui/empty/index.js";
    import { N_WEEKDAYS, PERSON_COLORS } from "$lib/defs.js";
    import { getLead, getSupport } from "$lib/slot.js";

    import InspectEmployee from "./InspectEmployee.svelte";
    import InspectorDay from "./InspectorDay.svelte";

    let dayIndex = $derived(selection.selectedDayOfWeek);
    let weekIndex = $derived(selection.selectedWeek);
    let selectedPerson = $derived(app.people[selection.person ?? -1]);

    // Derive day data from selection
    const dayData = $derived.by(() => {
        if (dayIndex == undefined || weekIndex == undefined) return null;

        const baseDate = startOfISOWeek(parseISO(app.startDate));
        const monday = addWeeks(baseDate, weekIndex);
        const date = addDays(monday, dayIndex);
        const daySlot = weekIndex * N_WEEKDAYS + dayIndex;

        const leadSlot = getLead(app.slots[daySlot]);
        const suppSlot = getSupport(app.slots[daySlot]);

        return {
            date,
            weekNumber: weekIndex + 1,
            lead: leadSlot != null ? leadSlot : null,
            support: suppSlot != null ? suppSlot : null,
        };
    });

    const panelState = $derived(
        dayIndex != undefined && weekIndex != undefined
            ? "day"
            : selectedPerson
              ? "employee"
              : "idle",
    );
</script>

<aside class="w-72 shrink-0 flex flex-col border-l border-border bg-card overflow-hidden">
    <!-- Header -->
    <div class="px-3.5 py-3 border-b border-border shrink-0">
        {#if panelState === "idle"}
            <span class="text-[13px] font-semibold">Inspector</span>
        {:else if panelState === "day" && dayData}
            <div class="flex flex-col gap-0.5">
                <span class="text-[13px] font-semibold">
                    {dayData.date.toLocaleDateString("en-GB", { weekday: "long" })}
                </span>

                <span class="text-[11.5px] text-muted-foreground">
                    {dayData.date.toLocaleDateString("en-GB", {
                        day: "numeric",
                        month: "long",
                        year: "numeric",
                    })} &ndash; Week {weekIndex! + 1}
                </span>
            </div>
        {:else if panelState === "employee" && selection.person !== undefined}
            {@const { name, rate } = app.people[selection.person]}
            {@const swatch = PERSON_COLORS[selection.person]}

            <div class="flex items-center gap-2.5">
                <span class="w-1.5 h-9 rounded-[3px] shrink-0" style="background:{swatch}"></span>
                <div>
                    <span class="text-[13px] font-semibold block">{name}</span>
                    <span class="text-[11px] text-muted-foreground">{rate}%</span>
                </div>
            </div>
        {/if}
    </div>

    <div class="flex-1 overflow-y-auto flex flex-col py-2.5">
        <!-- ── IDLE STATE ── -->
        {#if panelState === "idle"}
            <Empty.Root class="flex-1 opacity-60">
                <Empty.Header>
                    <Empty.Media variant="icon">
                        <SquareDashedMousePointer />
                    </Empty.Media>

                    <Empty.Title>Inspector</Empty.Title>
                    <Empty.Description>Click on something to view its details.</Empty.Description>
                </Empty.Header>
            </Empty.Root>

            <!-- ── DAY STATE ── -->
        {:else if panelState === "day" && selection.day}
            <InspectorDay day={selection.day} />

            <!-- ── EMPLOYEE STATE ── -->
        {:else if panelState === "employee" && selectedPerson}
            {#if selection.person !== undefined}
                <InspectEmployee person={selection.person} />
            {/if}
        {/if}
    </div>
</aside>

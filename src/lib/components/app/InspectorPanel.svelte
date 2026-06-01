<script lang="ts">
    import { addDays, addWeeks, parseISO, startOfISOWeek } from "date-fns";

    import { app, selection } from "$lib/app.svelte.js";
    import * as Badge from "$lib/components/ui/badge/index.js";
    import * as Tabs from "$lib/components/ui/tabs/index.js";
    import { N_WEEKDAYS, PERSON_COLORS } from "$lib/defs.js";
    import { getLead, getSupport } from "$lib/slot.js";

    import InspectEmployee from "./InspectEmployee.svelte";
    import InspectorConflicts from "./InspectorConflicts.svelte";
    import InspectorDay from "./InspectorDay.svelte";

    let activeTab = $state("conflicts");

    let dayIndex = $derived(selection.selectedDayOfWeek);
    let weekIndex = $derived(selection.selectedWeek);
    let selectedPerson = $derived(app.people[selection.person ?? -1]);

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

    const conflictCount = $derived(app.conflicts.length);

    // Auto-switch tabs based on selection
    $effect(() => {
        if (selection.day !== undefined) {
            activeTab = "day";
        } else if (selection.person !== undefined) {
            activeTab = "person";
        } else {
            activeTab = "conflicts";
        }
    });
</script>

<aside class="w-72 shrink-0 flex flex-col border-l border-border bg-card overflow-hidden">
    <!-- Header -->
    <div class="px-3.5 py-3 border-b border-border shrink-0">
        {#if activeTab === "day" && dayData}
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
        {:else if activeTab === "person" && selection.person !== undefined}
            {@const { name, rate } = app.people[selection.person]}
            {@const swatch = PERSON_COLORS[selection.person]}

            <div class="flex items-center gap-2.5">
                <span class="w-1.5 h-9 rounded-[3px] shrink-0" style="background:{swatch}"></span>
                <div>
                    <span class="text-[13px] font-semibold block">{name}</span>
                    <span class="text-[11px] text-muted-foreground">{rate}%</span>
                </div>
            </div>
        {:else}
            <span class="text-[13px] font-semibold">Inspector</span>
        {/if}
    </div>

    <!-- Tabs -->
    <Tabs.Root bind:value={activeTab} class="flex-1 flex flex-col overflow-hidden">
        <Tabs.List class="mx-3.5 mt-2 mb-1 shrink-0">
            <Tabs.Trigger
                value="day"
                disabled={dayIndex == undefined}
                title={dayIndex == undefined ? "Click a day in the grid" : ""}
            >
                Day
            </Tabs.Trigger>
            <Tabs.Trigger
                value="person"
                disabled={selection.person === undefined}
                title={selection.person === undefined ? "Click an employee in the roster" : ""}
            >
                Person
            </Tabs.Trigger>
            <Tabs.Trigger value="conflicts" class="gap-1">
                Conflicts
                {#if conflictCount > 0}
                    <Badge.Badge variant="destructive" class="text-[9px] px-1 h-3.5 font-mono">
                        {conflictCount}
                    </Badge.Badge>
                {/if}
            </Tabs.Trigger>
        </Tabs.List>

        <Tabs.Content value="day" class="flex-1 overflow-y-auto py-2.5">
            {#if selection.day !== undefined}
                <InspectorDay day={selection.day} />
            {/if}
        </Tabs.Content>

        <Tabs.Content value="person" class="flex-1 overflow-y-auto py-2.5">
            {#if selection.person !== undefined}
                <InspectEmployee person={selection.person} />
            {/if}
        </Tabs.Content>

        <Tabs.Content value="conflicts" class="flex-1 overflow-y-auto py-2.5">
            <InspectorConflicts />
        </Tabs.Content>
    </Tabs.Root>
</aside>

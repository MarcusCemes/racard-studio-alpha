<script lang="ts">
    import { SquareDashedMousePointer } from "@lucide/svelte";
    import { addDays, addWeeks, parseISO, startOfISOWeek } from "date-fns";

    import { app } from "$lib/app.svelte.js";
    import * as Empty from "$lib/components/ui/empty/index.js";
    import { Separator } from "$lib/components/ui/separator/index.js";
    import { N_WEEKDAYS, PERSON_COLORS } from "$lib/defs.js";
    import { getLead, getSupport } from "$lib/slot.js";

    import InspectEmployee from "./InspectEmployee.svelte";

    let dayIndex = $derived(app.selectedDayOfWeek);
    let weekIndex = $derived(app.selectedWeek);
    let selectedPerson = $derived(app.people[app.activeBrush ?? -1]);

    // Derive day data from selection
    const dayData = $derived.by(() => {
        if (dayIndex == null || weekIndex == null) return null;

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
        dayIndex != null && weekIndex != null ? "day" : selectedPerson ? "employee" : "idle",
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
                    })}
                </span>
            </div>
        {:else if panelState === "employee" && app.activeBrush !== undefined}
            {@const { name, rate } = app.people[app.activeBrush]}
            {@const swatch = PERSON_COLORS[app.activeBrush % PERSON_COLORS.length][1]}

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
        {:else if panelState === "day" && dayData}
            <div class="mx-3.5 mb-3 rounded-lg border border-border overflow-hidden bg-background">
                <div class="flex items-center gap-1.5 px-3 py-2.5 min-h-10.5">
                    <span
                        class="text-[10px] font-bold uppercase tracking-[0.07em] text-muted-foreground w-11 shrink-0"
                        >Lead</span
                    >
                    {#if dayData.lead !== null}
                        {@const person = app.people[dayData.lead]}
                        {@const swatch = PERSON_COLORS[dayData.lead % PERSON_COLORS.length][1]}
                        <span class="w-2.5 h-2.5 rounded-[3px] shrink-0" style="background:{swatch}"
                        ></span>
                        <span class="flex-1 text-[12.5px] font-medium truncate">{person.name}</span>
                    {:else}
                        <span class="text-xs text-muted-foreground italic">Unassigned</span>
                    {/if}
                </div>
                <Separator />
                <div class="flex items-center gap-1.5 px-3 py-2.5 min-h-10.5">
                    <span
                        class="text-[10px] font-bold uppercase tracking-[0.07em] text-muted-foreground w-11 shrink-0"
                        >Support</span
                    >
                    {#if dayData.support !== null}
                        {@const person = app.people[dayData.support]}
                        {@const swatch = PERSON_COLORS[dayData.support % PERSON_COLORS.length][1]}

                        <span class="w-2.5 h-2.5 rounded-[3px] shrink-0" style="background:{swatch}"
                        ></span>
                        <span class="flex-1 text-[12.5px] font-medium truncate">{person.name}</span>
                    {:else}
                        <span class="text-xs text-muted-foreground italic">Unassigned</span>
                    {/if}
                </div>
            </div>

            <div class="mx-3.5 mb-2.5 text-[11.5px] text-muted-foreground italic">
                No violations
            </div>

            <div class="mx-3.5 pt-2.5 border-t border-border">
                <span
                    class="text-[10px] font-semibold uppercase tracking-[0.07em] text-muted-foreground"
                    >Week {dayData.weekNumber}</span
                >
            </div>

            <!-- ── EMPLOYEE STATE ── -->
        {:else if panelState === "employee" && selectedPerson}
            {#if app.activeBrush !== undefined}
                <InspectEmployee person={app.activeBrush} />
            {/if}
        {/if}
    </div>
</aside>

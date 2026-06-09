<script lang="ts">
    import { Check } from "@lucide/svelte";

    import { app, selection } from "$lib/app.svelte.js";
    import * as Badge from "$lib/components/ui/badge/index.js";
    import * as Empty from "$lib/components/ui/empty/index.js";
    import { N_WEEKDAYS, PERSON_TEXT_COLORS, WEEKDAYS } from "$lib/defs.js";
    import { ConflictKind, type ParsedConflict, iterParsedConflict } from "$lib/misc.js";
    import type { Conflict } from "$lib/schemas.js";

    interface ConflictGroup {
        kind: ConflictKind;
        label: string;
        conflicts: ParsedConflict[];
    }

    const CONFLICT_TYPES: { kind: ConflictKind; label: string }[] = [
        { kind: ConflictKind.ConsecutiveDay, label: "Consecutive Day" },
        { kind: ConflictKind.Holiday, label: "Holiday" },
        { kind: ConflictKind.Role, label: "Role" },
        { kind: ConflictKind.WorkCount, label: "Work Count" },
    ];

    const groups = $derived.by<ConflictGroup[]>(() => {
        const parsed = app.conflicts.flatMap((c: Conflict) => [...iterParsedConflict(c)]);

        return CONFLICT_TYPES.map(({ kind, label }) => ({
            kind,
            label,
            conflicts: parsed.filter((p) => p.kind === kind),
        }));
    });

    function describeConflict(parsed: ParsedConflict): string {
        switch (parsed.kind) {
            case ConflictKind.ConsecutiveDay:
                return `${formatDayIdx(parsed.dayIdx)} → ${formatDayIdx(parsed.otherDayIdx!)}`;
            case ConflictKind.Holiday:
            case ConflictKind.Role:
                return formatDayIdx(parsed.dayIdx);
            case ConflictKind.WorkCount:
                return `Week ${Math.floor(parsed.dayIdx / N_WEEKDAYS) + 1}`;
        }
    }

    function onclick(parsed: ParsedConflict) {
        if (parsed.kind === ConflictKind.WorkCount) {
            selection.selectPerson(parsed.personIdx);
        } else {
            selection.selectSlot(parsed.dayIdx);
        }
    }

    export function formatDayIdx(dayIdx: number): string {
        const week = Math.floor(dayIdx / N_WEEKDAYS) + 1;
        const day = WEEKDAYS[dayIdx % N_WEEKDAYS];
        return `${day} Wk ${week}`;
    }
</script>

<div class="flex flex-col gap-3 px-3.5">
    {#if app.conflicts.length === 0}
        <Empty.Root class="flex-1 opacity-60 py-8">
            <Empty.Header>
                <Empty.Media variant="icon">
                    <Check class="size-5 text-green-500" />
                </Empty.Media>
                <Empty.Title>All Clear</Empty.Title>
                <Empty.Description>No constraint violations found.</Empty.Description>
            </Empty.Header>
        </Empty.Root>
    {:else}
        {#each groups as group}
            <div class="flex flex-col gap-1">
                <div class="flex items-center gap-2 px-1">
                    <span
                        class="text-[10px] font-semibold uppercase tracking-wider text-muted-foreground"
                    >
                        {group.label}
                    </span>
                    <Badge.Badge variant="secondary" class="text-[9px] px-1.5 h-4 font-mono">
                        {group.conflicts.length}
                    </Badge.Badge>
                </div>

                {#if group.conflicts.length === 0}
                    <div
                        class="flex items-center gap-1.5 px-2 py-1.5 text-[11px] text-muted-foreground"
                    >
                        <Check class="size-3 text-green-500" />
                        None
                    </div>
                {:else}
                    <div class="flex flex-col">
                        {#each group.conflicts as parsed}
                            {@const color =
                                PERSON_TEXT_COLORS[parsed.personIdx % PERSON_TEXT_COLORS.length]}

                            <button
                                class="flex items-center gap-2 px-2 py-1.5 rounded-md text-left hover:bg-accent/50 transition-colors cursor-pointer border-0 bg-transparent"
                                onclick={() => onclick(parsed)}
                            >
                                <span class="text-[12px] font-medium flex-1 truncate {color}">
                                    {app.formattedNames[parsed.personIdx]}
                                </span>
                                <span class="text-[11px] text-muted-foreground font-mono shrink-0">
                                    {describeConflict(parsed)}
                                </span>
                            </button>
                        {/each}
                    </div>
                {/if}
            </div>
        {/each}
    {/if}
</div>

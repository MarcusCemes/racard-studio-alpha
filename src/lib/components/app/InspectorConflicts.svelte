<script lang="ts">
    import { Check } from "@lucide/svelte";

    import { app, selection } from "$lib/app.svelte.js";
    import * as Badge from "$lib/components/ui/badge/index.js";
    import * as Empty from "$lib/components/ui/empty/index.js";
    import { PERSON_TEXT_COLORS } from "$lib/defs.js";
    import { type ParsedConflict, formatDayIdx, parseConflict } from "$lib/misc.js";
    import type { Conflict } from "$lib/schemas.js";
    import { cn } from "$lib/utils.js";

    interface ConflictGroup {
        type: ParsedConflict["type"];
        label: string;
        conflicts: ParsedConflict[];
    }

    const CONFLICT_TYPES: { type: ParsedConflict["type"]; label: string }[] = [
        { type: "ConsecutiveDay", label: "Consecutive Day" },
        { type: "Holiday", label: "Holiday" },
        { type: "Role", label: "Role" },
        { type: "WorkCount", label: "Work Count" },
    ];

    const groups = $derived.by<ConflictGroup[]>(() => {
        const parsed = app.conflicts.map((c: Conflict) => parseConflict(c, formatDayIdx));

        return CONFLICT_TYPES.map(({ type, label }) => ({
            type,
            label,
            conflicts: parsed.filter((p) => p.type === type),
        }));
    });

    function onclick(parsed: ParsedConflict) {
        if (parsed.scope === "day") {
            selection.selectSlot(parsed.scopeIndex);
        } else {
            selection.selectPerson(parsed.personIdx);
        }
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
                                class={cn(
                                    "flex items-center gap-2 px-2 py-1.5 rounded-md text-left hover:bg-accent/50 transition-colors cursor-pointer border-0 bg-transparent",
                                )}
                                onclick={() => onclick(parsed)}
                            >
                                <span class={cn("text-[12px] font-medium flex-1 truncate", color)}>
                                    {app.formattedNames[parsed.personIdx]}
                                </span>
                                <span class="text-[11px] text-muted-foreground font-mono shrink-0">
                                    {parsed.description}
                                </span>
                            </button>
                        {/each}
                    </div>
                {/if}
            </div>
        {/each}
    {/if}
</div>

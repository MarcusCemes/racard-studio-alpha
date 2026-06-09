<script lang="ts">
    import { Check, TriangleAlert } from "@lucide/svelte";

    import { app } from "$lib/app.svelte.js";
    import StatusMonitor from "$lib/components/misc/StatusMonitor.svelte";
    import { Badge } from "$lib/components/ui/badge/index.js";
    import { plural } from "$lib/misc";

    let fitnessTotal = $derived(
        app.statistics?.fitness
            ? Object.values(app.statistics.fitness).reduce((a, b) => a + b, 0)
            : 0,
    );

    let subScores = $derived(
        app.statistics?.fitness
            ? [
                  { label: "Hours", score: app.statistics.fitness.annual_hours },
                  { label: "Consec", score: app.statistics.fitness.consecutive_days },
                  { label: "WkEnd", score: app.statistics.fitness.consecutive_weekends },
              ]
            : [],
    );

    let numberConflicts = $derived(app.conflicts.length);
</script>

<footer
    class="flex items-center justify-between h-8 px-3.5 shrink-0 border-t border-border bg-card text-[11px]"
>
    <div class="flex items-center gap-2">
        <div class="flex items-baseline gap-1">
            <span class="font-mono text-[13px] font-bold">{fitnessTotal.toFixed(1)}</span>
            <span class="text-[10px] uppercase tracking-widest text-muted-foreground">fitness</span>
            {#each subScores as sub}
                <span class="text-[10px] text-muted-foreground ml-1" title="{sub.label} fitness">
                    {sub.label}:
                    <b class="font-mono font-semibold text-foreground">{sub.score.toFixed(1)}</b>
                </span>
            {/each}
        </div>
    </div>

    <div class="flex items-center gap-4">
        {#if numberConflicts == 0}
            <Badge class="flex items-center gap-2" variant="destructive">
                <TriangleAlert class="size-3" />
                <span>{numberConflicts} {plural(numberConflicts, "conflict")}</span>
            </Badge>
        {:else}
            <span class="flex items-center gap-1 text-green-600"
                ><Check class="size-3" /> Valid schedule</span
            >
        {/if}

        <StatusMonitor />
    </div>
</footer>

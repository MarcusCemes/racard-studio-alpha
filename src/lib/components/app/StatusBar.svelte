<script lang="ts">
    import { AlertTriangle } from "@lucide/svelte";

    import { type ActiveMode, app } from "$lib/app.svelte";
    import { Button } from "$lib/components/ui/button/index.js";
    import { Separator } from "$lib/components/ui/separator/index.js";

    import StatusMonitor from "../misc/StatusMonitor.svelte";

    const modeLabels: Record<ActiveMode, string> = {
        select: "Select",
        set: "Set",
        swap_day: "Swap Day",
        swap_role: "Swap Role",
        erase: "Erase",
    };

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

    let violationCount = $derived(app.conflicts.length);
</script>

<StatusMonitor />

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

    <div class="flex items-center gap-2">
        {#if violationCount > 0}
            <Button
                variant="ghost"
                size="xs"
                class="text-red-500 hover:bg-red-500/10 text-[11px] font-semibold py-0.5 px-1.5 h-auto rounded"
                title="View all violations"
            >
                <AlertTriangle size={12} />
                <span>{violationCount} violation{violationCount !== 1 ? "s" : ""}</span>
            </Button>
        {:else}
            <span class="text-green-500 font-semibold text-[11px]">✓ No violations</span>
        {/if}
        <Separator orientation="vertical" class="h-3.5" />
        <span class="text-[10.5px] text-muted-foreground font-mono">Undo: 0 steps</span>
    </div>
</footer>

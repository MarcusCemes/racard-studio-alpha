<script lang="ts">
    import {
        Activity,
        Check,
        Flame,
        Square,
        Target,
        TrendingDown,
        Trophy,
        X,
    } from "@lucide/svelte";
    import { LineChart } from "layerchart";

    import { app } from "$lib/app.svelte";
    import { Button } from "$lib/components/ui/button/index.js";
    import * as Dialog from "$lib/components/ui/dialog/index.js";

    let open = $state(false);

    let maxIterations = $state(app.refinerRounds);
    let temperature = $derived(app.refinerProgress?.temperature);
    let currentFitness = $derived(app.refinerProgress?.current_fitness);
    let bestFitness = $derived(app.refinerProgress?.best_fitness);
    let acceptedCount = $state(app.refinerProgress?.accepted ?? 0);
    let rejectedCount = $state(app.refinerProgress?.rejected ?? 0);
    let history = $state([
        { iteration: 5, fitness: 10000 },
        { iteration: 10, fitness: 9000 },
        { iteration: 20, fitness: 8500 },
        { iteration: 30, fitness: 8000 },
        { iteration: 40, fitness: 7500 },
        { iteration: 50, fitness: 7000 },
        { iteration: 60, fitness: 6500 },
        { iteration: 70, fitness: 6000 },
        { iteration: 80, fitness: 5500 },
        { iteration: 90, fitness: 5000 },
        { iteration: 100, fitness: 4500 },
    ]);

    let totalCount = $derived(acceptedCount + rejectedCount);
    let acceptanceRate = $derived(acceptedCount / (totalCount || 1)); // avoid division by zero
    let progress = $derived(Math.min(1, totalCount / maxIterations));
    let progressPercent = $derived(Math.round(progress * 100));

    $effect(() => {
        open = app.refinerActive;
    });

    function fmt(num: number) {
        return new Intl.NumberFormat().format(num);
    }

    function fmtDec(num: number) {
        return new Intl.NumberFormat(undefined, { maximumFractionDigits: 3 }).format(num);
    }

    function onstop() {
        app.refinerActive = false;
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content style="max-width: 48rem;">
        <Dialog.Header>
            <Dialog.Title class="flex items-center gap-2">
                <Activity class="h-5 w-5 text-primary animate-pulse" />
                Refiner
            </Dialog.Title>
            <Dialog.Description>
                Generating permutations to improve schedule quality.
            </Dialog.Description>
        </Dialog.Header>

        <div class="space-y-4 rounded-lg border p-4 bg-card text-card-foreground">
            <!-- Title & Overall Refiner Progress -->
            <div class="space-y-2">
                <div class="flex items-center justify-between">
                    <div class="space-y-0.5">
                        <h3 class="text-sm font-semibold tracking-tight">Iterative Refinement</h3>
                        <p class="text-xs text-muted-foreground">
                            Cooling schedule and score optimization
                        </p>
                    </div>
                </div>

                <!-- Main Progress Bar -->
                <div class="space-y-1">
                    <div class="h-2 w-full bg-secondary rounded-full overflow-hidden">
                        <div
                            class="bg-primary h-full transition-all duration-300"
                            style="width: {progressPercent}%"
                        ></div>
                    </div>
                    <div class="flex justify-between text-[10px] text-muted-foreground font-mono">
                        <span>0%</span>
                        <span>{progressPercent}% Complete</span>
                        <span>100%</span>
                    </div>
                </div>
            </div>

            <!-- Core Metrics Grid -->
            <div class="grid grid-cols-2 md:grid-cols-4 gap-2">
                <!-- Current Fitness -->
                <div class="rounded bg-muted/40 p-2.5 border border-border/50">
                    <div class="flex items-center gap-1.5 text-xs text-muted-foreground mb-1">
                        <Target class="h-3.5 w-3.5" />
                        <span>Current Score</span>
                    </div>
                    <div class="text-base font-bold font-mono tracking-tight">
                        {currentFitness ? fmt(currentFitness) : "—"}
                    </div>
                </div>

                <!-- Best Fitness -->
                <div class="rounded bg-emerald-500/5 p-2.5 border border-emerald-500/10">
                    <div
                        class="flex items-center gap-1.5 text-xs text-emerald-600 dark:text-emerald-400 mb-1"
                    >
                        <Trophy class="h-3.5 w-3.5" />
                        <span>Best Score</span>
                    </div>
                    <div
                        class="text-base font-bold font-mono tracking-tight text-emerald-600 dark:text-emerald-400"
                    >
                        {bestFitness ? fmt(bestFitness) : "—"}
                    </div>
                </div>

                <!-- Temperature -->
                <div class="rounded bg-amber-500/5 p-2.5 border border-amber-500/10">
                    <div
                        class="flex items-center gap-1.5 text-xs text-amber-600 dark:text-amber-400 mb-1"
                    >
                        <Flame class="h-3.5 w-3.5" />
                        <span>Temperature</span>
                    </div>
                    <div
                        class="text-base font-bold font-mono tracking-tight text-amber-600 dark:text-amber-400"
                    >
                        {temperature ? fmtDec(temperature) : "—"}
                    </div>
                </div>

                <!-- Acceptance Ratio -->
                <div class="rounded bg-muted/40 p-2.5 border border-border/50">
                    <div class="flex items-center gap-1.5 text-xs text-muted-foreground mb-1">
                        <TrendingDown class="h-3.5 w-3.5" />
                        <span>Accept Rate</span>
                    </div>
                    <div class="text-base font-bold font-mono tracking-tight">
                        {acceptanceRate ? `${acceptanceRate * 100}%` : "—"}
                    </div>
                </div>
            </div>

            <!-- Interactive LayerChart -->
            <div class="space-y-1.5">
                <span
                    class="text-[11px] font-semibold text-muted-foreground uppercase tracking-wider block"
                >
                    Fitness Convergence Curve
                </span>
                <div class="h-40 w-full border rounded bg-muted/10 p-2 relative">
                    {#if history.length > 1}
                        <!-- LayerChart configuration mapping the x/y variables -->
                        <LineChart
                            data={history}
                            x="iteration"
                            y="fitness"
                            padding={{ top: 10, right: 10, bottom: 20, left: 35 }}
                            props={{
                                line: { class: "stroke-primary stroke-2" },
                                grid: { class: "stroke-muted/30" },
                            }}
                        />
                    {:else}
                        <div
                            class="absolute inset-0 flex items-center justify-center text-xs text-muted-foreground"
                        >
                            Accumulating iteration path...
                        </div>
                    {/if}
                </div>
            </div>
        </div>

        <Dialog.Footer>
            <Button onclick={onstop} variant="destructive">
                <Square />
                Stop
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>

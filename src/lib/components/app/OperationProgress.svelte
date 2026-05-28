<script lang="ts">
    import {
        Activity,
        CalendarCheck,
        CalendarRange,
        Check,
        Layers,
        Loader2,
        Square,
        Target,
        Trophy,
        X,
    } from "@lucide/svelte";

    import { apiInterrupt } from "$lib/api.js";
    import { app } from "$lib/app.svelte.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import * as Dialog from "$lib/components/ui/dialog/index.js";

    let open = $state(false);

    let operation = $derived(app.activeOp);
    let phase = $derived(app.operationPhase);
    let showSolver = $derived(operation === "solve" || operation === "orchestrate");
    let showRefiner = $derived(operation === "refine" || operation === "orchestrate");

    let weekendStats = $derived(app.solverProgress?.[0]);
    let weekendTotal = $derived((weekendStats?.accepted ?? 0) + (weekendStats?.rejected ?? 0));
    let weekendTarget = $derived(app.solverParams.weekend.number_permutations);
    let weekendPercent = $derived(Math.min(100, Math.round((weekendTotal / weekendTarget) * 100)));

    let fridayStats = $derived(app.solverProgress?.[1]);
    let fridayAttempts = $derived((fridayStats?.accepted ?? 0) + (fridayStats?.rejected ?? 0));
    let weekdayStats = $derived(app.solverProgress?.[2]);
    let weekdayAttempts = $derived((weekdayStats?.accepted ?? 0) + (weekdayStats?.rejected ?? 0));

    let accepted = $derived(app.refinerProgress?.accepted ?? 0);
    let rejected = $derived(app.refinerProgress?.rejected ?? 0);
    let refinerTotal = $derived(accepted + rejected);
    let refinerPercent = $derived(
        Math.min(100, Math.round((refinerTotal / app.refinerParams.num_iterations) * 100)),
    );

    let refined = $derived(app.orchestrationProgress?.refined ?? 0);
    let total = $derived(app.orchestrationProgress?.total ?? 0);
    let orchestrationPercent = $derived(total > 0 ? Math.round((refined / total) * 100) : 0);

    $effect(() => {
        open = app.activeOp !== null;
    });

    function fmt(num: number) {
        return new Intl.NumberFormat().format(Math.round(num));
    }

    function fmtScore(num: number | undefined | null) {
        return num == null || !Number.isFinite(num) || num >= 3e38
            ? "--"
            : new Intl.NumberFormat(undefined, { maximumFractionDigits: 2 }).format(num);
    }

    function title() {
        if (operation === "solve") return "Solve";
        if (operation === "refine") return "Refine";
        return "Auto";
    }

    function onstop() {
        apiInterrupt();
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content style="max-width: 48rem;">
        <Dialog.Header>
            <Dialog.Title class="flex items-center gap-2">
                <Activity class="h-5 w-5 text-primary animate-pulse" />
                {title()}
            </Dialog.Title>
            <Dialog.Description>
                {#if operation === "orchestrate"}
                    Solving candidates and refining the best set.
                {:else if operation === "solve"}
                    Generating candidate schedules.
                {:else}
                    Refining the current schedule.
                {/if}
            </Dialog.Description>
        </Dialog.Header>

        {#if operation === "orchestrate"}
            <div
                class="flex items-center justify-between rounded border bg-muted/30 px-3 py-2 text-xs"
            >
                <div class="flex items-center gap-2">
                    <Loader2 class="h-3.5 w-3.5 animate-spin text-primary" />
                    <span class="text-muted-foreground">Phase</span>
                    <span class="font-semibold text-foreground">
                        {phase === "refining" ? "Refining" : "Solving"}
                    </span>
                </div>
                <span class="font-mono text-muted-foreground">
                    Refined {fmt(refined)} / {fmt(total)}
                </span>
            </div>
        {/if}

        <div class="grid gap-3 py-2">
            {#if showSolver}
                <section class="rounded border p-3 {phase === 'refining' ? 'opacity-70' : ''}">
                    <div class="mb-3 flex items-center justify-between">
                        <div class="flex items-center gap-2 text-sm font-semibold">
                            <Layers class="h-4 w-4 text-primary" />
                            Solver
                        </div>
                        <span class="text-xs font-mono text-muted-foreground">
                            {fmt(weekendTotal)} / {fmt(weekendTarget)}
                        </span>
                    </div>

                    <div class="mb-3 h-1.5 w-full overflow-hidden rounded-full bg-secondary">
                        <div
                            class="h-full bg-primary transition-all duration-300"
                            style="width: {weekendPercent}%"
                        ></div>
                    </div>

                    <div class="grid grid-cols-3 gap-2 text-xs">
                        <div class="rounded border bg-muted/40 p-2">
                            <div class="mb-1 flex items-center gap-1 text-muted-foreground">
                                <Layers class="h-3.5 w-3.5" />
                                Weekends
                            </div>
                            <div class="font-mono">{fmt(weekendStats?.accepted ?? 0)} accepted</div>
                        </div>
                        <div class="rounded border bg-muted/40 p-2">
                            <div class="mb-1 flex items-center gap-1 text-muted-foreground">
                                <CalendarRange class="h-3.5 w-3.5" />
                                Fridays
                            </div>
                            <div class="font-mono">{fmt(fridayAttempts)} attempts</div>
                        </div>
                        <div class="rounded border bg-muted/40 p-2">
                            <div class="mb-1 flex items-center gap-1 text-muted-foreground">
                                <CalendarCheck class="h-3.5 w-3.5" />
                                Weekdays
                            </div>
                            <div class="font-mono">{fmt(weekdayAttempts)} attempts</div>
                        </div>
                    </div>
                </section>
            {/if}

            {#if showRefiner}
                <section
                    class="rounded border p-3 {operation === 'orchestrate' && phase !== 'refining'
                        ? 'opacity-60'
                        : ''}"
                >
                    <div class="mb-3 flex items-center justify-between">
                        <div class="flex items-center gap-2 text-sm font-semibold">
                            <Target class="h-4 w-4 text-primary" />
                            Refiner
                        </div>
                        <span class="text-xs font-mono text-muted-foreground">
                            {operation === "orchestrate"
                                ? `${fmt(refined)} / ${fmt(total)} candidates`
                                : `${fmt(refinerTotal)} iterations`}
                        </span>
                    </div>

                    <div class="mb-3 h-1.5 w-full overflow-hidden rounded-full bg-secondary">
                        <div
                            class="h-full bg-primary transition-all duration-300"
                            style="width: {operation === 'orchestrate'
                                ? orchestrationPercent
                                : refinerPercent}%"
                        ></div>
                    </div>

                    <div class="grid grid-cols-2 md:grid-cols-4 gap-2 text-xs">
                        <div class="rounded border bg-muted/40 p-2">
                            <div class="mb-1 flex items-center gap-1 text-muted-foreground">
                                <Target class="h-3.5 w-3.5" />
                                Current
                            </div>
                            <div class="font-mono">
                                {fmtScore(app.refinerProgress?.current_fitness)}
                            </div>
                        </div>
                        <div
                            class="rounded border bg-emerald-500/5 p-2 text-emerald-600 dark:text-emerald-400"
                        >
                            <div class="mb-1 flex items-center gap-1">
                                <Trophy class="h-3.5 w-3.5" />
                                Best
                            </div>
                            <div class="font-mono">
                                {fmtScore(
                                    app.orchestrationProgress?.best_fitness ??
                                        app.refinerProgress?.best_fitness,
                                )}
                            </div>
                        </div>
                        <div class="rounded border bg-muted/40 p-2">
                            <div class="mb-1 flex items-center gap-1 text-muted-foreground">
                                <Check class="h-3.5 w-3.5" />
                                Accepted
                            </div>
                            <div class="font-mono">{fmt(accepted)}</div>
                        </div>
                        <div class="rounded border bg-muted/40 p-2">
                            <div class="mb-1 flex items-center gap-1 text-muted-foreground">
                                <X class="h-3.5 w-3.5" />
                                Rejected
                            </div>
                            <div class="font-mono">{fmt(rejected)}</div>
                        </div>
                    </div>
                </section>
            {/if}
        </div>

        <Dialog.Footer>
            <Button onclick={onstop} variant="destructive">
                <Square />
                Stop
            </Button>
        </Dialog.Footer>
    </Dialog.Content>
</Dialog.Root>

<script lang="ts">
    import {
        Activity,
        CalendarCheck,
        CalendarRange,
        Check,
        Layers,
        Square,
        X,
    } from "@lucide/svelte";

    import { app } from "$lib/app.svelte.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import * as Dialog from "$lib/components/ui/dialog/index.js";

    let open = $state(false);

    let seeds = $state(500);
    let progress = $state([
        [50, 43],
        [439024, 42903],
        [12903014, 3494],
    ]);

    let weekendStatistics = $derived(app.solverProgress?.[0]);
    let weekendTotal = $derived(
        weekendStatistics ? weekendStatistics.accepted + weekendStatistics.rejected : 0,
    );
    let weekendAcceptance = $derived(weekendStatistics?.accepted ?? 0 / (weekendTotal || 1));
    let weekendPercent = $derived(Math.min(100, Math.round(weekendAcceptance * 100)));

    let fridayAttempts = $derived(progress[1][0]);
    let fridaySuccesses = $derived(progress[1][1]);
    let fridayFails = $derived(fridayAttempts - fridaySuccesses);

    let weekdayAttempts = $derived(progress[2][0]);
    let weekdaySuccesses = $derived(progress[2][1]);
    let weekdayFails = $derived(weekdayAttempts - weekdaySuccesses);

    $effect(() => {
        open = app.solverActive;
    });

    function fmt(num: number) {
        return new Intl.NumberFormat().format(num);
    }

    function onstop() {
        app.solverActive = false;
    }
</script>

<Dialog.Root bind:open>
    <Dialog.Content>
        <Dialog.Header>
            <Dialog.Title class="flex items-center gap-2">
                <Activity class="h-5 w-5 text-primary animate-pulse" />
                Solver
            </Dialog.Title>
            <Dialog.Description>
                Generating variations and pruning invalid schedules down the pipeline.
            </Dialog.Description>
        </Dialog.Header>

        <!-- Pipeline Content -->
        <div class="flex flex-col gap-1 py-4">
            <!-- Step 1: Weekend Variations -->
            <div class="relative pl-12 pb-6">
                <!-- Vertical Line -->
                <div
                    class="absolute left-4 top-8 bottom-0 w-0.5 bg-border border-dashed border-l"
                ></div>

                <!-- Icon Badge -->
                <div
                    class="absolute left-0 top-1 flex h-8 w-8 items-center justify-center rounded-full border bg-background shadow-sm"
                >
                    <Layers class="h-4 w-4 text-muted-foreground" />
                </div>

                <div class="space-y-1.5">
                    <h4 class="text-sm font-semibold leading-none">Weekends</h4>
                    <p class="text-xs text-muted-foreground">
                        Creates 10,000 variations per input permutation and picks the best to
                        continue.
                    </p>

                    <!-- Progress Bar -->
                    <div class="h-1.5 w-full bg-secondary rounded-full overflow-hidden mt-2">
                        <div
                            class="bg-primary h-full transition-all duration-300"
                            style="width: {weekendPercent}%"
                        ></div>
                    </div>

                    <div class="grid grid-cols-2 gap-2 pt-1 text-xs text-muted-foreground">
                        <div>
                            Seeds processed: <span class="font-medium text-foreground"
                                >{fmt(weekendTotal)} / {fmt(seeds)}</span
                            >
                        </div>

                        <div class="text-right">
                            Selected best: <span class="font-medium text-foreground"
                                >{fmt(weekendAcceptance)}</span
                            >
                        </div>
                    </div>
                </div>
            </div>

            <!-- Step 2: Friday Assignment -->
            <div class="relative pl-12 pb-6">
                <!-- Vertical Line -->
                <div
                    class="absolute left-4 top-8 bottom-0 w-0.5 bg-border border-dashed border-l"
                ></div>

                <!-- Icon Badge -->
                <div
                    class="absolute left-0 top-1 flex h-8 w-8 items-center justify-center rounded-full border bg-background shadow-sm"
                >
                    <CalendarRange class="h-4 w-4 text-muted-foreground" />
                </div>

                <div class="space-y-1.5">
                    <h4 class="text-sm font-semibold leading-none">Fridays</h4>

                    <p class="text-xs text-muted-foreground">
                        Attempts Friday schedules based on weekend outputs.
                    </p>

                    <div class="grid grid-cols-3 gap-2 pt-1 text-xs">
                        <div class="bg-muted/50 p-2 rounded">
                            <span
                                class="block text-[10px] text-muted-foreground uppercase tracking-wider"
                                >Attempts</span
                            >
                            <span class="font-medium text-foreground">{fmt(fridayAttempts)}</span>
                        </div>

                        <div class="bg-emerald-500/10 p-2 rounded border border-emerald-500/20">
                            <span
                                class="text-[10px] text-emerald-600 dark:text-emerald-400 uppercase tracking-wider flex items-center gap-1"
                            >
                                <Check class="h-3 w-3" /> Passed
                            </span>
                            <span class="font-semibold text-emerald-600 dark:text-emerald-400"
                                >{fmt(fridaySuccesses)}</span
                            >
                        </div>

                        <div class="bg-destructive/5 p-2 rounded border border-destructive/10">
                            <span
                                class="text-[10px] text-destructive uppercase tracking-wider flex items-center gap-1"
                            >
                                <X class="h-3 w-3" /> Dead Ends
                            </span>
                            <span class="font-medium text-destructive">{fmt(fridayFails)}</span>
                        </div>
                    </div>
                </div>
            </div>

            <!-- Step 3: Weekday Assignment -->
            <div class="relative pl-12">
                <!-- Icon Badge -->
                <div
                    class="absolute left-0 top-1 flex h-8 w-8 items-center justify-center rounded-full border bg-background shadow-sm"
                >
                    <CalendarCheck class="h-4 w-4 text-muted-foreground" />
                </div>

                <div class="space-y-1.5">
                    <div class="flex items-center justify-between">
                        <h4 class="text-sm font-semibold leading-none">Weekdays</h4>
                    </div>

                    <p class="text-xs text-muted-foreground">
                        Fills out weekday schedules and selects final candidates.
                    </p>

                    <div class="grid grid-cols-3 gap-2 pt-1 text-xs">
                        <div class="bg-muted/50 p-2 rounded">
                            <span
                                class="block text-[10px] text-muted-foreground uppercase tracking-wider"
                                >Attempts</span
                            >

                            <span class="font-medium text-foreground">{fmt(weekdayAttempts)}</span>
                        </div>

                        <div class="bg-emerald-500/10 p-2 rounded border border-emerald-500/20">
                            <span
                                class="text-[10px] text-emerald-600 dark:text-emerald-400 uppercase tracking-wider flex items-center gap-1"
                            >
                                <Check class="h-3 w-3" /> Solutions
                            </span>

                            <span class="font-bold text-emerald-600 dark:text-emerald-400"
                                >{fmt(weekdaySuccesses)}</span
                            >
                        </div>

                        <div class="bg-destructive/5 p-2 rounded border border-destructive/10">
                            <span
                                class="text-[10px] text-destructive uppercase tracking-wider flex items-center gap-1"
                            >
                                <X class="h-3 w-3" /> Dead Ends
                            </span>

                            <span class="font-medium text-destructive">{fmt(weekdayFails)}</span>
                        </div>
                    </div>
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

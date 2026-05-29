<script lang="ts">
    import {
        ArrowLeftRight,
        ChevronDown,
        Eraser,
        FlipHorizontal2,
        MousePointer2,
        Paintbrush,
        Square,
        Wand2,
        Zap,
    } from "@lucide/svelte";
    import { toast } from "svelte-sonner";

    import { apiInterrupt, apiOrchestrate, apiRefine, apiSolve } from "$lib/api.js";
    import { type ActiveMode, app } from "$lib/app.svelte.js";
    import SettingsDialog from "$lib/components/app/SettingsDialog.svelte";
    import ToolbarHistory from "$lib/components/app/ToolbarHistory.svelte";
    import ToolbarMenu from "$lib/components/app/ToolbarMenu.svelte";
    import ProgressRing from "$lib/components/misc/ProgressRing.svelte";
    import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import { Toggle } from "$lib/components/ui/toggle/index.js";
    import * as Tooltip from "$lib/components/ui/tooltip/index.js";
    import { APP_NAME } from "$lib/defs.js";
    import { useHotKeys } from "$lib/hooks/useHotkey.svelte";

    import Checkpoints from "./Checkpoints.svelte";

    const modes: { id: ActiveMode; label: string; icon: typeof MousePointer2; shortcut: string }[] =
        [
            { id: "select", label: "Select", icon: MousePointer2, shortcut: "S" },
            { id: "set", label: "Set", icon: Paintbrush, shortcut: "A" },
            { id: "swap_day", label: "Swap Day", icon: ArrowLeftRight, shortcut: "D" },
            { id: "swap_role", label: "Swap Role", icon: FlipHorizontal2, shortcut: "R" },
            { id: "erase", label: "Erase", icon: Eraser, shortcut: "E" },
        ];

    type RequestedOperation = "solve" | "refine" | "orchestrate";

    useHotKeys(null, handleHotkey);

    let confirmOperation = $state<RequestedOperation | null>(null);

    let activeOp = $derived(app.activeOp);

    let solverRatio = $derived(
        Math.min(
            1,
            (app.solverProgress?.[0].accepted ?? 0) / app.solverParams.weekend.number_permutations,
        ),
    );

    let refinerRatio = $derived(
        Math.min(1, (app.refinerProgress?.iteration ?? 0) / app.refinerParams.num_iterations),
    );

    let orchestrateRatio = $derived.by(() => {
        const p = app.orchestrationProgress;
        if (!p || p.total === 0) return 0;
        return Math.min(1, p.refined / p.total);
    });

    let solverPercent = $derived(solverRatio * 100);
    let refinerPercent = $derived(refinerRatio * 100);
    let orchestratePercent = $derived(orchestrateRatio * 100);
    let activePercent = $derived.by(() => {
        if (activeOp === "solve") return solverPercent;
        if (activeOp === "refine") return refinerPercent;
        if (activeOp === "orchestrate") {
            return app.operationPhase === "solving" ? solverPercent : orchestratePercent;
        }
        return 0;
    });

    function requestOperation(operation: RequestedOperation) {
        confirmOperation = operation;
    }

    function onstop() {
        apiInterrupt();
    }

    async function runOperation(operation: RequestedOperation) {
        try {
            if (operation === "solve") {
                const { solution } = await apiSolve(app, app.solverParams, app.weights);
                app.loadSlots(solution);
            } else if (operation === "refine") {
                const [, solution] = await apiRefine(
                    app,
                    app.refinerParams,
                    app.slots,
                    app.weights,
                );
                app.loadSlots(solution);
            } else {
                const { solution } = await apiOrchestrate(
                    app,
                    { top_k: app.topK },
                    app.solverParams,
                    app.refinerParams,
                    app.weights,
                );
                app.loadSlots(solution);
            }
        } catch (error) {
            toast.error("Failed to run operation", { description: `${error}` });
        }
    }

    function onconfirm() {
        if (!confirmOperation) return;
        const operation = confirmOperation;
        confirmOperation = null;
        runOperation(operation);
    }

    function confirmTitle() {
        if (confirmOperation === "solve") return "Run solver?";
        if (confirmOperation === "refine") return "Run refiner?";
        return "Run Auto?";
    }

    function confirmDescription() {
        if (confirmOperation === "solve") {
            return "The solver will replace the current schedule with a generated schedule.";
        }

        if (confirmOperation === "refine") {
            return "The refiner will replace the current schedule with the best refined result.";
        }

        return "Auto will solve candidate schedules, refine the top results, and replace the current schedule with the best final result.";
    }

    function handleHotkey(event: KeyboardEvent) {
        for (const m of modes) {
            if (event.key.toUpperCase() === m.shortcut) {
                app.activeMode = m.id;
                return;
            }
        }
    }
</script>

<header class="flex items-center h-11 px-3 gap-2 shrink-0 border-b border-border bg-card">
    <div class="flex-1 flex items-center gap-4">
        <span class="text-[13px] font-extrabold whitespace-nowrap uppercase">{APP_NAME}</span>
        <ToolbarMenu />
    </div>

    <div class="flex-1 flex justify-center">
        <div
            class="flex items-center gap-0.5 rounded-md border p-0.5"
            role="toolbar"
            style:--muted="var(--primary)"
        >
            {#each modes as m}
                {@const pressed = app.activeMode === m.id}
                <Tooltip.Root>
                    <Tooltip.Trigger>
                        {#snippet child({ props })}
                            <Toggle
                                {...props}
                                bind:pressed={() => pressed, () => (app.activeMode = m.id)}
                                class="gap-1 h-7 text-xs hover:text-white {pressed
                                    ? 'text-white'
                                    : ''}"
                                size="sm"
                            >
                                <m.icon />
                            </Toggle>
                        {/snippet}
                    </Tooltip.Trigger>
                    <Tooltip.Content>
                        <span>{m.label} ({m.shortcut})</span>
                    </Tooltip.Content>
                </Tooltip.Root>
            {/each}
        </div>
    </div>

    <div class="flex-1 flex items-center justify-end gap-4">
        <Checkpoints />
        <SettingsDialog />
        <ToolbarHistory />

        <div class="flex items-center">
            {#if activeOp}
                <Button variant="destructive" onclick={onstop} class="rounded-r-none">
                    <ProgressRing value={activePercent} />
                    Stop
                </Button>
            {:else}
                <Button onclick={() => requestOperation("orchestrate")} class="rounded-r-none">
                    <Wand2 />
                    Auto
                </Button>
            {/if}

            <DropdownMenu.Root>
                <DropdownMenu.Trigger disabled={activeOp !== null}>
                    {#snippet child({ props })}
                        <Button
                            {...props}
                            variant={activeOp ? "destructive" : "default"}
                            disabled={activeOp !== null}
                            class="rounded-l-none border-l border-primary-foreground/20 px-2"
                        >
                            {#if activeOp}
                                <Square />
                            {:else}
                                <ChevronDown />
                            {/if}
                        </Button>
                    {/snippet}
                </DropdownMenu.Trigger>

                <DropdownMenu.Content align="end" class="w-40">
                    <DropdownMenu.Item onclick={() => requestOperation("solve")}>
                        <Wand2 />
                        Solve
                    </DropdownMenu.Item>

                    <DropdownMenu.Item onclick={() => requestOperation("refine")}>
                        <Zap />
                        Refine
                    </DropdownMenu.Item>
                </DropdownMenu.Content>
            </DropdownMenu.Root>
        </div>
    </div>
</header>

<!-- Confirmation dialog -->
<AlertDialog.Root bind:open={() => confirmOperation !== null, () => (confirmOperation = null)}>
    <AlertDialog.Content>
        <AlertDialog.Header>
            <AlertDialog.Title>{confirmTitle()}</AlertDialog.Title>
            <AlertDialog.Description>
                {confirmDescription()}
            </AlertDialog.Description>
        </AlertDialog.Header>
        <AlertDialog.Footer>
            <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
            <AlertDialog.Action onclick={onconfirm}>Continue</AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>

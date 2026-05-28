<script lang="ts">
    import {
        ArrowLeftRight,
        Eraser,
        FlipHorizontal2,
        Import,
        MousePointer2,
        Paintbrush,
        Play,
        Save,
        Wand2,
        Zap,
    } from "@lucide/svelte";

    import { type ActiveMode, app } from "$lib/app.svelte";
    import * as AlertDialog from "$lib/components/ui/alert-dialog/index.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import { Separator } from "$lib/components/ui/separator/index.js";
    import { Toggle } from "$lib/components/ui/toggle/index.js";
    import * as Tooltip from "$lib/components/ui/tooltip/index.js";
    import { APP_NAME } from "$lib/defs.js";

    import ProgressRing from "../misc/ProgressRing.svelte";
    import ToolbarHistory from "./ToolbarHistory.svelte";
    import ToolbarSettings from "./ToolbarSettings.svelte";

    const modes: { id: ActiveMode; label: string; icon: typeof MousePointer2; shortcut: string }[] =
        [
            { id: "select", label: "Select", icon: MousePointer2, shortcut: "S" },
            { id: "set", label: "Set", icon: Paintbrush, shortcut: "A" },
            { id: "swap_day", label: "Swap Day", icon: ArrowLeftRight, shortcut: "D" },
            { id: "swap_role", label: "Swap Role", icon: FlipHorizontal2, shortcut: "R" },
            { id: "erase", label: "Erase", icon: Eraser, shortcut: "E" },
        ];

    let confirmSolve = $state(false);

    let solverRatio = $derived(
        Math.min(1, (app.solverProgress?.[0].accepted ?? 0) / app.solverPopulation),
    );

    let refinerRatio = $derived(
        Math.min(1, (app.refinerProgress?.iteration ?? 0) / app.refinerRounds),
    );

    let orchestrateRatio = $derived(() => {
        const p = app.orchestratorProgress;
        if (!p || p.total === 0) return 0;
        return Math.min(1, p.refined / p.total);
    });

    let solverPercent = $derived(solverRatio * 100);
    let refinerPercent = $derived(refinerRatio * 100);
    let orchestratePercent = $derived(orchestrateRatio * 100);

    function onsolve() {
        confirmSolve = true;
    }

    function onstopsolver() {
        app.solverActive = false;
    }

    function onrefine() {
        app.refinerActive = true;
    }

    function onstoprefine() {
        app.refinerActive = false;
    }

    function onorchestrate() {
        app.orchestratorActive = true;
    }

    function onstoporchestrate() {
        app.orchestratorActive = false;
    }

    function onconfirm() {
        confirmSolve = false;
        app.solverActive = true;
    }

    function onkeydown(event: KeyboardEvent) {
        for (const m of modes) {
            if (event.key.toUpperCase() === m.shortcut) {
                app.activeMode = m.id;
                return;
            }
        }
    }
</script>

<svelte:window {onkeydown} />

<header
    class="flex items-center justify-between h-11 px-3 gap-2 shrink-0 border-b border-border bg-card"
>
    <div class="flex items-center gap-1">
        <span class="text-[13px] font-semibold whitespace-nowrap">{APP_NAME}</span>
        <Separator orientation="vertical" class="h-5 mx-1" />
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

    <div class="flex items-center gap-1">
        <ToolbarHistory />
        <ToolbarSettings />

        <Separator orientation="vertical" class="h-5 mx-1" />

        <div class="w-2"></div>

        {#if app.solverActive}
            <Button variant="destructive" onclick={onstopsolver}>
                <ProgressRing value={solverPercent} />
                Stop
            </Button>
        {:else}
            <Button variant="outline" onclick={onsolve} disabled={app.refinerActive || app.orchestratorActive}>
                <Play />
                Solve
            </Button>
        {/if}

        {#if app.refinerActive}
            <Button variant="destructive" onclick={onstoprefine}>
                <ProgressRing value={refinerPercent} />
                Stop
            </Button>
        {:else}
            <Button onclick={onrefine} disabled={app.solverActive || app.orchestratorActive}>
                <Zap />
                Refine
            </Button>
        {/if}

        {#if app.orchestratorActive}
            <Button variant="destructive" onclick={onstoporchestrate}>
                <ProgressRing value={orchestratePercent} />
                Stop
            </Button>
        {:else}
            <Button onclick={onorchestrate} disabled={app.solverActive || app.refinerActive}>
                <Wand2 />
                Orchestrate
            </Button>
        {/if}
    </div>
</header>

<AlertDialog.Root bind:open={confirmSolve}>
    <AlertDialog.Content>
        <AlertDialog.Header>
            <AlertDialog.Title>Generate new schedule?</AlertDialog.Title>

            <AlertDialog.Description>
                Starting the solver will replace the current schedule. Please make sure you have
                saved your changes.
            </AlertDialog.Description>
        </AlertDialog.Header>

        <AlertDialog.Footer>
            <AlertDialog.Cancel>Cancel</AlertDialog.Cancel>
            <AlertDialog.Action onclick={onconfirm}>Continue</AlertDialog.Action>
        </AlertDialog.Footer>
    </AlertDialog.Content>
</AlertDialog.Root>

<script lang="ts">
    import { Redo2, Undo2 } from "@lucide/svelte";

    import { app } from "$lib/app.svelte";
    import { Button, buttonVariants } from "$lib/components/ui/button/index.js";
    import * as Tooltip from "$lib/components/ui/tooltip/index.js";

    let undoDisabled = $derived(app.historyCursor === 0);
    let redoDisabled = $derived(app.historyCursor === app.history.length);

    function undo() {
        if (app.historyCursor === 0) return;
        app.historyCursor = Math.max(0, app.historyCursor - 1);
        app.slots = app.history[app.historyCursor]!;
    }

    function redo() {
        if (app.historyCursor === app.history.length - 1) return;
        app.historyCursor = Math.min(app.history.length - 1, app.historyCursor + 1);
        app.slots = app.history[app.historyCursor]!;
    }

    function onkeydown(event: KeyboardEvent) {
        if (!event.ctrlKey) return;
        if (event.key === "z") undo();
        if (event.key === "y") redo();
    }
</script>

<svelte:window {onkeydown} />

<Tooltip.Root>
    <Tooltip.Trigger
        class={buttonVariants({ variant: "ghost", size: "icon" })}
        disabled={undoDisabled}
    >
        <Undo2 />
    </Tooltip.Trigger>

    <Tooltip.Content>Undo (Ctrl+Z)</Tooltip.Content>
</Tooltip.Root>

<Tooltip.Root>
    <Tooltip.Trigger
        class={buttonVariants({ variant: "ghost", size: "icon" })}
        disabled={redoDisabled}
    >
        <Redo2 />
    </Tooltip.Trigger>

    <Tooltip.Content>Redo (Ctrl+Y)</Tooltip.Content>
</Tooltip.Root>

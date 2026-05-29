<script lang="ts">
    import { Redo2, Undo2 } from "@lucide/svelte";

    import { redo, undo } from "$lib/actions.js";
    import { app } from "$lib/app.svelte.js";
    import { buttonVariants } from "$lib/components/ui/button/index.js";
    import * as Tooltip from "$lib/components/ui/tooltip/index.js";
    import { useHotKeys } from "$lib/hooks/useHotkey.svelte.js";

    useHotKeys(null, handleHotKey, true);

    function handleHotKey(event: KeyboardEvent) {
        if (!event.ctrlKey) return;
        if (event.key === "z") undo();
        if (event.key === "y") redo();
    }
</script>

<Tooltip.Root>
    <Tooltip.Trigger
        class={buttonVariants({ variant: "ghost", size: "icon" })}
        disabled={!app.history.canUndo}
        onclick={undo}
    >
        <Undo2 />
    </Tooltip.Trigger>

    <Tooltip.Content>Undo (Ctrl+Z)</Tooltip.Content>
</Tooltip.Root>

<Tooltip.Root>
    <Tooltip.Trigger
        class={buttonVariants({ variant: "ghost", size: "icon" })}
        disabled={!app.history.canRedo}
        onclick={redo}
    >
        <Redo2 />
    </Tooltip.Trigger>

    <Tooltip.Content>Redo (Ctrl+Y)</Tooltip.Content>
</Tooltip.Root>

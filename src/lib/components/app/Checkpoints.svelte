<script lang="ts">
    import { BookmarkIcon, PlusIcon } from "@lucide/svelte";
    import { toast } from "svelte-sonner";

    import { type Checkpoint, app } from "$lib/app.svelte.js";
    import { Button } from "$lib/components/ui/button/index.js";
    import * as Empty from "$lib/components/ui/empty/index.js";
    import * as Popover from "$lib/components/ui/popover/index.js";
    import * as Tooltip from "$lib/components/ui/tooltip/index.js";
    import { useHotKey } from "$lib/hooks/useHotkey.svelte";
    import { timestamp } from "$lib/misc.js";

    import Kbd from "../ui/kbd/kbd.svelte";
    import CheckpointItem from "./CheckpointItem.svelte";

    let open = $state(false);

    useHotKey("h", () => (open = !open));

    function createCheckpoint() {
        app.checkpoints.push({
            name: "Unnamed Checkpoint",
            slots: [...app.slots],
            timestamp: timestamp(),
        });

        toast.success("Checkpoint created");
    }

    function deleteCheckpoint(index: number) {
        app.checkpoints.splice(index, 1);
        toast.success("Checkpoint deleted");
    }

    function restoreCheckpoint(checkpoint: Checkpoint) {
        app.loadSlots(checkpoint.slots);
        toast.success("Checkpoint restored", { description: checkpoint.name });
    }
</script>

<Popover.Root bind:open>
    <Popover.Trigger>
        {#snippet child({ props })}
            <Tooltip.Root>
                <Tooltip.Trigger>
                    <Button {...props} variant="ghost" size="icon">
                        <BookmarkIcon />
                    </Button>
                </Tooltip.Trigger>

                <Tooltip.Content>
                    Checkpoints <Kbd>h</Kbd>
                </Tooltip.Content>
            </Tooltip.Root>
        {/snippet}
    </Popover.Trigger>

    <Popover.Content class="w-80">
        {#if app.checkpoints.length > 0}
            <Button onclick={createCheckpoint} variant="ghost">
                <div class="px-4 py-8 flex justify-center items-center">
                    <PlusIcon /> New
                </div>
            </Button>

            {#each app.checkpoints.toReversed() as checkpoint, i}
                <CheckpointItem
                    {checkpoint}
                    ondelete={() => deleteCheckpoint(i)}
                    onrestore={() => restoreCheckpoint(checkpoint)}
                />
            {/each}
        {:else}
            {@render noCheckpoints()}
        {/if}
    </Popover.Content>
</Popover.Root>

{#snippet noCheckpoints()}
    <Empty.Root>
        <Empty.Header>
            <Empty.Media variant="icon"><BookmarkIcon /></Empty.Media>
            <Empty.Title>No Checkpoints Yet</Empty.Title>
            <Empty.Description>Create a checkpoint to save your current schedule.</Empty.Description
            >
        </Empty.Header>

        <Empty.Content>
            <Button onclick={createCheckpoint}>Create Checkpoint</Button>
        </Empty.Content>
    </Empty.Root>
{/snippet}

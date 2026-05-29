<script lang="ts">
    import { ArchiveRestoreIcon, CheckIcon, Trash2Icon } from "@lucide/svelte";
    import { format } from "date-fns";

    import type { Checkpoint } from "$lib/app.svelte.js";
    import { Button, buttonVariants } from "$lib/components/ui/button/index.js";
    import { Input } from "$lib/components/ui/input/index.js";
    import * as Item from "$lib/components/ui/item/index.js";
    import * as Tooltip from "$lib/components/ui/tooltip/index.js";
    import { dateFromTimestamp } from "$lib/misc.js";

    interface Props {
        checkpoint: Checkpoint;
        ondelete: () => void;
        onrestore: () => void;
    }

    let { checkpoint, ondelete, onrestore }: Props = $props();

    let edit = $state(false);

    let date = $derived(dateFromTimestamp(checkpoint.timestamp));

    function editTitle() {
        edit = !edit;
    }
</script>

<Item.Root variant="outline">
    <Item.Content>
        <Item.Title>
            {#if edit}
                <Input bind:value={checkpoint.name} onblur={editTitle} />
                <Button variant="ghost" size="icon" onclick={editTitle} class="text-green-500"
                    ><CheckIcon /></Button
                >
            {:else}
                <button
                    class={[
                        "cursor-pointer",
                        checkpoint.name.includes("Unnamed") && "italic text-muted-foreground",
                    ]}
                    onclick={editTitle}
                >
                    {checkpoint.name}
                </button>
            {/if}
        </Item.Title>

        <Item.Description>{format(date, "dd MMM yyyy")}</Item.Description>
    </Item.Content>

    <Item.Actions>
        <Tooltip.Root>
            <Tooltip.Trigger
                class={buttonVariants({ variant: "outline", size: "icon" })}
                onclick={onrestore}
            >
                <ArchiveRestoreIcon />
            </Tooltip.Trigger>

            <Tooltip.Content>Restore checkpoint</Tooltip.Content>
        </Tooltip.Root>

        <Tooltip.Root>
            <Tooltip.Trigger
                class={buttonVariants({ variant: "destructive", size: "icon" })}
                onclick={ondelete}
            >
                <Trash2Icon />
            </Tooltip.Trigger>

            <Tooltip.Content>Delete checkpoint</Tooltip.Content>
        </Tooltip.Root>
    </Item.Actions>
</Item.Root>

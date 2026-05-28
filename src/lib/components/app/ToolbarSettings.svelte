<script lang="ts">
    import { Settings } from "@lucide/svelte";

    import { app, resetApp } from "$lib/app.svelte";
    import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js";
    import * as Tooltip from "$lib/components/ui/tooltip/index.js";

    import { buttonVariants } from "../ui/button";

    function onNew() {
        resetApp();
    }

    function onOpen() {}

    function onSave() {}

    function onExport() {}
</script>

<DropdownMenu.Root>
    <DropdownMenu.Trigger>
        {#snippet child({ props })}
            <Tooltip.Root>
                <Tooltip.Trigger {...props} class={buttonVariants({ variant: "ghost" })}>
                    <Settings class="size-4" />
                </Tooltip.Trigger>

                <Tooltip.Content>Settings</Tooltip.Content>
            </Tooltip.Root>
        {/snippet}
    </DropdownMenu.Trigger>

    <DropdownMenu.Content class="w-56" align="end">
        <DropdownMenu.Group>
            <DropdownMenu.Item onclick={onNew}>New</DropdownMenu.Item>
            <DropdownMenu.Item onclick={onOpen}>Open</DropdownMenu.Item>
            <DropdownMenu.Item onclick={onSave}>Save</DropdownMenu.Item>
            <DropdownMenu.Item onclick={onExport}>Export PDF</DropdownMenu.Item>
        </DropdownMenu.Group>

        <DropdownMenu.Separator />

        <DropdownMenu.Group>
            <DropdownMenu.Label>Zoom level</DropdownMenu.Label>

            <DropdownMenu.RadioGroup bind:value={app.zoomLevel}>
                <DropdownMenu.RadioItem value="micro">Micro</DropdownMenu.RadioItem>
                <DropdownMenu.RadioItem value="standard">Standard</DropdownMenu.RadioItem>
                <DropdownMenu.RadioItem value="detail">Detail</DropdownMenu.RadioItem>
            </DropdownMenu.RadioGroup>
        </DropdownMenu.Group>
    </DropdownMenu.Content>
</DropdownMenu.Root>

<script lang="ts">
    import { redo, undo } from "$lib/actions.js";
    import { GridMode, ZoomLevel, app, resetApp, view } from "$lib/app.svelte.js";
    import * as Menubar from "$lib/components/ui/menubar/index.js";
</script>

<Menubar.Root>
    <Menubar.Menu>
        <Menubar.Trigger>File</Menubar.Trigger>

        <Menubar.Content>
            <Menubar.Item onclick={resetApp}>New</Menubar.Item>
            <Menubar.Item>Open</Menubar.Item>
            <Menubar.Item>Save</Menubar.Item>
            <Menubar.Item>Export PDF</Menubar.Item>
        </Menubar.Content>
    </Menubar.Menu>

    <Menubar.Menu>
        <Menubar.Trigger>Edit</Menubar.Trigger>

        <Menubar.Content>
            <Menubar.Item onclick={undo} disabled={!app.history.canUndo}>
                Undo <Menubar.Shortcut>⌘Z</Menubar.Shortcut>
            </Menubar.Item>
            <Menubar.Item onclick={redo} disabled={!app.history.canRedo}>
                Redo <Menubar.Shortcut>⌘Y</Menubar.Shortcut>
            </Menubar.Item>
        </Menubar.Content>
    </Menubar.Menu>

    <Menubar.Menu>
        <Menubar.Trigger>View</Menubar.Trigger>

        <Menubar.Content>
            <Menubar.RadioGroup bind:value={view.zoom}>
                <Menubar.GroupHeading>Zoom level</Menubar.GroupHeading>
                <Menubar.RadioItem value={ZoomLevel.Micro}>Micro</Menubar.RadioItem>
                <Menubar.RadioItem value={ZoomLevel.Standard}>Standard</Menubar.RadioItem>
                <Menubar.RadioItem value={ZoomLevel.Detail}>Detail</Menubar.RadioItem>
            </Menubar.RadioGroup>

            <Menubar.Separator />

            <Menubar.RadioGroup bind:value={view.gridMode}>
                <Menubar.GroupHeading>Grid mode</Menubar.GroupHeading>
                <Menubar.RadioItem value={GridMode.Bars}>Bars</Menubar.RadioItem>
                <Menubar.RadioItem value={GridMode.Filled}>Filled</Menubar.RadioItem>
            </Menubar.RadioGroup>
        </Menubar.Content>
    </Menubar.Menu>
</Menubar.Root>

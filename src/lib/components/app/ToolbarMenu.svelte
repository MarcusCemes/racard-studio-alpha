<script lang="ts">
    import { app } from "$lib/app.svelte.js";
    import * as Menubar from "$lib/components/ui/menubar/index.js";

    let bookmarks = $state(false);
    let fullUrls = $state(true);
    let profileRadioValue = $state("benoit");
</script>

<Menubar.Root>
    <Menubar.Menu>
        <Menubar.Trigger>File</Menubar.Trigger>

        <Menubar.Content>
            <Menubar.Item>New</Menubar.Item>
            <Menubar.Item>Open</Menubar.Item>
            <Menubar.Item>Save</Menubar.Item>
            <Menubar.Item>Export PDF</Menubar.Item>
        </Menubar.Content>
    </Menubar.Menu>

    <Menubar.Menu>
        <Menubar.Trigger>Edit</Menubar.Trigger>

        <Menubar.Content>
            <Menubar.Item>
                Undo <Menubar.Shortcut>⌘Z</Menubar.Shortcut>
            </Menubar.Item>
            <Menubar.Item>
                Redo <Menubar.Shortcut>⌘Y</Menubar.Shortcut>
            </Menubar.Item>
        </Menubar.Content>
    </Menubar.Menu>

    <Menubar.Menu>
        <Menubar.Trigger>View</Menubar.Trigger>

        <Menubar.Content>
            <Menubar.RadioGroup bind:value={app.zoomLevel}>
                <Menubar.GroupHeading>Zoom level</Menubar.GroupHeading>
                <Menubar.RadioItem value="micro">Micro</Menubar.RadioItem>
                <Menubar.RadioItem value="standard">Standard</Menubar.RadioItem>
                <Menubar.RadioItem value="detail">Detail</Menubar.RadioItem>
            </Menubar.RadioGroup>
        </Menubar.Content>
    </Menubar.Menu>
</Menubar.Root>

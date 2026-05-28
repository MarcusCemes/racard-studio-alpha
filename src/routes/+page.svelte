<script lang="ts">
    import InspectorPanel from "$lib/components/app/InspectorPanel.svelte";
    import RefinerProgress from "$lib/components/app/RefinerProgress.svelte";
    import RosterPanel from "$lib/components/app/RosterPanel.svelte";
    import ScheduleGrid from "$lib/components/app/ScheduleGrid.svelte";
    import SolverProgress from "$lib/components/app/SolverProgress.svelte";
    import StatusBar from "$lib/components/app/StatusBar.svelte";
    import Toolbar from "$lib/components/app/Toolbar.svelte";
</script>

<div class="flex flex-col h-screen overflow-hidden select-none">
    <Toolbar />

    <div class="flex flex-1 min-h-0 overflow-hidden">
        <RosterPanel />
        <ScheduleGrid />
        <InspectorPanel />
    </div>

    <StatusBar />
</div>

<SolverProgress />
<RefinerProgress />

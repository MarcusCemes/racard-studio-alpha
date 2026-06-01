<script lang="ts">
    import { BarChart3 } from "@lucide/svelte";

    import AnalyticsCharts from "$lib/components/app/AnalyticsCharts.svelte";
    import AnalyticsDistribution from "$lib/components/app/AnalyticsDistribution.svelte";
    import AnalyticsHeatmap from "$lib/components/app/AnalyticsHeatmap.svelte";
    import AnalyticsSummary from "$lib/components/app/AnalyticsSummary.svelte";
    import { Button } from "$lib/components/ui/button/index.js";
    import * as Dialog from "$lib/components/ui/dialog/index.js";
    import Kbd from "$lib/components/ui/kbd/kbd.svelte";
    import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";
    import * as Tabs from "$lib/components/ui/tabs/index.js";
    import * as Tooltip from "$lib/components/ui/tooltip/index.js";
    import { useHotKey } from "$lib/hooks/useHotkey.svelte.js";

    let open = $state(false);
    let activeTab = $state("summary");

    useHotKey("f", () => (open = !open));
</script>

<Dialog.Root bind:open>
    <Dialog.Trigger>
        {#snippet child({ props })}
            <Tooltip.Root>
                <Tooltip.Trigger>
                    <Button {...props} variant="ghost" size="icon">
                        <BarChart3 />
                    </Button>
                </Tooltip.Trigger>

                <Tooltip.Content>
                    Open analytics <Kbd>f</Kbd>
                </Tooltip.Content>
            </Tooltip.Root>
        {/snippet}
    </Dialog.Trigger>

    <Dialog.Content class="sm:h-[85vh] sm:max-w-5xl flex flex-col">
        <div class="flex">
            <Dialog.Header class="flex-1">
                <Dialog.Title>Analytics</Dialog.Title>
                <Dialog.Description>Schedule statistics and visualisations.</Dialog.Description>
            </Dialog.Header>

            <div class="flex justify-center items-center">
                <Tabs.Root bind:value={activeTab}>
                    <Tabs.List>
                        <Tabs.Trigger value="summary">Summary</Tabs.Trigger>
                        <Tabs.Trigger value="distribution">Distribution</Tabs.Trigger>
                        <Tabs.Trigger value="heatmap">Heatmap</Tabs.Trigger>
                        <Tabs.Trigger value="charts">Charts</Tabs.Trigger>
                    </Tabs.List>
                </Tabs.Root>
            </div>

            <div class="flex-1"></div>
        </div>

        <ScrollArea class="flex-1 overflow-y-auto px-2">
            {#if activeTab === "summary"}
                <AnalyticsSummary />
            {:else if activeTab === "distribution"}
                <AnalyticsDistribution />
            {:else if activeTab === "heatmap"}
                <AnalyticsHeatmap />
            {:else if activeTab === "charts"}
                <AnalyticsCharts />
            {/if}
        </ScrollArea>
    </Dialog.Content>
</Dialog.Root>

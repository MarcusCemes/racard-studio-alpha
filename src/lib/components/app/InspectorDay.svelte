<script lang="ts">
    import { app } from "$lib/app.svelte.js";
    import { Separator } from "$lib/components/ui/separator/index.js";
    import * as Table from "$lib/components/ui/table/index.js";
    import { N_WEEKDAYS, PERSON_COLORS } from "$lib/defs.js";
    import { getLead, getSupport } from "$lib/slot.js";

    interface Props {
        day: number;
    }

    let { day }: Props = $props();

    let slot = $derived(app.slots[day]);
    let week = $derived(Math.floor(day / N_WEEKDAYS));

    let lead = $derived(getLead(slot));
    let support = $derived(getSupport(slot));
</script>

<div class="mx-3.5 mb-3 rounded-lg border border-border overflow-hidden bg-background">
    <div class="flex items-center gap-1.5 px-3 py-2.5 min-h-10.5">
        <span
            class="text-[10px] font-bold uppercase tracking-[0.07em] text-muted-foreground w-11 shrink-0"
            >Lead</span
        >

        {#if lead !== undefined}
            {@const [, swatch] = PERSON_COLORS[lead]}
            <span class="w-2.5 h-2.5 rounded-[3px] shrink-0" style="background:{swatch}"></span>
            <span class="flex-1 text-[12.5px] font-medium truncate">{app.people[lead].name}</span>
        {:else}
            <span class="text-xs text-muted-foreground italic">Unassigned</span>
        {/if}
    </div>

    <Separator />

    <div class="flex items-center gap-1.5 px-3 py-2.5 min-h-10.5">
        <span
            class="text-[10px] font-bold uppercase tracking-[0.07em] text-muted-foreground w-11 shrink-0"
            >Support</span
        >
        {#if support !== undefined}
            {@const [, swatch] = PERSON_COLORS[support]}

            <span class="w-2.5 h-2.5 rounded-[3px] shrink-0" style="background:{swatch}"></span>
            <span class="flex-1 text-[12.5px] font-medium truncate">{app.people[support].name}</span
            >
        {:else}
            <span class="text-xs text-muted-foreground italic">Unassigned</span>
        {/if}
    </div>
</div>

<div class="mx-3.5 mb-2.5 text-[11.5px] text-muted-foreground italic">No violations</div>

{#if app.statistics}
    <div class="mt-8 px-4">
        <Table.Root class="text-xs">
            <Table.Caption>Weekly totals</Table.Caption>
            <Table.Header>
                <Table.Row>
                    <Table.Head class="w-25">Person</Table.Head>
                    <Table.Head class="text-end">Lead</Table.Head>
                    <Table.Head class="text-end">Support</Table.Head>
                </Table.Row>
            </Table.Header>

            <Table.Body>
                {#each app.people as _, i}
                    {@const {
                        hours_by_role: [lead, support],
                    } = app.statistics.people[i].weeks[week]}
                    {@const name = app.formattedNames[i]}

                    <Table.Row>
                        <Table.Cell class="font-medium">{name}</Table.Cell>
                        <Table.Cell class="text-end font-mono">{@render delta(lead)}</Table.Cell>
                        <Table.Cell class="text-end font-mono">{@render delta(support)}</Table.Cell>
                    </Table.Row>
                {/each}
            </Table.Body>
        </Table.Root>
    </div>
{/if}

{#snippet delta(amount: number)}
    {#if amount === 0}
        <span class="text-muted-foreground">&mdash;</span>
    {:else}
        <span class="text-green-500">+{amount.toFixed(2)}</span>
    {/if}
{/snippet}

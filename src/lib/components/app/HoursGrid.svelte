<script lang="ts">
    import { Input } from "$lib/components/ui/input/index.js";
    import { WEEKDAYS } from "$lib/defs.js";

    let { hours = $bindable([]) }: { hours: [number, number][] } = $props();
</script>

<div class="flex flex-col gap-1.5">
    <!-- Header: Lead / Support -->
    <div class="flex items-center gap-2 px-1">
        <span class="flex-1 text-[10px] font-semibold uppercase tracking-[0.06em] text-muted-foreground"></span>
        <span class="w-12 text-center text-[9.5px] font-semibold uppercase tracking-[0.05em] text-muted-foreground">Lead</span>
        <span class="w-12 text-center text-[9.5px] font-semibold uppercase tracking-[0.05em] text-muted-foreground">Supp.</span>
    </div>

    <!-- Rows -->
    {#each WEEKDAYS as day, i}
        <div class="flex items-center gap-2">
            <span class="flex-1 text-[11.5px] text-muted-foreground">{day}</span>
            <Input
                type="number"
                bind:value={hours[i][0]}
                class="w-12! h-7! text-[11.5px]! font-mono! text-right!"
            />
            <Input
                type="number"
                bind:value={hours[i][1]}
                class="w-12! h-7! text-[11.5px]! font-mono! text-right!"
            />
        </div>
    {/each}
</div>

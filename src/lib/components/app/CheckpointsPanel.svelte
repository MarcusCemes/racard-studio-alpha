<script lang="ts">
    import { app } from "$lib/app.svelte";
    import { Button } from "$lib/components/ui/button/index.js";
</script>

<div>
    <div
        class="px-3.5 pt-2 pb-1 text-[10px] font-semibold uppercase tracking-[0.06em] text-muted-foreground"
    >
        Checkpoints
    </div>
    <div class="px-3.5 pb-2 flex flex-col gap-1">
        {#if app.checkpoints.length === 0}
            <span class="text-[11.5px] text-muted-foreground italic">No checkpoints yet</span>
        {:else}
            {#each app.checkpoints as cp}
                <div
                    class="flex items-center gap-1.5 text-[11.5px] px-2 py-1 rounded-md bg-background border border-border"
                >
                    <span class="flex-1 font-medium truncate">{cp.name}</span>
                    <span class="font-mono text-[10.5px] text-muted-foreground">
                        {new Date(cp.timestamp).toLocaleTimeString("en-GB", {
                            hour: "2-digit",
                            minute: "2-digit",
                        })}
                    </span>
                    <Button
                        class="text-blue-500 h-auto! p-0! text-[10.5px]!"
                        onclick={() => app.restoreCheckpoint(cp)}
                        variant="link"
                        size="xs">Restore</Button
                    >
                </div>
            {/each}
        {/if}
    </div>
</div>

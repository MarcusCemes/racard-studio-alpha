<script lang="ts">
    import type { HTMLAttributes } from "svelte/elements";

    import { type WithElementRef, cn } from "$lib/utils.js";

    let {
        ref = $bindable(null),
        class: className,
        children,
        ...restProps
    }: WithElementRef<HTMLAttributes<HTMLDivElement>> = $props();
</script>

<div
    bind:this={ref}
    data-slot="card-footer"
    class={cn(
        "rounded-b-[min(var(--radius-4xl),24px)] px-5 group-data-[size=sm]/card:px-4 [.border-t]:pt-5 group-data-[size=sm]/card:[.border-t]:pt-4 flex items-center",
        className,
    )}
    {...restProps}
>
    {@render children?.()}
</div>

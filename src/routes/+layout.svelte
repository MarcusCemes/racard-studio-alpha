<script lang="ts">
    import "@fontsource-variable/geist/wght.css";

    import { Toaster } from "$lib/components/ui/sonner/index.js";
    import { TooltipProvider } from "$lib/components/ui/tooltip/index.js";
    import { TOOLTIP_DELAY } from "$lib/defs.js";
    import { useDev } from "$lib/hooks/useDev.svelte.js";
    import { useProgressEvents } from "$lib/hooks/useProgress.svelte.js";

    import "../app.css";

    let { children } = $props();

    useDev();
    useProgressEvents();
</script>

<Toaster richColors />

<TooltipProvider delayDuration={TOOLTIP_DELAY}>
    {@render children()}
</TooltipProvider>

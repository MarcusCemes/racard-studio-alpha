<script lang="ts">
    import type { SVGAttributes } from "svelte/elements";

    // Extend standard SVG attributes so the component accepts native props like class or style
    interface Props extends SVGAttributes<SVGSVGElement> {
        value?: number; // 0 to 100 progress value
        size?: number | string; // Defaults to 16px to match standard Lucide icons
        strokeWidth?: number; // Thickness of the progress stroke
        indeterminate?: boolean; // If true, acts as a spinning loading ring
    }

    let {
        value = 0,
        size = 16,
        strokeWidth = 2,
        class: className = "",
        indeterminate = false,
        ...restProps
    }: Props = $props();

    // SVG coordinates
    const r = 10; // Radius of the circle path
    const center = 12; // Center coordinates in a 24x24 viewBox
    const circumference = 2 * Math.PI * r; // ~62.83

    // Keep value bound between 0 and 100
    let normalizedValue = $derived(Math.min(100, Math.max(0, value)));

    // Calculate how much circle outline to hide
    let strokeDashoffset = $derived(circumference - (normalizedValue / 100) * circumference);

    // Format size for style injection (if a pure number is provided)
    let sizeStyle = $derived(typeof size === "number" ? `${size}px` : size);
</script>

<svg
    xmlns="http://www.w3.org/2000/svg"
    viewBox="0 0 24 24"
    fill="none"
    class="inline-block shrink-0 align-middle {className} {indeterminate ? 'animate-spin' : ''}"
    style="width: {sizeStyle}; height: {sizeStyle};"
    {...restProps}
>
    <!-- Background Track (low opacity) -->
    <circle
        cx={center}
        cy={center}
        {r}
        stroke="currentColor"
        stroke-width={strokeWidth}
        class="opacity-20"
    />

    <!-- Active Progress Indicator -->
    <circle
        cx={center}
        cy={center}
        {r}
        stroke="currentColor"
        stroke-width={strokeWidth}
        stroke-linecap="round"
        stroke-dasharray={circumference}
        stroke-dashoffset={indeterminate ? circumference * 0.75 : strokeDashoffset}
        class="transition-all duration-300 origin-center -rotate-90"
    />
</svg>

<script lang="ts">
    import { app } from "$lib/app.svelte.js";
    import { apiStatistics, apiValidate, buildProblemConfig } from "$lib/api.js";

    // Refresh statistics and conflicts whenever slots change
    $effect(() => {
        const controller = new AbortController();
        const slots = app.slots;

        Promise.all([
            apiStatistics(app, slots, app.weights),
            apiValidate(app, slots),
        ]).then(([statistics, conflicts]) => {
            if (!controller.signal.aborted) {
                app.statistics = statistics;
                app.conflicts = conflicts;
            }
        });

        return () => controller.abort();
    });
</script>

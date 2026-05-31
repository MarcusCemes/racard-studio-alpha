<script lang="ts">
    import { apiStatistics, apiValidate } from "$lib/api.js";
    import { app } from "$lib/app.svelte.js";

    let busy = $state(false);
    let error = $state(false);

    // Refresh statistics and conflicts whenever slots change
    $effect(() => {
        busy = true;

        let aborted = false;

        Promise.all([apiStatistics(app, app.slots, app.weights), apiValidate(app, app.slots)])
            .then(([statistics, conflicts]) => {
                if (aborted) {
                    return;
                }

                error = false;

                app.statistics = statistics;
                app.conflicts = conflicts;
            })
            .catch((err) => {
                error = true;
            })
            .finally(() => {
                if (!aborted) {
                    busy = false;
                }
            });

        return () => (aborted = true);
    });
</script>

<div
    class="size-2 rounded-full {error ? 'bg-red-600' : busy ? 'bg-amber-600' : 'bg-green-600'}"
></div>

import { selection } from "$lib/app.svelte.js";
import { N_DAYS, N_WEEKDAYS } from "$lib/defs";
import { useHotKeys } from "$lib/hooks/useHotkey.svelte.js";

export function useGridNavigation() {
    useHotKeys(["ArrowUp", "ArrowDown", "ArrowLeft", "ArrowRight"], (event) => {
        if (selection.day === undefined) {
            return;
        }

        event.preventDefault();

        switch (event.key) {
            case "ArrowUp":
                if (selection.day >= N_WEEKDAYS) {
                    selection.day -= N_WEEKDAYS;
                }

                break;

            case "ArrowDown":
                if (selection.day <= N_DAYS - N_WEEKDAYS) {
                    selection.day += N_WEEKDAYS;
                }

                break;

            case "ArrowLeft":
                if (selection.day > 0) {
                    selection.day -= 1;
                }

                break;

            case "ArrowRight":
                if (selection.day < N_DAYS - 1) {
                    selection.day += 1;
                }

                break;
        }
    });
}

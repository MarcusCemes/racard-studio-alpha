import { type Event, listen } from "@tauri-apps/api/event";

import { app } from "$lib/app.svelte.js";
import type { RefinerProgress, SolverProgress } from "$lib/schemas.js";

const SOLVER_PROGRESS_KEY = "solver-progress";
const REFINER_PROGRESS_KEY = "refiner-progress";

export function useProgressEvents() {
    $effect(() => {
        function solverHandler(event: Event<SolverProgress>) {
            app.solverProgress = event.payload;
        }

        function refinerHandler(event: Event<RefinerProgress>) {
            app.refinerProgress = event.payload;
        }

        const solverUnlisten = listen<SolverProgress>(SOLVER_PROGRESS_KEY, solverHandler);
        const refinerUnlisten = listen<RefinerProgress>(REFINER_PROGRESS_KEY, refinerHandler);

        return () => {
            solverUnlisten.then((f) => f());
            refinerUnlisten.then((f) => f());
        };
    });
}

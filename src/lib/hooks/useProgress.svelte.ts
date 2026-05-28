import { type Event, listen } from "@tauri-apps/api/event";

import { app } from "$lib/app.svelte.js";
import type { OperationEvent } from "$lib/schemas.js";

const OPERATION_EVENT_KEY = "operation-event";

export function useProgressEvents() {
    $effect(() => {
        function operationHandler(event: Event<OperationEvent>) {
            const payload = event.payload;
            const active = payload.status === "started" || payload.status === "running";

            app.activeOp = active ? payload.operation : null;
            app.operationStatus = payload.status;
            app.operationPhase = payload.phase ?? null;

            if (payload.progress.solver) app.solverProgress = payload.progress.solver;
            if (payload.progress.refiner) app.refinerProgress = payload.progress.refiner;
            if (payload.progress.orchestration) {
                app.orchestrationProgress = payload.progress.orchestration;
            }

            if (payload.status === "started") {
                app.solverProgress = undefined;
                app.refinerProgress = undefined;
                app.orchestrationProgress = undefined;
            }
        }

        const operationUnlisten = listen<OperationEvent>(OPERATION_EVENT_KEY, operationHandler);

        return () => {
            operationUnlisten.then((f) => f());
        };
    });
}

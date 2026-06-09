import { isModifierKeyPressed } from "$lib/misc";

export function useHotKey(
    key: string,
    callback: (event: KeyboardEvent) => void,
    allowModifier = false,
) {
    $effect(() => {
        const handler = (event: KeyboardEvent) => {
            // @ts-ignore
            if (event.target?.matches("input, textarea, select, [contenteditable]")) {
                return;
            }

            if (event.key === key && (allowModifier || !isModifierKeyPressed(event))) {
                callback(event);
            }
        };

        window.addEventListener("keydown", handler);
        return () => window.removeEventListener("keydown", handler);
    });
}

export function useHotKeys(
    keys: string[] | null,
    callback: (event: KeyboardEvent) => void,
    allowModifier = false,
) {
    $effect(() => {
        const handler = (event: KeyboardEvent) => {
            // @ts-ignore
            if (event.target?.matches("input, textarea, select, [contenteditable]")) {
                return;
            }

            if (
                (keys?.includes(event.key) ?? true) &&
                (allowModifier || !isModifierKeyPressed(event))
            ) {
                callback(event);
            }
        };

        window.addEventListener("keydown", handler);
        return () => window.removeEventListener("keydown", handler);
    });
}

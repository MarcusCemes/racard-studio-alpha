import { dev } from "$app/environment";
import { app } from "$lib/app.svelte";

export function useDev() {
    if (!dev) return;

    $effect(() => {
        const { problemParameters, solution } = sampleSave();

        app.slots = solution;
        app.people = problemParameters.people;
        app.startDate = problemParameters.start_date;
    });
}

function sampleSave() {
    return {
        problemParameters: {
            weekday_hours: [
                [13.5, 7.5],
                [14.5, 5.5],
                [14.5, 7.5],
                [14.5, 7.5],
                [14.5, 7.5],
                [20.0, 7.5],
                [14.5, 5.0],
            ],
            bank_holiday_default_hours: [
                [20.0, 9.0],
                [20.0, 5.0],
                [20.0, 5.0],
                [20.0, 5.0],
                [20.0, 5.0],
                [20.0, 5.0],
                [20.0, 5.0],
            ],
            bank_holidays: [
                {
                    date: "2024-09-05",
                    name: "Jeune Genevois",
                    enabled: true,
                    lead_hours: null,
                    support_hours: null,
                },
                {
                    date: "2024-12-25",
                    name: "Christmas",
                    enabled: true,
                    lead_hours: null,
                    support_hours: null,
                },
                {
                    date: "2024-12-31",
                    name: "Restoration Day",
                    enabled: true,
                    lead_hours: null,
                    support_hours: null,
                },
                {
                    date: "2025-01-01",
                    name: "New Year's Day",
                    enabled: true,
                    lead_hours: null,
                    support_hours: null,
                },
                {
                    date: "2025-04-18",
                    name: "Good Friday",
                    enabled: true,
                    lead_hours: null,
                    support_hours: null,
                },
                {
                    date: "2025-04-21",
                    name: "Easter Monday",
                    enabled: true,
                    lead_hours: null,
                    support_hours: null,
                },
                {
                    date: "2025-05-29",
                    name: "Ascension Day",
                    enabled: true,
                    lead_hours: null,
                    support_hours: null,
                },
                {
                    date: "2025-06-09",
                    name: "Whit Monday",
                    enabled: true,
                    lead_hours: null,
                    support_hours: null,
                },
            ],
            custom_overrides: [],
            people: [
                { name: "Franca Ferrari", holidays: [10, 18, 25, 33, 43, 44], rate: 75 },
                { name: "Sacha Marchal", holidays: [0, 11, 15, 28, 29], rate: 70 },
                { name: "Alexandra Favre", holidays: [2, 10, 21, 36, 43, 44], rate: 80 },
                { name: "Matteo Solcà", holidays: [9, 18, 27, 36, 37], rate: 80 },
                { name: "Sandrine Pilleul", holidays: [2, 9, 19, 28, 35], rate: 60 },
                { name: "Franck Commare", holidays: [6, 20, 32, 37, 38], rate: 60 },
                { name: "Ariane Hubleur", holidays: [3, 19, 34, 35, 41], rate: 55 },
                { name: "Alexandre Orzan", holidays: [4, 20, 25, 34, 41], rate: 40 },
            ],
            start_date: "2024-08-19",
        },
        refineParameters: {
            cooling_rate: 0.995,
            initial_temperature: 1,
            num_iterations: 10000,
            polish: true,
            searches: 1000,
        },
        solution: [
            38, 64, 83, 103, 4, 50, 35, 84, 50, 117, 36, 99, 16, 1, 245, 23, 3, 22, 63, 117, 87, 48,
            18, 117, 49, 5, 66, 36, 16, 35, 65, 98, 84, 3, 48, 38, 64, 83, 7, 50, 97, 22, 48, 20,
            98, 3, 113, 36, 66, 3, 37, 71, 18, 101, 48, 3, 65, 55, 36, 3, 97, 82, 37, 6, 37, 97, 7,
            82, 113, 23, 86, 23, 84, 19, 71, 99, 54, 52, 80, 98, 53, 39, 4, 64, 35, 70, 2, 22, 112,
            53, 83, 2, 54, 80, 55, 70, 33, 18, 48, 18, 86, 3, 33, 116, 71, 37, 48, 116, 82, 67, 6,
            96, 19, 71, 2, 97, 80, 35, 50, 1, 39, 48, 18, 99, 69, 84, 38, 84, 97, 87, 36, 23, 113,
            5, 35, 23, 80, 23, 50, 35, 16, 38, 52, 18, 67, 6, 96, 49, 87, 96, 53, 6, 20, 65, 3, 33,
            103, 66, 16, 83, 53, 66, 53, 100, 19, 117, 32, 2, 52, 7, 36, 80, 50, 97, 22, 37, 99, 81,
            36, 22, 52, 67, 18, 3, 70, 49, 80, 114, 39, 70, 1, 114, 20, 38, 5, 80, 103, 37, 3, 86,
            7, 35, 50, 86, 32, 55, 2, 53, 70, 100, 3, 36, 87, 50, 100, 16, 1, 83, 66, 97, 5, 33, 55,
            115, 22, 52, 2, 99, 112, 66, 36, 49, 70, 39, 100, 50, 81, 21, 35, 84, 1, 69, 16, 35, 50,
            87, 35, 1, 53, 18, 7, 112, 20, 6, 87, 16, 117, 100, 70, 1, 103, 36, 6, 71, 33, 18, 4,
            39, 19, 66, 97, 48, 3, 33, 71, 48, 18, 3, 86, 101, 49, 64, 99, 2, 87, 18, 33, 67, 37,
            16, 82, 20, 3, 48, 36, 16, 55, 2, 49, 101, 86, 20, 55, 101, 49, 86, 71, 116, 49, 246,
            84, 246, 116, 19, 49, 37, 55, 64, 21, 3, 38, 98, 16, 55, 66, 19, 38, 80, 5, 35, 23, 6,
            50, 69, 245, 255,
        ],
        solveParameters: {
            friday: { number_permutations: 1000, max_resolve_attempts: 50 },
            weekday: { number_permutations: 20, max_resolve_attempts: 50 },
            weekend: { number_permutations: 500, max_resolve_attempts: 50 },
        },
    };
}

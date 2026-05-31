import type { Holiday } from "./schemas.js";

export enum Role {
    Lead = "lead",
    Support = "support",
}

export const APP_NAME = "Racard Studio";

export const MIN_PEOPLE = 2;
export const MAX_PEOPLE = 15;
export const N_DAYS = 336; // N_WEEKS * N_WEEKDAYS
export const N_WEEKDAYS = 7;
export const N_WEEKS = 48;
export const N_ROLES = 2;

export const NULL_ID = 0xf;
export const NULL_SLOT = 0xff;

export const WEEKDAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];
export const WEEKDAYS_FR = ["Lun", "Mar", "Mer", "Jeu", "Ven", "Sam", "Dim"];

export const WEEKDAYS_FULL_FR = [
    "Lundi",
    "Mardi",
    "Mercredi",
    "Jeudi",
    "Vendredi",
    "Samedi",
    "Dimanche",
];

export const HOLIDAY_NAMES: Record<Holiday, string> = {
    NearYear: "New Year's Day",
    EasterFriday: "Good Friday",
    EasterMonday: "Easter Monday",
    AscensionThursday: "Ascension Day",
    WhitMonday: "Whit Monday",
    NationalDay: "National Day",
    JeuneGenevois: "Jeune Genevois",
    Christmas: "Christmas Day",
    PublicRestoration: "Restoration Day",
};

export const DATE_LOCALE = "en-GB";
export const TOOLTIP_DELAY = 100;

export const DEFAULT_WEEKDAY_HOURS: [number, number][] = [
    [13.5, 7.5], // Mon
    [14.5, 5.5], // Tue
    [14.5, 7.5], // Wed
    [14.5, 7.5], // Thu
    [14.5, 7.5], // Fri
    [20.0, 7.5], // Sat
    [14.5, 5.0], // Sun
];

export const DEFAULT_BANK_HOLIDAY_HOURS: [number, number][] = [
    [20.0, 9.0], // Mon
    [20.0, 5.0], // Tue
    [20.0, 5.0], // Wed
    [20.0, 5.0], // Thu
    [20.0, 5.0], // Fri
    [20.0, 5.0], // Sat
    [20.0, 5.0], // Sun
];

export const PERSON_COLORS = [
    ["bg-blue-500/90 border-transparent text-white", "bg-blue-500"],
    ["bg-green-500/90 border-transparent text-white", "bg-green-500"],
    ["bg-red-500/90 border-transparent text-white", "bg-red-500"],
    ["bg-yellow-500/90 border-transparent text-white", "bg-yellow-500"],
    ["bg-purple-500/90 border-transparent text-white", "bg-purple-500"],
    ["bg-pink-500/90 border-transparent text-white", "bg-pink-500"],
    ["bg-indigo-500/90 border-transparent text-white", "bg-indigo-500"],
    ["bg-teal-500/90 border-transparent text-white", "bg-teal-500"],
    ["bg-blue-400/90 border-transparent text-white", "bg-blue-400"],
    ["bg-green-400/90 border-transparent text-white", "bg-green-400"],
    ["bg-red-400/90 border-transparent text-white", "bg-red-400"],
    ["bg-yellow-400/90 border-transparent text-white", "bg-yellow-400"],
    ["bg-purple-400/90 border-transparent text-white", "bg-purple-400"],
    ["bg-pink-400/90 border-transparent text-white", "bg-pink-400"],
    ["bg-indigo-400/90 border-transparent text-white", "bg-indigo-400"],
];

export const PERSON_TEXT_COLORS = [
    "text-blue-500",
    "text-green-500",
    "text-red-500",
    "text-yellow-500",
    "text-purple-500",
    "text-pink-500",
    "text-indigo-500",
    "text-teal-500",
    "text-blue-400",
    "text-green-400",
    "text-red-400",
    "text-yellow-400",
    "text-purple-400",
    "text-pink-400",
    "text-indigo-400",
];

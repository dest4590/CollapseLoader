import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { emit as emitAppEvent } from "@tauri-apps/api/event";
import { settingsService } from "@services/settings/settingsService";

const SCHEDULE_STORAGE_KEY = "themeSchedule";

export interface ThemeSchedule {
    enabled: boolean;
    lightStart: string;
    lightEnd: string;
}

const defaultSchedule: ThemeSchedule = {
    enabled: false,
    lightStart: "09:00",
    lightEnd: "18:00",
};

const schedule = ref<ThemeSchedule>({ ...defaultSchedule });
let intervalId: ReturnType<typeof setInterval> | null = null;

const loadSchedule = (): void => {
    try {
        const raw = localStorage.getItem(SCHEDULE_STORAGE_KEY);
        if (raw) {
            const parsed = JSON.parse(raw) as Partial<ThemeSchedule>;
            schedule.value = {
                enabled: parsed.enabled ?? defaultSchedule.enabled,
                lightStart: parsed.lightStart ?? defaultSchedule.lightStart,
                lightEnd: parsed.lightEnd ?? defaultSchedule.lightEnd,
            };
        }
    } catch {
        schedule.value = { ...defaultSchedule };
    }
};

const saveSchedule = (): void => {
    localStorage.setItem(SCHEDULE_STORAGE_KEY, JSON.stringify(schedule.value));
};

const toMinutes = (time: string): number => {
    const [h, m] = time.split(":").map(Number);
    return h * 60 + m;
};

const getScheduledTheme = (): "light" | "dark" => {
    const now = new Date();
    const currentMinutes = now.getHours() * 60 + now.getMinutes();
    const startMinutes = toMinutes(schedule.value.lightStart);
    const endMinutes = toMinutes(schedule.value.lightEnd);

    let isLight: boolean;

    if (startMinutes < endMinutes) {
        isLight =
            currentMinutes >= startMinutes && currentMinutes < endMinutes;
    } else {
        isLight =
            currentMinutes >= startMinutes || currentMinutes < endMinutes;
    }

    return isLight ? "light" : "dark";
};

const applyScheduledTheme = async (): Promise<void> => {
    if (!schedule.value.enabled) return;

    const theme = getScheduledTheme();
    const current =
        document.documentElement.getAttribute("data-theme") || "dark";

    if (current === theme) return;

    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem("theme", theme);

    try {
        await invoke("set_window_theme", { theme });
        await settingsService.editSetting("theme", theme, false);
        await emitAppEvent("theme-mode-update", theme);
    } catch {}
};

const startScheduler = (): void => {
    loadSchedule();
    void applyScheduledTheme();
    if (intervalId !== null) clearInterval(intervalId);
    intervalId = setInterval(() => {
        void applyScheduledTheme();
    }, 60_000);
};

const stopScheduler = (): void => {
    if (intervalId !== null) {
        clearInterval(intervalId);
        intervalId = null;
    }
};

const updateSchedule = (patch: Partial<ThemeSchedule>): void => {
    schedule.value = { ...schedule.value, ...patch };
    saveSchedule();

    if (schedule.value.enabled) {
        void applyScheduledTheme();
    }
};

const previewTheme = computed<"light" | "dark">(() =>
    getScheduledTheme()
);

export const themeScheduler = {
    schedule,
    previewTheme,
    loadSchedule,
    saveSchedule,
    startScheduler,
    stopScheduler,
    updateSchedule,
    applyScheduledTheme,
};

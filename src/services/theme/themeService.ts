import { reactive, watchEffect } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { listen, emit } from "@tauri-apps/api/event";
import { achievementService } from "@features/social/achievementService";
import { STORAGE_KEYS } from "@shared/utils/storageKeys";
import {
    applyThemeSettingsToDocument,
    clearThemeCustomization,
    cloneThemeSettings,
    createPresetExportPayload,
    defaultThemeSettings,
    extractThemeSettingsFromPreset,
    loadThemeSettings,
    saveThemeSettings,
    THEME_SETTINGS_STORAGE_KEY,
    themeCssVarList,
    type ThemeSettings,
} from "./themeSettings";

const THEME_STORAGE_KEY = STORAGE_KEYS.THEME;

const presetSettings = reactive<ThemeSettings>({ ...defaultThemeSettings });

const persistPresetSettings = () => {
    saveThemeSettings(THEME_SETTINGS_STORAGE_KEY, presetSettings);
};

const applyThemeMode = (theme: string) => {
    document.documentElement.setAttribute("data-theme", theme);
    localStorage.setItem(THEME_STORAGE_KEY, theme);
};

const applyNativeWindowTheme = async (theme: string) => {
    try {
        await invoke("set_window_theme", { theme });
    } catch (error) {
        console.error("Failed to apply native window theme:", error);
    }
};

export const cssVarList = themeCssVarList;

const applyPreset = () => {
    applyThemeSettingsToDocument(presetSettings);
};

let isApplyingExternalUpdate = false;

const loadSettings = () => {
    Object.assign(presetSettings, loadThemeSettings(THEME_SETTINGS_STORAGE_KEY));
    applyPreset();
};

const _updateInternalState = (newSettings: Partial<ThemeSettings>) => {
    Object.assign(presetSettings, newSettings);
};

listen<ThemeSettings>("theme-update", (event) => {
    console.log("Received theme update from another window:", event.payload);
    isApplyingExternalUpdate = true;
    _updateInternalState(event.payload);
    persistPresetSettings();
    applyPreset();
    setTimeout(() => {
        isApplyingExternalUpdate = false;
    }, 50);
});

listen<string>("theme-mode-update", (event) => {
    if (!event.payload) return;
    applyThemeMode(event.payload);
    void applyNativeWindowTheme(event.payload);
});
const saveCardSettings = () => {
    if (isApplyingExternalUpdate) return;

    try {
        persistPresetSettings();
        emit("theme-update", cloneThemeSettings(presetSettings));
        console.log("Saved preset settings to localStorage:", presetSettings);

        void achievementService.unlockAchievement("PRESET_MAX");
    } catch (error) {
        console.error("Failed to save preset settings:", error);
    }
};

const updatePresetSettings = (settings: Partial<ThemeSettings>) => {
    Object.assign(presetSettings, settings);
    saveCardSettings();
    applyPreset();
};

const resetPresetSettings = () => {
    Object.assign(presetSettings, defaultThemeSettings);
    saveCardSettings();
    applyPreset();
};

const clearCustomTheme = () => {
    clearThemeCustomization();
};

const emergencyReset = () => {
    Object.assign(presetSettings, defaultThemeSettings);
    presetSettings.enableCustomCSS = false;
    saveCardSettings();
    applyPreset();

    clearCustomTheme();
};

const exportPreset = (): string => {
    const preset = createPresetExportPayload(presetSettings);
    return JSON.stringify(preset, null, 2);
};

const importPreset = (presetJSON: string): void => {
    try {
        const parsed = JSON.parse(presetJSON) as Record<string, unknown>;
        updatePresetSettings(extractThemeSettingsFromPreset(parsed));
    } catch (e) {
        console.error("Failed to import theme preset:", e);
        throw e;
    }
};

loadSettings();

watchEffect(() => {
    applyPreset();
});

export const themeService = {
    presetSettings,
    updatePresetSettings,
    resetPresetSettings,
    clearCustomTheme,
    emergencyReset,
    loadSettings,
    saveCardSettings,
    exportPreset,
    importPreset,
};

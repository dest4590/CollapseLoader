import { reactive, watchEffect } from "vue";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen, emit } from "@tauri-apps/api/event";
import { achievementService } from "@features/social/achievementService";
import { STORAGE_KEYS } from "@shared/utils/storageKeys";

const PRESET_SETTINGS_STORAGE_KEY = "presetSettings";
const THEME_STORAGE_KEY = STORAGE_KEYS.THEME;

interface ThemeSettings {
    customCSS: string;
    enableCustomCSS: boolean;
    primary: string | null;

    base100?: string | null;
    base200?: string | null;
    base300?: string | null;
    baseContent?: string | null;

    primaryContent?: string | null;
    secondary?: string | null;
    secondaryContent?: string | null;
    accent?: string | null;
    accentContent?: string | null;
    neutral?: string | null;
    neutralContent?: string | null;
    info?: string | null;
    infoContent?: string | null;
    success?: string | null;
    successContent?: string | null;
    warning?: string | null;
    warningContent?: string | null;
    error?: string | null;
    errorContent?: string | null;

    backgroundImage?: string | null;
    backgroundBlur?: number | null;
    backgroundOpacity?: number | null;
}

const defaultSettings: ThemeSettings = {
    customCSS: "",
    enableCustomCSS: false,
    primary: null,

    base100: null,
    base200: null,
    base300: null,
    baseContent: null,

    primaryContent: null,
    secondary: null,
    secondaryContent: null,
    accent: null,
    accentContent: null,
    neutral: null,
    neutralContent: null,
    info: null,
    infoContent: null,
    success: null,
    successContent: null,
    warning: null,
    warningContent: null,
    error: null,
    errorContent: null,

    backgroundImage: null,
    backgroundBlur: 0,
    backgroundOpacity: 100,
};

const presetSettings = reactive<ThemeSettings>({ ...defaultSettings });

const persistPresetSettings = () => {
    localStorage.setItem(
        PRESET_SETTINGS_STORAGE_KEY,
        JSON.stringify(presetSettings)
    );
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

export const cssVarList = [
    "--color-primary",
    "--color-base-100",
    "--color-base-200",
    "--color-base-300",
    "--color-base-content",
    "--color-primary-content",
    "--color-secondary",
    "--color-secondary-content",
    "--color-accent",
    "--color-accent-content",
    "--color-neutral",
    "--color-neutral-content",
    "--color-info",
    "--color-info-content",
    "--color-success",
    "--color-success-content",
    "--color-warning",
    "--color-warning-content",
    "--color-error",
    "--color-error-content",
    "--background-image",
    "--background-blur",
    "--background-opacity",
];

const applyPreset = () => {
    const root = document.documentElement;
    const settings = presetSettings as Record<string, any>;

    const varMap: Record<string, string> = {
        primary: "--color-primary",
        base100: "--color-base-100",
        base200: "--color-base-200",
        base300: "--color-base-300",
        baseContent: "--color-base-content",
        primaryContent: "--color-primary-content",
        secondary: "--color-secondary",
        secondaryContent: "--color-secondary-content",
        accent: "--color-accent",
        accentContent: "--color-accent-content",
        neutral: "--color-neutral",
        neutralContent: "--color-neutral-content",
        info: "--color-info",
        infoContent: "--color-info-content",
        success: "--color-success",
        successContent: "--color-success-content",
        warning: "--color-warning",
        warningContent: "--color-warning-content",
        error: "--color-error",
        errorContent: "--color-error-content",
        backgroundImage: "--background-image",
        backgroundBlur: "--background-blur",
        backgroundOpacity: "--background-opacity",
    };

    Object.entries(varMap).forEach(([key, cssVar]) => {
        const value = settings[key];
        const isEmpty =
            value === undefined ||
            value === null ||
            (typeof value === "string" && value.trim().length === 0);

        if (isEmpty) {
            root.style.removeProperty(cssVar);
            return;
        }

        let cssValue = String(value);
        if (key === "backgroundBlur") cssValue = `${value}px`;
        if (key === "backgroundImage") {
            const trimmed = value.trim();
            const isUrl =
                trimmed.startsWith("http://") ||
                trimmed.startsWith("https://") ||
                trimmed.startsWith("data:") ||
                trimmed.startsWith("blob:");

            if (isUrl) {
                cssValue = `url("${trimmed}")`;
            } else if (trimmed.length > 0) {
                cssValue = `url("${convertFileSrc(trimmed)}")`;
            } else {
                root.style.removeProperty(cssVar);
                return;
            }
        }
        root.style.setProperty(cssVar, cssValue);
    });

    const hasBg =
        presetSettings.backgroundImage?.trim() &&
        (presetSettings.backgroundOpacity ?? 100) > 0;

    if (hasBg) {
        root.setAttribute("data-has-background", "true");
    } else {
        root.removeAttribute("data-has-background");
    }

    const styleEl =
        document.getElementById("custom-theme-styles") ||
        (() => {
            const el = document.createElement("style");
            el.id = "custom-theme-styles";
            document.head.appendChild(el);
            return el;
        })();

    styleEl.textContent =
        presetSettings.enableCustomCSS && presetSettings.customCSS
            ? presetSettings.customCSS
            : "";
};

let isApplyingExternalUpdate = false;

const loadSettings = () => {
    try {
        const savedSettings = localStorage.getItem(PRESET_SETTINGS_STORAGE_KEY);
        if (savedSettings) {
            const parsedSettings = JSON.parse(savedSettings);
            Object.assign(presetSettings, {
                ...defaultSettings,
                ...parsedSettings,
            });
        }
    } catch (error) {
        console.error("Failed to load settings:", error);
        Object.assign(presetSettings, defaultSettings);
    }

    applyPreset();
};

const _updateInternalState = (newSettings: Partial<ThemeSettings>) => {
    Object.entries(newSettings).forEach(([key, value]) => {
        (presetSettings as any)[key] = value;
    });
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
        emit("theme-update", JSON.parse(JSON.stringify(presetSettings)));
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
    Object.assign(presetSettings, defaultSettings);
    saveCardSettings();
    applyPreset();
};

const clearCustomTheme = () => {
    const styleEl = document.getElementById("custom-theme-styles");
    if (styleEl && styleEl.parentNode) {
        styleEl.parentNode.removeChild(styleEl);
    }

    const root = document.documentElement;
    cssVarList.forEach((v) => root.style.removeProperty(v));
};

const emergencyReset = () => {
    Object.assign(presetSettings, defaultSettings);
    presetSettings.enableCustomCSS = false;
    saveCardSettings();
    applyPreset();

    clearCustomTheme();
};

const exportPreset = (): string => {
    const preset = {
        customCSS: presetSettings.customCSS,
        enableCustomCSS: presetSettings.enableCustomCSS,
        primaryColorOverride: presetSettings.primary,

        base100: presetSettings.base100,
        base200: presetSettings.base200,
        base300: presetSettings.base300,
        baseContent: presetSettings.baseContent,

        primaryContent: presetSettings.primaryContent,
        secondary: presetSettings.secondary,
        secondaryContent: presetSettings.secondaryContent,
        accent: presetSettings.accent,
        accentContent: presetSettings.accentContent,
        neutral: presetSettings.neutral,
        neutralContent: presetSettings.neutralContent,
        info: presetSettings.info,
        infoContent: presetSettings.infoContent,
        success: presetSettings.success,
        successContent: presetSettings.successContent,
        warning: presetSettings.warning,
        warningContent: presetSettings.warningContent,
        error: presetSettings.error,
        errorContent: presetSettings.errorContent,

        backgroundImage: presetSettings.backgroundImage,
        backgroundBlur: presetSettings.backgroundBlur,
        backgroundOpacity: presetSettings.backgroundOpacity,
    };
    return JSON.stringify(preset, null, 2);
};

const importPreset = (presetJSON: string): void => {
    try {
        const parsed = JSON.parse(presetJSON);
        const newSettings: Partial<ThemeSettings> = {};

        const stringFields = [
            "customCSS",
            "primary",
            "base100",
            "base200",
            "base300",
            "baseContent",
            "primaryContent",
            "secondary",
            "secondaryContent",
            "accent",
            "accentContent",
            "neutral",
            "neutralContent",
            "info",
            "infoContent",
            "success",
            "successContent",
            "warning",
            "warningContent",
            "error",
            "errorContent",
            "backgroundImage",
        ];

        const numberFields = ["backgroundBlur", "backgroundOpacity"];
        const booleanFields = ["enableCustomCSS"];

        // Map legacy field names if necessary
        if (parsed.primaryColorOverride !== undefined) {
            parsed.primary = parsed.primaryColorOverride;
        }

        stringFields.forEach((field) => {
            if (typeof parsed[field] === "string" || parsed[field] === null) {
                (newSettings as any)[field] = parsed[field];
            }
        });

        numberFields.forEach((field) => {
            if (typeof parsed[field] === "number" || parsed[field] === null) {
                (newSettings as any)[field] = parsed[field];
            }
        });

        booleanFields.forEach((field) => {
            if (typeof parsed[field] === "boolean") {
                (newSettings as any)[field] = parsed[field];
            }
        });

        updatePresetSettings(newSettings);
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

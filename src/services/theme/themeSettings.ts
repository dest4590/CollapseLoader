import { convertFileSrc } from "@tauri-apps/api/core";
import type { CreatePresetInput, ThemePreset } from "@features/presets/types";

export interface ThemeSettings {
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
    spotlightBlur?: number | null;
    historyBlur?: number | null;
}

export const THEME_SETTINGS_STORAGE_KEY = "presetSettings";

export const defaultThemeSettings: ThemeSettings = {
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
    spotlightBlur: 24,
    historyBlur: 20,
};

const themeCssVariables: Array<[keyof ThemeSettings, string]> = [
    ["primary", "--color-primary"],
    ["base100", "--color-base-100"],
    ["base200", "--color-base-200"],
    ["base300", "--color-base-300"],
    ["baseContent", "--color-base-content"],
    ["primaryContent", "--color-primary-content"],
    ["secondary", "--color-secondary"],
    ["secondaryContent", "--color-secondary-content"],
    ["accent", "--color-accent"],
    ["accentContent", "--color-accent-content"],
    ["neutral", "--color-neutral"],
    ["neutralContent", "--color-neutral-content"],
    ["info", "--color-info"],
    ["infoContent", "--color-info-content"],
    ["success", "--color-success"],
    ["successContent", "--color-success-content"],
    ["warning", "--color-warning"],
    ["warningContent", "--color-warning-content"],
    ["error", "--color-error"],
    ["errorContent", "--color-error-content"],
    ["backgroundImage", "--background-image"],
    ["backgroundBlur", "--background-blur"],
    ["backgroundOpacity", "--background-opacity"],
    ["spotlightBlur", "--spotlight-blur"],
    ["historyBlur", "--history-blur"],
];

const themePresetStringFields = [
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
] as const;

const themePresetNumberFields = [
    "backgroundBlur",
    "backgroundOpacity",
] as const;
const themePresetBooleanFields = ["enableCustomCSS"] as const;

const remoteImagePrefixes = ["http://", "https://", "data:", "blob:"];

const isEmptyThemeValue = (value: unknown): boolean =>
    value === undefined ||
    value === null ||
    (typeof value === "string" && value.trim().length === 0);

const isRemoteImageSource = (value: string): boolean =>
    remoteImagePrefixes.some((prefix) => value.startsWith(prefix));

const createCustomThemeStyleElement = (): HTMLStyleElement => {
    const styleElement = document.createElement("style");
    styleElement.id = "custom-theme-styles";
    document.head.appendChild(styleElement);
    return styleElement;
};

const getThemeCssValue = (
    key: keyof ThemeSettings,
    value: unknown
): string | null => {
    if (isEmptyThemeValue(value)) {
        return null;
    }

    if (key === "backgroundImage") {
        const trimmedValue = String(value).trim();

        if (trimmedValue.length === 0) {
            return null;
        }

        if (isRemoteImageSource(trimmedValue)) {
            return `url("${trimmedValue}")`;
        }

        return `url("${convertFileSrc(trimmedValue)}")`;
    }

    if (
        key === "backgroundBlur" ||
        key === "backgroundOpacity" ||
        key === "spotlightBlur" ||
        key === "historyBlur"
    ) {
        return key === "backgroundOpacity" ? String(value) : `${value}px`;
    }

    return String(value);
};

const readThemeSettingFromStorage = (storageKey: string): ThemeSettings => {
    try {
        const savedSettings = localStorage.getItem(storageKey);

        if (!savedSettings) {
            return { ...defaultThemeSettings };
        }

        const parsedSettings = JSON.parse(
            savedSettings
        ) as Partial<ThemeSettings>;

        return {
            ...defaultThemeSettings,
            ...parsedSettings,
        };
    } catch {
        return { ...defaultThemeSettings };
    }
};

export const themeCssVarList = themeCssVariables.map(([, cssVar]) => cssVar);

export const cloneThemeSettings = (settings: ThemeSettings): ThemeSettings =>
    JSON.parse(JSON.stringify(settings)) as ThemeSettings;

export const loadThemeSettings = (storageKey: string): ThemeSettings =>
    readThemeSettingFromStorage(storageKey);

export const saveThemeSettings = (
    storageKey: string,
    settings: ThemeSettings
): void => {
    localStorage.setItem(storageKey, JSON.stringify(settings));
};

export const applyThemeSettingsToDocument = (settings: ThemeSettings): void => {
    const root = document.documentElement;

    themeCssVariables.forEach(([key, cssVar]) => {
        const cssValue = getThemeCssValue(key, settings[key]);

        if (cssValue === null) {
            root.style.removeProperty(cssVar);
            return;
        }

        root.style.setProperty(cssVar, cssValue);
    });

    const hasBackgroundImage =
        settings.backgroundImage?.trim() &&
        (settings.backgroundOpacity ?? 100) > 0;

    if (hasBackgroundImage) {
        root.setAttribute("data-has-background", "true");
    } else {
        root.removeAttribute("data-has-background");
    }

    const styleElement =
        document.getElementById("custom-theme-styles") ||
        createCustomThemeStyleElement();

    styleElement.textContent =
        settings.enableCustomCSS && settings.customCSS
            ? settings.customCSS
            : "";
};

export const clearThemeCustomization = (): void => {
    const styleElement = document.getElementById("custom-theme-styles");

    if (styleElement?.parentNode) {
        styleElement.parentNode.removeChild(styleElement);
    }

    const root = document.documentElement;
    themeCssVarList.forEach((cssVar) => root.style.removeProperty(cssVar));
};

export const createPresetInputFromThemeSettings = (
    settings: ThemeSettings
): Omit<CreatePresetInput, "name" | "description"> => ({
    customCSS: settings.customCSS,
    enableCustomCSS: settings.enableCustomCSS,

    base100: settings.base100 || undefined,
    base200: settings.base200 || undefined,
    base300: settings.base300 || undefined,
    baseContent: settings.baseContent || undefined,

    primary: settings.primary || undefined,
    primaryContent: settings.primaryContent || undefined,
    secondary: settings.secondary || undefined,
    secondaryContent: settings.secondaryContent || undefined,
    accent: settings.accent || undefined,
    accentContent: settings.accentContent || undefined,
    neutral: settings.neutral || undefined,
    neutralContent: settings.neutralContent || undefined,
    info: settings.info || undefined,
    infoContent: settings.infoContent || undefined,
    success: settings.success || undefined,
    successContent: settings.successContent || undefined,
    warning: settings.warning || undefined,
    warningContent: settings.warningContent || undefined,
    error: settings.error || undefined,
    errorContent: settings.errorContent || undefined,
    backgroundImage: settings.backgroundImage || undefined,
    backgroundBlur: settings.backgroundBlur || undefined,
    backgroundOpacity: settings.backgroundOpacity || undefined,
});

export const createPresetExportPayload = (
    settings: ThemeSettings
): Record<string, string | number | boolean | null | undefined> => ({
    customCSS: settings.customCSS,
    enableCustomCSS: settings.enableCustomCSS,
    primaryColorOverride: settings.primary,

    base100: settings.base100,
    base200: settings.base200,
    base300: settings.base300,
    baseContent: settings.baseContent,

    primaryContent: settings.primaryContent,
    secondary: settings.secondary,
    secondaryContent: settings.secondaryContent,
    accent: settings.accent,
    accentContent: settings.accentContent,
    neutral: settings.neutral,
    neutralContent: settings.neutralContent,
    info: settings.info,
    infoContent: settings.infoContent,
    success: settings.success,
    successContent: settings.successContent,
    warning: settings.warning,
    warningContent: settings.warningContent,
    error: settings.error,
    errorContent: settings.errorContent,

    backgroundImage: settings.backgroundImage,
    backgroundBlur: settings.backgroundBlur,
    backgroundOpacity: settings.backgroundOpacity,
});

export const extractThemeSettingsFromPreset = (
    presetData: ThemePreset | Record<string, unknown>
): Partial<ThemeSettings> => {
    const extractedSettings: Partial<ThemeSettings> = {};
    const preset = presetData as Record<string, unknown>;

    if (preset.primaryColorOverride !== undefined) {
        preset.primary = preset.primaryColorOverride;
    }

    themePresetStringFields.forEach((field) => {
        if (typeof preset[field] === "string" || preset[field] === null) {
            (extractedSettings as Record<string, string | null | undefined>)[
                field
            ] = preset[field] as string | null;
        }
    });

    themePresetNumberFields.forEach((field) => {
        if (typeof preset[field] === "number" || preset[field] === null) {
            (extractedSettings as Record<string, number | null | undefined>)[
                field
            ] = preset[field] as number | null;
        }
    });

    themePresetBooleanFields.forEach((field) => {
        if (typeof preset[field] === "boolean") {
            (extractedSettings as Record<string, boolean | undefined>)[field] =
                preset[field] as boolean;
        }
    });

    return extractedSettings;
};

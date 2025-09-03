import { reactive, watchEffect } from 'vue';

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
}

const defaultSettings: ThemeSettings = {
    customCSS: '',
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
};

const presetSettings = reactive<ThemeSettings>({ ...defaultSettings });

export const cssVarList = [
    '--color-primary',
    '--color-base-100', '--color-base-200', '--color-base-300', '--color-base-content',
    '--color-primary-content', '--color-secondary', '--color-secondary-content', '--color-accent', '--color-accent-content',
    '--color-neutral', '--color-neutral-content', '--color-info', '--color-info-content', '--color-success', '--color-success-content',
    '--color-warning', '--color-warning-content', '--color-error', '--color-error-content'
];

const applyPreset = () => {
    const root = document.documentElement;

    if (presetSettings.primary && presetSettings.primary.trim().length > 0) {
        root.style.setProperty('--color-primary', presetSettings.primary);
    } else {
        root.style.removeProperty('--color-primary');
    }

    const setOrRemove = (varName: string, value?: string | null) => {
        if (value && value.trim().length > 0) {
            root.style.setProperty(varName, value);
        } else {
            root.style.removeProperty(varName);
        }
    };

    setOrRemove('--color-base-100', presetSettings.base100 ?? null);
    setOrRemove('--color-base-200', presetSettings.base200 ?? null);
    setOrRemove('--color-base-300', presetSettings.base300 ?? null);
    setOrRemove('--color-base-content', presetSettings.baseContent ?? null);

    setOrRemove('--color-primary-content', presetSettings.primaryContent ?? null);
    setOrRemove('--color-secondary', presetSettings.secondary ?? null);
    setOrRemove('--color-secondary-content', presetSettings.secondaryContent ?? null);
    setOrRemove('--color-accent', presetSettings.accent ?? null);
    setOrRemove('--color-accent-content', presetSettings.accentContent ?? null);

    setOrRemove('--color-neutral', presetSettings.neutral ?? null);
    setOrRemove('--color-neutral-content', presetSettings.neutralContent ?? null);

    setOrRemove('--color-info', presetSettings.info ?? null);
    setOrRemove('--color-info-content', presetSettings.infoContent ?? null);

    setOrRemove('--color-success', presetSettings.success ?? null);
    setOrRemove('--color-success-content', presetSettings.successContent ?? null);

    setOrRemove('--color-warning', presetSettings.warning ?? null);
    setOrRemove('--color-warning-content', presetSettings.warningContent ?? null);

    setOrRemove('--color-error', presetSettings.error ?? null);
    setOrRemove('--color-error-content', presetSettings.errorContent ?? null);

    let styleEl = document.getElementById('custom-theme-styles');
    if (!styleEl) {
        styleEl = document.createElement('style');
        styleEl.id = 'custom-theme-styles';
        document.head.appendChild(styleEl);
    }

    if (presetSettings.enableCustomCSS) {
        let customStyles = '';

        if (presetSettings.customCSS) {
            customStyles += presetSettings.customCSS;
        }

        styleEl.textContent = customStyles;
    } else {
        styleEl.textContent = '';
    }
};

const loadSettings = () => {
    try {
        const savedSettings = localStorage.getItem('presetSettings');
        if (savedSettings) {
            const parsedSettings = JSON.parse(savedSettings);
            Object.assign(presetSettings, {
                ...defaultSettings,
                ...parsedSettings
            });
        }
    } catch (error) {
        console.error('Failed to load settings:', error);
        Object.assign(presetSettings, defaultSettings);
    }

    applyPreset();
};


const saveCardSettings = () => {
    try {
        localStorage.setItem('presetSettings', JSON.stringify(presetSettings));
        console.log('Saved preset settings to localStorage:', presetSettings);
    } catch (error) {
        console.error('Failed to save preset settings:', error);
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
    const styleEl = document.getElementById('custom-theme-styles');
    if (styleEl && styleEl.parentNode) {
        styleEl.parentNode.removeChild(styleEl);
    }

    const root = document.documentElement;
    cssVarList.forEach(v => root.style.removeProperty(v));
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
        errorContent: presetSettings.errorContent
    };
    return JSON.stringify(preset, null, 2);
};

const importPreset = (presetJSON: string): void => {
    try {
        const parsed = JSON.parse(presetJSON);
        updatePresetSettings({
            customCSS: typeof parsed.customCSS === 'string' ? parsed.customCSS : presetSettings.customCSS,
            enableCustomCSS: typeof parsed.enableCustomCSS === 'boolean' ? parsed.enableCustomCSS : presetSettings.enableCustomCSS,
            primary: typeof parsed.primaryColorOverride === 'string' || parsed.primaryColorOverride === null
                ? parsed.primaryColorOverride
                : presetSettings.primary,

            base100: typeof parsed.base100 === 'string' || parsed.base100 === null ? parsed.base100 : presetSettings.base100,
            base200: typeof parsed.base200 === 'string' || parsed.base200 === null ? parsed.base200 : presetSettings.base200,
            base300: typeof parsed.base300 === 'string' || parsed.base300 === null ? parsed.base300 : presetSettings.base300,
            baseContent: typeof parsed.baseContent === 'string' || parsed.baseContent === null ? parsed.baseContent : presetSettings.baseContent,

            primaryContent: typeof parsed.primaryContent === 'string' || parsed.primaryContent === null ? parsed.primaryContent : presetSettings.primaryContent,
            secondary: typeof parsed.secondary === 'string' || parsed.secondary === null ? parsed.secondary : presetSettings.secondary,
            secondaryContent: typeof parsed.secondaryContent === 'string' || parsed.secondaryContent === null ? parsed.secondaryContent : presetSettings.secondaryContent,
            accent: typeof parsed.accent === 'string' || parsed.accent === null ? parsed.accent : presetSettings.accent,
            accentContent: typeof parsed.accentContent === 'string' || parsed.accentContent === null ? parsed.accentContent : presetSettings.accentContent,
            neutral: typeof parsed.neutral === 'string' || parsed.neutral === null ? parsed.neutral : presetSettings.neutral,
            neutralContent: typeof parsed.neutralContent === 'string' || parsed.neutralContent === null ? parsed.neutralContent : presetSettings.neutralContent,
            info: typeof parsed.info === 'string' || parsed.info === null ? parsed.info : presetSettings.info,
            infoContent: typeof parsed.infoContent === 'string' || parsed.infoContent === null ? parsed.infoContent : presetSettings.infoContent,
            success: typeof parsed.success === 'string' || parsed.success === null ? parsed.success : presetSettings.success,
            successContent: typeof parsed.successContent === 'string' || parsed.successContent === null ? parsed.successContent : presetSettings.successContent,
            warning: typeof parsed.warning === 'string' || parsed.warning === null ? parsed.warning : presetSettings.warning,
            warningContent: typeof parsed.warningContent === 'string' || parsed.warningContent === null ? parsed.warningContent : presetSettings.warningContent,
            error: typeof parsed.error === 'string' || parsed.error === null ? parsed.error : presetSettings.error,
            errorContent: typeof parsed.errorContent === 'string' || parsed.errorContent === null ? parsed.errorContent : presetSettings.errorContent
        });
    } catch (e) {
        console.error('Failed to import theme preset:', e);
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
    importPreset
};
import { reactive, watchEffect } from 'vue';

interface CardSettings {
    borderRadius: string;
    shadow: string;
    padding: string;
    customCSS: string;
    enableCustomCSS: boolean;
    globalRadius: string;
    primaryColorOverride: string | null;
    reduceMotion: boolean;
}

const defaultCardSettings: CardSettings = {
    borderRadius: '0.5rem',
    shadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
    padding: '0.5rem',
    customCSS: '',
    enableCustomCSS: false,
    globalRadius: '0.5rem',
    primaryColorOverride: null,
    reduceMotion: false
};

const cardSettings = reactive<CardSettings>({ ...defaultCardSettings });

const applyCardSettings = () => {
    const root = document.documentElement;

    root.style.setProperty('--client-card-radius', cardSettings.borderRadius);
    root.style.setProperty('--client-card-shadow', cardSettings.shadow);
    root.style.setProperty('--client-card-padding', cardSettings.padding);

    root.style.setProperty('--radius-box', cardSettings.globalRadius);
    root.style.setProperty('--radius-field', cardSettings.globalRadius);
    root.style.setProperty('--radius-selector', cardSettings.globalRadius);

    if (cardSettings.primaryColorOverride && cardSettings.primaryColorOverride.trim().length > 0) {
        root.style.setProperty('--color-primary', cardSettings.primaryColorOverride);
    } else {
        root.style.removeProperty('--color-primary');
    }

    root.setAttribute('data-reduce-motion', cardSettings.reduceMotion ? 'true' : 'false');

    let styleEl = document.getElementById('custom-theme-styles');
    if (!styleEl) {
        styleEl = document.createElement('style');
        styleEl.id = 'custom-theme-styles';
        document.head.appendChild(styleEl);
    }

    if (cardSettings.enableCustomCSS) {
        let customStyles = '';

        if (cardSettings.customCSS) {
            customStyles += cardSettings.customCSS;
        }

        styleEl.textContent = customStyles;
    } else {
        styleEl.textContent = '';
    }
};

const loadCardSettings = () => {
    try {
        const savedSettings = localStorage.getItem('cardSettings');
        if (savedSettings) {
            const parsedSettings = JSON.parse(savedSettings);
            Object.assign(cardSettings, {
                ...defaultCardSettings,
                ...parsedSettings
            });
        }
    } catch (error) {
        console.error('Failed to load card settings:', error);
        Object.assign(cardSettings, defaultCardSettings);
    }

    applyCardSettings();
};

/**
 * Save current settings to localStorage
 */
const saveCardSettings = () => {
    try {
        localStorage.setItem('cardSettings', JSON.stringify(cardSettings));
        console.log('Saved card settings to localStorage:', cardSettings);
    } catch (error) {
        console.error('Failed to save card settings:', error);
    }
};

const updateCardSettings = (settings: Partial<CardSettings>) => {
    Object.assign(cardSettings, settings);
    saveCardSettings();
    applyCardSettings();
};

const resetCardSettings = () => {
    Object.assign(cardSettings, defaultCardSettings);
    saveCardSettings();
    applyCardSettings();
};

const exportPreset = (): string => {
    const preset = {
        borderRadius: cardSettings.borderRadius,
        shadow: cardSettings.shadow,
        padding: cardSettings.padding,
        customCSS: cardSettings.customCSS,
        enableCustomCSS: cardSettings.enableCustomCSS,
        globalRadius: cardSettings.globalRadius,
        primaryColorOverride: cardSettings.primaryColorOverride,
        reduceMotion: cardSettings.reduceMotion
    };
    return JSON.stringify(preset, null, 2);
};

const importPreset = (presetJSON: string): void => {
    try {
        const parsed = JSON.parse(presetJSON);
        updateCardSettings({
            borderRadius: typeof parsed.borderRadius === 'string' ? parsed.borderRadius : cardSettings.borderRadius,
            shadow: typeof parsed.shadow === 'string' ? parsed.shadow : cardSettings.shadow,
            padding: typeof parsed.padding === 'string' ? parsed.padding : cardSettings.padding,
            customCSS: typeof parsed.customCSS === 'string' ? parsed.customCSS : cardSettings.customCSS,
            enableCustomCSS: typeof parsed.enableCustomCSS === 'boolean' ? parsed.enableCustomCSS : cardSettings.enableCustomCSS,
            globalRadius: typeof parsed.globalRadius === 'string' ? parsed.globalRadius : cardSettings.globalRadius,
            primaryColorOverride: typeof parsed.primaryColorOverride === 'string' || parsed.primaryColorOverride === null
                ? parsed.primaryColorOverride
                : cardSettings.primaryColorOverride,
            reduceMotion: typeof parsed.reduceMotion === 'boolean' ? parsed.reduceMotion : cardSettings.reduceMotion
        });
    } catch (e) {
        console.error('Failed to import theme preset:', e);
        throw e;
    }
};

loadCardSettings();

watchEffect(() => {
    applyCardSettings();
});

export const themeService = {
    settings: cardSettings,
    updateCardSettings,
    resetCardSettings,
    loadCardSettings,
    saveCardSettings,
    exportPreset,
    importPreset
};
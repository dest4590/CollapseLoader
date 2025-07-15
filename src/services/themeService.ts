import { reactive, watchEffect } from 'vue';

interface CardSettings {
    borderRadius: string;
    shadow: string;
    padding: string;
    customCSS: string;
    enableCustomCSS: boolean;
}

const defaultCardSettings: CardSettings = {
    borderRadius: '0.5rem',
    shadow: '0 4px 6px -1px rgba(0, 0, 0, 0.1)',
    padding: '0.5rem',
    customCSS: '',
    enableCustomCSS: false
};

const cardSettings = reactive<CardSettings>({ ...defaultCardSettings });

/**
 * Apply CSS variables to the document root element
 */
const applyCardSettings = () => {
    const root = document.documentElement;
    root.style.setProperty('--client-card-radius', cardSettings.borderRadius);
    root.style.setProperty('--client-card-shadow', cardSettings.shadow);
    root.style.setProperty('--client-card-padding', cardSettings.padding);

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

/**
 * Load settings from localStorage
 */
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

/**
 * Update card settings
 */
const updateCardSettings = (settings: Partial<CardSettings>) => {
    Object.assign(cardSettings, settings);
    saveCardSettings();
    applyCardSettings();
};

/**
 * Reset card settings to defaults
 */
const resetCardSettings = () => {
    Object.assign(cardSettings, defaultCardSettings);
    saveCardSettings();
    applyCardSettings();
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
    saveCardSettings
};
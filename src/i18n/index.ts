import { createI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';

import en from './locales/en.json';
import ru from './locales/ru.json';

const messages = {
    en,
    ru
};

interface Setting<T> {
    value: T;
    show: boolean;
}

interface AppSettings {
    language?: Setting<string>;
    theme?: Setting<string>;
    [key: string]: any;
}

const getInitialLanguage = async () => {
    try {
        const settings = await invoke<AppSettings>('get_settings');
        if (settings?.language?.value && messages[settings.language.value as keyof typeof messages]) {
            return settings.language.value;
        }
    } catch (error) {
        console.warn('Failed to load language from settings:', error);
    }

    return localStorage.getItem('language') || 'en';
};

const getStoredLanguage = () => {
    return localStorage.getItem('language') || 'en';
};

const i18n = createI18n({
    legacy: false,
    locale: getStoredLanguage(),
    fallbackLocale: 'en',
    messages,
    globalInjection: true
});

const initializeLanguage = async () => {
    const language = await getInitialLanguage();
    if (language !== i18n.global.locale.value) {
        i18n.global.locale.value = language as any;
        document.documentElement.setAttribute('lang', language);
    }
};

initializeLanguage().catch(error => {
    console.warn('Failed to initialize language:', error);
});

export default i18n;

export const changeLanguage = async (locale: string) => {
    i18n.global.locale.value = locale as any;
    localStorage.setItem('language', locale);
    document.documentElement.setAttribute('lang', locale);

    try {
        const currentSettings = await invoke<AppSettings>('get_settings');
        if (currentSettings && typeof currentSettings === 'object') {
            const updatedSettings = {
                ...currentSettings,
                language: {
                    value: locale,
                    show: currentSettings.language?.show ?? true
                }
            };
            await invoke('save_settings', { inputSettings: updatedSettings });
        }
    } catch (error) {
        console.error('Failed to save language to settings:', error);
    }
};

export const getCurrentLanguage = () => {
    return i18n.global.locale.value;
};

export const getAvailableLanguages = () => {
    return [
        { code: 'en', name: 'English', nativeName: 'English' },
        { code: 'ru', name: 'Russian', nativeName: 'Русский' }
    ];
};
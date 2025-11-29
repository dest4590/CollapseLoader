import { createI18n } from 'vue-i18n';
import { settingsService } from '../services/settingsService';

import en from './locales/en.json';
import ru from './locales/ru.json';
import ua from './locales/ua.json';
import zh_cn from './locales/zh_cn.json'

const messages = {
    en,
    ru,
    ua,
    zh_cn
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
        await settingsService.loadSettings();
        const settings = settingsService.getSettings() as AppSettings | null;
        if (settings && settings.language?.value && messages[settings.language.value as keyof typeof messages]) {
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
        await settingsService.editSetting('language', locale);
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
        { code: 'ru', name: 'Russian', nativeName: 'Русский' },
        { code: 'ua', name: 'Ukrainian', nativeName: 'Українська' },
        { code: 'zh_cn', name: 'Chinese', nativeName: '简体中文' }
    ];
};
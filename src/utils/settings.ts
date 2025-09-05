import { invoke } from '@tauri-apps/api/core';
import { changeLanguage } from '../i18n';
import { themeService } from '../services/themeService';

export interface Setting<T> {
    description: string;
    value: T;
}

export interface AppSettings {
    theme: Setting<string>;
    language: Setting<string>;
    [key: string]: Setting<any>;
}

export const fetchSettings = async (): Promise<AppSettings | null> => {
    try {
        return await invoke<AppSettings>('get_settings');
    } catch (err) {
        console.warn('fetchSettings failed', err);
        return null;
    }
};

export const applyLanguageOnStartup = async () => {
    const settings = await fetchSettings();
    const lang = settings?.language?.value || localStorage.getItem('language') || 'en';
    try {
        await changeLanguage(lang);
    } catch (err) {
        console.error('Failed to change language:', err);
    }
};

export const applyThemeOnStartup = async () => {
    const settings = await fetchSettings();
    const themeFromSettings = settings?.theme?.value;
    const localTheme = localStorage.getItem('theme');

    const chosenTheme =
        themeFromSettings || (localTheme && ['light', 'dark'].includes(localTheme) ? localTheme : 'dark');

    document.documentElement.setAttribute('data-theme', chosenTheme);

    try {
        themeService.loadSettings();
    } catch (e) {
        console.error('Failed to load theme settings in service:', e);
    }
};

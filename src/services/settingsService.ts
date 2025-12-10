import { reactive, ref, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';

export interface Setting<T> {
    value: T;
    show: boolean;
}

export interface SettingsMap {
    [key: string]: Setting<any>;
}

export interface FlagsMap {
    [key: string]: any;
}

class SettingsService {
    constructor() {
        this.initAutoSaveWatcher();
    }
    settings = reactive<SettingsMap>({});
    flags = reactive<FlagsMap>({});
    isLoading = ref(false);
    isSaving = ref(false);
    isEditing = ref(false);

    async loadSettings(): Promise<void> {
        this.isLoading.value = true;
        try {
            const loaded = await invoke<SettingsMap>('get_settings');
            Object.keys(this.settings).forEach((k) => delete this.settings[k]);
            Object.entries(loaded || {}).forEach(([k, v]) => {
                this.settings[k] = v;
            });
        } catch (err) {
            console.error('Failed to load settings:', err);
            throw err;
        } finally {
            this.isLoading.value = false;
        }
    }

    async saveSettings(settingsArg?: SettingsMap): Promise<void> {
        this.isSaving.value = true;
        try {
            const payload = settingsArg ?? this.settings;
            await invoke('save_settings', { inputSettings: payload });
        } catch (err) {
            console.error('Failed to save settings:', err);
            throw err;
        } finally {
            this.isSaving.value = false;
        }
    }

    async loadFlags(): Promise<void> {
        try {
            const loaded = await invoke<FlagsMap>('get_flags');
            Object.keys(this.flags).forEach((k) => delete this.flags[k]);
            Object.entries(loaded || {}).forEach(([k, v]) => {
                this.flags[k] = v;
            });
        } catch (err) {
            console.error('Failed to load flags:', err);
            throw err;
        }
    }

    setSetting(key: string, value: any) {
        if (!this.settings[key]) {
            this.settings[key] = { value, show: true } as Setting<any>;
            return;
        }
        this.settings[key].value = value;
    }

    async editSetting(key: string, value: any, show: boolean = true): Promise<void> {
        this.isEditing.value = true;
        try {
            if (this.settings[key]) {
                this.settings[key].value = value;
                this.settings[key].show = show;
            } else {
                this.settings[key] = { value, show };
            }

            await this.saveSettings();
        } finally {
            this.isEditing.value = false;
        }
    }

    getSetting<T = any>(key: string): Setting<T> | undefined {
        return this.settings[key] as Setting<T> | undefined;
    }

    getSettings(): SettingsMap {
        return this.settings;
    }

    getFlags(): FlagsMap {
        return this.flags;
    }

    initAutoSaveWatcher() {
        let saveTimeout: number | null = null;
        watch(
            () => this.settings,
            () => {
                if (this.isLoading.value || this.isEditing.value) return;

                if (saveTimeout) {
                    clearTimeout(saveTimeout);
                }

                saveTimeout = setTimeout(async () => {
                    try {
                        await this.saveSettings();
                    } catch (e) {
                        console.error('Auto-save failed', e);
                    }
                }, 500) as unknown as number;
            },
            { deep: true }
        );
    }
}

export const settingsService = new SettingsService();

export default settingsService;

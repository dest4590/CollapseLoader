import { invoke } from '@tauri-apps/api/core';
import type { ThemePreset, CreatePresetInput, UpdatePresetInput } from '../types/presets';
import { themeService } from './themeService';

export class PresetService {
    async getAllPresets(): Promise<ThemePreset[]> {
        return await invoke<ThemePreset[]>('get_all_presets');
    }

    async getPreset(id: string): Promise<ThemePreset | null> {
        return await invoke<ThemePreset | null>('get_preset', { id });
    }

    async createPreset(input: CreatePresetInput): Promise<ThemePreset> {
        return await invoke<ThemePreset>('create_preset', { input });
    }

    async updatePreset(input: UpdatePresetInput): Promise<ThemePreset> {
        return await invoke<ThemePreset>('update_preset', { input });
    }

    async deletePreset(id: string): Promise<void> {
        await invoke<void>('delete_preset', { id });
    }

    async duplicatePreset(id: string, newName: string): Promise<ThemePreset> {
        return await invoke<ThemePreset>('duplicate_preset', { id, newName: newName });
    }

    createPresetFromCurrentSettings(name: string, description?: string): CreatePresetInput {
        const settings = themeService.presetSettings;

        return {
            name,
            description,
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
        };
    }

    applyPresetToTheme(preset: ThemePreset): void {
        themeService.updatePresetSettings({
            customCSS: preset.customCSS,
            enableCustomCSS: preset.enableCustomCSS,

            base100: preset.base100,
            base200: preset.base200,
            base300: preset.base300,
            baseContent: preset.baseContent,

            primary: preset.primary,
            primaryContent: preset.primaryContent,
            secondary: preset.secondary,
            secondaryContent: preset.secondaryContent,
            accent: preset.accent,
            accentContent: preset.accentContent,
            neutral: preset.neutral,
            neutralContent: preset.neutralContent,
            info: preset.info,
            infoContent: preset.infoContent,
            success: preset.success,
            successContent: preset.successContent,
            warning: preset.warning,
            warningContent: preset.warningContent,
            error: preset.error,
            errorContent: preset.errorContent,
        });
    }
}

export const presetService = new PresetService();
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
            border_radius: settings.borderRadius,
            shadow: settings.shadow,
            padding: settings.padding,
            custom_css: settings.customCSS,
            enable_custom_css: settings.enableCustomCSS,
            global_radius: settings.globalRadius,
            primary_color_override: settings.primaryColorOverride || undefined,
            reduce_motion: settings.reduceMotion,

            base100: settings.base100 || undefined,
            base200: settings.base200 || undefined,
            base300: settings.base300 || undefined,
            base_content: settings.baseContent || undefined,

            primary_content: settings.primaryContent || undefined,
            secondary: settings.secondary || undefined,
            secondary_content: settings.secondaryContent || undefined,
            accent: settings.accent || undefined,
            accent_content: settings.accentContent || undefined,
            neutral: settings.neutral || undefined,
            neutral_content: settings.neutralContent || undefined,
            info: settings.info || undefined,
            info_content: settings.infoContent || undefined,
            success: settings.success || undefined,
            success_content: settings.successContent || undefined,
            warning: settings.warning || undefined,
            warning_content: settings.warningContent || undefined,
            error: settings.error || undefined,
            error_content: settings.errorContent || undefined,
        };
    }

    applyPresetToTheme(preset: ThemePreset): void {
        themeService.updatePresetSettings({
            borderRadius: preset.border_radius,
            shadow: preset.shadow,
            padding: preset.padding,
            customCSS: preset.custom_css,
            enableCustomCSS: preset.enable_custom_css,
            globalRadius: preset.global_radius,
            primaryColorOverride: preset.primary_color_override,
            reduceMotion: preset.reduce_motion,

            base100: preset.base100,
            base200: preset.base200,
            base300: preset.base300,
            baseContent: preset.base_content,

            primaryContent: preset.primary_content,
            secondary: preset.secondary,
            secondaryContent: preset.secondary_content,
            accent: preset.accent,
            accentContent: preset.accent_content,
            neutral: preset.neutral,
            neutralContent: preset.neutral_content,
            info: preset.info,
            infoContent: preset.info_content,
            success: preset.success,
            successContent: preset.success_content,
            warning: preset.warning,
            warningContent: preset.warning_content,
            error: preset.error,
            errorContent: preset.error_content,
        });
    }
}

export const presetService = new PresetService();
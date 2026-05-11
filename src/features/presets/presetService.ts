import { invoke } from "@tauri-apps/api/core";
import type {
    ThemePreset,
    CreatePresetInput,
    UpdatePresetInput,
} from "@features/presets/types";
import { themeService } from "@services/theme/themeService";
import {
    createPresetInputFromThemeSettings,
    extractThemeSettingsFromPreset,
} from "@services/theme/themeSettings";

export class PresetService {
    async getAllPresets(): Promise<ThemePreset[]> {
        return await invoke<ThemePreset[]>("get_all_presets");
    }

    async getPreset(id: string): Promise<ThemePreset | null> {
        return await invoke<ThemePreset | null>("get_preset", { id });
    }

    async createPreset(input: CreatePresetInput): Promise<ThemePreset> {
        return await invoke<ThemePreset>("create_preset", { input });
    }

    async updatePreset(input: UpdatePresetInput): Promise<ThemePreset> {
        return await invoke<ThemePreset>("update_preset", { input });
    }

    async deletePreset(id: string): Promise<void> {
        await invoke<void>("delete_preset", { id });
    }

    async duplicatePreset(id: string, newName: string): Promise<ThemePreset> {
        return await invoke<ThemePreset>("duplicate_preset", {
            id,
            newName: newName,
        });
    }

    createPresetFromCurrentSettings(
        name: string,
        description?: string
    ): CreatePresetInput {
        const themeFields = createPresetInputFromThemeSettings(
            themeService.presetSettings
        );

        return {
            name,
            description,
            ...themeFields,
        };
    }

    applyPresetToTheme(preset: ThemePreset): void {
        themeService.updatePresetSettings(
            extractThemeSettingsFromPreset(preset)
        );
    }
}

export const presetService = new PresetService();

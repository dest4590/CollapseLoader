import { ref, type Ref } from 'vue';
import { useI18n } from 'vue-i18n';
import type { ThemePreset, CreatePresetInput, UpdatePresetInput } from '../types/presets';
import { presetService } from '../services/presetService';
import { useToast } from '../services/toastService';

const presets: Ref<ThemePreset[]> = ref([]);
const loading = ref(false);
const error = ref<string | null>(null);

export function usePresets() {
    const { t } = useI18n();
    const { addToast } = useToast();

    const loadPresets = async (): Promise<void> => {
        try {
            loading.value = true;
            error.value = null;
            presets.value = await presetService.getAllPresets();
        } catch (err) {
            error.value = err instanceof Error ? err.message : t('theme.presets.messages.load_error');
            addToast(t('theme.presets.messages.load_error'), 'error');
        } finally {
            loading.value = false;
        }
    };

    const createPreset = async (input: CreatePresetInput): Promise<ThemePreset | null> => {
        try {
            const preset = await presetService.createPreset(input);
            presets.value.push(preset);
            addToast(t('theme.presets.messages.create_success', { name: preset.name }), 'success');
            return preset;
        } catch (err) {
            const message = err instanceof Error ? err.message : t('theme.presets.messages.create_error');
            error.value = message;
            addToast(message, 'error');
            return null;
        }
    };

    const updatePreset = async (input: UpdatePresetInput): Promise<ThemePreset | null> => {
        try {
            const updatedPreset = await presetService.updatePreset(input);
            const index = presets.value.findIndex(p => p.id === input.id);
            if (index !== -1) {
                presets.value[index] = updatedPreset;
            }
            addToast(t('theme.presets.messages.update_success', { name: updatedPreset.name }), 'success');
            return updatedPreset;
        } catch (err) {
            const message = err instanceof Error ? err.message : t('theme.presets.messages.update_error');
            error.value = message;
            addToast(message, 'error');
            return null;
        }
    };

    const deletePreset = async (id: string): Promise<boolean> => {
        try {
            const preset = presets.value.find(p => p.id === id);
            await presetService.deletePreset(id);
            presets.value = presets.value.filter(p => p.id !== id);
            addToast(t('theme.presets.messages.delete_success', { name: preset?.name || 'Unknown' }), 'success');
            return true;
        } catch (err) {
            const message = err instanceof Error ? err.message : t('theme.presets.messages.delete_error');
            error.value = message;
            addToast(message, 'error');
            return false;
        }
    };

    const duplicatePreset = async (id: string, newName: string): Promise<ThemePreset | null> => {
        try {
            const duplicatedPreset = await presetService.duplicatePreset(id, newName);
            presets.value.push(duplicatedPreset);
            addToast(t('theme.presets.messages.duplicate_success', { name: duplicatedPreset.name }), 'success');
            return duplicatedPreset;
        } catch (err) {
            const message = err instanceof Error ? err.message : t('theme.presets.messages.duplicate_error');
            error.value = message;
            addToast(message, 'error');
            return null;
        }
    };

    const applyPreset = (preset: ThemePreset): void => {
        try {
            presetService.applyPresetToTheme(preset);
            addToast(t('theme.presets.messages.apply_success', { name: preset.name }), 'success');
        } catch (err) {
            const message = err instanceof Error ? err.message : t('theme.presets.messages.apply_error');
            error.value = message;
            addToast(message, 'error');
        }
    };

    const saveCurrentAsPreset = async (name: string, description?: string): Promise<ThemePreset | null> => {
        const input = presetService.createPresetFromCurrentSettings(name, description);
        return await createPreset(input);
    };

    const getPresetById = (id: string): ThemePreset | undefined => {
        return presets.value.find(p => p.id === id);
    };

    const exportPreset = (preset: ThemePreset): string => {
        return JSON.stringify(preset, null, 2);
    };

    const importPresetFromJSON = async (jsonString: string): Promise<ThemePreset | null> => {
        try {
            const presetData = JSON.parse(jsonString);

            if (!presetData.name) {
                throw new Error(t('theme.presets.messages.import_no_name'));
            }

            const input: CreatePresetInput = {
                name: presetData.name + ' (Imported)',
                description: presetData.description,
                custom_css: presetData.custom_css || presetData.customCSS || '',
                enable_custom_css: presetData.enable_custom_css ?? presetData.enableCustomCSS ?? false,
                primary: presetData.primary || presetData.primaryColorOverride,

                base100: presetData.base100,
                base200: presetData.base200,
                base300: presetData.base300,
                base_content: presetData.base_content || presetData.baseContent,

                primary_content: presetData.primary_content || presetData.primaryContent,
                secondary: presetData.secondary,
                secondary_content: presetData.secondary_content || presetData.secondaryContent,
                accent: presetData.accent,
                accent_content: presetData.accent_content || presetData.accentContent,
                neutral: presetData.neutral,
                neutral_content: presetData.neutral_content || presetData.neutralContent,
                info: presetData.info,
                info_content: presetData.info_content || presetData.infoContent,
                success: presetData.success,
                success_content: presetData.success_content || presetData.successContent,
                warning: presetData.warning,
                warning_content: presetData.warning_content || presetData.warningContent,
                error: presetData.error,
                error_content: presetData.error_content || presetData.errorContent,
            };

            return await createPreset(input);
        } catch (err) {
            const message = err instanceof Error ? err.message : t('theme.presets.messages.import_error');
            error.value = message;
            addToast(message, 'error');
            return null;
        }
    };

    return {
        presets,
        loading,
        error,
        loadPresets,
        createPreset,
        updatePreset,
        deletePreset,
        duplicatePreset,
        applyPreset,
        saveCurrentAsPreset,
        getPresetById,
        exportPreset,
        importPresetFromJSON
    };
}
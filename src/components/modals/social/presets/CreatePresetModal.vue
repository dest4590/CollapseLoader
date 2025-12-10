<template>
    <div class="p-6 space-y-4">
        <div class="form-control">
            <label class="label">
                <span class="label-text">{{ $t('theme.presets.create_modal.name_label') }} *</span>
            </label>
            <input v-model="form.name" type="text" :placeholder="$t('theme.presets.create_modal.name_placeholder')"
                class="input input-bordered w-full" :class="{ 'input-error': errors.name }"
                @input="clearError('name')" />
            <label v-if="errors.name" class="label">
                <span class="label-text-alt text-error">{{ errors.name }}</span>
            </label>
        </div>

        <div class="form-control">
            <label class="label">
                <span class="label-text">{{ $t('theme.presets.create_modal.description_label') }}</span>
            </label>
            <textarea v-model="form.description" :placeholder="$t('theme.presets.create_modal.description_placeholder')"
                class="textarea textarea-bordered w-full" rows="3"></textarea>
        </div>

        <div class="form-control">
            <label class="label">
                <span class="label-text flex items-center gap-2">
                    <Info class="w-4 h-4" />
                    {{ $t('theme.presets.create_modal.source_info') }}
                </span>
            </label>
            <div class="text-sm text-base-content/70">
                {{ editingPreset ? $t('theme.presets.create_modal.source_edit') :
                    $t('theme.presets.create_modal.source_create') }}
            </div>
        </div>
    </div>

    <div class="flex gap-3 p-6 border-t border-base-300">
        <button @click="save" class="btn btn-primary flex-1" :class="{ 'loading': saving }"
            :disabled="saving || !form.name.trim()">
            <Save class="w-4 h-4 mr-2" v-if="!saving" />
            {{ editingPreset ? $t('theme.presets.create_modal.update_button') :
                $t('theme.presets.create_modal.create_button') }}
        </button>
        <button @click="close" class="btn btn-ghost flex-1">
            {{ $t('theme.presets.create_modal.cancel') }}
        </button>
    </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue';
import { Save, Info } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';
import type { ThemePreset, UpdatePresetInput } from '../../../../types/presets';

interface Props {
    editingPreset?: ThemePreset | null;
}

interface Emits {
    (e: 'close'): void;
    (e: 'save', data: { name: string; description?: string }): void;
    (e: 'update', data: UpdatePresetInput): void;
}

const props = withDefaults(defineProps<Props>(), {
    editingPreset: null
});

const emit = defineEmits<Emits>();
const { t } = useI18n();

const saving = ref(false);

const form = reactive({
    name: '',
    description: ''
});

const errors = reactive({
    name: ''
});

const clearError = (field: keyof typeof errors) => {
    errors[field] = '';
};

const validateForm = (): boolean => {
    let isValid = true;

    if (!form.name.trim()) {
        errors.name = t('theme.presets.create_modal.name_required');
        isValid = false;
    } else if (form.name.trim().length > 50) {
        errors.name = t('theme.presets.create_modal.name_too_long');
        isValid = false;
    }

    return isValid;
};

const save = async () => {
    if (!validateForm()) return;

    try {
        saving.value = true;

        if (props.editingPreset) {
            emit('update', {
                id: props.editingPreset.id,
                name: form.name.trim(),
                description: form.description.trim() || undefined,
                customCSS: props.editingPreset.customCSS,
                enableCustomCSS: props.editingPreset.enableCustomCSS,

                base100: props.editingPreset.base100,
                base200: props.editingPreset.base200,
                base300: props.editingPreset.base300,
                baseContent: props.editingPreset.baseContent,

                primary: props.editingPreset.primary,
                primaryContent: props.editingPreset.primaryContent,
                secondary: props.editingPreset.secondary,
                secondaryContent: props.editingPreset.secondaryContent,
                accent: props.editingPreset.accent,
                accentContent: props.editingPreset.accentContent,
                neutral: props.editingPreset.neutral,
                neutralContent: props.editingPreset.neutralContent,
                info: props.editingPreset.info,
                infoContent: props.editingPreset.infoContent,
                success: props.editingPreset.success,
                successContent: props.editingPreset.successContent,
                warning: props.editingPreset.warning,
                warningContent: props.editingPreset.warningContent,
                error: props.editingPreset.error,
                errorContent: props.editingPreset.errorContent,
            });
        } else {
            emit('save', {
                name: form.name.trim(),
                description: form.description.trim() || undefined
            });
        }
    } finally {
        saving.value = false;
        close();
    }
};

const close = () => {
    emit('close');
};

const resetForm = () => {
    form.name = '';
    form.description = '';
    errors.name = '';
};

watch(() => props.editingPreset, (preset) => {
    if (preset) {
        form.name = preset.name;
        form.description = preset.description || '';
    } else {
        resetForm();
    }
}, { immediate: true });
</script>
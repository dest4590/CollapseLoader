<template>

    <div class="p-6 space-y-4">
        <div class="form-control">
            <label class="label">
                <span class="label-text">{{ $t('theme.presets.import_modal.method_label') }}</span>
            </label>
            <div class="tabs tabs-boxed">
                <button @click="importMethod = 'file'" class="tab" :class="{ 'tab-active': importMethod === 'file' }">
                    <Upload class="w-4 h-4 mr-2" />
                    {{ $t('theme.presets.import_modal.method_file') }}
                </button>
                <button @click="importMethod = 'json'" class="tab" :class="{ 'tab-active': importMethod === 'json' }">
                    <FileText class="w-4 h-4 mr-2" />
                    {{ $t('theme.presets.import_modal.method_json') }}
                </button>
            </div>
        </div>

        <div v-if="importMethod === 'file'" class="form-control">
            <label class="label">
                <span class="label-text">{{ $t('theme.presets.import_modal.file_label') }}</span>
            </label>
            <input ref="fileInput" type="file" accept=".json" @change="handleFileChange"
                class="file-input file-input-bordered w-full" />
            <label class="label">
                <span class="label-text-alt">{{ $t('theme.presets.import_modal.file_help') }}</span>
            </label>
        </div>

        <div v-if="importMethod === 'json'" class="form-control">
            <label class="label">
                <span class="label-text">{{ $t('theme.presets.import_modal.json_label') }}</span>
            </label>
            <textarea v-model="jsonInput" :placeholder="$t('theme.presets.import_modal.json_placeholder')"
                class="textarea textarea-bordered w-full h-32" :class="{ 'textarea-error': error }"
                @input="error = ''"></textarea>
            <label v-if="error" class="label">
                <span class="label-text-alt text-error">{{ error }}</span>
            </label>
        </div>

        <div v-if="previewData" class="card bg-base-200 border border-base-300">
            <div class="card-body p-4">
                <h4 class="card-title text-sm">{{ $t('theme.presets.import_modal.preview_title') }}</h4>
                <div class="text-sm space-y-1">
                    <div><strong>{{ $t('theme.presets.import_modal.preview_name') }}:</strong> {{ previewData.name }}
                    </div>
                    <div v-if="previewData.description">
                        <strong>{{ $t('theme.presets.import_modal.preview_description') }}:</strong> {{
                        previewData.description }}
                    </div>
                </div>
            </div>
        </div>
    </div>

    <div class="flex gap-3 p-6 border-t border-base-300">
        <button @click="importPreset" class="btn btn-primary flex-1" :class="{ 'loading': importing }"
            :disabled="importing || !canImport">
            <Upload class="w-4 h-4 mr-2" v-if="!importing" />
            {{ importing ? $t('theme.presets.import_modal.importing') : $t('theme.presets.import_modal.import_button')
            }}
        </button>
        <button @click="close" class="btn btn-ghost flex-1">
            {{ $t('theme.presets.import_modal.cancel') }}
        </button>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { Upload, FileText } from 'lucide-vue-next';
import { usePresets } from '../../../../composables/usePresets';

const { t } = useI18n();

interface Emits {
    (e: 'close'): void;
    (e: 'import'): void;
}

const emit = defineEmits<Emits>();

const { importPresetFromJSON } = usePresets();

const importMethod = ref<'file' | 'json'>('file');
const fileInput = ref<HTMLInputElement>();
const jsonInput = ref('');
const error = ref('');
const importing = ref(false);
const previewData = ref<any>(null);

const canImport = computed(() => {
    if (importMethod.value === 'file') {
        return previewData.value !== null;
    }
    return jsonInput.value.trim().length > 0 && !error.value;
});

const handleFileChange = async (event: Event) => {
    const target = event.target as HTMLInputElement;
    const file = target.files?.[0];

    if (!file) {
        previewData.value = null;
        return;
    }

    try {
        const text = await file.text();
        const data = JSON.parse(text);
        previewData.value = data;
        error.value = '';
    } catch (e) {
        console.error('Failed to parse preset file:', e);
        error.value = t('theme.presets.import_modal.error_invalid_file');
        previewData.value = null;
    }
};

const validateJSON = (jsonString: string) => {
    try {
        const data = JSON.parse(jsonString);
        if (!data.name) {
            throw new Error(t('theme.presets.import_modal.error_no_name'));
        }
        previewData.value = data;
        error.value = '';
    } catch (err) {
        error.value = err instanceof Error ? err.message : t('theme.presets.import_modal.error_invalid_json');
        previewData.value = null;
    }
};

const importPreset = async () => {
    if (!canImport.value) return;

    try {
        importing.value = true;

        let jsonData = '';
        if (importMethod.value === 'file' && previewData.value) {
            jsonData = JSON.stringify(previewData.value);
        } else {
            jsonData = jsonInput.value.trim();
        }

        const result = await importPresetFromJSON(jsonData);
        if (result) {
            emit('import');
            close();
        }
    } finally {
        importing.value = false;
    }
};

const close = () => {
    emit('close');
};

watch(jsonInput, (newValue) => {
    if (importMethod.value === 'json' && newValue.trim()) {
        validateJSON(newValue);
    } else {
        previewData.value = null;
    }
});

watch(importMethod, () => {
    previewData.value = null;
    error.value = '';
});
</script>
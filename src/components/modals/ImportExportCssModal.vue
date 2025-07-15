<template>
    <div>
        <h2 class="text-lg font-bold mb-4">{{ t('theme.import_export_css') }}</h2>
        <div v-if="mode === 'export'">
            <p class="mb-2">{{ t('theme.export_desc') }}</p>
            <textarea class="textarea textarea-bordered w-full mb-4" rows="4" readonly :value="exportedCode" />
            <button class="btn btn-primary w-full" @click="copyToClipboard(exportedCode)">{{ t('theme.copy') }}</button>
        </div>
        <div v-else>
            <p class="mb-2">{{ t('theme.import_desc') }}</p>
            <textarea v-model="importCode" class="textarea textarea-bordered w-full mb-4" rows="4" />
            <button class="btn btn-primary w-full mb-2" @click="importCss">{{ t('theme.import') }}</button>
            <div v-if="importError" class="text-error text-sm mt-2">{{ importError }}</div>
        </div>
        <div class="flex justify-end mt-4">
            <button class="btn btn-outline btn-error btn-sm flex items-center gap-2" @click="$emit('close')"
                aria-label="Close">
                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24"
                    stroke="currentColor">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                </svg>
                <span>{{ t('common.close') }}</span>
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import { useToast } from '../../services/toastService';

const props = defineProps<{
    mode: 'import' | 'export',
    css?: string
}>();
const emit = defineEmits(['import', 'close']);
const { t } = useI18n();
const { addToast } = useToast();

const importCode = ref('');
const importError = ref('');
const exportedCode = ref('');

onMounted(async () => {
    if (props.mode === 'export' && props.css) {
        try {
            exportedCode.value = await invoke('encode_base64', { input: props.css });
        } catch (e) {
            exportedCode.value = '';
        }
    }
});

const copyToClipboard = async (text: string) => {
    try {
        await navigator.clipboard.writeText(text);
        addToast(t('theme.copied'), 'success');
    } catch {
        addToast(t('theme.copy_failed'), 'error');
    }
};

const importCss = async () => {
    importError.value = '';
    try {
        const decoded = await invoke('decode_base64', { input: importCode.value });

        if (typeof decoded !== 'string' || /script|@import|url\(|expression|<|>|javascript:/i.test(decoded)) {
            importError.value = t('theme.import_invalid');
            return;
        }
        emit('import', decoded);
        addToast(t('theme.import_success'), 'success');
        emit('close');
    } catch {
        importError.value = t('theme.import_invalid');
    }
};
</script>

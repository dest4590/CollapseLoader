<template>
    <div>
        <div v-if="mode === 'export'">
            <p class="mb-2">{{ t('theme.export_desc') }}</p>
            <textarea class="textarea textarea-bordered w-full mb-4" rows="4" readonly :value="exportedCode" />
            <button class="btn btn-primary w-full" @click="copyToClipboard(exportedCode)">{{ t('theme.copy') }}</button>
        </div>
        <div v-else>
            <p class="mb-2">{{ t('theme.import_desc') }}</p>
            <textarea v-model="importCode" class="textarea textarea-bordered w-full mb-4" rows="4" />

            <div v-if="importError" class="text-error text-sm mt-2">{{ importError }}</div>
        </div>
        <div class="flex gap-3 p-6 border-t border-base-300">
            <button class="btn btn-primary flex-1" @click="importCss">
                <Upload class="w-4 h-4 mr-2" />
                {{ t('theme.import') }}
            </button>
            <button @click="$emit('close')" class="btn btn-ghost flex-1">
                {{ $t('theme.presets.import_modal.cancel') }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
import { useToast } from '../../../../services/toastService';
import { Upload } from 'lucide-vue-next';

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
            console.error('Failed to encode CSS to base64:', e);
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

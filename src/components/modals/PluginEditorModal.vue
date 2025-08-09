<template>
    <div class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
        <div class="bg-base-100 rounded-lg shadow-xl w-full max-w-7xl h-[90vh] flex flex-col">
            <div class="flex items-center justify-between p-4 border-b border-base-300">
                <div class="flex items-center gap-3">
                    <Code class="w-6 h-6 text-primary" />
                    <div>
                        <h3 class="text-lg font-semibold">
                            {{ pluginName }}
                        </h3>
                    </div>
                </div>
                <div class="flex items-center gap-2">
                    <button @click="saveCode" class="btn btn-sm btn-primary" :disabled="isSaving">
                        <Save class="w-4 h-4 mr-2" />
                        {{ t('common.save') }}
                    </button>
                    <button @click="closeEditor" class="btn btn-sm btn-ghost">
                        <X class="w-4 h-4" />
                    </button>
                </div>
            </div>

            <div class="flex-1 p-4">
                <VueMonacoEditor ref="editorRef" v-model:value="code" language="javascript"
                    :theme="currentTheme === 'dark' ? 'vs-dark' : 'vs'" :options="{
                        minimap: { enabled: false },
                        fontSize: 14,
                        tabSize: 4,
                        wordWrap: 'on',
                        automaticLayout: true,
                        scrollBeyondLastLine: false,
                        contextmenu: true,
                        formatOnPaste: true,
                        formatOnType: true,
                        suggestOnTriggerCharacters: true,
                        acceptSuggestionOnEnter: 'on',
                        snippetSuggestions: 'inline',
                        quickSuggestions: {
                            other: true,
                            comments: false,
                            strings: false
                        },
                        parameterHints: { enabled: true },
                        hover: { enabled: true },
                        bracketPairColorization: { enabled: true },
                        guides: {
                            bracketPairs: true,
                            indentation: true
                        }
                    }" class="h-full" style="border-radius: 0.5rem; border: 1px solid rgba(255, 255, 255, 0.1);"
                    @keydown.ctrl.s.prevent="saveCode" @keydown.meta.s.prevent="saveCode" />
            </div>

            <div class="flex items-center justify-between p-4 border-t border-base-300">
                <div class="flex gap-2">
                    <button @click="closeEditor" class="btn btn-sm btn-ghost">
                        {{ t('common.cancel') }}
                    </button>
                    <button @click="saveAndClose" class="btn btn-sm btn-primary" :disabled="isSaving">
                        {{ t('plugins.edit.save_and_close') }}
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { Code, Save, X } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';
import { VueMonacoEditor } from '@guolao/vue-monaco-editor';
import { getPluginService } from '../../services/pluginService';
import { useToast } from '../../services/toastService';

interface Props {
    pluginId: string;
    pluginName: string;
    currentTheme?: string;
}

const props = withDefaults(defineProps<Props>(), {
    currentTheme: 'dark'
});

const emit = defineEmits<{
    'close': [];
    'saved': [pluginId: string];
}>();

const { t } = useI18n();
const { addToast } = useToast();
const pluginService = getPluginService();

const code = ref('');
const isSaving = ref(false);
const editorRef = ref<InstanceType<typeof VueMonacoEditor> | null>(null);

const loadPluginCode = async () => {
    try {
        const pluginCode = await pluginService.getPluginCode(props.pluginId);
        if (pluginCode) {
            code.value = pluginCode;
        }
    } catch (error) {
        console.error('Failed to load plugin code:', error);
        addToast(t('plugins.edit.load_error'), 'error');
    }
};

const saveCode = async () => {
    if (isSaving.value) return;

    isSaving.value = true;
    try {
        const success = await pluginService.updatePluginCode(props.pluginId, code.value);
        if (success) {
            addToast(t('plugins.edit.save_success'), 'success');
            emit('saved', props.pluginId);
        } else {
            addToast(t('plugins.edit.save_error'), 'error');
        }
    } catch (error) {
        console.error('Failed to save plugin code:', error);
        addToast(t('plugins.edit.save_error'), 'error');
    } finally {
        isSaving.value = false;
    }
};

const saveAndClose = async () => {
    await saveCode();
    if (!isSaving.value) {
        closeEditor();
    }
};

const closeEditor = () => {
    emit('close');
};

onMounted(() => {
    loadPluginCode();
});
</script>

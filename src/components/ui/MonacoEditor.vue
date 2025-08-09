<template>
    <div ref="editorContainer" class="monaco-editor-container w-full h-full"></div>
</template>

<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch, nextTick } from 'vue';
import * as monaco from 'monaco-editor';
import { useI18n } from 'vue-i18n';

interface Props {
    modelValue: string;
    language?: string;
    theme?: 'vs' | 'vs-dark' | 'hc-black';
    readonly?: boolean;
    fontSize?: number;
    tabSize?: number;
    wordWrap?: 'off' | 'on' | 'wordWrapColumn' | 'bounded';
    minimap?: boolean;
    folding?: boolean;
    lineNumbers?: 'on' | 'off' | 'relative' | 'interval';
}

const props = withDefaults(defineProps<Props>(), {
    language: 'javascript',
    theme: 'vs-dark',
    readonly: false,
    fontSize: 14,
    tabSize: 4,
    wordWrap: 'on',
    minimap: true,
    folding: true,
    lineNumbers: 'on'
});

const emit = defineEmits<{
    'update:modelValue': [value: string];
    'change': [value: string];
    'save': [value: string];
}>();

const { t } = useI18n();
const editorContainer = ref<HTMLDivElement | null>(null);
let editor: monaco.editor.IStandaloneCodeEditor | null = null;
let model: monaco.editor.ITextModel | null = null;

const setupMonaco = () => {
    monaco.languages.typescript.javascriptDefaults.setCompilerOptions({
        target: monaco.languages.typescript.ScriptTarget.ES2020,
        allowNonTsExtensions: true,
        moduleResolution: monaco.languages.typescript.ModuleResolutionKind.NodeJs,
        module: monaco.languages.typescript.ModuleKind.CommonJS,
        noEmit: true,
        allowSyntheticDefaultImports: true,
        esModuleInterop: true,
    });

    const pluginApiTypes = `
        interface PluginAPI {
            invoke(command: string, args?: any): Promise<any>;
            addToast(message: string, type: 'info' | 'success' | 'warning' | 'error', duration?: number): void;
            getClients(): Promise<any[]>;
            getConfig(): any;
            updateConfig(config: any): void;
            subscribe(event: string, callback: (data: any) => void): () => void;
            emit(event: string, data: any): void;
            getAppSettings(): Promise<any>;
            dom: {
                createElement(tag: string, className?: string): HTMLElement;
                addToBody(element: HTMLElement): void;
                removeFromBody(element: HTMLElement): void;
                querySelector(selector: string): HTMLElement | null;
                addStyles(css: string): void;
            };
            i18n: {
                translate(key: string, params?: Record<string, any>): string;
                getCurrentLanguage(): string;
            };
        }

        interface PluginContext {
            api: PluginAPI;
            metadata: any;
        }

        declare const context: PluginContext;
    `;

    monaco.languages.typescript.javascriptDefaults.addExtraLib(
        pluginApiTypes,
        'plugin-api.d.ts'
    );
};

const createEditor = () => {
    if (!editorContainer.value) return;

    setupMonaco();

    model = monaco.editor.createModel(props.modelValue, props.language);

    editor = monaco.editor.create(editorContainer.value, {
        model,
        theme: props.theme,
        readOnly: props.readonly,
        fontSize: props.fontSize,
        tabSize: props.tabSize,
        wordWrap: props.wordWrap,
        minimap: { enabled: props.minimap },
        folding: props.folding,
        lineNumbers: props.lineNumbers,
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
    });

    model.onDidChangeContent(() => {
        const value = model?.getValue() || '';
        emit('update:modelValue', value);
        emit('change', value);
    });

    editor.addAction({
        id: 'save-plugin',
        label: t('common.save'),
        keybindings: [monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS],
        contextMenuGroupId: 'navigation',
        contextMenuOrder: 1.5,
        run: () => {
            const value = model?.getValue() || '';
            emit('save', value);
        }
    });
};

const updateEditorValue = (newValue: string) => {
    if (model && model.getValue() !== newValue) {
        model.setValue(newValue);
    }
};

const resizeEditor = () => {
    if (editor) {
        editor.layout();
    }
};

watch(() => props.modelValue, updateEditorValue);
watch(() => props.theme, (newTheme) => {
    if (editor) {
        monaco.editor.setTheme(newTheme);
    }
});

onMounted(async () => {
    await nextTick();
    createEditor();

    const resizeObserver = new ResizeObserver(() => {
        resizeEditor();
    });

    if (editorContainer.value) {
        resizeObserver.observe(editorContainer.value);
    }
});

onUnmounted(() => {
    if (editor) {
        editor.dispose();
        editor = null;
    }
    if (model) {
        model.dispose();
        model = null;
    }
});

defineExpose({
    getEditor: () => editor,
    getModel: () => model,
    focus: () => editor?.focus(),
    format: () => editor?.getAction('editor.action.formatDocument')?.run()
});
</script>

<style scoped>
.monaco-editor-container {
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 0.5rem;
    overflow: hidden;
}

:deep(.monaco-editor) {
    background: transparent !important;
}

:deep(.monaco-editor .margin) {
    background: transparent !important;
}
</style>

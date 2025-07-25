<template>
    <div class="space-y-6 max-h-[70vh] overflow-y-auto">
        <div class="space-y-4">
            <div class="form-control flex flex-col gap-2">
                <label class="flex items-center justify-between">
                    <span class="font-medium">{{ t('plugins.add_modal.paste_label') }}</span>
                </label>
                <textarea ref="textareaRef" v-model="pluginText" @paste="handlePaste"
                    class="textarea textarea-bordered h-40 w-full font-mono text-sm resize-none"></textarea>
                <span class="text-xs text-base-content/60">
                    {{ t('plugins.add_modal.paste_hint') }}
                </span>
            </div>

            <div v-if="pluginText.trim() && !isAdding" class="space-y-4">
                <div class="divider">{{ t('plugins.add_modal.preview') }}</div>

                <div v-if="parsedPlugin" class="space-y-4">
                    <div v-if="parsedPlugin.errors.length > 0" class="alert alert-error">
                        <div class="flex-col items-start">
                            <div class="flex items-center gap-2 mb-2">
                                <AlertCircle class="w-5 h-5" />
                                <span class="font-semibold">{{ t('plugins.add_modal.validation_errors') }}</span>
                            </div>
                            <ul class="list-disc list-inside text-sm space-y-1 max-h-32 overflow-y-auto">
                                <li v-for="error in parsedPlugin.errors" :key="error">
                                    {{ error }}
                                </li>
                            </ul>
                        </div>
                    </div>

                    <div v-if="parsedPlugin.metadata.name" class="card bg-base-300 shadow-md border border-base-200">
                        <div class="card-body p-4">
                            <div class="flex items-start gap-4">
                                <div class="flex-shrink-0">
                                    <div v-if="parsedPlugin.metadata.icon" class="text-3xl">
                                        {{ parsedPlugin.metadata.icon }}
                                    </div>
                                    <div v-else
                                        class="w-12 h-12 bg-primary/20 rounded-lg flex items-center justify-center">
                                        <Puzzle class="w-6 h-6 text-primary" />
                                    </div>
                                </div>

                                <div class="flex-grow space-y-2 min-w-0">
                                    <div class="flex items-center gap-3 flex-wrap">
                                        <h3 class="font-semibold text-lg truncate">{{ parsedPlugin.metadata.name }}</h3>
                                        <div class="badge badge-outline">v{{ parsedPlugin.metadata.version }}</div>
                                    </div>

                                    <p class="text-sm text-base-content/80">
                                        {{ parsedPlugin.metadata.description }}
                                    </p>

                                    <div class="flex items-center gap-4 text-xs text-base-content/60 flex-wrap">
                                        <span class="flex items-center gap-1">
                                            <User class="w-3 h-3" />
                                            <span class="truncate">{{ parsedPlugin.metadata.author }}</span>
                                        </span>
                                        <span v-if="parsedPlugin.metadata.website" class="flex items-center gap-1">
                                            <ExternalLink class="w-3 h-3" />
                                            <a :href="parsedPlugin.metadata.website" target="_blank"
                                                class="link link-primary truncate">
                                                {{ t('plugins.add_modal.website') }}
                                            </a>
                                        </span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div v-if="!parsedPlugin.metadata.name" class="alert alert-info">
                        <div class="flex-col items-start">
                            <div class="flex items-center gap-2 mb-2">
                                <Info class="w-5 h-5" />
                                <span class="font-semibold">{{ t('plugins.add_modal.metadata_help_title') }}</span>
                            </div>
                            <div class="text-sm space-y-2">
                                <p>{{ t('plugins.add_modal.metadata_help_desc') }}</p>
                                <pre class="bg-base-100 p-3 rounded text-xs overflow-x-auto whitespace-pre-wrap">/**
 * @name My Awesome Plugin
 * @version 1.0.0
 * @author Your Name
 * @description This plugin does amazing things
 * @category appearance
 * @icon 🎨
 * @website https://example.com
 */</pre>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div class="flex gap-3 justify-end pt-4 border-t border-base-300 sticky bottom-0 bg-base-200">
            <button @click="$emit('cancel')" class="btn btn-outline">
                {{ t('common.cancel') }}
            </button>
            <button @click="handleAdd" :disabled="!canAdd" class="btn btn-primary" :class="{ 'loading': isAdding }">
                <Plus v-if="!isAdding" class="w-4 h-4 mr-2" />
                {{ isAdding ? t('plugins.add_modal.adding') : t('plugins.add_modal.add_button') }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick } from 'vue';
import { useI18n } from 'vue-i18n';
import {
    Puzzle,
    AlertCircle,
    User,
    ExternalLink,
    Info,
    Plus
} from 'lucide-vue-next';
import { getPluginService } from '../../services/pluginService';
import { useToast } from '../../services/toastService';
import type { ParsedPlugin } from '../../types/plugin';

const emit = defineEmits<{
    cancel: [];
    'plugin-added': [plugin: ParsedPlugin];
}>();

const { t } = useI18n();
const pluginService = getPluginService();
const { addToast } = useToast();

const textareaRef = ref<HTMLTextAreaElement>();
const pluginText = ref('');
const isAdding = ref(false);

const parsedPlugin = computed<ParsedPlugin | null>(() => {
    if (!pluginText.value.trim()) return null;
    return pluginService.parsePluginFromText(pluginText.value);
});

const canAdd = computed(() => {
    return parsedPlugin.value?.isValid === true && !isAdding.value;
});

const handlePaste = async (_: ClipboardEvent) => {
    await nextTick();
};

const handleAdd = async () => {
    if (!parsedPlugin.value || !parsedPlugin.value.isValid) {
        addToast(t('plugins.add_modal.invalid_plugin'), 'error');
        return;
    }

    isAdding.value = true;

    try {
        const success = await pluginService.addPlugin(parsedPlugin.value);

        if (success) {
            addToast(
                t('plugins.add_modal.add_success', { name: parsedPlugin.value.metadata.name }),
                'success'
            );
            emit('plugin-added', parsedPlugin.value);
        } else {
            addToast(t('plugins.add_modal.add_failed'), 'error');
        }
    } catch (error) {
        console.error('Failed to add plugin:', error);
        addToast(t('plugins.add_modal.add_error'), 'error');
    } finally {
        isAdding.value = false;
    }
};

nextTick(() => {
    textareaRef.value?.focus();
});
</script>

<style scoped>
.textarea {
    resize: none;
}

pre {
    white-space: pre-wrap;
    word-break: break-word;
}
</style>

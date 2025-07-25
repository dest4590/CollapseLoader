<template>
    <div class="card bg-base-200 shadow-md border border-base-300 hover:shadow-lg transition-all duration-200">
        <div class="card-body p-4">
            <div class="flex items-start gap-4">
                <div class="flex-shrink-0">
                    <div v-if="plugin.metadata.icon" class="text-2xl">
                        {{ plugin.metadata.icon }}
                    </div>
                    <div v-else class="w-10 h-10 bg-primary/20 rounded-lg flex items-center justify-center">
                        <Puzzle class="w-5 h-5 text-primary" />
                    </div>
                </div>

                <div class="flex-grow min-w-0">
                    <div class="flex items-center gap-2 mb-2">
                        <h3 class="font-semibold truncate">{{ plugin.metadata.name }}</h3>
                        <div class="badge badge-outline badge-xs">v{{ plugin.metadata.version }}</div>
                    </div>

                    <p class="text-sm text-base-content/70 line-clamp-2 mb-3">
                        {{ plugin.metadata.description }}
                    </p>

                    <div class="flex items-center justify-between">
                        <div class="flex items-center gap-2">
                            <span class="text-xs text-base-content/60">
                                {{ plugin.metadata.author }}
                            </span>
                        </div>

                        <div class="flex items-center gap-2">
                            <button @click="togglePlugin" class="btn btn-xs"
                                :class="plugin.enabled ? 'btn-success' : 'btn-outline'"
                                :title="plugin.enabled ? t('plugins.disable') : t('plugins.enable')">
                                <Power class="w-3 h-3" />
                            </button>

                            <button @click="$emit('configure', plugin)" class="btn btn-ghost btn-xs"
                                :title="t('plugins.configure')">
                                <Settings class="w-3 h-3" />
                            </button>

                            <button @click="editPlugin" class="btn btn-ghost btn-xs">
                                <Code class="w-3 h-3" />
                            </button>

                            <button v-if="plugin.metadata.website" @click="openWebsite" class="btn btn-ghost btn-xs"
                                :title="t('plugins.visit_website')">
                                <ExternalLink class="w-3 h-3" />
                            </button>

                            <button @click="removePlugin" class="btn btn-ghost btn-xs text-error hover:bg-error/10"
                                :title="t('plugins.remove')">
                                <Trash2 class="w-3 h-3" />
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import {
    Puzzle,
    ExternalLink,
    Settings,
    Power,
    Trash2,
    Code
} from 'lucide-vue-next';
import { getPluginService } from '../../../services/pluginService';
import { useToast } from '../../../services/toastService';
import type { PluginState } from '../../../types/plugin';

const props = defineProps<{
    plugin: PluginState;
    allPlugins: PluginState[];
}>();

const emit = defineEmits<{
    configure: [plugin: PluginState];
    edit: [plugin: PluginState];
}>();

const { t } = useI18n();
const { addToast } = useToast();
const pluginService = getPluginService();

const togglePlugin = async () => {
    const success = await pluginService.togglePlugin(props.plugin.id);
    if (success) {
        addToast(
            t(props.plugin.enabled ? 'plugins.enabled' : 'plugins.disabled', {
                name: props.plugin.metadata.name
            }),
            props.plugin.enabled ? 'success' : 'info'
        );
    } else {
        addToast(
            t('plugins.toggle_failed', { name: props.plugin.metadata.name }),
            'error'
        );
    }
};

const removePlugin = async () => {
    const success = await pluginService.removePlugin(props.plugin.id);
    if (success) {
        addToast(
            t('plugins.removed', { name: props.plugin.metadata.name }),
            'info'
        );
    } else {
        addToast(
            t('plugins.remove_failed', { name: props.plugin.metadata.name }),
            'error'
        );
    }
};

const openWebsite = () => {
    if (props.plugin.metadata.website) {
        window.open(props.plugin.metadata.website, '_blank');
    }
};

const editPlugin = () => {
    emit('edit', props.plugin);
};
</script>
<template>
    <div class="space-y-6">
        <div class="space-y-6">
            <div v-if="noOptions" class="text-base-content/70 text-center py-8">
                {{ $t('plugins.no_config_options') }}
            </div>

            <div v-for="(schema, key) in plugin.metadata.configSchema" :key="key" class="form-control">
                <label class="label">
                    <span class="label-text font-medium">{{ schema.label }}</span>
                </label>

                <div v-if="schema.type === 'boolean'"
                    class="flex items-center justify-between p-3 border border-base-300 rounded-lg">
                    <div>
                        <div class="font-medium">{{ schema.label }}</div>
                        <div v-if="schema.description" class="text-sm text-base-content/70">{{ schema.description }}
                        </div>
                    </div>
                    <input type="checkbox" v-model="localConfig[key]" class="toggle toggle-primary" />
                </div>

                <div v-else-if="schema.type === 'string'">
                    <input type="text" v-model="localConfig[key]" :placeholder="schema.default"
                        class="input input-bordered w-full" />
                    <label v-if="schema.description" class="label">
                        <span class="label-text-alt text-base-content/70">{{ schema.description }}</span>
                    </label>
                </div>

                <div v-else-if="schema.type === 'number'">
                    <input type="number" v-model.number="localConfig[key]" :min="schema.min" :max="schema.max"
                        :step="schema.step || 1" class="input input-bordered w-full" />
                    <label v-if="schema.description" class="label">
                        <span class="label-text-alt text-base-content/70">{{ schema.description }}</span>
                    </label>
                </div>

                <div v-else-if="schema.type === 'select'">
                    <select v-model="localConfig[key]" class="select select-bordered w-full">
                        <option v-for="option in schema.options" :key="option.value" :value="option.value">
                            {{ option.label }}
                        </option>
                    </select>
                    <label v-if="schema.description" class="label">
                        <span class="label-text-alt text-base-content/70">{{ schema.description }}</span>
                    </label>
                </div>

                <div v-else-if="schema.type === 'color'">
                    <div class="flex items-center gap-2">
                        <input type="color" v-model="localConfig[key]"
                            class="w-12 h-12 rounded-lg border border-base-300 cursor-pointer" />
                        <input type="text" v-model="localConfig[key]" class="input input-bordered flex-1" />
                    </div>
                    <label v-if="schema.description" class="label">
                        <span class="label-text-alt text-base-content/70">{{ schema.description }}</span>
                    </label>
                </div>

                <div v-else-if="schema.type === 'range'">
                    <div class="flex items-center gap-4">
                        <span class="text-sm">{{ schema.min }}</span>
                        <input type="range" v-model.number="localConfig[key]" :min="schema.min" :max="schema.max"
                            :step="schema.step || 1" class="range range-primary flex-1" />
                        <span class="text-sm">{{ schema.max }}</span>
                        <span class="badge badge-primary">{{ localConfig[key] }}</span>
                    </div>
                    <label v-if="schema.description" class="label">
                        <span class="label-text-alt text-base-content/70">{{ schema.description }}</span>
                    </label>
                </div>
            </div>
        </div>

        <div class="flex gap-3 justify-end">
            <button @click="resetToDefaults" v-if="!noOptions" class="btn btn-ghost">
                <RotateCcw class="w-4 h-4 mr-2" />
                {{ $t('plugins.reset_defaults') }}
            </button>
            <button @click="$emit('cancel')" class="btn btn-ghost">
                {{ $t('common.cancel') }}
            </button>
            <button @click="saveConfig" v-if="!noOptions" class="btn btn-primary">
                <Save class="w-4 h-4 mr-2" />
                {{ $t('common.save') }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { Save, RotateCcw } from 'lucide-vue-next';
import type { PluginState, PluginConfig } from '../../types/plugin';
import { getPluginService } from '../../services/pluginService';
import { useToast } from '../../services/toastService';
import { useI18n } from 'vue-i18n';

interface Props {
    plugin: PluginState;
}

const props = defineProps<Props>();
const emit = defineEmits<{
    save: [config: PluginConfig];
    cancel: [];
}>();

const { t } = useI18n();
const pluginService = getPluginService();
const { addToast } = useToast();

const noOptions = !props.plugin.metadata.configSchema || Object.keys(props.plugin.metadata.configSchema).length === 0;
const localConfig = ref<PluginConfig>({ ...props.plugin.config });

const resetToDefaults = () => {
    if (props.plugin.metadata.configSchema) {
        for (const [key, schema] of Object.entries(props.plugin.metadata.configSchema)) {
            localConfig.value[key] = schema.default;
        }
    }
};

const saveConfig = async () => {
    try {
        const success = await pluginService.updatePluginConfig(props.plugin.id, localConfig.value);
        if (success) {
            addToast(t('plugins.config_saved'), 'success');
            emit('save', localConfig.value);
        } else {
            addToast(t('plugins.config_save_failed'), 'error');
        }
    } catch (error) {
        const errorMsg = error instanceof Error ? error.message : String(error);
        addToast(t('plugins.config_save_failed', { error: errorMsg }), 'error');
    }
};
</script>

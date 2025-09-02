<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from '../../../services/toastService';

const { t } = useI18n();
const { addToast } = useToast();

const selectedPath = ref<string>('');
const mode = ref<'move' | 'wipe'>('move');
const busy = ref(false);

const browse = async () => {
    const dir = await open({ directory: true, multiple: false });
    if (typeof dir === 'string') {
        selectedPath.value = dir;
    }
};

const confirm = async () => {
    if (!selectedPath.value) {
        addToast(t('settings.change_root.select_folder_first'), 'warning');
        return;
    }
    try {
        busy.value = true;
        await invoke('change_data_folder', {
            newPath: selectedPath.value,
            mode: mode.value,
        } as any);
        addToast(t('settings.change_root.success'), 'success');
        addToast(t('settings.change_root.restart_required'), 'info', 5000);
        emit('done');
    } catch (e: any) {
        addToast(t('settings.change_root.failed', { error: String(e) }), 'error');
    } finally {
        busy.value = false;
    }
};

const emit = defineEmits<{ (e: 'close'): void; (e: 'done'): void }>();
</script>

<template>
    <div class="space-y-4">
        <div>
            <label class="label">
                <span class="label-text">{{ t('settings.change_root.target_folder') }}</span>
            </label>
            <div class="flex gap-2">
                <input class="input input-bordered w-full bg-base-100" :placeholder="t('common.browse')"
                    v-model="selectedPath" />
                <button class="btn btn-secondary" @click="browse">{{ t('common.browse') }}</button>
            </div>
            <p class="text-xs opacity-70 mt-2">{{ t('settings.change_root.hint') }}</p>
        </div>

        <div>
            <label class="label"><span class="label-text">{{ t('settings.change_root.mode') }}</span></label>
            <div class="join join-vertical sm:join-horizontal w-full">
                <input class="join-item btn" type="radio" name="mode" :aria-label="t('settings.change_root.mode_move')"
                    value="move" v-model="mode" />
                <input class="join-item btn" type="radio" name="mode" :aria-label="t('settings.change_root.mode_wipe')"
                    value="wipe" v-model="mode" />
            </div>
            <ul class="list-disc ml-5 mt-2 text-xs opacity-70">
                <li v-if="mode === 'move'">{{ t('settings.change_root.mode_move_desc') }}</li>
                <li v-else>{{ t('settings.change_root.mode_wipe_desc') }}</li>
            </ul>
        </div>

        <div class="flex justify-end gap-2 mt-4">
            <button class="btn" @click="$emit('close')">{{ t('common.cancel') }}</button>
            <button class="btn btn-primary" :class="{ 'btn-disabled': busy }" @click="confirm">
                <span v-if="!busy">{{ t('common.save') }}</span>
                <span v-else class="loading loading-spinner loading-sm"></span>
            </button>
        </div>
    </div>
</template>

<style scoped>
@reference "tailwindcss";
@plugin "daisyui";
</style>

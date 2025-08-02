<script lang="ts" setup>
import { invoke } from '@tauri-apps/api/core';
import { useToast } from '../../services/toastService';
import { onMounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';

defineEmits(['close'])

const { addToast } = useToast();
const { t } = useI18n();
const displayOption = ref<string>('separate');
const selectedOption = ref<string>('separate');

const getFlags = async () => {
    try {
        const flags = await invoke('get_flags');
        const typedFlags = flags as { custom_clients_display?: string };
        displayOption.value = typedFlags.custom_clients_display || 'separate';
        return typedFlags;
    } catch (err) {
        console.error('Error loading flags:', err);
        addToast(`Failed to load flags: ${err}`, 'error');
        return {};
    }
};

const setCustomClientsDisplay = async () => {
    try {
        await invoke('set_custom_clients_display', { display: selectedOption.value });
    } catch (error) {
        console.error('Failed to save flags:', error);
        addToast(t('settings.save_flags_failed', { error }), 'error');
    }
};

const handleSelect = (event: Event) => {
    const target = event.target as HTMLSelectElement;
    selectedOption.value = target.value;
};

onMounted(async () => {
    await getFlags();
});

watch(selectedOption, async () => {
    await setCustomClientsDisplay();
});
</script>
<template>
    <div class="form-control w-full mb-6">
        <label class="label mb-2">
            <span class="label-text font-medium">{{ $t('custom_clients.display_mode') }}</span>
        </label>
        <select v-model="selectedOption" @select="handleSelect" class="select select-bordered w-full">
            <option value="separate">{{ $t('custom_clients.display_mode_separate') }}</option>
            <option value="global">{{ $t('custom_clients.display_mode_global') }}</option>
        </select>
    </div>
    <div class="flex justify-end">
        <button class="btn btn-primary" @click="$emit('close')">{{ $t('common.save') }}</button>
    </div>

</template>
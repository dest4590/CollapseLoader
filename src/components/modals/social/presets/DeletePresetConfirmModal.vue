<template>
    <div>
        <p class="text-sm text-base-content/70">
            {{ $t('theme.presets.delete_modal.message', { name: preset?.name }) }}
            {{ $t('theme.presets.delete_modal.warning') }}
        </p>

        <div class="flex justify-end space-x-2 mt-6">
            <button @click="confirmDelete" class="btn btn-error">
                <Trash2 class="w-4 h-4 mr-2" />
                {{ $t('theme.presets.delete_modal.delete_button') }}
            </button>
            <button @click="$emit('close')" class="btn btn-outline">
                <X class="w-4 h-4 mr-2" />
                {{ $t('theme.presets.delete_modal.cancel') }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { Trash2, X } from 'lucide-vue-next';
import type { ThemePreset } from '../../../../types/presets';

const props = defineProps<{
    preset: ThemePreset;
}>();

const emit = defineEmits(['close', 'preset-deleted']);

const confirmDelete = () => {
    emit('preset-deleted', props.preset.id);
    emit('close');
};
</script>
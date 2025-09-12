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
import { computed } from 'vue';

const props = defineProps<{
    preset?: ThemePreset;
    id?: string | number;
}>();

const emit = defineEmits(['close', 'preset-deleted', 'deleted']);

const targetId = computed(() => {
    return props.preset?.id ?? props.id;
});

const confirmDelete = () => {
    const id = targetId.value;
    if (id === undefined || id === null) {
        emit('close');
        return;
    }
    const idStr = String(id);
    emit('preset-deleted', idStr);
    emit('deleted', idStr);
    emit('close');
};
</script>
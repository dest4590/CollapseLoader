<template>
    <div class="p-4">
        <h3 class="text-lg font-semibold mb-2">{{ title }}</h3>
        <p class="text-sm text-base-content/70 mb-4">{{ message }}</p>
        <div class="flex justify-end gap-2">
            <button class="btn btn-ghost btn-sm" @click="$emit('cancel')">{{ cancelLabel }}</button>
            <button class="btn btn-danger btn-sm" @click="$emit('confirm')">{{ confirmLabel }}</button>
        </div>
    </div>
</template>

<script setup lang="ts">
const { title, message, confirmLabel, cancelLabel } = defineProps({
    title: { type: String, default: 'Confirm' },
    message: { type: String, default: '' },
    confirmLabel: { type: String, default: 'Confirm' },
    cancelLabel: { type: String, default: 'Cancel' },
});

defineEmits<{
    (e: 'confirm'): void;
    (e: 'cancel'): void;
}>();
</script>

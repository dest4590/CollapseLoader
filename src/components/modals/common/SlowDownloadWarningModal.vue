<template>
    <div class="space-y-3 mb-4">
        <p class="text-md text-base-content/80">
            {{ isStalled ? t('modals.slow_download.stalled_description') : t('modals.slow_download.description') }}
        </p>

        <div v-if="!isStalled" class="bg-base-200 rounded-lg p-3">
            <div class="flex items-center justify-between">
                <span class="text-sm text-base-content/70">{{ t('modals.slow_download.current_speed') }}</span>
                <span class="font-mono text-warning">{{ formattedSpeed }}</span>
            </div>
        </div>

        <div v-else class="bg-error/10 border border-error/20 rounded-lg p-3">
            <div class="flex items-center gap-2">
                <WifiOff class="w-5 h-5 text-error" />
                <span class="text-sm text-error">{{ t('modals.slow_download.no_progress') }}</span>
            </div>
        </div>

        <ul class="text-sm text-base-content/70 space-y-1 list-disc list-inside">
            <li>{{ t('modals.slow_download.tip_1') }}</li>
            <li>{{ t('modals.slow_download.tip_2') }}</li>
            <li>{{ t('modals.slow_download.tip_3') }}</li>
            <li class="text-warning font-medium">{{ t('modals.slow_download.tip_zapret') }}</li>
        </ul>
    </div>

    <div class="flex flex-col sm:flex-row gap-3 justify-end items-center">
        <button @click="handleClose" class="btn btn-primary">
            {{ t('common.ok') }}
        </button>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { WifiOff } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

const props = defineProps<{
    currentSpeed: number;
    isStalled?: boolean;
}>();

const emit = defineEmits(['close']);

const { t } = useI18n();

const formattedSpeed = computed(() => {
    const speed = props.currentSpeed;
    if (speed < 1024) {
        return `${speed.toFixed(0)} B/s`;
    } else if (speed < 1024 * 1024) {
        return `${(speed / 1024).toFixed(1)} KB/s`;
    } else {
        return `${(speed / 1024 / 1024).toFixed(2)} MB/s`;
    }
});

const handleClose = () => {
    emit('close');
};
</script>

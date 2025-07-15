<template>
    <transition name="disclaimer-fade">
        <div v-if="show"
            class="fixed inset-0 bg-black/80 z-[1400] p-9 flex items-center justify-center backdrop-blur-md">
            <transition name="disclaimer-scale">
                <div v-if="show"
                    class="bg-base-200 rounded-xl shadow-2xl border border-base-300 w-full max-w-2xl flex flex-col max-h-[80vh]">
                    <div class="flex flex-col items-center text-center p-6 flex-grow overflow-y-auto">
                        <h2 class="text-2xl font-bold text-primary mb-4">
                            {{ t('modals.telemetry_consent.title') }}
                        </h2>

                        <div class="text-left space-y-4 max-w-xl">
                            <p class="text-base-content/80">
                                {{ t('modals.telemetry_consent.description') }}
                            </p>

                            <div class="bg-base-300 rounded-lg p-4 space-y-3">
                                <h3 class="font-semibold text-primary">
                                    {{
                                        t(
                                            'modals.telemetry_consent.data_collected_title'
                                        )
                                    }}
                                </h3>
                                <ul class="text-sm text-base-content/70 space-y-2">
                                    <li class="flex items-start">
                                        <span class="text-primary mr-2">•</span>
                                        {{
                                            t(
                                                'modals.telemetry_consent.data_collected.performance'
                                            )
                                        }}
                                    </li>
                                    <li class="flex items-start">
                                        <span class="text-primary mr-2">•</span>
                                        {{
                                            t(
                                                'modals.telemetry_consent.data_collected.usage_patterns'
                                            )
                                        }}
                                    </li>
                                    <li class="flex items-start">
                                        <span class="text-primary mr-2">•</span>
                                        {{
                                            t(
                                                'modals.telemetry_consent.data_collected.crash_reports'
                                            )
                                        }}
                                    </li>
                                    <li class="flex items-start">
                                        <span class="text-primary mr-2">•</span>
                                        {{
                                            t(
                                                'modals.telemetry_consent.data_collected.feature_usage'
                                            )
                                        }}
                                    </li>
                                </ul>
                            </div>

                            <div class="bg-info/10 border border-info/20 rounded-lg p-4">
                                <p class="text-sm text-base-content/80">
                                    <span class="font-semibold text-info">{{
                                        t(
                                            'modals.telemetry_consent.privacy_note_title'
                                        )
                                    }}</span>
                                    {{
                                        t(
                                            'modals.telemetry_consent.privacy_note'
                                        )
                                    }}
                                </p>
                            </div>
                        </div>
                    </div>

                    <div class="p-6 border-t border-base-300 flex flex-col sm:flex-row gap-3 justify-end">
                        <button @click="handleDecline" class="btn btn-ghost">
                            {{ t('modals.telemetry_consent.decline') }}
                        </button>
                        <button @click="handleAccept" class="btn btn-primary">
                            {{ t('modals.telemetry_consent.accept') }}
                        </button>
                    </div>
                </div>
            </transition>
        </div>
    </transition>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from '../../services/toastService';

defineProps<{
    show: boolean;
}>();

const emit = defineEmits(['accept', 'decline', 'close']);

const { t } = useI18n();
const { addToast } = useToast();

const handleAccept = async () => {
    try {
        await invoke('set_optional_telemetry', { enabled: true });
        addToast(t('toast.telemetry.analytics_enabled'), 'success');
        emit('accept');
        emit('close');
    } catch (error) {
        console.error('Failed to enable analytics:', error);
        addToast(
            t('toast.telemetry.analytics_enable_failed', { error }),
            'error'
        );
    }
};

const handleDecline = async () => {
    try {
        await invoke('set_optional_telemetry', { enabled: false });
        addToast(t('toast.telemetry.analytics_disabled'), 'info');
        emit('decline');
        emit('close');
    } catch (error) {
        console.error('Failed to disable analytics:', error);
        addToast(
            t('toast.telemetry.analytics_disable_failed', { error }),
            'error'
        );
    }
};
</script>

<style scoped>
.disclaimer-fade-enter-active,
.disclaimer-fade-leave-active {
    transition: opacity 0.3s ease;
}

.disclaimer-fade-enter-from,
.disclaimer-fade-leave-to {
    opacity: 0;
}

.disclaimer-scale-enter-active,
.disclaimer-scale-leave-active {
    transition:
        transform 0.3s ease,
        opacity 0.3s ease;
}

.disclaimer-scale-enter-from,
.disclaimer-scale-leave-to {
    transform: scale(0.95);
    opacity: 0;
}
</style>

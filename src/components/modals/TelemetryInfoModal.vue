<template>
    <div class="mx-auto">
        <!-- same text as TelemetryConsentModal.vue -->
        <div class="bg-base-300 rounded-xl shadow-md p-5 space-y-4">
            <h3 class="font-bold text-lg text-primary">
                {{ t('modals.telemetry_consent.data_collected_title') }}
            </h3>
            <ul class="text-sm text-base-content/70 space-y-3">
                <li class="flex items-start">
                    <span class="text-primary text-lg mr-2.5 leading-5">•</span>
                    <span>{{
                        t('modals.telemetry_consent.data_collected.performance')
                        }}</span>
                </li>
                <li class="flex items-start">
                    <span class="text-primary text-lg mr-2.5 leading-5">•</span>
                    <span>{{
                        t(
                            'modals.telemetry_consent.data_collected.usage_patterns'
                        )
                    }}</span>
                </li>
                <li class="flex items-start">
                    <span class="text-primary text-lg mr-2.5 leading-5">•</span>
                    <span>{{
                        t(
                            'modals.telemetry_consent.data_collected.crash_reports'
                        )
                    }}</span>
                </li>
                <li class="flex items-start">
                    <span class="text-primary text-lg mr-2.5 leading-5">•</span>
                    <span>{{
                        t(
                            'modals.telemetry_consent.data_collected.feature_usage'
                        )
                    }}</span>
                </li>
            </ul>
        </div>

        <div class="flex justify-end mt-8">
            <button @click="$emit('close')"
                class="btn btn-primary px-6 hover:opacity-90 transition-opacity duration-200 flex items-center">
                <CheckIcon class="w-4.5 h-4.5 mr-2.5" />
                {{ $t('common.ok') }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { Check as CheckIcon } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

const emit = defineEmits(['close']);
const { t } = useI18n();
</script>

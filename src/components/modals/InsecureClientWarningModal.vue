<template>
    <div class="bg-warning/10 border border-warning/30 rounded-lg shadow-sm p-3 space-y-2 mb-4">
        <div class="flex items-center gap-2">
            <AlertTriangle class="w-6 h-6 text-warning flex-shrink-0" />
            <div>
                <h3 class="font-semibold text-base text-warning">
                    {{ t('modals.insecure_client_warning.title') }}
                </h3>
                <p class="text-xs text-warning/80">
                    {{ t('modals.insecure_client_warning.subtitle') }}
                </p>
            </div>
        </div>
    </div>

    <div class="space-y-3 mb-4">
        <p class="text-md text-base-content/80">
            {{
                t('modals.insecure_client_warning.description', {
                    clientName: client.name,
                })
            }}
        </p>

        <div class="form-control">
            <label class="label cursor-pointer justify-start gap-3">
                <input type="checkbox" class="checkbox checkbox-sm" v-model="dontShowAgain" />
                <span class="label-text">
                    {{ t('modals.insecure_client_warning.dont_show_again') }}
                </span>
            </label>
        </div>
    </div>

    <div class="flex flex-col sm:flex-row gap-3 justify-end">
        <button @click="handleCancel" class="btn btn-outline order-2 sm:order-1">
            <X class="w-4 h-4 mr-2" />
            {{ t('modals.insecure_client_warning.cancel') }}
        </button>
        <button @click="handleProceed" class="btn btn-warning order-1 sm:order-2">
            <Play class="w-4 h-4 mr-2" />
            {{ t('modals.insecure_client_warning.proceed_anyway') }}
        </button>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { AlertTriangle, X, Play } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

interface Client {
    id: number;
    name: string;
    insecure: boolean;
}

const props = defineProps<{
    client: Client;
}>();

const emit = defineEmits(['proceed', 'cancel', 'close']);

const { t } = useI18n();
const dontShowAgain = ref(false);

const handleProceed = () => {
    emit('proceed', {
        client: props.client,
        dontShowAgain: dontShowAgain.value,
    });
    emit('close');
};

const handleCancel = () => {
    emit('cancel');
    emit('close');
};
</script>

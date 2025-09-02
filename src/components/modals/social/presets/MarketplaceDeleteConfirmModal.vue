<template>
    <div>
        <p class="text-sm text-base-content/70">{{ t('marketplace.delete_confirm') }}</p>
        <div class="flex justify-end gap-2 mt-6">
            <button class="btn" @click="$emit('close')">{{ t('common.cancel') }}</button>
            <button class="btn btn-error" :class="{ loading: deleting }" @click="confirm" :disabled="deleting">{{
                t('common.delete') }}</button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { marketplaceService } from '../../../../services/marketplaceService';
import { useToast } from '../../../../services/toastService';

const props = defineProps<{ id: number }>();
const emit = defineEmits(['deleted', 'close']);

const { t } = useI18n();
const { addToast } = useToast();
const deleting = ref(false);

async function confirm() {
    try {
        deleting.value = true;
        await marketplaceService.deletePreset(props.id);
        addToast(t('marketplace.deleted_success'), 'success');
        emit('deleted');
        emit('close');
    } catch (e) {
        console.error('Failed to delete preset:', e);
        addToast(t('marketplace.deleted_failed'), 'error');
    } finally {
        deleting.value = false;
    }
}
</script>
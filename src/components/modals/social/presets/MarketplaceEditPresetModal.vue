<template>
    <div class="space-y-4">
        <div class="form-control">
            <label class="label"><span class="label-text">{{ t('marketplace.title_label') }}</span></label>
            <input v-model="title" class="input input-bordered w-full"
                :placeholder="t('marketplace.title_placeholder')" />
        </div>
        <div class="form-control">
            <label class="label"><span class="label-text">{{ t('marketplace.description_label') }}</span></label>
            <textarea v-model="description" class="textarea textarea-bordered w-full" rows="4"
                :placeholder="t('marketplace.description_placeholder')"></textarea>
        </div>
        <div class="form-control">
            <label class="cursor-pointer label">
                <span class="label-text">{{ t('marketplace.public_label') }}</span>
                <input type="checkbox" class="toggle" v-model="isPublic" />
            </label>
        </div>
        <div class="flex justify-end gap-2">
            <button class="btn" @click="$emit('close')">{{ t('common.cancel') }}</button>
            <button class="btn btn-primary" :disabled="saving || !title" @click="save">{{ saving ?
                t('common.please_wait') : t('common.save') }}</button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { marketplaceService } from '../../../../services/marketplaceService';
import { useToast } from '../../../../services/toastService';

const props = defineProps<{ preset: any }>();
const emit = defineEmits(['updated', 'close']);

const { t } = useI18n();
const { addToast } = useToast();
const title = ref(props.preset?.title || '');
const description = ref(props.preset?.description || '');
const isPublic = ref(!!props.preset?.is_public);
const saving = ref(false);

async function save() {
    try {
        saving.value = true;
        await marketplaceService.updatePreset(props.preset.id, {
            title: title.value,
            description: description.value,
            is_public: isPublic.value,
        });
        addToast(t('marketplace.updated_success'), 'success');
        emit('updated');
        emit('close');
    } catch (e) {
        console.error('Failed to update preset:', e);
        addToast(t('marketplace.updated_failed'), 'error');
    } finally {
        saving.value = false;
    }
}
</script>
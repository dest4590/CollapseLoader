<template>
  <div class="p-4">
    <h3 class="text-lg font-medium mb-4">{{ title }}</h3>
    <div>
      <AvatarUploader :current-url="currentUrl" @uploaded="onUploaded" />
    </div>
    <div class="mt-4 flex justify-between items-center">
      <div>
        <button class="btn btn-error btn-sm" @click="onResetAvatar">{{ t('account.reset_avatar') }}</button>
      </div>
      <div>
        <button class="btn btn-ghost mr-2" @click="closeModal">{{ t('common.cancel') }}</button>
        <button class="btn btn-primary" @click="closeModal">{{ t('common.done') }}</button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import AvatarUploader from '../../../../components/features/friends/AvatarUploader.vue';
import { userService } from '../../../../services/userService';
import { useToast } from '../../../../services/toastService';

const props = defineProps<{ title?: string; currentUrl?: string }>();
const emit = defineEmits<{
  (e: 'uploaded'): void;
  (e: 'close'): void;
}>();

const { t } = useI18n();

const { addToast } = useToast();

const currentUrl = props.currentUrl || null;

const onUploaded = () => {
  emit('uploaded');
};

async function onResetAvatar() {
  try {
    const { success, error } = await userService.resetAvatar();
    if (!success) throw new Error(error || 'Failed');
    addToast(t('account.avatar_reset_success'), 'success');
    emit('uploaded');
    closeModal();
  } catch (e: any) {
    addToast(e.message || t('account.avatar_reset_failed'), 'error');
  }
}

const closeModal = () => {
  emit('close');
};
</script>

<style scoped>
.avatar-preview {
  max-width: 240px;
}
</style>

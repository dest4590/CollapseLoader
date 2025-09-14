<template>
  <div class="p-4">
    <h3 class="text-lg font-medium mb-4">{{ title }}</h3>
    <div>
      <AvatarUploader :current-url="currentUrl" @uploaded="onUploaded" />
    </div>
    <div class="mt-4 flex justify-end">
      <button class="btn btn-ghost mr-2" @click="closeModal">{{ t('common.cancel') }}</button>
      <button class="btn btn-primary" @click="closeModal">{{ t('common.done') }}</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useI18n } from 'vue-i18n';
import AvatarUploader from '../../../../components/features/profile/AvatarUploader.vue';

const props = defineProps<{ title?: string; currentUrl?: string }>();
const emit = defineEmits<{
  (e: 'uploaded'): void;
  (e: 'close'): void;
}>();

const { t } = useI18n();

const currentUrl = props.currentUrl || null;

const onUploaded = () => {
  emit('uploaded');
};

const closeModal = () => {
  emit('close');
};
</script>

<style scoped>
.avatar-preview {
  max-width: 240px;
}
</style>

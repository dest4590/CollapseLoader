<template>
  <div class="max-w-6xl mx-auto p-2 md:p-4">
    <div class="flex items-center justify-between mb-4">
      <button class="btn btn-ghost btn-sm mr-2" @click="goBack">
        <ArrowLeft class="w-5 h-5" />
      </button>
      <h1 class="text-2xl font-bold">{{ t('marketplace.title') }}</h1>
      <div class="flex gap-2">
        <button class="btn" @click="openShareLocalModal">{{ t('marketplace.share_local') }}</button>
        <button class="btn btn-primary" @click="openShareModal">{{ t('marketplace.share_current') }}</button>
      </div>
    </div>

    <PresetGallery :key="reloadKey" />
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import PresetGallery from '../components/presets/PresetGallery.vue';
import SharePresetModal from '../components/modals/social/presets/SharePresetModal.vue';
import ShareLocalPresetModal from '../components/modals/social/presets/ShareLocalPresetModal.vue';
import { useModal } from '../services/modalService';
import { ArrowLeft } from 'lucide-vue-next';
import { useRouter } from '../services/router';

const { showModal, hideModal } = useModal();
const { t } = useI18n();
const router = useRouter();

const reloadKey = ref(0);

function goBack() {
  router.back();
}
function openShareModal() {
  showModal(
    'share-preset',
    SharePresetModal,
    { title: t('marketplace.share_modal_title') },
    {},
    {
      shared: () => {
        hideModal('share-preset');
        reloadKey.value++;
      },
    },
  );
}

function openShareLocalModal() {
  showModal(
    'share-local-preset',
    ShareLocalPresetModal,
    { title: t('marketplace.share_modal_title') },
    {},
    {
      shared: () => {
        hideModal('share-local-preset');
        reloadKey.value++;
      },
    },
  );
}
</script>

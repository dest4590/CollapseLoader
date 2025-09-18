<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { userService } from '../../../services/userService';
import { useToast } from '../../../services/toastService';
import { useI18n } from 'vue-i18n';
import { apiInvalidateProfile } from '../../../services/apiClient';
import { getCurrentWebview } from '@tauri-apps/api/webview';
import { readFile } from '@tauri-apps/plugin-fs';
import { basename } from '@tauri-apps/api/path';

const props = defineProps<{ currentUrl?: string | null }>();
const emit = defineEmits<{ (e: 'uploaded', url: string | null): void }>();

const { addToast } = useToast();
const { t } = useI18n();

const hovering = ref(false);
const previewUrl = ref<string | null>(props.currentUrl || null);
const fileRef = ref<HTMLInputElement | null>(null);
const uploading = ref(false);
let unlisten: (() => void) | null = null;

const maxSize = 5 * 1024 * 1024;
const allowed = ['image/png', 'image/jpeg', 'image/gif', 'image/webp'];

const helpText = computed(() => t('account.avatar_help'));

function onBrowse() {
  fileRef.value?.click();
}

async function onFileChange(e: Event) {
  const input = e.target as HTMLInputElement;
  const file = input.files?.[0];
  if (file) await handleFile(file);
}

async function handleFile(file: File) {
  if (!allowed.includes(file.type)) {
    addToast(t('account.avatar_type_error'), 'error');
    return;
  }
  if (file.size > maxSize) {
    addToast(t('account.avatar_size_error'), 'error');
    return;
  }

  uploading.value = true;
  try {
    const { success, profile, error } = await userService.uploadAvatar(file);
    if (!success) throw new Error(error || 'Upload failed');
  const url = profile?.avatar_url || null;
  previewUrl.value = url;
  apiInvalidateProfile();
  emit('uploaded', url);
    addToast(t('account.avatar_uploaded'), 'success');
  } catch (err: any) {
    addToast(err.message || t('account.avatar_upload_failed'), 'error');
  } finally {
    uploading.value = false;
  }
}

onMounted(async () => {
  unlisten = await getCurrentWebview().onDragDropEvent(async (event) => {
    if (event.payload.type === 'over') {
      hovering.value = true;
    } else if (event.payload.type === 'drop') {
      hovering.value = false;
      const paths = event.payload.paths;
      if (paths.length > 0) {
        const filePath = paths[0];
        try {
          const data = await readFile(filePath);
          const fileName = await basename(filePath);
          const mimeType = getMimeType(fileName);
          if (!allowed.includes(mimeType)) {
            addToast(t('account.avatar_type_error'), 'error');
            return;
          }
          if (data.length > maxSize) {
            addToast(t('account.avatar_size_error'), 'error');
            return;
          }
          const file = new File([data], fileName, { type: mimeType });
          await handleFile(file);
        } catch (e) {
          console.log("Failed to read dropped file:", e);
          addToast(t('account.avatar_upload_failed'), 'error');
        }
      }
    } else {
      hovering.value = false;
    }
  });
});

onUnmounted(() => {
  if (unlisten) {
    unlisten();
  }
});

function getMimeType(fileName: string): string {
  const ext = fileName.split('.').pop()?.toLowerCase();
  switch (ext) {
    case 'png': return 'image/png';
    case 'jpg':
    case 'jpeg': return 'image/jpeg';
    case 'gif': return 'image/gif';
    case 'webp': return 'image/webp';
    default: return '';
  }
}
</script>
<template>
  <div class="flex items-center space-x-3">
    <div @click="onBrowse" :class="[
      'rounded-xl p-4 flex items-center gap-3 cursor-pointer border-2 border-dashed',
      hovering ? 'border-primary bg-base-200' : 'border-base-300 bg-transparent'
    ]">
      <img v-if="previewUrl" :src="previewUrl" alt="avatar" class="w-8 h-8 rounded-full object-cover" />
      <div class="text-left">
        <div class="text-sm">{{ t('account.avatar_drop') }}</div>
        <div class="text-xs opacity-70">{{ helpText }}</div>
      </div>
    </div>

    <input ref="fileRef" type="file" class="hidden" accept="image/png,image/jpeg,image/gif,image/webp"
      @change="onFileChange" />
  </div>
</template>
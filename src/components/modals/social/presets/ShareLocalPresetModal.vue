<template>
    <div class="space-y-4">
        <div class="form-control">
            <label class="label"><span class="label-text">{{ t('marketplace.preset_picker_label') }}</span></label>
            <select v-model="selectedId" class="select select-bordered w-full">
                <option :value="''" disabled>{{ t('marketplace.preset_picker_placeholder') }}</option>
                <option v-for="p in presets" :key="p.id" :value="p.id">{{ p.name }}</option>
            </select>
            <p v-if="!presetsLoading && presets.length === 0" class="text-sm text-base-content/60 mt-2">
                {{ t('marketplace.no_local_presets') }}
            </p>
        </div>

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
                <span class="label-text block">{{ t('marketplace.public_label') }}</span>
                <input type="checkbox" class="toggle" v-model="isPublic" />
            </label>
        </div>

        <div class="flex justify-end gap-2">
            <button class="btn" @click="$emit('close')">{{ t('common.cancel') }}</button>
            <button class="btn btn-primary" :disabled="submitting || !title || !selectedPreset" @click="share">
                {{ submitting ? t('marketplace.sharing') : t('marketplace.share') }}
            </button>
        </div>
    </div>

</template>

<script setup lang="ts">
import { ref, watch, computed, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import { usePresets } from '../../../../composables/usePresets';
import type { ThemePreset } from '../../../../types/presets';
import { marketplaceService } from '../../../../services/marketplaceService';
import { useToast } from '../../../../services/toastService';

const emit = defineEmits(['shared', 'close']);

const { t } = useI18n();
const { addToast } = useToast();
const { presets, loading: presetsLoading, loadPresets } = usePresets();

const selectedId = ref<string>('');
const title = ref('');
const description = ref('');
const isPublic = ref(true);
const submitting = ref(false);

const selectedPreset = computed<ThemePreset | undefined>(() => presets.value.find(p => p.id === selectedId.value));

watch(selectedPreset, (p) => {
    if (p && !title.value) {
        title.value = p.name;
        if (!description.value && p.description) description.value = p.description;
    }
});

onMounted(() => {
    loadPresets();
});

async function share() {
    if (!selectedPreset.value) return;
    try {
        const s = selectedPreset.value;
        const payload = {
            title: title.value.trim(),
            description: description.value.trim(),
            preset_data: {
                custom_css: s.custom_css,
                enable_custom_css: s.enable_custom_css,
                primary: s.primary,
                base100: s.base100,
                base200: s.base200,
                base300: s.base300,
                base_content: s.base_content,
                primary_content: s.primary_content,
                secondary: s.secondary,
                secondary_content: s.secondary_content,
                accent: s.accent,
                accent_content: s.accent_content,
                neutral: s.neutral,
                neutral_content: s.neutral_content,
                info: s.info,
                info_content: s.info_content,
                success: s.success,
                success_content: s.success_content,
                warning: s.warning,
                warning_content: s.warning_content,
                error: s.error,
                error_content: s.error_content,
            },
            is_public: isPublic.value,
        };

        submitting.value = true;
        await marketplaceService.createPreset(payload);
        addToast(t('marketplace.shared_success'), 'success');
        emit('shared');
        emit('close');
    } catch (e) {
        console.error('Failed to share preset:', e);
        addToast(t('marketplace.shared_failed'), 'error');
    } finally {
        submitting.value = false;
    }
}
</script>

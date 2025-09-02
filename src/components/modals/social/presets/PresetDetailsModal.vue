<template>
    <div class="max-w-4xl">
        <div v-if="loading" class="flex items-center gap-2 text-base-content/70">
            <span class="loading loading-spinner loading-sm"></span>
            <span>{{ t('common.loading') }}</span>
        </div>
        <div v-else-if="preset" class="space-y-4">
            <div class="flex items-start justify-between gap-4 sticky top-0 bg-base-200 z-10 py-2 -mt-2">
                <div class="flex-1 min-w-0">
                    <h2 class="text-xl font-semibold truncate">{{ preset.title }}</h2>
                    <p class="text-xs text-base-content/60 truncate">{{ t('marketplace.by_user', {
                        name:
                            preset.owner_username
                    }) }}</p>
                </div>
                <div class="flex items-center gap-2">
                    <span class="badge badge-ghost">
                        <ThumbsUp class="w-4 h-4" /> {{ preset.likes_count }}
                    </span>
                    <span class="badge badge-ghost">
                        <Download class="w-4 h-4" />
                        {{ preset.downloads_count }}
                    </span>
                    <span v-if="isOwner" class="badge badge-ghost"
                        :class="preset.is_public ? 'text-success' : 'text-warning'">
                        {{ preset.is_public ? t('marketplace.public_label') : t('marketplace.private_label') }}
                    </span>
                    <div v-if="onNavigate" class="ml-2 join mr-4">
                        <button class="btn btn-sm join-item" @click="onNavigate('prev')">
                            <ChevronLeft class="w-4 h-4" />
                        </button>
                        <button class="btn btn-sm join-item" @click="onNavigate('next')">
                            <ChevronRight class="w-4 h-4" />
                        </button>
                    </div>
                </div>

            </div>

            <div class="flex items-center gap-2 ml-2">
                <button class="btn btn-neutral btn-sm" @click="applyFromDetails">{{ t('marketplace.apply')
                }}</button>
                <button class="btn btn-neutral btn-sm" :disabled="downloading" @click="downloadFromDetails">{{
                    t('common.download')
                }}</button>
                <button class="btn btn-neutral btn-sm" :disabled="liking" @click="likeFromDetails">{{
                    t('marketplace.like') }}</button>
                <template v-if="isOwner">
                    <button class="btn btn-neutral btn-sm" @click="openEdit">{{ t('common.edit') }}</button>
                    <button class="btn btn-neutral btn-sm" :disabled="toggling" @click="toggleVisibility">
                        {{ preset.is_public ? t('marketplace.make_private') : t('marketplace.make_public') }}
                    </button>
                    <button class="btn btn-error btn-sm" @click="askDelete">{{ t('common.delete')
                    }}</button>
                </template>
            </div>

            <p class="text-sm whitespace-pre-line">{{ preset.description }}</p>

            <div class="divider my-2"></div>

            <div>
                <h3 class="font-medium mb-2">{{ t('marketplace.preset_details') }}</h3>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
                    <div class="md:col-span-2">
                        <div v-if="displayedColors.length" class="grid grid-cols-3 sm:grid-cols-6 gap-2">
                            <div v-for="c in displayedColors" :key="c" class="tooltip" :data-tip="c">
                                <div class="h-10 rounded-box border border-base-300"
                                    :style="{ backgroundColor: pData[c] }"></div>
                                <div class="text-xs opacity-70 mt-1 truncate">{{ c }}</div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="divider my-2"></div>

            <div>
                <h3 class="font-medium mb-3 flex items-center justify-between">
                    <span>{{ t('marketplace.comments') }}</span>
                    <span class="text-sm opacity-60">{{ comments.length }}</span>
                </h3>

                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div class="md:col-span-2">
                        <div class="space-y-3 max-h-80 overflow-auto pr-1">
                            <div v-if="commentsLoading" class="flex items-center gap-2 text-base-content/70">
                                <span class="loading loading-spinner loading-sm"></span>
                                <span>{{ t('common.loading') }}</span>
                            </div>

                            <div v-else>
                                <div v-if="!comments.length" class="text-sm opacity-60">
                                    {{ t('marketplace.no_comments') }}
                                </div>

                                <div v-for="c in comments" :key="c.id"
                                    class="p-3 rounded-box border border-base-300 bg-base-100 mt-2">
                                    <div class="flex gap-3">
                                        <div
                                            class="w-10 h-10 rounded-full bg-base-300 flex items-center justify-center text-sm font-semibold">
                                            {{ c.author_username ? c.author_username.charAt(0).toUpperCase() : '?' }}
                                        </div>

                                        <div class="flex-1 min-w-0">
                                            <div class="flex items-center justify-between gap-2 text-xs opacity-70">
                                                <div class="truncate">
                                                    <span class="font-medium mr-2">{{ c.author_username }}</span>
                                                    <span class="text-[11px] opacity-60">{{ formatDate(c.created_at)
                                                    }}</span>
                                                </div>
                                                <div v-if="canDelete(c)" class="flex items-center gap-2">
                                                    <button class="btn btn-ghost btn-xs" @click="onDeleteComment(c)"
                                                        aria-label="Delete comment">
                                                        {{ t('common.delete') }}
                                                    </button>
                                                </div>
                                            </div>

                                            <p class="mt-2 text-sm break-words whitespace-pre-wrap">{{ c.text }}</p>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="md:col-span-1">
                        <div class="sticky top-2 bg-base-200 p-3 rounded-box border border-base-300">
                            <label class="label mb-2">
                                <span class="label-text">{{ t('marketplace.add_comment') }}</span>
                            </label>

                            <textarea v-model="newComment" class="textarea textarea-bordered w-full" rows="4"
                                maxlength="1000" :placeholder="t('marketplace.comment_placeholder')"
                                aria-label="Add a comment"></textarea>

                            <div class="mt-2 flex items-center justify-between gap-2">
                                <div class="text-xs opacity-60">{{ newComment.length }}/1000</div>
                                <button class="btn btn-primary" :disabled="!newComment.trim() || creating"
                                    @click="onAddComment" aria-label="Send comment">
                                    <span v-if="creating" class="loading loading-spinner loading-xs"></span>
                                    <span v-else>{{ t('common.send') }}</span>
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

</template>

<script setup lang="ts">
import { onMounted, ref, watch, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { marketplaceService } from '../../../../services/marketplaceService';
import { useUser } from '../../../../composables/useUser';
import { ChevronLeft, ChevronRight, Download, ThumbsUp } from 'lucide-vue-next';
import { presetService } from '../../../../services/presetService';
import { useToast } from '../../../../services/toastService';
import { useModal } from '../../../../services/modalService';
import MarketplaceEditPresetModal from '../presets/MarketplaceEditPresetModal.vue';
import MarketplaceDeleteConfirmModal from '../presets/MarketplaceDeleteConfirmModal.vue';

const props = defineProps<{ id: number; onNavigate?: (dir: 'prev' | 'next') => void }>();
const emit = defineEmits(['close']);

const { t } = useI18n();
const { username } = useUser();
const { addToast } = useToast();
const { showModal, hideModal } = useModal();

const loading = ref(false);
const preset = ref<any | null>(null);
const comments = ref<any[]>([]);
const commentsLoading = ref(false);
const newComment = ref('');
const creating = ref(false);
const liking = ref(false);
const downloading = ref(false);
const toggling = ref(false);

const isOwner = computed(() => !!username.value && preset.value && preset.value.owner_username === username.value);

const pData = computed(() => ({
    base100: preset.value?.preset_data?.base100 ?? preset.value?.base100,
    base200: preset.value?.preset_data?.base200 ?? preset.value?.base200,
    base300: preset.value?.preset_data?.base300 ?? preset.value?.base300,
    base_content: preset.value?.preset_data?.base_content ?? preset.value?.base_content,
    primary: preset.value?.preset_data?.primary ?? preset.value?.primary,
    primary_content: preset.value?.preset_data?.primary_content ?? preset.value?.primary_content,
    secondary: preset.value?.preset_data?.secondary ?? preset.value?.secondary,
    secondary_content: preset.value?.preset_data?.secondary_content ?? preset.value?.secondary_content,
    accent: preset.value?.preset_data?.accent ?? preset.value?.accent,
    accent_content: preset.value?.preset_data?.accent_content ?? preset.value?.accent_content,
    neutral: preset.value?.preset_data?.neutral ?? preset.value?.neutral,
    neutral_content: preset.value?.preset_data?.neutral_content ?? preset.value?.neutral_content,
    info: preset.value?.preset_data?.info ?? preset.value?.info,
    info_content: preset.value?.preset_data?.info_content ?? preset.value?.info_content,
    success: preset.value?.preset_data?.success ?? preset.value?.success,
    success_content: preset.value?.preset_data?.success_content ?? preset.value?.success_content,
    warning: preset.value?.preset_data?.warning ?? preset.value?.warning,
    warning_content: preset.value?.preset_data?.warning_content ?? preset.value?.warning_content,
    error: preset.value?.preset_data?.error ?? preset.value?.error,
    error_content: preset.value?.preset_data?.error_content ?? preset.value?.error_content,
} as Record<string, any>));

const displayedColors = computed(() => {
    const pd = preset.value?.preset_data ?? {};
    const keys = [
        'base100', 'base200', 'base300', 'base_content',
        'primary_content', 'secondary', 'secondary_content', 'accent', 'accent_content',
        'neutral', 'neutral_content', 'info', 'info_content',
        'success', 'success_content', 'warning', 'warning_content', 'error', 'error_content'
    ];
    console.log(pd);

    return keys.filter(k => pd[k] !== undefined && pd[k] !== null && pd[k] !== '');
});

const formatDate = (dateString: string) => {
    try {
        const date = new Date(dateString);
        const day = String(date.getDate()).padStart(2, '0');
        const month = String(date.getMonth() + 1).padStart(2, '0');
        const year = date.getFullYear();
        const hours = String(date.getHours()).padStart(2, '0');
        const minutes = String(date.getMinutes()).padStart(2, '0');

        return `${day}/${month}/${year} ${hours}:${minutes}`;
    } catch (e) {
        console.error('Invalid date string:', dateString, e);
        return 'N/A';
    }
};

function canDelete(c: any): boolean {
    return !!username.value && (c.author_username === username.value || (preset.value && preset.value.owner_username === username.value));
}

async function loadPreset() {
    loading.value = true;
    try {
        preset.value = await marketplaceService.getPreset(props.id);
    } finally {
        loading.value = false;
    }
}

async function loadComments() {
    commentsLoading.value = true;
    try {
        comments.value = await marketplaceService.listComments(props.id);
    } finally {
        commentsLoading.value = false;
    }
}

async function onAddComment() {
    const text = newComment.value.trim();
    if (!text) return;
    creating.value = true;
    try {
        await marketplaceService.addComment(props.id, text);
        newComment.value = '';
        await loadComments();
    } finally {
        creating.value = false;
    }
}

async function onDeleteComment(c: any) {
    try {
        await marketplaceService.deleteComment(props.id, c.id);
        await loadComments();
    } catch (e) {
        console.error('Failed to delete comment:', e);
    }
}

function applyFromDetails() {
    const p: any = preset.value;
    if (!p) return;
    const payload = p.preset_data || p;
    presetService.applyPresetToTheme({
        custom_css: payload.custom_css,
        enable_custom_css: payload.enable_custom_css,
        base100: payload.base100,
        base200: payload.base200,
        base300: payload.base300,
        base_content: payload.base_content,
        primary: payload.primary,
        primary_content: payload.primary_content,
        secondary: payload.secondary,
        secondary_content: payload.secondary_content,
        accent: payload.accent,
        accent_content: payload.accent_content,
        neutral: payload.neutral,
        neutral_content: payload.neutral_content,
        info: payload.info,
        info_content: payload.info_content,
        success: payload.success,
        success_content: payload.success_content,
        warning: payload.warning,
        warning_content: payload.warning_content,
        error: payload.error,
        error_content: payload.error_content,
    } as any);
}

async function likeFromDetails() {
    if (!preset.value || liking.value) return;
    liking.value = true;
    try {
        await marketplaceService.likePreset(preset.value.id);
        preset.value.likes_count = (preset.value.likes_count || 0) + 1;
    } catch (e) {
        console.error('Failed to like preset:', e);
    } finally {
        liking.value = false;
    }
}

async function downloadFromDetails() {
    if (downloading.value) return;
    try {
        downloading.value = true;
        const p: any = preset.value;
        if (!p) return;
        const payload = p.preset_data || p;
        const input: any = {
            name: p.title || 'Imported preset',
            description: p.description || undefined,
            custom_css: payload.custom_css,
            enable_custom_css: payload.enable_custom_css,
            primary: payload.primary,
            base100: payload.base100,
            base200: payload.base200,
            base300: payload.base300,
            base_content: payload.base_content,
            primary_content: payload.primary_content,
            secondary: payload.secondary,
            secondary_content: payload.secondary_content,
            accent: payload.accent,
            accent_content: payload.accent_content,
            neutral: payload.neutral,
            neutral_content: payload.neutral_content,
            info: payload.info,
            info_content: payload.info_content,
            success: payload.success,
            success_content: payload.success_content,
            warning: payload.warning,
            warning_content: payload.warning_content,
            error: payload.error,
            error_content: payload.error_content,
        };
        await presetService.createPreset(input);
        addToast(t('theme.presets.messages.import_success'), 'success');
        try {
            await marketplaceService.downloadPreset(p.id);
            preset.value.downloads_count = (preset.value.downloads_count || 0) + 1;
        } catch (e) {
            console.error('Failed to record download:', e);
        }
    } catch (e) {
        console.error('Failed to import preset:', e);
        addToast(t('theme.presets.messages.import_error'), 'error');
    } finally {
        downloading.value = false;
    }
}

function openEdit() {
    if (!preset.value) return;
    const id = `edit-preset-${preset.value.id}`;
    showModal(
        id,
        MarketplaceEditPresetModal,
        { title: t('marketplace.edit_modal_title') },
        { preset: preset.value },
        {
            updated: async () => {
                await loadPreset();
                hideModal(id);
            },
        }
    );
}

async function toggleVisibility() {
    if (!preset.value || toggling.value) return;
    const prev = preset.value.is_public;
    preset.value.is_public = !prev;
    toggling.value = true;
    try {
        await marketplaceService.updatePreset(preset.value.id, { is_public: preset.value.is_public });
    } catch (e) {
        preset.value.is_public = prev;
        console.error('Failed to update preset visibility:', e);
        addToast(t('marketplace.updated_failed'), 'error');
    } finally {
        toggling.value = false;
    }
}

function askDelete() {
    if (!preset.value) return;
    const cid = `delete-preset-${preset.value.id}`;
    showModal(
        cid,
        MarketplaceDeleteConfirmModal,
        { title: t('common.delete') },
        { id: preset.value.id },
        {
            deleted: async () => {
                hideModal(cid);
                emit('close');
            },
        }
    );
}

watch(() => props.id, async () => {
    if (props.id) {
        await Promise.all([loadPreset(), loadComments()]);
    }
}, { immediate: true });

onMounted(async () => {
    if (props.id) {
        await Promise.all([loadPreset(), loadComments()]);
    }
});
</script>

<style scoped></style>

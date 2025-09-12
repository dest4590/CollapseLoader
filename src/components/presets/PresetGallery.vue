<template>
    <div class="preset-gallery">
        <div v-if="loading" class="flex items-center gap-2 text-base-content/70">
            <span class="loading loading-spinner loading-sm"></span>
            <span>{{ t('common.loading') }}</span>
        </div>
        <div v-else>
            <div class="mb-4">
                <input type="text" class="input input-bordered w-full"
                    :placeholder="t('marketplace.search_placeholder')" v-model="search" />
            </div>
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                <div class="card bg-base-200 border border-base-300 shadow-sm hover:shadow" v-for="p in filteredPresets"
                    :key="p.id">
                    <div class="card-body p-4">
                        <div class="cursor-pointer rounded-box hover:bg-base-300/20 transition-colors p-1 -m-1"
                            @click="openDetails(p)">
                            <h4 class="card-title text-lg">
                                {{ p.title }}
                            </h4>
                            <p class="text-xs text-base-content/70">
                                {{ t('marketplace.by_user', { name: p.owner_username }) }}
                            </p>
                            <p class="text-sm text-base-content/80 line-clamp-3 mt-2" v-if="p.description">{{ p.description }}</p>
                            <div class="mt-2 flex items-center gap-3 text-xs text-base-content/60">
                                <span class="badge badge-ghost">
                                    <ThumbsUp class="w-4 h-4" />
                                    {{ p.likes_count }}
                                </span>
                                <span class="badge badge-ghost">
                                    <Download class="w-4 h-4" />
                                    {{ p.downloads_count }}
                                </span>
                                <span v-if="isOwner(p)" class="badge"
                                    :class="p.is_public ? 'badge-success' : 'badge-warning'">
                                    {{ p.is_public ? t('marketplace.public_label') : t('marketplace.private_label') }}
                                </span>
                            </div>
                        </div>
                        <div class="card-actions justify-end mt-3">
                            <button class="btn btn-primary btn-sm" @click.stop="apply(p)">{{ t('marketplace.apply')
                                }}</button>
                            <button class="btn btn-primary btn-sm" @click.stop="download(p)">{{ t('common.download')
                                }}</button>
                            <button class="btn btn-secondary btn-sm" @click.stop="like(p)">{{ t('marketplace.like')
                                }}</button>
                            <button class="btn btn-secondary btn-sm" @click.stop="openDetails(p)">{{ t('common.details')
                                }}</button>
                            <template v-if="isOwner(p)">
                                <button class="btn btn-neutral btn-sm" @click.stop="openEdit(p)">{{ t('common.edit')
                                    }}</button>
                                <button class="btn btn-neutral btn-sm" @click.stop="toggleVisibility(p)">
                                    {{ p.is_public ? t('marketplace.make_private') : t('marketplace.make_public') }}
                                </button>
                                <button class="btn btn-error btn-sm" @click.stop="askDelete(p)">{{
                                    t('common.delete') }}</button>
                            </template>
                        </div>
                    </div>
                </div>
            </div>
            <div v-if="!filteredPresets.length" class="text-sm text-base-content/60 mt-4">{{ t('marketplace.no_items')
            }}</div>
        </div>
    </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { marketplaceService } from '../../services/marketplaceService';
import { presetService } from '../../services/presetService';
import { useUser } from '../../composables/useUser';
import { useToast } from '../../services/toastService';
import { useModal } from '../../services/modalService';
import MarketplaceEditPresetModal from '../modals/social/presets/MarketplaceEditPresetModal.vue';
import PresetDetailsModal from '../modals/social/presets/PresetDetailsModal.vue';
import MarketplaceDeleteConfirmModal from '../modals/social/presets/DeletePresetConfirmModal.vue';
import { Download, ThumbsUp } from 'lucide-vue-next';

export default defineComponent({
    name: 'PresetGallery',
    components: { Download, ThumbsUp },
    props: {
        ownerId: { type: Number, required: false }
    },
    setup(props) {
        const presets = ref<any[]>([]);
        const loading = ref(true);
        const search = ref('');
        const { t } = useI18n();
        const { username } = useUser();
        const { addToast } = useToast();

        const { showModal, hideModal } = useModal();

        async function load() {
            loading.value = true;
            try {
                const params: any = {};
                if (props.ownerId) params.owner = props.ownerId;
                const data = await marketplaceService.listPresets(params);
                presets.value = (data || []).map((it: any) => ({
                    liked_by_user: it.liked_by_user ?? false,
                    liking: false,
                    ...it,
                }));
            } finally {
                loading.value = false;
            }
        }

        const filteredPresets = computed(() => {
            const q = search.value.trim().toLowerCase();
            if (!q) return presets.value;
            return presets.value.filter((p: any) => {
                const parts = [p.title, p.description, p.owner_username]
                    .filter(Boolean)
                    .map((v: string) => String(v).toLowerCase());
                return parts.some((text: string) => text.includes(q));
            });
        });

        function apply(p: any) {
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

        async function like(p: any) {
            if (!p || p.liking) return;
            p.liking = true;
            try {
                if (p.liked_by_user) {
                    await marketplaceService.unlikePreset(p.id);
                    p.likes_count = Math.max(0, (p.likes_count || 0) - 1);
                    p.liked_by_user = false;
                } else {
                    await marketplaceService.likePreset(p.id);
                    p.likes_count = (p.likes_count || 0) + 1;
                    p.liked_by_user = true;
                }
            } catch (e) {
                console.error("Failed to toggle like preset:", e);
            } finally {
                p.liking = false;
            }
        }

        async function download(p: any) {
            try {
                const payload = p.preset_data || p;
                const input = {
                    name: p.title || 'Imported preset',
                    description: p.description || undefined,
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
                } as any;
                await presetService.createPreset(input);
                addToast(t('theme.presets.messages.import_success'), 'success');
                try {
                    await marketplaceService.downloadPreset(p.id);
                    p.downloads_count = (p.downloads_count || 0) + 1;
                } catch (e) {
                    console.error("Failed to record preset download:", e);
                }
            } catch (e) {
                console.error("Failed to import preset:", e);
                addToast(t('theme.presets.messages.import_error'), 'error');
            }
        }

        function isOwner(p: any): boolean {
            return !!username.value && p.owner_username === username.value;
        }

        function openEdit(p: any) {
            const id = `edit-preset-${p.id}`;
            showModal(
                id,
                MarketplaceEditPresetModal,
                { title: t('marketplace.edit_modal_title') },
                { preset: p },
                {
                    updated: async () => {
                        await load();
                        hideModal(id);
                    },
                }
            );
        }

        function openDetails(p: any) {
            const id = `preset-details-${p.id}`;
            showModal(
                id,
                PresetDetailsModal,
                { title: t('marketplace.preset_details_title'), contentClass: "wide" },
                { id: p.id, onNavigate: (dir: 'prev' | 'next') => navigateFrom(p.id, dir) },
                {}
            );
        }

        function navigateFrom(currentId: number, dir: 'prev' | 'next') {
            const list = filteredPresets.value;
            const idx = list.findIndex((x: any) => x.id === currentId);
            if (idx === -1) return;
            const nextIdx = dir === 'next' ? (idx + 1) % list.length : (idx - 1 + list.length) % list.length;
            const next = list[nextIdx];
            const oldId = `preset-details-${currentId}`;
            hideModal(oldId);
            const newId = `preset-details-${next.id}`;
            showModal(
                newId,
                PresetDetailsModal,
                { title: t('marketplace.preset_details_title'), contentClass: "wide" },
                { id: next.id, onNavigate: (d: 'prev' | 'next') => navigateFrom(next.id, d) },
                {}
            );
        }

        async function toggleVisibility(p: any) {
            const prev = p.is_public;
            p.is_public = !p.is_public;
            try {
                await marketplaceService.updatePreset(p.id, { is_public: p.is_public });
            } catch (e) {
                p.is_public = prev;
                addToast(t('marketplace.updated_failed'), 'error');
                console.error("Failed to update preset visibility:", e);
            }
        }

        function askDelete(p: any) {
            const id = `delete-preset-${p.id}`;
            showModal(
                id,
                MarketplaceDeleteConfirmModal,
                { title: t('common.delete') },
                { id: p.id },
                {
                    deleted: async () => {
                        const list = presets.value;
                        presets.value = list.filter(x => x.id !== p.id);
                        hideModal(id);
                    },
                }
            );
        }

        onMounted(load);

        return {
            presets,
            loading,
            search,
            filteredPresets,
            apply,
            like,
            download,
            isOwner,
            openEdit,
            openDetails,
            navigateFrom,
            toggleVisibility,
            askDelete,
            t,

        };
    }
});
</script>
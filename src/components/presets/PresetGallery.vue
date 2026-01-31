<template>
    <div class="preset-gallery">
        <div v-if="loading" class="flex items-center gap-2 text-base-content/70">
            <span class="loading loading-spinner loading-sm"></span>
            <span>{{ t('common.loading') }}</span>
        </div>
        <div v-else>
            <div class="flex flex-col md:flex-row gap-4 mb-4">
                <div class="relative flex-1">
                    <Search class="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-white/40" />
                    <input type="text" class="input input-bordered w-full pl-10 bg-white/5 border-white/10 focus:border-white/20 transition-all"
                        :placeholder="t('marketplace.search_placeholder')" v-model="search" @input="debouncedLoad" />
                </div>
                <div class="flex gap-2">
                    <select class="select select-bordered bg-white/5 border-white/10" v-model="sortBy" @change="load">
                        <option value="newest">{{ t('marketplace.sort_newest') }}</option>
                        <option value="popular">{{ t('marketplace.sort_popular') }}</option>
                        <option value="downloads">{{ t('marketplace.sort_downloads') }}</option>
                        <option value="comments">{{ t('marketplace.sort_comments') }}</option>
                    </select>
                </div>
            </div>
            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                <div class="card bg-base-100 border border-white/5 shadow-xl hover:bg-base-300 transition-all duration-300 rounded-xl group" 
                     v-for="p in filteredPresets" :key="p.id">
                    <div class="card-body p-3.5">
                        <div class="cursor-pointer" @click="openDetails(p)">
                            <h4 class="text-lg font-bold text-white tracking-tight mb-0">
                                {{ p.title ?? p.name }}
                            </h4>
                            <p class="text-[10px] font-black text-white/30 uppercase tracking-widest mb-2 hover:text-white/50 transition-colors cursor-pointer"
                                @click.stop="$emit('show-user-profile', p.author?.id)">
                                {{ t('marketplace.by_author', { name: getOwnerName(p).toUpperCase() || '' }) }}
                            </p>

                            <PresetColorPreview :preset="p" class="mt-2!" />

                            <p class="text-xs text-white/50 line-clamp-2 mt-3 leading-snug min-h-8" v-if="p.description">
                                {{ p.description }}
                            </p>

                            <div class="mt-4 flex items-center gap-4 text-white/30">
                                <span class="flex items-center gap-1.5 hover:text-white/50 transition-colors">
                                    <ThumbsUp class="w-3.5 h-3.5" />
                                    <span class="text-[11px] font-bold">{{ p.likes_count || 0 }}</span>
                                </span>
                                <span class="flex items-center gap-1.5 hover:text-white/50 transition-colors">
                                    <Download class="w-3.5 h-3.5" />
                                    <span class="text-[11px] font-bold">{{ p.downloads_count || 0 }}</span>
                                </span>
                                <span class="flex items-center gap-1.5 hover:text-white/50 transition-colors">
                                    <MessageSquare class="w-3.5 h-3.5" />
                                    <span class="text-[11px] font-bold">{{ p.comments_count || 0 }}</span>
                                </span>
                            </div>
                        </div>

                        <div class="card-actions justify-end mt-2">
                            <div class="flex items-center gap-1.5">
                                <template v-if="isOwner(p)">
                                    <div class="dropdown dropdown-end dropdown-top">
                                        <button tabindex="0" class="btn btn-circle btn-xs btn-ghost hover:bg-white/10" @click.stop>
                                            <MoreVertical class="w-3.5 h-3.5 text-white/30" />
                                        </button>
                                        <ul tabindex="0" class="dropdown-content z-10 menu p-1.5 shadow-2xl bg-[#1a1a1a] rounded-xl w-48 border border-white/10 mb-2">
                                            <li><a @click.stop="openEdit(p)" class="hover:bg-white/5 py-2 text-sm">
                                                <Edit class="w-3.5 h-3.5" /> {{ t('common.edit') }}
                                            </a></li>
                                            <li><a @click.stop="toggleVisibility(p)" class="hover:bg-white/5 py-2 text-sm">
                                                <component :is="p.is_public ? 'EyeOff' : 'Eye'" class="w-3.5 h-3.5" />
                                                {{ p.is_public ? t('marketplace.make_private') : t('marketplace.make_public') }}
                                            </a></li>
                                            <li><a @click.stop="askDelete(p)" class="text-error hover:bg-error/10 py-2 text-sm">
                                                <Trash2 class="w-3.5 h-3.5" /> {{ t('common.delete') }}
                                            </a></li>
                                        </ul>
                                    </div>
                                </template>
                                
                                <button class="btn btn-circle btn-sm btn-primary bg-white hover:bg-white/90 border-none group/apply" 
                                    @click.stop="apply(p)">
                                    <Play class="w-4 h-4 text-black fill-black" />
                                </button>
                                <button class="btn btn-circle btn-sm bg-white/10 hover:bg-white/20 border-none" 
                                    @click.stop="download(p)">
                                    <Download class="w-4 h-4 text-white" />
                                </button>
                                <button class="btn btn-circle btn-sm bg-white/10 hover:bg-white/20 border-none"
                                    :class="{ 'bg-primary/20 text-primary': p.liked }"
                                    @click.stop="like(p)">
                                    <ThumbsUp class="w-4 h-4" :class="{ 'fill-current': p.liked }" />
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
        <div v-if="!filteredPresets.length" class="text-sm text-base-content/60 mt-4">{{ t('marketplace.no_items')
            }}</div>
    </div>
</template>

<script lang="ts">
import { defineComponent, ref, onMounted, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { marketplaceService } from '../../services/marketplaceService';
import type { MarketplacePreset, MarketplaceTheme } from '../../types/presets';
import { presetService } from '../../services/presetService';
import { useUser } from '../../composables/useUser';
import { useToast } from '../../services/toastService';
import { useModal } from '../../services/modalService';
import MarketplaceEditPresetModal from '../modals/social/presets/MarketplaceEditPresetModal.vue';
import PresetDetailsModal from '../modals/social/presets/PresetDetailsModal.vue';
import MarketplaceDeleteConfirmModal from '../modals/social/presets/MarketplaceDeleteConfirmModal.vue';
import { Download, ThumbsUp, MessageSquare, Search, Play, MoreVertical, Edit, Trash2, Eye, EyeOff } from 'lucide-vue-next';
import { buildPresetCreatePayload } from '../../utils/presetPayload';
import PresetColorPreview from './PresetColorPreview.vue';

type MarketplacePresetView = MarketplacePreset & {
    liking?: boolean;
};

let debounceTimer: any = null;

function getThemeValues(preset: MarketplacePreset): MarketplaceTheme {
    return preset.theme || {};
}

export default defineComponent({
    name: 'PresetGallery',
    components: { Download, ThumbsUp, MessageSquare, Search, Play, MoreVertical, Edit, Trash2, Eye, EyeOff, PresetColorPreview },
    props: {
        ownerId: { type: Number, required: false },
        initialPresets: { type: Array as () => MarketplacePresetView[], required: false }
    },
    emits: ['show-user-profile'],
    setup(props, { emit }) {
        const presets = ref<MarketplacePresetView[]>([]);
        const loading = ref(true);
        const search = ref('');
        const { t } = useI18n();
        const { username } = useUser();
        const { addToast } = useToast();

        const { showModal, hideModal } = useModal();

        const sortBy = ref('newest');

        async function load() {
            loading.value = true;
            try {
                if (props.initialPresets && !search.value.trim() && sortBy.value === 'newest' && presets.value.length === 0) {
                     presets.value = props.initialPresets.map((preset) => ({ ...preset, liking: false }));
                     loading.value = false;
                     return;
                }

                const params: any = {};
                if (props.ownerId) params.owner = props.ownerId;
                if (search.value.trim()) params.q = search.value.trim();
                params.sort = sortBy.value;

                const data = await marketplaceService.listPresets(params);
                presets.value = data.map((preset) => ({ ...preset, liking: false }));
            } finally {
                loading.value = false;
            }
        }

        function debouncedLoad() {
            if (debounceTimer) clearTimeout(debounceTimer);
            debounceTimer = setTimeout(() => {
                load();
            }, 300);
        }

        const filteredPresets = computed(() => presets.value);

        function getOwnerName(p: MarketplacePreset): string {
            return p.author?.displayName ?? p.author?.username ?? p.owner_username ?? '';
        }

        function apply(p: MarketplacePreset) {
            const theme = getThemeValues(p);
            presetService.applyPresetToTheme({
                customCSS: theme.customCSS ?? '',
                enableCustomCSS: theme.enableCustomCSS ?? false,
                base100: theme.base100,
                base200: theme.base200,
                base300: theme.base300,
                baseContent: theme.baseContent,
                primary: theme.primary,
                primaryContent: theme.primaryContent,
                secondary: theme.secondary,
                secondaryContent: theme.secondaryContent,
                accent: theme.accent,
                accentContent: theme.accentContent,
                neutral: theme.neutral,
                neutralContent: theme.neutralContent,
                info: theme.info,
                infoContent: theme.infoContent,
                success: theme.success,
                successContent: theme.successContent,
                warning: theme.warning,
                warningContent: theme.warningContent,
                error: theme.error,
                errorContent: theme.errorContent,
            } as any);
        }

        async function like(p: MarketplacePresetView) {
            if (!p || p.liking) return;
            p.liking = true;
            try {
                if (p.liked) {
                    await marketplaceService.unlikePreset(p.id);
                    p.likes_count = Math.max(0, (p.likes_count || 0) - 1);
                    p.liked = false;
                } else {
                    await marketplaceService.likePreset(p.id);
                    p.likes_count = (p.likes_count || 0) + 1;
                    p.liked = true;
                }
            } catch (e) {
                console.error('Failed to toggle like preset:', e);
            } finally {
                p.liking = false;
            }
        }

        async function download(p: MarketplacePreset) {
            try {
                const name = p.title ?? p.name ?? 'Imported preset';
                const input = buildPresetCreatePayload(name, p.description, getThemeValues(p));
                await presetService.createPreset(input);
                addToast(t('theme.presets.messages.import_success', { name }), 'success');
                try {
                    await marketplaceService.downloadPreset(p.id);
                    p.downloads_count = (p.downloads_count || 0) + 1;
                } catch (e) {
                    console.error('Failed to record preset download:', e);
                }
            } catch (e) {
                console.error('Failed to import preset:', e);
                addToast(t('theme.presets.messages.import_error'), 'error');
            }
        }

        function isOwner(p: MarketplacePreset): boolean {
            const owner = p.author?.username ?? p.owner_username ?? '';
            return !!username.value && owner === username.value;
        }

        function openEdit(p: MarketplacePreset) {
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

        function openDetails(p: MarketplacePreset) {
            const id = `preset-details-${p.id}`;
            showModal(
                id,
                PresetDetailsModal,
                { title: t('marketplace.preset_details_title'), contentClass: 'wide' },
                { id: p.id, onNavigate: (dir: 'prev' | 'next') => navigateFrom(p.id, dir) },
                {
                    'show-user-profile': (userId: number) => {
                        hideModal(id);
                        emit('show-user-profile', userId);
                    }
                }
            );
        }

        function navigateFrom(currentId: number | string, dir: 'prev' | 'next') {
            const list = filteredPresets.value;
            const idx = list.findIndex((x) => x.id === currentId);
            if (idx === -1 || !list.length) return;
            const nextIdx = dir === 'next' ? (idx + 1) % list.length : (idx - 1 + list.length) % list.length;
            const next = list[nextIdx];
            const oldId = `preset-details-${currentId}`;
            hideModal(oldId);
            const newId = `preset-details-${next.id}`;
            showModal(
                newId,
                PresetDetailsModal,
                { title: t('marketplace.preset_details_title'), contentClass: 'wide' },
                { id: next.id, onNavigate: (d: 'prev' | 'next') => navigateFrom(next.id, d) },
                {
                    'show-user-profile': (userId: number) => {
                        hideModal(newId);
                        emit('show-user-profile', userId);
                    }
                }
            );
        }

        async function toggleVisibility(p: MarketplacePreset) {
            const prev = p.is_public;
            p.is_public = !p.is_public;
            try {
                await marketplaceService.updatePreset(p.id, { is_public: p.is_public });
            } catch (e) {
                p.is_public = prev;
                addToast(t('marketplace.updated_failed'), 'error');
                console.error('Failed to update preset visibility:', e);
            }
        }

        function askDelete(p: MarketplacePreset) {
            const id = `delete-preset-${p.id}`;
            showModal(
                id,
                MarketplaceDeleteConfirmModal,
                { title: t('common.delete') },
                { id: p.id },
                {
                    deleted: async () => {
                        const list = presets.value;
                        presets.value = list.filter((x) => x.id !== p.id);
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
            sortBy,
            debouncedLoad,
            getOwnerName,
            load,
        };
    }
});
</script>

<template>
    <div class="max-w-4xl">
        <div
            v-if="loading"
            class="flex items-center gap-2 text-base-content/70"
        >
            <span class="loading loading-spinner loading-sm"></span>
            <span>{{ t("common.loading") }}</span>
        </div>
        <div v-else-if="preset" class="space-y-4">
            <div
                class="flex items-start justify-between gap-4 sticky top-0 bg-base-200 z-10 py-2 -mt-2"
            >
                <div class="flex-1 min-w-0">
                    <h2 class="text-xl font-semibold truncate">
                        {{ preset.title ?? preset.name }}
                    </h2>
                    <div class="flex items-center gap-2 mt-1">
                        <div
                            v-if="
                                preset.author?.author_avatar ||
                                preset.author?.avatar
                            "
                            class="w-5 h-5 rounded-full overflow-hidden border border-white/10 shrink-0 cursor-pointer"
                            @click="
                                emit('show-user-profile', preset.author?.id)
                            "
                        >
                            <img
                                :src="
                                    resolveApiAssetUrl(
                                        preset.author.author_avatar ||
                                            preset.author.avatar
                                    )
                                "
                                class="w-full h-full object-cover"
                                :alt="ownerDisplayName"
                            />
                        </div>
                        <p
                            class="text-xs text-base-content/60 truncate cursor-pointer hover:text-base-content/80 transition-colors"
                            @click="
                                emit('show-user-profile', preset.author?.id)
                            "
                        >
                            {{
                                t("marketplace.by_user", {
                                    name: ownerDisplayName,
                                })
                            }}
                        </p>
                    </div>
                </div>
                <div class="flex items-center gap-2">
                    <span class="badge badge-ghost">
                        <ThumbsUp class="w-4 h-4" /> {{ preset.likes_count }}
                    </span>
                    <span class="badge badge-ghost">
                        <Download class="w-4 h-4" />
                        {{ preset.downloads_count }}
                    </span>
                    <span
                        class="badge badge-ghost hover:bg-base-300 transition-colors cursor-pointer"
                        @click="scrollToComments"
                    >
                        <MessageSquare class="w-4 h-4" />
                        {{ preset.comments_count || 0 }}
                    </span>
                    <span
                        v-if="isOwner"
                        class="badge badge-ghost"
                        :class="
                            preset.is_public ? 'text-success' : 'text-warning'
                        "
                    >
                        {{
                            preset.is_public
                                ? t("marketplace.public_label")
                                : t("marketplace.private_label")
                        }}
                    </span>
                    <div v-if="onNavigate" class="ml-2 join mr-4">
                        <button
                            class="btn btn-sm join-item"
                            @click="onNavigate('prev')"
                        >
                            <ChevronLeft class="w-4 h-4" />
                        </button>
                        <button
                            class="btn btn-sm join-item"
                            @click="onNavigate('next')"
                        >
                            <ChevronRight class="w-4 h-4" />
                        </button>
                    </div>
                </div>
            </div>

            <div class="flex items-center gap-2 ml-2">
                <button
                    class="btn btn-neutral btn-sm"
                    @click="applyFromDetails"
                >
                    {{ t("marketplace.apply") }}
                </button>
                <button
                    class="btn btn-neutral btn-sm"
                    :disabled="downloading"
                    @click="downloadFromDetails"
                >
                    {{ t("common.download") }}
                </button>
                <button
                    class="btn btn-neutral btn-sm"
                    :disabled="preset?.liking"
                    @click="likeFromDetails"
                >
                    {{ t("marketplace.like") }}
                </button>
                <template v-if="isOwner">
                    <button class="btn btn-neutral btn-sm" @click="openEdit">
                        {{ t("common.edit") }}
                    </button>
                    <button
                        class="btn btn-neutral btn-sm"
                        :disabled="toggling"
                        @click="toggleVisibility"
                    >
                        {{
                            preset.is_public
                                ? t("marketplace.make_private")
                                : t("marketplace.make_public")
                        }}
                    </button>
                    <button class="btn btn-error btn-sm" @click="askDelete">
                        {{ t("common.delete") }}
                    </button>
                </template>
            </div>

            <p class="text-sm whitespace-pre-line">{{ preset.description }}</p>

            <div v-if="themeSource.backgroundImage" class="space-y-2">
                <h3 class="font-medium flex items-center gap-2">
                    <ImageIcon class="w-4 h-4" />
                    {{ t("customization.background_image") }}
                </h3>
                <div
                    class="w-full h-48 rounded-xl bg-cover bg-center border border-white/5"
                    :style="{
                        backgroundImage: `url(${themeSource.backgroundImage})`,
                    }"
                ></div>
                <div class="flex gap-4 text-xs opacity-60">
                    <span v-if="themeSource.backgroundBlur !== undefined"
                        >{{ t("customization.background_blur") }}:
                        {{ themeSource.backgroundBlur }}px</span
                    >
                    <span v-if="themeSource.backgroundOpacity !== undefined"
                        >{{ t("customization.background_opacity") }}:
                        {{ themeSource.backgroundOpacity }}%</span
                    >
                </div>
            </div>

            <div class="divider my-2"></div>

            <div>
                <h3 class="font-medium mb-2">
                    {{ t("marketplace.preset_details") }}
                </h3>
                <div class="grid grid-cols-1 md:grid-cols-3 gap-4 text-sm">
                    <div class="md:col-span-2">
                        <div
                            v-if="displayedColors.length"
                            class="grid grid-cols-3 sm:grid-cols-6 gap-2"
                        >
                            <div
                                v-for="c in displayedColors"
                                :key="c"
                                class="tooltip"
                                :data-tip="c"
                            >
                                <div
                                    class="h-10 rounded-box border border-base-300 cursor-pointer"
                                    :style="{ backgroundColor: pData[c] }"
                                    role="button"
                                    tabindex="0"
                                    :aria-label="`Copy ${c} color ${pData[c]}`"
                                    @click="copyColor(c)"
                                    @keydown.enter.prevent="copyColor(c)"
                                    @keydown.space.prevent="copyColor(c)"
                                ></div>
                                <div class="text-xs opacity-70 mt-1 truncate">
                                    {{ c }}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="divider my-2"></div>

            <div id="preset-comments-section">
                <h3 class="font-medium mb-3 flex items-center justify-between">
                    <span>{{ t("marketplace.comments") }}</span>
                    <span class="text-sm opacity-60">{{
                        comments.length
                    }}</span>
                </h3>

                <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                    <div class="md:col-span-2">
                        <div class="space-y-3 max-h-80 overflow-auto pr-1">
                            <div
                                v-if="commentsLoading"
                                class="flex items-center gap-2 text-base-content/70"
                            >
                                <span
                                    class="loading loading-spinner loading-sm"
                                ></span>
                                <span>{{ t("common.loading") }}</span>
                            </div>

                            <div v-else>
                                <div
                                    v-if="!comments.length"
                                    class="text-sm opacity-60"
                                >
                                    {{ t("marketplace.no_comments") }}
                                </div>

                                <div
                                    v-for="c in comments"
                                    :key="c.id"
                                    class="p-3 rounded-xl border border-white/5 bg-white/5 mt-2"
                                >
                                    <div class="flex gap-3">
                                        <div
                                            class="w-8 h-8 rounded-full bg-white/10 flex items-center justify-center overflow-hidden text-xs font-bold text-white/80 shrink-0 cursor-pointer hover:bg-white/20 transition-colors"
                                            @click="
                                                emit(
                                                    'show-user-profile',
                                                    c.author_id || c.authorId
                                                )
                                            "
                                        >
                                            <img
                                                v-if="
                                                    c.author_avatar || c.avatar
                                                "
                                                :src="
                                                    resolveApiAssetUrl(
                                                        c.author_avatar ||
                                                            c.avatar
                                                    )
                                                "
                                                class="w-full h-full object-cover"
                                                :alt="
                                                    c.author_username ||
                                                    c.authorUsername
                                                "
                                            />
                                            <span v-else>{{
                                                (
                                                    c.author_nickname ||
                                                    c.author_username ||
                                                    c.authorNickname ||
                                                    c.authorUsername ||
                                                    "?"
                                                )
                                                    .charAt(0)
                                                    .toUpperCase()
                                            }}</span>
                                        </div>

                                        <div class="flex-1 min-w-0">
                                            <div
                                                class="flex items-center justify-between gap-2 text-xs opacity-70"
                                            >
                                                <div class="truncate">
                                                    <span
                                                        class="font-bold text-white/80 mr-2 cursor-pointer hover:text-white transition-colors"
                                                        @click="
                                                            emit(
                                                                'show-user-profile',
                                                                c.author_id ||
                                                                    c.authorId
                                                            )
                                                        "
                                                    >
                                                        {{
                                                            c.author_nickname ||
                                                            c.author_username ||
                                                            c.authorNickname ||
                                                            c.authorUsername
                                                        }}
                                                    </span>
                                                    <span
                                                        class="text-[10px] text-white/40 font-medium"
                                                        >{{
                                                            formatDate(
                                                                c.created_at ||
                                                                    c.createdAt
                                                            )
                                                        }}</span
                                                    >
                                                </div>
                                                <div
                                                    v-if="canDelete(c)"
                                                    class="flex items-center gap-2"
                                                >
                                                    <button
                                                        class="btn btn-ghost btn-xs"
                                                        @click="
                                                            onDeleteComment(c)
                                                        "
                                                        aria-label="Delete comment"
                                                    >
                                                        {{ t("common.delete") }}
                                                    </button>
                                                </div>
                                            </div>

                                            <p
                                                class="mt-2 text-sm wrap-break-word whitespace-pre-wrap"
                                            >
                                                {{ c.text }}
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="md:col-span-1">
                        <div
                            class="sticky top-2 bg-base-200 p-3 rounded-box border border-base-300"
                        >
                            <label class="label mb-2">
                                <span class="label-text">{{
                                    t("marketplace.add_comment")
                                }}</span>
                            </label>

                            <textarea
                                v-model="newComment"
                                class="textarea textarea-bordered w-full"
                                rows="4"
                                maxlength="1000"
                                :placeholder="
                                    t('marketplace.comment_placeholder')
                                "
                                aria-label="Add a comment"
                            ></textarea>

                            <div
                                class="mt-2 flex items-center justify-between gap-2"
                            >
                                <div class="text-xs opacity-60">
                                    {{ newComment.length }}/1000
                                </div>
                                <button
                                    class="btn btn-primary"
                                    :disabled="!newComment.trim() || creating"
                                    @click="onAddComment"
                                    aria-label="Send comment"
                                >
                                    <span
                                        v-if="creating"
                                        class="loading loading-spinner loading-xs"
                                    ></span>
                                    <span v-else>{{ t("common.send") }}</span>
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
import { onMounted, ref, watch, computed } from "vue";
import { useI18n } from "vue-i18n";
import { formatDate } from "@shared/utils/utils";
import { resolveApiAssetUrl } from "@shared/utils/url";
import { marketplaceService } from "@features/marketplace/marketplaceService";
import { useUser } from "@features/auth/useUser";
import {
    ChevronLeft,
    ChevronRight,
    Download,
    ThumbsUp,
    MessageSquare,
    Image as ImageIcon,
} from "lucide-vue-next";
import { presetService } from "@features/presets/presetService";
import { useToast } from "@shared/composables/useToast";
import { useModal } from "@shared/composables/useModal";
import MarketplaceEditPresetModal from "./MarketplaceEditPresetModal.vue";
import MarketplaceDeleteConfirmModal from "./MarketplaceDeleteConfirmModal.vue";
import { buildPresetCreatePayload } from "@features/presets/utils/presetPayload";
import type {
    MarketplacePreset,
    MarketplaceTheme,
} from "@features/presets/types";

const props = defineProps<{
    id: number;
    onNavigate?: (dir: "prev" | "next") => void;
}>();
const emit = defineEmits(["close", "show-user-profile"]);

const { t } = useI18n();
const { username } = useUser();
const { addToast } = useToast();
const { showModal, hideModal } = useModal();

const loading = ref(false);
const preset = ref<MarketplacePreset | null>(null);
const comments = ref<any[]>([]);
const commentsLoading = ref(false);
const newComment = ref("");
const creating = ref(false);
const downloading = ref(false);
const toggling = ref(false);

const ownerDisplayName = computed(() => {
    const author = preset.value?.author;
    return (
        author?.displayName ??
        author?.username ??
        preset.value?.owner_username ??
        ""
    );
});

const isOwner = computed(() => {
    if (!username.value || !preset.value) return false;
    const owner =
        preset.value.author?.username ?? preset.value.owner_username ?? "";
    return owner === username.value;
});

const themeSource = computed<MarketplaceTheme>(() => {
    return (preset.value?.theme ??
        preset.value?.preset_data ??
        {}) as MarketplaceTheme;
});

const pData = computed(() => themeSource.value as Record<string, any>);

const displayedColors = computed(() => {
    const pd = themeSource.value;
    const keys: string[] = [
        "base100",
        "base200",
        "base300",
        "baseContent",
        "primary",
        "primaryContent",
        "secondary",
        "secondaryContent",
        "accent",
        "accentContent",
        "neutral",
        "neutralContent",
        "info",
        "infoContent",
        "success",
        "successContent",
        "warning",
        "warningContent",
        "error",
        "errorContent",
    ];

    return keys.filter((k) => {
        const value = pd[k as keyof MarketplaceTheme];
        return value !== undefined && value !== null && value !== "";
    });
});

async function copyColor(key: string) {
    const color = pData.value?.[key];
    if (!color) {
        return;
    }

    try {
        if (navigator.clipboard && navigator.clipboard.writeText) {
            await navigator.clipboard.writeText(color);
        }
        addToast(
            t("common.copied_to_clipboard", { value: color }) || color,
            "success"
        );
    } catch (e) {
        console.error("Copy failed", e);
    }
}

function canDelete(c: any): boolean {
    const owner =
        preset.value?.author?.username ?? preset.value?.owner_username;
    const author = c.author_username || c.authorUsername;
    return (
        !!username.value &&
        (author === username.value || owner === username.value)
    );
}

async function loadPreset() {
    loading.value = true;
    try {
        const data = await marketplaceService.getPreset(props.id);
        preset.value = data ? { ...data, liking: false } : null;
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
        newComment.value = "";
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
        console.error("Failed to delete comment:", e);
    }
}

function applyFromDetails() {
    const p = preset.value;
    if (!p) return;
    const theme = themeSource.value;
    presetService.applyPresetToTheme({
        customCSS: theme.customCSS ?? "",
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
        backgroundImage: theme.backgroundImage,
        backgroundBlur: theme.backgroundBlur,
        backgroundOpacity: theme.backgroundOpacity,
    } as any);
}

async function likeFromDetails() {
    if (!preset.value) return;
    if (preset.value.liking) return;
    preset.value.liking = true;
    try {
        if (preset.value.liked) {
            await marketplaceService.unlikePreset(preset.value.id);
            preset.value.likes_count = Math.max(
                0,
                (preset.value.likes_count || 0) - 1
            );
            preset.value.liked = false;
        } else {
            await marketplaceService.likePreset(preset.value.id);
            preset.value.likes_count = (preset.value.likes_count || 0) + 1;
            preset.value.liked = true;
        }
    } catch (e) {
        console.error("Failed to toggle like preset:", e);
    } finally {
        preset.value.liking = false;
    }
}

async function downloadFromDetails() {
    if (downloading.value) return;
    try {
        downloading.value = true;
        const p: any = preset.value;
        if (!p) return;
        const name = p.title ?? p.name ?? "Imported preset";
        const input = buildPresetCreatePayload(
            name,
            p.description || undefined,
            themeSource.value
        );
        await presetService.createPreset(input);
        addToast(
            t("theme.presets.messages.import_success", { name }),
            "success"
        );
        try {
            await marketplaceService.downloadPreset(p.id);
            if (preset.value) {
                preset.value.downloads_count =
                    (preset.value.downloads_count || 0) + 1;
            }
        } catch (e) {
            console.error("Failed to record download:", e);
        }
    } catch (e) {
        console.error("Failed to import preset:", e);
        addToast(t("theme.presets.messages.import_error"), "error");
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
        { title: t("marketplace.edit_modal_title") },
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
        await marketplaceService.updatePreset(preset.value.id, {
            is_public: preset.value.is_public,
        });
    } catch (e) {
        preset.value.is_public = prev;
        console.error("Failed to update preset visibility:", e);
        addToast(t("marketplace.updated_failed"), "error");
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
        { title: t("common.delete") },
        { id: preset.value.id },
        {
            deleted: async () => {
                hideModal(cid);
                emit("close");
            },
        }
    );
}

watch(
    () => props.id,
    async () => {
        if (props.id) {
            await Promise.all([loadPreset(), loadComments()]);
        }
    },
    { immediate: true }
);

onMounted(async () => {
    if (props.id) {
        await Promise.all([loadPreset(), loadComments()]);
    }
});
function scrollToComments() {
    const el = document.getElementById("preset-comments-section");
    if (el) {
        el.scrollIntoView({ behavior: "smooth" });
    }
}
</script>

<style scoped></style>

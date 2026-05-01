<script setup lang="ts">
import { ref, shallowRef, onMounted, computed } from "vue";
import { MessageSquare, Send, Trash2 } from "lucide-vue-next";
import { useI18n } from "vue-i18n";
import { apiDelete, apiGet, apiPost } from "@api/clients/internal";
import { userService } from "@core/auth/userService";
import { formatDate } from "@shared/utils/utils";
import { resolveApiAssetUrl } from "@shared/utils/url";
import type { ClientComment } from "@shared/types/ui";

const props = defineProps<{
    clientId: number;
    commentsCount?: number;
}>();

const emit = defineEmits(["update:comments-count", "show-user-profile"]);

const { t } = useI18n();

const comments = shallowRef<ClientComment[]>([]);
const isLoadingComments = ref(false);
const newCommentText = ref("");
const isPostingComment = ref(false);

const MAX_COMMENT_LENGTH = 500;
type CurrentUser = { username: string };
const currentUser = shallowRef<CurrentUser | null>(null);

const canComment = computed(() => !!currentUser.value);

const loadCurrentUser = async () => {
    try {
        const info = await userService.initializeUser();
        if (info && info.user_info && info.user_info.username) {
            currentUser.value = { username: info.user_info.username };
        } else {
            currentUser.value = null;
        }
    } catch (e) {
        console.error("Failed to load user info", e);
    }
};

const fetchComments = async () => {
    if (isLoadingComments.value) return;
    isLoadingComments.value = true;
    try {
        comments.value = await apiGet<ClientComment[]>(
            `/clients/${props.clientId}/comments`
        );
    } catch (error) {
        console.error("Failed to fetch comments:", error);
    } finally {
        isLoadingComments.value = false;
    }
};

const postComment = async () => {
    const trimmed = newCommentText.value.trim();
    if (!trimmed || isPostingComment.value) return;

    if (trimmed.length > MAX_COMMENT_LENGTH) {
        newCommentText.value = trimmed.slice(0, MAX_COMMENT_LENGTH);
        return;
    }

    isPostingComment.value = true;
    try {
        const newComment = await apiPost<ClientComment>(
            `/clients/${props.clientId}/comments`,
            { content: trimmed }
        );

        comments.value.unshift(newComment);
        newCommentText.value = "";
        emit("update:comments-count", (props.commentsCount || 0) + 1);
    } catch (error) {
        console.error("Failed to post comment:", error);
    } finally {
        isPostingComment.value = false;
    }
};

const deleteComment = async (commentId: number) => {
    try {
        await apiDelete(`/clients/${props.clientId}/comments/${commentId}`);

        comments.value = comments.value.filter((c) => c.id !== commentId);
        emit(
            "update:comments-count",
            Math.max(0, (props.commentsCount || 0) - 1)
        );
    } catch (error) {
        console.error("Failed to delete comment:", error);
    }
};

const openProfileFromComment = (comment: ClientComment) => {
    emit("show-user-profile", comment.user);
};

onMounted(async () => {
    await loadCurrentUser();
    await fetchComments();
});
</script>

<template>
    <div class="flex flex-col">
        <div class="flex gap-2">
            <input
                v-model="newCommentText"
                type="text"
                :placeholder="t('client.comments.placeholder')"
                class="input input-bordered w-full input-sm"
                :maxlength="MAX_COMMENT_LENGTH"
                @keyup.enter="postComment"
                :disabled="isPostingComment || !canComment"
            />
            <button
                class="btn btn-primary btn-sm btn-square"
                @click="postComment"
                :disabled="
                    isPostingComment || !newCommentText.trim() || !canComment
                "
            >
                <span
                    v-if="isPostingComment"
                    class="loading loading-spinner loading-xs"
                ></span>
                <Send v-else class="w-4 h-4" />
            </button>
        </div>

        <div class="flex justify-between text-[10px] mt-1 opacity-50">
            <span v-if="!canComment">{{ t("login") }}</span>
            <span
                >{{ Math.min(newCommentText.length, MAX_COMMENT_LENGTH) }}/{{
                    MAX_COMMENT_LENGTH
                }}</span
            >
        </div>

        <div v-if="isLoadingComments" class="flex justify-center py-8">
            <span
                class="loading loading-spinner loading-md text-primary"
            ></span>
        </div>
        <div
            v-else-if="comments.length === 0"
            class="text-center py-8 text-base-content/50"
        >
            <MessageSquare class="w-8 h-8 mx-auto mb-2 opacity-50" />
            <p>{{ t("client.comments.empty") }}</p>
        </div>
        <div
            v-else
            class="space-y-4 max-h-100 overflow-y-auto pr-2 custom-scrollbar"
        >
            <div
                v-for="comment in comments"
                :key="comment.id"
                class="chat chat-start"
            >
                <div class="chat-image avatar">
                    <div
                        class="w-8 rounded-full cursor-pointer select-none"
                        role="button"
                        tabindex="0"
                        @click.stop="openProfileFromComment(comment)"
                        @keydown.enter.stop.prevent="
                            openProfileFromComment(comment)
                        "
                        @keydown.space.stop.prevent="
                            openProfileFromComment(comment)
                        "
                    >
                        <img
                            v-if="comment.author_avatar"
                            :src="resolveApiAssetUrl(comment.author_avatar)"
                            :alt="comment.author_username"
                        />
                        <div
                            v-else
                            class="bg-base-200 text-base-content/70 w-8 h-8 flex items-center justify-center text-xs font-semibold"
                        >
                            {{
                                (
                                    comment.author_username?.[0] || "?"
                                ).toUpperCase()
                            }}
                        </div>
                    </div>
                </div>
                <div
                    class="chat-header text-xs opacity-50 mb-1 flex items-center gap-2"
                >
                    <span
                        class="cursor-pointer hover:underline"
                        @click.stop="openProfileFromComment(comment)"
                    >
                        {{ comment.author_username }}
                    </span>
                    <time class="text-[10px]">{{
                        formatDate(comment.created_at)
                    }}</time>
                    <button
                        v-if="
                            currentUser &&
                            currentUser.username === comment.author_username
                        "
                        class="btn btn-ghost btn-xs btn-circle text-error"
                        @click="deleteComment(comment.id)"
                        :title="t('client.comments.delete')"
                    >
                        <Trash2 class="w-3 h-3" />
                    </button>
                </div>
                <div
                    class="chat-bubble chat-bubble-secondary text-sm wrap-break-word group relative"
                >
                    {{ comment.content }}
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
    width: 6px;
}
.custom-scrollbar::-webkit-scrollbar-track {
    background: transparent;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
    background: rgba(0, 0, 0, 0.1);
    border-radius: 3px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
    background: rgba(0, 0, 0, 0.2);
}
</style>

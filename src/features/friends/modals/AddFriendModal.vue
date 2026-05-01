<template>
    <div class="space-y-4 max-h-[70vh] overflow-hidden flex flex-col">
        <div class="form-control shrink-0">
            <label class="label">
                <span class="label-text">{{
                    t("modals.add_friend.search_label")
                }}</span>
            </label>
            <input
                v-model="searchQuery"
                type="text"
                :placeholder="t('modals.add_friend.search_placeholder')"
                class="input input-bordered w-full"
                @input="handleSearch"
            />
        </div>

        <div class="flex-1 min-h-0 overflow-y-auto">
            <div v-if="searching" class="flex justify-center py-4">
                <span class="loading loading-spinner loading-md"></span>
            </div>

            <div v-else-if="searchResults.length > 0" class="space-y-3">
                <div
                    v-for="user in searchResults"
                    :key="user.id"
                    class="card bg-base-100 shadow-sm border border-base-300"
                >
                    <div class="card-body p-3">
                        <div class="flex items-center justify-between gap-3">
                            <div class="flex items-center gap-3 min-w-0 flex-1">
                                <div
                                    @click="$emit('view-profile', user.id)"
                                    class="avatar-click-area shrink-0"
                                >
                                    <UserAvatar
                                        :name="getDisplayNickname(user)"
                                        :is-clickable="true"
                                        :src="user.avatar_url || null"
                                        :original-src="user.avatar_url || null"
                                    />
                                </div>
                                <div class="min-w-0 flex-1">
                                    <p class="font-medium truncate">
                                        {{ getDisplayNickname(user) }}
                                    </p>
                                    <p
                                        class="text-sm text-base-content/70 truncate"
                                    >
                                        @{{ getDisplayUsername(user) }}
                                    </p>
                                </div>
                            </div>

                            <div class="shrink-0">
                                <button
                                    @click="
                                        sendFriendRequest(
                                            user.username,
                                            user.id
                                        )
                                    "
                                    class="btn btn-primary btn-sm"
                                    :disabled="
                                        sendingRequest ||
                                        user.friendship_status === 'friends'
                                    "
                                >
                                    <UserPlus class="w-4 h-4 mr-1" />
                                    <span class="hidden sm:inline">
                                        {{ getFriendButtonLabel(user) }}
                                    </span>
                                </button>

                                <template
                                    v-if="
                                        user.friendship_status ===
                                        'request_sent'
                                    "
                                >
                                    <div
                                        class="badge badge-warning ml-2 hidden sm:inline-block"
                                    >
                                        {{
                                            t("modals.add_friend.request_sent")
                                        }}
                                    </div>
                                </template>

                                <template
                                    v-else-if="
                                        user.friendship_status ===
                                        'request_received'
                                    "
                                >
                                    <div
                                        class="badge badge-info ml-2 hidden sm:inline-block"
                                    >
                                        {{
                                            t(
                                                "modals.add_friend.request_received"
                                            )
                                        }}
                                    </div>
                                </template>

                                <template
                                    v-else-if="
                                        user.friendship_status === 'blocked'
                                    "
                                >
                                    <div
                                        class="badge badge-error ml-2 hidden sm:inline-block"
                                    >
                                        {{ t("modals.add_friend.blocked") }}
                                    </div>
                                </template>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div
                v-else-if="searchQuery.length >= 2 && !searching"
                class="text-center py-4 mt-4 text-base-content/70"
            >
                <Users class="w-8 h-8 mx-auto mb-2 opacity-50" />
                <p>{{ t("modals.add_friend.no_users_found") }}</p>
            </div>

            <div
                v-else-if="searchQuery.length > 0 && searchQuery.length < 2"
                class="text-center py-4 mt-4 text-base-content/70"
            >
                <p class="text-sm">{{ t("modals.add_friend.search_hint") }}</p>
            </div>
        </div>

        <div
            class="modal-action shrink-0 pt-4"
            :class="{ 'border-t border-base-300': searchResults.length !== 0 }"
        >
            <button
                @click="$emit('close')"
                class="btn btn-outline w-full sm:w-auto"
            >
                {{ t("modals.add_friend.close") }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useToast } from "@shared/composables/useToast";
import { type SearchUser, userService } from "@core/auth/userService";
import { UserPlus, Users } from "lucide-vue-next";
import UserAvatar from "@shared/components/ui/UserAvatar.vue";
import { useI18n } from "vue-i18n";
import { useStreamerMode } from "@features/social/useStreamerMode";

const emit = defineEmits(["close", "friend-added", "view-profile"]);
const { addToast } = useToast();
const { t } = useI18n();
const streamer = useStreamerMode();

const searchQuery = ref("");
const searching = ref(false);
const sendingRequest = ref(false);
const searchResults = ref<SearchUser[]>([]);

const getDisplayNickname = (user: SearchUser) => {
    return streamer.getDisplayName(user.nickname, user.username);
};

const getDisplayUsername = (user: SearchUser) => {
    return streamer.getDisplayUsername(user.username);
};

const getFriendButtonLabel = (user: SearchUser) => {
    if (!user || !user.friendship_status)
        return t("modals.add_friend.add_friend");
    switch (user.friendship_status) {
        case "friends":
            return t("modals.add_friend.already_friends");
        case "request_sent":
            return t("modals.add_friend.request_sent");
        case "request_received":
            return t("modals.add_friend.request_received");
        case "blocked":
            return t("modals.add_friend.blocked");
        default:
            return t("modals.add_friend.add_friend");
    }
};

let searchTimeout: ReturnType<typeof setTimeout> | null = null;

const handleSearch = () => {
    if (searchTimeout) {
        clearTimeout(searchTimeout);
    }

    if (searchQuery.value.length < 2) {
        searchResults.value = [];
        return;
    }

    searchTimeout = setTimeout(async () => {
        await performSearch();
    }, 300);
};

const performSearch = async () => {
    if (searchQuery.value.length < 2) return;

    searching.value = true;
    try {
        searchResults.value = await userService.searchUsers(searchQuery.value);
    } catch (error) {
        console.error("Failed to search users:", error);
        addToast(t("toast.friends.search_failed"), "error");
        searchResults.value = [];
    } finally {
        searching.value = false;
    }
};

const sendFriendRequest = async (username: string, userId: number) => {
    sendingRequest.value = true;
    try {
        await userService.sendFriendRequest(userId);
        addToast(
            t("toast.friends.request_sent", { name: username }),
            "success"
        );

        const userIndex = searchResults.value.findIndex(
            (u) => u.username === username
        );
        if (userIndex !== -1) {
            searchResults.value[userIndex].friendship_status = "request_sent";
        }

        emit("friend-added");
    } catch (error) {
        console.error("Failed to send friend request:", error);
    } finally {
        sendingRequest.value = false;
    }
};
</script>

<style scoped>
.avatar-click-area {
    transition: all 0.2s ease;
    border-radius: 50%;
    padding: 2px;
}

.avatar-click-area:hover {
    background: hsl(var(--p) / 0.1);
}

.avatar-click-area:active {
    transform: scale(0.98);
}
</style>

<template>
    <div class="space-y-4">
        <div class="form-control">
            <label class="label">
                <span class="label-text">{{
                    t('modals.add_friend.search_label')
                    }}</span>
            </label>
            <input v-model="searchQuery" type="text" :placeholder="t('modals.add_friend.search_placeholder')"
                class="input input-bordered w-full" @input="handleSearch" />
        </div>

        <div v-if="searching" class="flex justify-center py-4">
            <span class="loading loading-spinner loading-md"></span>
        </div>

        <div v-else-if="searchResults.length > 0" class="space-y-3 max-h-60 overflow-y-auto">
            <div v-for="user in searchResults" :key="user.id" class="card bg-base-100 shadow-sm border border-base-300">
                <div class="card-body p-3">
                    <div class="flex items-center justify-between">
                        <div class="flex items-center gap-3">
                            <div @click="$emit('view-profile', user.id)" class="avatar-click-area">
                                <UserAvatar :name="getDisplayNickname(user)" :is-clickable="true" />
                            </div>
                            <div>
                                <p class="font-medium">
                                    {{ getDisplayNickname(user) }}
                                </p>
                                <p class="text-sm text-base-content/70">
                                    @{{ getDisplayUsername(user) }}
                                </p>
                            </div>
                        </div>

                        <div>
                            <button v-if="user.friendship_status === null" @click="sendFriendRequest(user.username)"
                                class="btn btn-primary btn-sm" :disabled="sendingRequest">
                                <UserPlus class="w-4 h-4 mr-1" />
                                {{ t('modals.add_friend.add_friend') }}
                            </button>

                            <div v-else-if="user.friendship_status === 'friends'" class="badge badge-success">
                                {{ t('modals.add_friend.already_friends') }}
                            </div>

                            <div v-else-if="
                                user.friendship_status === 'request_sent'
                            " class="badge badge-warning">
                                {{ t('modals.add_friend.request_sent') }}
                            </div>

                            <div v-else-if="
                                user.friendship_status ===
                                'request_received'
                            " class="badge badge-info">
                                {{ t('modals.add_friend.request_received') }}
                            </div>

                            <div v-else-if="user.friendship_status === 'blocked'" class="badge badge-error">
                                {{ t('modals.add_friend.blocked') }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div v-else-if="searchQuery.length >= 2 && !searching" class="text-center py-4 mt-4 text-base-content/70">
            <Users class="w-8 h-8 mx-auto mb-2 opacity-50" />
            <p>{{ t('modals.add_friend.no_users_found') }}</p>
        </div>

        <div v-else-if="searchQuery.length > 0 && searchQuery.length < 2"
            class="text-center py-4 mt-4 text-base-content/70">
            <p class="text-sm">{{ t('modals.add_friend.search_hint') }}</p>
        </div>

        <div class="modal-action">
            <button @click="$emit('close')" class="btn btn-outline">
                {{ t('modals.add_friend.close') }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import { useToast } from '../../services/toastService';
import { userService, type SearchUser } from '../../services/userService';
import { UserPlus, Users } from 'lucide-vue-next';
import UserAvatar from '../ui/UserAvatar.vue';
import { useI18n } from 'vue-i18n';
import { globalUserStatus } from '../../composables/useUserStatus';

const emit = defineEmits(['close', 'friend-added', 'view-profile']);
const { addToast } = useToast();
const { t } = useI18n();

const searchQuery = ref('');
const searching = ref(false);
const sendingRequest = ref(false);
const searchResults = ref<SearchUser[]>([]);

const getDisplayNickname = (user: SearchUser) => {
    if (globalUserStatus.isStreamer.value) {
        return '??????';
    }
    return user.nickname || user.username;
};

const getDisplayUsername = (user: SearchUser) => {
    if (globalUserStatus.isStreamer.value) {
        return 'unknown';
    }
    return user.username;
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
        const results = await userService.searchUsers(searchQuery.value);
        searchResults.value = results;
    } catch (error) {
        console.error('Failed to search users:', error);
        addToast(t('toast.friends.search_failed'), 'error');
        searchResults.value = [];
    } finally {
        searching.value = false;
    }
};

const sendFriendRequest = async (username: string) => {
    sendingRequest.value = true;
    try {
        await userService.sendFriendRequest(username);
        addToast(
            t('toast.friends.request_sent', { name: username }),
            'success'
        );

        const userIndex = searchResults.value.findIndex(
            (u) => u.username === username
        );
        if (userIndex !== -1) {
            searchResults.value[userIndex].friendship_status = 'request_sent';
        }

        emit('friend-added');
    } catch (error) {
        console.error('Failed to send friend request:', error);
        addToast(t('toast.friends.request_send_failed'), 'error');
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

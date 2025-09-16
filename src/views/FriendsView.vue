<template>
    <div class="header flex flex-col md:flex-row md:items-center md:justify-between gap-3 mb-6">
        <div class="title-row flex items-center gap-3">
            <h1 class="text-2xl font-semibold text-primary-focus">
                {{ t('friends.title', { count: friends.length }) }}
            </h1>
        </div>

        <div class="controls flex items-center gap-2">
            <div v-if="blockedUsers.length > 0" class="relative">
                <button @click.stop="toggleBlockedDropdown()" class="btn btn-outline btn-sm flex items-center gap-2"
                    aria-haspopup="true" aria-expanded="false">
                    <Shield class="w-4 h-4" />
                    <span class="ml-1">{{ t('blockedUsers.title', { count: blockedUsers.length }) }}</span>
                </button>

                <div ref="blockedDropdown"
                    class="hidden dropdown-content card card-compact w-72 p-2 shadow bg-base-100 border border-base-300 absolute right-0 mt-2 z-[9999]"
                    role="dialog">
                    <div class="card-body p-2">
                        <div class="space-y-2 max-h-64 overflow-y-auto">
                            <div v-for="user in blockedUsers" :key="user.id"
                                class="flex items-center justify-between p-2 bg-base-200 rounded-lg">
                                <div class="flex items-center gap-2">
                                    <div @click="$emit('show-user-profile', user.id)"
                                        class="avatar-click-area cursor-pointer">
                                        <UserAvatar :name="getDisplayNickname(user)" size="sm" />
                                    </div>
                                    <div>
                                        <p class="font-medium text-sm">
                                            {{ getDisplayNickname(user) }}
                                        </p>
                                        <p class="text-xs text-base-content/70">
                                            @{{ getDisplayUsername(user) }}
                                        </p>
                                    </div>
                                </div>
                                <button @click="unblockUser(user)" class="btn btn-primary btn-xs">
                                    <UserCheck class="w-3 h-3 mr-1" />
                                    {{ t('common.unblock') }}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <button @click="showAddFriendModal" class="btn btn-primary btn-sm flex items-center gap-2">
                <UserPlus class="w-4 h-4" />
                {{ t('friends.addFriend') }}
            </button>
        </div>
    </div>

    <div v-if="friendRequests.received.length > 0 || friendRequests.sent.length > 0" class="mb-6">
        <div class="requests-grid grid gap-4 sm:grid-cols-2 items-stretch">
            <div v-if="friendRequests.received.length > 0" class="requests-panel">
                <div class="panel-header flex items-center justify-between mb-2">
                    <h3 class="text-md font-medium text-info">{{ t('friends.receivedRequests') }}</h3>
                    <span class="text-sm text-base-content/60">{{ friendRequests.received.length }}</span>
                </div>
                <div class="requests-list grid gap-2 auto-rows-fr items-stretch">
                    <FriendRequestCard v-for="request in friendRequests.received" :key="request.id"
                        :user="{ ...request.requester, status: { ...request.requester.status, username: request.requester.username } }"
                        :request-id="request.id" type="received" @accept="respondToRequest($event, 'accept')"
                        @reject="respondToRequest($event, 'reject')"
                        @view-profile="$emit('show-user-profile', $event)" />
                </div>
            </div>

            <div v-if="friendRequests.sent.length > 0" class="requests-panel">
                <h3 class="text-md font-medium text-warning mb-2">{{ t('friends.sentRequests') }}</h3>
                <div class="requests-list grid gap-2 auto-rows-fr items-stretch">
                    <FriendRequestCard v-for="request in friendRequests.sent" :key="request.id"
                        :user="{ ...request.addressee, status: { ...request.addressee.status, username: request.addressee.username } }"
                        :request-id="request.id" type="sent" @cancel="cancelRequest"
                        @view-profile="$emit('show-user-profile', $event)" />
                </div>
            </div>
        </div>
    </div>

    <div>
        <div v-if="friends.length === 0" class="text-center py-8 text-base-content/70">
            <Users class="w-12 h-12 mx-auto mb-3 opacity-50" />
            <p>{{ t('friends.noFriends') }}</p>
        </div>

        <div v-else class="grid gap-4 grid-cols-1 sm:grid-cols-2 lg:grid-cols-3">
            <FriendCard v-for="friend in friends" :key="friend.id"
                :friend="{ ...friend, status: { ...friend.status, username: friend.username } }"
                @remove-friend="removeFriend" @block-friend="blockFriend"
                @view-profile="$emit('show-user-profile', $event)" />
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { useToast } from '../services/toastService';
import { useModal } from '../services/modalService';
import { useI18n } from 'vue-i18n';
import {
    userService,
    type Friend,
    type UserStatus,
} from '../services/userService';
import { useFriends } from '../composables/useFriends';
import AddFriendModal from '../components/modals/social/friends/AddFriendModal.vue';
import BlockUnblockConfirmModal from '../components/modals/social/friends/BlockUnblockConfirmModal.vue';
import RemoveFriendConfirmModal from '../components/modals/social/friends/RemoveFriendConfirmModal.vue';
import FriendCard from '../components/features/friends/FriendCard.vue';
import FriendRequestCard from '../components/features/friends/FriendRequestCard.vue';
import UserAvatar from '../components/ui/UserAvatar.vue';
import { UserPlus, Users, Shield, UserCheck } from 'lucide-vue-next';
import { globalUserStatus } from '../composables/useUserStatus';

const { t } = useI18n();
const { addToast } = useToast();
const { showModal, hideModal } = useModal();

defineProps<{
    isOnline: boolean;
    userId: number | null;
}>();

const emit = defineEmits([
    'show-user-profile',
    'logged-out',
    'logged-in',
    'registered',
    'change-view',
    'back-to-friends',
    'unreadCountUpdated',
]);

const { friends, friendRequests, loadFriendsData, updateFriendStatuses } =
    useFriends();
const currentUserStatus = ref<UserStatus | null>(null);
const blockedUsers = ref<Friend[]>([]);
const blockedDropdown = ref<HTMLElement | null>(null);

const toggleBlockedDropdown = () => {
    if (!blockedDropdown.value) return;
    blockedDropdown.value.classList.toggle('hidden');
};

const getDisplayNickname = (user: Friend) => {
    if (globalUserStatus.isStreamer.value) {
        return '??????';
    }
    return user.nickname || user.username;
};

const getDisplayUsername = (user: Friend) => {
    if (globalUserStatus.isStreamer.value) {
        return 'unknown';
    }
    return user.username;
};

const STATUS_UPDATE_INTERVAL = 3000;
const FULL_DATA_UPDATE_INTERVAL = 15000;

let statusRefreshInterval: number | null = null;
let fullDataRefreshInterval: number | null = null;

onMounted(async () => {
    await loadFriendsAndStatus(true);

    statusRefreshInterval = window.setInterval(
        updateStatuses,
        STATUS_UPDATE_INTERVAL
    );
    fullDataRefreshInterval = window.setInterval(
        loadFriendsAndStatus,
        FULL_DATA_UPDATE_INTERVAL
    );
});

onUnmounted(() => {
    if (statusRefreshInterval) {
        clearInterval(statusRefreshInterval);
    }
    if (fullDataRefreshInterval) {
        clearInterval(fullDataRefreshInterval);
    }
});

const loadFriendsAndStatus = async (forceReload = false) => {
    try {
        if (forceReload || friends.value.length === 0) {
            await loadFriendsData();
        } else {
            await updateFriendStatuses();
        }

        await loadBlockedUsers();

        const userStatusData = await userService.getUserStatus();
        currentUserStatus.value = userStatusData;
    } catch (error) {
        console.error('Failed to load friends data:', error);
        addToast(t('friends.load_failed'), 'error');
    }
};

const updateStatuses = async () => {
    try {
        await updateFriendStatuses();

        const userStatusData = await userService.getUserStatus();
        currentUserStatus.value = userStatusData;
    } catch (error) {
        console.error('Failed to update friend statuses:', error);
    }
};

const loadBlockedUsers = async () => {
    try {
        blockedUsers.value = await userService.getBlockedUsers();
    } catch (error) {
        console.error('Failed to load blocked users:', error);
    }
};

const showAddFriendModal = () => {
    showModal(
        'add-friend',
        AddFriendModal,
        { title: t('friends.addFriend') },
        {},
        {
            'friend-added': async () => {
                await loadFriendsData();
                hideModal('add-friend');
            },
            'view-profile': (userId: number) => {
                hideModal('add-friend');
                emit('show-user-profile', userId);
            },
            close: () => hideModal('add-friend'),
        }
    );
};

const respondToRequest = async (
    requestId: number,
    action: 'accept' | 'reject'
) => {
    try {
        await userService.respondToFriendRequest(requestId, action);
        addToast(
            action === 'accept'
                ? t('friends.request_accepted')
                : t('friends.request_rejected'),
            'success'
        );
        await loadFriendsData(true);
    } catch (error) {
        console.error(`Failed to ${action} friend request:`, error);
        addToast(t('friends.request_failed', { action }), 'error');
    }
};

const cancelRequest = async (requestId: number) => {
    try {
        await userService.cancelFriendRequest(requestId);
        addToast(t('friends.request_canceled'), 'success');
        await loadFriendsData(true);
    } catch (error) {
        console.error('Failed to cancel friend request:', error);
        addToast(t('friends.request_failed', { action: 'cancel' }), 'error');
    }
};

const removeFriend = async (friend: Friend) => {
    showModal(
        'remove-friend-confirm',
        RemoveFriendConfirmModal,
        { title: t('userProfile.remove_friend') },
        { friend: friend },
        {
            confirm: async (confirmedFriend: Friend) => {
                try {
                    await userService.removeFriend(confirmedFriend.id);
                    addToast(
                        t('friends.remove_success', {
                            name:
                                confirmedFriend.nickname ||
                                confirmedFriend.username,
                        }),
                        'success'
                    );
                    await loadFriendsData(true);
                } catch (error) {
                    console.error('Failed to remove friend:', error);
                    addToast(t('friends.remove_failed'), 'error');
                }
                hideModal('remove-friend-confirm');
            },
            close: () => hideModal('remove-friend-confirm'),
        }
    );
};

const blockFriend = async (friend: Friend) => {
    showModal(
        'block-confirm',
        BlockUnblockConfirmModal,
        { title: t('userProfile.block_user') },
        { user: friend, action: 'block' },
        {
            confirm: async (user: Friend) => {
                try {
                    await userService.blockUser(user.id);
                    addToast(
                        t('friends.block_success', {
                            name: user.nickname || user.username,
                        }),
                        'success'
                    );
                    await Promise.all([loadFriendsData(true), loadBlockedUsers()]);
                } catch (error) {
                    console.error('Failed to block user:', error);
                    addToast(t('friends.block_failed'), 'error');
                }
                hideModal('block-confirm');
            },
            close: () => hideModal('block-confirm'),
        }
    );
};

const unblockUser = async (user: Friend) => {
    showModal(
        'unblock-confirm',
        BlockUnblockConfirmModal,
        { title: t('userProfile.unblock_user') },
        { user: user, action: 'unblock' },
        {
            confirm: async (confirmedUser: Friend) => {
                try {
                    await userService.unblockUser(confirmedUser.id);
                    addToast(
                        t('friends.unblock_success', {
                            name:
                                confirmedUser.nickname ||
                                confirmedUser.username,
                        }),
                        'success'
                    );
                    await loadBlockedUsers();
                } catch (error) {
                    console.error('Failed to unblock user:', error);
                    addToast(t('friends.unblock_failed'), 'error');
                }
                hideModal('unblock-confirm');
            },
            close: () => hideModal('unblock-confirm'),
        }
    );
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

.requests-grid {
    grid-template-columns: 1fr;
}

.requests-panel {
    background: var(--b3);
    border: 1px solid var(--b2);
    border-radius: 0.5rem;
}

.requests-list {
    max-height: 18rem;
    overflow-y: auto;
    padding-right: 0.25rem;
}

.panel-header {
    border-bottom: 1px dashed rgba(0, 0, 0, 0.04);
    padding-bottom: 0.4rem;
}

@media (min-width: 768px) {
    .requests-grid {
        grid-template-columns: repeat(2, minmax(0, 1fr));
    }
}

.badge-sm {
    padding: 0.15rem 0.5rem;
    font-size: 0.7rem;
}
</style>

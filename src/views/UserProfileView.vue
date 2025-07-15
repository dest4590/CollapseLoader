<template>
    <div class="max-w-2xl mx-auto p-6 slide-up">
        <div v-if="error" class="text-center py-8">
            <div class="alert alert-error max-w-md mx-auto">
                <span>{{ error }}</span>
            </div>
            <button @click="$emit('change-view', 'friends')" class="btn btn-primary mt-4">
                {{ t('userProfile.back_to_friends') }}
            </button>
        </div>

        <div v-else-if="userProfile" class="space-y-6">
            <div class="card bg-gradient-to-br from-primary/10 to-primary/5 shadow-md border border-primary/20">
                <div class="card-body">
                    <div class="flex items-center gap-6">
                        <UserAvatar :name="displayNickname" size="lg" :show-status="false"
                            :is-online="userProfile.status.is_online" />
                        <div class="flex-1">
                            <h1 class="text-2xl font-bold text-primary-focus">
                                {{ displayNickname }}
                            </h1>
                            <p class="text-lg text-base-content/70">
                                @{{ displayUsername }}
                            </p>

                            <div class="mt-3">
                                <div v-if="userProfile.status.current_client" class="flex items-center gap-2">
                                    <Gamepad2 class="w-4 h-4 text-primary" />
                                    <span class="text-primary">{{ t('userProfile.playing') }}
                                        {{
                                            userProfile.status.current_client
                                        }}</span>
                                    <span v-if="userProfile.status.client_version" class="text-base-content/50 text-sm">
                                        ({{
                                            userProfile.status.client_version
                                        }})
                                    </span>
                                </div>
                                <div v-else-if="userProfile.status.is_online" class="flex items-center gap-2">
                                    <div class="w-3 h-3 bg-success rounded-full"></div>
                                    <span class="text-success font-medium">{{
                                        t('userProfile.online')
                                    }}</span>
                                </div>
                                <div v-else-if="userProfile.status.last_seen" class="flex items-center gap-2">
                                    <div class="w-3 h-3 bg-base-content/30 rounded-full"></div>
                                    <span class="text-base-content/70">{{ t('userProfile.last_seen') }}
                                        {{
                                            formatLastSeen(
                                                userProfile.status.last_seen
                                            )
                                        }}</span>
                                </div>
                                <div v-else class="flex items-center gap-2">
                                    <div class="w-3 h-3 bg-base-content/30 rounded-full"></div>
                                    <span class="text-base-content/70">{{
                                        t('userProfile.offline')
                                    }}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </div>

            <div class="card bg-base-200 shadow-md border border-base-300">
                <div class="card-body">
                    <h2 class="card-title text-lg font-medium text-primary-focus mb-4">
                        {{ t('userProfile.profile_information') }}
                    </h2>
                    <div class="space-y-3">
                        <div class="flex justify-between items-center">
                            <span class="text-base-content/70">{{ t('userProfile.username') }}:</span>
                            <span class="font-medium">{{
                                displayUsername
                            }}</span>
                        </div>
                        <div v-if="userProfile.nickname" class="flex justify-between items-center">
                            <span class="text-base-content/70">{{ t('userProfile.nickname') }}:</span>
                            <span class="font-medium">{{
                                displayNickname
                            }}</span>
                        </div>
                        <div v-if="userProfile.member_since" class="flex justify-between items-center">
                            <span class="text-base-content/70">{{ t('userProfile.member_since') }}:</span>
                            <span class="font-medium">{{
                                formatDate(userProfile.member_since)
                            }}</span>
                        </div>
                    </div>
                </div>
            </div>

            <div class="card bg-base-200 shadow-md border border-base-300">
                <div class="card-body">
                    <h2 class="card-title text-lg font-medium text-primary-focus mb-4">
                        {{ t('userProfile.actions') }}
                    </h2>

                    <div v-if="userProfile.friendship_status === 'friends'" class="space-y-3">
                        <div class="alert alert-success">
                            <UserCheck class="w-5 h-5" />
                            <span>{{
                                t('userProfile.friends_with', {
                                    name: displayNickname,
                                })
                            }}</span>
                        </div>
                        <button @click="handleRemoveFriend" class="btn btn-error btn-outline w-full">
                            <UserMinus class="w-4 h-4 mr-2" />
                            {{ t('userProfile.remove_friend') }}
                        </button>
                    </div>

                    <div v-else-if="
                        userProfile.friendship_status === 'request_sent'
                    " class="space-y-3">
                        <div class="alert alert-warning">
                            <Clock class="w-5 h-5" />
                            <span>{{
                                t('userProfile.friend_request_sent', {
                                    name: displayNickname,
                                })
                            }}</span>
                        </div>
                        <button @click="handleCancelFriendRequest" class="btn btn-error btn-outline w-full"
                            :disabled="sendingRequest">
                            <X class="w-4 h-4 mr-2" />
                            {{
                                sendingRequest
                                    ? t('userProfile.canceling')
                                    : t('userProfile.cancel_friend_request')
                            }}
                        </button>
                    </div>

                    <div v-else-if="
                        userProfile.friendship_status === 'request_received'
                    " class="space-y-3">
                        <div class="alert alert-info">
                            <UserPlus class="w-5 h-5" />
                            <span>{{
                                t('userProfile.friend_request_received', {
                                    name: displayNickname,
                                })
                            }}</span>
                        </div>
                        <div class="flex gap-3">
                            <button @click="handleRespondToRequest('accept')" class="btn btn-success flex-1">
                                <Check class="w-4 h-4 mr-2" />
                                {{ t('userProfile.accept') }}
                            </button>
                            <button @click="handleRespondToRequest('reject')" class="btn btn-error btn-outline flex-1">
                                <X class="w-4 h-4 mr-2" />
                                {{ t('userProfile.decline') }}
                            </button>
                        </div>
                    </div>

                    <div v-else-if="userProfile.friendship_status === 'blocked'" class="alert alert-error">
                        <Ban class="w-5 h-5" />
                        <span>{{ t('userProfile.user_blocked') }}</span>
                    </div>

                    <div v-else>
                        <div class="space-y-3">
                            <button @click="handleSendFriendRequest" class="btn btn-primary w-full"
                                :disabled="sendingRequest">
                                <UserPlus class="w-4 h-4 mr-2" />
                                {{
                                    sendingRequest
                                        ? t('userProfile.sending')
                                        : t('userProfile.send_friend_request')
                                }}
                            </button>
                            <button @click="handleBlockUser" class="btn btn-error btn-outline w-full"
                                :disabled="sendingRequest">
                                <Ban class="w-4 h-4 mr-2" />
                                {{ t('userProfile.block_user') }}
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <div class="text-center">
                <button @click="$emit('change-view', 'friends')" class="btn btn-outline">
                    <ArrowLeft class="w-4 h-4 mr-2" />
                    {{ t('userProfile.back_to_friends') }}
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useToast } from '../services/toastService';
import { useModal } from '../services/modalService';
import { userService, type PublicUserProfile } from '../services/userService';
import BlockUnblockConfirmModal from '../components/modals/BlockUnblockConfirmModal.vue';
import RemoveFriendConfirmModal from '../components/modals/RemoveFriendConfirmModal.vue';
import UserAvatar from '../components/ui/UserAvatar.vue';
import {
    Gamepad2,
    Clock,
    UserCheck,
    UserMinus,
    UserPlus,
    Check,
    X,
    Ban,
    ArrowLeft,
} from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';
import { globalUserStatus } from '../composables/useUserStatus';

interface Props {
    userId?: number;
}

const props = defineProps<Props>();
const emit = defineEmits(['change-view']);

const { t } = useI18n();
const { addToast } = useToast();
const { showModal, hideModal } = useModal();

const userProfile = ref<PublicUserProfile | null>(null);
const loading = ref(true);
const error = ref<string | null>(null);
const sendingRequest = ref(false);

const displayNickname = computed(() => {
    if (!userProfile.value) return '';
    if (globalUserStatus.isStreamer.value) {
        return 'Streamer';
    }
    return userProfile.value.nickname || userProfile.value.username;
});

const displayUsername = computed(() => {
    if (!userProfile.value) return '';
    if (globalUserStatus.isStreamer.value) {
        return 'streamer';
    }
    return userProfile.value.username;
});

const loadUserProfile = async () => {
    if (!props.userId) {
        error.value = t('userProfile.no_user_id');
        loading.value = false;
        return;
    }

    try {
        loading.value = true;
        error.value = null;
        userProfile.value = await userService.getUserProfile(props.userId);
    } catch (err: any) {
        console.error('Failed to load user profile:', err);
        if (err.response?.status === 404) {
            error.value = t('userProfile.user_not_found');
        } else if (err.response?.status === 401) {
            error.value = t('userProfile.login_required');
        } else {
            error.value = t('userProfile.profile_load_failed');
        }
    } finally {
        loading.value = false;
    }
};

const handleSendFriendRequest = async () => {
    if (!userProfile.value) return;

    sendingRequest.value = true;
    try {
        await userService.sendFriendRequest(userProfile.value.username);
        addToast(
            t('userProfile.friend_request_sent_success', {
                name: userProfile.value.nickname || userProfile.value.username,
            }),
            'success'
        );
        userProfile.value.friendship_status = 'request_sent';
    } catch (error) {
        console.error('Failed to send friend request:', error);
        addToast(t('userProfile.friend_request_failed'), 'error');
    } finally {
        sendingRequest.value = false;
    }
};

const handleRemoveFriend = async () => {
    if (!userProfile.value) return;

    showModal(
        'remove-friend-confirm',
        RemoveFriendConfirmModal,
        { title: t('userProfile.remove_friend') },
        { friend: userProfile.value },
        {
            confirm: async (confirmedFriend: any) => {
                try {
                    await userService.removeFriend(confirmedFriend.id);
                    addToast(
                        t('userProfile.friend_removed_success', {
                            name:
                                confirmedFriend.nickname ||
                                confirmedFriend.username,
                        }),
                        'success'
                    );
                    userProfile.value!.friendship_status = null;
                } catch (error) {
                    console.error('Failed to remove friend:', error);
                    addToast(t('userProfile.friend_remove_failed'), 'error');
                }
                hideModal('remove-friend-confirm');
            },
            close: () => hideModal('remove-friend-confirm'),
        }
    );
};

const handleRespondToRequest = async (action: 'accept' | 'reject') => {
    if (!userProfile.value) return;

    try {
        const requests = await userService.getFriendRequests();
        const request = requests.received.find(
            (req) => req.requester.id === userProfile.value!.id
        );

        if (!request) {
            addToast(t('userProfile.friend_request_not_found'), 'error');
            return;
        }

        await userService.respondToFriendRequest(request.id, action);
        addToast(
            t('userProfile.friend_request_responded', { action }),
            'success'
        );

        if (action === 'accept') {
            userProfile.value.friendship_status = 'friends';
        } else {
            userProfile.value.friendship_status = null;
        }
    } catch (error) {
        console.error(`Failed to ${action} friend request:`, error);
        addToast(
            t('userProfile.friend_request_respond_failed', { action }),
            'error'
        );
    }
};

const handleBlockUser = async () => {
    if (!userProfile.value) return;

    showModal(
        'block-confirm',
        BlockUnblockConfirmModal,
        { title: t('userProfile.block_user') },
        { user: userProfile.value, action: 'block' },
        {
            confirm: async (user: any) => {
                try {
                    await userService.blockUser(user.id);
                    addToast(
                        t('userProfile.user_blocked_success', {
                            name: user.nickname || user.username,
                        }),
                        'success'
                    );
                    userProfile.value!.friendship_status = 'blocked';
                } catch (error) {
                    console.error('Failed to block user:', error);
                    addToast(t('userProfile.user_block_failed'), 'error');
                }
                hideModal('block-confirm');
            },
            close: () => hideModal('block-confirm'),
        }
    );
};

const handleCancelFriendRequest = async () => {
    if (!userProfile.value) return;

    sendingRequest.value = true;
    try {
        const requests = await userService.getFriendRequests();
        const request = requests.sent.find(
            (req) => req.addressee.id === userProfile.value!.id
        );

        if (!request) {
            addToast(t('userProfile.friend_request_not_found'), 'error');
            return;
        }

        await userService.cancelFriendRequest(request.id);
        addToast(t('userProfile.friend_request_canceled'), 'success');
        userProfile.value.friendship_status = null;
    } catch (error) {
        console.error('Failed to cancel friend request:', error);
        addToast(t('userProfile.friend_request_cancel_failed'), 'error');
    } finally {
        sendingRequest.value = false;
    }
};

const formatLastSeen = (lastSeen: string): string => {
    const date = new Date(lastSeen);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

    if (diffMins < 1) return t('userProfile.just_now');
    if (diffMins < 60)
        return t('userProfile.minutes_ago', {
            count: diffMins,
            s: diffMins === 1 ? 'у' : '',
        });
    if (diffHours < 24)
        return t('userProfile.hours_ago', {
            count: diffHours,
            s: diffHours === 1 ? '' : 'ов',
        });
    if (diffDays < 7)
        return t('userProfile.days_ago', {
            count: diffDays,
            s: diffDays === 1 ? 'ень' : 'я',
        });

    return date.toLocaleDateString();
};

const formatDate = (dateString: string): string => {
    try {
        const date = new Date(dateString);
        const day = String(date.getDate()).padStart(2, '0');
        const month = String(date.getMonth() + 1).padStart(2, '0');
        const year = date.getFullYear();

        return `${day}/${month}/${year}`;
    } catch (e) {
        return t('userProfile.unknown');
    }
};

onMounted(() => {
    loadUserProfile();
});
</script>

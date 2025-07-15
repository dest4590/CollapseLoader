<template>
    <div class="card bg-base-200 shadow-sm border border-base-300">
        <div class="card-body p-4">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <div @click="$emit('viewProfile', user.id)" class="avatar-click-area">
                        <UserAvatar :name="displayNickname" :is-clickable="true" />
                    </div>
                    <div>
                        <p class="font-medium">{{ displayNickname }}</p>
                        <p class="text-sm text-base-content/70">
                            @{{ displayUsername }}
                        </p>
                    </div>
                </div>
                <div v-if="type === 'received'" class="flex gap-2">
                    <button @click="$emit('accept', requestId)" class="btn btn-success btn-sm">
                        <Check class="w-4 h-4" />
                    </button>
                    <button @click="$emit('reject', requestId)" class="btn btn-error btn-sm">
                        <X class="w-4 h-4" />
                    </button>
                </div>
                <div v-else-if="type === 'sent'" class="flex gap-2 items-center">
                    <div class="badge badge-warning">
                        {{ t('friends.pending') }}
                    </div>
                    <button @click="confirmCancel" class="btn btn-error btn-outline btn-sm">
                        <X class="w-4 h-4" />
                        {{ t('common.cancel') }}
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { Check, X } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';
import { useModal } from '../../../services/modalService';
import UserAvatar from '../../ui/UserAvatar.vue';
import CancelFriendRequestConfirmModal from '../../modals/CancelFriendRequestConfirmModal.vue';
import type { Friend } from '../../../services/userService';
import { globalUserStatus } from '../../../composables/useUserStatus';
import { computed } from 'vue';

const { t } = useI18n();
const { showModal, hideModal } = useModal();

const props = defineProps<{
    user: Friend;
    requestId: number;
    type: 'received' | 'sent';
}>();

const emit = defineEmits<{
    accept: [requestId: number];
    reject: [requestId: number];
    viewProfile: [userId: number];
    cancel: [requestId: number];
}>();

const displayNickname = computed(() => {
    if (globalUserStatus.isStreamer.value) {
        return '??????';
    }
    return props.user.nickname || props.user.username;
});

const displayUsername = computed(() => {
    if (globalUserStatus.isStreamer.value) {
        return 'unknown';
    }
    return props.user.username;
});

const confirmCancel = () => {
    showModal(
        'cancel-friend-request-confirm',
        CancelFriendRequestConfirmModal,
        { title: t('friends.cancel_friend_request') },
        { user: props.user, requestId: props.requestId },
        {
            confirm: (requestId: number) => {
                emit('cancel', requestId);
                hideModal('cancel-friend-request-confirm');
            },
            close: () => hideModal('cancel-friend-request-confirm'),
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
</style>

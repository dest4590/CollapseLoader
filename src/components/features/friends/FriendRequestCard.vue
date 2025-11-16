<template>
    <div class="card bg-base-200 shadow-sm border border-base-300 h-full flex flex-col">
        <div class="card-body p-4 flex-1 flex items-center">
            <div class="flex items-center justify-between w-full">
                <div class="flex items-center gap-3">
                    <div @click="$emit('viewProfile', user.id)">
                        <UserAvatar :name="displayNickname" :is-clickable="true"
                            :src="(props.user as any).avatar_url || null"
                            :original-src="(props.user as any).avatar_url || null" />
                    </div>
                    <div class="min-h-14">
                        <p class="font-medium">{{ displayNickname }}</p>
                        <p class="text-sm text-base-content/70">
                            @{{ displayUsername }}
                        </p>
                    </div>
                </div>
                <div v-if="type === 'received'" class="flex gap-2 items-center">
                    <button @click="$emit('accept', requestId)" class="btn btn-success btn-sm">
                        <Check class="w-4 h-4" />
                    </button>
                    <button @click="$emit('reject', requestId)" class="btn btn-error btn-sm">
                        <X class="w-4 h-4" />
                    </button>
                </div>
                <div v-else-if="type === 'sent'" class="flex gap-2 items-center">
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
import CancelFriendRequestConfirmModal from '../../modals/social/friends/CancelFriendRequestConfirmModal.vue';
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

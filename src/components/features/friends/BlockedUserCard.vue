<template>
    <div class="card bg-base-100 shadow-sm border border-base-300">
        <div class="card-body p-4">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-3">
                    <div @click="$emit('view-profile', blockedUser.id)" class="avatar-click-area cursor-pointer">
                        <UserAvatar :name="displayNickname" />
                    </div>
                    <div>
                        <p class="font-medium">{{ displayNickname }}</p>
                        <p class="text-sm text-base-content/70">
                            @{{ displayUsername }}
                        </p>
                        <p class="text-xs text-base-content/50">
                            {{ $t('common.blocked') }}
                        </p>
                    </div>
                </div>

                <div class="flex items-center gap-2">
                    <button @click="handleUnblockUser" class="btn btn-primary btn-sm">
                        <UserCheck class="w-4 h-4 mr-1" />
                        {{ $t('userProfile.unblock_user') }}
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { UserCheck } from 'lucide-vue-next';
import UserAvatar from '../../ui/UserAvatar.vue';
import type { Friend } from '../../../services/userService';
import { globalUserStatus } from '../../../composables/useUserStatus';
import { computed } from 'vue';

const props = defineProps<{
    blockedUser: Friend;
}>();

const emit = defineEmits<{
    unblockUser: [user: Friend];
    'view-profile': [userId: number];
}>();

const displayNickname = computed(() => {
    if (globalUserStatus.isStreamer.value) {
        return '??????';
    }
    return props.blockedUser.nickname || props.blockedUser.username;
});

const displayUsername = computed(() => {
    if (globalUserStatus.isStreamer.value) {
        return 'unknown';
    }
    return props.blockedUser.username;
});

const handleUnblockUser = () => {
    emit('unblockUser', props.blockedUser);
};
</script>

<style scoped>
.avatar-click-area {
    transition: all 0.2s ease;
    border-radius: 50%;
    padding: 2px;
}

.avatar-click-area:hover {
    transform: scale(1.05);
    background-color: rgba(var(--primary), 0.1);
}
</style>

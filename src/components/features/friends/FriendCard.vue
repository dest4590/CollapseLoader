<template>
    <div
        class="card bg-base-200 shadow-md border border-base-300 hover:shadow-lg transition-shadow h-full flex flex-col">
        <div class="card-body p-4 flex-1 flex items-center">
            <div class="flex items-center justify-between w-full">
                <div class="flex items-center gap-4">
                    <div @click="handleViewProfile" class="avatar-click-area">
                        <UserAvatar :name="friend.nickname || friend.username" :show-status="true"
                            :is-online="friend.status.is_online" :is-clickable="true"
                            :src="(friend as any).avatar_url || null"
                            :original-src="(friend as any).avatar_url || null" />
                    </div>
                    <div class="flex-1 min-h-14">
                        <p class="font-medium">{{ displayNickname }}</p>
                        <p class="text-sm text-base-content/70">
                            @{{ displayUsername }}
                        </p>

                        <div v-if="friend.status.current_client" class="flex items-center gap-2 mt-1">
                            <Gamepad2 class="w-3 h-3 text-primary" />
                            <span class="text-xs text-primary">{{ t('userProfile.playing') }}
                                {{ friend.status.current_client }}</span>
                            <span v-if="friend.status.client_version" class="text-xs text-base-content/50">
                                ({{ friend.status.client_version }})
                            </span>
                        </div>

                        <div v-else-if="friend.status.is_online && !globalUserStatus.isStreamer.value"
                            class="flex items-center gap-2 mt-1">
                            <span class="w-2 h-2 rounded-full bg-success inline-block" aria-hidden="true"></span>
                            <span class="text-xs text-success">{{ t('userProfile.online') }}</span>
                        </div>

                        <div v-else-if="friend.status.last_seen && !globalUserStatus.isStreamer.value"
                            class="flex items-center gap-2 mt-1">
                            <Clock class="w-3 h-3 text-base-content/50" />
                            <span class="text-xs text-base-content/50">
                                {{ t('userProfile.last_seen') }}
                                {{ formatLastSeen(friend.status.last_seen) }}
                            </span>
                        </div>
                    </div>
                </div>

                <div class="dropdown dropdown-end">
                    <div tabindex="0" role="button" class="btn btn-ghost btn-sm btn-circle">
                        <MoreVertical class="w-4 h-4" />
                    </div>
                    <ul tabindex="0"
                        class="dropdown-content menu p-2 bg-base-200 shadow-xl border border-base-300 rounded-box w-52 z-9999">
                        <li>
                            <a @click="handleRemoveFriend" class="text-error">
                                <UserMinus class="w-4 h-4" />
                                {{ t('userProfile.remove_friend') }}
                            </a>
                        </li>
                        <li>
                            <a @click="handleBlockFriend" class="text-error">
                                <Shield class="w-4 h-4" />
                                {{ t('userProfile.block_user') }}
                            </a>
                        </li>
                    </ul>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import {
    Gamepad2,
    Clock,
    MoreVertical,
    UserMinus,
    Shield,
} from 'lucide-vue-next';
import UserAvatar from '../../ui/UserAvatar.vue';
import type { Friend } from '../../../services/userService';
import { useI18n } from 'vue-i18n';
import { globalUserStatus } from '../../../composables/useUserStatus';
import { computed } from 'vue';

const props = defineProps<{
    friend: Friend;
}>();

const emit = defineEmits<{
    removeFriend: [friend: Friend];
    blockFriend: [friend: Friend];
    viewProfile: [userId: number];
}>();

const { t } = useI18n();

const displayNickname = computed(() => {
    if (globalUserStatus.isStreamer.value) {
        return '??????';
    }
    return props.friend.nickname || props.friend.username;
});

const displayUsername = computed(() => {
    if (globalUserStatus.isStreamer.value) {
        return 'unknown';
    }
    return props.friend.username;
});

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

    const day = String(date.getDate()).padStart(2, '0');
    const month = String(date.getMonth() + 1).padStart(2, '0');
    const year = date.getFullYear();
    return `${day}/${month}/${year}`;
};

const handleRemoveFriend = () => {
    emit('removeFriend', props.friend);
};

const handleBlockFriend = () => {
    emit('blockFriend', props.friend);
};

const handleViewProfile = () => {
    // emit both forms to be resilient to different listener naming conventions
    emit('viewProfile', props.friend.id);
    //emit('view-profile', props.friend.id as any);
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

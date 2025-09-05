<template>
    <div>
        <p class="text-sm text-base-content/70">
            {{ $t('modals.remove_friend_confirm.message', { displayName }) }}
        </p>

        <div class="flex justify-end space-x-2 mt-6">
            <button @click="confirmAction" class="btn btn-error">
                <UserMinus class="w-4 h-4 mr-2" />
                {{ $t('modals.remove_friend_confirm.yes_remove') }}
            </button>
            <button @click="$emit('close')" class="btn btn-outline">
                <X class="w-4 h-4 mr-2" />
                {{ $t('common.cancel') }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { UserMinus, X } from 'lucide-vue-next';

interface Friend {
    id: number;
    username: string;
    nickname?: string;
}

const props = defineProps<{
    friend: Friend;
}>();

const emit = defineEmits(['close', 'confirm']);

const displayName = computed(
    () => props.friend.nickname || props.friend.username
);

const confirmAction = () => {
    emit('confirm', props.friend);
    emit('close');
};
</script>

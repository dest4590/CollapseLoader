<template>
    <div>
        <p class="text-sm text-base-content/70">
            {{
                $t('modals.cancel_friend_request_confirm.message', {
                    displayName,
                })
            }}
        </p>

        <div class="flex justify-end space-x-2 mt-6">
            <button @click="confirmAction" class="btn btn-error">
                <X class="w-4 h-4 mr-2" />
                {{ $t('modals.cancel_friend_request_confirm.yes_cancel') }}
            </button>
            <button @click="$emit('close')" class="btn btn-outline">
                <ArrowLeft class="w-4 h-4 mr-2" />
                {{ $t('modals.cancel_friend_request_confirm.keep_request') }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { X, ArrowLeft } from 'lucide-vue-next';

interface User {
    id: number;
    username: string;
    nickname?: string;
}

const props = defineProps<{
    user: User;
    requestId: number;
}>();

const emit = defineEmits(['close', 'confirm']);

const displayName = computed(() => props.user.nickname || props.user.username);

const confirmAction = () => {
    emit('confirm', props.requestId);
    emit('close');
};
</script>

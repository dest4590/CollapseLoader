<template>
    <div>
        <p class="text-sm text-base-content/70">
            {{ confirmationMessage }}
        </p>

        <div class="flex justify-end space-x-2 mt-6">
            <button @click="confirmAction" class="btn" :class="actionButtonClass">
                <component :is="actionIcon" class="w-4 h-4 mr-2" />
                {{ actionButtonText }}
            </button>
            <button @click="$emit('close')" class="btn btn-outline">
                <x-icon class="w-4 h-4 mr-2" />
                Cancel
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { Shield, ShieldOff, X as XIcon } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

interface User {
    id: number;
    username: string;
    nickname?: string;
}

const props = defineProps<{
    user: User;
    action: 'block' | 'unblock';
}>();

const emit = defineEmits(['close', 'confirm']);
const { t } = useI18n();

const displayName = computed(() => props.user.nickname || props.user.username);

const confirmationMessage = computed(() => {
    if (props.action === 'block') {
        return t('modals.block_unblock_confirm.block_message', {
            displayName: displayName.value,
        });
    } else {
        return t('modals.block_unblock_confirm.unblock_message', {
            displayName: displayName.value,
        });
    }
});

const actionButtonText = computed(() => {
    return props.action === 'block'
        ? t('modals.block_unblock_confirm.yes_block')
        : t('modals.block_unblock_confirm.yes_unblock');
});

const actionButtonClass = computed(() => {
    return props.action === 'block' ? 'btn-error' : 'btn-primary';
});

const actionIcon = computed(() => {
    return props.action === 'block' ? Shield : ShieldOff;
});

const confirmAction = () => {
    emit('confirm', props.user);
    emit('close');
};
</script>

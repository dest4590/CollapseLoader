<template>
    <div class="max-w-4xl mx-auto p-6">
        <div class="flex justify-between items-center mb-6">
            <h1 class="text-2xl font-semibold text-primary-focus">
                {{
                    $t('blockedUsers.title', {
                        count: blockedUsers.length,
                    })
                }}
            </h1>
            <button @click="$emit('back-to-friends')" class="btn btn-ghost btn-sm">
                <ArrowLeft class="w-4 h-4 mr-2" />
                {{ $t('common.back') }}
            </button>
        </div>

        <div v-if="isLoading" class="flex justify-center py-8">
            <span class="loading loading-spinner loading-md"></span>
        </div>

        <div v-else-if="blockedUsers.length === 0" class="text-center py-8 text-base-content/70">
            <Shield class="w-12 h-12 mx-auto mb-3 opacity-50" />
            <p>{{ $t('blockedUsers.noBlockedUsers') }}</p>
        </div>

        <div v-else class="grid gap-4">
            <BlockedUserCard v-for="user in blockedUsers" :key="user.id" :blocked-user="user"
                @unblock-user="unblockUser" @view-profile="$emit('show-user-profile', $event)" />
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useToast } from '../services/toastService';
import { useModal } from '../services/modalService';
import { userService, type Friend } from '../services/userService';
import { ArrowLeft, Shield } from 'lucide-vue-next';
import BlockedUserCard from '../components/features/friends/BlockedUserCard.vue';
import BlockUnblockConfirmModal from '../components/modals/BlockUnblockConfirmModal.vue';
import { useI18n } from 'vue-i18n';

const { addToast } = useToast();
const { showModal, hideModal } = useModal();
const { t } = useI18n();

const emit = defineEmits(['back-to-friends', 'show-user-profile']);

const blockedUsers = ref<Friend[]>([]);
const isLoading = ref(false);

onMounted(async () => {
    await loadBlockedUsers();
});

const loadBlockedUsers = async () => {
    isLoading.value = true;
    try {
        blockedUsers.value = await userService.getBlockedUsers();
    } catch (error) {
        console.error('Failed to load blocked users:', error);
        addToast(t('toast.users.blocked_load_failed'), 'error');
    } finally {
        isLoading.value = false;
    }
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
                        t('toast.users.unblocked_success', {
                            name:
                                confirmedUser.nickname ||
                                confirmedUser.username,
                        }),
                        'success'
                    );
                    await loadBlockedUsers();
                } catch (error) {
                    console.error('Failed to unblock user:', error);
                    addToast(t('toast.users.unblock_failed'), 'error');
                }
                hideModal('unblock-confirm');
            },
            close: () => hideModal('unblock-confirm'),
        }
    );
};
</script>

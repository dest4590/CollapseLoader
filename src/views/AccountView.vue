<template>
    <div class="max-w-xl mx-auto p-6 slide-up">
        <div class="card bg-gradient-to-br from-primary/10 to-primary/5 shadow-md border border-primary/20 mb-6">
            <div class="card-body">
                <div class="flex items-center gap-4 mb-4">
                    <UserAvatar :name="userInfo.nickname || userInfo.username || 'User'" size="lg" />
                    <div class="flex-1">
                        <h2 class="text-xl font-semibold text-primary-focus flex items-center gap-2">
                            {{
                                userInfo.nickname || userInfo.username || 'User'
                            }}
                            <button @click="openNicknameModal" class="btn btn-ghost btn-xs p-1 h-auto min-h-0"
                                :disabled="isLoadingFromCache">
                                <edit-icon class="w-3 h-3" />
                            </button>
                        </h2>
                        <p class="text-base-content/70 text-sm">
                            @{{ userInfo.username || 'username' }}
                        </p>
                        <p class="text-base-content/60 text-xs">
                            {{ userInfo.email || t('account.no_email') }}
                        </p>
                        <div class="flex items-center mt-2 text-sm">
                            <div class="badge" :class="invisibleMode
                                ? 'badge-secondary'
                                : 'badge-success'
                                ">
                                {{
                                    invisibleMode
                                        ? t('time.offline')
                                        : t('time.online')
                                }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div class="card bg-base-200 shadow-md border border-base-300 mb-6">
            <div class="card-body">
                <h2 class="card-title text-lg font-medium text-primary-focus mb-4">
                    {{ t('account.invisible_mode') }}
                </h2>
                <div class="flex items-center justify-between">
                    <div>
                        <h3 class="font-medium">
                            {{ t('account.invisible_mode') }}
                        </h3>
                        <p class="text-sm text-base-content/70">
                            {{ t('account.invisible_mode_description') }}
                        </p>
                    </div>
                    <input type="checkbox" class="checkbox" v-model="invisibleMode" @change="handleInvisibleModeToggle"
                        :disabled="isLoadingFromCache" />
                </div>
                <div v-if="isLoadingFromCache" class="text-sm text-warning">
                    <span v-if="isLoadingFromCache">{{
                        t('account.using_cached_data')
                    }}</span>
                </div>
            </div>
        </div>

        <div class="card bg-base-200 shadow-md border border-base-300 mb-6">
            <div class="card-body">
                <h2 class="card-title text-lg font-medium text-primary-focus mb-4">
                    {{ t('account.streamer_mode') }}
                </h2>
                <div class="flex items-center justify-between">
                    <div>
                        <h3 class="font-medium">
                            {{ t('account.streamer_mode') }}
                        </h3>
                        <p class="text-sm text-base-content/70">
                            {{ t('account.streamer_mode_description') }}
                        </p>
                    </div>
                    <input type="checkbox" class="checkbox" v-model="streamerMode" @change="handleStreamerModeToggle" />
                </div>
            </div>
        </div>

        <div class="card bg-base-200 shadow-md border border-base-300 mb-6">
            <div class="card-body">
                <h2 class="card-title text-lg font-medium text-primary-focus mb-4">
                    {{ t('account.change_password') }}
                </h2>
                <form @submit.prevent="handleChangePassword">
                    <div class="form-control mb-4">
                        <label class="label">
                            <span class="label-text">{{
                                t('account.current_password')
                            }}</span>
                        </label>
                        <input v-model="currentPassword" type="password" :placeholder="t('account.current_password')"
                            class="input input-bordered w-full bg-base-100" required :disabled="isLoadingFromCache" />
                    </div>
                    <div class="form-control mb-4">
                        <label class="label">
                            <span class="label-text">{{
                                t('account.new_password')
                            }}</span>
                        </label>
                        <input v-model="newPassword" type="password" :placeholder="t('account.new_password')"
                            class="input input-bordered w-full bg-base-100" required :disabled="isLoadingFromCache" />
                    </div>
                    <div class="form-control mb-6">
                        <label class="label">
                            <span class="label-text">{{
                                t('account.confirm_password')
                            }}</span>
                        </label>
                        <input v-model="confirmNewPassword" type="password" :placeholder="t('account.confirm_password')"
                            class="input input-bordered w-full bg-base-100" required :disabled="isLoadingFromCache" />
                    </div>
                    <button type="submit" class="btn btn-primary w-full" :disabled="isLoadingFromCache">
                        {{
                            isLoadingFromCache
                                ? t('account.using_cached_data')
                                : t('account.change_password')
                        }}
                    </button>
                </form>
            </div>
        </div>

        <div class="card bg-base-200 shadow-md border border-base-300">
            <div class="card-body">
                <h2 class="card-title text-lg font-medium text-primary-focus mb-4">
                    {{ t('common.logout') }}
                </h2>
                <p class="text-base-content/70 mb-4">
                    {{ t('auth.logout.confirm') }}
                </p>
                <button @click="handleLogout" class="btn btn-error w-full">
                    {{ t('common.logout') }}
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue';
import { useToast } from '../services/toastService';
import { useModal } from '../services/modalService';
import { useI18n } from 'vue-i18n';
import EditNicknameModal from '../components/modals/EditNicknameModal.vue';
import ChangePasswordConfirmModal from '../components/modals/ChangePasswordConfirmModal.vue';
import LogoutConfirmModal from '../components/modals/LogoutConfirmModal.vue';
import UserAvatar from '../components/ui/UserAvatar.vue';
import { userService } from '../services/userService';
import { syncService, type SyncServiceState } from '../services/syncService';
import { EditIcon } from 'lucide-vue-next';
import { apiPost } from '../services/authClient';
import { getCurrentLanguage } from '../i18n';
import { globalUserStatus } from '../composables/useUserStatus';
import { useUser } from '../composables/useUser';

const { t } = useI18n();
const { addToast } = useToast();
const { showModal, hideModal } = useModal();

const currentPassword = ref('');
const newPassword = ref('');
const confirmNewPassword = ref('');
const nickname = ref('');
const cacheStatus = ref({
    hasCache: false,
    isExpired: false,
    lastUpdated: null as string | null,
});

const {
    username,
    email,
    nickname: userNickname,
    isLoading: isLoadingUserData,
    updateUserProfile: updateGlobalUserProfile,
} = useUser();

const invisibleMode = computed({
    get: () => globalUserStatus.isInvisible.value,
    set: (_) => { },
});

const streamerMode = computed({
    get: () => globalUserStatus.isStreamer.value,
    set: (_) => { },
});

const userInfo = computed(() => {
    const nickname = userNickname.value;
    const user = username.value;
    const mail = email.value;

    if (globalUserStatus.isStreamer.value) {
        return {
            nickname: '??????',
            username: 'unknown',
            email: 'unknown@*****.***',
        };
    }

    return {
        nickname: nickname,
        username: user,
        email: mail,
    };
});

const isLoadingFromCache = computed(() => isLoadingUserData.value);

const syncState = ref<SyncServiceState>(syncService.getState());
let unsubscribeSyncService: (() => void) | null = null;

const emit = defineEmits(['logged-out']);

onMounted(async () => {
    cacheStatus.value = userService.getCacheStatus();

    unsubscribeSyncService = syncService.subscribe((state) => {
        syncState.value = state;
    });

    await syncService.initializeSyncStatus();

    nickname.value = userNickname.value || '';

    await globalUserStatus.fetchCurrentStatus();
});

onUnmounted(() => {
    if (unsubscribeSyncService) {
        unsubscribeSyncService();
    }
});

const handleStreamerModeToggle = async () => {
    try {
        globalUserStatus.setStreamerMode(!streamerMode.value);
        addToast(
            streamerMode.value
                ? t('account.streamer_enabled')
                : t('account.streamer_disabled'),
            'success'
        );
    } catch (error) {
        console.error('Failed to toggle streamer mode:', error);
        addToast(t('account.streamer_failed'), 'error');
    }
};

const handleInvisibleModeToggle = async () => {
    try {
        globalUserStatus.setInvisibleMode(!invisibleMode.value);
        await globalUserStatus.forceSyncStatus();
        addToast(
            invisibleMode.value
                ? t('account.invisible_enabled')
                : t('account.invisible_disabled'),
            'success'
        );
    } catch (error) {
        console.error('Failed to toggle invisible mode:', error);
        addToast(t('account.invisible_failed'), 'error');
    }
};

const openNicknameModal = () => {
    showModal(
        'edit-nickname',
        EditNicknameModal,
        { title: t('account.nickname_label') },
        { currentNickname: nickname.value },
        {
            'nickname-updated': async (newNickname: string) => {
                const success = await updateGlobalUserProfile(newNickname);
                if (success) {
                    nickname.value = newNickname;
                    addToast(t('account.nickname_update_success'), 'success');
                } else {
                    addToast(t('account.nickname_update_failed'), 'error');
                }
            },
            close: () => hideModal('edit-nickname'),
        }
    );
};

const handleChangePassword = async () => {
    if (newPassword.value !== confirmNewPassword.value) {
        addToast(t('account.password_mismatch'), 'error');
        return;
    }
    if (newPassword.value.length < 8) {
        addToast(t('account.password_too_short'), 'error');
        return;
    }

    showModal(
        'change-password-confirm',
        ChangePasswordConfirmModal,
        { title: t('account.change_password_confirm_title') },
        {
            currentPassword: currentPassword.value,
            newPassword: newPassword.value,
        },
        {
            confirm: async (passwordData: {
                currentPassword: string;
                newPassword: string;
            }) => {
                try {
                    await userService.changePassword(
                        passwordData.currentPassword,
                        passwordData.newPassword
                    );
                    addToast(t('account.password_change_success'), 'success');
                    currentPassword.value = '';
                    newPassword.value = '';
                    confirmNewPassword.value = '';
                } catch (error: any) {
                    console.error('Failed to change password:', error);
                    if (error.response && error.response.data) {
                        let errorMessage = t('account.password_change_failed');
                        const errors = error.response.data;
                        if (errors.new_password) {
                            errorMessage = errors.new_password.join(' ');
                        } else if (errors.current_password) {
                            errorMessage = errors.current_password.join(' ');
                        } else if (typeof errors === 'string') {
                            errorMessage = errors;
                        } else if (errors.detail) {
                            errorMessage = errors.detail;
                        }
                        addToast(errorMessage, 'error');
                    } else {
                        addToast(t('account.password_change_failed'), 'error');
                    }
                }
                hideModal('change-password-confirm');
            },
            close: () => hideModal('change-password-confirm'),
        }
    );
};

const handleLogout = async () => {
    showModal(
        'logout-confirm',
        LogoutConfirmModal,
        { title: t('account.logout_confirm_title') },
        {},
        {
            confirm: async () => {
                try {
                    await apiPost(
                        '/auth/token/logout/',
                        {},
                        {
                            headers: {
                                Authorization: `Token ${localStorage.getItem('authToken')}`,
                                'Content-Type': 'application/json',
                                'Accept-Language': getCurrentLanguage() || 'en',
                            },
                        }
                    );

                    userService.clearCache();
                    localStorage.removeItem('authToken');
                    addToast(t('auth.logout.success'), 'success');
                    emit('logged-out');
                } catch (error) {
                    console.error('Failed to logout:', error);
                    userService.clearCache();
                    localStorage.removeItem('authToken');
                    emit('logged-out');
                    addToast(t('auth.logout.local_only'), 'warning');
                }
                hideModal('logout-confirm');
            },
            close: () => hideModal('logout-confirm'),
        }
    );
};
</script>

<style scoped>
.slide-up {
    animation: slideUp 0.5s ease-out forwards;
}

@keyframes slideUp {
    0% {
        opacity: 0;
        transform: translateY(20px);
    }

    100% {
        opacity: 1;
        transform: translateY(0);
    }
}
</style>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { router } from './services/router';
import { useI18n } from 'vue-i18n';
import GlobalModal from './components/modals/GlobalModal.vue';
import DevMenuModal from './components/core/DevMenuModal.vue';
import InitialSetupModals from './components/core/InitialSetupModals.vue';
import DownloadProgress from './components/features/download/DownloadProgress.vue';
import Sidebar from './components/layout/Sidebar.vue';
import RegisterPromptModal from './components/modals/social/account/RegisterPromptModal.vue';
import ToastContainer from './components/notifications/ToastContainer.vue';
import { useUser } from './composables/useUser';
import { globalUserStatus } from './composables/useUserStatus';
import { syncService } from './services/syncService';
import { settingsService } from './services/settingsService';
import { useToast } from './services/toastService';
import { themeService } from './services/themeService';
import { updaterService } from './services/updaterService';
import About from './views/About.vue';
import AccountView from './views/AccountView.vue';
import AdminView from './views/AdminView.vue';
import AppLogs from './views/AppLogs.vue';
import FriendsView from './views/FriendsView.vue';
import Home from './views/Home.vue';
import LoginView from './views/LoginView.vue';
import RegisterView from './views/RegisterView.vue';
import Settings from './views/Settings.vue';
import Customization from './views/Customization.vue';
import UserProfileView from './views/UserProfileView.vue';
import News from './views/News.vue';
import CustomClients from './views/CustomClients.vue';
import Marketplace from './views/Marketplace.vue';
import { fetchSettings } from './utils/settings';
import { getDiscordState } from './utils/discord';
import { VALID_TABS } from './utils/tabs';
import { getIsDevelopment } from './utils/isDevelopment';
import Preloader from './components/core/Preloader.vue';

interface Setting<T> {
    description: string;
    value: T;
}

interface Flags {
    disclaimer_shown: Setting<boolean>;
    first_run: Setting<boolean>;
    optional_analytics: Setting<boolean>;
}

import { useAppInit } from './composables/useAppInit';

const { t, locale } = useI18n();

const {
    showPreloader,
    loadingState,
    currentProgress,
    totalSteps,
    isOnline,
    showFirstRunInfo,
    showInitialDisclaimer,
    halloweenActive,
    currentTheme,
    initApp,
    initializeUserDataWrapper
} = useAppInit();

const activeTab = computed(() => router.currentRoute.value as any);
const showDevMenu = ref(false);
const { addToast } = useToast();
const isAuthenticated = ref(false);
const showRegistrationPrompt = ref(false);
const currentUserId = ref<number | null>(null);
const previousTab = ref<string>('home');
const news = ref<any[]>([]);
const unreadNewsCount = ref(0);
const isDev = ref(false);

const sidebarPosition = ref(localStorage.getItem('sidebarPosition') as 'left' | 'right' | 'top' | 'bottom' || 'left');

const updateSidebarPosition = (newPosition: 'left' | 'right' | 'top' | 'bottom') => {
    sidebarPosition.value = newPosition;
    localStorage.setItem('sidebarPosition', newPosition);
};

const mainClasses = computed(() => {
    const base = 'w-full p-6 bg-base-200 overflow-y-auto overflow-x-hidden';
    const pos = sidebarPosition.value;

    if (pos === 'left') return `${base} ml-20 h-screen`;
    if (pos === 'right') return `${base} mr-20 h-screen`;
    if (pos === 'top') return `${base} mt-20 h-[calc(100vh-5rem)]`;
    if (pos === 'bottom') return `${base} h-[calc(100vh-5rem)]`;

    return base;
});


const {
    stopStatusSync
} = globalUserStatus;

const handleUnreadNewsCountUpdated = (count: number) => {
    unreadNewsCount.value = count;
};

const setActiveTab = (tab: string, opts?: { userId?: number | null }) => {
    if (!VALID_TABS.includes(tab)) return;
    previousTab.value = router.currentRoute.value;
    if (opts && Object.prototype.hasOwnProperty.call(opts, 'userId')) {
        currentUserId.value = opts!.userId ?? null;
    } else {
        currentUserId.value = null;
    }
    router.push(tab);
};

const showUserProfile = (userId: number) => {
    previousTab.value = router.currentRoute.value;

    setActiveTab('user-profile', { userId });
};


const checkAuthStatus = () => {
    const token = localStorage.getItem('authToken');
    isAuthenticated.value = !!token;
};

const handleFirstRunAccepted = async () => {
    try {
        await invoke('mark_first_run_shown');
        showFirstRunInfo.value = false;
        const flags = await invoke<Flags>('get_flags');
        if (!flags.disclaimer_shown.value) {
            showInitialDisclaimer.value = true;
        } else {
            if (showPreloader.value) showPreloader.value = false;

            if (!isAuthenticated.value) {
                setTimeout(() => {
                    showRegistrationPrompt.value = true;
                }, 500);
            }
        }
    } catch (error) {
        console.error('Failed to mark first run as shown:', error);
        addToast(t('toast.settings.first_run_save_failed', { error }), 'error');
    }
};

const handleDisclaimerAccepted = async () => {
    try {
        await invoke('mark_disclaimer_shown');
        showInitialDisclaimer.value = false;
        if (showPreloader.value) showPreloader.value = false;

        if (!isAuthenticated.value) {
            setTimeout(() => {
                showRegistrationPrompt.value = true;
            }, 500);
        }
    } catch (error) {
        console.error('Failed to mark disclaimer as shown:', error);
        addToast(
            t('toast.settings.disclaimer_save_failed', { error }),
            'error'
        );
    }
};

const handleOpenDevMenu = () => {
    showDevMenu.value = true;
    addToast(t('toast.dev.menu_opened'), 'info');
};

const closeDevMenu = () => {
    showDevMenu.value = false;
};

const handleLoggedOut = () => {
    isAuthenticated.value = false;
    localStorage.removeItem('authToken');
    setActiveTab('login');
    syncService.destroy();

    clearUserData();

    globalUserStatus.stopStatusSync();
};

const handleLoggedIn = async () => {
    isAuthenticated.value = true;
    setActiveTab('home');

    await initializeUserDataWrapper(isAuthenticated.value);

    await syncService.initializeSyncStatus();
    await syncService.checkAndRestoreOnStartup();

    globalUserStatus.initializeStatusSystem();
};

const handleRegistered = () => {
    setActiveTab('login');
    addToast(t('toast.auth.registration_success'), 'success');
};

const views: Record<string, any> = {
    home: Home,
    news: News,
    settings: Settings,
    about: About,
    customization: Customization,
    custom_clients: CustomClients,
    app_logs: AppLogs,
    account: AccountView,
    login: LoginView,
    register: RegisterView,
    friends: FriendsView,
    'user-profile': UserProfileView,
    admin: AdminView,
    marketplace: Marketplace,
};

const currentView = computed(() => views[activeTab.value] || Home);

const updateDiscordRPC = async (tab?: string) => {
    try {
        const settings = await fetchSettings();
        if (!settings?.discord_rpc_enabled?.value) {
            console.log('Discord RPC is disabled in settings, skipping update');
            return;
        }

        const currentTab = tab || activeTab.value;
        const details = t('discord.details.in_menu');
        const state = getDiscordState(currentTab, (k: string) => t(k));

        await invoke('update_presence', { details, state });
        console.log(`Discord RPC updated for tab: ${currentTab} - ${details}: ${state}`);
    } catch (error) {
        console.error('Failed to update Discord RPC:', error);
    }
};

watch(isAuthenticated, (newVal) => {
    if (newVal) {
        showRegistrationPrompt.value = false;
    }
});

watch(activeTab, async (newTab) => {
    await updateDiscordRPC(newTab);
});

watch(locale, async () => {
    await updateDiscordRPC();
});

const hideRegistrationPrompt = () => {
    showRegistrationPrompt.value = false;
    localStorage.setItem('registrationPromptShown', new Date().toISOString());
};

const handleRegisterPrompt = () => {
    setActiveTab('register');
    showRegistrationPrompt.value = false;
    localStorage.setItem('registrationPromptShown', new Date().toISOString());
};

const { clearUserData } = useUser();



const getTransitionName = () => {
    const tabOrder = [
        'home',
        'custom_clients',
        'friends',
        'settings',
        'customization',
        'app_logs',
        'admin',
        'account',
        'login',
        'register',
        'about',
    ];
    const currentIndex = tabOrder.indexOf(activeTab.value);
    const previousIndex = tabOrder.indexOf(previousTab.value);

    const transitionName =
        currentIndex > previousIndex
            ? 'slide-down'
            : currentIndex < previousIndex
                ? 'slide-up'
                : 'fade-slide';

    return transitionName;
};

onMounted(() => {
    initApp(isAuthenticated, checkAuthStatus, news, unreadNewsCount);
    settingsService.loadSettings();
    checkAuthStatus();

    (async () => {
        isDev.value = await getIsDevelopment();
    })();

    listen('client-launched', async (event) => {
        const payload = event.payload as {
            id: number;
            name: string;
            version?: string;
        };
        console.log(`Client ${payload.name} launched, updating status...`);

        try {
            globalUserStatus.setPlayingClient(`${payload.name} (${payload.version || 'unknown version'})`);
            await settingsService.loadSettings();
            const settings = settingsService.getSettings() as any;
            if (settings.discord_rpc_enabled?.value) {
                await invoke('update_presence', {
                    details: t('discord.details.in_game'),
                    state: payload.name,
                }).catch((error) => {
                    console.error('Failed to update Discord presence:', error);
                });
            }
        } catch (error) {
            console.error('Failed to update playing status:', error);
        }
    });

    listen('client-exited', async (event) => {
        const payload = event.payload as {
            id: number;
            name: string;
            exitCode?: number;
        };
        console.log(`Client ${payload.name} exited, updating status...`);

        try {
            globalUserStatus.setOnline();
            await settingsService.loadSettings();
            const settings = settingsService.getSettings() as any;
            if (settings.discord_rpc_enabled?.value) {
                await invoke('update_presence', {
                    details: t('discord.details.in_menu'),
                    state: t('discord.states.browsing_clients'),
                }).catch((error) => {
                    console.error('Failed to reset Discord presence:', error);
                });
            }
        } catch (error) {
            console.error('Failed to update online status:', error);
        }
    });

    listen('update-user-status', async (event) => {
        const payload = event.payload as {
            status: string;
            currentClient: string | null;
        };

        console.log('Received status update event from backend:', payload);
        console.log('Backend status event ignored to prevent conflicts');
    });

    listen('toast-error', (event) => {
        console.log(event);

        let message: string;
        message = String(event.payload);

        addToast(message, 'error');
    });

    window.addEventListener('beforeunload', () => {
        if (globalUserStatus.isAuthenticated.value) {
            globalUserStatus.setOffline();
        }
    });

    const emergencyHandler = (e: KeyboardEvent) => {
        try {
            if (e.ctrlKey && e.shiftKey && (e.key === 'Home' || e.code === 'Home')) {
                const active = document.activeElement as HTMLElement | null;
                const isTyping = !!active && (active.tagName === 'INPUT' || active.tagName === 'TEXTAREA' || active.isContentEditable);
                if (isTyping) return;

                console.warn('Emergency theme reset triggered via Ctrl+Shift+Home');
                themeService.emergencyReset();

                addToast(
                    t('toast.theme.emergency_reset_done', {
                        action: t('toast.theme.emergency_reset_toggle_instruction')
                    }),
                    'info',
                    8000
                );
            }
        } catch (err) {
            console.error('Error during emergency theme reset:', err);
        }
    };

    window.addEventListener('keydown', emergencyHandler);
    onUnmounted(() => {
        window.removeEventListener('keydown', emergencyHandler);
    });
});

onUnmounted(() => {
    console.log('App unmounting, stopping systems...');
    stopStatusSync();
    updaterService.stopPeriodicCheck();
    window.removeEventListener('beforeunload', () => { });
    console.log('Status sync stopped');
});
</script>

<template>
    <Preloader v-model:show="showPreloader" :is-dev="isDev" :loading-state="loadingState"
        :current-progress="currentProgress" :total-steps="totalSteps" :halloween-active="halloweenActive"
        :current-theme="currentTheme" />

    <InitialSetupModals :show-first-run="showFirstRunInfo" :show-disclaimer="showInitialDisclaimer"
        :current-theme="currentTheme" @first-run-accepted="handleFirstRunAccepted"
        @disclaimer-accepted="handleDisclaimerAccepted" @auto-login="handleLoggedIn" />

    <DevMenuModal :show-dev-menu="showDevMenu" :registerPrompt="showRegistrationPrompt" @close="closeDevMenu" />

    <div :class="['flex h-screen']" v-if="!showPreloader && !showInitialDisclaimer && !showFirstRunInfo">
        <Sidebar :activeTab="activeTab" @changeTab="setActiveTab" @open-dev-menu="handleOpenDevMenu"
            :is-online="isOnline" :is-authenticated="isAuthenticated" :position="sidebarPosition"
            @update:position="updateSidebarPosition" />
        <main :class="mainClasses">
            <transition :name="getTransitionName()" mode="out-in" appear>
                <div :key="activeTab + (currentUserId || '')">
                    <component :is="currentView" @logged-out="handleLoggedOut" @logged-in="handleLoggedIn"
                        @registered="handleRegistered" @change-view="setActiveTab" @show-user-profile="showUserProfile"
                        @back-to-friends="() => setActiveTab('friends')"
                        @unread-count-updated="handleUnreadNewsCountUpdated" :key="activeTab" :is-online="isOnline"
                        :user-id="currentUserId" v-bind="activeTab === 'home' ? { unreadNewsCount } : {}" />
                </div>
            </transition>
        </main>
    </div>

    <DownloadProgress />
    <ToastContainer />
    <GlobalModal />
    <RegisterPromptModal v-model="showRegistrationPrompt" @register="handleRegisterPrompt"
        @cancel="hideRegistrationPrompt" />
</template>

<style scoped>
.loading-status {
    display: flex;
    transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.slide-up-enter-active,
.slide-up-leave-active {
    transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.slide-up-enter-from {
    opacity: 0;
    transform: translateY(-60px);
}

.slide-up-leave-to {
    opacity: 0;
    transform: translateY(60px);
}

.slide-down-enter-active,
.slide-down-leave-active {
    transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.slide-down-enter-from {
    opacity: 0;
    transform: translateY(60px);
}

.slide-down-leave-to {
    opacity: 0;
    transform: translateY(-60px);
}

.fade-slide-enter-active,
.fade-slide-leave-active {
    transition: all 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.fade-slide-enter-from {
    opacity: 0;
    transform: translateY(30px);
}

.fade-slide-leave-to {
    opacity: 0;
    transform: translateY(-30px);
}


.slide-up-appear-active,
.slide-down-appear-active,
.fade-slide-appear-active {
    transition: all 0.6s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.slide-up-appear-from,
.slide-down-appear-from,
.fade-slide-appear-from {
    opacity: 0;
    transform: translateY(20px) scale(0.98);
}

.slide-fade-enter-active,
.slide-fade-leave-active {
    transition: all 0.4s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.slide-fade-enter-from {
    transform: translateY(15px);
    opacity: 0;
}

.slide-fade-leave-to {
    transform: translateY(-15px);
    opacity: 0;
}
</style>

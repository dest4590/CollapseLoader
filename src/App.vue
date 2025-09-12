<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { Vue3Lottie } from 'vue3-lottie';
import preloader from './assets/misc/preloader.json';
import GlobalModal from './components/modals/GlobalModal.vue';
import DevMenuModal from './components/core/DevMenuModal.vue';
import InitialSetupModals from './components/core/InitialSetupModals.vue';
import DownloadProgress from './components/features/download/DownloadProgress.vue';
import Sidebar from './components/layout/Sidebar.vue';
import ClientCrashModal from './components/modals/clients/ClientCrashModal.vue';
import RegisterPromptModal from './components/modals/social/account/RegisterPromptModal.vue';
import ToastContainer from './components/notifications/ToastContainer.vue';
import BootLogs from './components/core/BootLogs.vue';
import { globalFriends } from './composables/useFriends';
import { useUser } from './composables/useUser';
import { globalUserStatus } from './composables/useUserStatus';
import { useModal } from './services/modalService';
import { syncService } from './services/syncService';
import { useToast } from './services/toastService';
import { themeService } from './services/themeService';
import { updaterService } from './services/updaterService';
import { bootLogService } from './services/bootLogService';
import About from './views/About.vue';
import AccountView from './views/AccountView.vue';
import AdminView from './views/AdminView.vue';
import AppLogs from './views/AppLogs.vue';
import BlockedUsersView from './views/BlockedUsersView.vue';
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
import { apiGet } from './services/apiClient';
import { getCurrentLanguage } from './i18n';
import { fetchSettings, applyLanguageOnStartup, applyThemeOnStartup } from './utils/settings';
import { getDiscordState } from './utils/discord';
import { VALID_TABS } from './utils/tabs';
import { getIsDevelopment } from './utils/isDevelopment';

interface Setting<T> {
    description: string;
    value: T;
}

interface Flags {
    disclaimer_shown: Setting<boolean>;
    first_run: Setting<boolean>;
    optional_analytics: Setting<boolean>;
}

interface AppSettings {
    theme: Setting<string>;
    language: Setting<string>;
    [key: string]: Setting<any>;
}

const { t, locale } = useI18n();

const activeTab = ref<'home' | 'settings' | 'app_logs' | 'customization' | 'custom_clients' | 'about' | 'account' | 'login' | 'register' | 'friends' | 'user-profile' | 'blocked-users' | 'admin' | 'news' | 'marketplace'>('home');
const showPreloader = ref(true);
const contentVisible = ref(false);
const loadingState = ref(t('preloader.initializing'));
const loadingStates = [
    t('preloader.initializing'),
    t('preloader.connecting_servers'),
];
const currentProgress = ref(0);
const totalSteps = ref(4);
const showInitialDisclaimer = ref(false);
const showFirstRunInfo = ref(false);
const initialModalsLoaded = ref(false);
const showDevMenu = ref(false);
const { addToast } = useToast();
const { showModal } = useModal();
const isOnline = ref(true);
const currentTheme = ref('dark');
const isAuthenticated = ref(false);
const showRegistrationPrompt = ref(false);
const currentUserId = ref<number | null>(null);
const isNavigatingToProfile = ref(false);
const previousTab = ref<string>('home');
const news = ref<any[]>([]);
const unreadNewsCount = ref(0);
const isDev = ref(false);


const { loadUserData, displayName, isAuthenticated: userAuthenticated } = useUser();
const {
    friends,
    onlineFriendsCount,
    loadFriendsData,
    isLoading: friendsLoading
} = globalFriends;

const {
    isOnline: userOnline,
    connectionStatus,
    initializeStatusSystem,
    stopStatusSync
} = globalUserStatus;

const handleUnreadNewsCountUpdated = (count: number) => {
    unreadNewsCount.value = count;
};

const setActiveTab = (tab: string) => {
    if (!VALID_TABS.includes(tab)) return;
    previousTab.value = activeTab.value;
    isNavigatingToProfile.value = false;
    activeTab.value = tab as any;
    currentUserId.value = null;
};

const showUserProfile = (userId: number | string) => {
    previousTab.value = activeTab.value;
    isNavigatingToProfile.value = true;

    if (userId === 'blocked-users') {
        activeTab.value = 'blocked-users';
        currentUserId.value = null;
    } else {
        currentUserId.value = userId as number;
        activeTab.value = 'user-profile';
    }

    setTimeout(() => (isNavigatingToProfile.value = false), 600);
};


const checkAuthStatus = () => {
    const token = localStorage.getItem('authToken');
    isAuthenticated.value = !!token;
};

const fetchNewsAndUpdateUnreadCount = async () => {
    try {
        const currentLanguage = getCurrentLanguage() || 'en';
        const response = await apiGet('/news/', {
            headers: {
                'Accept-Language': currentLanguage,
                'Content-Type': 'application/json',
            },
        });
        const allNews = response as any[];
        let filteredNews = allNews.filter(
            (article) => article.language === currentLanguage
        );
        news.value = filteredNews;

        const getNewsUniqueId = (article: any) =>
            `news_${article.language}_${article.id}`;
        const readNews = JSON.parse(localStorage.getItem('readNews') || '[]');
        unreadNewsCount.value = news.value.filter(
            (article) => !readNews.includes(getNewsUniqueId(article))
        ).length;
    } catch (e) {
        console.error('Failed to fetch news:', e);
        unreadNewsCount.value = 0;
    }
};

const initApp = async () => {
    bootLogService.start();
    bootLogService.systemInit();

    try {
        await invoke('initialize_rpc');
    } catch (error) {
        console.error('Failed to initialize Discord RPC:', error);
        bootLogService.addCustomLog('WARN', 'rpc', `Discord RPC init failed: ${String(error)}`);
    }

    await applyThemeOnStartup();

    bootLogService.themeApplied(currentTheme.value);

    await applyLanguageOnStartup();

    bootLogService.languageApplied(locale.value || getCurrentLanguage() || 'en');

    const { getToastPosition } = useToast();
    getToastPosition();
    bootLogService.toastSystemReady();

    showPreloader.value = true;
    currentProgress.value = 0;
    totalSteps.value = 4;

    bootLogService.eventListenersInit();

    listen('client-crashed', (event) => {
        const payload = event.payload as {
            id: number;
            name: string;
            error?: string;
        };
        addToast(
            t('toast.client.crashed', {
                name: payload.name,
                error: payload.error || '',
            }),
            'error'
        );
    });

    listen('client-crash-details', (event) => {
        const payload = event.payload as {
            id: number;
            name: string;
            logs: string[];
            error?: string;
        };
        showModal(
            `client-crash-${payload.id}`,
            ClientCrashModal,
            {
                title: t('modal.client_crash.title', { name: payload.name }),
                contentClass: 'wide',
            },
            {
                clientName: payload.name,
                clientId: payload.id,
                logs: payload.logs,
                error: payload.error,
            }
        );
    });

    bootLogService.eventListenersReady();

    loadingState.value = loadingStates[0];
    currentProgress.value = 1;
    await new Promise((resolve) => setTimeout(resolve, 1000));

    try {
        bootLogService.serverConnectivityCheck();
        const connectivity = await invoke<{
            cdn_online: boolean;
            auth_online: boolean;
        }>('get_server_connectivity_status');
        isOnline.value = connectivity.cdn_online && connectivity.auth_online;
        console.log('Server connectivity status:', connectivity);

        if (connectivity.cdn_online) bootLogService.cdnOnline();
        else bootLogService.cdnOffline();

        if (connectivity.auth_online) bootLogService.webApiOnline();
        else bootLogService.webApiOffline();

        if (!isOnline.value) {
            let offlineMessage = t('toast.server.offline_base');
            if (!connectivity.cdn_online && !connectivity.auth_online) {
                offlineMessage += t('toast.server.cdn_and_api_offline');
            } else if (!connectivity.cdn_online) {
                offlineMessage += t('toast.server.cdn_offline');
            } else {
                offlineMessage += t('toast.server.api_offline');
            }
            addToast(offlineMessage, 'warning');
        }
    } catch (error) {
        console.error('Failed to get server connectivity status:', error);
        isOnline.value = false;
        addToast(t('toast.server.offline'), 'error');
        bootLogService.addCustomLog('ERROR', 'network', `Connectivity check failed: ${String(error)}`);
    }

    loadingState.value = loadingStates[1];
    currentProgress.value = 2;
    await new Promise((resolve) => setTimeout(resolve, 1000));

    try {
        bootLogService.apiInit();
        await invoke('initialize_api');
        bootLogService.apiInitSuccess();
    } catch (error) {
        console.error('Failed to initialize API:', error);
        addToast(t('toast.server.api_init_failed', { error }), 'error');
        bootLogService.apiInitFailed();
    }

    bootLogService.authCheck();
    checkAuthStatus();
    if (isAuthenticated.value) bootLogService.authSuccess();
    else bootLogService.authSkipped();

    if (isAuthenticated.value && isOnline.value) {
        try {
            bootLogService.userDataInit();
            await initializeUserData();
            bootLogService.userDataSuccess();

            globalUserStatus.initializeStatusSystem();
            bootLogService.syncInit();
            bootLogService.syncReady();
        } catch (error) {
            console.error('Failed to initialize user data on startup:', error);
            bootLogService.userDataFailed();
        }
    }

    currentProgress.value = 3;
    await new Promise((resolve) => setTimeout(resolve, 1000));

    try {
        const currentFlags = await invoke<Flags>('get_flags');
        if (currentFlags.first_run.value) {
            showFirstRunInfo.value = true;
        } else if (!currentFlags.disclaimer_shown.value) {
            showInitialDisclaimer.value = true;
        }
        initialModalsLoaded.value = true;
        bootLogService.flagsLoaded();
    } catch (error) {
        console.error('Failed to load flags for initial modals:', error);
        addToast(t('toast.settings.flags_load_failed', { error }), 'error');
        initialModalsLoaded.value = true;
        bootLogService.flagsLoadFailed();
    }

    await new Promise<void>((resolve) => {
        const checkInterval = setInterval(() => {
            if (initialModalsLoaded.value) {
                clearInterval(checkInterval);
                resolve();
            }
        }, 100);
    });

    try {
        await fetchNewsAndUpdateUnreadCount();
        console.log('News loaded successfully on startup');
    } catch (error) {
        console.error('Failed to load news on startup:', error);
    }

    updaterService.startPeriodicCheck(t);

    currentProgress.value = 4;
    loadingState.value = t('preloader.ready');

    await new Promise((resolve) => setTimeout(resolve, 1000));

    const preloaderElement = document.querySelector(
        '#preloader'
    ) as HTMLElement;
    if (preloaderElement) {
        preloaderElement.style.opacity = '0';
        preloaderElement.classList.add('animate-out');

        setTimeout(() => {
            showPreloader.value = false;
            try {
                document.documentElement.classList.add('app-ready');
            } catch (e) {
                console.error('Failed to add app-ready class:', e);
            }
            setTimeout(() => {
                contentVisible.value = true;
                bootLogService.systemReady();
                try {
                    bootLogService.clear();
                } catch (e) {
                    console.error('Failed to clear boot logs:', e);
                }
            }, 80);
        }, 800);
    } else {
        showPreloader.value = false;
    }
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

const handleThemeChanged = async (newTheme: string) => {
    currentTheme.value = newTheme;
    try {
        const settings = await invoke<AppSettings>('get_settings');
        const newSettings = {
            ...settings,
            theme: { ...settings.theme, value: newTheme },
        };
        await invoke('save_settings', { inputSettings: newSettings });
    } catch (error) {
        console.error('Failed to save theme from initial setup:', error);
        addToast(t('toast.settings.theme_save_failed', { error }), 'error');
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
    activeTab.value = 'login';
    syncService.destroy();

    clearUserData();

    globalUserStatus.stopStatusSync();
};

const handleLoggedIn = async () => {
    isAuthenticated.value = true;
    activeTab.value = 'home';

    await initializeUserData();

    syncService.initializeSyncStatus();

    globalUserStatus.initializeStatusSystem();
};

const handleRegistered = () => {
    activeTab.value = 'login';
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
    'blocked-users': BlockedUsersView,
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

const initializeUserData = async () => {
    if (!isAuthenticated.value || !isOnline.value) return;

    try {
        await loadUserData();
        console.log(`User loaded: ${displayName.value || 'Unknown'}`);

        initializeStatusSystem();
        console.log(`Status system initialized, connection: ${connectionStatus.value}`);

        await loadFriendsData();
        console.log(`Friends loaded: ${friends.value.length} total, ${onlineFriendsCount.value} online`);

        console.log(
            'User data and friends system initialized successfully on startup'
        );
        console.log(`Loading state: ${friendsLoading.value ? 'Loading...' : 'Complete'}`);
        console.log(`User authentication: ${userAuthenticated.value ? 'Authenticated' : 'Not authenticated'}`);
        console.log(`User online status: ${userOnline.value ? 'Online' : 'Offline'}`);
    } catch (error) {
        console.error('Failed to initialize user data on startup:', error);
    }
};

const getTransitionName = () => {
    if (isNavigatingToProfile.value) {
        return 'profile-slide';
    }

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
    initApp();
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

            const settings = await invoke<AppSettings>('get_settings');
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

            const settings = await invoke<AppSettings>('get_settings');
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
    <div id="preloader" v-if="showPreloader" class="fixed inset-0 bg-base-300 flex items-center justify-center">
        <BootLogs v-if="isDev" :current-progress="currentProgress / totalSteps" :loading-state="loadingState" />

        <div class="flex flex-col items-center justify-center h-full w-screen relative z-[10]">
            <div class="w-48 h-48 animate-pulse-subtle">
                <Vue3Lottie :animation-data="preloader" :height="200" :width="200" />
            </div>
            <div class="loading-status mt-6">
                <transition name="slide-fade" mode="out-in">
                    <span :key="loadingState" class="text-lg font-medium"
                        :class="{ invert: currentTheme === 'light' }">{{
                            loadingState
                        }}</span>
                </transition>
            </div>
            <div class="w-80 progress-container mt-4" :class="{ invert: currentTheme === 'light' }">
                <div class="bg-base-100 rounded-full h-3 overflow-hidden shadow-inner progress-track">
                    <div class="bg-primary h-full rounded-full transition-all duration-700 ease-out progress-fill"
                        :style="{
                            width: `${(currentProgress / totalSteps) * 100}%`,
                        }"></div>
                </div>
                <div class="text-center mt-4 text-sm opacity-60">
                    {{ currentProgress }} / {{ totalSteps }}
                </div>
            </div>
        </div>
    </div>

    <InitialSetupModals :show-first-run="showFirstRunInfo" :show-disclaimer="showInitialDisclaimer"
        :current-theme="currentTheme" @first-run-accepted="handleFirstRunAccepted"
        @disclaimer-accepted="handleDisclaimerAccepted" @theme-changed="handleThemeChanged"
        @auto-login="handleLoggedIn" />

    <DevMenuModal :show-dev-menu="showDevMenu" :registerPrompt="showRegistrationPrompt" @close="closeDevMenu" />

    <div :class="['flex h-screen', contentVisible ? 'content-entered' : 'content-hidden']"
        v-if="!showPreloader && !showInitialDisclaimer && !showFirstRunInfo">
        <Sidebar :activeTab="activeTab" @changeTab="setActiveTab" @open-dev-menu="handleOpenDevMenu"
            :is-online="isOnline" :is-authenticated="isAuthenticated" />
        <main class="ml-20 w-full p-6 bg-base-200 min-h-screen overflow-scroll overflow-x-hidden">
            <transition :name="getTransitionName()" mode="out-in" appear>
                <div :class="{ 'profile-transition': isNavigatingToProfile }" :key="activeTab + (currentUserId || '')">
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
#preloader {
    position: fixed;
    width: 100%;
    height: 100%;
    left: 0;
    top: 0;
    background-color: rgba(0, 0, 0, 0.96);
    z-index: 1337;
    transition:
        opacity 0.5s,
        transform 0.8s,
        filter 0.8s,
        background-color 1s;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
}

#preloader.animate-out {
    background-color: rgba(0, 0, 0, 0);
}

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
    filter: blur(2px);
}

.fade-slide-leave-to {
    opacity: 0;
    transform: translateY(-30px);
    filter: blur(2px);
}

.profile-slide-enter-active,
.profile-slide-leave-active {
    transition: all 0.7s cubic-bezier(0.23, 1, 0.32, 1);
}

.profile-slide-enter-from {
    opacity: 0;
    transform: translateX(80px) scale(0.9);
}

.profile-slide-leave-to {
    opacity: 0;
    transform: translateX(-30px);
}

.profile-transition {
    transition:
        transform 0.7s cubic-bezier(0.23, 1, 0.32, 1),
        opacity 0.7s cubic-bezier(0.23, 1, 0.32, 1);
}

.slide-up-appear-active,
.slide-down-appear-active,
.fade-slide-appear-active,
.profile-slide-appear-active {
    transition: all 0.6s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.slide-up-appear-from,
.slide-down-appear-from,
.fade-slide-appear-from,
.profile-slide-appear-from {
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

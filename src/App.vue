<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { computed, onMounted, onUnmounted, ref, watch } from 'vue';
import { useI18n } from 'vue-i18n';
import { Vue3Lottie } from 'vue3-lottie';
import preloader from './assets/misc/preloader.json';
import GlobalModal from './components/common/GlobalModal.vue';
import DevMenuModal from './components/core/DevMenuModal.vue';
import InitialSetupModals from './components/core/InitialSetupModals.vue';
import DownloadProgress from './components/features/download/DownloadProgress.vue';
import Sidebar from './components/layout/Sidebar.vue';
import ClientCrashModal from './components/modals/ClientCrashModal.vue';
import RegisterPromptModal from './components/modals/RegisterPromptModal.vue';
import ToastContainer from './components/notifications/ToastContainer.vue';
import { useFriends } from './composables/useFriends';
import { useUser } from './composables/useUser';
import { globalUserStatus } from './composables/useUserStatus';
import { changeLanguage } from './i18n';
import { useModal } from './services/modalService';
import { syncService } from './services/syncService';
import { useToast } from './services/toastService';
import { themeService } from './services/themeService';
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
import Theme from './views/Theme.vue';
import UserProfileView from './views/UserProfileView.vue';
import News from './views/News.vue';
import { apiGet } from './services/apiClient';
import { getCurrentLanguage } from './i18n';

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

const activeTab = ref<
    | 'home'
    | 'settings'
    | 'app_logs'
    | 'theme'
    | 'about'
    | 'account'
    | 'login'
    | 'register'
    | 'friends'
    | 'user-profile'
    | 'blocked-users'
    | 'admin'
    | 'news'
>('home');
const showPreloader = ref(true);
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

const handleUnreadNewsCountUpdated = (count: number) => {
    unreadNewsCount.value = count;
};

const setActiveTab = (tab: string) => {
    if (
        [
            'home',
            'settings',
            'app_logs',
            'theme',
            'about',
            'account',
            'login',
            'register',
            'friends',
            'admin',
            'news',
        ].includes(tab)
    ) {
        previousTab.value = activeTab.value;
        isNavigatingToProfile.value = false;
        activeTab.value = tab as
            | 'home'
            | 'settings'
            | 'app_logs'
            | 'theme'
            | 'about'
            | 'account'
            | 'login'
            | 'register'
            | 'friends'
            | 'admin'
            | 'news';
        currentUserId.value = null;
    }
};

const showUserProfile = (userId: number | string) => {
    previousTab.value = activeTab.value;

    if (userId === 'blocked-users') {
        isNavigatingToProfile.value = true;
        activeTab.value = 'blocked-users';
        currentUserId.value = null;
    } else {
        isNavigatingToProfile.value = true;
        currentUserId.value = userId as number;
        activeTab.value = 'user-profile';
    }

    setTimeout(() => {
        isNavigatingToProfile.value = false;
    }, 600);
};

const applyLanguageOnStartup = async () => {
    try {
        const currentSettings = await invoke<AppSettings>('get_settings');
        if (currentSettings.language && currentSettings.language.value) {
            await changeLanguage(currentSettings.language.value);
        } else {
            const localLanguage = localStorage.getItem('language') || 'en';
            await changeLanguage(localLanguage);
        }
    } catch (error) {
        console.error('Failed to apply language on startup:', error);
        const localLanguage = localStorage.getItem('language') || 'en';
        await changeLanguage(localLanguage);
    }
};

const applyThemeOnStartup = async () => {
    try {
        const currentSettings = await invoke<AppSettings>('get_settings');
        if (currentSettings.theme && currentSettings.theme.value) {
            document.documentElement.setAttribute(
                'data-theme',
                currentSettings.theme.value
            );
            currentTheme.value = currentSettings.theme.value;
        } else {
            const localTheme = localStorage.getItem('theme');
            if (localTheme && ['light', 'dark'].includes(localTheme)) {
                document.documentElement.setAttribute('data-theme', localTheme);
                currentTheme.value = localTheme;
            } else {
                document.documentElement.setAttribute('data-theme', 'dark');
                currentTheme.value = 'dark';
            }
        }

        themeService.loadCardSettings();
    } catch (error) {
        console.error('Failed to apply theme on startup:', error);
        const localTheme = localStorage.getItem('theme');
        if (localTheme && ['light', 'dark'].includes(localTheme)) {
            document.documentElement.setAttribute('data-theme', localTheme);
            currentTheme.value = localTheme;
        } else {
            document.documentElement.setAttribute('data-theme', 'dark');
            currentTheme.value = 'dark';
        }

        themeService.loadCardSettings();
    }
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
        const allNews = response.data as any[];
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
    } catch (err) {
        unreadNewsCount.value = 0;
    }
};

const initApp = async () => {
    try {
        await invoke('initialize_rpc');

    } catch (error) {
        console.error('Failed to initialize Discord RPC:', error);
    }

    await applyThemeOnStartup();

    await applyLanguageOnStartup();

    const { getToastPosition } = useToast();
    getToastPosition();

    showPreloader.value = true;
    currentProgress.value = 0;
    totalSteps.value = 4;

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

    loadingState.value = loadingStates[0];
    currentProgress.value = 1;
    await new Promise((resolve) => setTimeout(resolve, 1000));

    try {
        const connectivity = await invoke<{
            cdn_online: boolean;
            auth_online: boolean;
        }>('get_server_connectivity_status');
        isOnline.value = connectivity.cdn_online && connectivity.auth_online;
        console.log('Server connectivity status:', connectivity);

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
    }

    loadingState.value = loadingStates[1];
    currentProgress.value = 2;
    await new Promise((resolve) => setTimeout(resolve, 1000));

    try {
        await invoke('initialize_api');
    } catch (error) {
        console.error('Failed to initialize API:', error);
        addToast(t('toast.server.api_init_failed', { error }), 'error');
    }

    checkAuthStatus();

    if (isAuthenticated.value && isOnline.value) {
        try {
            await initializeUserData();

            globalUserStatus.initializeStatusSystem();
        } catch (error) {
            console.error('Failed to initialize user data on startup:', error);
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
    } catch (error) {
        console.error('Failed to load flags for initial modals:', error);
        addToast(t('toast.settings.flags_load_failed', { error }), 'error');
        initialModalsLoaded.value = true;
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
    theme: Theme,
    app_logs: AppLogs,
    account: AccountView,
    login: LoginView,
    register: RegisterView,
    friends: FriendsView,
    'user-profile': UserProfileView,
    'blocked-users': BlockedUsersView,
    admin: AdminView,
};

const currentView = computed(() => views[activeTab.value] || Home);

const updateDiscordRPC = async (tab?: string) => {
    try {
        const settings = await invoke<AppSettings>('get_settings');
        if (!settings.discord_rpc_enabled?.value) {
            console.log('Discord RPC is disabled in settings, skipping update');
            return;
        }

        const currentTab = tab || activeTab.value;

        let details = t('discord.details.in_menu');
        let state;

        console.log(`Updating Discord RPC for tab: ${currentTab}`);

        switch (currentTab) {
            case 'home':
                state = t('discord.states.browsing_clients');
                break;
            case 'news':
                state = t('discord.states.browsing_news');
                break;
            case 'settings':
                state = t('discord.states.configuring_settings');
                break;
            case 'friends':
                state = t('discord.states.browsing_friends');
                break;
            case 'blocked-users':
                state = t('discord.states.configuring_settings');
                break;
            case 'theme':
                state = t('discord.states.enjoying_visuals');
                break;
            case 'app_logs':
                state = t('discord.states.watching_client_behavior');
                break;
            case 'user-profile':
                state = t('discord.states.in_profile');
                break;
            case 'about':
                state = t('discord.states.watching_about');
                break;
            case 'login':
                state = t('discord.states.logging_in');
                break;
            default:
                state = t('discord.states.browsing_clients');
                break;
        }

        await invoke('update_presence', { details, state });
        console.log(
            `Discord RPC updated for tab: ${currentTab} - ${details}: ${state}`
        );
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

const { loadUserData: loadGlobalUserData, clearUserData } = useUser();

const initializeUserData = async () => {
    if (!isAuthenticated.value || !isOnline.value) return;

    try {
        await loadGlobalUserData();

        const { loadFriendsData } = useFriends();
        await loadFriendsData();

        console.log(
            'User data and friends data initialized successfully on startup'
        );
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
        'friends',
        'settings',
        'theme',
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
    listen('client-launched', async (event) => {
        const payload = event.payload as {
            id: number;
            name: string;
            version?: string;
        };
        console.log(`Client ${payload.name} launched, updating status...`);

        try {
            globalUserStatus.setPlayingClient(payload.name, payload.version);

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
            clientVersion: string | null;
        };
        console.log('Received status update event from backend:', payload);

        console.log('Backend status event ignored to prevent conflicts');
    });

    window.addEventListener('beforeunload', () => {
        if (globalUserStatus.isAuthenticated.value) {
            globalUserStatus.setOffline();
        }
    });
});

onUnmounted(() => {
    globalUserStatus.stopStatusSync();
    window.removeEventListener('beforeunload', () => { });
});
</script>

<template>
    <div id="preloader" v-if="showPreloader" class="fixed inset-0 bg-base-300 flex items-center justify-center">
        <div class="flex flex-col items-center justify-center h-full w-screen relative z-10">
            <div class="w-48 h-48 animate-pulse-subtle" :class="{ invert: currentTheme === 'light' }">
                <Vue3Lottie :animation-data="preloader" :height="200" :width="200" />
            </div>
            <div class="loading-status mt-6">
                <transition name="slide-fade" mode="out-in">
                    <span :key="loadingState" class="text-lg font-medium">{{
                        loadingState
                        }}</span>
                </transition>
            </div>
            <div class="w-80 progress-container mt-4">
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

    <div class="flex animate-fadeIn h-screen" v-if="!showPreloader && !showInitialDisclaimer && !showFirstRunInfo">
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
    transform: translateY(20px);
    opacity: 0;
}

.slide-fade-leave-to {
    transform: translateY(-20px);
    opacity: 0;
}

@media (prefers-reduced-motion: reduce) {

    .slide-up-enter-active,
    .slide-up-leave-active,
    .slide-down-enter-active,
    .slide-down-leave-active,
    .fade-slide-enter-active,
    .fade-slide-leave-active,
    .profile-slide-enter-active,
    .profile-slide-leave-active {
        transition: opacity 0.2s ease;
    }

    .slide-up-enter-from,
    .slide-up-leave-to,
    .slide-down-enter-from,
    .slide-down-leave-to,
    .fade-slide-enter-from,
    .fade-slide-leave-to,
    .profile-slide-enter-from,
    .profile-slide-leave-to {
        transform: none;
        filter: none;
    }
}
</style>

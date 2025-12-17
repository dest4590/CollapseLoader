import { ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { bootLogService } from '../services/bootLogService';
import { applyThemeOnStartup, applyLanguageOnStartup } from '../utils/settings';
import { applyCursorForEvent, isHalloweenEvent } from '../utils/events';
import { useToast } from '../services/toastService';
import { globalUserStatus } from './useUserStatus';
import { useUser } from './useUser';
import { globalFriends } from './useFriends';
import { updaterService } from '../services/updaterService';
import { getCurrentLanguage } from '../i18n';
import { useModal } from '../services/modalService';
import ClientCrashModal from '../components/modals/clients/ClientCrashModal.vue';
import { apiGet } from '../services/apiClient';

interface Flags {
    disclaimer_shown: { value: boolean };
    first_run: { value: boolean };
    optional_analytics: { value: boolean };
}

export function useAppInit() {
    const { t, locale } = useI18n();
    const { addToast } = useToast();
    const { showModal } = useModal();
    const { loadUserData } = useUser();
    const { loadFriendsData } = globalFriends;
    const { initializeStatusSystem } = globalUserStatus;

    const showPreloader = ref(true);
    const contentVisible = ref(false);
    const loadingState = ref(t('preloader.initializing'));
    const loadingStates = [
        t('preloader.initializing'),
        t('preloader.connecting_servers'),
    ];
    const currentProgress = ref(0);
    const totalSteps = ref(4);
    const isOnline = ref(true);
    const initialModalsLoaded = ref(false);
    const showFirstRunInfo = ref(false);
    const showInitialDisclaimer = ref(false);
    const halloweenActive = ref(isHalloweenEvent());
    const currentTheme = ref('dark');

    const initializeUserDataWrapper = async (isAuthenticated: boolean) => {
        if (!isAuthenticated || !isOnline.value) return;

        try {
            await loadUserData();
            initializeStatusSystem();
            await loadFriendsData();
        } catch (error) {
            console.error('Failed to initialize user data on startup:', error);
        }
    };

    const fetchNewsAndUpdateUnreadCount = async (news: any, unreadNewsCount: any) => {
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
                (article: any) => article.language === currentLanguage
            );
            news.value = filteredNews;

            const getNewsUniqueId = (article: any) =>
                `news_${article.language}_${article.id}`;
            const readNews = JSON.parse(localStorage.getItem('readNews') || '[]');
            unreadNewsCount.value = news.value.filter(
                (article: any) => !readNews.includes(getNewsUniqueId(article))
            ).length;
        } catch (e) {
            console.error('Failed to fetch news:', e);
            unreadNewsCount.value = 0;
        }
    };

    const initApp = async (isAuthenticated: any, checkAuthStatus: () => void, news: any, unreadNewsCount: any) => {
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

        await applyCursorForEvent();

        bootLogService.cursorApplied();

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

        try {
            bootLogService.serverConnectivityCheck();
            const connectivity = await invoke<{
                cdn_online: boolean;
                auth_online: boolean;
            }>('get_server_connectivity_status');
            isOnline.value = connectivity.cdn_online && connectivity.auth_online;

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
                await initializeUserDataWrapper(isAuthenticated.value);
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

        try {
            await fetchNewsAndUpdateUnreadCount(news, unreadNewsCount);
        } catch (error) {
            console.error('Failed to load news on startup:', error);
        }

        updaterService.startPeriodicCheck(t);

        currentProgress.value = 4;
        loadingState.value = t('preloader.ready');

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

    return {
        showPreloader,
        contentVisible,
        loadingState,
        currentProgress,
        totalSteps,
        isOnline,
        initialModalsLoaded,
        showFirstRunInfo,
        showInitialDisclaimer,
        halloweenActive,
        currentTheme,
        initApp,
        initializeUserDataWrapper
    };
}

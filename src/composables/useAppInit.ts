import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { bootLogService } from "@services/logs/bootLogService";
import { applyLanguageOnStartup, applyThemeOnStartup } from "../utils/settings";
import { useToast } from "@shared/composables/useToast";
import { globalUserStatus } from "@features/auth/useUserStatus";
import { useUser } from "@features/auth/useUser";
import { userService } from "@features/auth/userService";
import { globalFriends } from "@features/friends/useFriends";
import { updaterService } from "@services/updater/updaterService";
import { syncService } from "../services/syncService";
import { getCurrentLanguage } from "@services/i18n";
import { useModal } from "@shared/composables/useModal";
import ClientCrashModal from "@features/clients/modals/ClientCrashModal.vue";
import { apiGet } from "@api/clients/internal";
import { webSocketService } from "@services/network/webSocketService";
import { wait } from "@shared/utils/utils";

interface Flags {
    disclaimer_shown: { value: boolean };
    first_run: { value: boolean };
    optional_analytics: { value: boolean };
}

export function useAppInit() {
    const { t, locale } = useI18n();
    const { addToast } = useToast();
    const { showModal } = useModal();
    const { loadUserData, hydrateUser } = useUser();
    const { loadFriendsData, hydrateFriends } = globalFriends;
    const { initializeStatusSystem } = globalUserStatus;

    const showPreloader = ref(true);
    const contentVisible = ref(false);
    const currentProgress = ref(0);
    const isOnline = ref(true);
    const initialModalsLoaded = ref(false);
    const showFirstRunInfo = ref(false);
    const showInitialDisclaimer = ref(false);
    const apiInitialized = ref(false);

    const loadingStates = [
        t("preloader.initializing"),
        t("preloader.connecting_servers"),
        t("preloader.loading_user_data"),
    ];
    const loadingState = ref(loadingStates[0]);

    const initialTheme =
        (document.documentElement.getAttribute("data-theme") as string) ||
        localStorage.getItem("theme") ||
        "dark";
    const currentTheme = ref(initialTheme);

    const initializeUserDataWrapper = async (isAuthenticated: boolean) => {
        if (!isAuthenticated || !isOnline.value) return;

        try {
            const initData = await userService.initializeUserFull();
            const { user, friends } = initData;

            hydrateUser(user.profile, {
                id: user.id,
                username: user.username,
                email: user.email,
                role: user.role,
                created_at: user.created_at,
                updated_at: user.updated_at,
                last_login_at: user.last_login_at ?? null,
            });
            hydrateFriends(friends);
            initializeStatusSystem();
            await syncService.restoreFromInitData(initData);
        } catch (error) {
            console.error("Init fallback:", error);
            await performFallbackInit();
        }
    };

    const performFallbackInit = async () => {
        try {
            await Promise.all([
                loadUserData(),
                loadFriendsData(),
                syncService.checkAndRestoreOnStartup(),
            ]);
            initializeStatusSystem();
        } catch (error) {
            console.error("Fallback init failed:", error);
        }
    };

    const fetchNewsAndUpdateUnreadCount = async (
        news: any,
        unreadNewsCount: any
    ) => {
        try {
            const currentLanguage = getCurrentLanguage() || "en";
            const response = await apiGet("/news/", {
                headers: {
                    "Accept-Language": currentLanguage,
                    "Content-Type": "application/json",
                },
            });

            const allNews = normalizeNewsResponse(response);
            news.value = allNews.filter(
                (item: any) => item.language === currentLanguage
            );

            updateUnreadCount(news.value, unreadNewsCount);
        } catch (error) {
            console.error("Failed to fetch news:", error);
            unreadNewsCount.value = 0;
        }
    };

    const normalizeNewsResponse = (response: any): any[] => {
        if (Array.isArray(response)) return response;
        if (Array.isArray(response?.data)) return response.data;
        if (response && typeof response === "object") {
            return Object.values(response).filter(
                (v) => v && typeof v === "object"
            );
        }
        return [];
    };

    const updateUnreadCount = (newsList: any[], unreadNewsCount: any) => {
        const readNews = JSON.parse(localStorage.getItem("readNews") || "[]");
        const unread = newsList.filter((item: any) => {
            const uniqueId = `news_${item.language}_${item.id}`;
            return !readNews.includes(uniqueId);
        });
        unreadNewsCount.value = unread.length;
    };

    const setupEventListeners = async () => {
        await listen("theme-mode-update", (event: any) => {
            if (event.payload) currentTheme.value = event.payload;
        });

        await listen("client-crashed", (event: any) => {
            addToast(
                t("toast.client.crashed", {
                    name: event.payload.name,
                    error: event.payload.error || "",
                }),
                "error"
            );
        });

        await listen("client-crash-details", (event: any) => {
            showModal(
                `client-crash-${event.payload.id}`,
                ClientCrashModal,
                {
                    title: t("modal.client_crash.title", {
                        name: event.payload.name,
                    }),
                    contentClass: "wide",
                },
                {
                    clientName: event.payload.name,
                    clientId: event.payload.id,
                    logs: event.payload.logs,
                    error: event.payload.error,
                }
            );
        });
    };

    const checkConnectivity = async () => {
        try {
            const connectivity = await invoke<{
                cdn_online?: boolean;
                api_online?: boolean;
                auth_online?: boolean;
            }>("get_server_connectivity_status");

            const cdnOnline = connectivity.cdn_online ?? false;
            const apiOnline =
                connectivity.api_online ?? connectivity.auth_online ?? false;
            isOnline.value = Boolean(cdnOnline && apiOnline);

            if (cdnOnline) bootLogService.cdnOnline();
            else bootLogService.cdnOffline();
            if (apiOnline) bootLogService.webApiOnline();
            else bootLogService.webApiOffline();

            if (!isOnline.value) addToast(t("toast.server.offline"), "error");
        } catch (error) {
            console.error("Connectivity check failed:", error);
            isOnline.value = false;
        }
    };

    const initializeCoreSystems = async () => {
        bootLogService.start();
        bootLogService.systemInit();

        loadingState.value = loadingStates[0];
        currentProgress.value = 5;

        await setupEventListeners();

        const rpcTask = invoke("initialize_rpc").catch((e) =>
            bootLogService.addCustomLog("WARN", "rpc", String(e))
        );

        const themeTask = applyThemeOnStartup().then((theme) => {
            currentTheme.value = (theme as string) || "dark";
            bootLogService.themeApplied(currentTheme.value);
        });

        const languageTask = applyLanguageOnStartup().then(() => {
            const lang = locale.value || getCurrentLanguage() || "en";
            bootLogService.languageApplied(lang);
        });

        await Promise.all([rpcTask, themeTask, languageTask]);

        useToast().getToastPosition();
        bootLogService.eventListenersInit();

        currentProgress.value = 25;
        await wait(1000);
    };

    const initializeConnectivity = async () => {
        loadingState.value = loadingStates[1];
        await checkConnectivity();

        try {
            await invoke("initialize_api");
            apiInitialized.value = true;
            bootLogService.apiInitSuccess();
            invoke("update_tray_menu").catch(() => {});
        } catch (error) {
            console.error("API initialization failed:", error);
            bootLogService.apiInitFailed();
        }

        currentProgress.value = 50;
        await wait(500);
    };

    const initializeApiAndAuth = async (
        isAuthenticated: any,
        checkApiStatus: () => void
    ) => {
        loadingState.value = loadingStates[2];
        bootLogService.authCheck();
        checkApiStatus();

        if (!isAuthenticated.value) {
            bootLogService.authSkipped();
            return;
        }

        bootLogService.authSuccess();
        if (isOnline.value) {
            await initializeUserDataWrapper(isAuthenticated.value);
            bootLogService.syncReady();
            webSocketService.connect();
        }
    };

    const finalizeInitialization = async (news: any, unreadNewsCount: any) => {
        const flagsTask = invoke<Flags>("get_flags")
            .then((currentFlags) => {
                if (currentFlags.first_run.value) {
                    showFirstRunInfo.value = true;
                } else if (!currentFlags.disclaimer_shown.value) {
                    showInitialDisclaimer.value = true;
                }
                initialModalsLoaded.value = true;
            })
            .catch(() => {
                initialModalsLoaded.value = true;
            });

        fetchNewsAndUpdateUnreadCount(news, unreadNewsCount);
        await flagsTask;

        updaterService.startPeriodicCheck(t);
        currentProgress.value = 100;
        await wait(1000);

        showPreloader.value = false;
        await wait(500);

        document.documentElement.classList.add("app-ready");
        setTimeout(() => {
            contentVisible.value = true;
            bootLogService.clear();
        }, 80);
    };

    const initApp = async (
        isAuthenticated: any,
        checkApiStatus: () => void,
        news: any,
        unreadNewsCount: any
    ) => {
        await initializeCoreSystems();
        await initializeConnectivity();
        await initializeApiAndAuth(isAuthenticated, checkApiStatus);
        await finalizeInitialization(news, unreadNewsCount);
    };

    return {
        showPreloader,
        contentVisible,
        loadingState,
        currentProgress,
        isOnline,
        initialModalsLoaded,
        showFirstRunInfo,
        showInitialDisclaimer,
        currentTheme,
        apiInitialized,
        initApp,
        initializeUserDataWrapper,
    };
}

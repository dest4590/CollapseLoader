import { ref } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { bootLogService } from "../services/bootLogService";
import { applyLanguageOnStartup, applyThemeOnStartup } from "../utils/settings";
import { applyCursorForEvent, isHalloweenEvent } from "../utils/events";
import { useToast } from "../services/toastService";
import { globalUserStatus } from "./useUserStatus";
import { useUser } from "./useUser";
import { userService } from "../services/userService";
import { globalFriends } from "./useFriends";
import { updaterService } from "../services/updaterService";
import { syncService } from "../services/syncService";
import { getCurrentLanguage } from "../i18n";
import { useModal } from "../services/modalService";
import ClientCrashModal from "../components/modals/clients/ClientCrashModal.vue";
import { apiGet } from "../services/apiClient";
import { webSocketService } from "../services/webSocketService";

interface Flags {
    disclaimer_shown: { value: boolean };
    first_run: { value: boolean };
    optional_analytics: { value: boolean };
}

const wait = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export function useAppInit() {
    const { t, locale } = useI18n();
    const { addToast } = useToast();
    const { showModal } = useModal();
    const { loadUserData, hydrateUser } = useUser();
    const { loadFriendsData, hydrateFriends } = globalFriends;
    const { initializeStatusSystem } = globalUserStatus;

    const showPreloader = ref(true);
    const contentVisible = ref(false);

    const loadingStates = [
        t("preloader.initializing"),
        t("preloader.connecting_servers"),
        t("preloader.loading_user_data"),
    ];

    const loadingState = ref(loadingStates[0]);

    const currentProgress = ref(0);
    const isOnline = ref(true);
    const initialModalsLoaded = ref(false);
    const showFirstRunInfo = ref(false);
    const showInitialDisclaimer = ref(false);
    const halloweenActive = ref(isHalloweenEvent());

    const initialTheme =
        (document.documentElement.getAttribute("data-theme") as string) ||
        localStorage.getItem("theme") ||
        "dark";
    const currentTheme = ref(initialTheme);
    const apiInitialized = ref(false);

    const initializeUserDataWrapper = async (isAuthenticated: boolean) => {
        if (!isAuthenticated || !isOnline.value) return;
        try {
            const initData = await userService.initializeUserFull();
            const userInfo = {
                id: initData.user.id,
                username: initData.user.username,
                email: initData.user.email,
                role: initData.user.role,
                created_at: initData.user.created_at,
                updated_at: initData.user.updated_at,
                last_login_at: initData.user.last_login_at ?? null,
            };
            hydrateUser(initData.user.profile, userInfo);
            hydrateFriends(initData.friends);
            initializeStatusSystem();
            await syncService.restoreFromInitData(initData);
        } catch (error) {
            console.error("Init fallback:", error);
            try {
                await loadUserData();
                initializeStatusSystem();
                await loadFriendsData();
                await syncService.checkAndRestoreOnStartup();
            } catch (fallbackError) {
                console.error(fallbackError);
            }
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

            let allNews: any[] = [];
            if (Array.isArray(response)) {
                allNews = response;
            } else if (response && Array.isArray((response as any).data)) {
                allNews = (response as any).data;
            } else if (response && typeof response === "object") {
                allNews = Object.values(response).filter(
                    (v) => typeof v === "object"
                );
            }

            news.value = allNews.filter(
                (a: any) => a.language === currentLanguage
            );
            const getNewsUniqueId = (a: any) => `news_${a.language}_${a.id}`;
            const readNews = JSON.parse(
                localStorage.getItem("readNews") || "[]"
            );
            unreadNewsCount.value = news.value.filter(
                (a: any) => !readNews.includes(getNewsUniqueId(a))
            ).length;
        } catch (error) {
            console.error("Failed to fetch news:", error);
            unreadNewsCount.value = 0;
        }
    };

    const initApp = async (
        isAuthenticated: any,
        checkApiStatus: () => void,
        news: any,
        unreadNewsCount: any
    ) => {
        bootLogService.start();
        bootLogService.systemInit();

        loadingState.value = loadingStates[0];
        currentProgress.value = 5;

        // Listen for theme changes from other windows or settings
        await listen("theme-mode-update", (event: any) => {
            if (event.payload) {
                currentTheme.value = event.payload;
            }
        });

        const rpcTask = invoke("initialize_rpc").catch((e) =>
            bootLogService.addCustomLog("WARN", "rpc", String(e))
        );

        const themeTask = applyThemeOnStartup().then((theme) => {
            currentTheme.value = (theme as string) || "dark";
            bootLogService.themeApplied(currentTheme.value);
        });

        const languageTask = applyLanguageOnStartup().then(() => {
            bootLogService.languageApplied(
                locale.value || getCurrentLanguage() || "en"
            );
        });

        const cursorTask = applyCursorForEvent().then(() =>
            bootLogService.cursorApplied()
        );

        await Promise.all([rpcTask, themeTask, languageTask, cursorTask]);

        const { getToastPosition } = useToast();
        getToastPosition();

        bootLogService.eventListenersInit();

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

        currentProgress.value = 25;

        await wait(1000);

        loadingState.value = loadingStates[1];

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

        loadingState.value = loadingStates[2];

        bootLogService.authCheck();
        checkApiStatus();

        if (isAuthenticated.value) {
            bootLogService.authSuccess();
            if (isOnline.value) {
                await initializeUserDataWrapper(isAuthenticated.value);
                bootLogService.syncReady();
                webSocketService.connect();
            }
        } else {
            bootLogService.authSkipped();
        }

        const flagsTask = invoke<Flags>("get_flags")
            .then((currentFlags) => {
                if (currentFlags.first_run.value) showFirstRunInfo.value = true;
                else if (!currentFlags.disclaimer_shown.value)
                    showInitialDisclaimer.value = true;
                initialModalsLoaded.value = true;
            })
            .catch(() => {
                initialModalsLoaded.value = true;
            });

        fetchNewsAndUpdateUnreadCount(news, unreadNewsCount);
        await flagsTask;

        updaterService.startPeriodicCheck(t);

        loadingState.value = loadingStates[3] ?? loadingStates[2] ?? "";
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

    return {
        showPreloader,
        contentVisible,
        loadingState,
        currentProgress,
        isOnline,
        initialModalsLoaded,
        showFirstRunInfo,
        showInitialDisclaimer,
        halloweenActive,
        currentTheme,
        apiInitialized,
        initApp,
        initializeUserDataWrapper,
    };
}

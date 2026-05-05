import { ref, type Ref } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
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
import { webSocketService } from "@services/network/webSocketService";
import { wait } from "@shared/utils/utils";
import {
    telegramNewsService,
    type NewsArticle,
} from "@services/telegramNewsService";

interface Flags {
    disclaimer_shown: { value: boolean };
    first_run: { value: boolean };
    optional_analytics: { value: boolean };
}

interface ThemeModeUpdateEvent {
    payload?: string;
}

interface ClientCrashEvent {
    payload: {
        name: string;
        error?: string;
    };
}

interface ClientCrashDetailsEvent {
    payload: {
        id: number;
        name: string;
        logs: string[];
        error?: string;
    };
}

export function useAppInit() {
    const { t } = useI18n();
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

    class AppInitializationCoordinator {
        async initializeUserData(isAuthenticated: boolean) {
            if (!isAuthenticated || !isOnline.value) {
                return;
            }

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
                await this.performFallbackInit();
            }
        }

        async performFallbackInit() {
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
        }

        async fetchNewsAndUpdateUnreadCount(
            news: Ref<NewsArticle[]>,
            unreadNewsCount: Ref<number>
        ) {
            try {
                const currentLanguage = getCurrentLanguage() || "en";
                news.value =
                    await telegramNewsService.fetchNews(currentLanguage);
                unreadNewsCount.value = telegramNewsService.getUnreadCount(
                    news.value
                );
            } catch (error) {
                console.error("Failed to fetch news:", error);
                unreadNewsCount.value = 0;
            }
        }

        async setupEventListeners() {
            await listen("theme-mode-update", (event: ThemeModeUpdateEvent) => {
                if (event.payload) {
                    currentTheme.value = event.payload;
                }
            });

            await listen("client-crashed", (event: ClientCrashEvent) => {
                addToast(
                    t("toast.client.crashed", {
                        name: event.payload.name,
                        error: event.payload.error || "",
                    }),
                    "error"
                );
            });

            await listen(
                "client-crash-details",
                (event: ClientCrashDetailsEvent) => {
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
                }
            );
        }

        async checkConnectivity() {
            try {
                const connectivity = await invoke<{
                    cdn_online?: boolean;
                    api_online?: boolean;
                    auth_online?: boolean;
                }>("get_server_connectivity_status");

                const cdnOnline = connectivity.cdn_online ?? false;
                const apiOnline =
                    connectivity.api_online ??
                    connectivity.auth_online ??
                    false;
                isOnline.value = Boolean(cdnOnline && apiOnline);

                if (!isOnline.value) {
                    addToast(t("toast.server.offline"), "error");
                }
            } catch (error) {
                console.error("Connectivity check failed:", error);
                isOnline.value = false;
            }
        }

        async initializeCoreSystems() {
            loadingState.value = loadingStates[0];
            currentProgress.value = 5;

            await this.setupEventListeners();

            const rpcTask = invoke("initialize_rpc").catch((error) =>
                console.error("RPC initialization failed:", error)
            );

            const themeTask = applyThemeOnStartup().then((theme) => {
                currentTheme.value = (theme as string) || "dark";
            });

            const languageTask = applyLanguageOnStartup();

            await Promise.all([rpcTask, themeTask, languageTask]);

            useToast().getToastPosition();

            currentProgress.value = 25;
            await wait(1000);
        }

        async initializeConnectivity() {
            loadingState.value = loadingStates[1];
            await this.checkConnectivity();

            try {
                await invoke("initialize_api");
                apiInitialized.value = true;
                invoke("update_tray_menu").catch(() => {});
            } catch (error) {
                console.error("API initialization failed:", error);
            }

            currentProgress.value = 50;
            await wait(500);
        }

        async initializeApiAndAuth(
            isAuthenticated: Ref<boolean>,
            checkApiStatus: () => void
        ) {
            loadingState.value = loadingStates[2];
            checkApiStatus();

            if (!isAuthenticated.value) {
                return;
            }

            if (isOnline.value) {
                await this.initializeUserData(isAuthenticated.value);
                webSocketService.connect();
            }
        }

        async finalizeInitialization(
            news: Ref<NewsArticle[]>,
            unreadNewsCount: Ref<number>
        ) {
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

            void this.fetchNewsAndUpdateUnreadCount(news, unreadNewsCount);
            await flagsTask;

            updaterService.startPeriodicCheck(t);
            currentProgress.value = 100;
            await wait(1000);

            showPreloader.value = false;
            await wait(500);

            document.documentElement.classList.add("app-ready");
            setTimeout(() => {
                contentVisible.value = true;
            }, 80);
        }

        async run(
            isAuthenticated: Ref<boolean>,
            checkApiStatus: () => void,
            news: Ref<NewsArticle[]>,
            unreadNewsCount: Ref<number>
        ) {
            await this.initializeCoreSystems();
            await this.initializeConnectivity();
            await this.initializeApiAndAuth(isAuthenticated, checkApiStatus);
            await this.finalizeInitialization(news, unreadNewsCount);
        }
    }

    const initializationCoordinator = new AppInitializationCoordinator();

    const initializeUserDataWrapper = async (isAuthenticated: boolean) => {
        await initializationCoordinator.initializeUserData(isAuthenticated);
    };

    const initApp = async (
        isAuthenticated: Ref<boolean>,
        checkApiStatus: () => void,
        news: Ref<NewsArticle[]>,
        unreadNewsCount: Ref<number>
    ) => {
        await initializationCoordinator.run(
            isAuthenticated,
            checkApiStatus,
            news,
            unreadNewsCount
        );
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

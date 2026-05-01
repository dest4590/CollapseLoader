<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { router } from "./services/router";
import { useI18n } from "vue-i18n";
import GlobalModal from "@shared/components/common/GlobalModal.vue";
import DevMenuModal from "./components/core/DevMenuModal.vue";
import InitialSetupModals from "./components/core/InitialSetupModals.vue";
import DownloadProgress from "@features/download/components/DownloadProgress.vue";
import Sidebar from "./components/layout/Sidebar.vue";
import Titlebar from "./components/layout/Titlebar.vue";
import RegisterPromptModal from "@features/social/modals/RegisterPromptModal.vue";
import ToastContainer from "@shared/components/notifications/ToastContainer.vue";
import { useUser } from "@core/auth/useUser";
import { globalUserStatus } from "@core/auth/useUserStatus";
import { syncService } from "./services/syncService";
import { settingsService } from "@core/settings/settingsService";
import { useToast } from "@shared/composables/useToast";
import { themeService } from "@core/theme/themeService";
import { updaterService } from "@core/updater/updaterService";
import { webSocketService } from "@core/network/webSocketService";
import About from "./views/About.vue";
import AccountView from "./views/AccountView.vue";
import AppLogs from "./views/AppLogs.vue";
import FriendsView from "./views/FriendsView.vue";
import Home from "./views/Home.vue";
import LoginView from "./views/LoginView.vue";
import RegisterView from "./views/RegisterView.vue";
import VerifyEmailView from "./views/VerifyEmailView.vue";
import Settings from "./views/Settings.vue";
import Customization from "./views/Customization.vue";
import UserProfileView from "./views/UserProfileView.vue";
import News from "./views/News.vue";
import CustomClients from "./views/CustomClients.vue";
import Marketplace from "./views/Marketplace.vue";
import AuthModal from "./components/layout/modals/AuthModal.vue";
import NetworkDebug from "./views/NetworkDebug.vue";
import { fetchSettings } from "./utils/settings";
import { getDiscordState } from "@features/social/utils/discord";
import { VALID_TABS } from "./utils/tabs";
import { getIsDevelopment } from "@shared/utils/isDevelopment";
import Preloader from "./components/core/Preloader.vue";
import SpotlightSearch from "./components/core/SpotlightSearch.vue";

import { useAppInit } from "./composables/useAppInit";
import { initNetworkDebug } from "./services/networkDebugService";
import type { Client } from "@shared/types/ui";
import notificationSound from "./assets/misc/notification.mp3";
import { userService } from "@core/auth/userService";
import { localTrackerService } from "./services/localTrackerService";
import { persistenceService } from "./services/persistenceService";

const isMacOS = ref(false);

interface Setting<T> {
    description: string;
    value: T;
}

interface Flags {
    disclaimer_shown: Setting<boolean>;
    first_run: Setting<boolean>;
    optional_analytics: Setting<boolean>;
}

const { t, locale } = useI18n();

const {
    showPreloader,
    loadingState,
    currentProgress,
    isOnline,
    showFirstRunInfo,
    showInitialDisclaimer,
    currentTheme,
    apiInitialized,
    contentVisible,
    initApp,
    initializeUserDataWrapper,
} = useAppInit();

const appOnline = computed(() => isOnline?.value ?? true);

const activeTab = computed(() => router.currentRoute.value as any);
const showDevMenu = ref(false);
const showSpotlight = ref(false);

const { addToast } = useToast();
const isAuthenticated = ref(false);
const showRegistrationPrompt = ref(false);
const showAuthModal = ref(false);
const authModalView = ref<"LOGIN" | "REGISTER" | "VERIFY">("LOGIN");
const pendingVerifyEmail = ref("");
const currentUserId = ref<number | null>(null);
const previousTab = ref<string>("home");
const news = ref<any[]>([]);
const unreadNewsCount = ref(0);
const isDev = ref(false);

const sidebarPosition = ref(
    (localStorage.getItem("sidebarPosition") as
        | "left"
        | "right"
        | "top"
        | "bottom") || "left"
);

const updateSidebarPosition = (
    newPosition: "left" | "right" | "top" | "bottom"
) => {
    sidebarPosition.value = newPosition;
    localStorage.setItem("sidebarPosition", newPosition);
};

const mainClasses = computed(() => {
    const base =
        "relative w-full p-6 pb-8 bg-base-200 overflow-y-auto overflow-x-hidden flex-1";
    const pos = sidebarPosition.value;

    if (pos === "left") return `${base} ml-20`;
    if (pos === "right") return `${base} mr-20`;
    if (pos === "top") return `${base} mt-20`;
    if (pos === "bottom") return `${base} mb-20`;

    return base;
});

const { stopStatusSync } = globalUserStatus;

const handleUnreadNewsCountUpdated = (count: number) => {
    unreadNewsCount.value = count;
};

const setActiveTab = (tab: string, opts?: { userId?: number | null }) => {
    if (!VALID_TABS.includes(tab)) return;
    previousTab.value = router.currentRoute.value;
    if (opts && Object.prototype.hasOwnProperty.call(opts, "userId")) {
        currentUserId.value = opts!.userId ?? null;
    } else {
        currentUserId.value = null;
    }
    router.push(tab);
};

const showUserProfile = (userId: number) => {
    previousTab.value = router.currentRoute.value;

    setActiveTab("user-profile", { userId });
};

const checkAuthStatus = () => {
    const token = localStorage.getItem("authToken");
    isAuthenticated.value = !!token;
};

const handleFirstRunAccepted = async () => {
    try {
        await invoke("mark_first_run_shown");
        showFirstRunInfo.value = false;
        const flags = await invoke<Flags>("get_flags");
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
        console.error("Failed to mark first run as shown:", error);
        addToast(t("toast.settings.first_run_save_failed", { error }), "error");
    }
};

const handleDisclaimerAccepted = async () => {
    try {
        await invoke("mark_disclaimer_shown");
        showInitialDisclaimer.value = false;
        if (showPreloader.value) showPreloader.value = false;

        if (!isAuthenticated.value) {
            setTimeout(() => {
                showRegistrationPrompt.value = true;
            }, 500);
        }
    } catch (error) {
        console.error("Failed to mark disclaimer as shown:", error);
        addToast(
            t("toast.settings.disclaimer_save_failed", { error }),
            "error"
        );
    }
};

const handleOpenDevMenu = () => {
    showDevMenu.value = true;
    addToast(t("toast.dev.menu_opened"), "info");
};

const closeDevMenu = () => {
    showDevMenu.value = false;
};

const handleLoggedOut = () => {
    isAuthenticated.value = false;
    localStorage.removeItem("authToken");
    setActiveTab("login");
    syncService.destroy();

    clearUserData();

    globalUserStatus.stopStatusSync();
    webSocketService.disconnect();
};

const handleLoggedIn = async () => {
    isAuthenticated.value = true;
    setActiveTab("home");

    await initializeUserDataWrapper(isAuthenticated.value);

    await syncService.initializeSyncStatus();
    await syncService.checkAndRestoreOnStartup();

    globalUserStatus.initializeStatusSystem();
    webSocketService.connect();
};

const handleRegistered = () => {
    setActiveTab("login");
    addToast(t("toast.auth.registration_success"), "success");
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
    verify: VerifyEmailView,
    friends: FriendsView,
    "user-profile": UserProfileView,
    marketplace: Marketplace,
    network_debug: NetworkDebug,
};

const currentView = computed(() => views[activeTab.value] || Home);

const updateDiscordRPC = async (tab?: string) => {
    try {
        const settings = await fetchSettings();
        if (!settings?.discord_rpc_enabled?.value) {
            console.log("Discord RPC is disabled in settings, skipping update");
            return;
        }

        const currentTab = tab || activeTab.value;
        const details = t("discord.details.in_menu");
        const state = getDiscordState(currentTab, (k: string) => t(k));

        await invoke("update_presence", { details, state });
        console.log(
            `Discord RPC updated for tab: ${currentTab} - ${details}: ${state}`
        );
    } catch (error) {
        console.error("Failed to update Discord RPC:", error);
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
    localStorage.setItem("registrationPromptShown", new Date().toISOString());
};

const handleRegisterPrompt = () => {
    authModalView.value = "REGISTER";
    showAuthModal.value = true;
    showRegistrationPrompt.value = false;
    localStorage.setItem("registrationPromptShown", new Date().toISOString());
};

const pendingVerifyCode = ref("");

const handleShowVerify = (email: string, code?: string) => {
    pendingVerifyEmail.value = email;
    if (code) pendingVerifyCode.value = code;

    setActiveTab("verify");
};

const handleVerified = (token?: string) => {
    if (token) {
        localStorage.setItem("authToken", token);
        userService.clearCache();
        addToast(
            t("auth.verify.success") || "Email verified! Logging you in...",
            "success"
        );
        handleLoggedIn();
    } else {
        addToast(
            t("auth.verify.success") || "Email verified! Please log in.",
            "success"
        );
        setActiveTab("login");
    }
};

onMounted(async () => {
    await persistenceService.init();
    try {
        await initNetworkDebug();
    } catch (e) {
        console.warn("Failed to init global network debug service", e);
    }
    await listen("verify-email", (event: any) => {
        const { code, email } = event.payload;
        console.log("Received verification deep link:", code, email);
        if (code) {
            pendingVerifyCode.value = code;
            if (email) pendingVerifyEmail.value = email;

            if (activeTab.value !== "verify") {
                setActiveTab("verify");
            }
        }
    });
});

const { clearUserData } = useUser();

const getTransitionName = () => {
    const tabOrder = [
        "home",
        "custom_clients",
        "friends",
        "settings",
        "customization",
        "app_logs",
        "account",
        "login",
        "register",
        "about",
    ];
    const currentIndex = tabOrder.indexOf(activeTab.value);
    const previousIndex = tabOrder.indexOf(previousTab.value);

    return currentIndex > previousIndex
        ? "slide-down"
        : currentIndex < previousIndex
          ? "slide-up"
          : "fade-slide";
};

onMounted(async () => {
    initApp(isAuthenticated, checkAuthStatus, news, unreadNewsCount);
    settingsService.loadSettings();
    checkAuthStatus();

    try {
        isMacOS.value = await invoke("is_macos");
        isDev.value = await getIsDevelopment();
    } catch (e) {
        console.error("Failed to determine platform or environment:", e);
        isMacOS.value = false;
        isDev.value = false;
    }

    listen("tray-launch-client", async (event) => {
        const { id } = event.payload as { id: number };
        if (!id) return;
        const clients = await invoke<Client[]>("get_clients");
        const target = clients.find((c) => c.id === id);
        if (!target) return;
        try {
            const userToken = localStorage.getItem("authToken") || "null";
            if (!target.meta.installed) {
                addToast(
                    t("home.starting_download", { name: target.name }),
                    "info",
                    3000
                );
                await invoke("download_client_only", { id: target.id });
                target.meta.installed = true;
            }
            addToast(
                t("home.launching", { client: target.name }),
                "info",
                2000
            );
            await invoke("launch_client", { id: target.id, userToken });
            await getCurrentWindow().minimize();
        } catch (e) {
            console.error("Cannot start client from tray", e);
            addToast(String(e), "error", 5000);
        }
    });

    listen("launch-client", async (event) => {
        const { id, was_already_running } = event.payload as {
            id: string;
            was_already_running: boolean;
        };
        const clientId = Number(id);

        if (clientId) {
            const clients = await invoke<Client[]>("get_clients");
            const target = clients.find((c) => c.id === clientId);

            if (target) {
                try {
                    const userToken =
                        localStorage.getItem("authToken") || "null";

                    if (!target.meta.installed) {
                        addToast(
                            t("home.starting_download", { name: target.name }),
                            "info",
                            3000
                        );
                        await invoke("download_client_only", {
                            id: target.id,
                        });
                        target.meta.installed = true;
                    }

                    addToast(
                        t("home.launching", { client: target.name }),
                        "info",
                        2000
                    );

                    await invoke("launch_client", {
                        id: target.id,
                        userToken,
                    });

                    if (!was_already_running) {
                        await getCurrentWindow().minimize();
                    }
                } catch (e) {
                    console.error("Cannot start client from deeplink", e);
                    addToast(String(e), "error", 5000);
                }
            }
        }
    });

    listen("client-launched", async (event) => {
        const payload = event.payload as {
            id: number;
            name: string;
            version?: string;
        };
        console.log(`Client ${payload.name} launched, updating status...`);
        localTrackerService.trackLaunch(payload.name);
        localTrackerService.startPlaytimeTracking(payload.name);

        try {
            globalUserStatus.setPlayingClient(
                `${payload.name} (${payload.version || "unknown version"})`
            );
            await settingsService.loadSettings();
            const settings = settingsService.getSettings() as any;
            if (settings.discord_rpc_enabled?.value) {
                await invoke("update_presence", {
                    details: t("discord.details.in_game"),
                    state: payload.name,
                }).catch((error) => {
                    console.error("Failed to update Discord presence:", error);
                });
            }
        } catch (error) {
            console.error("Failed to update playing status:", error);
        }
    });

    listen("client-exited", async (event) => {
        const payload = event.payload as {
            id: number;
            name: string;
            exitCode?: number;
        };
        console.log(`Client ${payload.name} exited, updating status...`);
        localTrackerService.stopPlaytimeTracking();

        try {
            globalUserStatus.setOnline();
            await settingsService.loadSettings();
            const settings = settingsService.getSettings() as any;
            if (settings.discord_rpc_enabled?.value) {
                await invoke("update_presence", {
                    details: t("discord.details.in_menu"),
                    state: t("discord.states.browsing_clients"),
                }).catch((error) => {
                    console.error("Failed to reset Discord presence:", error);
                });
            }
        } catch (error) {
            console.error("Failed to update online status:", error);
        }
    });

    listen("update-user-status", async (event) => {
        const payload = event.payload as {
            status: string;
            currentClient: string | null;
        };

        console.log("Received status update event from backend:", payload);
        console.log("Backend status event ignored to prevent conflicts");
    });

    listen("toast-error", (event) => {
        console.log(event);

        let message: string;
        message = String(event.payload);

        addToast(message, "error");
    });

    window.addEventListener("beforeunload", () => {
        if (globalUserStatus.isAuthenticated.value) {
            globalUserStatus.setOffline();
        }
    });

    const emergencyHandler = (e: KeyboardEvent) => {
        try {
            if (
                e.ctrlKey &&
                e.shiftKey &&
                (e.key === "Home" || e.code === "Home")
            ) {
                const active = document.activeElement as HTMLElement | null;
                const isTyping =
                    !!active &&
                    (active.tagName === "INPUT" ||
                        active.tagName === "TEXTAREA" ||
                        active.isContentEditable);
                if (isTyping) return;

                console.warn(
                    "Emergency theme reset triggered via Ctrl+Shift+Home"
                );
                themeService.emergencyReset();

                addToast(
                    t("toast.theme.emergency_reset_done", {
                        action: t(
                            "toast.theme.emergency_reset_toggle_instruction"
                        ),
                    }),
                    "info",
                    8000
                );
            }
        } catch (err) {
            console.error("Error during emergency theme reset:", err);
        }
    };

    window.addEventListener("keydown", emergencyHandler);

    const spotlightHandler = (e: KeyboardEvent) => {
        try {
            if (e.ctrlKey && e.code === "Space") {
                e.preventDefault();
                e.stopPropagation();
                showSpotlight.value = !showSpotlight.value;
            }
        } catch (err) {
            console.error("Error handling spotlight shortcut:", err);
        }
    };

    window.addEventListener("keydown", spotlightHandler);

    const networkDebugHandler = (e: KeyboardEvent) => {
        try {
            const active = document.activeElement as HTMLElement | null;
            const isTyping =
                !!active &&
                (active.tagName === "INPUT" ||
                    active.tagName === "TEXTAREA" ||
                    active.isContentEditable);
            if (isTyping) return;

            if (e.key === "F9" || e.code === "F9") {
                setActiveTab("network_debug");
            }
        } catch (err) {
            console.error("Error handling F9 keybind:", err);
        }
    };

    window.addEventListener("keydown", networkDebugHandler);

    window.addEventListener("achievement-unlocked", (event: any) => {
        const { key } = event.detail;
        const name = t(`achievements.list.${key}.name`);
        const description = t(`achievements.list.${key}.description`);
        addToast(
            `${t("achievements.unlocked_title", { name })}\n${description}`,
            "success",
            9000
        );
    });

    window.addEventListener("friend-request-received", (event: any) => {
        const { sender, nickname } = event.detail;
        const displayName = nickname || sender;
        const audio = new Audio(notificationSound);
        audio
            .play()
            .catch((e) =>
                console.error("Failed to play notification sound:", e)
            );
        addToast(
            t("notifications.friend_request_received.message", {
                name: displayName,
            }),
            "info",
            15000
        );
    });

    window.addEventListener("friend-request-accepted", (event: any) => {
        const { sender, nickname } = event.detail;
        const displayName = nickname || sender;
        const audio = new Audio(notificationSound);
        audio
            .play()
            .catch((e) =>
                console.error("Failed to play notification sound:", e)
            );
        addToast(
            t("notifications.friend_request_accepted.message", {
                name: displayName,
            }),
            "success",
            15000
        );
    });

    const broadcastHandler = (event: any) => {
        const { message, type, sticky } = event.detail;
        const audio = new Audio(notificationSound);
        audio
            .play()
            .catch((e) =>
                console.error("Failed to play notification sound:", e)
            );

        let toastType = "info";
        if (type === "warning") toastType = "warning";
        if (type === "error") toastType = "error";

        const duration = sticky ? 60000 : 10000;
        addToast(message, toastType as any, duration);
    };

    window.addEventListener("system-broadcast", broadcastHandler);

    const updateHandler = async () => {
        await updaterService.checkForUpdates(false, t);
    };

    window.addEventListener("trigger-update-check", updateHandler);

    onUnmounted(() => {
        window.removeEventListener("keydown", emergencyHandler);
        window.removeEventListener("keydown", networkDebugHandler);
        window.removeEventListener("system-broadcast", broadcastHandler);
        window.removeEventListener("trigger-update-check", updateHandler);
    });
});

onUnmounted(() => {
    console.log("App unmounting, stopping systems...");
    stopStatusSync();
    updaterService.stopPeriodicCheck();
    window.removeEventListener("beforeunload", () => {});
    console.log("Status sync stopped");
});
</script>

<template>
    <div
        :style="{
            '--sidebar-bottom-height':
                sidebarPosition === 'bottom' ? '80px' : '0px',
            '--sidebar-top-height': sidebarPosition === 'top' ? '80px' : '0px',
            '--toast-bottom-offset':
                sidebarPosition === 'bottom' ? 'calc(1rem + 80px)' : '1rem',
            '--toast-top-offset':
                sidebarPosition === 'top' ? 'calc(4rem + 80px)' : '4rem',
            '--toast-left-offset':
                sidebarPosition === 'left' ? 'calc(1rem + 80px)' : '1rem',
            '--toast-right-offset':
                sidebarPosition === 'right' ? 'calc(1rem + 80px)' : '1rem',
        }"
    >
        <Preloader
            v-model:show="showPreloader"
            :is-dev="isDev"
            :loading-state="loadingState"
            :current-progress="currentProgress"
            :current-theme="currentTheme"
        />

        <InitialSetupModals
            :show-first-run="showFirstRunInfo"
            :show-disclaimer="showInitialDisclaimer"
            :current-theme="currentTheme"
            @first-run-accepted="handleFirstRunAccepted"
            @disclaimer-accepted="handleDisclaimerAccepted"
            @auto-login="handleLoggedIn"
        />

        <DevMenuModal
            :show-dev-menu="showDevMenu"
            :registerPrompt="showRegistrationPrompt"
            @close="closeDevMenu"
        />

        <div
            class="flex h-screen flex-col overflow-hidden"
            v-if="!showInitialDisclaimer && !showFirstRunInfo && contentVisible"
        >
            <Titlebar v-if="!isMacOS" />

            <div
                class="flex-1 flex overflow-hidden relative"
                :class="{ 'pt-10': !isMacOS }"
            >
                <Sidebar
                    :activeTab="activeTab"
                    @changeTab="setActiveTab"
                    @open-dev-menu="handleOpenDevMenu"
                    :is-online="appOnline"
                    :is-authenticated="isAuthenticated"
                    :position="sidebarPosition"
                    :isMacOS="isMacOS"
                    @update:position="updateSidebarPosition"
                />

                <main :class="[mainClasses, 'main-content']">
                    <transition
                        :name="getTransitionName()"
                        mode="out-in"
                        appear
                    >
                        <div :key="activeTab + (currentUserId || '')">
                            <component
                                :is="currentView"
                                @logged-out="handleLoggedOut"
                                @logged-in="handleLoggedIn"
                                @registered="handleRegistered"
                                @change-view="setActiveTab"
                                @show-user-profile="showUserProfile"
                                @back-to-friends="() => setActiveTab('friends')"
                                @unread-count-updated="
                                    handleUnreadNewsCountUpdated
                                "
                                :key="activeTab"
                                :is-online="appOnline"
                                :user-id="currentUserId"
                                v-on="
                                    activeTab === 'login' ||
                                    activeTab === 'register' ||
                                    activeTab === 'verify'
                                        ? {
                                              'show-verify': handleShowVerify,
                                              verified: handleVerified,
                                          }
                                        : {}
                                "
                                v-bind="
                                    activeTab === 'verify'
                                        ? {
                                              email: pendingVerifyEmail,
                                              code: pendingVerifyCode,
                                          }
                                        : activeTab === 'home'
                                          ? { unreadNewsCount, apiInitialized }
                                          : {}
                                "
                            />
                        </div>
                    </transition>
                </main>
            </div>
        </div>

        <DownloadProgress />
        <ToastContainer />
        <GlobalModal />
        <SpotlightSearch
            :show="showSpotlight"
            @close="showSpotlight = false"
            @navigate="
                (tab) => {
                    setActiveTab(tab);
                    showSpotlight = false;
                }
            "
        />
        <RegisterPromptModal
            v-model="showRegistrationPrompt"
            @register="handleRegisterPrompt"
            @cancel="hideRegistrationPrompt"
        />
        <AuthModal
            v-model="showAuthModal"
            :initial-view="authModalView"
            @logged-in="handleLoggedIn"
        />
    </div>
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

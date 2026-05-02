<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { router } from "@router";
import { useI18n } from "vue-i18n";

// Components
import GlobalModal from "@shared/components/common/GlobalModal.vue";
import DevMenuModal from "./components/core/DevMenuModal.vue";
import InitialSetupModals from "./components/core/InitialSetupModals.vue";
import DownloadProgress from "@features/download/components/DownloadProgress.vue";
import Sidebar from "@layouts/Sidebar.vue";
import Titlebar from "@layouts/Titlebar.vue";
import RegisterPromptModal from "@features/social/modals/RegisterPromptModal.vue";
import ToastContainer from "@shared/components/notifications/ToastContainer.vue";
import Preloader from "./components/core/Preloader.vue";
import SpotlightSearch from "./components/core/SpotlightSearch.vue";
import AuthModal from "@layouts/modals/AuthModal.vue";

// Views
import { views, tabOrder } from "@router/views";

// Services & Composables
import { globalUserStatus } from "@features/auth/useUserStatus";
import { settingsService } from "@services/settings/settingsService";
import { useToast } from "@shared/composables/useToast";
import { useAppInit } from "./composables/useAppInit";
import { useAppHandlers } from "./composables/useAppHandlers";
import { initNetworkDebug } from "./services/networkDebugService";
import { persistenceService } from "./services/persistenceService";
import { userService } from "@features/auth/userService";
import { getIsDevelopment } from "@shared/utils/isDevelopment";

// Types & Utils
import { updaterService } from "./services/updater/updaterService";

const { t } = useI18n();
const { addToast } = useToast();

// State
const isMacOS = ref(false);
const isDev = ref(false);
const isAuthenticated = ref(false);
const showRegistrationPrompt = ref(false);
const showAuthModal = ref(false);
const authModalView = ref<"LOGIN" | "REGISTER" | "VERIFY">("LOGIN");
const pendingVerifyEmail = ref("");
const pendingVerifyCode = ref("");
const currentUserId = ref<number | null>(null);
const previousTab = ref<string>("home");
const news = ref<any[]>([]);
const unreadNewsCount = ref(0);
const showDevMenu = ref(false);
const showSpotlight = ref(false);

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

const activeTab = computed(() => router.currentRoute.value as any);

const {
    sidebarPosition,
    updateSidebarPosition,
    mainClasses,
    setActiveTab,
    handleFirstRunAccepted,
    handleDisclaimerAccepted,
    handleLoggedOut,
    handleLoggedIn,
    setupTauriListeners,
    setupWindowListeners,
} = useAppHandlers({
    isAuthenticated,
    showPreloader,
    showFirstRunInfo,
    showInitialDisclaimer,
    showRegistrationPrompt,
    activeTab,
    currentUserId,
    previousTab,
    authModalView,
    showAuthModal,
});

const appOnline = computed(() => isOnline?.value ?? true);

const handleUnreadNewsCountUpdated = (count: number) => {
    unreadNewsCount.value = count;
};

const showUserProfile = (userId: number) => {
    setActiveTab("user-profile", { userId });
};

const checkAuthStatus = () => {
    isAuthenticated.value = !!localStorage.getItem("authToken");
};

const handleOpenDevMenu = () => {
    showDevMenu.value = true;
    addToast(t("toast.dev.menu_opened"), "info");
};

const closeDevMenu = () => {
    showDevMenu.value = false;
};

const handleRegistered = () => {
    setActiveTab("login");
    addToast(t("toast.auth.registration_success"), "success");
};

const currentView = computed(() => views[activeTab.value] || views.home);

watch(isAuthenticated, (newVal) => {
    if (newVal) showRegistrationPrompt.value = false;
});

const hideRegistrationPrompt = () => {
    showRegistrationPrompt.value = false;
    localStorage.setItem("registrationPromptShown", new Date().toISOString());
};

const handleRegisterPrompt = () => {
    authModalView.value = "REGISTER";
    showAuthModal.value = true;
    hideRegistrationPrompt();
};

const handleShowVerify = (email: string, code?: string) => {
    pendingVerifyEmail.value = email;
    if (code) {
        pendingVerifyCode.value = code;
    }
    setActiveTab("verify");
};

const handleVerified = (token?: string) => {
    const successMsg = t("auth.verify.success") || "Email verified!";

    if (token) {
        localStorage.setItem("authToken", token);
        userService.clearCache();
        addToast(`${successMsg} Logging you in...`, "success");
        handleLoggedIn(initializeUserDataWrapper);
        return;
    }

    addToast(`${successMsg} Please log in.`, "success");
    setActiveTab("login");
};

const setupDeepLinks = async () => {
    await listen("verify-email", (event: any) => {
        const { code, email } = event.payload;
        if (code) {
            pendingVerifyCode.value = code;
            if (email) pendingVerifyEmail.value = email;
            if (activeTab.value !== "verify") setActiveTab("verify");
        }
    });
};

const initSystemInfo = async () => {
    try {
        const [mac, dev] = await Promise.all([
            invoke<boolean>("is_macos"),
            getIsDevelopment(),
        ]);
        isMacOS.value = mac;
        isDev.value = dev;
    } catch (e) {
        console.error("Failed to fetch system info:", e);
    }
};

onMounted(async () => {
    await persistenceService.init();
    initNetworkDebug().catch(console.warn);
    await setupDeepLinks();

    const cleanupTauri = await setupTauriListeners();
    const cleanupWindow = setupWindowListeners();

    initApp(isAuthenticated, checkAuthStatus, news, unreadNewsCount);
    settingsService.loadSettings();
    checkAuthStatus();

    await initSystemInfo();

    const spotlightHandler = (e: KeyboardEvent) => {
        if (e.ctrlKey && e.code === "Space") {
            e.preventDefault();
            e.stopPropagation();
            showSpotlight.value = !showSpotlight.value;
        }
    };
    window.addEventListener("keydown", spotlightHandler);

    onUnmounted(() => {
        cleanupTauri();
        cleanupWindow();
        window.removeEventListener("keydown", spotlightHandler);
    });
});

const getTransitionName = () => {
    const currentIndex = tabOrder.indexOf(activeTab.value);
    const previousIndex = tabOrder.indexOf(previousTab.value);

    if (currentIndex === -1 || previousIndex === -1) return "fade-slide";
    if (currentIndex === previousIndex) return "fade-slide";

    return currentIndex > previousIndex ? "slide-down" : "slide-up";
};

onUnmounted(() => {
    console.log("App unmounting, stopping systems...");
    globalUserStatus.stopStatusSync();
    updaterService.stopPeriodicCheck();
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

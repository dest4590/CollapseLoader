import { ref, computed, watch } from "vue";
import { useI18n } from "vue-i18n";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { router } from "@router";
import { useToast } from "@shared/composables/useToast";
import { syncService } from "@services/syncService";
import { globalUserStatus } from "@features/auth/useUserStatus";
import { webSocketService } from "@services/network/webSocketService";
import { useUser } from "@features/auth/useUser";
import { fetchSettings } from "@/utils/settings";
import { getDiscordState } from "@features/social/utils/discord";
import { VALID_TABS } from "@/utils/tabs";
import { settingsService } from "@services/settings/settingsService";
import { localTrackerService } from "@services/localTrackerService";
import type { Client } from "@shared/types/ui";
import notificationSound from "@/assets/misc/notification.mp3";

export function useAppHandlers(props: {
    isAuthenticated: any;
    showPreloader: any;
    showFirstRunInfo: any;
    showInitialDisclaimer: any;
    showRegistrationPrompt: any;
    activeTab: any;
    currentUserId: any;
    previousTab: any;
    authModalView: any;
    showAuthModal: any;
    initializeUserDataWrapper?: (auth: boolean) => Promise<void>;
}) {
    const { t, locale } = useI18n();
    const { addToast } = useToast();
    const { clearUserData } = useUser();

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

    const setActiveTab = (tab: string, opts?: { userId?: number | null }) => {
        if (!VALID_TABS.includes(tab)) return;
        props.previousTab.value = props.activeTab.value;
        props.currentUserId.value = opts?.userId ?? null;
        router.push(tab);
    };

    const markFlagShown = async (
        flag: "first_run" | "disclaimer",
        refToUpdate: any
    ) => {
        try {
            const command =
                flag === "first_run"
                    ? "mark_first_run_shown"
                    : "mark_disclaimer_shown";
            await invoke(command);
            refToUpdate.value = false;

            if (flag === "first_run") {
                const flags = await invoke<any>("get_flags");
                if (!flags.disclaimer_shown.value) {
                    props.showInitialDisclaimer.value = true;
                    return;
                }
            }

            if (props.showPreloader.value) {
                props.showPreloader.value = false;
            }

            if (!props.isAuthenticated.value) {
                setTimeout(() => {
                    props.showRegistrationPrompt.value = true;
                }, 500);
            }
        } catch (error) {
            console.error(`Failed to mark ${flag} as shown:`, error);
            addToast(
                t(`toast.settings.${flag}_save_failed`, { error }),
                "error"
            );
        }
    };

    const handleFirstRunAccepted = () =>
        markFlagShown("first_run", props.showFirstRunInfo);
    const handleDisclaimerAccepted = () =>
        markFlagShown("disclaimer", props.showInitialDisclaimer);

    const handleLoggedOut = () => {
        props.isAuthenticated.value = false;
        localStorage.removeItem("authToken");
        setActiveTab("login");
        syncService.destroy();
        clearUserData();
        globalUserStatus.stopStatusSync();
        webSocketService.disconnect();
    };

    const handleLoggedIn = async () => {
        props.isAuthenticated.value = true;
        setActiveTab("home");
        if (props.initializeUserDataWrapper) {
            await props.initializeUserDataWrapper(true);
        }
        await syncService.initializeSyncStatus();
        await syncService.checkAndRestoreOnStartup();
        globalUserStatus.initializeStatusSystem();
        webSocketService.connect();
    };

    const launchOrDownloadClient = async (
        target: Client,
        wasAlreadyRunning: boolean = false
    ) => {
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

            if (!wasAlreadyRunning) {
                await getCurrentWindow().minimize();
            }
        } catch (e) {
            addToast(String(e), "error", 5000);
        }
    };

    const findClientById = async (id: number | string) => {
        const clientId = typeof id === "string" ? Number(id) : id;
        if (!clientId) return null;

        const clients = await invoke<Client[]>("get_clients");
        return clients.find((c) => c.id === clientId) || null;
    };

    const setupTauriListeners = async () => {
        const unlistenTray = await listen(
            "tray-launch-client",
            async (event) => {
                const { id } = event.payload as { id: number };
                const target = await findClientById(id);
                if (target) await launchOrDownloadClient(target);
            }
        );

        const unlistenLaunch = await listen("launch-client", async (event) => {
            const { id, was_already_running } = event.payload as {
                id: string;
                was_already_running: boolean;
            };
            const target = await findClientById(id);
            if (target)
                await launchOrDownloadClient(target, was_already_running);
        });

        const unlistenLaunched = await listen(
            "client-launched",
            async (event) => {
                const payload = event.payload as {
                    id: number;
                    name: string;
                    version?: string;
                };

                localTrackerService.trackLaunch(payload.name);
                localTrackerService.startPlaytimeTracking(payload.name);

                try {
                    const version = payload.version || "unknown version";
                    globalUserStatus.setPlayingClient(
                        `${payload.name} (${version})`
                    );

                    await settingsService.loadSettings();
                    const settings = settingsService.getSettings() as any;

                    if (settings.discord_rpc_enabled?.value) {
                        await invoke("update_presence", {
                            details: t("discord.details.in_game"),
                            state: payload.name,
                        });
                    }
                } catch (error) {
                    console.error("Failed to update playing status:", error);
                }
            }
        );

        return () => {
            unlistenTray();
            unlistenLaunch();
            unlistenLaunched();
        };
    };

    const updateDiscordRPC = async (tab?: string) => {
        try {
            const settings = await fetchSettings();
            if (!settings?.discord_rpc_enabled?.value) return;

            const currentTab = tab || props.activeTab.value;
            const details = t("discord.details.in_menu");
            const state = getDiscordState(currentTab, (k: string) => t(k));

            await invoke("update_presence", { details, state });
        } catch (error) {
            console.error("Failed to update Discord RPC:", error);
        }
    };

    const playNotification = () => {
        new Audio(notificationSound)
            .play()
            .catch((e) =>
                console.error("Failed to play notification sound:", e)
            );
    };

    const handleAchievementUnlocked = (event: any) => {
        const { key } = event.detail;
        addToast(
            `${t("achievements.unlocked_title", { name: t(`achievements.list.${key}.name`) })}\n${t(`achievements.list.${key}.description`)}`,
            "success",
            9000
        );
    };

    const handleFriendRequestReceived = (event: any) => {
        playNotification();
        addToast(
            t("notifications.friend_request_received.message", {
                name: event.detail.nickname || event.detail.sender,
            }),
            "info",
            15000
        );
    };

    const handleFriendRequestAccepted = (event: any) => {
        playNotification();
        addToast(
            t("notifications.friend_request_accepted.message", {
                name: event.detail.nickname || event.detail.sender,
            }),
            "success",
            15000
        );
    };

    const handleSystemBroadcast = (event: any) => {
        playNotification();
        const { message, type, sticky } = event.detail;
        const toastType =
            type === "warning"
                ? "warning"
                : type === "error"
                  ? "error"
                  : "info";
        addToast(message, toastType as any, sticky ? 60000 : 10000);
    };

    const setupWindowListeners = () => {
        window.addEventListener(
            "achievement-unlocked",
            handleAchievementUnlocked
        );
        window.addEventListener(
            "friend-request-received",
            handleFriendRequestReceived
        );
        window.addEventListener(
            "friend-request-accepted",
            handleFriendRequestAccepted
        );
        window.addEventListener("system-broadcast", handleSystemBroadcast);

        return () => {
            window.removeEventListener(
                "achievement-unlocked",
                handleAchievementUnlocked
            );
            window.removeEventListener(
                "friend-request-received",
                handleFriendRequestReceived
            );
            window.removeEventListener(
                "friend-request-accepted",
                handleFriendRequestAccepted
            );
            window.removeEventListener(
                "system-broadcast",
                handleSystemBroadcast
            );
        };
    };

    watch(
        () => props.activeTab.value,
        async (newTab) => {
            await updateDiscordRPC(newTab);
        }
    );

    watch(locale, async () => {
        await updateDiscordRPC();
    });

    return {
        sidebarPosition,
        updateSidebarPosition,
        mainClasses,
        setActiveTab,
        handleFirstRunAccepted,
        handleDisclaimerAccepted,
        handleLoggedOut,
        handleLoggedIn,
        updateDiscordRPC,
        setupTauriListeners,
        setupWindowListeners,
    };
}

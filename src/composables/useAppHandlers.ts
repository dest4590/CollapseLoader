import { ref, computed, watch, type ComputedRef, type Ref } from "vue";
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

type SidebarPosition = "left" | "right" | "top" | "bottom";
type AuthModalView = "LOGIN" | "REGISTER" | "VERIFY";

interface AppHandlerProps {
    isAuthenticated: Ref<boolean>;
    showPreloader: Ref<boolean>;
    showFirstRunInfo: Ref<boolean>;
    showInitialDisclaimer: Ref<boolean>;
    showRegistrationPrompt: Ref<boolean>;
    activeTab: ComputedRef<string>;
    currentUserId: Ref<number | null>;
    previousTab: Ref<string>;
    authModalView: Ref<AuthModalView>;
    showAuthModal: Ref<boolean>;
    initializeUserDataWrapper?: (auth: boolean) => Promise<void>;
}

interface TrayLaunchPayload {
    id: number;
}

interface LaunchClientPayload {
    id: string;
    was_already_running: boolean;
}

interface ClientLaunchedPayload {
    id: number;
    name: string;
    version?: string;
}

export function useAppHandlers(props: AppHandlerProps) {
    const { t, locale } = useI18n();
    const { addToast } = useToast();
    const { clearUserData } = useUser();

    class ListenerGroup {
        private readonly cleanups: Array<() => void> = [];

        add(cleanup: () => void) {
            this.cleanups.push(cleanup);
        }

        dispose() {
            for (const cleanup of this.cleanups.splice(0)) {
                cleanup();
            }
        }
    }

    class ClientLaunchCoordinator {
        async findClientById(id: number | string) {
            const clientId = typeof id === "string" ? Number(id) : id;
            if (!clientId) {
                return null;
            }

            const clients = await invoke<Client[]>("get_clients");
            return clients.find((client) => client.id === clientId) || null;
        }

        async launch(target: Client, wasAlreadyRunning: boolean = false) {
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
            } catch (error) {
                addToast(String(error), "error", 5000);
            }
        }
    }

    const clientLaunchCoordinator = new ClientLaunchCoordinator();

    const sidebarPosition = ref(
        (localStorage.getItem("sidebarPosition") as SidebarPosition) || "left"
    );

    const updateSidebarPosition = (newPosition: SidebarPosition) => {
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
        refToUpdate: Ref<boolean>
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

    const findClientById = async (id: number | string) => {
        return clientLaunchCoordinator.findClientById(id);
    };

    const launchOrDownloadClient = async (
        target: Client,
        wasAlreadyRunning: boolean = false
    ) => {
        await clientLaunchCoordinator.launch(target, wasAlreadyRunning);
    };

    const setupTauriListeners = async () => {
        const listeners = new ListenerGroup();

        listeners.add(
            await listen("tray-launch-client", async (event) => {
                const { id } = event.payload as TrayLaunchPayload;
                const target = await findClientById(id);
                if (target) {
                    await launchOrDownloadClient(target);
                }
            })
        );

        listeners.add(
            await listen("launch-client", async (event) => {
                const { id, was_already_running } =
                    event.payload as LaunchClientPayload;
                const target = await findClientById(id);
                if (target) {
                    await launchOrDownloadClient(target, was_already_running);
                }
            })
        );

        listeners.add(
            await listen("client-exited", () => {
                localTrackerService.stopPlaytimeTracking();
                globalUserStatus.setOnline();
            })
        );

        listeners.add(
            await listen("client-launched", async (event) => {
                const payload = event.payload as ClientLaunchedPayload;

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
            })
        );

        return () => {
            listeners.dispose();
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
        const listeners = new ListenerGroup();
        const registerListener = (
            eventName: string,
            handler: EventListener
        ) => {
            window.addEventListener(eventName, handler);
            listeners.add(() => {
                window.removeEventListener(eventName, handler);
            });
        };

        registerListener(
            "achievement-unlocked",
            handleAchievementUnlocked as EventListener
        );
        registerListener(
            "friend-request-received",
            handleFriendRequestReceived as EventListener
        );
        registerListener(
            "friend-request-accepted",
            handleFriendRequestAccepted as EventListener
        );
        registerListener(
            "system-broadcast",
            handleSystemBroadcast as EventListener
        );

        return () => {
            listeners.dispose();
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

import { ref, computed, reactive, nextTick } from "vue";
import { apiClient } from "../services/apiClient";
import { useStreamerMode } from "./useStreamerMode";

interface StatusData {
  isOnline: boolean;
  currentClientName: string | null;
  invisibleMode: boolean;
  lastSeen: string | null;
  username: string;
  nickname: string | null;
  lastStatusUpdate: string | null;
  startedAt: string | null;
}

const globalStatus = reactive<StatusData>({
  isOnline: false,
  currentClientName: null,
  invisibleMode: false,
  lastSeen: null,
  username: "",
  nickname: null,
  lastStatusUpdate: null,
  startedAt: null,
});

const isAuthenticated = ref(false);

const pollingConfig = {
  interval: 30000,
};

let statusSyncInterval: ReturnType<typeof setInterval> | null = null;
let pendingStatusUpdate: Promise<any> | null = null;
let lastRequestId = 0;

export function useUserStatus() {
  const streamer = useStreamerMode();

  const checkAuthStatus = (): boolean => {
    const token = localStorage.getItem("authToken");
    const isAuth = !!token;
    isAuthenticated.value = isAuth;

    return isAuth;
  };

  const extractApiData = (resp: any) => (resp && resp.data ? resp.data : resp);

  const syncStatusToServer = async (force = false): Promise<any> => {
    if (!checkAuthStatus()) {
      return null;
    }

    if (pendingStatusUpdate && !force) {
      return pendingStatusUpdate;
    }

    const currentRequestId = ++lastRequestId;

    try {
      const statusPayload = {
        status:
          globalStatus.isOnline && !globalStatus.invisibleMode
            ? "ONLINE"
            : "OFFLINE",
        client_name: globalStatus.currentClientName ?? null,
      };

      pendingStatusUpdate = apiClient.put("/users/me/status", statusPayload);
      const raw = await pendingStatusUpdate;
      const response = extractApiData(raw);

      if (currentRequestId === lastRequestId) {
        updateLocalStatus(response);
        console.log(
          `Status synced: ${globalStatus.isOnline ? "online" : "offline"}${globalStatus.invisibleMode ? " (invisible)" : ""} `,
          globalStatus.currentClientName
            ? `on client: ${globalStatus.currentClientName} `
            : "",
        );
      } else {
        console.log("Ignoring stale status response");
      }

      return response;
    } catch (error) {
      console.error("Failed to sync status to server:", error);
      throw error;
    } finally {
      if (currentRequestId === lastRequestId) {
        pendingStatusUpdate = null;
      }
    }
  };

  const updateLocalStatus = (serverResponse: any): boolean => {
    const oldStatus = { ...globalStatus };

    if (!serverResponse) return false;

    if (serverResponse.status !== undefined) {
      const normalized = String(serverResponse.status || "").toUpperCase();
      globalStatus.isOnline =
        normalized.length > 0 &&
        normalized !== "OFFLINE" &&
        normalized !== "INVISIBLE";
    }
    if (serverResponse.client_name !== undefined) {
      globalStatus.currentClientName = serverResponse.client_name ?? null;
    }
    if (serverResponse.updated_at) {
      globalStatus.lastStatusUpdate = serverResponse.updated_at;
      globalStatus.lastSeen = globalStatus.isOnline
        ? null
        : serverResponse.updated_at;
    }
    if (serverResponse.started_at) {
      globalStatus.startedAt = serverResponse.started_at;
    }

    const hasChanges =
      oldStatus.isOnline !== globalStatus.isOnline ||
      oldStatus.currentClientName !== globalStatus.currentClientName ||
      oldStatus.invisibleMode !== globalStatus.invisibleMode ||
      oldStatus.startedAt !== globalStatus.startedAt;

    if (hasChanges) {
      console.log("Status change detected:", { ...globalStatus });
    }

    return hasChanges;
  };

  const setOnline = (shouldQueue: boolean = true) => {
    console.log("Setting user online (no client)");
    globalStatus.isOnline = true;
    globalStatus.currentClientName = null;
    if (shouldQueue)
      syncStatusToServer(true).catch((error) => {
        console.error("Immediate status update (online) failed:", error);
      });
  };

  const setOffline = (shouldQueue: boolean = true) => {
    console.log("Setting user offline");
    globalStatus.isOnline = false;
    globalStatus.currentClientName = null;
    if (shouldQueue)
      syncStatusToServer(true).catch((error) => {
        console.error("Immediate status update (offline) failed:", error);
      });
  };

  const setPlayingClient = (
    clientName?: string | null,
    shouldQueue: boolean = true,
  ) => {
    console.log(`Setting user playing client name: ${clientName} `);
    globalStatus.isOnline = true;
    if (clientName !== undefined)
      globalStatus.currentClientName = clientName ?? null;
    if (shouldQueue)
      syncStatusToServer(true).catch((error) => {
        console.error("Immediate status update (client_change) failed:", error);
      });
  };

  const setInvisibleMode = (enable: boolean, shouldQueue: boolean = true) => {
    console.log(`Setting invisible mode: ${enable ? "enabled" : "disabled"} `);
    globalStatus.invisibleMode = enable;

    if (enable) {
      globalStatus.isOnline = false;
      globalStatus.currentClientName = null;
    } else {
      globalStatus.isOnline = true;
    }
    if (shouldQueue)
      syncStatusToServer(true).catch((error) => {
        console.error(
          "Immediate status update (invisible_toggle) failed:",
          error,
        );
      });
  };

  const setStreamerMode = (enable: boolean) => {
    console.log(`Setting streamer mode: ${enable ? "enabled" : "disabled"} `);
    streamer.setStreamerMode(enable);
  };

  const startPolling = () => {
    if (statusSyncInterval) {
      clearInterval(statusSyncInterval);
    }

    const pollWrapper = async () => {
      if (checkAuthStatus()) {
        syncStatusToServer().catch((error) => {
          console.error("Scheduled status sync failed:", error);
        });
      } else {
        console.log("Auth check failed in sync interval, stopping sync");
        await stopStatusSync();
      }
    };

    statusSyncInterval = setInterval(pollWrapper, pollingConfig.interval);

    document.addEventListener("visibilitychange", () => {
      if (document.visibilityState === "visible") {
        pollWrapper().catch(() => {});
      }
    });
  };

  const startStatusSync = () => {
    if (!checkAuthStatus()) {
      console.log("Cannot start status sync - user not authenticated");
      return;
    }

    syncStatusToServer(true).catch((error) => {
      console.error("Failed to sync status on start:", error);
    });

    startPolling();
  };

  const stopStatusSync = async () => {
    console.log("Stopping status sync...");

    if (statusSyncInterval) {
      clearInterval(statusSyncInterval);
      statusSyncInterval = null;
    }

    if (checkAuthStatus() && globalStatus.isOnline) {
      setOffline(false);
      try {
        await syncStatusToServer(true);
      } catch (error) {
        console.error("Failed to mark user offline on stop:", error);
      }
    }
    console.log("Status sync stopped");
  };

  const forceSyncStatus = async () => {
    return await syncStatusToServer(true);
  };

  const fetchCurrentStatus = async () => {
    if (!checkAuthStatus()) return null;

    try {
      const statusRaw = await apiClient.get("/users/me/status");
      const status = extractApiData(statusRaw);
      updateLocalStatus(status);
      return status;
    } catch (error) {
      console.error("Failed to fetch current status:", error);
      return null;
    }
  };

  const initializeStatusSystem = () => {
    checkAuthStatus();
    if (isAuthenticated.value) {
      setOnline(false);
      startStatusSync();
    } else {
      console.log(
        "User not authenticated, skipping status system initialization",
      );
    }
  };

  const restartStatusSystem = async () => {
    await stopStatusSync();
    await nextTick();
    initializeStatusSystem();
  };

  const isOnline = computed(() => globalStatus.isOnline);
  const isInvisible = computed(() => globalStatus.invisibleMode);
  const isStreamer = computed(() => streamer.enabled.value);
  const lastSeen = computed(() => globalStatus.lastSeen);
  const username = computed(() => globalStatus.username);
  const nickname = computed(() => globalStatus.nickname);

  return {
    isAuthenticated,
    globalStatus,

    isOnline,
    isInvisible,
    isStreamer,
    lastSeen,
    username,
    nickname,

    setOnline,
    setOffline,
    setPlayingClient,
    setStreamerMode,
    setInvisibleMode,

    syncStatusToServer,
    forceSyncStatus,
    startStatusSync,
    stopStatusSync,
    initializeStatusSystem,
    restartStatusSystem,
    fetchCurrentStatus,
    checkAuthStatus,
  };
}

export const globalUserStatus = useUserStatus();

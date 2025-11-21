import { ref, computed, reactive } from 'vue';
import { apiClient, apiHeartbeat } from '../services/apiClient';
import { useStreamerMode } from './useStreamerMode';

interface StatusData {
    isOnline: boolean;
    currentClient: string | null;
    invisibleMode: boolean;
    lastSeen: string | null;
    username: string;
    nickname: string | null;
    lastStatusUpdate: string | null;
}


const globalStatus = reactive<StatusData>({
    isOnline: false,
    currentClient: null,
    invisibleMode: false,
    lastSeen: null,
    username: '',
    nickname: null,
    lastStatusUpdate: null
});


const isAuthenticated = ref(false);

const pollingConfig = {
    interval: 30000,
};

const heartbeatConfig = {
    interval: 45000,
    enabled: true,
    maxConsecutiveErrors: 3,
    consecutiveErrors: 0
};

let statusSyncInterval: ReturnType<typeof setInterval> | null = null;
let heartbeatInterval: ReturnType<typeof setInterval> | null = null;
let pendingStatusUpdate: Promise<any> | null = null;

export function useUserStatus() {
    const streamer = useStreamerMode();

    const checkAuthStatus = (): boolean => {
        const token = localStorage.getItem('authToken');
        const isAuth = !!token;
        isAuthenticated.value = isAuth;

        return isAuth;
    };

    const sendHeartbeat = async (): Promise<boolean> => {
        if (!checkAuthStatus()) {
            return false;
        }

        try {
            await apiHeartbeat();
            heartbeatConfig.consecutiveErrors = 0;
            return true;
        } catch (error) {
            heartbeatConfig.consecutiveErrors++;
            console.error('Heartbeat failed:', error);

            if (heartbeatConfig.consecutiveErrors >= heartbeatConfig.maxConsecutiveErrors) {
                console.warn('Too many heartbeat errors, disabling for this session');
                heartbeatConfig.enabled = false;
            }
            return false;
        }
    };

    const syncStatusToServer = async (force = false): Promise<any> => {
        if (!checkAuthStatus()) {
            return null;
        }

        if (pendingStatusUpdate && !force) {
            return pendingStatusUpdate;
        }


        try {
            const statusPayload = {
                is_online: globalStatus.isOnline,
                current_client: globalStatus.currentClient || undefined,
                invisible_mode: globalStatus.invisibleMode
            };

            pendingStatusUpdate = apiClient.post('/auth/status/', statusPayload);
            const response = await pendingStatusUpdate;

            updateLocalStatus(response);

            console.log(`Status synced: ${globalStatus.isOnline ? 'online' : 'offline'}${globalStatus.invisibleMode ? ' (invisible)' : ''}`,
                globalStatus.currentClient ? `playing ${globalStatus.currentClient}` : '');

            return response;
        } catch (error) {
            console.error('Failed to sync status to server:', error);
            throw error;
        } finally {
            pendingStatusUpdate = null;
        }
    };

    const updateLocalStatus = (serverResponse: any): boolean => {
        const oldStatus = { ...globalStatus };

        if (typeof serverResponse.is_online !== 'undefined') globalStatus.isOnline = !!serverResponse.is_online;
        if ('current_client' in serverResponse) globalStatus.currentClient = serverResponse.current_client || null;
        if (serverResponse.username) globalStatus.username = serverResponse.username;
        if (serverResponse.nickname !== undefined) globalStatus.nickname = serverResponse.nickname;
        if (serverResponse.last_seen) globalStatus.lastSeen = serverResponse.last_seen;
        if (serverResponse.invisible_mode !== undefined) globalStatus.invisibleMode = serverResponse.invisible_mode;
        if (serverResponse.last_status_update) globalStatus.lastStatusUpdate = serverResponse.last_status_update;
        const hasChanges = (
            oldStatus.isOnline !== globalStatus.isOnline ||
            oldStatus.currentClient !== globalStatus.currentClient ||
            oldStatus.invisibleMode !== globalStatus.invisibleMode
        );

        if (hasChanges) {
            console.log('Status change detected:', { ...globalStatus });
        }

        return hasChanges;
    };

    const setOnline = (shouldQueue: boolean = true) => {
        console.log('Setting user online (no client)');
        globalStatus.isOnline = true;
        globalStatus.currentClient = null;
        if (shouldQueue) syncStatusToServer(true).catch(error => {
            console.error('Immediate status update (online) failed:', error);
        });
    };

    const setOffline = (shouldQueue: boolean = true) => {
        console.log('Setting user offline');
        globalStatus.isOnline = false;
        globalStatus.currentClient = null;
        if (shouldQueue) syncStatusToServer(true).catch(error => {
            console.error('Immediate status update (offline) failed:', error);
        });
    };

    const setPlayingClient = (clientName: string, shouldQueue: boolean = true) => {
        console.log(`Setting user playing client: ${clientName}`);
        globalStatus.isOnline = true;
        globalStatus.currentClient = clientName;
        if (shouldQueue) syncStatusToServer(true).catch(error => {
            console.error('Immediate status update (client_change) failed:', error);
        });
    };

    const setInvisibleMode = (enable: boolean, shouldQueue: boolean = true) => {
        console.log(`Setting invisible mode: ${enable ? 'enabled' : 'disabled'}`);
        globalStatus.invisibleMode = enable;

        if (enable) {
            globalStatus.isOnline = false;
            globalStatus.currentClient = null;
        } else {
            globalStatus.isOnline = true;
        }
        if (shouldQueue) syncStatusToServer(true).catch(error => {
            console.error('Immediate status update (invisible_toggle) failed:', error);
        });
    };

    const setStreamerMode = (enable: boolean) => {
        console.log(`Setting streamer mode: ${enable ? 'enabled' : 'disabled'}`);
        streamer.setStreamerMode(enable);
    };




    const startPolling = () => {
        if (statusSyncInterval) {
            clearInterval(statusSyncInterval);
        }
        if (heartbeatInterval) {
            clearInterval(heartbeatInterval);
        }

        const pollWrapper = async () => {
            if (document && document.visibilityState === 'hidden') {
                return;
            }

            if (checkAuthStatus()) {
                syncStatusToServer().catch(error => {
                    console.error('Scheduled status sync failed:', error);
                });
            } else {
                console.log('Auth check failed in sync interval, stopping sync');
                stopStatusSync();
            }
        };

        statusSyncInterval = setInterval(pollWrapper, pollingConfig.interval);

        document.addEventListener('visibilitychange', () => {
            if (document.visibilityState === 'visible') {
                pollWrapper().catch(() => { });
            }
        });

        if (heartbeatConfig.enabled) {
            heartbeatInterval = setInterval(() => {
                if (checkAuthStatus() && heartbeatConfig.enabled) {
                    sendHeartbeat();
                }
            }, heartbeatConfig.interval);
        }
    };

    const startStatusSync = () => {
        if (!checkAuthStatus()) {
            console.log('Cannot start status sync - user not authenticated');
            return;
        }

        syncStatusToServer(true).catch(error => {
            console.error('Failed to sync status on start:', error);
        });

        startPolling();
    };


    const stopStatusSync = () => {
        console.log('Stopping status sync...');

        if (statusSyncInterval) {
            clearInterval(statusSyncInterval);
            statusSyncInterval = null;
        }

        if (heartbeatInterval) {
            clearInterval(heartbeatInterval);
            heartbeatInterval = null;
        }


        if (checkAuthStatus() && globalStatus.isOnline) {
            setOffline();
            syncStatusToServer(true).catch(error => {
                console.error('Failed to mark user offline on stop:', error);
            });
        }
        console.log('Status sync stopped');
    };


    const forceSyncStatus = async () => {
        return await syncStatusToServer(true);
    };


    const fetchCurrentStatus = async () => {
        if (!checkAuthStatus()) return null;

        try {
            const status = await apiClient.get('/auth/status/');

            globalStatus.isOnline = status.is_online;
            globalStatus.currentClient = status.current_client || null;
            globalStatus.invisibleMode = status.invisible_mode || false;
            globalStatus.lastSeen = status.last_seen;
            globalStatus.username = status.username;
            globalStatus.nickname = status.nickname || null;

            return status;
        } catch (error) {
            console.error('Failed to fetch current status:', error);
            return null;
        }
    };


    const initializeStatusSystem = () => {
        checkAuthStatus();
        if (isAuthenticated.value) {

            setOnline(false);
            startStatusSync();
        } else {
            console.log('User not authenticated, skipping status system initialization');
        }
    };


    const restartStatusSystem = () => {
        stopStatusSync();
        setTimeout(() => {
            initializeStatusSystem();
        }, 1000);
    };

    const isOnline = computed(() => globalStatus.isOnline);
    const currentClient = computed(() => globalStatus.currentClient);
    const isInvisible = computed(() => globalStatus.invisibleMode);
    const isStreamer = computed(() => streamer.enabled.value);
    const lastSeen = computed(() => globalStatus.lastSeen);
    const username = computed(() => globalStatus.username);
    const nickname = computed(() => globalStatus.nickname);

    return {
        isAuthenticated,
        globalStatus,

        isOnline,
        currentClient,
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
        checkAuthStatus
    };
}

export const globalUserStatus = useUserStatus();
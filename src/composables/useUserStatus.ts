import { ref, computed, reactive } from 'vue';
import { apiClient } from '../services/apiClient';

interface StatusData {
    isOnline: boolean;
    currentClient: string | null;
    invisibleMode: boolean;
    streamerMode: boolean;
    lastSeen: string | null;
    username: string;
    nickname: string | null;
    lastStatusUpdate: string | null;
}

interface StatusChangeEvent {
    type: 'online' | 'offline' | 'client_change' | 'invisible_toggle';
    timestamp: number;
    data: Partial<StatusData>;
}

interface StatusMetrics {
    totalUpdates: number;
    skippedUpdates: number;
    errorCount: number;
    avgResponseTime: number;
    lastSuccessfulSync: number;
    consecutiveErrors: number;
}

const globalStatus = reactive<StatusData>({
    isOnline: false,
    currentClient: null,
    invisibleMode: false,
    streamerMode: false,
    lastSeen: null,
    username: '',
    nickname: null,
    lastStatusUpdate: null
});

const statusMetrics = reactive<StatusMetrics>({
    totalUpdates: 0,
    skippedUpdates: 0,
    errorCount: 0,
    avgResponseTime: 0,
    lastSuccessfulSync: 0,
    consecutiveErrors: 0
});

const isAuthenticated = ref(false);
const connectionStatus = ref<'online' | 'offline' | 'connecting' | 'error'>('offline');
const lastStatusUpdate = ref<Date | null>(null);
const statusChangeQueue: StatusChangeEvent[] = [];

const pollingConfig = {
    baseInterval: 30000,
    maxInterval: 300000,
    backoffMultiplier: 1.5,
    currentInterval: 30000,
    consecutiveUnchangedPolls: 0,
    maxUnchangedPolls: 5,
};

let statusSyncInterval: ReturnType<typeof setInterval> | null = null;
let changeDetectionTimeout: NodeJS.Timeout | null = null;
let pendingStatusUpdate: Promise<any> | null = null;

export function useUserStatus() {

    const checkAuthStatus = (): boolean => {
        const token = localStorage.getItem('authToken');
        const isAuth = !!token;
        isAuthenticated.value = isAuth;

        const savedStreamerMode = localStorage.getItem('streamerModeEnabled') === 'true';
        globalStatus.streamerMode = savedStreamerMode;

        return isAuth;
    };

    const syncStatusToServer = async (force = false): Promise<any> => {
        if (!checkAuthStatus()) {
            connectionStatus.value = 'offline';
            return null;
        }

        if (pendingStatusUpdate && !force) {
            console.log('Status update already in progress, skipping');
            statusMetrics.skippedUpdates++;
            return pendingStatusUpdate;
        }

        connectionStatus.value = 'connecting';
        const startTime = Date.now();

        try {
            const statusPayload = {
                is_online: globalStatus.isOnline,
                current_client: globalStatus.currentClient || undefined,
                invisible_mode: globalStatus.invisibleMode
            };

            pendingStatusUpdate = apiClient.post('/auth/status/', statusPayload);
            const response = await pendingStatusUpdate;

            const wasChanged = updateLocalStatus(response);

            const responseTime = Date.now() - startTime;
            updateResponseTimeMetric(responseTime);
            statusMetrics.totalUpdates++;
            statusMetrics.lastSuccessfulSync = Date.now();
            statusMetrics.consecutiveErrors = 0;

            adjustPollingFrequency(wasChanged);

            lastStatusUpdate.value = new Date();
            connectionStatus.value = globalStatus.isOnline ? 'online' : 'offline';

            console.log(`Status synced (${responseTime}ms): ${globalStatus.isOnline ? 'online' : 'offline'}${globalStatus.invisibleMode ? ' (invisible)' : ''}`,
                globalStatus.currentClient ? `playing ${globalStatus.currentClient}` : '');

            return response;
        } catch (error) {
            console.error('Failed to sync status to server:', error);
            statusMetrics.errorCount++;
            statusMetrics.consecutiveErrors++;

            if (statusMetrics.consecutiveErrors > 3) {
                pollingConfig.currentInterval = Math.min(
                    pollingConfig.currentInterval * pollingConfig.backoffMultiplier,
                    pollingConfig.maxInterval
                );
                console.log(`Increased polling interval to ${pollingConfig.currentInterval}ms due to errors`);
            }

            connectionStatus.value = 'error';
            throw error;
        } finally {
            pendingStatusUpdate = null;
        }
    };


    const updateLocalStatus = (serverResponse: any): boolean => {
        const oldStatus = { ...globalStatus };

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
            const changeEvent: StatusChangeEvent = {
                type: globalStatus.isOnline ? 'online' : 'offline',
                timestamp: Date.now(),
                data: { ...globalStatus }
            };
            statusChangeQueue.push(changeEvent);

            if (statusChangeQueue.length > 50) {
                statusChangeQueue.shift();
            }
        }

        return hasChanges;
    };

    const adjustPollingFrequency = (hasChanges: boolean) => {
        if (hasChanges) {
            pollingConfig.currentInterval = pollingConfig.baseInterval;
            pollingConfig.consecutiveUnchangedPolls = 0;
            console.log('Status changes detected, reset polling to base interval');
        } else {
            pollingConfig.consecutiveUnchangedPolls++;

            if (pollingConfig.consecutiveUnchangedPolls >= pollingConfig.maxUnchangedPolls) {
                const newInterval = Math.min(
                    pollingConfig.currentInterval * 1.2,
                    pollingConfig.maxInterval
                );

                if (newInterval !== pollingConfig.currentInterval) {
                    pollingConfig.currentInterval = newInterval;
                    console.log(`Increased polling interval to ${newInterval}ms (no changes for ${pollingConfig.consecutiveUnchangedPolls} polls)`);
                }
            }
        }

        if (statusSyncInterval) {
            clearInterval(statusSyncInterval);
            startPolling();
        }
    };


    const updateResponseTimeMetric = (responseTime: number) => {
        statusMetrics.avgResponseTime = statusMetrics.avgResponseTime * 0.8 + responseTime * 0.2;
    };

    const setOnline = () => {
        console.log('Setting user online (no client)');
        globalStatus.isOnline = true;
        globalStatus.currentClient = null;
        queueStatusUpdate('online');
    };

    const setOffline = () => {
        console.log('Setting user offline');
        globalStatus.isOnline = false;
        globalStatus.currentClient = null;
        queueStatusUpdate('offline');
    };

    const setPlayingClient = (clientName: string) => {
        console.log(`Setting user playing client: ${clientName}`);
        globalStatus.isOnline = true;
        globalStatus.currentClient = clientName;
        queueStatusUpdate('client_change');
    };

    const setInvisibleMode = (enable: boolean) => {
        console.log(`Setting invisible mode: ${enable ? 'enabled' : 'disabled'}`);
        globalStatus.invisibleMode = enable;

        if (enable) {
            globalStatus.isOnline = false;
            globalStatus.currentClient = null;
        } else {
            globalStatus.isOnline = true;
        }
        queueStatusUpdate('invisible_toggle');
    };

    const setStreamerMode = (enable: boolean) => {
        console.log(`Setting streamer mode: ${enable ? 'enabled' : 'disabled'}`);
        globalStatus.streamerMode = enable;
        localStorage.setItem('streamerModeEnabled', enable.toString());
    };


    const queueStatusUpdate = (_: StatusChangeEvent['type']) => {
        if (changeDetectionTimeout) {
            clearTimeout(changeDetectionTimeout);
        }

        changeDetectionTimeout = setTimeout(() => {
            syncStatusToServer(true).catch(error => {
                console.error('Queued status update failed:', error);
            });
        }, 500);
    };


    const startPolling = () => {
        if (statusSyncInterval) {
            clearInterval(statusSyncInterval);
        }

        statusSyncInterval = setInterval(() => {
            if (checkAuthStatus()) {
                syncStatusToServer().catch(error => {
                    console.error('Scheduled status sync failed:', error);
                });
            } else {
                console.log('Auth check failed in sync interval, stopping sync');
                stopStatusSync();
            }
        }, pollingConfig.currentInterval);

        console.log(`Started intelligent polling with ${pollingConfig.currentInterval}ms interval`);
    };

    const startStatusSync = () => {
        if (!checkAuthStatus()) {
            console.log('Cannot start status sync - user not authenticated');
            return;
        }

        console.log('Starting status sync system');

        syncStatusToServer(true).catch(error => {
            console.error('Failed to sync status on start:', error);
        });

        startPolling();

        console.log('Status sync system started');
    };


    const stopStatusSync = () => {
        console.log('Stopping status sync...');

        if (statusSyncInterval) {
            clearInterval(statusSyncInterval);
            statusSyncInterval = null;
        }

        if (changeDetectionTimeout) {
            clearTimeout(changeDetectionTimeout);
            changeDetectionTimeout = null;
        }

        if (checkAuthStatus() && globalStatus.isOnline) {
            setOffline();
            syncStatusToServer(true).catch(error => {
                console.error('Failed to mark user offline on stop:', error);
            });
        }

        connectionStatus.value = 'offline';
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
            console.log('Initializing status sync system...');

            pollingConfig.currentInterval = pollingConfig.baseInterval;
            pollingConfig.consecutiveUnchangedPolls = 0;

            setOnline();
            startStatusSync();

            console.log('Status sync system initialized');
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
    const isStreamer = computed(() => globalStatus.streamerMode);
    const lastSeen = computed(() => globalStatus.lastSeen);
    const username = computed(() => globalStatus.username);
    const nickname = computed(() => globalStatus.nickname);

    return {
        isAuthenticated,
        connectionStatus,
        lastStatusUpdate,
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
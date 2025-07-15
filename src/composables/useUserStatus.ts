import { ref, computed, reactive } from 'vue';
import { userService } from '../services/userService';

const globalStatus = reactive({
    isOnline: false,
    currentClient: null as string | null,
    clientVersion: null as string | null,
    invisibleMode: false,
    streamerMode: false,
    lastSeen: null as string | null,
    username: '',
    nickname: null as string | null
});

const isAuthenticated = ref(false);
const connectionStatus = ref<'online' | 'offline' | 'connecting'>('offline');
const lastStatusUpdate = ref<Date | null>(null);

let statusSyncInterval: ReturnType<typeof setInterval> | null = null;

export function useUserStatus() {
    const checkAuthStatus = () => {
        const token = localStorage.getItem('authToken');
        const isAuth = !!token;
        isAuthenticated.value = isAuth;

        const savedStreamerMode = localStorage.getItem('streamerModeEnabled') === 'true';
        globalStatus.streamerMode = savedStreamerMode;

        return isAuth;
    };

    /**
     * Sends the current user status to the server
     */
    const syncStatusToServer = async () => {
        if (!checkAuthStatus()) {
            connectionStatus.value = 'offline';
            return;
        }

        try {
            connectionStatus.value = 'connecting';

            const status = await userService.updateUserStatus(
                globalStatus.isOnline,
                globalStatus.currentClient || undefined,
                globalStatus.clientVersion || undefined,
                globalStatus.invisibleMode
            );

            globalStatus.username = status.username;
            globalStatus.nickname = status.nickname || null;
            globalStatus.lastSeen = status.last_seen;
            if (status.invisible_mode !== undefined) {
                globalStatus.invisibleMode = status.invisible_mode;
            }

            lastStatusUpdate.value = new Date();
            connectionStatus.value = globalStatus.isOnline ? 'online' : 'offline';

            console.log(`Status synced: ${globalStatus.isOnline ? 'online' : 'offline'}${globalStatus.invisibleMode ? ' (invisible)' : ''}`,
                globalStatus.currentClient ? `playing ${globalStatus.currentClient}` : '');

            return status;
        } catch (error) {
            console.error('Failed to sync status to server:', error);
            connectionStatus.value = 'offline';
            throw error;
        }
    };

    /**
     * Set the user status to online without a specific client
     */
    const setOnline = () => {
        console.log('Setting user online (no client)');
        globalStatus.isOnline = true;
        globalStatus.currentClient = null;
        globalStatus.clientVersion = null;
    };
    /**
     * Sets the user offline
     */
    const setOffline = () => {
        console.log('Setting user offline');
        globalStatus.isOnline = false;
        globalStatus.currentClient = null;
        globalStatus.clientVersion = null;
    };

    /**
     * Sets the status to "playing client"
     */
    const setPlayingClient = (clientName: string, clientVersion?: string) => {
        console.log(`Setting user playing client: ${clientName}${clientVersion ? ` v${clientVersion}` : ''}`);
        globalStatus.isOnline = true;
        globalStatus.currentClient = clientName;
        globalStatus.clientVersion = clientVersion || null;
    };

    /**
     * Toggles invisible mode
     */
    const setInvisibleMode = (enable: boolean) => {
        console.log(`Setting invisible mode: ${enable ? 'enabled' : 'disabled'}`);
        globalStatus.invisibleMode = enable;

        if (enable) {
            globalStatus.isOnline = false;
            globalStatus.currentClient = null;
            globalStatus.clientVersion = null;
        } else {
            globalStatus.isOnline = true;
        }
    };

    /**
     * Toggles streamer mode
     */
    const setStreamerMode = (enable: boolean) => {
        console.log(`Setting streamer mode: ${enable ? 'enabled' : 'disabled'}`);
        globalStatus.streamerMode = enable;
        localStorage.setItem('streamerModeEnabled', enable.toString());
    };

    /**
     * Forces status synchronization with the server (immediately)
     */
    const forceSyncStatus = async () => {
        return await syncStatusToServer();
    };

    /**
     * Starts automatic status synchronization every 10 seconds
     */
    const startStatusSync = () => {
        if (!checkAuthStatus()) {
            console.log('Cannot start status sync - user not authenticated');
            return;
        }

        if (statusSyncInterval) {
            console.log('Status sync already running, stopping previous interval');
            clearInterval(statusSyncInterval);
        }

        console.log('Starting status sync (interval: 10 seconds)');

        syncStatusToServer().catch(error => {
            console.error('Failed to sync status on start:', error);
        });

        statusSyncInterval = setInterval(() => {
            if (checkAuthStatus()) {
                syncStatusToServer().catch(error => {
                    console.error('Status sync failed:', error);
                });
            } else {
                console.log('Auth check failed in sync interval, stopping sync');
                stopStatusSync();
            }
        }, 10000);

        console.log('Status sync started successfully');
    };

    /**
     * Stops automatic synchronization
     */
    const stopStatusSync = () => {
        console.log('Stopping status sync...');

        if (statusSyncInterval) {
            clearInterval(statusSyncInterval);
            statusSyncInterval = null;
        }

        if (checkAuthStatus() && globalStatus.isOnline) {
            setOffline();
            syncStatusToServer().catch(error => {
                console.error('Failed to mark user offline on stop:', error);
            });
        }

        connectionStatus.value = 'offline';
        console.log('Status sync stopped');
    };

    /**
     * Initializes the status system
     */
    const initializeStatusSystem = () => {
        checkAuthStatus();
        if (isAuthenticated.value) {
            console.log('Initializing status system...');

            setOnline();
            startStatusSync();

            console.log('Status system initialized');
        } else {
            console.log('User not authenticated, skipping status system initialization');
        }
    };

    /**
     * Restarts the status system
     */
    const restartStatusSystem = () => {
        stopStatusSync();
        setTimeout(() => {
            initializeStatusSystem();
        }, 1000);
    };

    /**
     * Gets the current status from the server
     */
    const fetchCurrentStatus = async () => {
        if (!checkAuthStatus()) return null;

        try {
            const status = await userService.getUserStatus();

            globalStatus.isOnline = status.is_online;
            globalStatus.currentClient = status.current_client || null;
            globalStatus.clientVersion = status.client_version || null;
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

    const isOnline = computed(() => globalStatus.isOnline);
    const currentClient = computed(() => globalStatus.currentClient);
    const clientVersion = computed(() => globalStatus.clientVersion);
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
        clientVersion,
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
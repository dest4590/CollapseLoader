import { ref, computed, reactive, watch } from 'vue';
import { apiClient } from '../services/apiClient';

interface Friend {
    id: number;
    username: string;
    nickname?: string;
    status: {
        is_online: boolean;
        last_seen: string | null;
        current_client?: string;
        invisible_mode?: boolean;
    };
}

interface FriendRequest {
    id: number;
    requester: Friend;
    addressee: Friend;
    status: 'pending' | 'accepted' | 'blocked';
    created_at: string;
    updated_at: string;
}

interface FriendsMetrics {
    totalFriends: number;
    onlineFriends: number;
    lastBulkUpdate: number;
    cacheHitRate: number;
    statusUpdateCount: number;
    avgStatusResponseTime: number;
}

interface GlobalFriendsState {
    friends: Friend[];
    sentRequests: FriendRequest[];
    receivedRequests: FriendRequest[];
    isLoading: boolean;
    isLoaded: boolean;
    lastUpdated: string | null;
    lastStatusUpdate: number;
    bulkUpdateCount: number;
}

const globalFriendsState = reactive<GlobalFriendsState>({
    friends: [],
    sentRequests: [],
    receivedRequests: [],
    isLoading: false,
    isLoaded: false,
    lastUpdated: null,
    lastStatusUpdate: 0,
    bulkUpdateCount: 0
});

const friendsMetrics = reactive<FriendsMetrics>({
    totalFriends: 0,
    onlineFriends: 0,
    lastBulkUpdate: 0,
    cacheHitRate: 0,
    statusUpdateCount: 0,
    avgStatusResponseTime: 0
});

const isStatusLoading = ref(false);
const previousReceivedCount = ref(0);
const statusUpdateInterval: { current: NodeJS.Timeout | null } = { current: null };

const statusUpdateConfig = {
    baseInterval: 45000,
    maxInterval: 180000,
    currentInterval: 45000,
    backoffMultiplier: 1.3,
    consecutiveNoChanges: 0,
    maxNoChanges: 4
};

const isAuthenticated = computed(() => !!localStorage.getItem('authToken'));

export function useFriends() {

    const onlineFriends = computed(() =>
        globalFriendsState.friends.filter(friend =>
            friend.status.is_online && !friend.status.invisible_mode
        )
    );

    const onlineFriendsCount = computed(() => onlineFriends.value.length);

    const friendRequests = computed(() => ({
        sent: globalFriendsState.sentRequests,
        received: globalFriendsState.receivedRequests
    }));

    const loadFriendsData = async (forceRefresh = false): Promise<void> => {
        if (!isAuthenticated.value) {
            clearFriendsData();
            return;
        }

        if (globalFriendsState.isLoading) {
            console.log('Friends data loading already in progress');
            return;
        }

        if (globalFriendsState.isLoaded && !forceRefresh &&
            globalFriendsState.lastUpdated &&
            Date.now() - new Date(globalFriendsState.lastUpdated).getTime() < 30000) {
            return;
        }

        globalFriendsState.isLoading = true;
        const startTime = Date.now();

        try {
            console.log('Loading friends data via batch endpoint...');

            const batchData = await apiClient.get('/auth/friends/batch/');

            const responseTime = Date.now() - startTime;
            console.log(`Batch friends data loaded in ${responseTime}ms:`, {
                friends: batchData.friends?.length || 0,
                sentRequests: batchData.requests?.sent?.length || 0,
                receivedRequests: batchData.requests?.received?.length || 0,
                optimized: batchData.performance_info?.optimized || false
            });

            checkForNewRequests({
                sent: batchData.requests?.sent || [],
                received: batchData.requests?.received || []
            });

            globalFriendsState.friends = batchData.friends || [];
            globalFriendsState.sentRequests = batchData.requests?.sent || [];
            globalFriendsState.receivedRequests = batchData.requests?.received || [];
            globalFriendsState.lastUpdated = new Date().toISOString();
            globalFriendsState.isLoaded = true;
            globalFriendsState.bulkUpdateCount++;

            friendsMetrics.totalFriends = globalFriendsState.friends.length;
            friendsMetrics.onlineFriends = onlineFriendsCount.value;
            friendsMetrics.lastBulkUpdate = Date.now();
            friendsMetrics.cacheHitRate = apiClient.getCacheStats().hitRate;

            if (!statusUpdateInterval.current) {
                startStatusUpdates();
            }

            console.log('Friends data loaded successfully');
        } catch (error) {
            console.error('Failed to load friends data via batch endpoint:', error);

            await loadFriendsDataFallback();
        } finally {
            globalFriendsState.isLoading = false;
        }
    };


    const loadFriendsDataFallback = async (): Promise<void> => {
        try {
            console.log('Using fallback method for friends data...');

            const [friendsResult, requestsResult] = await Promise.all([
                apiClient.get('/auth/friends/'),
                apiClient.get('/auth/friends/requests/')
            ]);

            globalFriendsState.friends = friendsResult || [];
            globalFriendsState.sentRequests = requestsResult?.sent || [];
            globalFriendsState.receivedRequests = requestsResult?.received || [];
            globalFriendsState.lastUpdated = new Date().toISOString();
            globalFriendsState.isLoaded = true;

            friendsMetrics.totalFriends = globalFriendsState.friends.length;
            friendsMetrics.onlineFriends = onlineFriendsCount.value;

            console.log('Friends data loaded successfully via fallback method');
        } catch (fallbackError) {
            console.error('Fallback friends data loading also failed:', fallbackError);
        }
    };


    const updateFriendStatuses = async (): Promise<boolean> => {
        if (globalFriendsState.friends.length === 0 || isStatusLoading.value) {
            return false;
        }

        const startTime = Date.now();
        let hasChanges = false;

        try {
            isStatusLoading.value = true;
            friendsMetrics.statusUpdateCount++;

            const statusesData = await apiClient.get('/auth/friends/status/');

            globalFriendsState.friends.forEach(friend => {
                const updatedStatus = statusesData.find((status: any) => status.username === friend.username);
                if (updatedStatus) {
                    const oldOnlineStatus = friend.status.is_online;
                    const oldClient = friend.status.current_client;

                    friend.status.is_online = updatedStatus.is_online;
                    friend.status.last_seen = updatedStatus.last_seen;
                    friend.status.current_client = updatedStatus.current_client;
                    friend.status.invisible_mode = updatedStatus.invisible_mode;

                    if (oldOnlineStatus !== updatedStatus.is_online ||
                        oldClient !== updatedStatus.current_client) {
                        hasChanges = true;
                    }
                }
            });

            const responseTime = Date.now() - startTime;
            friendsMetrics.avgStatusResponseTime = friendsMetrics.avgStatusResponseTime * 0.8 + responseTime * 0.2;
            friendsMetrics.onlineFriends = onlineFriendsCount.value;
            globalFriendsState.lastStatusUpdate = Date.now();

            console.log(`Friend statuses updated in ${responseTime}ms (${hasChanges ? 'with changes' : 'no changes'})`);
        } catch (error) {
            console.error('Failed to update friend statuses:', error);
            hasChanges = false;
        } finally {
            isStatusLoading.value = false;
        }

        return hasChanges;
    };


    const startStatusUpdates = (): void => {
        if (statusUpdateInterval.current) {
            clearInterval(statusUpdateInterval.current);
        }

        const runStatusUpdate = async () => {
            if (document && document.visibilityState === 'hidden') {
                return;
            }

            if (!isAuthenticated.value || globalFriendsState.friends.length === 0) {
                return;
            }

            const hasChanges = await updateFriendStatuses();
            adjustStatusUpdateFrequency(hasChanges);
        };

        runStatusUpdate();

        statusUpdateInterval.current = setInterval(runStatusUpdate, statusUpdateConfig.currentInterval);

        document.addEventListener('visibilitychange', () => {
            if (document.visibilityState === 'visible') {
                runStatusUpdate().catch(() => { });
            }
        });

        console.log(`Started status updates with ${statusUpdateConfig.currentInterval}ms interval`);
    };


    const adjustStatusUpdateFrequency = (hasChanges: boolean): void => {
        if (hasChanges) {
            statusUpdateConfig.currentInterval = statusUpdateConfig.baseInterval;
            statusUpdateConfig.consecutiveNoChanges = 0;
        } else {
            statusUpdateConfig.consecutiveNoChanges++;

            if (statusUpdateConfig.consecutiveNoChanges >= statusUpdateConfig.maxNoChanges) {
                const newInterval = Math.min(
                    statusUpdateConfig.currentInterval * statusUpdateConfig.backoffMultiplier,
                    statusUpdateConfig.maxInterval
                );

                if (newInterval !== statusUpdateConfig.currentInterval) {
                    statusUpdateConfig.currentInterval = newInterval;
                    console.log(`Increased status update interval to ${newInterval}ms (no changes for ${statusUpdateConfig.consecutiveNoChanges} updates)`);
                }
            }
        }

        if (statusUpdateInterval.current) {
            clearInterval(statusUpdateInterval.current);
            statusUpdateInterval.current = setInterval(async () => {
                if (isAuthenticated.value && globalFriendsState.friends.length > 0) {
                    const changes = await updateFriendStatuses();
                    adjustStatusUpdateFrequency(changes);
                }
            }, statusUpdateConfig.currentInterval);
        }
    };


    const checkForNewRequests = (currentRequests: { sent: FriendRequest[]; received: FriendRequest[] }): FriendRequest[] => {
        if (currentRequests.received.length > previousReceivedCount.value) {
            const knownRequestIds = new Set(globalFriendsState.receivedRequests.map(req => req.id));
            const newRequests = currentRequests.received.filter(req => !knownRequestIds.has(req.id));
            previousReceivedCount.value = currentRequests.received.length;
            return newRequests;
        }
        return [];
    };

    const searchUsers = async (query: string): Promise<any[]> => {
        if (query.length < 2) {
            return [];
        }

        try {
            return await apiClient.get('/auth/users/search/', {
                params: { q: query }
            });
        } catch (error) {
            console.error('Failed to search users:', error);
            return [];
        }
    };


    const sendFriendRequest = async (username: string): Promise<boolean> => {
        try {
            const result = await apiClient.post('/auth/friends/send/', { username });

            globalFriendsState.sentRequests.push(result);

            console.log(`Friend request sent to ${username}`);
            return true;
        } catch (error) {
            console.error('Failed to send friend request:', error);
            return false;
        }
    };

    const respondToFriendRequest = async (requestId: number, action: 'accept' | 'reject'): Promise<boolean> => {
        try {
            await apiClient.post(`/auth/friends/respond/${requestId}/`, { action });

            const request = globalFriendsState.receivedRequests.find(req => req.id === requestId);
            if (request) {
                if (action === 'accept') {
                    globalFriendsState.friends.push({
                        id: request.requester.id,
                        username: request.requester.username,
                        nickname: request.requester.nickname,
                        status: request.requester.status
                    });
                }

                const index = globalFriendsState.receivedRequests.indexOf(request);
                if (index > -1) {
                    globalFriendsState.receivedRequests.splice(index, 1);
                }
            }

            console.log(`Friend request ${action}ed`);
            return true;
        } catch (error) {
            console.error(`Failed to ${action} friend request:`, error);
            return false;
        }
    };


    const removeFriend = async (userId: number): Promise<boolean> => {
        try {
            await apiClient.delete(`/auth/friends/remove/${userId}/`);

            const index = globalFriendsState.friends.findIndex(friend => friend.id === userId);
            if (index > -1) {
                globalFriendsState.friends.splice(index, 1);
                friendsMetrics.totalFriends = globalFriendsState.friends.length;
            }

            console.log('Friend removed');
            return true;
        } catch (error) {
            console.error('Failed to remove friend:', error);
            return false;
        }
    };

    const getOnlineFriends = (): Friend[] => {
        return globalFriendsState.friends.filter(friend =>
            friend.status.is_online && !friend.status.invisible_mode
        );
    };


    const refreshFriendsData = async (): Promise<void> => {
        console.log('Force refreshing friends data...');
        await loadFriendsData(true);
    };


    const clearFriendsData = (): void => {
        globalFriendsState.friends = [];
        globalFriendsState.sentRequests = [];
        globalFriendsState.receivedRequests = [];
        globalFriendsState.isLoading = false;
        globalFriendsState.isLoaded = false;
        globalFriendsState.lastUpdated = null;
        globalFriendsState.lastStatusUpdate = 0;
        previousReceivedCount.value = 0;

        if (statusUpdateInterval.current) {
            clearInterval(statusUpdateInterval.current);
            statusUpdateInterval.current = null;
        }

        friendsMetrics.totalFriends = 0;
        friendsMetrics.onlineFriends = 0;
    };


    const stopStatusUpdates = (): void => {
        if (statusUpdateInterval.current) {
            clearInterval(statusUpdateInterval.current);
            statusUpdateInterval.current = null;
            console.log('Stopped status updates');
        }
    };

    watch(isAuthenticated, (newValue) => {
        if (!newValue) {
            clearFriendsData();
        }
    });

    return {
        friends: computed(() => globalFriendsState.friends),
        sentRequests: computed(() => globalFriendsState.sentRequests),
        receivedRequests: computed(() => globalFriendsState.receivedRequests),
        friendRequests,
        isLoading: computed(() => globalFriendsState.isLoading),
        isLoaded: computed(() => globalFriendsState.isLoaded),
        lastUpdated: computed(() => globalFriendsState.lastUpdated),
        isStatusLoading,

        onlineFriends,
        onlineFriendsCount,

        loadFriendsData,
        updateFriendStatuses,
        refreshFriendsData,
        clearFriendsData,

        searchUsers,
        sendFriendRequest,
        respondToFriendRequest,
        removeFriend,

        getOnlineFriends,
        checkForNewRequests,

        startStatusUpdates,
        stopStatusUpdates,

        lastStatusUpdate: computed(() => globalFriendsState.lastStatusUpdate)
    };
}

export const globalFriends = useFriends();
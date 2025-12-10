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

interface GlobalFriendsState {
    friends: Friend[];
    sentRequests: FriendRequest[];
    receivedRequests: FriendRequest[];
    isLoading: boolean;
    isLoaded: boolean;
    lastUpdated: string | null;
    lastStatusUpdate: number;
}

const globalFriendsState = reactive<GlobalFriendsState>({
    friends: [],
    sentRequests: [],
    receivedRequests: [],
    isLoading: false,
    isLoaded: false,
    lastUpdated: null,
    lastStatusUpdate: 0
});

const isStatusLoading = ref(false);
const statusUpdateInterval: { current: NodeJS.Timeout | null } = { current: null };
const STATUS_UPDATE_INTERVAL = 45000;

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
                receivedRequests: batchData.requests?.received?.length || 0
            });

            globalFriendsState.friends = batchData.friends || [];
            globalFriendsState.sentRequests = batchData.requests?.sent || [];
            globalFriendsState.receivedRequests = batchData.requests?.received || [];
            globalFriendsState.lastUpdated = new Date().toISOString();
            globalFriendsState.isLoaded = true;

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

            await updateFriendStatuses();
        };

        runStatusUpdate();

        statusUpdateInterval.current = setInterval(runStatusUpdate, STATUS_UPDATE_INTERVAL);

        document.addEventListener('visibilitychange', () => {
            if (document.visibilityState === 'visible') {
                runStatusUpdate().catch(() => { });
            }
        });

        console.log(`Started status updates with ${STATUS_UPDATE_INTERVAL}ms interval`);
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

        if (statusUpdateInterval.current) {
            clearInterval(statusUpdateInterval.current);
            statusUpdateInterval.current = null;
        }
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

        startStatusUpdates,
        stopStatusUpdates,

        lastStatusUpdate: computed(() => globalFriendsState.lastStatusUpdate)
    };
}

export const globalFriends = useFriends();
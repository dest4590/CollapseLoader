import { ref, computed, reactive, watch } from 'vue';
import { userService, type Friend, type FriendRequest } from '../services/userService';

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
            friend.status.is_online
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

            const batchData = await userService.getFriendsBatch();

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
        } finally {
            globalFriendsState.isLoading = false;
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
            const batch = await userService.getFriendsBatch();
            const updatedById = new Map<number, Friend>();
            for (const f of batch.friends || []) {
                updatedById.set(f.id, f);
            }

            globalFriendsState.friends.forEach(friend => {
                const updatedFriend = updatedById.get(friend.id);
                if (!updatedFriend) return;

                const oldOnlineStatus = friend.status.is_online;
                const oldClient = friend.status.current_client;

                const oldUpdatedAt = friend.status.updated_at ? new Date(friend.status.updated_at).getTime() : null;
                const newUpdatedAt = updatedFriend.status.updated_at ? new Date(updatedFriend.status.updated_at).getTime() : null;

                const shouldUpdate =
                    oldUpdatedAt === null ||
                    newUpdatedAt === null ||
                    Number.isNaN(oldUpdatedAt) ||
                    Number.isNaN(newUpdatedAt) ||
                    newUpdatedAt >= oldUpdatedAt;

                if (!shouldUpdate) return;

                friend.status.is_online = updatedFriend.status.is_online;
                friend.status.last_seen = updatedFriend.status.last_seen;
                friend.status.current_client = updatedFriend.status.current_client;
                friend.status.updated_at = updatedFriend.status.updated_at;
                friend.status.raw_status = updatedFriend.status.raw_status;

                if (oldOnlineStatus !== updatedFriend.status.is_online ||
                    oldClient !== updatedFriend.status.current_client) {
                    hasChanges = true;
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

        runStatusUpdate().then(
            () => console.log('Status updates started successfully'),
            error => console.error('Failed to start status updates:', error)
        );

        statusUpdateInterval.current = setInterval(runStatusUpdate, STATUS_UPDATE_INTERVAL);

        document.addEventListener('visibilitychange', () => {
            if (document.visibilityState === 'visible') {
                runStatusUpdate().catch(() => {
                });
            }
        });

        console.log(`Started status updates with ${STATUS_UPDATE_INTERVAL}ms interval`);
    };

    const searchUsers = async (query: string): Promise<any[]> => {
        if (query.length < 2) {
            return [];
        }

        try {
            return await userService.searchUsers(query);
        } catch (error) {
            console.error('Failed to search users:', error);
            return [];
        }
    };


    const sendFriendRequest = async (userId: number): Promise<boolean> => {
        try {
            const result = await userService.sendFriendRequest(userId);
            globalFriendsState.sentRequests.push(result);

            console.log(`Friend request sent to ${userId}`);
            return true;
        } catch (error) {
            console.error('Failed to send friend request:', error);
            return false;
        }
    };

    const respondToFriendRequest = async (requestId: number, action: 'accept' | 'reject'): Promise<boolean> => {
        try {
            await userService.respondToFriendRequest(requestId, action);

            const request = globalFriendsState.receivedRequests.find(req => req.id === requestId);
            if (request) {
                if (action === 'accept') {
                    if (!globalFriendsState.friends.some(f => f.id === request.requester.id)) {
                        globalFriendsState.friends.push({
                            id: request.requester.id,
                            username: request.requester.username,
                            nickname: request.requester.nickname,
                            avatar_url: request.requester.avatar_url,
                            status: request.requester.status
                        });
                    }
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
            await userService.removeFriend(userId);

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
            friend.status.is_online
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

    const hydrateFriends = (data: { friends: any[]; requests: { sent: any[]; received: any[]; blocked: any[] } }) => {
        globalFriendsState.friends = (data.friends || []).map((f: any) => userService.mapFriend(f));
        globalFriendsState.sentRequests = (data.requests?.sent || []).map((r: any) => userService.mapFriendRequest(r));
        globalFriendsState.receivedRequests = (data.requests?.received || []).map((r: any) => userService.mapFriendRequest(r));
        globalFriendsState.isLoading = false;
        globalFriendsState.isLoaded = true;
        globalFriendsState.lastUpdated = new Date().toISOString();

        if (!statusUpdateInterval.current) {
            startStatusUpdates();
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

        hydrateFriends,

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

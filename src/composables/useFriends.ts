import { ref, computed, reactive } from 'vue';
import { userService, type Friend, type FriendRequest } from '../services/userService';
import { sendNativeNotification } from '../services/notificationService';

interface GlobalFriendsState {
    friends: Friend[];
    sentRequests: FriendRequest[];
    receivedRequests: FriendRequest[];
    isLoading: boolean;
    isLoaded: boolean;
    lastUpdated: string | null;
}

const globalFriendsState = reactive<GlobalFriendsState>({
    friends: [],
    sentRequests: [],
    receivedRequests: [],
    isLoading: false,
    isLoaded: false,
    lastUpdated: null
});

const lastStatusUpdate = ref<number>(0);
const isStatusLoading = ref(false);
const previousReceivedCount = ref(0);

const isAuthenticated = computed(() => !!localStorage.getItem('authToken'));

export function useFriends() {
    const onlineFriends = computed(() =>
        globalFriendsState.friends.filter(friend => friend.status.is_online)
    );

    const onlineFriendsCount = computed(() => onlineFriends.value.length);

    const friendRequests = computed(() => ({
        sent: globalFriendsState.sentRequests,
        received: globalFriendsState.receivedRequests
    }));

    const loadFriendsData = async (): Promise<void> => {
        if (!isAuthenticated.value) {
            clearFriendsData();
            return;
        }

        if (globalFriendsState.isLoading) {
            return;
        }

        globalFriendsState.isLoading = true;

        try {
            const batchData = await userService.getBatchFriendsData();
            console.log('Batch friends data response:', batchData);

            globalFriendsState.friends = batchData.friends || [];
            globalFriendsState.sentRequests = batchData.requests?.sent || [];
            globalFriendsState.receivedRequests = batchData.requests?.received || [];
            globalFriendsState.lastUpdated = new Date().toISOString();
            globalFriendsState.isLoaded = true;

            console.log('Friends data loaded successfully via batch endpoint');
        } catch (error) {
            console.error('Failed to load friends data via batch endpoint, falling back to individual calls:', error);

            try {
                const [friendsResult, requestsResult] = await Promise.all([
                    userService.getFriends(),
                    userService.getFriendRequests()
                ]);

                globalFriendsState.friends = friendsResult;
                globalFriendsState.sentRequests = requestsResult.sent;
                globalFriendsState.receivedRequests = requestsResult.received;
                globalFriendsState.lastUpdated = new Date().toISOString();
                globalFriendsState.isLoaded = true;

                console.log('Friends data loaded successfully via fallback');
            } catch (fallbackError) {
                console.error('Failed to load friends data via fallback:', fallbackError);
            }
        } finally {
            globalFriendsState.isLoading = false;
        }
    };

    const updateFriendStatuses = async () => {
        if (globalFriendsState.friends.length === 0) return;

        try {
            isStatusLoading.value = true;
            const statusesData = await userService.getFriendsStatuses();

            globalFriendsState.friends.forEach(friend => {
                const updatedStatus = statusesData.find(status => status.id === friend.id);
                if (updatedStatus) {
                    friend.status.is_online = updatedStatus.is_online;
                    friend.status.last_seen = updatedStatus.last_seen;
                    friend.status.current_client = updatedStatus.current_client ?? undefined;
                    friend.status.client_version = updatedStatus.client_version ?? undefined;
                }
            });

            lastStatusUpdate.value = Date.now();
        } catch (error) {
            console.error('Failed to update friend statuses:', error);
        } finally {
            isStatusLoading.value = false;
        }
    };

    const checkForNewRequests = (currentRequests: { sent: FriendRequest[]; received: FriendRequest[] }): FriendRequest[] => {
        if (currentRequests.received.length > previousReceivedCount.value) {
            const knownRequestIds = new Set(globalFriendsState.receivedRequests.map(req => req.id));
            return currentRequests.received.filter(req => !knownRequestIds.has(req.id));
        }
        return [];
    };

    const notifyNewFriendRequests = (newRequests: FriendRequest[]) => {
        newRequests.forEach(request => {
            const requester = request.requester;
            const displayName = requester.nickname || requester.username;

            sendNativeNotification(
                'New Friend Request',
                `${displayName} sent you a friend request`,
            );
        });
    };

    const clearFriendsData = () => {
        globalFriendsState.friends = [];
        globalFriendsState.sentRequests = [];
        globalFriendsState.receivedRequests = [];
        globalFriendsState.isLoading = false;
        globalFriendsState.isLoaded = false;
        globalFriendsState.lastUpdated = null;
        previousReceivedCount.value = 0;
    };

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
        lastStatusUpdate,
        clearFriendsData,
        checkForNewRequests,
        notifyNewFriendRequests
    };
}
import { apiClient } from './apiClient';

interface UserProfile {
    nickname?: string;
    settings_data?: any;
    favorites_data?: number[];
    accounts_data?: any[];
    last_sync_timestamp?: string;
}

interface UserInfo {
    username: string;
    email: string;
}

interface CachedUserData {
    profile: UserProfile | null;
    info: UserInfo | null;
    lastUpdated: string;
}

interface SyncStatus {
    last_sync_timestamp: string | null;
    has_cloud_data: boolean;
}

interface SyncData {
    settings_data?: any;
    favorites_data?: number[];
    accounts_data?: any[];
}

interface UserStatus {
    username: string;
    nickname?: string;
    is_online: boolean;
    invisible_mode?: boolean;
    last_seen: string | null;
    current_client?: string;
    client_version?: string;
}

interface Friend {
    id: number;
    username: string;
    nickname?: string;
    status: UserStatus;
}

interface FriendRequest {
    id: number;
    requester: Friend;
    addressee: Friend;
    status: 'pending' | 'accepted' | 'blocked';
    created_at: string;
    updated_at: string;
}

interface SearchUser {
    id: number;
    username: string;
    nickname?: string;
    friendship_status?: 'friends' | 'request_sent' | 'request_received' | 'blocked' | null;
}

interface PublicUserProfile {
    id: number;
    username: string;
    nickname?: string;
    friendship_status?: 'friends' | 'request_sent' | 'request_received' | 'blocked' | null;
    status: {
        is_online: boolean;
        last_seen: string | null;
        current_client?: string;
        client_version?: string;
    };
    member_since: string | null;
}

const CACHE_KEY = 'userData';
const CACHE_EXPIRY_HOURS = 24;

class UserService {
    private getCachedData(): CachedUserData | null {
        try {
            const cached = localStorage.getItem(CACHE_KEY);
            if (!cached) return null;

            const parsedData: CachedUserData = JSON.parse(cached);
            const now = new Date();
            const cacheTime = new Date(parsedData.lastUpdated);
            const hoursDiff = (now.getTime() - cacheTime.getTime()) / (1000 * 60 * 60);

            if (hoursDiff > CACHE_EXPIRY_HOURS) {
                localStorage.removeItem(CACHE_KEY);
                return null;
            }

            return parsedData;
        } catch (error) {
            console.error('Error reading cached user data:', error);
            localStorage.removeItem(CACHE_KEY);
            return null;
        }
    }

    private setCachedData(data: Partial<CachedUserData>): void {
        try {
            const existing = this.getCachedData() || { profile: null, info: null, lastUpdated: new Date().toISOString() };
            const updated = {
                ...existing,
                ...data,
                lastUpdated: new Date().toISOString()
            };
            localStorage.setItem(CACHE_KEY, JSON.stringify(updated));
        } catch (error) {
            console.error('Error caching user data:', error);
        }
    }

    async loadUserProfile(useCache: boolean = true): Promise<{ data: UserProfile | null; fromCache: boolean }> {
        if (useCache) {
            const cached = this.getCachedData();
            if (cached?.profile) {
                console.log('Returning cached user profile');
                return { data: cached.profile, fromCache: true };
            }
        }

        try {
            console.log('Fetching user profile from server...');
            const profile = await apiClient.get('/auth/profile/');

            this.setCachedData({ profile });
            console.log('User profile loaded and cached');
            return { data: profile, fromCache: false };
        } catch (error) {
            console.error('Failed to load user profile:', error);
            return { data: null, fromCache: false };
        }
    }

    async loadUserInfo(useCache: boolean = true): Promise<{ data: UserInfo | null; fromCache: boolean }> {
        if (useCache) {
            const cached = this.getCachedData();
            if (cached?.info) {
                console.log('Returning cached user info');
                return { data: cached.info, fromCache: true };
            }
        }

        try {
            console.log('Fetching user info from server...');
            const info = await apiClient.get('/auth/user/');

            this.setCachedData({ info });
            console.log('User info loaded and cached');
            return { data: info, fromCache: false };
        } catch (error) {
            console.error('Failed to load user info:', error);
            return { data: null, fromCache: false };
        }
    }

    async updateUserProfile(nickname: string): Promise<{ success: boolean; error?: string }> {
        try {
            const updatedProfile = await apiClient.patch('/auth/profile/', { nickname });


            this.setCachedData({ profile: updatedProfile });

            console.log('User profile updated successfully');
            return { success: true };
        } catch (error: any) {
            console.error('Failed to update user profile:', error);
            const errorMessage = error.response?.data?.error || 'Failed to update profile';
            return { success: false, error: errorMessage };
        }
    }

    async syncDataToCloud(data: SyncData): Promise<{ success: boolean; error?: string }> {
        try {
            await apiClient.post('/auth/sync/', data);
            console.log('Data synced to cloud successfully');
            return { success: true };
        } catch (error: any) {
            console.error('Failed to sync data to cloud:', error);
            const errorMessage = error.response?.data?.error || 'Failed to sync data';
            return { success: false, error: errorMessage };
        }
    }

    async loadDataFromCloud(): Promise<{ data: any | null; error?: string }> {
        try {
            const data = await apiClient.get('/auth/sync/');
            console.log('Data loaded from cloud successfully');
            return { data };
        } catch (error: any) {
            console.error('Failed to load data from cloud:', error);
            const errorMessage = error.response?.data?.error || 'Failed to load data';
            return { data: null, error: errorMessage };
        }
    }

    async getSyncStatus(): Promise<SyncStatus | null> {
        try {
            const status = await apiClient.get('/auth/sync/status/');
            return status;
        } catch (error) {
            console.error('Failed to get sync status:', error);
            return null;
        }
    }

    async getUserStatus(): Promise<UserStatus> {
        try {
            const status = await apiClient.get('/auth/status/');
            return status;
        } catch (error) {
            console.error('Failed to get user status:', error);
            throw error;
        }
    }

    async updateUserStatus(isOnline: boolean, currentClient?: string, invisibleMode?: boolean): Promise<UserStatus> {
        try {
            const payload: any = {
                is_online: isOnline,
                invisible_mode: invisibleMode || false
            };

            if (currentClient) {
                payload.current_client = currentClient;
            }

            const status = await apiClient.post('/auth/status/', payload);
            return status;
        } catch (error) {
            console.error('Failed to update user status:', error);
            throw error;
        }
    }

    async getFriends(): Promise<Friend[]> {
        try {
            const friends = await apiClient.get('/auth/friends/');
            return friends;
        } catch (error) {
            console.error('Failed to get friends:', error);
            throw error;
        }
    }

    async getFriendRequests(): Promise<{ sent: FriendRequest[]; received: FriendRequest[] }> {
        try {
            const requests = await apiClient.get('/auth/friends/requests/');
            return requests;
        } catch (error) {
            console.error('Failed to get friend requests:', error);
            throw error;
        }
    }

    async getFriendsStatuses(): Promise<UserStatus[]> {
        try {
            const statuses = await apiClient.get('/auth/friends/status/');
            return statuses;
        } catch (error) {
            console.error('Failed to get friends statuses:', error);
            throw error;
        }
    }

    async sendFriendRequest(username: string): Promise<any> {
        try {
            const response = await apiClient.post('/auth/friends/send/', { username });
            return response;
        } catch (error) {
            console.error('Failed to send friend request:', error);
            throw error;
        }
    }

    async respondToFriendRequest(requestId: number, action: 'accept' | 'reject'): Promise<any> {
        try {
            const response = await apiClient.post(`/auth/friends/respond/${requestId}/`, { action });
            return response;
        } catch (error) {
            console.error('Failed to respond to friend request:', error);
            throw error;
        }
    }

    async cancelFriendRequest(requestId: number): Promise<any> {
        try {
            const response = await apiClient.delete(`/auth/friends/cancel/${requestId}/`);
            return response;
        } catch (error) {
            console.error('Failed to cancel friend request:', error);
            throw error;
        }
    }

    async removeFriend(userId: number): Promise<any> {
        try {
            const response = await apiClient.delete(`/auth/friends/remove/${userId}/`);
            return response;
        } catch (error) {
            console.error('Failed to remove friend:', error);
            throw error;
        }
    }

    async blockUser(userId: number): Promise<any> {
        try {
            const response = await apiClient.post(`/auth/users/block/${userId}/`, {});
            return response;
        } catch (error) {
            console.error('Failed to block user:', error);
            throw error;
        }
    }

    async unblockUser(userId: number): Promise<any> {
        try {
            const response = await apiClient.delete(`/auth/users/unblock/${userId}/`);
            return response;
        } catch (error) {
            console.error('Failed to unblock user:', error);
            throw error;
        }
    }

    async getBlockedUsers(): Promise<Friend[]> {
        try {
            const blockedUsers = await apiClient.get('/auth/users/blocked/');
            return blockedUsers;
        } catch (error) {
            console.error('Failed to get blocked users:', error);
            throw error;
        }
    }

    async searchUsers(query: string): Promise<SearchUser[]> {
        try {
            const users = await apiClient.get('/auth/users/search/', {
                params: { q: query }
            });
            return users;
        } catch (error) {
            console.error('Failed to search users:', error);
            throw error;
        }
    }

    async getUserProfile(userId: number): Promise<PublicUserProfile> {
        try {
            const profile = await apiClient.get(`/auth/users/${userId}/profile/`);
            return profile;
        } catch (error) {
            console.error('Failed to get user profile:', error);
            throw error;
        }
    }

    async changePassword(currentPassword: string, newPassword: string): Promise<{ success: boolean }> {
        try {
            await apiClient.post('/auth/users/set_password/', {
                new_password: newPassword,
                current_password: currentPassword,
            });
            return { success: true };
        } catch (error) {
            console.error('Failed to change password:', error);
            throw error;
        }
    }

    async initializeUser(): Promise<any> {
        try {
            const response = await apiClient.get('/auth/init/');
            console.log('Initialize user response:', response);
            return response;
        } catch (error) {
            console.error('Failed to initialize user:', error);
            throw error;
        }
    }

    async getBatchFriendsData(): Promise<{ friends: Friend[]; requests: { sent: FriendRequest[]; received: FriendRequest[] } }> {
        try {
            const response = await apiClient.get('/auth/friends/batch/');
            return response;
        } catch (error) {
            console.error('Failed to get batch friends data:', error);
            throw error;
        }
    }

    async downloadFromCloud(): Promise<UserProfile | null> {
        try {
            const response = await apiClient.get('/auth/profile/');
            return response.data;
        } catch (error) {
            console.error('Failed to download from cloud:', error);
            throw error;
        }
    }


    async syncToCloud(data: SyncData): Promise<UserProfile> {
        try {
            const response = await apiClient.post('/auth/sync/', data);

            const cachedData = this.getCachedData();
            this.setCachedData({ profile: response.data.data, info: cachedData?.info || null });

            return response.data.data;
        } catch (error) {
            console.error('Failed to sync to cloud:', error);
            throw error;
        }
    }

    clearCache(): void {
        localStorage.removeItem(CACHE_KEY);
        console.log('User data cache cleared');
    }
}

export const userService = new UserService();
export type { UserProfile, UserInfo, SyncStatus, SyncData, UserStatus, Friend, FriendRequest, SearchUser, PublicUserProfile };
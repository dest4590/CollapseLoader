import {apiClient} from './apiClient';

interface UserProfile {
    nickname?: string;
    settings_data?: any;
    favorites_data?: number[];
    accounts_data?: any[];
    last_sync_timestamp?: string;
    role?: string;
    avatar_url?: string | null;
}

interface UserInfo {
    id: number;
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
    avatar_url?: string | null;
    social_links?: Array<{
        id: number;
        platform: string;
        url: string;
        created_at?: string;
        updated_at?: string;
    }>;
    role?: string;
}

const CACHE_KEY = 'userData';
const CACHE_EXPIRY_HOURS = 24;

class UserService {
    private getCachedData(): CachedUserData | null {
        try {
            const cached = localStorage.getItem(CACHE_KEY);
            if (!cached) return null;

            const parsedData: any = JSON.parse(cached);

            if (!parsedData || typeof parsedData !== 'object' || !parsedData.lastUpdated) {
                localStorage.removeItem(CACHE_KEY);
                return null;
            }

            const now = new Date();
            const cacheTime = new Date(parsedData.lastUpdated);

            if (isNaN(cacheTime.getTime())) {
                localStorage.removeItem(CACHE_KEY);
                return null;
            }

            const hoursDiff = (now.getTime() - cacheTime.getTime()) / (1000 * 60 * 60);

            if (hoursDiff > CACHE_EXPIRY_HOURS) {
                localStorage.removeItem(CACHE_KEY);
                return null;
            }

            return parsedData as CachedUserData;
        } catch (error) {
            console.error('Error reading cached user data:', error);
            localStorage.removeItem(CACHE_KEY);
            return null;
        }
    }

    private setCachedData(data: Partial<CachedUserData>): void {
        try {
            const existing = this.getCachedData() || {profile: null, info: null, lastUpdated: new Date().toISOString()};
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

    async updateUserProfile(nickname: string): Promise<{ success: boolean; error?: string }> {
        try {
            const updatedProfile = await apiClient.patch('/auth/profile/', {nickname});

            this.setCachedData({profile: updatedProfile});
            apiClient.invalidateProfileCaches();

            console.log('User profile updated successfully');
            return {success: true};
        } catch (error: any) {
            console.error('Failed to update user profile:', error);
            const errorMessage = error.response?.data?.error || 'Failed to update profile';
            return {success: false, error: errorMessage};
        }
    }

    async uploadAvatar(file: File): Promise<{ success: boolean; profile?: UserProfile; error?: string }> {
        try {
            const form = new FormData();
            form.append('avatar', file);

            const resp = await apiClient.post('/auth/profile/avatar/', form);

            const profile = (resp as any).profile as UserProfile;
            if (profile) {
                this.setCachedData({profile});
                apiClient.invalidateProfileCaches();
            }
            return {success: true, profile};
        } catch (error: any) {
            const errorMessage = error.response?.data?.error || 'Failed to upload avatar';
            return {success: false, error: errorMessage};
        }
    }

    async resetAvatar(): Promise<{ success: boolean; profile?: UserProfile; error?: string }> {
        try {
            const resp = await apiClient.post('/auth/profile/avatar/reset/');
            const profile = (resp as any).profile as UserProfile;
            if (profile) {
                this.setCachedData({profile});
                apiClient.invalidateProfileCaches();
            }
            return {success: true, profile};
        } catch (error: any) {
            const errorMessage = error.response?.data?.error || 'Failed to reset avatar';
            return {success: false, error: errorMessage};
        }
    }

    async getSyncStatus(): Promise<SyncStatus | null> {
        try {
            return await apiClient.get('/auth/sync/status/');
        } catch (error) {
            console.error('Failed to get sync status:', error);
            return null;
        }
    }

    async getUserStatus(): Promise<UserStatus> {
        try {
            return await apiClient.get('/auth/status/');
        } catch (error) {
            console.error('Failed to get user status:', error);
            throw error;
        }
    }

    async getFriendRequests(): Promise<{ sent: FriendRequest[]; received: FriendRequest[] }> {
        try {
            return await apiClient.get('/auth/friends/requests/');
        } catch (error) {
            console.error('Failed to get friend requests:', error);
            throw error;
        }
    }

    async sendFriendRequest(username: string): Promise<any> {
        try {
            return await apiClient.post('/auth/friends/send/', {username});
        } catch (error) {
            console.error('Failed to send friend request:', error);
            throw error;
        }
    }

    async respondToFriendRequest(requestId: number, action: 'accept' | 'reject'): Promise<any> {
        try {
            return await apiClient.post(`/auth/friends/respond/${requestId}/`, {action});
        } catch (error) {
            console.error('Failed to respond to friend request:', error);
            throw error;
        }
    }

    async cancelFriendRequest(requestId: number): Promise<any> {
        try {
            return await apiClient.delete(`/auth/friends/cancel/${requestId}/`);
        } catch (error) {
            console.error('Failed to cancel friend request:', error);
            throw error;
        }
    }

    async removeFriend(userId: number): Promise<any> {
        try {
            return await apiClient.delete(`/auth/friends/remove/${userId}/`);
        } catch (error) {
            console.error('Failed to remove friend:', error);
            throw error;
        }
    }

    async blockUser(userId: number): Promise<any> {
        try {
            return await apiClient.post(`/auth/users/block/${userId}/`, {});
        } catch (error) {
            console.error('Failed to block user:', error);
            throw error;
        }
    }

    async unblockUser(userId: number): Promise<any> {
        try {
            return await apiClient.delete(`/auth/users/unblock/${userId}/`);
        } catch (error) {
            console.error('Failed to unblock user:', error);
            throw error;
        }
    }

    async getBlockedUsers(): Promise<Friend[]> {
        try {
            return await apiClient.get('/auth/users/blocked/');
        } catch (error) {
            console.error('Failed to get blocked users:', error);
            throw error;
        }
    }

    async searchUsers(query: string): Promise<SearchUser[]> {
        try {
            return await apiClient.get('/auth/users/search/', {
                params: {q: query}
            });
        } catch (error) {
            console.error('Failed to search users:', error);
            throw error;
        }
    }

    async getUserProfile(userId: number): Promise<PublicUserProfile> {
        try {
            return await apiClient.get(`/auth/users/${userId}/profile/`);
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
            return {success: true};
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

    async getBatchFriendsData(): Promise<{
        friends: Friend[];
        requests: { sent: FriendRequest[]; received: FriendRequest[] }
    }> {
        try {
            return await apiClient.get('/auth/friends/batch/');
        } catch (error) {
            console.error('Failed to get batch friends data:', error);
            throw error;
        }
    }

    async downloadFromCloud(): Promise<UserProfile | null> {
        try {
            return await apiClient.get('/auth/profile/');
        } catch (error) {
            console.error('Failed to download from cloud:', error);
            throw error;
        }
    }

    async syncToCloud(data: SyncData): Promise<UserProfile> {
        try {
            const response = await apiClient.post('/auth/sync/', data);

            const profile = response.data || response;
            const cachedData = this.getCachedData();
            this.setCachedData({profile: profile, info: cachedData?.info || null});
            apiClient.invalidateProfileCaches();

            return profile;
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
export type {
    UserProfile,
    UserInfo,
    SyncStatus,
    SyncData,
    UserStatus,
    Friend,
    FriendRequest,
    SearchUser,
    PublicUserProfile
};
import { apiGet, apiPost, apiPatch, apiDelete } from './authClient';
import { getCurrentLanguage } from '../i18n';

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
    private getAuthHeaders() {
        const token = localStorage.getItem('authToken');
        const headers: Record<string, string> = {
            'Accept-Language': getCurrentLanguage() || 'en'
        };

        if (token) {
            headers['Authorization'] = `Token ${token}`;
        }

        return headers;
    }

    private getCachedData(): CachedUserData | null {
        try {
            const cached = localStorage.getItem(CACHE_KEY);
            if (!cached) return null;

            const data = JSON.parse(cached) as CachedUserData;

            return data;
        } catch (error) {
            console.error('Error reading cached user data:', error);
            return null;
        }
    }

    private setCachedData(profile: UserProfile | null, info: UserInfo | null) {
        try {
            const cacheData: CachedUserData = {
                profile,
                info,
                lastUpdated: new Date().toISOString()
            };
            localStorage.setItem(CACHE_KEY, JSON.stringify(cacheData));
        } catch (error) {
            console.error('Error caching user data:', error);
        }
    }

    private isCacheExpired(cachedData: CachedUserData): boolean {
        const cacheAge = new Date().getTime() - new Date(cachedData.lastUpdated).getTime();
        const maxAge = CACHE_EXPIRY_HOURS * 60 * 60 * 1000;
        return cacheAge > maxAge;
    } async loadUserProfile(): Promise<{ data: UserProfile | null; fromCache: boolean }> {
        const cachedData = this.getCachedData();

        try {
            const response = await apiGet('/auth/profile/', {
                headers: this.getAuthHeaders()
            });

            console.log('User profile API response:', response.data);
            this.setCachedData(response.data, cachedData?.info || null);

            return { data: response.data, fromCache: false };
        } catch (error) {
            console.warn('Failed to load user profile from API, using cache:', error);

            if (cachedData?.profile) {
                console.log('Using cached profile data:', cachedData.profile);
                return { data: cachedData.profile, fromCache: true };
            }

            throw error;
        }
    }

    async loadUserInfo(): Promise<{ data: UserInfo | null; fromCache: boolean }> {
        const cachedData = this.getCachedData();

        try {
            const response = await apiGet('/auth/users/me/', {
                headers: this.getAuthHeaders()
            });

            this.setCachedData(cachedData?.profile || null, response.data);

            return { data: response.data, fromCache: false };
        } catch (error) {
            console.warn('Failed to load user info from API, using cache:', error);

            if (cachedData?.info) {
                return { data: cachedData.info, fromCache: true };
            }

            throw error;
        }
    }

    async updateUserProfile(nickname: string): Promise<{ success: boolean; fromCache: boolean }> {
        try {
            await apiPatch('/auth/profile/',
                { nickname },
                { headers: this.getAuthHeaders() }
            );

            const cachedData = this.getCachedData();
            this.setCachedData({ nickname }, cachedData?.info || null);

            return { success: true, fromCache: false };
        } catch (error) {
            console.error('Failed to update user profile:', error);
            return { success: false, fromCache: false };
        }
    }

    async changePassword(currentPassword: string, newPassword: string): Promise<{ success: boolean }> {
        try {
            await apiPost('/auth/users/set_password/', {
                new_password: newPassword,
                current_password: currentPassword,
            }, {
                headers: this.getAuthHeaders()
            });

            return { success: true };
        } catch (error) {
            console.error('Failed to change password:', error);
            throw error;
        }
    }

    async getSyncStatus(): Promise<SyncStatus> {
        try {
            const response = await apiGet('/auth/sync-status/', {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error) {
            console.error('Failed to get sync status:', error);
            throw error;
        }
    }

    async syncToCloud(data: SyncData): Promise<UserProfile> {
        try {
            const response = await apiPost('/auth/sync/', data, {
                headers: this.getAuthHeaders()
            });

            const cachedData = this.getCachedData();
            this.setCachedData(response.data.data, cachedData?.info || null);

            return response.data.data;
        } catch (error) {
            console.error('Failed to sync to cloud:', error);
            throw error;
        }
    }

    async downloadFromCloud(): Promise<UserProfile | null> {
        try {
            const response = await apiGet('/auth/profile/', {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error) {
            console.error('Failed to download from cloud:', error);
            throw error;
        }
    }

    getCacheStatus() {
        const cachedData = this.getCachedData();
        return {
            hasCache: !!cachedData,
            isExpired: cachedData ? this.isCacheExpired(cachedData) : false,
            lastUpdated: cachedData?.lastUpdated || null
        };
    }

    clearCache() {
        localStorage.removeItem(CACHE_KEY);
    }

    async updateUserStatus(isOnline: boolean, currentClient?: string, clientVersion?: string, invisibleMode?: boolean): Promise<UserStatus> {
        try {
            const response = await apiPost('/auth/status/', {
                is_online: isOnline,
                current_client: currentClient,
                client_version: clientVersion,
                invisible_mode: invisibleMode
            }, {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error: any) {
            console.error('Failed to update user status:', error);

            if (error.response?.status === 401) {
                localStorage.removeItem('authToken');
                throw new Error('Authentication expired. Please log in again.');
            } else if (error.response?.status && error.response.status >= 500) {
                throw new Error('Server error. Please try again later.');
            } else if (!error.response) {
                throw new Error('Network error. Check your connection.');
            }
            throw error;
        }
    }

    async toggleInvisibleMode(enable: boolean): Promise<UserStatus> {
        try {
            const currentStatus = await this.getUserStatus();

            const response = await apiPost('/auth/status/', {
                is_online: currentStatus.is_online,
                current_client: currentStatus.current_client,
                client_version: currentStatus.client_version,
                invisible_mode: enable
            }, {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error) {
            console.error('Failed to toggle invisible mode:', error);
            throw error;
        }
    }

    async getUserStatus(): Promise<UserStatus> {
        try {
            const response = await apiGet('/auth/status/', {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error: any) {
            console.error('Failed to get user status:', error);

            if (error.response?.status === 401) {
                localStorage.removeItem('authToken');
                throw new Error('Authentication expired. Please log in again.');
            }
            throw error;
        }
    }

    async getFriendsStatuses(): Promise<{ id: number; is_online: boolean; last_seen: string | null; current_client: string | null; client_version: string | null; }[]> {
        try {
            const response = await apiGet('/auth/friends/status/', {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error) {
            console.error('Failed to get friends statuses:', error);
            throw error;
        }
    }

    async getFriends(): Promise<Friend[]> {
        try {
            const response = await apiGet('/auth/friends/', {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error: any) {
            console.error('Failed to get friends:', error);

            if (error.response?.status === 401) {
                localStorage.removeItem('authToken');
                throw new Error('Authentication expired. Please log in again.');
            }
            throw error;
        }
    }

    async sendFriendRequest(username: string): Promise<FriendRequest> {
        try {
            const response = await apiPost('/auth/friends/send/', {
                username
            }, {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error) {
            console.error('Failed to send friend request:', error);
            throw error;
        }
    }

    async getFriendRequests(): Promise<{ sent: FriendRequest[]; received: FriendRequest[] }> {
        try {
            const response = await apiGet('/auth/friends/requests/', {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error) {
            console.error('Failed to get friend requests:', error);
            throw error;
        }
    }

    async respondToFriendRequest(friendshipId: number, action: 'accept' | 'reject'): Promise<any> {
        try {
            const response = await apiPost(`/auth/friends/respond/${friendshipId}/`, {
                action
            }, {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error) {
            console.error('Failed to respond to friend request:', error);
            throw error;
        }
    }

    async cancelFriendRequest(friendshipId: number): Promise<any> {
        try {
            const response = await apiDelete(`/auth/friends/cancel/${friendshipId}/`, {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error) {
            console.error('Failed to cancel friend request:', error);
            throw error;
        }
    }

    async removeFriend(userId: number): Promise<any> {
        try {
            const response = await apiDelete(`/auth/friends/remove/${userId}/`, {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error) {
            console.error('Failed to remove friend:', error);
            throw error;
        }
    }

    async blockUser(userId: number): Promise<any> {
        try {
            const response = await apiPost(`/auth/users/block/${userId}/`, {}, {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error) {
            console.error('Failed to block user:', error);
            throw error;
        }
    }

    async unblockUser(userId: number): Promise<any> {
        try {
            const response = await apiDelete(`/auth/users/unblock/${userId}/`, {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error) {
            console.error('Failed to unblock user:', error);
            throw error;
        }
    }

    async getBlockedUsers(): Promise<Friend[]> {
        try {
            const response = await apiGet('/auth/users/blocked/', {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error) {
            console.error('Failed to get blocked users:', error);
            throw error;
        }
    }

    async searchUsers(query: string): Promise<SearchUser[]> {
        try {
            const response = await apiGet(`/auth/users/search/?q=${encodeURIComponent(query)}`, {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error) {
            console.error('Failed to search users:', error);
            throw error;
        }
    }

    async getUserProfile(userId: number): Promise<PublicUserProfile> {
        try {
            const response = await apiGet(`/auth/users/${userId}/profile/`, {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error) {
            console.error('Failed to get user profile:', error);
            throw error;
        }
    }
    async initializeUser(): Promise<any> {
        try {
            const response = await apiGet('/auth/init/', {
                headers: this.getAuthHeaders()
            });
            console.log('Initialize user response:', response.data);
            return response.data;
        } catch (error) {
            console.error('Failed to initialize user:', error);
            throw error;
        }
    }
    async getBatchFriendsData(): Promise<{ friends: Friend[]; requests: { sent: FriendRequest[]; received: FriendRequest[] } }> {
        try {
            const response = await apiGet('/auth/friends/batch/', {
                headers: this.getAuthHeaders()
            });
            return response.data;
        } catch (error) {
            console.error('Failed to get batch friends data:', error);
            throw error;
        }
    }
}

export const userService = new UserService();
export type { UserProfile, UserInfo, SyncStatus, SyncData, UserStatus, Friend, FriendRequest, SearchUser, PublicUserProfile };
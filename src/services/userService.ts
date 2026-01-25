import { apiClient } from './apiClient';

interface UserProfile {
    id: number;
    nickname: string | null;
    avatar_url: string | null;
    role: string | null;
    social_links: Array<{ platform: string; url: string }>;
    created_at?: string;
    updated_at?: string;
}

interface UserInfo {
    id: number;
    username: string;
    email: string;
    role: string;
    created_at: string;
    updated_at: string;
    last_login_at?: string;
}

interface CachedUserData {
    profile: UserProfile | null;
    info: UserInfo | null;
    lastUpdated: string;
}

interface UserStatus {
    status: 'ONLINE' | 'OFFLINE';
    client_id: number | null;
    updated_at: string;
}

interface Friend {
    id: number;
    username: string;
    nickname?: string;
    status: {
        status: 'ONLINE' | 'OFFLINE';
        client_id: number | null;
        updated_at: string;
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
        status: 'ONLINE' | 'OFFLINE';
        client_id: number | null;
        updated_at: string;
    };
    member_since: string | null;
    avatar_url?: string | null;
    social_links?: Array<{
        platform: string;
        url: string;
    }>;
    role?: string;
}

const CACHE_KEY = 'userData';
const CACHE_EXPIRY_HOURS = 24;

class UserService {
    private unwrapApiResponse<T = any>(resp: any): T {
        if (!resp) return resp;
        if (typeof resp === 'object' && 'success' in resp && 'data' in resp) {
            return resp.data as T;
        }
        return resp as T;
    }

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
            const existing = this.getCachedData() || { profile: null, info: null, lastUpdated: new Date().toISOString() };
            const updated = { ...existing, ...data, lastUpdated: new Date().toISOString() };
            localStorage.setItem(CACHE_KEY, JSON.stringify(updated));
        } catch (error) {
            console.error('Error caching user data:', error);
        }
    }

    async updateUserProfile(nickname: string): Promise<{ success: boolean; error?: string }> {
        try {
            const resp = await apiClient.patch('/users/me/profile', { nickname });
            const updatedProfile = this.unwrapApiResponse(resp) as UserProfile;
            this.setCachedData({ profile: updatedProfile });
            return { success: true };
        } catch (error: any) {
            console.error('Failed to update user profile:', error);
            const errorMessage = error.response?.data?.error || 'Failed to update profile';
            return { success: false, error: errorMessage };
        }
    }

    async uploadAvatar(file: File): Promise<{ success: boolean; profile?: UserProfile; error?: string }> {
        try {
            const form = new FormData();
            form.append('avatar', file);
            const resp = await apiClient.post('/users/me/avatar', form);
            const profile = this.unwrapApiResponse(resp) as UserProfile;
            if (profile) this.setCachedData({ profile });
            return { success: true, profile };
        } catch (error: any) {
            const errorMessage = error.response?.data?.error || 'Failed to upload avatar';
            return { success: false, error: errorMessage };
        }
    }

    async resetAvatar(): Promise<{ success: boolean; profile?: UserProfile; error?: string }> {
        try {
            const resp = await apiClient.post('/users/me/avatar/reset');
            const profile = this.unwrapApiResponse(resp) as UserProfile;
            if (profile) this.setCachedData({ profile });
            return { success: true, profile };
        } catch (error: any) {
            const errorMessage = error.response?.data?.error || 'Failed to reset avatar';
            return { success: false, error: errorMessage };
        }
    }

    async getUserStatus(): Promise<UserStatus> {
        try {
            const resp = await apiClient.get('/users/me/status');
            return this.unwrapApiResponse(resp) as UserStatus;
        } catch (error) {
            console.error('Failed to get user status:', error);
            throw error;
        }
    }

    async changePassword(currentPassword: string, newPassword: string): Promise<{ success: boolean }> {
        try {
            await apiClient.post('/api/v1/auth/users/set_password/', {
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
            const resp = await apiClient.get('/users/me');
            const data = this.unwrapApiResponse(resp) as any;
            const result = {
                profile: data.profile,
                user_info: {
                    id: data.id,
                    username: data.username,
                    email: data.email,
                    role: data.role,
                    created_at: data.created_at,
                    updated_at: data.updated_at,
                    last_login_at: data.last_login_at
                },
                status: data.status
            };
            this.setCachedData({ profile: result.profile, info: result.user_info });
            return result;
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
            return await apiClient.get('/api/v1/auth/friends/batch/');
        } catch (error) {
            console.error('Failed to get batch friends data:', error);
            throw error;
        }
    }

    async replaceSocialLinks(request: { links: Array<{ platform: string; url: string; id?: number }> }): Promise<any> {
        try {
            const resp = await apiClient.put('/api/v1/users/me/social-links', request);
            return this.unwrapApiResponse(resp);
        } catch (error) {
            console.error('Failed to replace social links:', error);
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
    UserStatus,
    Friend,
    FriendRequest,
    SearchUser,
    PublicUserProfile
};

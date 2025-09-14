import { apiGet, apiPost } from './authClient';
import { getCurrentLanguage } from '../i18n';

export interface AdminStats {
    users: {
        total: number;
        online: number;
        new_today: number;
        with_nicknames: number;
    };
    friendships: {
        total: number;
        pending_requests: number;
        blocked: number;
    };
    server: {
        uptime: string;
    };
}

export interface AdminUser {
    id: number;
    username: string;
    email: string;
    is_staff: boolean;
    is_active: boolean;
    date_joined: string;
    last_login: string | null;
    profile: {
        nickname: string | null;
        created_at: string | null;
    };
    status: {
        is_online: boolean;
        last_seen: string | null;
        current_client: string | null;
        invisible_mode: boolean;
    } | null;
}

export interface AdminUsersResponse {
    users: AdminUser[];
    pagination: {
        page: number;
        page_size: number;
        total_count: number;
        total_pages: number;
    };
}

export interface AdminStatusResponse {
    is_admin: boolean;
    username: string;
}

export interface AdminHealthResponse {
    system_health: any;
    timestamp: string;
    // only on detailed
    analytics?: any;
    recent_status_changes?: any;
}

class AdminService {
    private getHeaders() {
        const token = localStorage.getItem('authToken');
        return {
            'Authorization': `Token ${token}`,
            'Content-Type': 'application/json',
            'Accept-Language': getCurrentLanguage() || 'en'
        };
    }

    async checkAdminStatus(): Promise<AdminStatusResponse> {
        const response = await apiGet('/auth/admin/check-status/', {
            headers: this.getHeaders(),
        });

        return response?.data ?? response;
    }

    async getDashboardStats(): Promise<AdminStats> {
        const response = await apiGet('/auth/admin/dashboard/', {
            headers: this.getHeaders(),
        });

        return response?.data ?? response;
    }

    async getUsersList(page: number = 1, pageSize: number = 20, search: string = ''): Promise<AdminUsersResponse> {
        const params = new URLSearchParams({
            page: page.toString(),
            page_size: pageSize.toString(),
        });

        if (search.trim()) {
            params.append('search', search);
        }

        const response = await apiGet(`/auth/admin/users/?${params}`, {
            headers: this.getHeaders(),
        });

        return response?.data ?? response;
    }

    async getStatusSystemHealth(includeDetailed: boolean = false): Promise<AdminHealthResponse> {
        const query = includeDetailed ? '?detailed=true' : '';
        const response = await apiGet(`/auth/admin/status-health/${query}`, {
            headers: this.getHeaders(),
        });

        return response?.data ?? response;
    }

    async toggleUserStatus(userId: number): Promise<void> {
        await apiPost('/auth/admin/users/toggle-status/',
            { user_id: userId },
            { headers: this.getHeaders() }
        );
    }

    async forceLogoutUser(userId: number): Promise<void> {
        await apiPost('/auth/admin/users/force-logout/',
            { user_id: userId },
            { headers: this.getHeaders() }
        );
    }

    async banUser(userId: number, reason: string = ''): Promise<void> {
        await apiPost('/auth/admin/users/ban/',
            { user_id: userId, reason },
            { headers: this.getHeaders() }
        );
    }

    async unbanUser(userId: number): Promise<void> {
        await apiPost('/auth/admin/users/unban/',
            { user_id: userId },
            { headers: this.getHeaders() }
        );
    }

    async updateUser(userId: number, payload: { email?: string; nickname?: string; is_active?: boolean; is_staff?: boolean; role?: string }): Promise<void> {
        await apiPost('/auth/admin/users/update/',
            { user_id: userId, ...payload },
            { headers: this.getHeaders() }
        );
    }

    async deleteUser(userId: number): Promise<void> {
        await apiPost('/auth/admin/users/delete/',
            { user_id: userId },
            { headers: this.getHeaders() }
        );
    }
}

export const adminService = new AdminService();
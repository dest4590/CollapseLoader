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

class AdminService {
    private getHeaders() {
        const token = localStorage.getItem('authToken');
        return {
            'Authorization': `Token ${token}`,
            'Content-Type': 'application/json',
            'Accept-Language': getCurrentLanguage() || 'en'
        };
    } async checkAdminStatus(): Promise<AdminStatusResponse> {
        const response = await apiGet('/auth/admin/check-status/', {
            headers: this.getHeaders(),
        });

        return response.data;
    } async getDashboardStats(): Promise<AdminStats> {
        const response = await apiGet('/auth/admin/dashboard/', {
            headers: this.getHeaders(),
        });

        return response.data;
    } async getUsersList(page: number = 1, pageSize: number = 20, search: string = ''): Promise<AdminUsersResponse> {
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

        return response.data;
    } async toggleUserStatus(userId: number): Promise<void> {
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
}

export const adminService = new AdminService();
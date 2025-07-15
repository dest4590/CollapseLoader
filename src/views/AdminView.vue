<template>
    <div class="max-w-4xl mx-auto p-6">
        <div class="flex justify-between items-center mb-6 pb-5 border-b-2 border-base-300">
            <h1 class="text-2xl font-semibold text-primary-focus">
                {{ $t('admin.title') }}
            </h1>
            <div class="flex items-center gap-3">
                <span class="badge badge-error badge-sm font-bold uppercase">{{
                    $t('admin.role')
                    }}</span>
                <span class="text-base-content/70 font-medium">{{
                    username
                    }}</span>
            </div>
        </div>

        <div class="space-y-6 mb-6" v-if="stats">
            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
                <div class="card bg-base-200 shadow-md border border-base-300 stats-card" style="animation-delay: 0s">
                    <div class="card-body p-4 text-center">
                        <div class="text-3xl mb-2">👥</div>
                        <div class="text-sm text-base-content/70 mb-1">
                            {{ $t('admin.stats.totalUsers') }}
                        </div>
                        <div class="text-2xl font-bold text-primary-focus">
                            {{ stats.users.total }}
                        </div>
                    </div>
                </div>

                <div class="card bg-base-200 shadow-md border border-base-300 stats-card" style="animation-delay: 0.1s">
                    <div class="card-body p-4 text-center">
                        <div class="text-3xl mb-2 text-success">🟢</div>
                        <div class="text-sm text-base-content/70 mb-1">
                            {{ $t('admin.stats.onlineUsers') }}
                        </div>
                        <div class="text-2xl font-bold text-success">
                            {{ stats.users.online }}
                        </div>
                    </div>
                </div>

                <div class="card bg-base-200 shadow-md border border-base-300 stats-card" style="animation-delay: 0.2s">
                    <div class="card-body p-4 text-center">
                        <div class="text-3xl mb-2">📈</div>
                        <div class="text-sm text-base-content/70 mb-1">
                            {{ $t('admin.stats.newToday') }}
                        </div>
                        <div class="text-2xl font-bold text-primary-focus">
                            {{ stats.users.new_today }}
                        </div>
                    </div>
                </div>

                <div class="card bg-base-200 shadow-md border border-base-300 stats-card" style="animation-delay: 0.3s">
                    <div class="card-body p-4 text-center">
                        <div class="text-3xl mb-2">🤝</div>
                        <div class="text-sm text-base-content/70 mb-1">
                            {{ $t('admin.stats.friendships') }}
                        </div>
                        <div class="text-2xl font-bold text-primary-focus">
                            {{ stats.friendships.total }}
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div class="card bg-base-200 shadow-md border border-base-300">
            <div class="card-body p-0">
                <div class="flex justify-between items-center p-6 border-b border-base-300">
                    <h2 class="text-lg font-semibold text-primary-focus">
                        {{ $t('admin.users.title') }}
                    </h2>
                    <div class="form-control">
                        <input v-model="searchQuery" type="text" :placeholder="$t('admin.users.searchPlaceholder')"
                            class="input input-bordered input-sm w-64" @input="debouncedSearch" />
                    </div>
                </div>

                <div class="overflow-x-auto" v-if="users.length > 0">
                    <table class="table table-zebra w-full">
                        <thead>
                            <tr class="border-b border-base-300">
                                <th class="text-base-content/70 font-medium">
                                    {{ $t('admin.users.username') }}
                                </th>
                                <th class="text-base-content/70 font-medium">
                                    {{ $t('admin.users.email') }}
                                </th>
                                <th class="text-base-content/70 font-medium">
                                    {{ $t('admin.users.status') }}
                                </th>
                                <th class="text-base-content/70 font-medium">
                                    {{ $t('admin.users.lastSeen') }}
                                </th>
                                <th class="text-base-content/70 font-medium">
                                    {{ $t('admin.users.actions') }}
                                </th>
                            </tr>
                        </thead>
                        <tbody>
                            <tr v-for="user in users" :key="user.id" class="hover transition-colors" :class="{
                                'opacity-60': !user.is_active,
                                'border-l-4 border-l-error': user.is_staff,
                                'bg-success/10': user.status?.is_online,
                            }">
                                <td class="py-4">
                                    <div class="flex items-center gap-2 flex-wrap">
                                        <span class="font-medium text-base-content">{{ user.username }}</span>
                                        <span v-if="user.profile?.nickname" class="text-base-content/70 text-sm">({{
                                            user.profile.nickname }})</span>
                                        <span v-if="user.is_staff"
                                            class="badge badge-error badge-xs font-bold">ADMIN</span>
                                    </div>
                                </td>

                                <td class="text-base-content/70 break-all py-4">
                                    {{ user.email }}
                                </td>

                                <td class="py-4">
                                    <span class="badge badge-sm" :class="{
                                        'badge-success':
                                            getUserStatusClass(user) ===
                                            'online',
                                        'badge-ghost':
                                            getUserStatusClass(user) ===
                                            'offline',
                                        'badge-error':
                                            getUserStatusClass(user) ===
                                            'inactive',
                                    }">
                                        {{ getUserStatusText(user) }}
                                    </span>
                                </td>

                                <td class="text-base-content/70 text-sm py-4">
                                    {{ formatLastSeen(user) }}
                                </td>

                                <td class="py-4">
                                    <div class="flex gap-2 flex-wrap">
                                        <button v-if="!user.is_staff" @click="toggleUserStatus(user)"
                                            class="btn btn-xs transition-all duration-200" :class="user.is_active
                                                    ? 'btn-error hover:scale-105'
                                                    : 'btn-success hover:scale-105'
                                                " :disabled="actionLoading">
                                            {{
                                                user.is_active
                                                    ? $t(
                                                        'admin.actions.deactivate'
                                                    )
                                                    : $t(
                                                        'admin.actions.activate'
                                                    )
                                            }}
                                        </button>

                                        <button v-if="
                                            user.status?.is_online &&
                                            !user.is_staff
                                        " @click="forceLogout(user)"
                                            class="btn btn-warning btn-xs hover:scale-105 transition-all duration-200"
                                            :disabled="actionLoading">
                                            {{
                                                $t('admin.actions.forceLogout')
                                            }}
                                        </button>
                                    </div>
                                </td>
                            </tr>
                        </tbody>
                    </table>
                </div>

                <div v-else-if="!loading" class="text-center py-12 text-base-content/70">
                    <div class="text-5xl mb-3 opacity-30">👥</div>
                    <h3 class="text-lg font-semibold mb-2">
                        {{ $t('admin.users.noUsers') }}
                    </h3>
                    <p class="text-sm">{{ $t('admin.users.adjustSearch') }}</p>
                </div>

                <div class="flex justify-center items-center gap-4 p-6 border-t border-base-300"
                    v-if="pagination.total_pages > 1">
                    <button @click="goToPage(pagination.page - 1)" :disabled="pagination.page <= 1"
                        class="btn btn-outline btn-sm hover:scale-105 transition-all duration-200">
                        {{ $t('common.previous') }}
                    </button>

                    <span class="text-base-content/70 text-sm font-medium">
                        {{
                            $t('admin.pagination.pageOf', {
                                current: pagination.page,
                                total: pagination.total_pages,
                            })
                        }}
                    </span>

                    <button @click="goToPage(pagination.page + 1)" :disabled="pagination.page >= pagination.total_pages"
                        class="btn btn-outline btn-sm hover:scale-105 transition-all duration-200">
                        {{ $t('common.next') }}
                    </button>
                </div>
            </div>
        </div>

        <div v-if="loading" class="flex flex-col items-center justify-center py-16 text-base-content/70">
            <div class="loading loading-spinner loading-lg mb-4"></div>
            <p class="font-medium">{{ $t('common.loading') }}</p>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useI18n } from 'vue-i18n';
import {
    adminService,
    type AdminStats,
    type AdminUser,
} from '../services/adminService';
import { useToast } from '../services/toastService';

const { t } = useI18n();
const { addToast } = useToast();

const stats = ref<AdminStats | null>(null);
const users = ref<AdminUser[]>([]);
const pagination = ref({
    page: 1,
    page_size: 20,
    total_count: 0,
    total_pages: 1,
});
const searchQuery = ref('');
const loading = ref(false);
const actionLoading = ref(false);
const username = ref('');

const getUserStatusClass = (user: AdminUser) => {
    if (!user.is_active) return 'inactive';
    if (user.status?.is_online) return 'online';
    return 'offline';
};

const getUserStatusText = (user: AdminUser) => {
    if (!user.is_active) return t('admin.status.inactive');
    if (user.status?.is_online) return t('admin.status.online');
    return t('admin.status.offline');
};

const formatLastSeen = (user: AdminUser) => {
    if (!user.is_active) return t('admin.status.inactive');
    if (user.status?.is_online) return t('admin.status.online');

    if (user.status?.last_seen) {
        const date = new Date(user.status.last_seen);
        return date.toLocaleString();
    }

    return t('admin.status.never');
};

const loadDashboardStats = async () => {
    try {
        stats.value = await adminService.getDashboardStats();
    } catch (error) {
        console.error('Failed to load dashboard stats:', error);
        addToast(t('admin.errors.statsLoadFailed'), 'error');
    }
};

const loadUsers = async (page: number = 1) => {
    loading.value = true;
    try {
        const response = await adminService.getUsersList(
            page,
            pagination.value.page_size,
            searchQuery.value
        );
        users.value = response.users;
        pagination.value = response.pagination;
    } catch (error) {
        console.error('Failed to load users:', error);
        addToast(t('admin.errors.usersLoadFailed'), 'error');
    } finally {
        loading.value = false;
    }
};

const toggleUserStatus = async (user: AdminUser) => {
    actionLoading.value = true;
    try {
        await adminService.toggleUserStatus(user.id);
        addToast(
            t('admin.success.userStatusChanged', { username: user.username }),
            'success'
        );
        await loadUsers(pagination.value.page);
    } catch (error: any) {
        console.error('Failed to toggle user status:', error);
        addToast(error.message || t('admin.errors.actionFailed'), 'error');
    } finally {
        actionLoading.value = false;
    }
};

const forceLogout = async (user: AdminUser) => {
    actionLoading.value = true;
    try {
        await adminService.forceLogoutUser(user.id);
        addToast(
            t('admin.success.userLoggedOut', { username: user.username }),
            'success'
        );
        await loadUsers(pagination.value.page);
    } catch (error: any) {
        console.error('Failed to force logout:', error);
        addToast(error.message || t('admin.errors.actionFailed'), 'error');
    } finally {
        actionLoading.value = false;
    }
};

const goToPage = (page: number) => {
    if (page >= 1 && page <= pagination.value.total_pages) {
        loadUsers(page);
    }
};

let searchTimeout: ReturnType<typeof setTimeout>;

const debouncedSearch = () => {
    clearTimeout(searchTimeout);
    searchTimeout = setTimeout(() => {
        loadUsers(1);
    }, 500);
};

const checkAdminAccess = async () => {
    try {
        const adminStatus = await adminService.checkAdminStatus();
        if (!adminStatus.is_admin) {
            addToast(t('admin.errors.accessDenied'), 'error');
            return false;
        }
        username.value = adminStatus.username;
        return true;
    } catch (error) {
        console.error('Failed to check admin access:', error);
        addToast(t('admin.errors.accessCheckFailed'), 'error');
        return false;
    }
};

onMounted(async () => {
    const hasAccess = await checkAdminAccess();
    if (hasAccess) {
        await Promise.all([loadDashboardStats(), loadUsers()]);
    }
});
</script>

<style scoped>
.slide-up {
    animation: slideUp 0.5s ease-out forwards;
}

@keyframes slideUp {
    0% {
        opacity: 0;
        transform: translateY(20px);
    }

    100% {
        opacity: 1;
        transform: translateY(0);
    }
}

.stats-card {
    opacity: 0;
    transform: translateY(15px);
    animation: fadeInUp 0.4s ease-out forwards;
}

@keyframes fadeInUp {
    from {
        opacity: 0;
        transform: translateY(15px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.btn:hover {
    transform: scale(1.05);
}

.btn:active {
    transform: scale(0.98);
}

.table tbody tr:hover {
    background-color: hsl(var(--b2));
}

.transition-colors {
    transition: background-color 0.2s ease;
}
</style>

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

                <button class="btn btn-xs btn-outline ml-3" :class="{ 'loading': healthLoading }" @click="openHealth()"
                    title="Show status system health">
                    {{ $t('admin.health.refresh') }}
                </button>

                <label class="ml-2 flex items-center gap-2 text-sm">
                    <input type="checkbox" v-model="includeDetailed" class="checkbox checkbox-sm" />
                    <span class="text-base-content/60">{{ $t('admin.health.detailed') }}</span>
                </label>
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

        <div v-if="showHealth && healthData" class="card bg-base-200 shadow-md border border-base-300 mt-6">
            <div class="card-body p-4">
                <div class="flex justify-between items-start">
                    <div>
                        <h3 class="text-lg font-semibold">{{ $t('admin.health.title') }}</h3>
                        <p class="text-sm text-base-content/70">{{ healthData.timestamp }}</p>
                    </div>

                    <div class="text-right">
                        <span
                            :class="healthData.system_health?.cache_health?.status === 'healthy' ? 'badge badge-success' : 'badge badge-warning'">
                            {{ healthData.system_health?.cache_health?.status || 'unknown' }}
                        </span>
                        <button class="btn btn-ghost btn-xs ml-2" @click="closeHealth()">{{ $t('common.close')
                        }}</button>
                    </div>
                </div>

                <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mt-4">
                    <div class="p-3 bg-base-100 rounded">
                        <div class="text-sm text-base-content/70">Online users</div>
                        <div class="text-2xl font-bold">{{ healthData.system_health?.cache_health?.online_users ?? '-'
                        }}</div>
                        <div class="text-xs text-base-content/60 mt-1">{{ $t('admin.health.responseTime') }} {{
                            healthData.system_health?.cache_health?.response_time_ms ?? '-' }}</div>
                    </div>

                    <div class="p-3 bg-base-100 rounded">
                        <div class="text-sm text-base-content/70">Cache coverage</div>
                        <div class="text-2xl font-bold">{{ formatPercent(healthData.system_health?.cache_coverage) }}
                        </div>
                        <div class="text-xs text-base-content/60 mt-1">Hit rate: {{
                            formatPercent(healthData.system_health?.cache_hit_rate) }}</div>
                    </div>

                    <div class="p-3 bg-base-100 rounded">
                        <div class="text-sm text-base-content/70">Totals</div>
                        <div class="text-2xl font-bold">{{ healthData.system_health?.total_users_with_status ?? '-' }}
                        </div>
                        <div class="text-xs text-base-content/60 mt-1">Cached: {{ healthData.system_health?.cached_users
                            ?? '-' }}</div>
                    </div>
                </div>

                <div v-if="healthData.system_health?.cache_health?.connection_pool"
                    class="mt-4 p-3 bg-base-100 rounded">
                    <div class="text-sm text-base-content/70">Redis</div>
                    <div class="text-sm">{{ healthData.system_health.cache_health.connection_pool.host }}:{{
                        healthData.system_health.cache_health.connection_pool.port }} (db {{
                            healthData.system_health.cache_health.connection_pool.db }})</div>
                </div>

                <div v-if="healthData.system_health?.operations_per_metric" class="mt-4">
                    <div class="text-sm text-base-content/70 mb-2">Key metrics</div>
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-2">
                        <div v-for="key in ['get_status', 'set_status', 'bulk_get', 'bulk_set', 'cache_hits', 'cache_misses']"
                            :key="key" class="p-2 bg-base-100 rounded text-sm">
                            <div class="text-xs text-base-content/60">{{ key }}</div>
                            <div class="font-medium">{{ healthData.system_health.operations_per_metric[key] ?? '-' }}
                            </div>
                        </div>
                    </div>
                </div>

                <div v-if="healthData.recent_status_changes?.length" class="mt-4">
                    <div class="text-sm text-base-content/70 mb-2">Recent status changes (latest)</div>
                    <ul class="max-h-40 overflow-y-auto text-sm space-y-1">
                        <li v-for="(c, idx) in healthData.recent_status_changes.slice(0, 12)" :key="idx"
                            class="p-2 bg-base-100 rounded flex justify-between">
                            <div>
                                <span class="font-medium">#{{ c.user_id }}</span>
                                <span class="text-base-content/70 ml-2">{{ c.action }}</span>
                                <div class="text-xs text-base-content/60">{{ c.timestamp }}</div>
                            </div>
                            <div class="text-xs text-right">
                                <div>{{ c.status?.current_client ?? '-' }}</div>
                                <div class="text-base-content/60">{{ c.status?.last_seen ?? '' }}</div>
                            </div>
                        </li>
                    </ul>
                </div>

                <div v-if="healthData.analytics" class="mt-4">
                    <div class="text-sm text-base-content/70 mb-2">Analytics (last hour)</div>
                    <div class="grid grid-cols-2 md:grid-cols-4 gap-2 text-sm">
                        <div class="p-2 bg-base-100 rounded">
                            <div class="text-xs text-base-content/60">Online users</div>
                            <div class="font-medium">{{ healthData.analytics.last_hour.current_online_users ?? '-' }}
                            </div>
                        </div>
                        <div class="p-2 bg-base-100 rounded">
                            <div class="text-xs text-base-content/60">Unique active</div>
                            <div class="font-medium">{{ healthData.analytics.last_hour.unique_active_users ?? '-' }}
                            </div>
                        </div>
                        <div class="p-2 bg-base-100 rounded">
                            <div class="text-xs text-base-content/60">Online events</div>
                            <div class="font-medium">{{ healthData.analytics.last_hour.online_events ?? '-' }}</div>
                        </div>
                        <div class="p-2 bg-base-100 rounded">
                            <div class="text-xs text-base-content/60">Avg changes/user</div>
                            <div class="font-medium">{{
                                formatNumber(healthData.analytics.last_hour.avg_changes_per_user) }}</div>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div class="card bg-base-200 shadow-md border border-base-300 mt-6">
            <div class="card-body p-0">
                <div class="flex justify-between items-center p-6 border-b border-base-300">
                    <h2 class="text-lg font-semibold text-primary-focus">
                        {{ $t('admin.users.title') }}
                    </h2>
                    <div class="form-control">
                        <input v-model="searchQuery" type="text" :placeholder="$t('admin.users.searchPlaceholder')"
                            class="input input-bordered input-sm w-64" @input="debouncedSearch" />
                    </div>
                    <div class="form-control ml-4">
                        <select class="select select-sm" v-model.number="pagination.page_size"
                            @change="onPageSizeChange">
                            <option :value="20">20</option>
                            <option :value="50">50</option>
                            <option :value="100">100</option>
                            <option :value="200">200</option>
                            <option :value="500">500</option>
                        </select>
                    </div>
                    <div class="form-control ml-4 flex items-center gap-2">
                        <select class="select select-sm" v-model="sortKey"
                            @change="() => { pagination.page = 1; loadUsers(1); }">
                            <option value="status">{{ $t('admin.users.status') }}</option>
                            <option value="last_seen">{{ $t('admin.users.lastSeen') }}</option>
                            <option value="username">{{ $t('admin.users.username') }}</option>
                        </select>
                        <select class="select select-sm" v-model="sortOrder"
                            @change="() => { pagination.page = 1; loadUsers(1); }">
                            <option value="desc">Desc</option>
                            <option value="asc">Asc</option>
                        </select>
                    </div>
                </div>

                <div class="overflow-x-auto" v-if="users.length > 0">
                    <table class="table table-zebra w-full">
                        <thead>
                            <tr class="border-b border-base-300">
                                <th class="text-base-content/70 font-medium">
                                    <button class="flex items-center gap-2" @click="setSort('username')">
                                        <span>{{ $t('admin.users.username') }}</span>
                                        <span class="text-xs text-base-content/50">
                                            <span v-if="sortKey === 'username'">
                                                {{ sortOrder === 'asc' ? '▲' : '▼' }}
                                            </span>
                                        </span>
                                    </button>
                                </th>
                                <th class="text-base-content/70 font-medium">
                                    {{ $t('admin.users.email') }}
                                </th>
                                <th class="text-base-content/70 font-medium">
                                    <button class="flex items-center gap-2" @click="setSort('status')">
                                        <span>{{ $t('admin.users.status') }}</span>
                                        <span class="text-xs text-base-content/50">
                                            <span v-if="sortKey === 'status'">{{ sortOrder === 'asc' ? '▲' : '▼'
                                                }}</span>
                                        </span>
                                    </button>
                                </th>
                                <th class="text-base-content/70 font-medium">
                                    <button class="flex items-center gap-2" @click="setSort('last_seen')">
                                        <span>{{ $t('admin.users.lastSeen') }}</span>
                                        <span class="text-xs text-base-content/50">
                                            <span v-if="sortKey === 'last_seen'">{{ sortOrder === 'asc' ? '▲' : '▼'
                                                }}</span>
                                        </span>
                                    </button>
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
                            }">
                                <td class="py-2">
                                    <div class="flex items-center gap-2 flex-wrap">
                                        <span class="font-medium text-base-content">{{ user.username }}</span>
                                        <span v-if="user.profile?.nickname" class="text-base-content/70 text-sm">({{
                                            user.profile.nickname }})</span>
                                        <span v-if="user.is_staff"
                                            class="badge badge-error badge-xs font-bold">ADMIN</span>
                                        <span v-else-if="user.profile?.role && user.profile.role !== 'user'"
                                            class="badge badge-ghost badge-xs">{{ user.profile.role }}</span>
                                    </div>
                                </td>

                                <td class="text-base-content/70 break-all py-2">
                                    {{ user.email }}
                                </td>

                                <td class="py-2">
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

                                <td class="text-base-content/70 text-sm py-2">
                                    {{ formatLastSeen(user) }}
                                </td>

                                <td class="py-0 align-middle">
                                    <div class="flex items-center h-12">
                                        <div class="dropdown dropdown-end">
                                            <label tabindex="0"
                                                class="btn btn-ghost btn-sm rounded-md px-2 py-1 flex items-center gap-2"
                                                :class="{ 'opacity-50': actionLoading }">
                                                <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" fill="none"
                                                    viewBox="0 0 24 24" stroke="currentColor">
                                                    <path stroke-linecap="round" stroke-linejoin="round"
                                                        stroke-width="2" d="M12 6v.01M12 12v.01M12 18v.01" />
                                                </svg>
                                            </label>
                                            <ul tabindex="0"
                                                class="dropdown-content menu p-2 shadow bg-base-100 rounded-box w-44 text-sm">
                                                <li>
                                                    <button class="w-full text-left" @click="openEditUser(user)"
                                                        :disabled="actionLoading">
                                                        {{ $t('admin.actions.edit') }}
                                                    </button>
                                                </li>

                                                <li>
                                                    <button class="w-full text-left" @click="toggleUserStatus(user)"
                                                        :disabled="actionLoading">
                                                        {{ user.is_active ? $t('admin.actions.deactivate') :
                                                            $t('admin.actions.activate') }}
                                                    </button>
                                                </li>

                                                <li v-if="user.status?.is_online && !user.is_staff">
                                                    <button class="w-full text-left" @click="forceLogout(user)"
                                                        :disabled="actionLoading">
                                                        {{ $t('admin.actions.forceLogout') }}
                                                    </button>
                                                </li>

                                                <li v-if="!user.is_staff">
                                                    <button class="w-full text-left" @click="banUser(user)"
                                                        :disabled="actionLoading">
                                                        {{ $t('admin.actions.ban') }}
                                                    </button>
                                                </li>

                                                <li v-if="user.is_staff">
                                                    <button class="w-full text-left" @click="unbanUser(user)"
                                                        :disabled="actionLoading">
                                                        {{ $t('admin.actions.unban') }}
                                                    </button>
                                                </li>

                                                <li>
                                                    <button class="w-full text-left text-error"
                                                        @click="deleteUser(user)" :disabled="actionLoading">
                                                        {{ $t('admin.actions.delete') }}
                                                    </button>
                                                </li>
                                            </ul>
                                        </div>
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
import { formatDate } from '../utils/utils';
import {
    adminService,
    type AdminStats,
    type AdminUser,
    type AdminHealthResponse,
} from '../services/adminService';
import { useToast } from '../services/toastService';
import { useModal } from '../services/modalService';
import UserEditModal from '../components/admin/UserEditModal.vue';
import ConfirmModal from '../components/common/ConfirmModal.vue';

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

const sortKey = ref<'status' | 'last_seen' | 'username'>('status');
const sortOrder = ref<'asc' | 'desc'>('desc');

const setSort = (key: typeof sortKey.value) => {
    if (sortKey.value === key) {
        sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc';
    } else {
        sortKey.value = key;
        sortOrder.value = key === 'username' ? 'asc' : 'desc';
    }
    
    pagination.value.page = 1;
    void loadUsers(1);
};

const statusWeight = (user: AdminUser) => {
    if (!user.is_active) return 0;
    if (user.status?.is_online) return 3;
    return 2;
};

const parseLastSeen = (user: AdminUser) => {
    const val = user.status?.last_seen;
    if (!val) return 0;
    const t = Date.parse(String(val));
    return Number.isNaN(t) ? 0 : t;
};

const sortUsers = () => {
    users.value.sort((a, b) => {
        if (sortKey.value === 'status') {
            const wa = statusWeight(a);
            const wb = statusWeight(b);
            if (wa !== wb) return sortOrder.value === 'desc' ? wb - wa : wa - wb;
            const la = parseLastSeen(a);
            const lb = parseLastSeen(b);
            if (la !== lb) return sortOrder.value === 'desc' ? lb - la : la - lb;
            return a.username.localeCompare(b.username);
        }

        if (sortKey.value === 'last_seen') {
            const la = parseLastSeen(a);
            const lb = parseLastSeen(b);
            if (la !== lb) return sortOrder.value === 'desc' ? lb - la : la - lb;
            return a.username.localeCompare(b.username);
        }

        return sortOrder.value === 'desc'
            ? b.username.localeCompare(a.username)
            : a.username.localeCompare(b.username);
    });
};

const healthData = ref<AdminHealthResponse | null>(null);
const healthLoading = ref(false);
const includeDetailed = ref(false);
const showHealth = ref(false);

const { showModal, hideModal } = useModal();

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
        try {
            return formatDate(user.status.last_seen);
        } catch (e) {
            console.error('Failed to format last_seen for admin view:', e);
            return String(user.status.last_seen);
        }
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

const computeOrdering = () => {
    if (sortKey.value === 'status') return 'status';
    if (sortKey.value === 'last_seen') return (sortOrder.value === 'desc' ? '-last_seen' : 'last_seen');
    
    return sortOrder.value === 'desc' ? '-username' : 'username';
};

const loadUsers = async (page: number = 1) => {
    loading.value = true;
    try {
        const ordering = computeOrdering();
        const response = await adminService.getUsersList(
            page,
            pagination.value.page_size,
            searchQuery.value,
            ordering
        );
        users.value = response.users;
        pagination.value = response.pagination;
        
        if (!['status', 'last_seen', 'username'].includes(computeOrdering().replace('-', ''))) {
            sortUsers();
        }
    } catch (error) {
        console.error('Failed to load users:', error);
        addToast(t('admin.errors.usersLoadFailed'), 'error');
    } finally {
        loading.value = false;
    }
};

const onPageSizeChange = () => {
    pagination.value.page = 1;
    loadUsers(1);
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

const banUser = async (user: AdminUser) => {
    const onConfirm = async () => {
        hideModal(`ban-${user.id}`);
        actionLoading.value = true;
        try {
            await adminService.banUser(user.id);
            addToast(t('admin.success.userBanned', { username: user.username }), 'success');
            await loadUsers(pagination.value.page);
        } catch (err: any) {
            console.error('Failed to ban user:', err);
            addToast(err.message || t('admin.errors.actionFailed'), 'error');
        } finally {
            actionLoading.value = false;
        }
    };

    showModal(`ban-${user.id}`, ConfirmModal, { title: 'Ban user', message: `Ban ${user.username}?` }, {}, { confirm: onConfirm, cancel: () => hideModal(`ban-${user.id}`) });
};

const unbanUser = async (user: AdminUser) => {
    actionLoading.value = true;
    try {
        await adminService.unbanUser(user.id);
        addToast(t('admin.success.userUnbanned', { username: user.username }), 'success');
        await loadUsers(pagination.value.page);
    } catch (err: any) {
        console.error('Failed to unban user:', err);
        addToast(err.message || t('admin.errors.actionFailed'), 'error');
    } finally {
        actionLoading.value = false;
    }
};



const openEditUser = (user: AdminUser) => {
    const id = `edit-${user.id}`;
    showModal(
        id,
        UserEditModal,
        {
            title: `Edit ${user.username}`,
            email: user.email,
            nickname: user.profile?.nickname || '',
            is_active: user.is_active,
            is_staff: user.is_staff,
            role: (user.profile && (user.profile as any).role) || 'user',
        },
        {},
        {
            save: async (payload: { email: string; nickname: string; role?: string; is_active?: boolean; is_staff?: boolean }) => {
                hideModal(id);
                actionLoading.value = true;
                try {
                    await adminService.updateUser(user.id, payload as any);
                    addToast(t('admin.success.userUpdated', { username: user.username }), 'success');
                    await loadUsers(pagination.value.page);
                } catch (err: any) {
                    console.error('Failed to update user:', err);
                    addToast(err.message || t('admin.errors.actionFailed'), 'error');
                } finally {
                    actionLoading.value = false;
                }
            },
            cancel: () => hideModal(id),
        }
    );
};

const deleteUser = async (user: AdminUser) => {
    const id = `delete-${user.id}`;
    showModal(id, ConfirmModal, { title: 'Delete user', message: `Permanently delete ${user.username}?` }, {}, {
        confirm: async () => {
            hideModal(id);
            actionLoading.value = true;
            try {
                await adminService.deleteUser(user.id);
                addToast(t('admin.success.userDeleted', { username: user.username }), 'success');
                await loadUsers(pagination.value.page);
            } catch (err: any) {
                console.error('Failed to delete user:', err);
                addToast(err.message || t('admin.errors.actionFailed'), 'error');
            } finally {
                actionLoading.value = false;
            }
        },
        cancel: () => hideModal(id),
    });
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

const formatPercent = (v: number | undefined | null) => {
    if (v === undefined || v === null) return '-';
    return `${Math.round((v as number) * 1000) / 10}%`;
};

const formatNumber = (v: number | undefined | null) => {
    if (v === undefined || v === null) return '-';
    return Number.isFinite(v) ? Math.round((v as number) * 100) / 100 : v;
};

const loadStatusHealth = async () => {
    healthLoading.value = true;
    try {
        const resp = await adminService.getStatusSystemHealth(includeDetailed.value);
        healthData.value = resp as unknown as AdminHealthResponse;
    } catch (error) {
        console.error('Failed to load status health:', error);
        addToast(t('admin.errors.statsLoadFailed'), 'error');
    } finally {
        healthLoading.value = false;
    }
};

const openHealth = async () => {
    showHealth.value = true;
    await loadStatusHealth();
};

const closeHealth = () => {
    showHealth.value = false;
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

table.table tr td,
table.table tr th {
    vertical-align: middle;
}

.table tbody tr {
    height: 48px;
}

.table tbody tr td {
    padding-top: 0.25rem;
    padding-bottom: 0.25rem;
}

.table .badge-xs {
    transform: translateY(-1px);
}

.dropdown-content.menu li button {
    padding: 0.4rem 0.5rem;
}

.table tbody tr td .font-medium {
    line-height: 1.1;
}

.table tbody tr td span.text-sm {
    margin-left: 6px;
}

.btn.btn-ghost.btn-sm {
    height: 32px;
    min-width: 36px;
}
</style>

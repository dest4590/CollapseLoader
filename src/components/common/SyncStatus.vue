<template>
    <div class="sync-status-widget" :class="{ offline: !syncState.isOnline }">
        <div class="flex items-center gap-2 text-xs">
            <div class="flex items-center gap-1">
                <div class="w-2 h-2 rounded-full mr-2" :class="{
                    'bg-success animate-pulse':
                        syncState.isOnline && !syncState.isSyncing,
                    'bg-error': !syncState.isOnline,
                    'bg-warning animate-spin': syncState.isSyncing,
                }"></div>
                <span class="font-medium">
                    {{
                        syncState.isOnline
                            ? syncState.isSyncing
                                ? t('settings.syncing')
                                : t('time.online')
                            : t('time.offline')
                    }}
                </span>
            </div>

            <div v-if="syncState.lastSyncTime" class="text-base-content/60">
                {{ t('sync.last_sync') }}:
                {{ formatSyncTime(syncState.lastSyncTime) }}
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue';
import { syncService, type SyncServiceState } from '../../services/syncService';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const syncState = ref<SyncServiceState>(syncService.getState());
let unsubscribe: (() => void) | null = null;

const formatSyncTime = (timestamp: string): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));

    if (diffMins < 1) return t('userProfile.just_now');
    if (diffMins < 60) return t('sync.minutes_ago', { count: diffMins });

    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return t('sync.hours_ago', { count: diffHours });

    const diffDays = Math.floor(diffHours / 24);
    return t('sync.days_ago', { count: diffDays });
};

onMounted(async () => {
    unsubscribe = syncService.subscribe((state) => {
        syncState.value = state;
    });

    await syncService.initializeSyncStatus();
});

onUnmounted(() => {
    if (unsubscribe) {
        unsubscribe();
    }
});
</script>

<style scoped>
.sync-status-widget {
    background-color: oklch(var(--color-base-200));
    border: 1px solid oklch(var(--color-base-300));
    transition: all 0.18s ease;
}

.sync-status-widget.offline {
    border-color: oklch(var(--color-error));
    background-color: oklch(var(--color-error) / 0.06);
}

.sync-status-widget:hover {
    background-color: oklch(var(--color-base-200));
    border-color: oklch(var(--color-base-300));
}

.status-indicator .spinner {
    width: 28px;
    height: 28px;
    border-radius: 9999px;
    border: 3px solid var(--tw-ring-color, rgba(0, 0, 0, 0.08));
    border-top-color: var(--pf, rgb(59 130 246));
    animation: spin 1s linear infinite;
}

.status-dot {
    width: 12px;
    height: 12px;
    border-radius: 9999px;
}

.status-dot.online {
    background: oklch(var(--color-success));
}

.status-dot.offline-dot {
    background: oklch(var(--color-error));
}

.progress-bar {
    height: 6px;
    background: rgba(0, 0, 0, 0.06);
    border-radius: 9999px;
    overflow: hidden;
}

.progress-indeterminate {
    width: 30%;
    height: 100%;
    background: linear-gradient(90deg, rgba(59, 130, 246, 0.95), rgba(99, 102, 241, 0.9));
    animation: progressIndeterminate 1.1s infinite linear;
}

@keyframes spin {
    to {
        transform: rotate(360deg);
    }
}

@keyframes progressIndeterminate {
    0% {
        transform: translateX(-120%);
    }

    100% {
        transform: translateX(240%);
    }
}
</style>

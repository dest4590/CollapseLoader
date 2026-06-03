<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { Bell, Trash2, X, CheckCircle2, AlertTriangle, Info, AlertCircle, Check } from "@lucide/vue";
import { useNotificationHistory } from "@shared/composables/useNotificationHistory";
import type { ToastType } from "@shared/types/toast";

defineProps<{ show: boolean }>();

const emit = defineEmits<{
    close: [];
}>();

const { t } = useI18n();
const { history, clearHistory, markAllAsRead, markAsRead } = useNotificationHistory();

const formatTime = (ts: number): string => {
    const date = new Date(ts);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMin = Math.floor(diffMs / 60000);
    const diffHour = Math.floor(diffMin / 60);
    const diffDay = Math.floor(diffHour / 24);

    if (diffMin < 1) return t("history.just_now");
    if (diffMin < 60) return t("history.minutes_ago", { n: diffMin });
    if (diffHour < 24) return t("history.hours_ago", { n: diffHour });
    if (diffDay < 7) return t("history.days_ago", { n: diffDay });

    return date.toLocaleDateString();
};

const groupedEntries = computed(() => {
    const groups: {
        label: string;
        items: (any & { globalIndex: number })[];
    }[] = [];
    const today: any[] = [];
    const yesterday: any[] = [];
    const older: any[] = [];

    const now = new Date();
    const todayStart = new Date(
        now.getFullYear(),
        now.getMonth(),
        now.getDate()
    );
    const yesterdayStart = new Date(todayStart.getTime() - 86400000);

    for (const entry of history.value) {
        const d = new Date(entry.timestamp);
        if (d >= todayStart) today.push(entry);
        else if (d >= yesterdayStart) yesterday.push(entry);
        else older.push(entry);
    }

    let globalIndex = 0;
    const mapWithIndex = (items: any[]) => {
        return items.map((item) => ({ ...item, globalIndex: globalIndex++ }));
    };

    if (today.length)
        groups.push({ label: t("history.today"), items: mapWithIndex(today) });
    if (yesterday.length)
        groups.push({
            label: t("history.yesterday"),
            items: mapWithIndex(yesterday),
        });
    if (older.length)
        groups.push({
            label: t("history.earlier"),
            items: mapWithIndex(older),
        });

    return groups;
});

const getIconForType = (type: ToastType) => {
    switch (type) {
        case "success": return CheckCircle2;
        case "error": return AlertTriangle;
        case "warning": return AlertCircle;
        case "info":
        default:
            return Info;
    }
};

const getColorForType = (type: ToastType) => {
    switch (type) {
        case "success": return "text-success";
        case "error": return "text-error";
        case "warning": return "text-warning";
        case "info":
        default:
            return "text-info";
    }
};

const getBgForType = (type: ToastType) => {
    switch (type) {
        case "success": return "bg-success/10";
        case "error": return "bg-error/10";
        case "warning": return "bg-warning/10";
        case "info":
        default:
            return "bg-info/10";
    }
};
</script>

<template>
    <Transition name="history-panel">
        <div v-if="show" class="history-panel">
            <div class="history-header">
                <div class="flex items-center gap-2">
                    <Bell class="w-4 h-4 text-primary" />
                    <span class="font-semibold text-sm">{{
                        t("notifications.title")
                    }}</span>
                </div>
                <div class="flex items-center gap-1">
                    <button
                        v-if="history.length > 0"
                        @click="markAllAsRead"
                        class="btn btn-ghost btn-xs text-base-content/50 hover:text-success gap-1"
                        :title="t('notifications.mark_read')"
                    >
                        <Check class="w-3 h-3" />
                    </button>
                    <button
                        v-if="history.length > 0"
                        @click="clearHistory"
                        class="btn btn-ghost btn-xs text-base-content/50 hover:text-error gap-1"
                        :title="t('notifications.clear_all')"
                    >
                        <Trash2 class="w-3 h-3" />
                    </button>
                    <button
                        @click="emit('close')"
                        class="btn btn-ghost btn-xs btn-circle"
                    >
                        <X class="w-4 h-4" />
                    </button>
                </div>
            </div>

            <div class="history-body">
                <div v-if="history.length === 0" class="history-empty">
                    <Bell class="w-8 h-8 opacity-20 mb-2" />
                    <p class="text-sm text-base-content/40">
                        {{ t("notifications.empty") }}
                    </p>
                </div>

                <div v-else class="history-list">
                    <div
                        v-for="group in groupedEntries"
                        :key="group.label"
                        class="history-group"
                    >
                        <div class="history-group-label">{{ group.label }}</div>
                        <div
                            v-for="entry in group.items"
                            :key="entry.id"
                            class="history-entry"
                            :class="{ 'opacity-60': entry.isRead }"
                            :style="{
                                animationDelay: `${entry.globalIndex * 35}ms`,
                            }"
                            @click="markAsRead(entry.id)"
                        >
                            <div class="history-entry-icon" :class="getBgForType(entry.type)">
                                <component :is="getIconForType(entry.type)" class="w-3 h-3" :class="getColorForType(entry.type)" />
                            </div>
                            <div class="history-entry-info">
                                <div class="history-entry-name">
                                    {{ entry.message }}
                                </div>
                            </div>
                            <div class="history-entry-time">
                                <div v-if="!entry.isRead" class="w-2 h-2 rounded-full bg-primary mb-1 ml-auto"></div>
                                {{ formatTime(entry.timestamp) }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </Transition>
</template>

<style scoped>
.history-panel {
    position: absolute;
    top: calc(100% + 8px);
    right: 0;
    width: 320px;
    max-height: 420px;
    background: hsl(var(--b2));
    backdrop-filter: blur(var(--notifications-blur, 20px));
    -webkit-backdrop-filter: blur(var(--notifications-blur, 20px));
    border: 1px solid hsl(var(--b3));
    border-radius: 12px;
    box-shadow:
        0 12px 40px rgba(0, 0, 0, 0.4),
        0 2px 8px rgba(0, 0, 0, 0.2);
    z-index: 9999;
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.history-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 10px 12px;
    border-bottom: 1px solid hsl(var(--b3));
    flex-shrink: 0;
}

.history-body {
    overflow-y: auto;
    flex: 1;
    padding: 6px 0;
}

.history-empty {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    padding: 32px 16px;
}

.history-list {
    display: flex;
    flex-direction: column;
}

.history-group {
    margin-bottom: 4px;
}

.history-group-label {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.06em;
    color: hsl(var(--bc) / 0.4);
    padding: 6px 14px 2px;
}

.history-entry {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 7px 14px;
    cursor: pointer;
    transition: background 0.15s, opacity 0.2s;
    border-radius: 6px;
    margin: 0 4px;
    animation: slideInUpFade 0.4s cubic-bezier(0.16, 1, 0.3, 1) both;
}

@keyframes slideInUpFade {
    from {
        opacity: 0;
        transform: translateY(8px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.history-entry:hover {
    background: hsl(var(--b3) / 0.6);
}

.history-entry-icon {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
}

.history-entry-info {
    flex: 1;
    min-width: 0;
}

.history-entry-name {
    font-size: 12px;
    font-weight: 500;
    word-break: break-word;
    color: hsl(var(--bc));
    line-height: 1.3;
}

.history-entry-time {
    font-size: 11px;
    color: hsl(var(--bc) / 0.35);
    flex-shrink: 0;
    white-space: nowrap;
    text-align: right;
    display: flex;
    flex-direction: column;
    align-items: flex-end;
}

.history-panel-enter-active,
.history-panel-leave-active {
    transition:
        opacity 0.25s cubic-bezier(0.16, 1, 0.3, 1),
        transform 0.25s cubic-bezier(0.16, 1, 0.3, 1);
}

.history-panel-enter-from,
.history-panel-leave-to {
    opacity: 0;
    transform: translateY(-8px) scale(0.96);
    pointer-events: none;
}
</style>

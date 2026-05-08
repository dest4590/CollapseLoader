<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { History, Trash2, X, Play, Clock } from "lucide-vue-next";

interface LaunchEntry {
    client_id: number;
    client_name: string;
    client_version: string;
    launched_at: string;
    account_name: string | null;
}

const emit = defineEmits<{
    close: [];
    launch: [clientId: number];
}>();

const { t } = useI18n();
const entries = ref<LaunchEntry[]>([]);
const isClearing = ref(false);

const loadHistory = async () => {
    entries.value = await invoke<LaunchEntry[]>("get_launch_history");
};

const clearHistory = async () => {
    isClearing.value = true;
    try {
        await invoke("clear_launch_history");
        entries.value = [];
    } finally {
        isClearing.value = false;
    }
};

const formatTime = (iso: string): string => {
    const date = new Date(iso);
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
    const groups: { label: string; items: LaunchEntry[] }[] = [];
    const today: LaunchEntry[] = [];
    const yesterday: LaunchEntry[] = [];
    const older: LaunchEntry[] = [];

    const now = new Date();
    const todayStart = new Date(now.getFullYear(), now.getMonth(), now.getDate());
    const yesterdayStart = new Date(todayStart.getTime() - 86400000);

    for (const entry of entries.value) {
        const d = new Date(entry.launched_at);
        if (d >= todayStart) today.push(entry);
        else if (d >= yesterdayStart) yesterday.push(entry);
        else older.push(entry);
    }

    if (today.length) groups.push({ label: t("history.today"), items: today });
    if (yesterday.length) groups.push({ label: t("history.yesterday"), items: yesterday });
    if (older.length) groups.push({ label: t("history.earlier"), items: older });

    return groups;
});

onMounted(loadHistory);
</script>

<template>
    <Transition name="history-panel">
        <div class="history-panel">
            <div class="history-header">
                <div class="flex items-center gap-2">
                    <History class="w-4 h-4 text-primary" />
                    <span class="font-semibold text-sm">{{ t("history.title") }}</span>
                </div>
                <div class="flex items-center gap-1">
                    <button
                        v-if="entries.length > 0"
                        @click="clearHistory"
                        :disabled="isClearing"
                        class="btn btn-ghost btn-xs text-base-content/50 hover:text-error gap-1"
                    >
                        <Trash2 class="w-3 h-3" />
                        {{ t("history.clear") }}
                    </button>
                    <button @click="emit('close')" class="btn btn-ghost btn-xs btn-circle">
                        <X class="w-4 h-4" />
                    </button>
                </div>
            </div>

            <div class="history-body">
                <div
                    v-if="entries.length === 0"
                    class="history-empty"
                >
                    <Clock class="w-8 h-8 opacity-20 mb-2" />
                    <p class="text-sm text-base-content/40">{{ t("history.empty") }}</p>
                </div>

                <div v-else class="history-list">
                    <div
                        v-for="group in groupedEntries"
                        :key="group.label"
                        class="history-group"
                    >
                        <div class="history-group-label">{{ group.label }}</div>
                        <div
                            v-for="(entry, idx) in group.items"
                            :key="idx"
                            class="history-entry"
                            @click="emit('launch', entry.client_id)"
                        >
                            <div class="history-entry-icon">
                                <Play class="w-3 h-3 text-primary" />
                            </div>
                            <div class="history-entry-info">
                                <div class="history-entry-name">{{ entry.client_name }}</div>
                                <div class="history-entry-meta">
                                    <span class="history-entry-version">{{ entry.client_version }}</span>
                                    <span v-if="entry.account_name" class="history-entry-account">
                                        · {{ entry.account_name }}
                                    </span>
                                </div>
                            </div>
                            <div class="history-entry-time">
                                {{ formatTime(entry.launched_at) }}
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
    backdrop-filter: blur(var(--history-blur, 20px));
    -webkit-backdrop-filter: blur(var(--history-blur, 20px));
    border: 1px solid hsl(var(--b3));
    border-radius: 12px;
    box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4), 0 2px 8px rgba(0, 0, 0, 0.2);
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
    transition: background 0.15s;
    border-radius: 6px;
    margin: 0 4px;
}

.history-entry:hover {
    background: hsl(var(--b3) / 0.6);
}

.history-entry-icon {
    width: 24px;
    height: 24px;
    border-radius: 6px;
    background: hsl(var(--p) / 0.12);
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
    font-size: 13px;
    font-weight: 500;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    color: hsl(var(--bc));
}

.history-entry-meta {
    font-size: 11px;
    color: hsl(var(--bc) / 0.5);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.history-entry-version {
    color: hsl(var(--p) / 0.8);
}

.history-entry-time {
    font-size: 11px;
    color: hsl(var(--bc) / 0.35);
    flex-shrink: 0;
    white-space: nowrap;
}

.history-panel-enter-active,
.history-panel-leave-active {
    transition: opacity 0.15s ease, transform 0.15s ease;
}

.history-panel-enter-from,
.history-panel-leave-to {
    opacity: 0;
    transform: translateY(-6px) scale(0.98);
}
</style>

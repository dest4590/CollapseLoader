import { ref, computed } from "vue";
import type { ToastType } from "@shared/types/toast";

export interface NotificationHistoryEntry {
    id: string;
    message: string;
    type: ToastType;
    timestamp: number;
    isRead: boolean;
}

const STORAGE_KEY = "notification_history";
const MAX_HISTORY_SIZE = 100;

const history = ref<NotificationHistoryEntry[]>([]);

function loadHistory() {
    try {
        const stored = localStorage.getItem(STORAGE_KEY);
        if (stored) {
            history.value = JSON.parse(stored);
        }
    } catch (e) {
        console.warn("Failed to load notification history", e);
    }
}

function saveHistory() {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(history.value));
    } catch (e) {
        console.warn("Failed to save notification history", e);
    }
}

loadHistory();

function addNotificationHistory(message: string, type: ToastType) {
    const entry: NotificationHistoryEntry = {
        id: crypto.randomUUID ? crypto.randomUUID() : Date.now().toString() + Math.random().toString(36).slice(2),
        message,
        type,
        timestamp: Date.now(),
        isRead: false,
    };
    
    history.value.unshift(entry);
    
    if (history.value.length > MAX_HISTORY_SIZE) {
        history.value = history.value.slice(0, MAX_HISTORY_SIZE);
    }
    
    saveHistory();
}

function markAllAsRead() {
    let changed = false;
    history.value.forEach((entry) => {
        if (!entry.isRead) {
            entry.isRead = true;
            changed = true;
        }
    });
    if (changed) {
        saveHistory();
    }
}

function markAsRead(id: string) {
    const entry = history.value.find((e) => e.id === id);
    if (entry && !entry.isRead) {
        entry.isRead = true;
        saveHistory();
    }
}

function clearHistory() {
    history.value = [];
    saveHistory();
}

export function useNotificationHistory() {
    const unreadCount = computed(() => history.value.filter((e) => !e.isRead).length);
    
    return {
        history,
        unreadCount,
        addNotificationHistory,
        markAllAsRead,
        markAsRead,
        clearHistory,
    };
}

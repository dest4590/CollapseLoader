import { ref, computed } from 'vue';

const STREAMER_MODE_KEY = 'streamerModeEnabled';

const isStreamerModeEnabled = ref<boolean>(
    localStorage.getItem(STREAMER_MODE_KEY) === 'true'
);

export function useStreamerMode() {
    const toggleStreamerMode = (enabled: boolean) => {
        isStreamerModeEnabled.value = enabled;
        localStorage.setItem(STREAMER_MODE_KEY, enabled.toString());
    };

    const getDisplayName = (nickname?: string, username?: string, fallback = 'User'): string => {
        if (isStreamerModeEnabled.value) {
            return '??????';
        }
        return nickname || username || fallback;
    };

    const getDisplayUsername = (username?: string): string => {
        if (isStreamerModeEnabled.value) {
            return 'unknown';
        }
        return username || 'user';
    };

    const getDisplayEmail = (email?: string): string => {
        if (isStreamerModeEnabled.value) {
            return 'unknown@*****.***';
        }
        return email || '';
    };

    return {
        isStreamerModeEnabled: computed(() => isStreamerModeEnabled.value),
        toggleStreamerMode,
        getDisplayName,
        getDisplayUsername,
        getDisplayEmail
    };
}
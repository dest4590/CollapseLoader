import { ref, computed } from 'vue';

const STORAGE_KEY = 'streamerModeEnabled';

function readStoredFlag(key: string): boolean {
    try {
        return localStorage.getItem(key) === 'true';
    } catch (e) {

        console.warn('useStreamerMode: failed to read storage', e);
        return false;
    }
}

function writeStoredFlag(key: string, value: boolean): void {
    try {
        localStorage.setItem(key, value ? 'true' : 'false');
    } catch (e) {
        console.warn('useStreamerMode: failed to write storage', e);
    }
}

const isStreamerModeEnabled = ref<boolean>(readStoredFlag(STORAGE_KEY));

type ChangeListener = (enabled: boolean) => void;
const listeners = new Set<ChangeListener>();

function emitChange(enabled: boolean) {
    for (const l of Array.from(listeners)) {
        try {
            l(enabled);
        } catch (e) {
            console.error('useStreamerMode listener error', e);
        }
    }
}

function maskString(input: string | undefined | null, maskChar = '*'): string {
    if (!input) return '';
    return Array.from(input).map(() => maskChar).join('');
}

function maskName(): string {
    return '??????';
}

function maskUsername(): string {
    return 'unknown';
}

function maskEmail(): string {
    return 'unknown@*****.***';
}

export function useStreamerMode() {
    const isEnabled = computed(() => isStreamerModeEnabled.value);

    function setStreamerMode(enabled: boolean) {
        isStreamerModeEnabled.value = enabled;
        writeStoredFlag(STORAGE_KEY, enabled);
        emitChange(enabled);
    }

    function toggleStreamerMode(enabled?: boolean) {
        if (typeof enabled === 'boolean') {
            setStreamerMode(enabled);
            return;
        }
        setStreamerMode(!isStreamerModeEnabled.value);
    }

    function onChange(cb: ChangeListener): () => void {
        listeners.add(cb);
        return () => listeners.delete(cb);
    }

    function getDisplayName(nickname?: string, username?: string, fallback = 'User'): string {
        if (isStreamerModeEnabled.value) return maskName();
        return nickname || username || fallback;
    }

    function getDisplayUsername(username?: string): string {
        if (isStreamerModeEnabled.value) return maskUsername();
        return username || 'user';
    }

    function getDisplayEmail(email?: string): string {
        if (isStreamerModeEnabled.value) return maskEmail();
        return email || '';
    }

    function maskIfEnabled(value?: string, masker: (v?: string) => string = maskString): string {
        return isStreamerModeEnabled.value ? masker(value) : (value || '');
    }

    return {
        isStreamerModeEnabled: isEnabled,
        toggleStreamerMode,
        setStreamerMode,
        onChange,
        getDisplayName,
        getDisplayUsername,
        getDisplayEmail,
        maskString,
        maskName,
        maskUsername,
        maskEmail,
        maskIfEnabled
    };
}
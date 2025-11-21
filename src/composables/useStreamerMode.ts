import { ref, computed } from 'vue';

const STORAGE_KEY = 'streamerModeEnabled';

function readStoredFlag(key: string): boolean {
    try {
        return localStorage.getItem(key) === 'true';
    } catch (e) {
        // localStorage may not be available in some environments (SSR), or read can fail.
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
let storageListenerInstalled = false;

function emitChange(enabled: boolean) {
    for (const l of Array.from(listeners)) {
        try {
            l(enabled);
        } catch (e) {
            console.error('useStreamerMode listener error', e);
        }
    }
}

/** Mask arbitrary string using maskChar and preserve character count (handles unicode code points) */
function maskString(input: string | undefined | null, maskChar = '*'): string {
    if (!input) return '';
    // Use spread to correctly handle unicode characters
    return maskChar.repeat([...input].length);
}

/** Convenient placeholders used when streamer mode is enabled */
function maskName(name?: string): string {
    if (!name) return 'User';
    // Keep last char and mask the rest for a friendlier look
    const chars = [...name];
    if (chars.length <= 1) return chars[0] || 'U';
    return maskString(chars.slice(0, -1).join('')) + chars[chars.length - 1];
}

function maskUsername(username?: string): string {
    if (!username) return 'user';
    const chars = [...username];
    // Keep up to 2 leading chars for readability
    const lead = chars.slice(0, 2).join('');
    return lead + maskString(chars.slice(2).join(''));
}

function maskEmail(email?: string): string {
    if (!email) return 'unknown@*****.***';
    // Basic email mask: keep the domain's TLD and mask the local part
    const [local, domain] = email.split('@');
    if (!domain) return maskString(email);
    const domainParts = domain.split('.');
    const tld = domainParts.pop();
    const maskedLocal = local ? local[0] + maskString(local.slice(1)) : '';
    const maskedDomain = domainParts.join('.') ? domainParts.map(() => '*****').join('.') : '*****';
    return `${maskedLocal}@${maskedDomain}.${tld}`;
}

export function useStreamerMode() {
    // Expose readonly computed ref to prevent direct mutation from consumers
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
        const name = nickname || username || fallback;
        return isStreamerModeEnabled.value ? maskName(name) : name;
    }

    function getDisplayUsername(username?: string): string {
        const value = username || 'user';
        return isStreamerModeEnabled.value ? maskUsername(value) : value;
    }

    function getDisplayEmail(email?: string): string {
        const value = email || '';
        return isStreamerModeEnabled.value ? maskEmail(value) : value;
    }

    function maskIfEnabled(value?: string, masker: (v?: string) => string = maskString): string {
        return isStreamerModeEnabled.value ? masker(value) : (value || '');
    }

    // Note: the storage event listener is installed once at module init to avoid
    // adding duplicate listeners every time the composable is used.

    return {
        isStreamerModeEnabled: isEnabled,
        // alias for convenience
        enabled: isEnabled,
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

export default useStreamerMode;

// Install storage listener once at module initialization to keep state in sync
if (typeof window !== 'undefined' && typeof window.addEventListener === 'function') {
    if (!storageListenerInstalled) {
        window.addEventListener('storage', (e: StorageEvent) => {
            if (e.key === STORAGE_KEY) {
                const newVal = e.newValue === 'true';
                if (isStreamerModeEnabled.value !== newVal) {
                    isStreamerModeEnabled.value = newVal;
                    emitChange(newVal);
                }
            }
        });
        storageListenerInstalled = true;
    }
}
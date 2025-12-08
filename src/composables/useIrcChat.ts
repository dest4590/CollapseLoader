import { ref } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { useToast } from '../services/toastService';

interface IrcMessage {
    time: string;
    content: string;
    type?: string;
    isHistory?: boolean;
}

interface IncomingIrcPayload {
    type: string;
    time?: string;
    content?: string;
    history?: boolean;
}

type IrcStatus = 'disconnected' | 'connecting' | 'connected' | 'reconnecting' | 'error';

const RECONNECT_DELAY_MS = 4000;

const messages = ref<IrcMessage[]>([]);
const connected = ref(false);
const isConnecting = ref(false);
const status = ref<IrcStatus>('disconnected');

let connectionPromise: Promise<void> | null = null;
let listenersRegistered = false;
let reconnectTimer: ReturnType<typeof setTimeout> | null = null;

const { addToast } = useToast();

const currentTime = (): string => {
    const now = new Date();
    return `${now.getHours().toString().padStart(2, '0')}:${now.getMinutes().toString().padStart(2, '0')}`;
};

const formatIsoToTime = (isoString?: string): string => {
    if (!isoString) return currentTime();
    const parsed = new Date(isoString);
    if (Number.isNaN(parsed.getTime())) return currentTime();

    const hours = parsed.getHours().toString().padStart(2, '0');
    const minutes = parsed.getMinutes().toString().padStart(2, '0');
    return `${hours}:${minutes}`;
};

const parseIrcPayload = (payload: unknown): IrcMessage | null => {
    const fallbackTime = currentTime();

    if (typeof payload === 'string') {
        try {
            const parsed = JSON.parse(payload) as IncomingIrcPayload;

            if (parsed.type === 'pong') return null;

            return {
                time: formatIsoToTime(parsed.time),
                content: parsed.content || '',
                type: parsed.type,
                isHistory: Boolean(parsed.history),
            };
        } catch {
            return { time: fallbackTime, content: payload, type: 'system' };
        }
    }

    return null;
};

const clearReconnectTimer = () => {
    if (reconnectTimer) {
        clearTimeout(reconnectTimer);
        reconnectTimer = null;
    }
};

const registerListeners = async (): Promise<void> => {
    if (listenersRegistered) {
        return;
    }

    listenersRegistered = true;

    try {
        await listen<string>('irc-message', (event) => {
            console.debug('IRC: message event', event.payload);
            const msg = parseIrcPayload(event.payload);
            if (msg) {
                messages.value.push(msg);
            }
        });

        await listen('irc-connected', () => {
            console.debug('IRC: connected event');
            connected.value = true;
            status.value = 'connected';
            clearReconnectTimer();
        });

        await listen('irc-disconnected', () => {
            console.debug('IRC: disconnected event');
            connected.value = false;
            status.value = 'reconnecting';
            messages.value.push({ time: currentTime(), content: 'Disconnected from IRC server.', type: 'error' });
            connectionPromise = null;
            scheduleReconnect('Connection lost. Reconnecting...');
        });

        await listen<string>('irc-error', (event) => {
            console.error('IRC: error event', event.payload);
            connected.value = false;
            status.value = 'error';
            connectionPromise = null;
            addToast(
                'IRC Error: ' + event.payload,
                'error',
            );
            scheduleReconnect('Error occurred. Attempting to reconnect...');
        });
    } catch (err) {
        listenersRegistered = false;
        throw err;
    }
};

const scheduleReconnect = (reason?: string) => {
    if (reconnectTimer) return;

    if (reason) {
        messages.value.push({ time: currentTime(), content: reason, type: 'system' });
    }

    status.value = 'reconnecting';
    reconnectTimer = setTimeout(async () => {
        reconnectTimer = null;
        try {
            await ensureIrcConnection(true);
        } catch (err) {
            console.error('IRC: reconnection attempt failed', err);
            scheduleReconnect('Reconnection failed. Retrying...');
        }
    }, RECONNECT_DELAY_MS);
};

export const ensureIrcConnection = async (isReconnect = false): Promise<void> => {
    await registerListeners();

    if (connectionPromise) {
        return connectionPromise;
    }

    connectionPromise = (async () => {
        isConnecting.value = true;
        status.value = isReconnect ? 'reconnecting' : 'connecting';
        const token = localStorage.getItem('authToken') || '';
        const tokenPresent = token.length > 0;
        console.debug('IRC: attempting connect, token present=', tokenPresent);

        clearReconnectTimer();
        await invoke('connect_irc', { token: tokenPresent ? token : null });
    })()
        .catch((err) => {
            connectionPromise = null;
            status.value = 'error';
            scheduleReconnect('Connection failed. Retrying...');
            throw err;
        })
        .finally(() => {
            isConnecting.value = false;
        });

    return connectionPromise;
};

export function forceReconnect(): void {
    console.debug('IRC: forceReconnect called');
    connectionPromise = null;
    ensureIrcConnection(true).catch((err) => {
        console.error('IRC: forceReconnect failed', err);
    });
}

export const sendIrcMessage = async (message: string): Promise<void> => {
    await invoke('send_irc_message', { message });
};

export function useIrcChat() {
    return {
        messages,
        connected,
        isConnecting,
        status,
        ensureIrcConnection,
        forceReconnect,
        sendIrcMessage
    };
}
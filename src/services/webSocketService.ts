import { Client } from '@stomp/stompjs';
import { getApiUrl } from '../config';

class WebSocketService {
    private client: Client | null = null;
    private connected: boolean = false;

    private initializeClient() {
        if (this.client) {
            return;
        }

        this.client = new Client({
            brokerURL: this.getBrokerUrl(),
            connectHeaders: {
                Authorization: `Bearer ${localStorage.getItem('authToken')}` || '',
            },
            reconnectDelay: 5000,
            heartbeatIncoming: 10000,
            heartbeatOutgoing: 10000,
            onConnect: () => {
                this.connected = true;
                console.log('Connected to WebSocket');
                this.subscribeToUserAchievements();
            },
            onDisconnect: () => {
                this.connected = false;
                console.log('Disconnected from WebSocket');
            },
            onStompError: (frame) => {
                console.error('Broker reported error: ' + frame.headers['message']);
                console.error('Additional details: ' + frame.body);
            },
            onWebSocketClose: () => {
                this.connected = false;
                console.log('WebSocket connection closed');
            },
        });
    }

    private getBrokerUrl(): string {
        const baseUrl = getApiUrl();

        if (!baseUrl) {
            throw new Error('API URL is not initialized');
        }

        const url = new URL(baseUrl);

        url.protocol = url.protocol === 'https:' ? 'wss:' : 'ws:';
        url.pathname = '/ws';

        return url.toString();
    }

    public connect() {
        if (!this.connected) {
            this.initializeClient();
            this.client?.activate();
        }
    }

    public disconnect() {
        if (this.connected && this.client) {
            this.client.deactivate();
        }
    }

    private subscribeToUserAchievements() {
        if (!this.client) {
            console.error('Cannot subscribe: WebSocket client is not initialized');
            return;
        }

        this.client.subscribe('/user/queue/achievements', (message) => {
            if (message.body) {
                const achievement = JSON.parse(message.body);
                this.handleAchievementUnlock(achievement);
            }
        });
    }

    private handleAchievementUnlock(achievement: any) {
        const event = new CustomEvent('achievement-unlocked', { detail: achievement });
        window.dispatchEvent(event);
    }
}

export const webSocketService = new WebSocketService();

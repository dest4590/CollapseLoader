import { Client } from "@stomp/stompjs";
import { getApiUrl } from "@/config";
import { updaterService } from "@core/updater/updaterService";
import i18n from "@core/i18n";
import { STORAGE_KEYS } from "@shared/utils/storageKeys";

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
                Authorization:
                    `Bearer ${localStorage.getItem(STORAGE_KEYS.AUTH_TOKEN)}` ||
                    "",
            },
            reconnectDelay: 5000,
            heartbeatIncoming: 10000,
            heartbeatOutgoing: 10000,
            onConnect: () => {
                this.connected = true;
                console.log("Connected to WebSocket");
                this.subscribeToUserAchievements();
                this.subscribeToFriendNotifications();
                this.subscribeToCommands();
                this.subscribeToBroadcasts();
            },
            onDisconnect: () => {
                this.connected = false;
                console.log("Disconnected from WebSocket");
            },
            onStompError: (frame) => {
                console.error(
                    "Broker reported error: " + frame.headers["message"]
                );
                console.error("Additional details: " + frame.body);
            },
            onWebSocketClose: () => {
                this.connected = false;
                console.log("WebSocket connection closed");
            },
        });
    }

    private getBrokerUrl(): string {
        const baseUrl = getApiUrl();

        if (!baseUrl) {
            throw new Error("API URL is not initialized");
        }

        const url = new URL(baseUrl);

        url.protocol = url.protocol === "https:" ? "wss:" : "ws:";
        url.pathname = "/ws";

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

    private safeSubscribe(
        destination: string,
        handler: (data: any) => void
    ): void {
        if (!this.client) {
            console.error(
                "Cannot subscribe: WebSocket client is not initialized"
            );
            return;
        }
        this.client.subscribe(destination, (message) => {
            if (message.body) handler(JSON.parse(message.body));
        });
    }

    private subscribeToUserAchievements() {
        this.safeSubscribe("/user/queue/achievements", (data) =>
            this.handleAchievementUnlock(data)
        );
    }

    private subscribeToFriendNotifications() {
        this.safeSubscribe("/user/queue/friends", (data) =>
            this.handleFriendNotification(data)
        );
    }

    private handleFriendNotification(data: any) {
        let eventType = "";
        if (data.type === "REQUEST_RECEIVED") {
            eventType = "friend-request-received";
        } else if (
            data.type === "REQUEST_ACCEPTED" ||
            data.type === "FRIEND_ADDED"
        ) {
            eventType = "friend-request-accepted";
        }

        if (eventType) {
            const event = new CustomEvent(eventType, { detail: data });
            window.dispatchEvent(event);
        }
    }

    private handleAchievementUnlock(achievement: any) {
        const event = new CustomEvent("achievement-unlocked", {
            detail: achievement,
        });
        window.dispatchEvent(event);
    }

    private subscribeToCommands() {
        this.safeSubscribe("/topic/commands", (data) =>
            this.handleCommand(data)
        );
    }

    private async handleCommand(data: any) {
        if (data.command === "CHECK_FOR_UPDATES") {
            const t = i18n.global.t;
            await updaterService.checkForUpdates(false, t);
        }
    }

    private subscribeToBroadcasts() {
        for (const topic of [
            "/topic/broadcast",
            "/topic/broadcast/users",
            "/topic/broadcast/guests",
        ]) {
            this.safeSubscribe(topic, (data) => this.handleBroadcast(data));
        }
    }

    private handleBroadcast(data: any) {
        const event = new CustomEvent("system-broadcast", {
            detail: data,
        });
        window.dispatchEvent(event);
    }
}

export const webSocketService = new WebSocketService();

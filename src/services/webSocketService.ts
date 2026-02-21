import { Client } from "@stomp/stompjs";
import { getApiUrl } from "../config";
import { updaterService } from "./updaterService";
import i18n from "../i18n";

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
                    `Bearer ${localStorage.getItem("authToken")}` || "",
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
                this.subscribeToOnlineCount();
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

    private subscribeToUserAchievements() {
        if (!this.client) {
            console.error(
                "Cannot subscribe: WebSocket client is not initialized"
            );
            return;
        }

        this.client.subscribe("/user/queue/achievements", (message) => {
            if (message.body) {
                const achievement = JSON.parse(message.body);
                this.handleAchievementUnlock(achievement);
            }
        });
    }

    private subscribeToFriendNotifications() {
        if (!this.client) {
            console.error(
                "Cannot subscribe: WebSocket client is not initialized"
            );
            return;
        }

        this.client.subscribe("/user/queue/friends", (message) => {
            if (message.body) {
                const data = JSON.parse(message.body);
                this.handleFriendNotification(data);
            }
        });
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
        if (!this.client) {
            console.error(
                "Cannot subscribe: WebSocket client is not initialized"
            );
            return;
        }

        this.client.subscribe("/topic/commands", (message) => {
            if (message.body) {
                const data = JSON.parse(message.body);
                this.handleCommand(data);
            }
        });
    }

    private async handleCommand(data: any) {
        if (data.command === "CHECK_FOR_UPDATES") {
            const t = i18n.global.t;
            await updaterService.checkForUpdates(false, t);
        }
    }

    private subscribeToOnlineCount() {
        if (!this.client) {
            return;
        }

        this.client.subscribe("/topic/online-count", (message) => {
            if (message.body) {
                const data = JSON.parse(message.body);
                const event = new CustomEvent("online-count-updated", {
                    detail: data,
                });
                window.dispatchEvent(event);
            }
        });
    }
}

export const webSocketService = new WebSocketService();

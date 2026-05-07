import { ref, readonly } from "vue";
import { supabase } from "./supabaseClient";
import { formatTime } from "@shared/utils/utils";
import type { RealtimeChannel } from "@supabase/supabase-js";

export interface ChatMessage {
    id: number;
    created_at: string;
    username: string;
    content: string;
    user_id: string | null;
    role: string;
    time: string;
}

type ChatStatus = "disconnected" | "connecting" | "connected" | "error";

const HISTORY_LIMIT = 50;
const MAX_MESSAGES = 200;
export const MESSAGE_MAX_LENGTH = 500;

const messages = ref<ChatMessage[]>([]);
const status = ref<ChatStatus>("disconnected");
const onlineCount = ref(0);
const isLoading = ref(false);
const error = ref<string | null>(null);

let channel: RealtimeChannel | null = null;
let presenceChannel: RealtimeChannel | null = null;
let connectedUsername = "";

const formatMessageTime = (isoString: string): string => {
    const date = new Date(isoString);
    if (isNaN(date.getTime())) return formatTime();
    return formatTime(date);
};

const appendMessage = (msg: ChatMessage) => {
    if (messages.value.some((m) => m.id === msg.id)) return;
    messages.value.push(msg);
    if (messages.value.length > MAX_MESSAGES) {
        messages.value = messages.value.slice(-MAX_MESSAGES);
    }
};

export function useChatService() {
    const loadHistory = async () => {
        isLoading.value = true;
        error.value = null;
        try {
            const { data, error: err } = await supabase
                .from("chat_messages")
                .select("*")
                .order("created_at", { ascending: false })
                .limit(HISTORY_LIMIT);

            if (err) throw err;

            if (data) {
                const historical = [...data].reverse().map((row) => ({
                    ...row,
                    time: formatMessageTime(row.created_at),
                }));
                messages.value = [];
                historical.forEach(appendMessage);
            }
        } catch (e: any) {
            console.error("[Chat] Failed to load history:", e);
            error.value = e.message ?? "Failed to load messages";
        } finally {
            isLoading.value = false;
        }
    };

    const connect = async (username: string, userId?: string | null) => {
        if (channel && connectedUsername === username) return;

        if (channel) await disconnect();

        connectedUsername = username;
        status.value = "connecting";
        error.value = null;

        await loadHistory();

        channel = supabase
            .channel("chat_messages_realtime")
            .on(
                "postgres_changes",
                {
                    event: "INSERT",
                    schema: "public",
                    table: "chat_messages",
                },
                (payload) => {
                    const row = payload.new as Omit<ChatMessage, "time">;
                    appendMessage({
                        ...row,
                        time: formatMessageTime(row.created_at),
                    });
                }
            )
            .subscribe((s) => {
                if (s === "SUBSCRIBED") {
                    status.value = "connected";
                } else if (s === "CHANNEL_ERROR" || s === "TIMED_OUT") {
                    status.value = "error";
                    error.value = "Realtime connection failed";
                }
            });

        presenceChannel = supabase
            .channel("chat_presence", {
                config: { presence: { key: userId ?? username } },
            })
            .on("presence", { event: "sync" }, () => {
                const state = presenceChannel!.presenceState();
                onlineCount.value = Object.keys(state).length;
            })
            .subscribe(async (s) => {
                if (s === "SUBSCRIBED") {
                    await presenceChannel!.track({
                        username,
                        online_at: new Date().toISOString(),
                    });
                }
            });
    };

    const disconnect = async () => {
        if (presenceChannel) {
            await presenceChannel.untrack();
            await supabase.removeChannel(presenceChannel);
            presenceChannel = null;
        }
        if (channel) {
            await supabase.removeChannel(channel);
            channel = null;
        }
        status.value = "disconnected";
        onlineCount.value = 0;
        connectedUsername = "";
    };

    const sendMessage = async (
        content: string,
        username: string,
        userId?: string | null,
        role = "user"
    ) => {
        const trimmed = content.trim();
        if (!trimmed) return;
        if (trimmed.length > MESSAGE_MAX_LENGTH) {
            throw new Error(
                `Message too long (max ${MESSAGE_MAX_LENGTH} chars)`
            );
        }

        // Only authenticated users (server accounts) can send messages
        if (!userId) {
            throw new Error("chat.error.login_required");
        }
        const { data: banData } = await supabase
            .from("chat_bans")
            .select("reason")
            .eq("user_id", userId)
            .maybeSingle();

        if (banData) {
            const reason = banData.reason ? `: ${banData.reason}` : "";
            throw new Error(`chat.error.banned${reason}`);
        }

        const { data: roleData } = await supabase
            .from("chat_roles")
            .select("role")
            .eq("user_id", userId)
            .maybeSingle();

        const resolvedRole = roleData?.role ?? role;

        const { error: err } = await supabase.from("chat_messages").insert({
            content: trimmed,
            username,
            user_id: userId ?? null,
            role: resolvedRole,
        });

        if (err) throw err;
    };

    return {
        messages: readonly(messages),
        status: readonly(status),
        onlineCount: readonly(onlineCount),
        isLoading: readonly(isLoading),
        error: readonly(error),
        connect,
        disconnect,
        sendMessage,
    };
}

<script setup lang="ts">
import {
    ref,
    computed,
    onMounted,
    onUnmounted,
    nextTick,
    watch,
} from "vue";
import { useI18n } from "vue-i18n";
import {
    Send,
    Users,
    Wifi,
    WifiOff,
    Loader2,
    MessageSquare,
    AlertCircle,
} from "lucide-vue-next";
import { useChatService } from "@services/chat/useChatService";
import { useToast } from "@shared/composables/useToast";
import UserAvatar from "@shared/components/ui/UserAvatar.vue";
import { useStreamerMode } from "@features/social/useStreamerMode";

const { t } = useI18n();
const { addToast } = useToast();
const streamer = useStreamerMode();

const {
    messages,
    status,
    onlineCount,
    isLoading,
    error,
    connect,
    disconnect,
    sendMessage,
} = useChatService();

// ── local username (from active account or auth) ──────────────────────────
const localUsername = computed<string>(() => {
    try {
        const accounts = JSON.parse(
            localStorage.getItem("accounts") ?? "[]"
        ) as Array<{ username: string; is_active?: boolean }>;
        const active = accounts.find((a) => a.is_active);
        if (active?.username) return active.username;
    } catch {}
    return "Guest" + Math.floor(Math.random() * 9000 + 1000);
});

const displayUsername = computed(() =>
    streamer.getDisplayName(null, localUsername.value)
);

const userId = computed<string | null>(() => {
    return localStorage.getItem("authToken") ? "auth" : null;
});

// ── input ─────────────────────────────────────────────────────────────────
const inputText = ref("");
const isSending = ref(false);
const messagesEndRef = ref<HTMLElement | null>(null);
const inputRef = ref<HTMLInputElement | null>(null);
const MAX_LENGTH = 500;

const charCount = computed(() => inputText.value.length);
const isOverLimit = computed(() => charCount.value > MAX_LENGTH);

// ── scroll to bottom ──────────────────────────────────────────────────────
const scrollToBottom = (smooth = true) => {
    nextTick(() => {
        messagesEndRef.value?.scrollIntoView({
            behavior: smooth ? "smooth" : "instant",
        });
    });
};

watch(
    () => messages.value.length,
    () => scrollToBottom()
);

// ── send ──────────────────────────────────────────────────────────────────
const handleSend = async () => {
    const text = inputText.value.trim();
    if (!text || isSending.value || isOverLimit.value) return;

    isSending.value = true;
    try {
        await sendMessage(text, localUsername.value, userId.value);
        inputText.value = "";
        inputRef.value?.focus();
    } catch (e: any) {
        addToast(e.message ?? t("chat.send_failed"), "error");
    } finally {
        isSending.value = false;
    }
};

const handleKeydown = (e: KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        handleSend();
    }
};

// ── status helpers ────────────────────────────────────────────────────────
const statusColor = computed(() => {
    switch (status.value) {
        case "connected":
            return "text-success";
        case "connecting":
            return "text-warning";
        case "error":
            return "text-error";
        default:
            return "text-base-content/40";
    }
});

const statusLabel = computed(() => {
    switch (status.value) {
        case "connected":
            return t("chat.status.connected");
        case "connecting":
            return t("chat.status.connecting");
        case "error":
            return t("chat.status.error");
        default:
            return t("chat.status.disconnected");
    }
});

// ── role badge ────────────────────────────────────────────────────────────
const roleBadgeClass = (role: string) => {
    switch (role) {
        case "admin":
            return "badge badge-error badge-xs";
        case "moderator":
            return "badge badge-warning badge-xs";
        default:
            return "";
    }
};

// ── own message ───────────────────────────────────────────────────────────
const isOwnMessage = (username: string) =>
    username === localUsername.value;

// ── lifecycle ─────────────────────────────────────────────────────────────
onMounted(async () => {
    await connect(localUsername.value, userId.value);
    scrollToBottom(false);
    inputRef.value?.focus();
});

onUnmounted(() => {
    disconnect();
});
</script>

<template>
    <div class="flex flex-col h-full max-h-[calc(100vh-6rem)] slide-up">
        <!-- Header -->
        <div
            class="flex items-center justify-between mb-4 flex-shrink-0"
        >
            <div class="flex items-center gap-3">
                <MessageSquare class="w-6 h-6 text-primary" />
                <h1 class="text-2xl font-semibold text-primary-focus">
                    {{ t("chat.title") }}
                </h1>
            </div>

            <div class="flex items-center gap-3">
                <!-- Online count -->
                <div
                    v-if="status === 'connected'"
                    class="flex items-center gap-1.5 text-sm text-base-content/60"
                >
                    <Users class="w-4 h-4" />
                    <span>{{ onlineCount }}</span>
                </div>

                <!-- Connection status -->
                <div
                    class="flex items-center gap-1.5 text-sm"
                    :class="statusColor"
                >
                    <Loader2
                        v-if="status === 'connecting'"
                        class="w-4 h-4 animate-spin"
                    />
                    <Wifi v-else-if="status === 'connected'" class="w-4 h-4" />
                    <WifiOff v-else class="w-4 h-4" />
                    <span class="hidden sm:inline">{{ statusLabel }}</span>
                </div>
            </div>
        </div>

        <!-- Error banner -->
        <div
            v-if="error"
            class="flex items-center gap-2 p-3 mb-3 bg-error/10 border border-error/20 rounded-lg text-sm text-error flex-shrink-0"
        >
            <AlertCircle class="w-4 h-4 shrink-0" />
            {{ error }}
        </div>

        <!-- Messages area -->
        <div
            class="flex-1 overflow-y-auto bg-base-200 rounded-xl border border-base-300 p-4 space-y-3 min-h-0"
        >
            <!-- Loading skeleton -->
            <div v-if="isLoading" class="space-y-3">
                <div
                    v-for="i in 5"
                    :key="i"
                    class="flex items-start gap-3 animate-pulse"
                >
                    <div
                        class="w-8 h-8 rounded-full bg-base-300 shrink-0"
                    ></div>
                    <div class="flex-1 space-y-1.5">
                        <div class="h-3 bg-base-300 rounded w-24"></div>
                        <div
                            class="h-3 bg-base-300 rounded"
                            :style="{ width: 40 + i * 10 + '%' }"
                        ></div>
                    </div>
                </div>
            </div>

            <!-- Empty state -->
            <div
                v-else-if="messages.length === 0"
                class="flex flex-col items-center justify-center h-full py-12 text-base-content/40"
            >
                <MessageSquare class="w-12 h-12 mb-3 opacity-30" />
                <p>{{ t("chat.no_messages") }}</p>
            </div>

            <!-- Message list -->
            <template v-else>
                <div
                    v-for="msg in messages"
                    :key="msg.id"
                    class="flex items-start gap-3 group"
                    :class="{ 'flex-row-reverse': isOwnMessage(msg.username) }"
                >
                    <!-- Avatar -->
                    <div class="shrink-0">
                        <UserAvatar
                            :name="msg.username"
                            size="sm"
                        />
                    </div>

                    <!-- Bubble -->
                    <div
                        class="flex flex-col max-w-[75%]"
                        :class="
                            isOwnMessage(msg.username)
                                ? 'items-end'
                                : 'items-start'
                        "
                    >
                        <!-- Meta row -->
                        <div
                            class="flex items-center gap-1.5 mb-1"
                            :class="
                                isOwnMessage(msg.username)
                                    ? 'flex-row-reverse'
                                    : 'flex-row'
                            "
                        >
                            <span class="text-xs font-medium text-base-content/70">
                                {{
                                    isOwnMessage(msg.username)
                                        ? displayUsername
                                        : msg.username
                                }}
                            </span>
                            <span
                                v-if="msg.role !== 'user'"
                                :class="roleBadgeClass(msg.role)"
                            >
                                {{ msg.role }}
                            </span>
                            <span class="text-[10px] text-base-content/30">
                                {{ msg.time }}
                            </span>
                        </div>

                        <!-- Text bubble -->
                        <div
                            class="px-3 py-2 rounded-2xl text-sm break-words"
                            :class="
                                isOwnMessage(msg.username)
                                    ? 'bg-primary text-primary-content rounded-tr-sm'
                                    : 'bg-base-300 text-base-content rounded-tl-sm'
                            "
                        >
                            {{ msg.content }}
                        </div>
                    </div>
                </div>
            </template>

            <!-- Scroll anchor -->
            <div ref="messagesEndRef" />
        </div>

        <!-- Input area -->
        <div class="mt-3 flex-shrink-0">
            <div
                class="flex items-end gap-2 bg-base-200 border border-base-300 rounded-xl p-2"
                :class="{ 'border-error': isOverLimit }"
            >
                <div class="flex-1 relative">
                    <input
                        ref="inputRef"
                        v-model="inputText"
                        type="text"
                        class="input input-ghost w-full focus:outline-none text-sm pr-12"
                        :placeholder="
                            status === 'connected'
                                ? t('chat.input_placeholder', {
                                      username: displayUsername,
                                  })
                                : t('chat.connecting_placeholder')
                        "
                        :disabled="status !== 'connected' || isSending"
                        @keydown="handleKeydown"
                        maxlength="520"
                    />
                    <!-- Char counter -->
                    <span
                        v-if="charCount > 400"
                        class="absolute right-2 bottom-2 text-[10px]"
                        :class="
                            isOverLimit
                                ? 'text-error'
                                : 'text-base-content/30'
                        "
                    >
                        {{ charCount }}/{{ MAX_LENGTH }}
                    </span>
                </div>

                <button
                    class="btn btn-primary btn-sm shrink-0"
                    :disabled="
                        !inputText.trim() ||
                        status !== 'connected' ||
                        isSending ||
                        isOverLimit
                    "
                    @click="handleSend"
                >
                    <Loader2
                        v-if="isSending"
                        class="w-4 h-4 animate-spin"
                    />
                    <Send v-else class="w-4 h-4" />
                </button>
            </div>

            <!-- Hint -->
            <p class="text-[11px] text-base-content/30 mt-1 ml-1">
                {{ t("chat.send_hint") }}
            </p>
        </div>
    </div>
</template>

<style scoped>
.slide-up {
    animation: slideUp 0.4s ease-out forwards;
}

@keyframes slideUp {
    from {
        opacity: 0;
        transform: translateY(16px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}
</style>

<script setup lang="ts">
import {
    ref,
    computed,
    onMounted,
    nextTick,
    watch,
} from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import {
    Send,
    Users,
    Wifi,
    WifiOff,
    Loader2,
    MessageSquare,
    ChevronDown,
    ChevronUp,
} from "lucide-vue-next";
import { useChatService, MESSAGE_MAX_LENGTH } from "@services/chat/useChatService";
import { useToast } from "@shared/composables/useToast";
import UserAvatar from "@shared/components/ui/UserAvatar.vue";
import { useUser } from "@features/auth/useUser";

const { t } = useI18n();
const { addToast } = useToast();
const { displayName, username, isAuthenticated } = useUser();

const {
    messages,
    status,
    onlineCount,
    isLoading,
    connect,
    sendMessage,
} = useChatService();

// ── username resolution ───────────────────────────────────────────────────
const resolvedUsername = ref("Guest");

const loadUsername = async () => {
    if (isAuthenticated.value && (displayName.value || username.value)) {
        resolvedUsername.value = displayName.value || username.value;
        return;
    }
    try {
        interface Account { id: string; username: string; is_active: boolean; }
        const accounts = await invoke<Account[]>("get_accounts");
        const active = accounts.find((a) => a.is_active) ?? accounts[0];
        if (active?.username) { resolvedUsername.value = active.username; return; }
    } catch { /* ignore */ }
    resolvedUsername.value = "Guest";
};

watch([isAuthenticated, displayName, username], loadUsername);

// ── collapse state ────────────────────────────────────────────────────────
// Always start collapsed on app launch — no persistence
const isCollapsed = ref(true);

const toggleCollapse = () => {
    isCollapsed.value = !isCollapsed.value;
    if (!isCollapsed.value) nextTick(() => scrollToBottom());
};

// ── input & scroll ────────────────────────────────────────────────────────
const inputText = ref("");
const isSending = ref(false);
const messagesContainerRef = ref<HTMLElement | null>(null);
const inputRef = ref<HTMLInputElement | null>(null);

const charCount = computed(() => inputText.value.length);
const isOverLimit = computed(() => charCount.value > MESSAGE_MAX_LENGTH);

const scrollToBottom = () => {
    nextTick(() => {
        if (messagesContainerRef.value) {
            messagesContainerRef.value.scrollTop = messagesContainerRef.value.scrollHeight;
        }
    });
};

watch(() => messages.value.length, () => {
    if (!isCollapsed.value) scrollToBottom();
});

// ── send ──────────────────────────────────────────────────────────────────
const handleSend = async () => {
    const text = inputText.value.trim();
    if (!text || isSending.value || isOverLimit.value) return;
    isSending.value = true;
    try {
        // Pass authToken regardless of account type (local_ or server)
        const userId = localStorage.getItem("authToken");
        await sendMessage(text, resolvedUsername.value, userId);
        inputText.value = "";
        inputRef.value?.focus();
    } catch (e: any) {
        const msg: string = e.message ?? "";
        if (msg === "chat.error.login_required") {
            addToast(t("chat.error.login_required"), "error");
        } else if (msg.startsWith("chat.error.banned")) {
            const reason = msg.replace("chat.error.banned", "").trim();
            addToast(t("chat.error.banned") + reason, "error", 6000);
        } else {
            addToast(msg || t("chat.send_failed"), "error");
        }
    } finally {
        isSending.value = false;
    }
};

const handleKeydown = (e: KeyboardEvent) => {
    if (e.key === "Enter" && !e.shiftKey) { e.preventDefault(); handleSend(); }
};

// ── status helpers ────────────────────────────────────────────────────────
const statusDotClass = computed(() => {
    switch (status.value) {
        case "connected":  return "bg-success shadow-[0_0_6px_1px] shadow-success/60";
        case "connecting": return "bg-warning animate-pulse";
        case "error":      return "bg-error";
        default:           return "bg-base-content/20";
    }
});

const isOwnMessage = (u: string) => u === resolvedUsername.value;

const roleBadgeClass = (role: string) => {
    if (role === "admin")     return "badge badge-error badge-xs ml-1";
    if (role === "moderator") return "badge badge-warning badge-xs ml-1";
    return "";
};

// ── expand/collapse animation hooks ──────────────────────────────────────
const onBeforeEnter = (el: Element) => {
    const e = el as HTMLElement;
    e.style.maxHeight = "0";
    e.style.opacity = "0";
};

const onEnter = (el: Element, done: () => void) => {
    const e = el as HTMLElement;
    // measure natural height
    e.style.maxHeight = "none";
    const h = e.scrollHeight;
    e.style.maxHeight = "0";
    e.style.opacity = "0";

    requestAnimationFrame(() => {
        requestAnimationFrame(() => {
            e.style.transition = "max-height 0.35s cubic-bezier(0.4,0,0.2,1), opacity 0.25s ease";
            e.style.maxHeight = h + "px";
            e.style.opacity = "1";
        });
    });

    const cleanup = () => {
        e.style.maxHeight = "none";
        e.style.transition = "";
        done();
    };
    e.addEventListener("transitionend", cleanup, { once: true });
};

const onLeave = (el: Element, done: () => void) => {
    const e = el as HTMLElement;
    e.style.maxHeight = e.scrollHeight + "px";
    e.style.opacity = "1";

    requestAnimationFrame(() => {
        requestAnimationFrame(() => {
            e.style.transition = "max-height 0.3s cubic-bezier(0.4,0,0.2,1), opacity 0.2s ease";
            e.style.maxHeight = "0";
            e.style.opacity = "0";
        });
    });

    e.addEventListener("transitionend", done, { once: true });
};

// ── lifecycle ─────────────────────────────────────────────────────────────
onMounted(async () => {
    await loadUsername();
    await connect(resolvedUsername.value, isAuthenticated.value ? localStorage.getItem("authToken") : null);
    if (!isCollapsed.value) scrollToBottom();
});
</script>

<template>
    <div class="chat-panel mb-4 rounded-xl border border-base-300 bg-base-200 overflow-hidden">

        <!-- ── Header ── -->
        <div
            class="flex items-center justify-between px-4 py-2.5 cursor-pointer select-none
                   hover:bg-base-300/40 active:bg-base-300/70 transition-colors duration-150"
            @click="toggleCollapse"
        >
            <div class="flex items-center gap-2.5 min-w-0 flex-1">
                <!-- Icon -->
                <MessageSquare
                    class="w-4 h-4 shrink-0 transition-colors duration-300"
                    :class="status === 'connected' ? 'text-primary' : 'text-base-content/40'"
                />

                <!-- Title always visible -->
                <span class="text-sm font-medium shrink-0">{{ t("chat.title") }}</span>

                <!-- Status dot -->
                <span class="w-2 h-2 rounded-full shrink-0 transition-all duration-300" :class="statusDotClass" />

                <!-- Online count -->
                <span
                    v-if="status === 'connected' && onlineCount > 0"
                    class="flex items-center gap-1 text-xs text-base-content/40 shrink-0"
                >
                    <Users class="w-3 h-3" />
                    {{ onlineCount }}
                </span>

                <!-- Last message preview (only when collapsed) -->
                <transition name="preview-switch" mode="out-in">
                    <div
                        v-if="isCollapsed && messages.length > 0"
                        key="preview"
                        class="flex items-center gap-1.5 min-w-0 ml-1 pl-2 border-l border-base-300"
                    >
                        <span class="text-xs font-semibold text-base-content/50 shrink-0">
                            {{ messages[messages.length - 1].username }}:
                        </span>
                        <span class="text-xs text-base-content/40 truncate">
                            {{ messages[messages.length - 1].content }}
                        </span>
                    </div>
                </transition>
            </div>

            <div class="flex items-center gap-2 shrink-0">
                <Loader2 v-if="status === 'connecting'" class="w-3.5 h-3.5 animate-spin text-warning" />
                <Wifi     v-else-if="status === 'connected'"   class="w-3.5 h-3.5 text-success" />
                <WifiOff  v-else class="w-3.5 h-3.5 text-base-content/25" />

                <!-- Chevron with rotation animation -->
                <ChevronDown
                    class="w-4 h-4 text-base-content/40 transition-transform duration-300"
                    :class="{ 'rotate-180': !isCollapsed }"
                />
            </div>
        </div>

        <!-- ── Body with JS-driven expand/collapse animation ── -->
        <Transition
            :css="false"
            @before-enter="onBeforeEnter"
            @enter="onEnter"
            @leave="onLeave"
        >
            <div v-if="!isCollapsed" class="overflow-hidden">

                <!-- Messages area -->
                <div
                    ref="messagesContainerRef"
                    class="h-48 overflow-y-auto px-3 py-2 space-y-2 border-t border-base-300/50
                           scrollbar-thin scrollbar-thumb-base-300"
                >
                    <!-- Loading skeleton -->
                    <div v-if="isLoading" class="space-y-2 pt-2">
                        <div v-for="i in 4" :key="i" class="flex items-start gap-2 animate-pulse">
                            <div class="w-6 h-6 rounded-full bg-base-300 shrink-0" />
                            <div class="flex-1 space-y-1">
                                <div class="h-2.5 bg-base-300 rounded w-16" />
                                <div class="h-2.5 bg-base-300 rounded" :style="{ width: 30 + i * 12 + '%' }" />
                            </div>
                        </div>
                    </div>

                    <!-- Empty state -->
                    <div
                        v-else-if="messages.length === 0"
                        class="flex flex-col items-center justify-center h-full text-base-content/25 text-xs gap-1"
                    >
                        <MessageSquare class="w-7 h-7 opacity-30" />
                        {{ t("chat.no_messages") }}
                    </div>

                    <!-- Message list -->
                    <template v-else>
                        <div
                            v-for="msg in messages"
                            :key="msg.id"
                            class="flex items-end gap-2 msg-row"
                            :class="{ 'flex-row-reverse': isOwnMessage(msg.username) }"
                        >
                            <UserAvatar :name="msg.username" size="xs" class="shrink-0" />

                            <div
                                class="flex flex-col max-w-[78%]"
                                :class="isOwnMessage(msg.username) ? 'items-end' : 'items-start'"
                            >
                                <!-- Meta row -->
                                <div
                                    class="flex items-center gap-1 mb-0.5 px-0.5"
                                    :class="isOwnMessage(msg.username) ? 'flex-row-reverse' : 'flex-row'"
                                >
                                    <span class="text-[10px] font-semibold text-base-content/50 truncate max-w-[100px]">
                                        {{ msg.username }}
                                    </span>
                                    <span v-if="msg.role !== 'user'" :class="roleBadgeClass(msg.role)">
                                        {{ msg.role }}
                                    </span>
                                    <span class="text-[9px] text-base-content/20">{{ msg.time }}</span>
                                </div>

                                <!-- Bubble -->
                                <div
                                    class="px-2.5 py-1.5 text-xs break-words leading-relaxed
                                           shadow-sm transition-all duration-150"
                                    :class="
                                        isOwnMessage(msg.username)
                                            ? 'bg-primary text-primary-content rounded-2xl rounded-br-sm'
                                            : 'bg-base-300 text-base-content rounded-2xl rounded-bl-sm'
                                    "
                                >
                                    {{ msg.content }}
                                </div>
                            </div>
                        </div>
                    </template>
                </div>

                <!-- Input row -->
                <div class="px-3 py-2 border-t border-base-300/50">
                    <div
                        class="flex items-center gap-2 bg-base-100 rounded-lg px-3 py-1.5
                               border border-base-300 transition-colors duration-150
                               focus-within:border-primary/50"
                        :class="{ '!border-error': isOverLimit }"
                    >
                        <input
                            ref="inputRef"
                            v-model="inputText"
                            type="text"
                            class="flex-1 bg-transparent text-xs outline-none
                                   placeholder:text-base-content/25 min-w-0"
                            :placeholder="
                                status === 'connected'
                                    ? t('chat.input_placeholder', { username: resolvedUsername })
                                    : t('chat.connecting_placeholder')
                            "
                            :disabled="status !== 'connected' || isSending"
                            @keydown="handleKeydown"
                            maxlength="520"
                        />

                        <span
                            v-if="charCount > 400"
                            class="text-[10px] shrink-0 transition-colors"
                            :class="isOverLimit ? 'text-error' : 'text-base-content/25'"
                        >
                            {{ charCount }}/{{ MESSAGE_MAX_LENGTH }}
                        </span>

                        <button
                            class="btn btn-primary btn-xs shrink-0 transition-all duration-150"
                            :disabled="!inputText.trim() || status !== 'connected' || isSending || isOverLimit"
                            @click="handleSend"
                        >
                            <Loader2 v-if="isSending" class="w-3 h-3 animate-spin" />
                            <Send v-else class="w-3 h-3" />
                        </button>
                    </div>
                </div>

            </div>
        </Transition>
    </div>
</template>

<style scoped>
/* Fade for online count badge */
.fade-count-enter-active,
.fade-count-leave-active {
    transition: opacity 0.3s ease, transform 0.3s ease;
}
.fade-count-enter-from,
.fade-count-leave-to {
    opacity: 0;
    transform: translateX(-4px);
}

/* Preview ↔ title switch */
.preview-switch-enter-active,
.preview-switch-leave-active {
    transition: opacity 0.2s ease, transform 0.2s ease;
}
.preview-switch-enter-from {
    opacity: 0;
    transform: translateY(4px);
}
.preview-switch-leave-to {
    opacity: 0;
    transform: translateY(-4px);
}

/* Message row pop-in */
.msg-row {
    animation: msgIn 0.2s ease-out both;
}
@keyframes msgIn {
    from { opacity: 0; transform: translateY(6px); }
    to   { opacity: 1; transform: translateY(0); }
}

/* Thin scrollbar */
.scrollbar-thin {
    scrollbar-width: thin;
    scrollbar-color: hsl(var(--b3)) transparent;
}
.scrollbar-thin::-webkit-scrollbar { width: 4px; }
.scrollbar-thin::-webkit-scrollbar-track { background: transparent; }
.scrollbar-thin::-webkit-scrollbar-thumb {
    background: hsl(var(--b3));
    border-radius: 2px;
}
</style>

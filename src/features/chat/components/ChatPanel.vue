<script setup lang="ts">
import { ref, computed, onMounted, nextTick, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { sendNotification } from "@tauri-apps/plugin-notification";
import { useI18n } from "vue-i18n";
import {
    Send,
    Users,
    Wifi,
    WifiOff,
    Loader2,
    MessageSquare,
    ChevronDown,
    CornerUpLeft,
    X,
} from "@lucide/vue";
import {
    useChatService,
    MESSAGE_MAX_LENGTH,
    type ChatMessage,
    type ChatReplyTarget,
} from "@services/chat/useChatService";
import { useToast } from "@shared/composables/useToast";
import UserAvatar from "@shared/components/ui/UserAvatar.vue";
import notificationSound from "@/assets/misc/notification.mp3";
import { useUser } from "@features/auth/useUser";
import { achievementService } from "@features/social/achievementService";

const { t } = useI18n();
const { addToast } = useToast();
const { displayName, username, isAuthenticated } = useUser();

const { messages, status, onlineCount, isLoading, connect, sendMessage } =
    useChatService();

const resolvedUsername = ref("Guest");

const loadUsername = async () => {
    if (isAuthenticated.value && (displayName.value || username.value)) {
        resolvedUsername.value = displayName.value || username.value;
        return;
    }
    try {
        interface Account {
            id: string;
            username: string;
            is_active: boolean;
        }
        const accounts = await invoke<Account[]>("get_accounts");
        const active = accounts.find((a) => a.is_active) ?? accounts[0];
        if (active?.username) {
            resolvedUsername.value = active.username;
            return;
        }
    } catch {}
    resolvedUsername.value = "Guest";
};

watch([isAuthenticated, displayName, username], loadUsername);

const isCollapsed = ref(true);

const toggleCollapse = () => {
    isCollapsed.value = !isCollapsed.value;
    if (!isCollapsed.value) nextTick(() => scrollToBottom());
};

const inputText = ref("");
const isSending = ref(false);
const messagesContainerRef = ref<HTMLElement | null>(null);
const inputRef = ref<HTMLInputElement | null>(null);
const replyTarget = ref<ChatReplyTarget | null>(null);

const charCount = computed(() => inputText.value.length);
const isOverLimit = computed(() => charCount.value > MESSAGE_MAX_LENGTH);

const scrollToBottom = () => {
    nextTick(() => {
        if (messagesContainerRef.value) {
            messagesContainerRef.value.scrollTop =
                messagesContainerRef.value.scrollHeight;
        }
    });
};

const escapeRegExp = (value: string) =>
    value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");

const mentionNames = computed(() => {
    const names = new Set<string>();
    const addName = (value: string | undefined | null) => {
        const normalized = value?.trim();
        if (normalized) names.add(normalized);
    };

    addName(resolvedUsername.value);
    addName(displayName.value);
    addName(username.value);

    return Array.from(names);
});

const isMentioned = (message: ChatMessage) => {
    const normalizedContent = message.content || "";
    return mentionNames.value.some((name) => {
        const mentionRegex = new RegExp(
            `(^|\\s)@${escapeRegExp(name)}(?=$|\\s|[.,!?])`,
            "i"
        );
        return mentionRegex.test(normalizedContent);
    });
};

const playMentionSound = () => {
    new Audio(notificationSound)
        .play()
        .catch((error) =>
            console.error("Failed to play mention sound:", error)
        );
};

const showMentionNotification = (message: ChatMessage) => {
    const title = t("chat.notification.mention_title", {
        user: message.username,
    });
    const body = t("chat.notification.mention_body", {
        content: message.content,
    });

    try {
        sendNotification({ title, body });
    } catch (error) {
        console.error("System notification failed:", error);
        addToast(title, "info", 8000);
    }
};

watch(
    () => messages.value.length,
    () => {
        if (!isCollapsed.value) scrollToBottom();
    }
);

watch(
    messages,
    (newMessages, oldMessages) => {
        if (!oldMessages || oldMessages.length === 0) return;
        if (newMessages.length <= oldMessages.length) return;

        const addedMessages = newMessages.slice(oldMessages.length);
        const mention = addedMessages.find(
            (msg) => msg.username !== resolvedUsername.value && isMentioned(msg)
        );

        if (!mention) return;

        playMentionSound();
        void showMentionNotification(mention);
    },
    { deep: false }
);

const handleSend = async () => {
    const text = inputText.value.trim();
    if (!text || isSending.value || isOverLimit.value) return;
    isSending.value = true;
    try {
        const userId = localStorage.getItem("authToken");
        await sendMessage(
            text,
            resolvedUsername.value,
            userId,
            "user",
            replyTarget.value
        );
        inputText.value = "";
        replyTarget.value = null;
        inputRef.value?.focus();
        void achievementService.unlockAchievement("CHAT_FIRST_MESSAGE");
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
    if (e.key === "Enter" && !e.shiftKey) {
        e.preventDefault();
        handleSend();
    }
};

const statusDotClass = computed(() => {
    switch (status.value) {
        case "connected":
            return "bg-success shadow-[0_0_6px_1px] shadow-success/60";
        case "connecting":
            return "bg-warning animate-pulse";
        case "error":
            return "bg-error";
        default:
            return "bg-base-content/20";
    }
});

const isOwnMessage = (u: string) => u === resolvedUsername.value;

const startReply = (message: ChatMessage) => {
    replyTarget.value = {
        id: message.id,
        username: message.username,
    };
    inputRef.value?.focus();
};

const clearReplyTarget = () => {
    replyTarget.value = null;
};

const getReplyPreview = (message: ChatMessage) => {
    if (!message.replyToId) return null;

    const sourceMessage = messages.value.find(
        (item) => item.id === message.replyToId
    );

    return {
        username: sourceMessage?.username ?? "Unknown",
        content: sourceMessage?.content ?? null,
    };
};

const scrollToMessage = (id?: number | null) => {
    if (!id) return;
    nextTick(() => {
        const el = document.querySelector(`[data-msg-id=\"${id}\"]`);
        if (el instanceof HTMLElement) {
            el.scrollIntoView({
                behavior: "smooth",
                block: "center",
            });
            el.classList.remove("reply-highlight", "reply-highlight-fade");
            void el.offsetWidth;
            el.classList.add("reply-highlight");
            window.setTimeout(
                () => el.classList.add("reply-highlight-fade"),
                400
            );
            window.setTimeout(
                () =>
                    el.classList.remove(
                        "reply-highlight",
                        "reply-highlight-fade"
                    ),
                5200
            );
        }
    });
};

const roleBadgeClass = (role: string) => {
    if (role === "admin") return "badge badge-error badge-xs ml-1";
    if (role === "moderator") return "badge badge-warning badge-xs ml-1";
    return "";
};

const getMessageDay = (message: ChatMessage) => {
    const date = new Date(message.created_at);
    if (Number.isNaN(date.getTime())) return "";
    return date.toLocaleDateString(undefined, {
        weekday: "short",
        day: "2-digit",
        month: "2-digit",
    });
};

const onBeforeEnter = (el: Element) => {
    const e = el as HTMLElement;
    e.style.maxHeight = "0";
    e.style.opacity = "0";
};

const onEnter = (el: Element, done: () => void) => {
    const e = el as HTMLElement;
    e.style.maxHeight = "none";
    const h = e.scrollHeight;
    e.style.maxHeight = "0";
    e.style.opacity = "0";

    requestAnimationFrame(() => {
        requestAnimationFrame(() => {
            e.style.transition =
                "max-height 0.35s cubic-bezier(0.4,0,0.2,1), opacity 0.25s ease";
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
            e.style.transition =
                "max-height 0.3s cubic-bezier(0.4,0,0.2,1), opacity 0.2s ease";
            e.style.maxHeight = "0";
            e.style.opacity = "0";
        });
    });

    e.addEventListener("transitionend", done, { once: true });
};

onMounted(async () => {
    await loadUsername();
    await connect(
        resolvedUsername.value,
        isAuthenticated.value ? localStorage.getItem("authToken") : null
    );
    if (!isCollapsed.value) scrollToBottom();
});
</script>

<template>
    <div
        class="chat-panel mb-4 rounded-xl border border-base-300 bg-base-200 overflow-hidden"
    >
        <div
            class="flex items-center justify-between px-4 py-2.5 cursor-pointer select-none hover:bg-base-300/40 active:bg-base-300/70 transition-colors duration-150"
            @click="toggleCollapse"
        >
            <div class="flex items-center gap-2.5 min-w-0 flex-1">
                <MessageSquare
                    class="w-4 h-4 shrink-0 transition-colors duration-300"
                    :class="
                        status === 'connected'
                            ? 'text-primary'
                            : 'text-base-content/40'
                    "
                />

                <span class="text-sm font-medium shrink-0">{{
                    t("chat.title")
                }}</span>

                <span
                    class="w-2 h-2 rounded-full shrink-0 transition-all duration-300"
                    :class="statusDotClass"
                />

                <span
                    v-if="status === 'connected' && onlineCount > 0"
                    class="flex items-center gap-1 text-xs text-base-content/40 shrink-0"
                >
                    <Users class="w-3 h-3" />
                    {{ onlineCount }}
                </span>

                <transition name="preview-switch" mode="out-in">
                    <div
                        v-if="isCollapsed && messages.length > 0"
                        key="preview"
                        class="flex items-center gap-1.5 min-w-0 ml-1 pl-2 border-l border-base-300"
                    >
                        <span
                            class="text-xs font-semibold text-base-content/50 shrink-0"
                        >
                            {{ messages[messages.length - 1].username }}:
                        </span>
                        <span class="text-xs text-base-content/40 truncate">
                            {{ messages[messages.length - 1].content }}
                        </span>
                    </div>
                </transition>
            </div>

            <div class="flex items-center gap-2 shrink-0">
                <Loader2
                    v-if="status === 'connecting'"
                    class="w-3.5 h-3.5 animate-spin text-warning"
                />
                <Wifi
                    v-else-if="status === 'connected'"
                    class="w-3.5 h-3.5 text-success"
                />
                <WifiOff v-else class="w-3.5 h-3.5 text-base-content/25" />

                <ChevronDown
                    class="w-4 h-4 text-base-content/40 transition-transform duration-300"
                    :class="{ 'rotate-180': !isCollapsed }"
                />
            </div>
        </div>

        <Transition
            :css="false"
            @before-enter="onBeforeEnter"
            @enter="onEnter"
            @leave="onLeave"
        >
            <div v-if="!isCollapsed" class="overflow-hidden">
                <div
                    ref="messagesContainerRef"
                    class="h-48 overflow-y-auto px-3 py-2 space-y-2 border-t border-base-300/50 scrollbar-thin scrollbar-thumb-base-300"
                >
                    <div v-if="isLoading" class="space-y-2 pt-2">
                        <div
                            v-for="i in 4"
                            :key="i"
                            class="flex items-start gap-2 animate-pulse"
                        >
                            <div
                                class="w-6 h-6 rounded-full bg-base-300 shrink-0"
                            />
                            <div class="flex-1 space-y-1">
                                <div class="h-2.5 bg-base-300 rounded w-16" />
                                <div
                                    class="h-2.5 bg-base-300 rounded"
                                    :style="{ width: 30 + i * 12 + '%' }"
                                />
                            </div>
                        </div>
                    </div>

                    <div
                        v-else-if="messages.length === 0"
                        class="flex flex-col items-center justify-center h-full text-base-content/25 text-xs gap-1"
                    >
                        <MessageSquare class="w-7 h-7 opacity-30" />
                        {{ t("chat.no_messages") }}
                    </div>

                    <template v-else>
                        <div
                            v-for="msg in messages"
                            :key="msg.id"
                            :data-msg-id="msg.id"
                            class="flex items-end gap-2 msg-row group"
                            :class="{
                                'flex-row-reverse': isOwnMessage(msg.username),
                            }"
                        >
                            <UserAvatar
                                :name="msg.username"
                                size="sm"
                                class="shrink-0"
                            />

                            <div
                                class="flex flex-col max-w-[78%]"
                                :class="
                                    isOwnMessage(msg.username)
                                        ? 'items-end'
                                        : 'items-start'
                                "
                            >
                                <div
                                    class="flex items-center gap-1 mb-0.5 px-0.5"
                                    :class="
                                        isOwnMessage(msg.username)
                                            ? 'flex-row-reverse'
                                            : 'flex-row'
                                    "
                                >
                                    <span
                                        class="text-[10px] font-semibold text-base-content/50 truncate max-w-25"
                                    >
                                        {{ msg.username }}
                                    </span>
                                    <span
                                        v-if="msg.role !== 'user'"
                                        :class="roleBadgeClass(msg.role)"
                                    >
                                        {{ msg.role }}
                                    </span>
                                    <span
                                        class="text-[9px] text-base-content/20"
                                    >
                                        {{ msg.time }}
                                        <span
                                            class="ml-1 text-[9px] text-base-content/30"
                                            v-if="msg.created_at"
                                        >
                                            {{ getMessageDay(msg) }}
                                        </span>
                                    </span>
                                    <button
                                        class="btn btn-ghost btn-xs h-4 min-h-4 px-1 opacity-0 group-hover:opacity-100 transition-opacity"
                                        @click="startReply(msg)"
                                        type="button"
                                        aria-label="Reply to message"
                                    >
                                        <CornerUpLeft class="w-2.5 h-2.5" />
                                    </button>
                                </div>

                                <div
                                    v-if="getReplyPreview(msg)"
                                    class="mb-1 w-full max-w-full rounded-lg border border-base-300/70 bg-base-100/80 px-2 py-1 text-[10px] text-base-content/60"
                                >
                                    <button
                                        class="w-full text-left"
                                        type="button"
                                        @click="scrollToMessage(msg.replyToId)"
                                    >
                                        <div
                                            class="font-medium text-base-content/70"
                                        >
                                            @{{
                                                getReplyPreview(msg)?.username
                                            }}
                                        </div>
                                        <div
                                            class="truncate text-base-content/45"
                                        >
                                            {{
                                                getReplyPreview(msg)?.content ??
                                                t("chat.reply_unknown")
                                            }}
                                        </div>
                                    </button>
                                </div>

                                <div
                                    class="px-2.5 py-1.5 text-xs wrap-break-word leading-relaxed shadow-sm transition-all duration-150"
                                    :class="
                                        isOwnMessage(msg.username)
                                            ? 'bg-primary text-primary-content rounded-2xl rounded-br-sm'
                                            : 'bg-base-300 text-base-content rounded-2xl rounded-bl-sm'
                                    "
                                    :data-msg-id="msg.id"
                                >
                                    {{ msg.content }}
                                </div>
                            </div>
                        </div>
                    </template>
                </div>

                <div class="px-3 py-2 border-t border-base-300/50">
                    <div
                        v-if="replyTarget"
                        class="mb-2 flex items-center justify-between gap-3 rounded-lg border border-base-300 bg-base-100/80 px-2.5 py-1.5 text-[10px]"
                    >
                        <div
                            class="min-w-0 flex items-center gap-1.5 text-base-content/70"
                        >
                            <CornerUpLeft
                                class="w-3 h-3 shrink-0 text-primary"
                            />
                            <span class="truncate">
                                Replying to @{{ replyTarget.username }}
                            </span>
                        </div>
                        <button
                            class="btn btn-ghost btn-xs h-5 min-h-5 px-1.5"
                            type="button"
                            @click="clearReplyTarget"
                        >
                            <X class="w-3 h-3" />
                        </button>
                    </div>

                    <div
                        class="flex items-center gap-2 bg-base-100 rounded-lg px-3 py-1.5 border border-base-300 transition-colors duration-150 focus-within:border-primary/50"
                        :class="{ 'border-error!': isOverLimit }"
                    >
                        <input
                            ref="inputRef"
                            v-model="inputText"
                            type="text"
                            class="flex-1 bg-transparent text-xs outline-none placeholder:text-base-content/25 min-w-0"
                            :placeholder="
                                status === 'connected'
                                    ? t('chat.input_placeholder', {
                                          username: resolvedUsername,
                                      })
                                    : t('chat.connecting_placeholder')
                            "
                            :disabled="status !== 'connected' || isSending"
                            @keydown="handleKeydown"
                            maxlength="520"
                        />

                        <span
                            v-if="charCount > 400"
                            class="text-[10px] shrink-0 transition-colors"
                            :class="
                                isOverLimit
                                    ? 'text-error'
                                    : 'text-base-content/25'
                            "
                        >
                            {{ charCount }}/{{ MESSAGE_MAX_LENGTH }}
                        </span>

                        <button
                            class="btn btn-primary btn-xs shrink-0 transition-all duration-150"
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
                                class="w-3 h-3 animate-spin"
                            />
                            <Send v-else class="w-3 h-3" />
                        </button>
                    </div>
                </div>
            </div>
        </Transition>
    </div>
</template>

<style scoped>
.fade-count-enter-active,
.fade-count-leave-active {
    transition:
        opacity 0.3s ease,
        transform 0.3s ease;
}
.fade-count-enter-from,
.fade-count-leave-to {
    opacity: 0;
    transform: translateX(-4px);
}

.preview-switch-enter-active,
.preview-switch-leave-active {
    transition:
        opacity 0.2s ease,
        transform 0.2s ease;
}
.preview-switch-enter-from {
    opacity: 0;
    transform: translateY(4px);
}
.preview-switch-leave-to {
    opacity: 0;
    transform: translateY(-4px);
}

.msg-row {
    animation: msgIn 0.2s ease-out both;
}
@keyframes msgIn {
    from {
        opacity: 0;
        transform: translateY(6px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.reply-highlight {
    background-color: rgba(96, 165, 250, 0.28);
    transition: background-color 1.2s ease-out;
}

.reply-highlight.reply-highlight-fade {
    background-color: transparent;
}

.scrollbar-thin {
    scrollbar-width: thin;
    scrollbar-color: hsl(var(--b3)) transparent;
}
.scrollbar-thin::-webkit-scrollbar {
    width: 4px;
}
.scrollbar-thin::-webkit-scrollbar-track {
    background: transparent;
}
.scrollbar-thin::-webkit-scrollbar-thumb {
    background: hsl(var(--b3));
    border-radius: 2px;
}
</style>

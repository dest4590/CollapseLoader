<template>
    <div v-bind="attrs"
        class="irc-chat flex flex-col bg-base-200 border border-base-300 rounded-lg overflow-hidden transition-all duration-300 ease-out relative mb-6"
        :class="isExpanded ? 'shadow-lg max-h-[380px]' : 'shadow-sm max-h-[68px] hover:shadow-md'">
        <button type="button" class="flex items-center justify-between w-full px-4 py-3 bg-base-300/40 cursor-pointer"
            @click="toggleExpanded">
            <div class="flex items-center gap-3">
                <MessageSquare class="w-5 h-5" />
                <div class="flex flex-col text-left leading-tight">
                    <span class="font-semibold text-sm">{{ $t('irc.inline.title') }}</span>
                    <span class="text-xs text-base-content/70 whitespace-nowrap"
                        :class="{ 'w-[70%] truncate overflow-hidden': latestActivity.length > 100 }">
                        {{ latestActivity }}
                    </span>
                </div>
            </div>
            <div class="flex items-center gap-3">
                <button class="p-1 rounded hover:bg-base-300/50 cursor-pointer" @click.stop="forceReconnect">
                    <RefreshCw class="w-4 h-4" />
                </button>

                <ChevronDown class="w-4 h-4 transition-transform duration-200 m-2"
                    :class="isExpanded ? 'rotate-180' : ''" />
            </div>
        </button>

        <div class="overflow-hidden transition-all duration-300 ease-out"
            :class="isExpanded ? 'max-h-80 opacity-100' : 'max-h-0 opacity-0'">
            <div class="flex flex-col h-[280px] bg-base-100/40">
                <div class="flex justify-between ">
                    <transition name="irc-status">
                        <div v-if="isExpanded"
                            class="p-4 flex items-center gap-2 text-xs font-semibold pointer-events-none select-none"
                            :class="statusMeta.textClass">
                            <component :is="statusMeta.icon" class="w-4 h-4" :class="statusMeta.iconClass" />
                            <span>{{ statusMeta.label }}</span>
                        </div>
                    </transition>
                </div>

                <div class="flex-1 overflow-y-auto px-4 pb-4 space-y-2" ref="messagesContainer">
                    <div v-for="(msg, index) in visibleMessages" :key="index"
                        class="text-sm wrap-break-word whitespace-pre-wrap">
                        <span class="opacity-70 mr-2">[{{ msg.time }}]</span>
                        <span v-for="(part, pIndex) in parseMessage(msg.content, msg.type)" :key="pIndex"
                            :style="{ color: part.color }"
                            :class="[{ 'cursor-pointer underline': part.isName, 'font-bold': part.bold }]"
                            @contextmenu.prevent="part.isName && openContextMenu($event, part.text, extractDisplayName(msg.content))">{{
                                part.text }}</span>
                    </div>
                </div>

                <transition name="irc-input">
                    <div v-show="isExpanded" class="p-4 bg-base-200 border-t border-base-300 flex gap-2">
                        <input ref="ircInput" v-model="inputMessage" @keyup.enter="sendMessage" type="text"
                            :placeholder="$t('irc.inline.placeholder')" class="input input-bordered flex-1"
                            :disabled="!connected" />
                        <button class="btn btn-primary" @click="sendMessage"
                            :disabled="!connected || !inputMessage.trim()">
                            {{ $t('common.send') }}
                        </button>
                    </div>
                </transition>
            </div>
        </div>
    </div>
    <div v-if="contextMenu.visible" class="fixed z-50"
        :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }" @click.stop>
        <div class="bg-base-100 border border-base-300 rounded shadow-md p-2 flex flex-col context-menu">
            <button class="text-left px-2 py-1 hover:bg-base-200 rounded flex items-center gap-2"
                @click.prevent.stop="startPrivateMessage">
                <AtSign class="w-4 h-4" />
                <span>{{ $t('irc.inline.privateMessage') }}</span>
            </button>
            <button class="text-left px-2 py-1 hover:bg-base-200 rounded flex items-center gap-2"
                @click.prevent.stop="copyToClipboard(contextMenu.username)">
                <Copy class="w-4 h-4" />
                <span>{{ $t('irc.inline.copyNickname') }}</span>
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, nextTick, onMounted, ref, watch, onUnmounted, useAttrs } from 'vue';
defineOptions({ inheritAttrs: false });
import { CheckCircle2, ChevronDown, Loader2, MessageSquare, RefreshCw, WifiOff, AtSign, Copy } from 'lucide-vue-next';
import { useToast } from '../../../services/toastService';
import { useIrcChat } from '../../../composables/useIrcChat';
import { useI18n } from 'vue-i18n';

const attrs = useAttrs();
const { messages, connected, status, sendIrcMessage, forceReconnect, ensureIrcConnection } = useIrcChat();
const { t } = useI18n();
const inputMessage = ref('');
const ircInput = ref<HTMLInputElement | null>(null);
const contextMenu = ref({ visible: false, x: 0, y: 0, username: '', displayName: '' });
let clickHandlerRef: (() => void) | null = null;
let keydownHandlerRef: ((e: KeyboardEvent) => void) | null = null;
const isExpanded = ref(false);
const messagesContainer = ref<HTMLElement | null>(null);
const { addToast } = useToast();

const parseMessage = (msg: string, type?: string) => {
    const colorMap: Record<string, string> = {
        '0': '#000000', '1': '#0000AA', '2': '#00AA00', '3': '#00AAAA',
        '4': '#AA0000', '5': '#AA00AA', '6': '#FFAA00', '7': '#AAAAAA',
        '8': '#555555', '9': '#5555FF', 'a': '#55FF55', 'b': '#55FFFF',
        'c': '#FF5555', 'd': '#FF55FF', 'e': '#FFFF55', 'f': '#FFFFFF'
    };

    const parts: { text: string; color?: string; isName?: boolean; bold?: boolean }[] = [];
    let currentColor: string | undefined = undefined;
    let currentBold = false;

    const regex = /§([0-9a-frklmnor])/gi;
    let lastIndex = 0;
    let match;

    const colonIndex = msg.indexOf(':');
    const headerEnd = colonIndex === -1 ? -1 : colonIndex;
    const headerRaw = colonIndex === -1 ? '' : msg.substring(0, colonIndex);
    const headerStripped = stripColorCodes(headerRaw).trim();
    const headerStrippedLower = headerStripped.toLowerCase();
    const headerFirstToken = headerStripped.split(/\s+/)[0] || '';
    const headerLooksLikeNick = (headerStripped.includes('(') || headerStripped.includes('[') || /^[A-Za-z0-9_\-]+$/.test(headerFirstToken))
        && !headerStrippedLower.startsWith('@') && !headerStrippedLower.includes('profile') && !/\b(id|ip|name):?/i.test(headerStripped);
    let namePartMarked = false;

    while ((match = regex.exec(msg)) !== null) {
        const text = msg.substring(lastIndex, match.index);
        if (text) {
            const isInHeader = headerEnd !== -1 && match.index <= headerEnd && lastIndex < headerEnd;
            const isName = isInHeader && !namePartMarked && text.trim().length > 0 && type !== 'system' && headerStrippedLower !== 'system' && headerLooksLikeNick;
            if (isName) namePartMarked = true;
            parts.push({ text, color: currentColor, isName: !!isName, bold: currentBold });
        }

        const code = match[1].toLowerCase();
        if (code === 'r') {
            currentColor = undefined;
            currentBold = false;
        } else if (code === 'l') {
            currentBold = true;
        } else if (colorMap[code]) {
            currentColor = colorMap[code];
        }

        lastIndex = regex.lastIndex;
    }

    const remaining = msg.substring(lastIndex);
    if (remaining) {
        const isInHeader = headerEnd !== -1 && lastIndex < headerEnd;
        const isName = isInHeader && !namePartMarked && remaining.trim().length > 0 && type !== 'system' && headerStrippedLower !== 'system' && headerLooksLikeNick;
        parts.push({ text: remaining, color: currentColor, isName: !!isName, bold: currentBold });
    }

    return parts;
};

const stripColorCodes = (text: string) => text.replace(/§[0-9a-frklmnor]/gi, '');

const extractDisplayName = (content: string) => {
    const idx = content.indexOf(':');
    const header = idx === -1 ? content : content.substring(0, idx);
    return stripColorCodes(header).trim();
};

const visibleMessages = computed(() => messages.value.filter(m => m && m.type !== 'ping'));

const latestActivity = computed(() => {
    const last = visibleMessages.value[visibleMessages.value.length - 1];
    if (!last) {
        return connected.value ? t('irc.inline.latest_activity_connected') : t('irc.inline.tap_to_connect');
    }
    return `${last.content.replace(/§[0-9a-f]|§r/gi, '')}`;
});

const statusMeta = computed(() => {
    switch (status.value) {
        case 'connected':
            return { label: t('irc.inline.status.connected'), textClass: 'text-success', icon: CheckCircle2, iconClass: 'text-success' };
        case 'reconnecting':
            return { label: t('irc.inline.status.reconnecting'), textClass: 'text-warning', icon: RefreshCw, iconClass: 'animate-spin-slow text-warning' };
        case 'connecting':
            return { label: t('irc.inline.status.connecting'), textClass: 'text-info', icon: Loader2, iconClass: 'animate-spin-slow text-info' };
        case 'error':
            return { label: t('irc.inline.status.error'), textClass: 'text-error', icon: WifiOff, iconClass: 'text-error' };
        default:
            return { label: t('irc.inline.status.disconnected'), textClass: 'text-base-content/70', icon: WifiOff, iconClass: 'text-base-content/60' };
    }
});

const scrollToBottom = async () => {
    await nextTick();
    if (messagesContainer.value) {
        messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight;
    }
};

const toggleExpanded = () => {
    isExpanded.value = !isExpanded.value;
    if (isExpanded.value) {
        scrollToBottom();
    }
};

const sendMessage = async () => {
    if (!inputMessage.value.trim()) return;

    try {
        await ensureIrcConnection();
        await sendIrcMessage(inputMessage.value);
        inputMessage.value = '';
    } catch (err) {
        console.error('Failed to send message:', err);
        addToast(
            t('irc.inline.send_failed'),
            'error',
        );
    }
};

const openContextMenu = (e: MouseEvent, username: string, displayName: string) => {
    e.preventDefault();
    contextMenu.value.visible = true;
    contextMenu.value.x = e.clientX;
    contextMenu.value.y = e.clientY;
    contextMenu.value.username = username;
    contextMenu.value.displayName = displayName;
};

const closeContextMenu = () => {
    contextMenu.value.visible = false;
    contextMenu.value.username = '';
    contextMenu.value.displayName = '';
};

const startPrivateMessage = async () => {
    if (!contextMenu.value.username) return;
    const name = contextMenu.value.username.trim();
    isExpanded.value = true;
    inputMessage.value = `@msg ${name} `;
    await nextTick();
    ircInput.value?.focus();
    closeContextMenu();
};

const copyToClipboard = async (text: string) => {
    try {
        await navigator.clipboard.writeText(text);
        addToast('Copied to clipboard', 'success');
    } catch {
        addToast('Failed to copy', 'error');
    }
    closeContextMenu();
};

onMounted(async () => {
    try {
        await ensureIrcConnection();
        scrollToBottom();
    } catch (err) {
        console.error('Failed to connect to IRC:', err);
        addToast(
            t('irc.inline.connection_error', { error: String(err) }),
            'error',
        );
    }
    clickHandlerRef = () => { if (contextMenu.value.visible) closeContextMenu(); };
    keydownHandlerRef = (e: KeyboardEvent) => { if (e.key === 'Escape' && contextMenu.value.visible) closeContextMenu(); };
    document.addEventListener('click', clickHandlerRef);
    window.addEventListener('keydown', keydownHandlerRef);
});

onUnmounted(() => {
    if (clickHandlerRef) document.removeEventListener('click', clickHandlerRef);
    if (keydownHandlerRef) window.removeEventListener('keydown', keydownHandlerRef as EventListener);
});

watch(() => visibleMessages.value.length, () => {
    scrollToBottom();
});
</script>

<style scoped>
.irc-input-enter-active,
.irc-input-leave-active {
    transition: opacity 0.2s ease, transform 0.2s ease;
}

.irc-input-enter-from,
.irc-input-leave-to {
    opacity: 0;
    transform: translateY(6px);
}

.irc-status-enter-active,
.irc-status-leave-active {
    transition: opacity 0.2s ease, transform 0.2s ease;
}

.irc-status-enter-from,
.irc-status-leave-to {
    opacity: 0;
    transform: translateY(-4px);
}

.animate-spin-slow {
    animation: spin 1.6s linear infinite;
}

.context-menu {
    min-width: 160px;
}

.context-menu button {
    cursor: pointer;
}
</style>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { settingsService } from "@services/settings/settingsService";
import {
    Home,
    Settings,
    Palette,
    Info,
    User,
    UserCheck,
    Newspaper,
    FileText,
    Search,
    Zap,
    ChevronRight,
    CheckCircle,
    Store,
} from "lucide-vue-next";
import type { Client } from "@shared/types/ui";

interface Account {
    id: string;
    username: string;
    tags: string[];
    is_active: boolean;
}

const { t } = useI18n();

const props = defineProps<{ show: boolean }>();
const emit = defineEmits<{
    (e: "close"): void;
    (e: "navigate", tab: string): void;
}>();

const query = ref("");
const selectedIndex = ref(0);
const inputRef = ref<HTMLInputElement | null>(null);
const resultsListRef = ref<HTMLElement | null>(null);
const clients = ref<Client[]>([]);
const accounts = ref<Account[]>([]);

const tabItems = [
    { id: "home", icon: Home, labelKey: "navigation.home" },
    { id: "settings", icon: Settings, labelKey: "navigation.settings" },
    {
        id: "customization",
        icon: Palette,
        labelKey: "navigation.customization",
    },
    { id: "account", icon: User, labelKey: "navigation.account" },
    { id: "news", icon: Newspaper, labelKey: "navigation.news" },
    { id: "marketplace", icon: Store, labelKey: "navigation.marketplace" },
    {
        id: "custom_clients",
        icon: FileText,
        labelKey: "navigation.custom_clients",
    },
    { id: "about", icon: Info, labelKey: "navigation.about" },
    { id: "app_logs", icon: FileText, labelKey: "navigation.app_logs" },
];

const settingsLabelMap = computed<Record<string, string>>(() => ({
    ram: "RAM",
    language: t("settings.language"),
    discord_rpc_enabled: "Discord Rich Presence",
    hash_verify: t("settings.hash_verify"),
    sync_client_settings: t("settings.sync_client_settings"),
    dpi_bypass: "DPI Bypass (Zapret by bol-van)",
    minimize_to_tray_on_launch: t("settings.minimize_to_tray_on_launch"),
    close_to_tray: t("settings.close_to_tray"),
    auto_update: t("settings.auto_update"),
    autostart: t("settings.autostart"),
    start_minimized: t("settings.start_minimized"),
    java_path: t("settings.java_path"),
    java_args: t("settings.java_args"),
}));

type ResultItem = {
    type: "tab" | "client" | "setting" | "account" | "separator";
    id: string;
    label: string;
    subtitle?: string;
    icon?: any;
    isActive?: boolean;
};

const results = computed<ResultItem[]>(() => {
    const qRaw = query.value.trim();
    const q = qRaw.toLowerCase();
    const items: ResultItem[] = [];

    const scoreText = (text = "") => {
        const s = text.toLowerCase();
        if (!q) return 0;
        if (s === q) return 100;
        if (s.startsWith(q)) return 80;
        if (s.includes(q)) return 50;
        const tokens = q.split(/\s+/).filter(Boolean);
        const ok = tokens.every((t) => s.includes(t));
        return ok ? 30 : 0;
    };

    const makeClientItem = (client: Client) => ({
        type: "client" as const,
        id: String(client.id),
        label: client.name,
        subtitle: client.meta.installed
            ? `${client.version} · ${t("spotlight.client_launch")}`
            : `${client.version} · ${t("spotlight.client_download")}`,
        icon: Zap,
    });

    const tabs: Array<{ item: ResultItem; score: number }> = [];
    const accs: Array<{ item: ResultItem; score: number }> = [];
    const cls: Array<{ item: ResultItem; score: number }> = [];
    const stgs: Array<{ item: ResultItem; score: number }> = [];

    for (const tab of tabItems) {
        const label = t(tab.labelKey);
        const s = q ? Math.max(scoreText(label), scoreText(tab.id)) : 1;
        if (!q || s > 0) {
            tabs.push({
                item: { type: "tab", id: tab.id, label, icon: tab.icon },
                score: s,
            });
        }
    }

    for (const acc of accounts.value) {
        const s = q ? scoreText(acc.username) : 1;
        const item = {
            type: "account" as const,
            id: acc.id,
            label: acc.username,
            subtitle: acc.is_active
                ? t("spotlight.account_active")
                : t("spotlight.account_switch"),
            icon: acc.is_active ? UserCheck : User,
            isActive: acc.is_active,
        };
        if (!q || s > 0) accs.push({ item, score: s });
    }

    for (const client of clients.value) {
        const text = `${client.name} ${client.version}`;
        const s = q
            ? Math.max(
                  scoreText(client.name),
                  scoreText(client.version),
                  scoreText(text)
              )
            : 1;
        const item = makeClientItem(client);
        if (!q || s > 0) cls.push({ item, score: s });
    }

    const KEY_ORDER = [
        "ram",
        "language",
        "discord_rpc_enabled",
        "hash_verify",
        "sync_client_settings",
        "dpi_bypass",
        "minimize_to_tray_on_launch",
        "close_to_tray",
        "auto_update",
        "autostart",
        "java_path",
        "java_args",
    ];

    const settings = settingsService.getSettings();
    const sortedSettings = Object.entries(settings).sort(([a], [b]) => {
        const ai = KEY_ORDER.indexOf(a);
        const bi = KEY_ORDER.indexOf(b);
        if (ai === -1 && bi === -1) return 0;
        if (ai === -1) return 1;
        if (bi === -1) return -1;
        return ai - bi;
    });

    for (const [key, field] of sortedSettings) {
        if (!field.show) continue;
        if (
            key === "irc_chat" ||
            key === "optional_telemetry" ||
            key === "start_minimized"
        )
            continue;
        const label = settingsLabelMap.value[key] || key.replace(/_/g, " ");
        const s = q ? Math.max(scoreText(label), scoreText(key)) : 1;
        const item = {
            type: "setting" as const,
            id: key,
            label,
            subtitle: t("navigation.settings"),
            icon: Settings,
        };
        if (!q || s > 0) stgs.push({ item, score: s });
    }

    const appendSection = (
        list: Array<{ item: ResultItem; score: number }>,
        sepId: string,
        sepLabelKey: string
    ) => {
        if (list.length === 0) return;
        items.push({
            type: "separator",
            id: `sep-${sepId}`,
            label: t(sepLabelKey),
        });
        if (q) {
            list.sort(
                (a, b) =>
                    b.score - a.score ||
                    a.item.label.localeCompare(b.item.label)
            );
        }
        for (const e of list) items.push(e.item);
    };

    appendSection(tabs, "tabs", "spotlight.type_page");
    appendSection(accs, "accs", "spotlight.type_account");
    appendSection(cls, "clients", "spotlight.type_client");
    appendSection(stgs, "settings", "spotlight.type_setting");

    return items;
});

const selectableResults = computed(() =>
    results.value.filter((i) => i.type !== "separator")
);

watch([selectedIndex, selectableResults], async () => {
    if (selectedIndex.value < 0) return;

    await nextTick();

    const activeItem = resultsListRef.value?.querySelector<HTMLElement>(
        `[data-spotlight-index="${selectedIndex.value}"]`
    );

    activeItem?.scrollIntoView({ block: "nearest" });
});

watch(query, () => {
    selectedIndex.value = 0;
});

const loadData = async () => {
    try {
        clients.value = await invoke<Client[]>("get_clients");
        accounts.value = await invoke<Account[]>("get_accounts");
    } catch (e) {
        console.error("Failed to load spotlight data", e);
    }
};

watch(
    () => props.show,
    async (val) => {
        if (val) {
            query.value = "";
            selectedIndex.value = 0;
            await nextTick();
            inputRef.value?.focus();
            await loadData();
        }
    }
);

const selectItem = async (item: ResultItem) => {
    if (item.type === "separator") return;

    if (item.type === "tab") {
        emit("navigate", item.id);
        emit("close");
    } else if (item.type === "client") {
        emit("close");
        const client = clients.value.find((c) => String(c.id) === item.id);
        if (client) {
            if (client.meta.installed) {
                try {
                    const userToken =
                        localStorage.getItem("authToken") || "null";
                    await invoke("increment_client_counter", {
                        id: client.id,
                        counterType: "launch",
                    });
                    await invoke("launch_client", { id: client.id, userToken });
                } catch (e) {
                    console.error("Failed to launch client from spotlight", e);
                }
            } else {
                try {
                    await invoke("increment_client_counter", {
                        id: client.id,
                        counterType: "download",
                    });
                    await invoke("download_client_only", { id: client.id });
                    clients.value = await invoke<Client[]>("get_clients");
                } catch (e) {
                    console.error(
                        "Failed to download client from spotlight",
                        e
                    );
                }
            }
        }
    } else if (item.type === "setting") {
        try {
            localStorage.setItem("spotlight_highlight_setting", item.id);
        } catch (e) {
            console.error("Failed to set spotlight highlight setting", e);
        }
        emit("navigate", "settings");
        emit("close");
    } else if (item.type === "account") {
        if (!item.isActive) {
            try {
                await invoke("set_active_account", { id: item.id });
                accounts.value = await invoke<Account[]>("get_accounts");
            } catch (e) {
                console.error("Failed to switch account", e);
            }
        }
    }
};

const getSelectableIndex = (item: ResultItem) => {
    return selectableResults.value.findIndex(
        (i) => i.id === item.id && i.type === item.type
    );
};

const handleKeydown = (e: KeyboardEvent) => {
    if (e.key === "ArrowDown") {
        e.preventDefault();
        selectedIndex.value = Math.min(
            selectedIndex.value + 1,
            selectableResults.value.length - 1
        );
    } else if (e.key === "ArrowUp") {
        e.preventDefault();
        selectedIndex.value = Math.max(selectedIndex.value - 1, 0);
    } else if (e.key === "Enter") {
        e.preventDefault();
        const item = selectableResults.value[selectedIndex.value];
        if (item) selectItem(item);
    } else if (e.key === "Escape") {
        emit("close");
    }
};
</script>

<template>
    <Teleport to="body">
        <div v-if="show"
            class="spotlight-overlay fixed inset-0 z-9999 flex items-start justify-center pt-[15vh] spotlight-overlay--visible"
            @click.self="emit('close')"
        >
            <div
                class="absolute inset-0 bg-black/20"
                @click="emit('close')"
            />

            <div
                class="spotlight-panel relative w-full max-w-xl mx-4 rounded-2xl shadow-2xl border border-base-300/60 overflow-hidden"
                :class="show ? 'spotlight-panel--visible' : 'spotlight-panel--hidden'"
                @keydown="handleKeydown"
            >
                    <div
                        class="flex items-center gap-3 px-4 py-3 border-b border-base-300"
                    >
                        <Search class="w-5 h-5 text-base-content/40 shrink-0" />
                        <input
                            ref="inputRef"
                            v-model="query"
                            :placeholder="t('spotlight.placeholder')"
                            class="flex-1 bg-transparent outline-none text-base-content text-sm placeholder:text-base-content/40"
                        />
                        <kbd class="kbd kbd-sm opacity-50">Esc</kbd>
                    </div>

                    <div
                        v-if="results.length > 0"
                        class="py-2 px-2 max-h-80 overflow-y-auto overflow-x-hidden"
                        ref="resultsListRef"
                        @mouseleave="selectedIndex = -1"
                    >
                        <template
                            v-for="item in results"
                            :key="item.type + item.id"
                        >
                            <div
                                v-if="item.type === 'separator'"
                                class="px-4 pt-3 pb-1 text-xs font-semibold text-base-content/30 uppercase tracking-wider"
                            >
                                {{ item.label }}
                            </div>
                            <button
                                v-else
                                :data-spotlight-index="getSelectableIndex(item)"
                                class="w-full flex items-center gap-3 px-3 py-2 text-left transition-colors rounded-lg"                                :class="
                                    getSelectableIndex(item) === selectedIndex
                                        ? 'bg-primary/10 text-primary'
                                        : 'hover:bg-base-200'
                                "
                                @click="selectItem(item)"
                                @mouseenter="
                                    selectedIndex = getSelectableIndex(item)
                                "
                            >
                                <div
                                    class="w-8 h-8 rounded-lg flex items-center justify-center shrink-0"
                                    :class="[
                                        getSelectableIndex(item) ===
                                        selectedIndex
                                            ? 'bg-primary/20'
                                            : 'bg-base-300',
                                        item.isActive
                                            ? 'ring-1 ring-success/50'
                                            : '',
                                    ]"
                                >
                                    <component
                                        :is="item.icon"
                                        class="w-4 h-4"
                                        :class="
                                            item.isActive ? 'text-success' : ''
                                        "
                                    />
                                </div>
                                <div class="flex-1 min-w-0">
                                    <div
                                        class="text-sm font-medium truncate flex items-center gap-1.5"
                                    >
                                        {{ item.label }}
                                        <CheckCircle
                                            v-if="item.isActive"
                                            class="w-3.5 h-3.5 text-success shrink-0"
                                        />
                                    </div>
                                    <div
                                        v-if="item.subtitle"
                                        class="text-xs text-base-content/50 truncate"
                                    >
                                        {{ item.subtitle }}
                                    </div>
                                </div>
                                <ChevronRight
                                    class="w-3.5 h-3.5 text-base-content/30 shrink-0"
                                />
                            </button>
                        </template>
                    </div>

                    <div
                        v-else-if="query.trim()"
                        class="py-10 text-center text-base-content/40 text-sm"
                    >
                        {{ t("spotlight.no_results") }}
                    </div>

                    <div
                        v-else
                        class="py-6 text-center text-base-content/30 text-xs"
                    >
                        {{ t("spotlight.hint") }}
                    </div>

                    <div
                        class="px-4 py-2 border-t border-base-300 flex items-center gap-4 text-xs text-base-content/30"
                    >
                        <span
                            ><kbd class="kbd kbd-xs">↑↓</kbd>
                            {{ t("spotlight.navigate") }}</span
                        >
                        <span
                            ><kbd class="kbd kbd-xs">↵</kbd>
                            {{ t("spotlight.select") }}</span
                        >
                        <span
                            ><kbd class="kbd kbd-xs">Esc</kbd>
                            {{ t("spotlight.close") }}</span
                        >
                    </div>
                </div>
            </div>
    </Teleport>
</template>

<style scoped>
.spotlight-overlay {
    transition: opacity 0.15s ease;
}

.spotlight-overlay--hidden {
    opacity: 0;
    pointer-events: none;
}

.spotlight-overlay--visible {
    opacity: 1;
    pointer-events: auto;
}

.spotlight-panel {
    background-color: color-mix(in srgb, var(--color-base-100) 45%, transparent);
    backdrop-filter: blur(24px) saturate(1.6) brightness(1.05);
    -webkit-backdrop-filter: blur(24px) saturate(1.6) brightness(1.05);
    transition:
        opacity 0.15s cubic-bezier(0.16, 1, 0.3, 1),
        transform 0.15s cubic-bezier(0.16, 1, 0.3, 1);
}

.spotlight-panel--hidden {
    opacity: 0;
    transform: scale(0.96) translateY(-6px);
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
}

.spotlight-panel--visible {
    opacity: 1;
    transform: scale(1) translateY(0);
}
</style>

<script setup lang="ts">
import { ref, onMounted, watch, computed, nextTick } from "vue";
import { invoke } from "@tauri-apps/api/core";
import AnimatedSlider from "@shared/components/ui/AnimatedSlider.vue";
import AnimatedDropdown from "@shared/components/ui/AnimatedDropdown.vue";
import {
    RotateCcw,
    Plus,
    Edit3,
    Trash2,
    User,
    Cloud,
    Download,
    Upload,
    Settings as SettingsIcon,
    MemoryStick,
    Languages,
    ChartNoAxesCombined,
    Info,
    Folder,
    Waypoints,
    MessagesSquare,
    BadgeCheck,
    FileText,
    FolderSync,
    CloudCog,
    Search,
    UserIcon,
    LogIn,
    Minimize2,
    MousePointer2,
    Coffee,
    Terminal,
    HardDrive,
    RefreshCcw,
} from "lucide-vue-next";
import { useToast } from "@shared/composables/useToast";
import type { ToastPosition } from "@shared/types/toast";
import { syncService } from "../services/syncService";
import { settingsService } from "@services/settings/settingsService";
import { globalUserStatus } from "@features/auth/useUserStatus";
import {
    userService,
    type UserExternalAccount,
} from "@features/auth/userService";
import AddAccountModal from "@features/social/modals/AddAccountModal.vue";
import EditAccountModal from "@features/social/modals/EditAccountModal.vue";
import ResetConfirmModal from "@services/settings/modals/ResetConfirmModal.vue";
import TelemetryInfoModal from "@features/clients/modals/TelemetryInfoModal.vue";
import ChangeRootFolderModal from "@services/settings/modals/ChangeRootFolderModal.vue";
import DeleteAccountConfirmModal from "@features/social/modals/DeleteAccountConfirmModal.vue";
import SettingCard from "../components/settings/SettingCard.vue";
import AccountCard from "../components/settings/AccountCard.vue";
import {
    changeLanguage,
    getAvailableLanguages,
    getCurrentLanguage,
} from "@services/i18n";
import { useI18n } from "vue-i18n";
import { formatDate } from "@shared/utils/utils";
import { useModal } from "@shared/composables/useModal";

interface Account {
    id: string;
    username: string;
    tags: string[];
    created_at: string;
    last_used?: string;
    is_active: boolean;
}

const settings = settingsService.settings;
const accounts = ref<Account[]>([]);
const remoteAccounts = ref<UserExternalAccount[]>([]);
const remoteAccountsLoading = ref(false);
const isAccountSyncing = ref(false);
const activeTab = ref<"general" | "sync" | "accounts">("general");
const loading = ref(true);
const isRefreshing = ref(false);
const { addToast, setToastPosition, getToastPosition } = useToast();

const editingAccount = ref<Account | null>(null);
const accountToDelete = ref<Account | null>(null);
const searchQuery = ref("");

const { showModal } = useModal();

const ramOptions = [
    { mb: 2048, label: "2 GB" },
    { mb: 4096, label: "4 GB" },
    { mb: 6144, label: "6 GB" },
    { mb: 8192, label: "8 GB" },
    { mb: 16384, label: "16 GB" },
    { mb: 32768, label: "32 GB" },
];
const ramOptionIndex = ref(0);
const systemMemory = ref<number | null>(null);
const showRamWarning = ref(false);

const selectedRamMb = computed(() => ramOptions[ramOptionIndex.value]?.mb || 0);

const checkRamWarning = () => {
    if (selectedRamMb.value > 6144) {
        showRamWarning.value = true;
    } else {
        showRamWarning.value = false;
    }
};

watch(ramOptionIndex, checkRamWarning);
watch(systemMemory, checkRamWarning);

const { t } = useI18n();
const availableLanguages = getAvailableLanguages();
const currentLanguage = ref(getCurrentLanguage());

const isAuthenticated = computed(() => globalUserStatus.isAuthenticated.value);

const filteredSettingsEntries = computed(() => {
    const KEY_ORDER = [
        "ram",
        "language",
        "discord_rpc_enabled",
        "optional_telemetry",
        "cordshare",
        "irc_chat",
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

    return Object.entries(settings)
        .filter(([, field]) => field.show)
        .filter(([key]) => key !== "irc_chat")
        .filter(([key]) => key !== "optional_telemetry")
        .filter(([key]) => key !== "start_minimized")
        .sort(([a], [b]) => {
            const ai = KEY_ORDER.indexOf(a);
            const bi = KEY_ORDER.indexOf(b);
            if (ai === -1 && bi === -1) return 0;
            if (ai === -1) return 1;
            if (bi === -1) return -1;
            return ai - bi;
        });
});

const handleSliderChange = () => {
    if (settings.ram && ramOptions[ramOptionIndex.value]) {
        if (settings.ram.value !== ramOptions[ramOptionIndex.value].mb) {
            settings.ram.value = ramOptions[ramOptionIndex.value].mb;
        }
    }
};

watch(
    () => settings.ram?.value,
    (newMb) => {
        if (newMb === undefined) return;
        if (loading.value && Object.keys(settings).length === 0) return;

        const exactMatchIndex = ramOptions.findIndex((opt) => opt.mb === newMb);
        if (exactMatchIndex !== -1) {
            if (ramOptionIndex.value !== exactMatchIndex) {
                ramOptionIndex.value = exactMatchIndex;
            }
        } else {
            let determinedClosestIndex = 0;
            let minDifference = Infinity;
            ramOptions.forEach((option, index) => {
                const diff = Math.abs(newMb - option.mb);
                if (diff < minDifference) {
                    minDifference = diff;
                    determinedClosestIndex = index;
                }
            });
            if (ramOptionIndex.value !== determinedClosestIndex) {
                ramOptionIndex.value = determinedClosestIndex;
            }
        }
    },
    { immediate: true }
);

const loadSettings = async () => {
    try {
        loading.value = true;
        await settingsService.loadSettings();
    } catch (error) {
        console.error("Failed to load settings:", error);
        addToast(t("settings.load_settings_failed", { error }), "error");
    } finally {
        loading.value = false;
    }
};

const loadAccounts = async (skipLoading = false) => {
    try {
        if (!skipLoading) {
            isRefreshing.value = true;
        }

        const fetchedAccounts = await invoke<Account[]>("get_accounts");

        await nextTick();
        accounts.value = fetchedAccounts;

        if (accounts.value.length === 0) {
            const { username, tags } = generateRandomAccount();
            try {
                const newAccountId = await invoke<string>("add_account", {
                    username,
                    tags,
                });

                await invoke("set_active_account", { id: newAccountId });
                accounts.value = await invoke<Account[]>("get_accounts");
            } catch (error) {
                console.error("Failed to create default account:", error);
            }
        }
    } catch (error) {
        console.error("Failed to load accounts:", error);
        addToast(t("settings.load_accounts_failed", { error }), "error");
    } finally {
        if (!skipLoading) {
            isRefreshing.value = false;
        }
    }
};

const loadRemoteAccounts = async () => {
    if (!isAuthenticated.value) {
        remoteAccounts.value = [];
        return;
    }

    remoteAccountsLoading.value = true;
    try {
        const fetchedAccounts = await userService.getExternalAccounts();
        remoteAccounts.value = fetchedAccounts;
    } catch (error) {
        console.error("Failed to load remote accounts:", error);
        addToast(t("settings.load_remote_accounts_failed", { error }), "error");
    } finally {
        remoteAccountsLoading.value = false;
    }
};

const syncLocalAccountsToCloud = async () => {
    if (!isAuthenticated.value) {
        addToast(t("settings.sync_login_required"), "error");
        return;
    }

    isAccountSyncing.value = true;
    try {
        const [localAccounts, externalAccounts] = await Promise.all([
            invoke<Account[]>("get_accounts"),
            userService.getExternalAccounts(),
        ]);

        const cloudAccounts = externalAccounts;
        const localUsernames = new Set(
            localAccounts.map((account) => account.username)
        );
        const cloudUsernameMap = new Map(
            cloudAccounts.map((account) => [account.display_name, account])
        );

        await Promise.allSettled(
            cloudAccounts
                .filter(
                    (account) =>
                        account.display_name &&
                        !localUsernames.has(account.display_name)
                )
                .map((account) => userService.deleteExternalAccount(account.id))
        );

        for (const localAccount of localAccounts) {
            if (!cloudUsernameMap.has(localAccount.username)) {
                try {
                    await userService.addExternalAccount({
                        display_name: localAccount.username,
                        metadata: { tags: localAccount.tags || [] },
                    });
                } catch (error) {
                    console.warn(
                        "Failed to sync local account to Atlas:",
                        localAccount.username,
                        error
                    );
                }
            }
        }

        addToast(t("settings.sync_accounts_to_cloud_success"), "success");
        await loadRemoteAccounts();
    } catch (error) {
        console.error("Failed to sync accounts to cloud:", error);
        addToast(
            t("settings.sync_accounts_to_cloud_failed", { error }),
            "error"
        );
    } finally {
        isAccountSyncing.value = false;
    }
};

const syncLocalAccountsFromCloud = async () => {
    if (!isAuthenticated.value) {
        addToast(t("settings.sync_login_required"), "error");
        return;
    }

    isAccountSyncing.value = true;
    try {
        const externalAccounts = await userService.getExternalAccounts();
        const cloudAccounts = externalAccounts;

        if (cloudAccounts.length === 0) {
            addToast(t("settings.remote_accounts_empty"), "info");
            return;
        }

        const localAccountsData = await invoke<Account[]>("get_accounts");
        const localUsernames = new Set(
            localAccountsData.map((account) => account.username)
        );

        for (const cloudAccount of cloudAccounts) {
            if (
                cloudAccount.display_name &&
                !localUsernames.has(cloudAccount.display_name)
            ) {
                await invoke("add_account", {
                    username: cloudAccount.display_name,
                    tags: Array.isArray((cloudAccount.metadata as any)?.tags)
                        ? (cloudAccount.metadata as any).tags
                        : ["cloud-sync"],
                });
            }
        }

        addToast(t("settings.sync_accounts_from_cloud_success"), "success");
        await loadAccounts();
    } catch (error) {
        console.error("Failed to import accounts from cloud:", error);
        addToast(
            t("settings.sync_accounts_from_cloud_failed", { error }),
            "error"
        );
    } finally {
        isAccountSyncing.value = false;
    }
};

watch(isAuthenticated, (authenticated) => {
    if (authenticated) {
        loadRemoteAccounts();
    } else {
        remoteAccounts.value = [];
    }
});

const loadFlags = async () => {
    try {
        await settingsService.loadFlags();
    } catch (error) {
        console.error("Failed to load flags:", error);
        addToast(t("settings.load_flags_failed", { error }), "error");
    }
};

const showAddAccountDialog = () => {
    showModal(
        "add-account",
        AddAccountModal,
        {
            title: t("settings.add_account_title"),
        },
        {},
        {
            "account-added": handleAccountAdded,
        }
    );
};

const showEditAccountDialog = (account: Account) => {
    editingAccount.value = account;
    showModal(
        "edit-account",
        EditAccountModal,
        {
            title: t("settings.edit_account_title"),
        },
        { account },
        {
            "account-updated": handleAccountUpdated,
        }
    );
};

const showResetConfirmDialog = () => {
    showModal(
        "reset-confirm",
        ResetConfirmModal,
        {
            title: t("settings.reset_title"),
        },
        {},
        {
            "settings-reset": handleSettingsReset,
        }
    );
};

const showDeleteConfirmDialog = (account: Account) => {
    accountToDelete.value = account;
    showModal(
        "delete-confirm",
        DeleteAccountConfirmModal,
        {
            title: t("settings.delete_account_title"),
        },
        { account },
        {
            "account-deleted": handleAccountDeleted,
        }
    );
};

const showTelemetryModal = () => {
    showModal(
        "telemetry-info",
        TelemetryInfoModal,
        {
            title: t("settings.telemetry_info_title"),
        },
        {},
        {}
    );
};

const showChangeRootFolderDialog = () => {
    showModal(
        "change-root-folder",
        ChangeRootFolderModal,
        { title: t("settings.change_root.title") },
        {},
        {}
    );
};

const handleAccountAdded = async () => {
    await loadAccounts();
};

const handleAccountUpdated = async () => {
    await loadAccounts();
};

const handleAccountDeleted = async () => {
    await loadAccounts();
};

const handleSettingsReset = async () => {
    await loadSettings();
};

const resetSettings = () => {
    showResetConfirmDialog();
};

const deleteAccount = (account: Account) => {
    showDeleteConfirmDialog(account);
};

const editAccount = (account: Account) => {
    showEditAccountDialog(account);
};

const setActiveAccount = async (account: Account) => {
    try {
        await invoke("set_active_account", { id: account.id });
        await loadAccounts(true);
        addToast(
            t("settings.account_set_active", { username: account.username }),
            "success"
        );
    } catch (error) {
        console.error("Failed to set active account:", error);
        addToast(t("settings.account_set_active_failed", { error }), "error");
    }
};

const getFormattedLabel = (key: string) => {
    const words = key.split("_");

    if (key === "ram") {
        return "RAM";
    }

    if (key == "irc_chat") {
        return t("settings.irc_chat");
    }

    if (key === "hash_verify") {
        return t("settings.hash_verify");
    }

    if (key === "sync_client_settings") {
        return t("settings.sync_client_settings");
    }

    if (key === "discord_rpc_enabled") {
        return "Discord Rich Presence";
    }

    if (key === "enable_telemetry") {
        return t("settings.telemetry");
    }

    if (key === "dpi_bypass") {
        return "DPI Bypass (Zapret by bol-van)";
    }

    if (key === "minimize_to_tray_on_launch") {
        return t("settings.minimize_to_tray_on_launch");
    }

    if (key === "close_to_tray") {
        return t("settings.close_to_tray");
    }

    if (key === "java_path") {
        return "Custom Java Path";
    }

    if (key === "java_args") {
        return "Custom Java Arguments";
    }

    if (key === "auto_update") {
        return t("settings.auto_update");
    }

    if (key === "autostart") {
        return t("settings.autostart");
    }

    if (key === "start_minimized") {
        return t("settings.start_minimized");
    }

    return words
        .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
        .join(" ");
};

const isFeatureOnline = (key: string) => {
    if (key == "irc_chat") {
        return true;
    }

    if (key === "cordshare") {
        return true;
    }
};

const generateRandomAccount = () => {
    const randomDigits = Math.floor(10000 + Math.random() * 90000);
    const username = `Collapse${randomDigits}`;
    return { username, tags: ["auto-generated"] };
};

const selectedTag = ref<string | null>(null);

const highlightedSetting = ref<string | null>(null);

const uniqueTags = computed(() => {
    const tagSet = new Set<string>();
    accounts.value.forEach((account) => {
        account.tags.forEach((tag) => {
            tagSet.add(tag);
        });
    });
    return Array.from(tagSet).sort();
});

const filteredAccounts = computed(() => {
    let result = accounts.value;

    if (selectedTag.value) {
        result = result.filter((account) =>
            account.tags.includes(selectedTag.value!)
        );
    }

    if (searchQuery.value) {
        const query = searchQuery.value.toLowerCase();
        result = result.filter((account) =>
            account.username.toLowerCase().includes(query)
        );
    }

    return result;
});

const selectTag = (tag: string) => {
    if (selectedTag.value === tag) {
        selectedTag.value = null;
    } else {
        selectedTag.value = tag;
    }
};

// let unsubscribeSyncService: (() => void) | null = null;

const handleLanguageChange = async (languageCode: string) => {
    try {
        currentLanguage.value = languageCode as any;
        await changeLanguage(languageCode);
    } catch (error) {
        console.error("Failed to change language:", error);
        addToast(t("settings.language_change_failed", { error }), "error");
    }
};

const getSettingDescription = (key: string) => {
    return t(`settings.descriptions.${key}`);
};

const openDataFolder = async () => {
    await invoke("open_data_folder");
};

const storageUsage = ref<{
    clients: number;
    libraries: number;
    natives: number;
    assets: number;
    java: number;
    other: number;
    total: number;
} | null>(null);
const storageLoading = ref(false);

const loadStorageUsage = async () => {
    storageLoading.value = true;
    try {
        storageUsage.value = await invoke("get_storage_usage");
    } catch (e) {
        console.error("Failed to get storage usage", e);
    } finally {
        storageLoading.value = false;
    }
};

const formatBytes = (bytes: number): string => {
    if (bytes === 0) return "0 B";
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    if (bytes < 1024 * 1024 * 1024)
        return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(2)} GB`;
};

const resetRequirements = async () => {
    try {
        await invoke("reset_requirements");
        addToast(t("settings.reset_requirements_success"), "success");
    } catch (error) {
        console.error("Failed to reset requirements:", error);
        addToast(t("settings.reset_requirements_failed"), "error");
    }
};

const draggedId = ref<string | null>(null);
const dragOverId = ref<string | null>(null);

let _ghostEl: HTMLElement | null = null;
let _ghostOffsetX = 0;
let _ghostOffsetY = 0;

const _reorderAccounts = async (sourceId: string, targetId: string) => {
    const fromIndex = accounts.value.findIndex((a) => a.id === sourceId);
    const toIndex = accounts.value.findIndex((a) => a.id === targetId);
    if (fromIndex === -1 || toIndex === -1) return;
    const reordered = [...accounts.value];
    const [moved] = reordered.splice(fromIndex, 1);
    reordered.splice(toIndex, 0, moved);
    accounts.value = reordered;
    try {
        await invoke("reorder_accounts", {
            orderedIds: reordered.map((a) => a.id),
        });
    } catch (e) {
        console.error("Failed to reorder accounts", e);
        await loadAccounts();
    }
};

const _onMouseMove = (e: MouseEvent) => {
    if (!_ghostEl) return;
    _ghostEl.style.top = `${e.clientY - _ghostOffsetY}px`;
    _ghostEl.style.left = `${e.clientX - _ghostOffsetX}px`;

    const cards = document.querySelectorAll<HTMLElement>("[data-account-id]");
    let found: string | null = null;
    for (const card of cards) {
        const id = card.dataset.accountId;
        if (!id || id === draggedId.value) continue;
        const r = card.getBoundingClientRect();
        if (
            e.clientX >= r.left &&
            e.clientX <= r.right &&
            e.clientY >= r.top &&
            e.clientY <= r.bottom
        ) {
            found = id;
            break;
        }
    }
    dragOverId.value = found;
};

const _onMouseUp = async () => {
    document.removeEventListener("mousemove", _onMouseMove);
    document.removeEventListener("mouseup", _onMouseUp);
    document.body.style.userSelect = "";
    document.body.style.cursor = "";

    if (_ghostEl) {
        _ghostEl.remove();
        _ghostEl = null;
    }

    const sourceId = draggedId.value;
    const targetId = dragOverId.value;
    draggedId.value = null;
    dragOverId.value = null;

    if (sourceId && targetId) {
        await _reorderAccounts(sourceId, targetId);
    }
};

const startDrag = (e: MouseEvent, accountId: string) => {
    e.preventDefault();
    draggedId.value = accountId;
    dragOverId.value = null;

    // Find root card element
    const cardEl =
        ((e.currentTarget as HTMLElement)?.closest?.(
            "[data-account-id]"
        ) as HTMLElement) ??
        document.querySelector<HTMLElement>(`[data-account-id="${accountId}"]`);

    if (cardEl) {
        const rect = cardEl.getBoundingClientRect();
        _ghostOffsetX = e.clientX - rect.left;
        _ghostOffsetY = e.clientY - rect.top;

        const clone = cardEl.cloneNode(true) as HTMLElement;
        clone.style.cssText = [
            `position:fixed`,
            `top:${rect.top}px`,
            `left:${rect.left}px`,
            `width:${rect.width}px`,
            `z-index:9999`,
            `pointer-events:none`,
            `opacity:0.85`,
            `transform:rotate(1.5deg) scale(1.03)`,
            `box-shadow:0 20px 40px rgba(0,0,0,0.35)`,
            `border-radius:0.5rem`,
            `transition:none`,
        ].join(";");
        document.body.appendChild(clone);
        _ghostEl = clone;
    }

    document.body.style.userSelect = "none";
    document.body.style.cursor = "grabbing";
    document.addEventListener("mousemove", _onMouseMove);
    document.addEventListener("mouseup", _onMouseUp);
};

onMounted(async () => {
    // unsubscribeSyncService = syncService.subscribe((state) => {
    //     syncState.value = state;
    // });

    await syncService.initializeSyncStatus();

    await loadSettings();
    await loadFlags();
    await loadAccounts();
    if (isAuthenticated.value) {
        await loadRemoteAccounts();
    }

    try {
        const memoryBytes = await invoke<number>("get_system_memory");
        systemMemory.value = Math.floor(memoryBytes / (1024 * 1024));
    } catch (error) {
        console.error("Failed to get system memory:", error);
        systemMemory.value = null;
    }

    loadStorageUsage();

    try {
        const requested = localStorage.getItem("spotlight_highlight_setting");
        if (requested) {
            localStorage.removeItem("spotlight_highlight_setting");
            highlightedSetting.value = requested;
            await nextTick();

            const el = document.querySelector<HTMLElement>(
                `[data-setting-key="${requested}"]`
            );
            if (el) {
                el.scrollIntoView({ behavior: "smooth", block: "center" });
            }

            setTimeout(() => (highlightedSetting.value = null), 3000);
        }
    } catch (e) {
        console.error("Failed to process spotlight highlight request", e);
    }
});

// onUnmounted(() => {
//     if (unsubscribeSyncService) {
//         unsubscribeSyncService();
//     }
// });

watch(
    () => settings.enable_telemetry?.value,
    async (newValue) => {
        if (loading.value) return;
        if (newValue === undefined) return;

        try {
            await invoke("set_optional_telemetry", { enabled: newValue });
        } catch (error) {
            console.error("Failed to update telemetry setting:", error);
            addToast(
                newValue
                    ? t("toast.telemetry.analytics_enable_failed", { error })
                    : t("toast.telemetry.analytics_disable_failed", { error }),
                "error"
            );

            if (settings.enable_telemetry) {
                settings.enable_telemetry.value = !newValue;
            }
        }
    }
);

const toastPosition = ref<ToastPosition>(getToastPosition());

const toastPositionOptions = [
    { value: "bottom-right", label: t("settings.toast_position.bottom_right") },
    { value: "bottom-left", label: t("settings.toast_position.bottom_left") },
    { value: "top-right", label: t("settings.toast_position.top_right") },
    { value: "top-left", label: t("settings.toast_position.top_left") },
    {
        value: "bottom-center",
        label: t("settings.toast_position.bottom_center"),
    },
    { value: "top-center", label: t("settings.toast_position.top_center") },
];

const handleToastPositionChange = (position: string) => {
    const pos = position as ToastPosition;
    toastPosition.value = pos;
    setToastPosition(pos);

    addToast(t("settings.toast_position.preview_message"), "info", 3000);
};
</script>
<template>
    <div class="max-w-6xl mx-auto slide-up px-4">
        <div v-if="!loading" class="transition-opacity duration-300">
            <div
                role="tablist"
                class="tabs tabs-boxed mb-4 flex justify-center gap-2"
            >
                <a
                    @click="activeTab = 'general'"
                    class="tab transition-all duration-300 z-50"
                    :class="{
                        'tab-active transform scale-105 shadow-md bg-base-300':
                            activeTab === 'general',
                        'hover:bg-base-300': activeTab !== 'general',
                    }"
                >
                    <SettingsIcon class="w-4 h-4 mr-2" />
                    {{ t("settings.general") }}
                </a>
                <a
                    @click="activeTab = 'accounts'"
                    class="tab transition-all duration-300"
                    :class="{
                        'tab-active transform scale-105 shadow-md bg-base-300':
                            activeTab === 'accounts',
                        'hover:bg-base-300': activeTab !== 'accounts',
                    }"
                >
                    <User class="w-4 h-4 mr-2" />
                    {{ t("settings.accounts") }}
                </a>
            </div>
            <transition name="tab-switch" mode="out-in">
                <div
                    v-if="activeTab === 'general'"
                    key="general"
                    class="space-y-4"
                >
                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-3">
                        <div
                            v-for="(
                                [key, field], index
                            ) in filteredSettingsEntries"
                            :key="key"
                            :data-setting-key="key"
                            :class="[
                                'flex w-full',
                                key === 'java_path' || key === 'java_args'
                                    ? 'lg:col-span-2'
                                    : 'h-full',
                                highlightedSetting === key
                                    ? 'spotlight-highlight'
                                    : '',
                            ]"
                        >
                            <SettingCard
                                :field="field"
                                :delay="index * 0.05"
                                :description="getSettingDescription(key)"
                                :layout="
                                    key === 'ram' ||
                                    (typeof field.value !== 'boolean' &&
                                        key !== 'language')
                                        ? 'col'
                                        : 'row'
                                "
                            >
                                <template #title>
                                    <MemoryStick
                                        v-if="key === 'ram'"
                                        class="w-5 h-5 text-primary"
                                    />
                                    <Languages
                                        v-if="key === 'language'"
                                        class="w-5 h-5 text-primary"
                                    />
                                    <img
                                        src="@/assets/icons/discord.svg"
                                        v-if="key === 'discord_rpc_enabled'"
                                        class="w-5 h-5 discord-icon"
                                    />
                                    <ChartNoAxesCombined
                                        v-if="key === 'optional_telemetry'"
                                        class="w-5 h-5 text-primary"
                                    />
                                    <Waypoints
                                        v-if="key === 'cordshare'"
                                        class="w-5 h-5 text-primary"
                                    />
                                    <MessagesSquare
                                        v-if="key === 'irc_chat'"
                                        class="w-5 h-5 text-primary"
                                    />
                                    <BadgeCheck
                                        v-if="key === 'hash_verify'"
                                        class="w-5 h-5 text-primary"
                                    />
                                    <FileText
                                        v-if="key === 'custom_clients_display'"
                                        class="w-5 h-5 text-primary"
                                    />
                                    <FolderSync
                                        v-if="key === 'sync_client_settings'"
                                        class="w-5 h-5 text-primary"
                                    />
                                    <CloudCog
                                        v-if="key === 'dpi_bypass'"
                                        class="w-5 h-5 text-primary"
                                    />
                                    <Minimize2
                                        v-if="
                                            key === 'minimize_to_tray_on_launch'
                                        "
                                        class="w-5 h-5 text-primary"
                                    />
                                    <MousePointer2
                                        v-if="key === 'close_to_tray'"
                                        class="w-5 h-5 text-primary"
                                    />
                                    <RefreshCcw
                                        v-if="key === 'auto_update'"
                                        class="w-5 h-5 text-primary"
                                    />
                                    <HardDrive
                                        v-if="key === 'autostart'"
                                        class="w-5 h-5 text-primary"
                                    />
                                    <Minimize2
                                        v-if="key === 'start_minimized'"
                                        class="w-5 h-5 text-primary"
                                    />
                                    <Coffee
                                        v-if="key === 'java_path'"
                                        class="w-5 h-5 text-primary"
                                    />
                                    <Terminal
                                        v-if="key === 'java_args'"
                                        class="w-5 h-5 text-primary"
                                    />
                                    {{ getFormattedLabel(key) }}
                                    <div
                                        v-if="key === 'optional_telemetry'"
                                        class="tooltip tooltip-top"
                                        :data-tip="
                                            $t('settings.telemetry_info_title')
                                        "
                                    >
                                        <Info
                                            class="w-5 h-5 text-primary cursor-pointer"
                                            @click="showTelemetryModal"
                                        />
                                    </div>
                                </template>

                                <div
                                    v-if="key === 'ram'"
                                    class="space-y-3 w-full"
                                >
                                    <div class="flex items-center gap-2">
                                        <AnimatedSlider
                                            v-model="ramOptionIndex"
                                            :min="0"
                                            :max="ramOptions.length - 1"
                                            @update:modelValue="
                                                handleSliderChange
                                            "
                                            class="grow"
                                        />
                                        <div
                                            class="flex items-center gap-2 rounded-md p-2 bg-base-200/50"
                                        >
                                            <input
                                                v-if="settings.ram"
                                                v-model.number="
                                                    settings.ram.value
                                                "
                                                type="number"
                                                min="512"
                                                step="512"
                                                class="input input-ghost input-xs text-right p-0 h-6 w-12 font-mono"
                                            />
                                            <span class="text-xs opacity-50"
                                                >MB</span
                                            >
                                        </div>
                                    </div>
                                    <div
                                        v-if="showRamWarning"
                                        class="alert alert-warning alert-dash mt-2 py-2 text-sm"
                                    >
                                        <span>{{
                                            t("settings.ram.warning", {
                                                selectedRamMb,
                                            })
                                        }}</span>
                                    </div>
                                </div>

                                <div v-else-if="key === 'language'">
                                    <AnimatedDropdown
                                        :modelValue="currentLanguage"
                                        :options="
                                            availableLanguages.map((l) => ({
                                                value: l.code,
                                                label: `${l.nativeName} (${l.name})`,
                                            }))
                                        "
                                        direction="down"
                                        @change="handleLanguageChange"
                                    />
                                </div>

                                <div
                                    v-else-if="typeof field.value === 'boolean'"
                                >
                                    <div
                                        v-if="key === 'autostart'"
                                        class="flex flex-col items-end gap-2"
                                    >
                                        <input
                                            type="checkbox"
                                            v-model="field.value"
                                            class="toggle toggle-primary toggle-sm"
                                        />
                                        <transition name="fade-slide">
                                            <div
                                                v-if="
                                                    field.value &&
                                                    settings.start_minimized
                                                "
                                                class="flex items-center gap-2 text-xs text-base-content/70"
                                            >
                                                <Minimize2
                                                    class="w-3.5 h-3.5"
                                                />
                                                <span>{{
                                                    t(
                                                        "settings.start_minimized"
                                                    )
                                                }}</span>
                                                <input
                                                    type="checkbox"
                                                    v-model="
                                                        settings.start_minimized
                                                            .value
                                                    "
                                                    class="toggle toggle-primary toggle-xs"
                                                />
                                            </div>
                                        </transition>
                                    </div>
                                    <input
                                        v-else-if="
                                            !isFeatureOnline(key) ||
                                            isAuthenticated
                                        "
                                        type="checkbox"
                                        v-model="field.value"
                                        class="toggle toggle-primary toggle-sm"
                                    />

                                    <div
                                        v-else
                                        class="tooltip tooltip-left"
                                        :data-tip="
                                            $t('settings.login_required')
                                        "
                                    >
                                        <input
                                            type="checkbox"
                                            :checked="false"
                                            class="toggle toggle-primary toggle-sm pointer-events-none opacity-50"
                                            tabindex="-1"
                                        />
                                    </div>
                                </div>

                                <div v-else class="w-full">
                                    <input
                                        v-model="field.value"
                                        class="input input-bordered w-full bg-base-100"
                                        :type="
                                            typeof field.value === 'number'
                                                ? 'number'
                                                : 'text'
                                        "
                                    />
                                </div>
                            </SettingCard>
                        </div>
                    </div>
                    <SettingCard
                        :delay="filteredSettingsEntries.length * 0.05"
                        layout="row"
                        :description="$t('settings.toast_position.description')"
                    >
                        <template #title>
                            <Info class="w-5 h-5 text-primary" />
                            {{ $t("settings.toast_position.title") }}
                        </template>
                        <div>
                            <AnimatedDropdown
                                v-model="toastPosition"
                                :options="toastPositionOptions"
                                direction="up"
                                @change="handleToastPositionChange"
                            />
                        </div>
                    </SettingCard>
                    <div
                        class="card bg-base-200 shadow-md border border-base-300"
                    >
                        <div class="card-body p-3">
                            <h2
                                class="card-title text-sm font-semibold text-primary-focus mb-2 flex items-center gap-2"
                            >
                                <SettingsIcon class="w-5 h-5" />
                                {{ $t("settings.actions") }}
                            </h2>
                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-2">
                                <button
                                    class="btn btn-neutral btn-sm w-full sm:w-auto flex items-center gap-2 hover:btn-primary-focus transition-all duration-200"
                                    @click="openDataFolder"
                                >
                                    <Folder class="w-4 h-4" />
                                    {{ $t("settings.open_data") }}
                                </button>
                                <button
                                    class="btn btn-neutral btn-sm w-full sm:w-auto flex items-center gap-2 hover:btn-primary-focus transition-all duration-200"
                                    @click="showChangeRootFolderDialog"
                                >
                                    <Folder class="w-4 h-4" />
                                    {{ $t("settings.change_root.action") }}
                                </button>
                                <button
                                    class="btn btn-neutral btn-sm w-full sm:w-auto flex items-center gap-2 hover:btn-secondary-focus transition-all duration-200"
                                    @click="resetRequirements"
                                >
                                    <RotateCcw class="w-4 h-4" />
                                    {{ $t("settings.reset_requirements") }}
                                </button>

                                <button
                                    class="btn btn-neutral btn-sm w-full sm:w-auto flex items-center gap-2 hover:btn-error-focus transition-all duration-200"
                                    @click="resetSettings"
                                >
                                    <RotateCcw class="w-4 h-4 text-warning" />
                                    {{ $t("settings.reset_settings") }}
                                </button>
                            </div>
                        </div>
                    </div>

                    <div
                        class="card bg-base-200 shadow-md border border-base-300"
                    >
                        <div class="card-body p-3">
                            <div class="flex items-center justify-between mb-3">
                                <h2
                                    class="card-title text-sm font-semibold text-primary-focus flex items-center gap-2"
                                >
                                    <HardDrive class="w-5 h-5" />
                                    {{ $t("settings.storage_usage") }}
                                </h2>
                                <button
                                    @click="loadStorageUsage"
                                    class="btn btn-ghost btn-xs btn-square"
                                >
                                    <RotateCcw
                                        class="w-3.5 h-3.5"
                                        :class="{
                                            'animate-spin': storageLoading,
                                        }"
                                    />
                                </button>
                            </div>

                            <div
                                v-if="storageLoading"
                                class="flex justify-center py-4"
                            >
                                <span
                                    class="loading loading-spinner loading-sm text-primary"
                                ></span>
                            </div>

                            <div v-else-if="storageUsage" class="space-y-2">
                                <div
                                    v-for="item in [
                                        {
                                            key: 'clients',
                                            label: $t(
                                                'settings.storage_clients'
                                            ),
                                            value: storageUsage.clients,
                                        },
                                        {
                                            key: 'libraries',
                                            label: $t(
                                                'settings.storage_libraries'
                                            ),
                                            value: storageUsage.libraries,
                                        },
                                        {
                                            key: 'natives',
                                            label: $t(
                                                'settings.storage_natives'
                                            ),
                                            value: storageUsage.natives,
                                        },
                                        {
                                            key: 'assets',
                                            label: $t(
                                                'settings.storage_assets'
                                            ),
                                            value: storageUsage.assets,
                                        },
                                        {
                                            key: 'java',
                                            label: $t('settings.storage_java'),
                                            value: storageUsage.java,
                                        },
                                        {
                                            key: 'other',
                                            label: $t('settings.storage_other'),
                                            value: storageUsage.other,
                                        },
                                    ].sort((a, b) => b.value - a.value)"
                                    :key="item.key"
                                    class="flex items-center justify-between text-xs"
                                >
                                    <span class="text-base-content/70">{{
                                        item.label
                                    }}</span>
                                    <div class="flex items-center gap-2">
                                        <div
                                            class="w-20 bg-base-300 rounded-full h-1.5"
                                        >
                                            <div
                                                class="bg-primary h-1.5 rounded-full transition-all"
                                                :style="{
                                                    width:
                                                        storageUsage.total > 0
                                                            ? `${Math.min(100, (item.value / storageUsage.total) * 100).toFixed(0)}%`
                                                            : '0%',
                                                }"
                                            ></div>
                                        </div>
                                        <span
                                            class="font-mono font-medium w-16 text-right"
                                            >{{ formatBytes(item.value) }}</span
                                        >
                                    </div>
                                </div>
                                <div
                                    class="border-t border-base-300 pt-2 flex items-center justify-between text-xs font-semibold"
                                >
                                    <span>{{
                                        $t("settings.storage_total")
                                    }}</span>
                                    <span class="font-mono text-primary">{{
                                        formatBytes(storageUsage.total)
                                    }}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div
                    v-else-if="activeTab === 'accounts'"
                    key="accounts"
                    class="space-y-3 overflow-x-hidden"
                >
                    <div
                        class="card bg-base-200 shadow-md border border-base-300"
                    >
                        <div class="card-body p-3">
                            <div
                                class="flex flex-col md:flex-row justify-between items-start md:items-center gap-3 mb-3"
                            >
                                <div>
                                    <h2
                                        class="card-title text-base font-semibold text-primary-focus flex items-center gap-2"
                                    >
                                        <User class="w-5 h-5" />
                                        {{ t("settings.accounts_management") }}
                                    </h2>
                                    <p
                                        class="text-sm text-base-content/70 mt-1"
                                    >
                                        {{ t("settings.accounts_description") }}
                                    </p>
                                </div>
                                <button
                                    @click="showAddAccountDialog"
                                    class="btn btn-primary btn-sm"
                                >
                                    <Plus class="w-4 h-4 mr-2" />
                                    {{ t("settings.add_account") }}
                                </button>
                            </div>

                            <div
                                v-if="false"
                                class="card bg-base-200 shadow-md border border-base-300 mb-3"
                            >
                                <div class="card-body p-3 space-y-3">
                                    <div
                                        class="flex flex-col gap-3 md:flex-row md:items-center md:justify-between"
                                    >
                                        <div>
                                            <h3
                                                class="font-semibold text-sm flex items-center gap-2"
                                            >
                                                <FolderSync
                                                    class="w-5 h-5 text-primary"
                                                />
                                                {{
                                                    t(
                                                        "settings.local_account_sync"
                                                    )
                                                }}
                                            </h3>
                                            <p
                                                class="text-sm text-base-content/70 mt-1"
                                            >
                                                {{
                                                    t(
                                                        "settings.local_account_sync_description"
                                                    )
                                                }}
                                            </p>
                                        </div>
                                        <!-- 
                                        :disabled="
                                            isAccountSyncing ||
                                            !isAuthenticated ||
                                            remoteAccountsLoading
                                        "

                                        this code from syncLocalAccountsFromCloud buttons
                                        -->
                                        <div class="flex flex-wrap gap-2">
                                            <button
                                                class="btn btn-outline btn-sm flex items-center gap-2"
                                                @click="
                                                    syncLocalAccountsFromCloud
                                                "
                                                disabled
                                            >
                                                <Download class="w-4 h-4" />
                                                {{
                                                    t(
                                                        "settings.sync_accounts_from_cloud_button"
                                                    )
                                                }}
                                            </button>
                                            <button
                                                class="btn btn-primary btn-sm flex items-center gap-2"
                                                @click="
                                                    syncLocalAccountsToCloud
                                                "
                                                disabled
                                            >
                                                <Upload class="w-4 h-4" />
                                                {{
                                                    t(
                                                        "settings.sync_accounts_to_cloud_button"
                                                    )
                                                }}
                                            </button>
                                        </div>
                                    </div>
                                    <div
                                        class="text-sm text-base-content/70 flex items-center gap-2"
                                    >
                                        <Cloud class="w-4 h-4 text-primary" />
                                        <span>
                                            <span v-if="isAccountSyncing">{{
                                                t("settings.syncing")
                                            }}</span>
                                            <span
                                                v-else-if="
                                                    remoteAccountsLoading
                                                "
                                            >
                                                {{
                                                    t(
                                                        "settings.remote_accounts_loading"
                                                    )
                                                }}
                                            </span>
                                            <span
                                                v-else-if="
                                                    remoteAccounts.length > 0
                                                "
                                            >
                                                {{
                                                    t(
                                                        "settings.remote_accounts_present",
                                                        {
                                                            count: remoteAccounts.length,
                                                        }
                                                    )
                                                }}
                                            </span>
                                            <span v-else>
                                                {{
                                                    t(
                                                        "settings.remote_accounts_empty"
                                                    )
                                                }}
                                            </span>
                                        </span>
                                    </div>
                                    <ul
                                        v-if="
                                            remoteAccounts.length > 0 &&
                                            !remoteAccountsLoading
                                        "
                                        class="text-sm text-base-content/70 space-y-1 pl-3 list-disc"
                                    >
                                        <li
                                            v-for="account in remoteAccounts.slice(
                                                0,
                                                3
                                            )"
                                            :key="account.id"
                                        >
                                            <span class="font-semibold">{{
                                                account.display_name
                                            }}</span>
                                        </li>
                                        <li
                                            v-if="remoteAccounts.length > 3"
                                            class="text-xs text-base-content/50"
                                        >
                                            {{
                                                t(
                                                    "settings.remote_accounts_more",
                                                    {
                                                        count:
                                                            remoteAccounts.length -
                                                            3,
                                                    }
                                                )
                                            }}
                                        </li>
                                    </ul>
                                    <div
                                        v-if="!isAuthenticated"
                                        class="alert alert-warning alert-sm rounded-md border border-base-300 p-3 text-sm flex items-center gap-2"
                                    >
                                        <LogIn class="w-4 h-4" />
                                        {{ t("settings.sync_login_required") }}
                                    </div>
                                </div>
                            </div>

                            <div class="flex flex-col sm:flex-row gap-3 mb-3">
                                <div class="relative flex-1">
                                    <input
                                        v-model="searchQuery"
                                        type="text"
                                        :placeholder="
                                            t('settings.search_accounts')
                                        "
                                        class="input input-bordered input-sm w-full pl-10"
                                    />
                                    <Search
                                        class="absolute left-3 top-1/2 transform -translate-y-1/2 w-4 h-4 z-50"
                                    />
                                </div>
                                <div
                                    class="flex flex-wrap gap-2"
                                    v-if="uniqueTags.length > 0"
                                >
                                    <button
                                        v-for="tag in uniqueTags"
                                        :key="tag"
                                        @click="selectTag(tag)"
                                        class="btn btn-sm"
                                        :class="{
                                            'btn-primary': selectedTag === tag,
                                            'btn-ghost bg-base-300':
                                                selectedTag !== tag,
                                        }"
                                    >
                                        {{ tag }}
                                    </button>
                                </div>
                            </div>

                            <div
                                class="grid grid-cols-1 lg:grid-cols-2 gap-3 overflow-hidden"
                            >
                                <AccountCard
                                    v-for="account in filteredAccounts"
                                    :key="account.id"
                                    :data-account-id="account.id"
                                    :account="account"
                                    :formatDate="formatDate"
                                    :isDragging="draggedId === account.id"
                                    :isDragOver="dragOverId === account.id"
                                    @drag-start="startDrag($event, account.id)"
                                    @set-active="setActiveAccount"
                                    @edit-account="editAccount"
                                    @delete-account="deleteAccount"
                                >
                                    <template #user-icon>
                                        <User class="w-4 h-4" />
                                    </template>
                                    <template #edit-icon>
                                        <Edit3 class="w-4 h-4" />
                                    </template>
                                    <template #delete-icon>
                                        <Trash2 class="w-4 h-4" />
                                    </template>
                                </AccountCard>
                            </div>

                            <div
                                v-if="filteredAccounts.length === 0"
                                class="text-center py-10 text-base-content/60 flex flex-col items-center"
                            >
                                <div class="text-5xl mb-3 opacity-30">
                                    <User v-if="accounts.length > 0" />
                                    <div v-else>
                                        <UserIcon class="w-4 h-4" />
                                    </div>
                                </div>
                                <h3 class="text-lg font-semibold mb-2">
                                    {{
                                        accounts.length === 0
                                            ? t("settings.no_accounts_title")
                                            : t("settings.no_matching_accounts")
                                    }}
                                </h3>
                                <p class="text-sm">
                                    {{
                                        accounts.length === 0
                                            ? t(
                                                  "settings.no_accounts_description"
                                              )
                                            : t("settings.try_different_search")
                                    }}
                                </p>
                            </div>
                        </div>
                    </div>
                </div>
            </transition>
        </div>
    </div>
</template>
<style scoped>
.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}

.tab {
    border-radius: var(--radius-box, 0.5rem) !important;
    overflow: hidden;
}

.tabs.tabs-boxed .tab {
    border-radius: var(--radius-box, 0.5rem) !important;
}

.tab.tab-active,
.tab-active {
    border-radius: var(--radius-box, 0.5rem) !important;
    overflow: hidden;
}

.tabs .tab,
.tabs .tab > * {
    border-radius: var(--radius-box, 0.5rem) !important;
}

.settings-card {
    opacity: 0;
    transform: translateY(10px);
    animation: fadeInUp 0.4s ease-out forwards;
}

@keyframes fadeInUp {
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.tab-switch-enter-active,
.tab-switch-leave-active {
    transition:
        opacity 0.3s ease,
        transform 0.3s ease;
}

.tab-switch-enter-from,
.tab-switch-leave-to {
    opacity: 0;
    transform: translateY(10px);
}

@keyframes scaleIn {
    0% {
        opacity: 0;
        transform: scale(0.9);
    }

    100% {
        opacity: 1;
        transform: scale(1);
    }
}

@keyframes scaleOut {
    0% {
        opacity: 1;
        transform: scale(1);
    }

    100% {
        opacity: 0;
        transform: scale(0.9);
    }
}

@keyframes fadeInUp {
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

html[data-theme="dark"] .discord-icon {
    filter: invert(100%);
}

html[data-theme="light"] .discord-icon {
    filter: invert(0%);
}

.spotlight-highlight {
    box-shadow:
        0 0 0 3px rgba(59, 130, 246, 0.15),
        0 6px 18px rgba(2, 6, 23, 0.04);
    border-color: rgba(59, 130, 246, 0.5) !important;
    transition:
        box-shadow 0.25s ease,
        border-color 0.25s ease;
    border-radius: 0.5rem;
    position: relative;
}

.spotlight-highlight::after {
    content: "";
    position: absolute;
    inset: 0;
    pointer-events: none;
    border-radius: inherit;
    box-shadow: 0 0 0 3px rgba(59, 130, 246, 0.12) inset;
}
</style>

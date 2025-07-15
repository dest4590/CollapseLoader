<script setup lang="ts">
import {
    ref,
    onMounted,
    onUnmounted,
    reactive,
    watch,
    computed,
    nextTick,
} from 'vue';
import { invoke } from '@tauri-apps/api/core';
import AnimatedSlider from '../components/ui/AnimatedSlider.vue';
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
} from 'lucide-vue-next';
import { useToast } from '../services/toastService';
import type { ToastPosition } from '../types/toast';
import { syncService, type SyncServiceState } from '../services/syncService';
import { globalUserStatus } from '../composables/useUserStatus';
import SyncStatus from '../components/common/SyncStatus.vue';
import AddAccountModal from '../components/modals/AddAccountModal.vue';
import EditAccountModal from '../components/modals/EditAccountModal.vue';
import ResetConfirmModal from '../components/modals/ResetConfirmModal.vue';
import TelemetryInfoModal from '../components/modals/TelemetryInfoModal.vue';
import DeleteAccountConfirmModal from '../components/modals/DeleteAccountConfirmModal.vue';
import {
    changeLanguage,
    getAvailableLanguages,
    getCurrentLanguage,
} from '../i18n';
import { useI18n } from 'vue-i18n';
import { useModal } from '../services/modalService';

interface Setting<T> {
    value: T;
    show: boolean;
}

interface Settings {
    [key: string]: Setting<any>;
}

interface Account {
    id: string;
    username: string;
    tags: string[];
    created_at: string;
    last_used?: string;
    is_active: boolean;
}

interface Flags {
    [key: string]: any;
}

const settings = reactive<Settings>({});
const accounts = ref<Account[]>([]);
const activeTab = ref<'general' | 'sync' | 'accounts'>('general');
const loading = ref(true);
const isSaving = ref(false);
const isRefreshing = ref(false);
const { addToast, setToastPosition, getToastPosition } = useToast();
let saveTimeout: number | null = null;

const editingAccount = ref<Account | null>(null);
const accountToDelete = ref<Account | null>(null);

const { showModal } = useModal();

const ramOptions = [
    { mb: 2048, label: '2 GB' },
    { mb: 4096, label: '4 GB' },
    { mb: 6144, label: '6 GB' },
    { mb: 8192, label: '8 GB' },
    { mb: 16384, label: '16 GB' },
    { mb: 32768, label: '32 GB' },
];
const ramOptionIndex = ref(0);

const flags = reactive<Flags>({});

const { t } = useI18n();
const availableLanguages = getAvailableLanguages();
const currentLanguage = ref(getCurrentLanguage());

const isAuthenticated = computed(() => globalUserStatus.isAuthenticated.value);

const filteredSettingsEntries = computed(() => {
    return Object.entries(settings).filter(([, field]) => field.show);
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
        const loadedSettings = await invoke<Settings>('get_settings');

        Object.keys(settings).forEach((key) => delete settings[key]);

        Object.entries(loadedSettings).forEach(([key, value]) => {
            settings[key] = value;
        });
    } catch (error) {
        console.error('Failed to load settings:', error);
        addToast(t('settings.load_settings_failed', { error }), 'error');
    } finally {
        loading.value = false;
    }
};

const loadAccounts = async (skipLoading = false) => {
    try {
        if (!skipLoading) {
            isRefreshing.value = true;
        }

        const fetchedAccounts = await invoke<Account[]>('get_accounts');

        await nextTick();
        accounts.value = fetchedAccounts;

        if (accounts.value.length === 0) {
            const { username, tags } = generateRandomAccount();
            try {
                const newAccountId = await invoke<string>('add_account', {
                    username,
                    tags,
                });

                await invoke('set_active_account', { id: newAccountId });
                accounts.value = await invoke<Account[]>('get_accounts');
            } catch (error) {
                console.error('Failed to create default account:', error);
            }
        }
    } catch (error) {
        console.error('Failed to load accounts:', error);
        addToast(t('settings.load_accounts_failed', { error }), 'error');
    } finally {
        if (!skipLoading) {
            isRefreshing.value = false;
        }
    }
};

const loadFlags = async () => {
    try {
        const loadedFlags = await invoke<Flags>('get_flags');
        Object.keys(flags).forEach((key) => delete flags[key]);
        Object.entries(loadedFlags).forEach(([key, value]) => {
            flags[key] = value;
        });
    } catch (error) {
        console.error('Failed to load flags:', error);
        addToast(t('settings.load_flags_failed', { error }), 'error');
    }
};

const saveSettings = async () => {
    try {
        isSaving.value = true;
        await invoke('save_settings', { inputSettings: settings });
    } catch (error) {
        console.error('Failed to save settings:', error);
        addToast(t('settings.save_settings_failed', { error }), 'error');
    } finally {
        isSaving.value = false;
    }
};

const debouncedSave = () => {
    if (saveTimeout) {
        clearTimeout(saveTimeout);
    }
    saveTimeout = setTimeout(() => {
        saveSettings();
    }, 500) as unknown as number;
};

watch(
    settings,
    () => {
        if (!loading.value) {
            debouncedSave();
        }
    },
    { deep: true }
);

const showAddAccountDialog = () => {
    showModal(
        'add-account',
        AddAccountModal,
        {
            title: t('settings.add_account_title'),
        },
        {},
        {
            'account-added': handleAccountAdded,
        }
    );
};

const showEditAccountDialog = (account: Account) => {
    editingAccount.value = account;
    showModal(
        'edit-account',
        EditAccountModal,
        {
            title: t('settings.edit_account_title'),
        },
        { account },
        {
            'account-updated': handleAccountUpdated,
        }
    );
};

const showResetConfirmDialog = () => {
    showModal(
        'reset-confirm',
        ResetConfirmModal,
        {
            title: t('settings.reset_title'),
        },
        {},
        {
            'settings-reset': handleSettingsReset,
        }
    );
};

const showDeleteConfirmDialog = (account: Account) => {
    accountToDelete.value = account;
    showModal(
        'delete-confirm',
        DeleteAccountConfirmModal,
        {
            title: t('settings.delete_account_title'),
        },
        { account },
        {
            'account-deleted': handleAccountDeleted,
        }
    );
};

const showTelemetryModal = () => {
    showModal(
        'telemetry-info',
        TelemetryInfoModal,
        {
            title: t('settings.telemetry_info_title'),
        },
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
        await invoke('set_active_account', { id: account.id });
        await loadAccounts(true);
        addToast(
            t('settings.account_set_active', { username: account.username }),
            'success'
        );
    } catch (error) {
        console.error('Failed to set active account:', error);
        addToast(t('settings.account_set_active_failed', { error }), 'error');
    }
};

const formatDate = (dateString: string) => {
    try {
        const date = new Date(dateString);
        const day = String(date.getDate()).padStart(2, '0');
        const month = String(date.getMonth() + 1).padStart(2, '0');
        const year = date.getFullYear();
        const hours = String(date.getHours()).padStart(2, '0');
        const minutes = String(date.getMinutes()).padStart(2, '0');

        return `${day}/${month}/${year} ${hours}:${minutes}`;
    } catch (e) {
        console.error('Invalid date string:', dateString);
        return 'N/A';
    }
};

const getFormattedLabel = (key: string) => {
    const words = key.split('_');

    if (key === 'ram') {
        return 'RAM';
    }

    if (key == "irc_chat") {
        return 'IRC Chat';
    }

    if (key === 'hash_verify') {
        return 'Hash Verification';
    }

    if (key === 'discord_rpc_enabled') {
        return 'Discord Rich Presence';
    }

    if (key === 'enable_telemetry') {
        return t('settings.telemetry');
    }

    return words
        .map((word) => word.charAt(0).toUpperCase() + word.slice(1))
        .join(' ');
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
    return { username, tags: ['auto-generated'] };
};

const selectedTag = ref<string | null>(null);

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
    if (!selectedTag.value) {
        return accounts.value;
    }

    return accounts.value.filter((account) =>
        account.tags.includes(selectedTag.value!)
    );
});

const selectTag = (tag: string) => {
    if (selectedTag.value === tag) {
        selectedTag.value = null;
    } else {
        selectedTag.value = tag;
    }
};

const syncState = ref<SyncServiceState>(syncService.getState());
let unsubscribeSyncService: (() => void) | null = null;

const handleUploadToCloud = async () => {
    if (typeof t !== 'function') {
        console.error('Translation function t is not available');
        addToast('Sync failed: Translation service not ready', 'error');
        return;
    }

    const toastAdapter = (message: string, type: string, duration?: number) => {
        addToast(message, type as any, duration);
    };

    await syncService.manualSync(toastAdapter, t);
};

const handleDownloadFromCloud = async () => {
    try {
        await syncService.downloadFromCloud();
        addToast(t('settings.download_success'), 'success');
        await loadSettings();
        await loadAccounts();
    } catch (error) {
        addToast(t('settings.download_failed', { error }), 'error');
    }
};

const toggleAutoSync = (event: Event) => {
    const target = event.target as HTMLInputElement;
    syncService.setAutoSyncEnabled(target.checked);
};

const handleLanguageChange = async (languageCode: string) => {
    try {
        currentLanguage.value = languageCode as any;
        await changeLanguage(languageCode);
    } catch (error) {
        console.error('Failed to change language:', error);
        addToast(t('settings.language_change_failed', { error }), 'error');
    }
};

const getSettingDescription = (key: string) => {
    return t(`settings.descriptions.${key}`);
};

const openDataFolder = async () => {
    await invoke('open_data_folder');
};

const resetRequirements = async () => {
    try {
        await invoke('reset_requirements');
        addToast(t('settings.reset_requirements_success'), 'success');
    } catch (error) {
        console.error('Failed to reset requirements:', error);
        addToast(t('settings.reset_requirements_failed'), 'error');
    }
};

const resetCache = async () => {
    try {
        await invoke('reset_cache');
        addToast(t('settings.reset_cache_success'), 'success');
    } catch (error) {
        console.error('Failed to reset cache:', error);
        addToast(t('settings.reset_cache_failed'), 'error');
    }
};

onMounted(async () => {
    unsubscribeSyncService = syncService.subscribe((state) => {
        syncState.value = state;
    });

    await syncService.initializeSyncStatus();

    await loadSettings();
    await loadFlags();
    await loadAccounts();
});

onUnmounted(() => {
    if (unsubscribeSyncService) {
        unsubscribeSyncService();
    }
});

watch(
    () => settings.enable_telemetry?.value,
    async (newValue) => {
        if (loading.value) return;
        if (newValue === undefined) return;

        try {
            await invoke('set_optional_telemetry', { enabled: newValue });
        } catch (error) {
            console.error('Failed to update telemetry setting:', error);
            addToast(
                newValue
                    ? t('toast.telemetry.analytics_enable_failed', { error })
                    : t('toast.telemetry.analytics_disable_failed', { error }),
                'error'
            );

            if (settings.enable_telemetry) {
                settings.enable_telemetry.value = !newValue;
            }
        }
    }
);

const toastPosition = ref<ToastPosition>(getToastPosition());

const toastPositionOptions = [
    { value: 'bottom-right', label: t('settings.toast_position.bottom_right') },
    { value: 'bottom-left', label: t('settings.toast_position.bottom_left') },
    { value: 'top-right', label: t('settings.toast_position.top_right') },
    { value: 'top-left', label: t('settings.toast_position.top_left') },
    {
        value: 'bottom-center',
        label: t('settings.toast_position.bottom_center'),
    },
    { value: 'top-center', label: t('settings.toast_position.top_center') },
];

const handleToastPositionChange = (position: ToastPosition) => {
    toastPosition.value = position;
    setToastPosition(position);

    addToast(t('settings.toast_position.preview_message'), 'info', 3000);
};
</script>

<template>
    <div class="max-w-4xl mx-auto slide-up">
        <div v-if="!loading" class="transition-opacity duration-300">
            <div role="tablist" class="tabs tabs-boxed mb-5 flex justify-center gap-4">
                <a @click="activeTab = 'general'" class="tab transition-all duration-300 z-50" :class="{
                    'tab-active transform scale-105 shadow-md bg-base-300':
                        activeTab === 'general',
                    'hover:bg-base-300': activeTab !== 'general',
                }">
                    <div class="tooltip tooltip-bottom mr-1" :data-tip="$t('settings.reset_tooltip')">
                        <button class="btn btn-circle btn-xs btn-ghost" @click.stop="resetSettings">
                            <RotateCcw class="w-3.5 h-3.5 text-warning hover:text-warning transition-colors" />
                        </button>
                    </div>
                    {{ t('settings.general') }}
                </a>
                <a v-if="isAuthenticated" @click="activeTab = 'sync'" class="tab transition-all duration-300" :class="{
                    'tab-active transform scale-105 shadow-md bg-base-300':
                        activeTab === 'sync',
                    'hover:bg-base-300': activeTab !== 'sync',
                }">
                    <Cloud class="w-4 h-4 mr-2" />
                    {{ t('settings.sync') }}
                </a>
                <a @click="activeTab = 'accounts'" class="tab transition-all duration-300" :class="{
                    'tab-active transform scale-105 shadow-md bg-base-300':
                        activeTab === 'accounts',
                    'hover:bg-base-300': activeTab !== 'accounts',
                }">
                    <User class="w-4 h-4 mr-2" />
                    {{ t('settings.accounts') }}
                </a>
            </div>

            <transition name="tab-switch" mode="out-in">
                <div v-if="activeTab === 'general'" key="general" class="space-y-6">
                    <div v-for="([key, field], index) in filteredSettingsEntries" :key="key"
                        class="card bg-base-200 shadow-md border border-base-300 settings-card"
                        :style="{ 'animation-delay': index * 0.05 + 's' }">
                        <div class="card-body p-4">
                            <h2 class="card-title text-base font-semibold text-primary-focus mb-2">
                                <MemoryStick v-if="key === 'ram'" class="w-5 h-5 text-primary" />
                                <Languages v-if="key === 'language'" class="w-5 h-5 text-primary" />
                                <img src="@/assets/icons/discord.svg" v-if="key === 'discord_rpc_enabled'"
                                    class="w-5 h-5 discord-icon" />
                                <ChartNoAxesCombined v-if="key === 'optional_telemetry'" class="w-5 h-5 text-primary" />
                                <Waypoints v-if="key === 'cordshare'" class="w-5 h-5 text-primary" />
                                <MessagesSquare v-if="key === 'irc_chat'" class="w-5 h-5 text-primary" />
                                <BadgeCheck v-if="key === 'hash_verify'" class="w-5 h-5 text-primary" />
                                {{ getFormattedLabel(key) }}

                                <div v-if="key === 'optional_telemetry'" class="tooltip tooltip-top" :data-tip="$t('settings.telemetry_info_title')
                                    ">
                                    <Info class="w-5 h-5 text-primary cursor-pointer" @click="showTelemetryModal" />
                                </div>
                            </h2>

                            <div v-if="key === 'ram'" class="space-y-3">
                                <div class="flex items-center gap-2">
                                    <AnimatedSlider v-model="ramOptionIndex" :min="0" :max="ramOptions.length - 1"
                                        @update:modelValue="handleSliderChange" class="flex-grow" />
                                    <div class="flex items-center gap-2 rounded-md p-2">
                                        <input v-if="settings.ram" v-model.number="settings.ram.value" type="number"
                                            min="512" step="512"
                                            class="input input-bordered input-xs bg-transparent h-6"
                                            style="width: 60px" />
                                    </div>
                                </div>
                                <p class="text-xs text-base-content/70">
                                    {{ getSettingDescription(key) }}
                                </p>
                            </div>
                            <div v-else-if="key === 'language'" class="space-y-3">
                                <select :value="currentLanguage" @change="
                                    handleLanguageChange(
                                        ($event.target as HTMLSelectElement)
                                            .value
                                    )
                                    " class="select select-bordered w-full bg-base-100">
                                    <option v-for="lang in availableLanguages" :key="lang.code" :value="lang.code">
                                        {{ lang.nativeName }} ({{ lang.name }})
                                    </option>
                                </select>
                                <p class="text-xs text-base-content/70">
                                    {{ getSettingDescription(key) }}
                                </p>
                            </div>

                            <div v-else-if="typeof field.value === 'boolean'" class="space-y-3">
                                <div class="flex items-center justify-between">
                                    <div class="flex-1">
                                        <p class="text-xs text-base-content/70">
                                            {{ getSettingDescription(key) }}
                                        </p>
                                    </div>

                                    <input type="checkbox" v-if="!isFeatureOnline(key) || isAuthenticated"
                                        v-model="field.value" class="toggle toggle-primary" />

                                    <div v-else class="tooltip tooltip-left" :data-tip="$t('settings.login_required')">
                                        <input type="checkbox" :checked="false" class="toggle toggle-primary pointer-events-none" tabindex="-1" />
                                    </div>
                                </div>
                            </div>

                            <div v-else class="form-control w-full space-y-2">
                                <input v-model="field.value" class="input input-bordered w-full bg-base-100" :type="typeof field.value === 'number'
                                    ? 'number'
                                    : 'text'
                                    " />
                                <label class="label">
                                    <span class="label-text-alt text-base-content/70 text-xs">{{
                                        getSettingDescription(key) }}</span>
                                </label>
                            </div>
                        </div>
                    </div>
                    <div class="card bg-base-200 shadow-md border border-base-300 settings-card" :style="{
                        'animation-delay':
                            filteredSettingsEntries.length * 0.05 + 's',
                    }">
                        <div class="card-body p-4">
                            <h2 class="card-title text-base font-semibold text-primary-focus mb-2">
                                <Info class="w-5 h-5 text-primary" />
                                {{ $t('settings.toast_position.title') }}
                            </h2>

                            <div class="space-y-3">
                                <select v-model="toastPosition" @change="
                                    handleToastPositionChange(toastPosition)
                                    " class="select select-bordered w-full bg-base-100">
                                    <option v-for="option in toastPositionOptions" :key="option.value"
                                        :value="option.value">
                                        {{ option.label }}
                                    </option>
                                </select>
                                <p class="text-xs text-base-content/70">
                                    {{
                                        $t(
                                            'settings.toast_position.description'
                                        )
                                    }}
                                </p>
                            </div>
                        </div>
                    </div>
                    <div class="card bg-base-200 shadow-md border border-base-300">
                        <div class="card-body p-4">
                            <h2
                                class="card-title text-lg font-semibold text-primary-focus mb-4 flex items-center gap-2">
                                <SettingsIcon class="w-5 h-5" />
                                {{ $t('settings.actions') }}
                            </h2>
                            <div class="grid grid-cols-1 sm:grid-cols-2 gap-4">
                                <button
                                    class="btn btn-neutral btn-sm w-full sm:w-auto flex items-center gap-2 hover:btn-primary-focus transition-all duration-200"
                                    @click="openDataFolder">
                                    <Folder class="w-4 h-4" />
                                    {{ $t('settings.open_data') }}
                                </button>
                                <button
                                    class="btn btn-neutral btn-sm w-full sm:w-auto flex items-center gap-2 hover:btn-secondary-focus transition-all duration-200"
                                    @click="resetRequirements">
                                    <RotateCcw class="w-4 h-4" />
                                    {{ $t('settings.reset_requirements') }}
                                </button>
                                <button class="btn btn-neutral btn-sm w-full sm:w-auto flex items-center gap-2 hover:btn-secondary-focus transition-all duration-200"
                                    @click="resetCache">
                                    <RotateCcw class="w-4 h-4" />
                                    {{ $t('settings.reset_cache') }}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>

                <div v-else-if="activeTab === 'sync'" key="sync" class="space-y-6">
                    <div class="card bg-base-200 shadow-md border border-base-300">
                        <div class="card-body p-4">
                            <h2
                                class="card-title text-lg font-semibold text-primary-focus mb-4 flex items-center gap-2">
                                <Cloud class="w-5 h-5" />
                                {{ t('settings.sync_status') }}
                            </h2>
                            <SyncStatus />
                        </div>
                    </div>

                    <div class="card bg-base-200 shadow-md border border-base-300">
                        <div class="card-body p-4">
                            <h2
                                class="card-title text-lg font-semibold text-primary-focus mb-4 flex items-center gap-2">
                                <SettingsIcon class="w-5 h-5" />
                                {{ t('settings.sync_controls') }}
                            </h2>

                            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                                <div class="p-4 border border-base-300 rounded-lg hover:bg-base-100 transition-colors">
                                    <div class="flex items-center gap-3 mb-3">
                                        <Upload class="w-5 h-5 text-primary" />
                                        <h3 class="font-semibold">
                                            {{ t('settings.upload_title') }}
                                        </h3>
                                    </div>
                                    <p class="text-sm text-base-content/70 mb-3">
                                        {{ t('settings.upload_description') }}
                                    </p>
                                    <button @click="handleUploadToCloud" class="btn btn-primary btn-sm w-full"
                                        :disabled="syncState.isSyncing ||
                                            !syncState.isOnline
                                            ">
                                        <Upload class="w-4 h-4 mr-2" />
                                        {{
                                            syncState.isSyncing
                                                ? t('settings.syncing')
                                                : t('settings.upload_button')
                                        }}
                                    </button>
                                </div>

                                <div class="p-4 border border-base-300 rounded-lg hover:bg-base-100 transition-colors">
                                    <div class="flex items-center gap-3 mb-3">
                                        <Download class="w-5 h-5" />
                                        <h3 class="font-semibold">
                                            {{ t('settings.download_title') }}
                                        </h3>
                                    </div>
                                    <p class="text-sm text-base-content/70 mb-3">
                                        {{ t('settings.download_description') }}
                                    </p>
                                    <button @click="handleDownloadFromCloud" class="btn btn-primary btn-sm w-full"
                                        :disabled="syncState.isSyncing ||
                                            !syncState.isOnline ||
                                            !syncState.hasCloudData
                                            ">
                                        <Download class="w-4 h-4 mr-2" />
                                        {{
                                            syncState.isSyncing
                                                ? t('settings.syncing')
                                                : t('settings.download_button')
                                        }}
                                    </button>
                                </div>
                            </div>

                            <div class="mt-6 p-4 border border-base-300 rounded-lg">
                                <div class="flex items-center justify-between">
                                    <div>
                                        <h3 class="font-semibold mb-1">
                                            {{ t('settings.auto_sync') }}
                                        </h3>
                                        <p class="text-sm text-base-content/70">
                                            {{
                                                t(
                                                    'settings.auto_sync_description'
                                                )
                                            }}
                                        </p>
                                    </div>
                                    <input type="checkbox" :checked="syncState.autoSyncEnabled" @change="toggleAutoSync"
                                        class="toggle toggle-primary" />
                                </div>
                            </div>

                            <div v-if="!syncState.isOnline" class="mt-4 alert alert-warning">
                                <div class="flex items-center gap-2">
                                    <div class="w-4 h-4 rounded-full bg-error"></div>
                                    <span>{{
                                        t('settings.offline_warning')
                                    }}</span>
                                </div>
                            </div>

                            <div v-else-if="!syncState.hasCloudData" class="mt-4 alert alert-info">
                                <div class="flex items-center gap-2">
                                    <Cloud class="w-4 h-4" />
                                    <span>{{
                                        t('settings.no_cloud_data')
                                        }}</span>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div v-else key="accounts" class="space-y-6 overflow-x-hidden">
                    <div class="flex justify-between items-center mb-4">
                        <div class="flex flex-wrap gap-2">
                            <button v-for="tag in uniqueTags" :key="tag" @click="selectTag(tag)" class="btn btn-sm"
                                :class="{
                                    'btn-primary': selectedTag === tag,
                                    'btn-outline btn-secondary':
                                        selectedTag !== tag,
                                }">
                                {{ tag }}
                            </button>
                        </div>
                        <button @click="showAddAccountDialog" class="btn btn-primary btn-sm">
                            <Plus class="w-4 h-4 mr-2" />
                            {{ t('settings.add_account') }}
                        </button>
                    </div>

                    <transition-group name="account-list" tag="div" class="grid grid-cols-1 gap-4 overflow-hidden">
                        <div v-for="account in filteredAccounts" :key="account.id"
                            class="card bg-base-200 shadow-md border border-base-300 account-card overflow-x-hidden">
                            <div class="card-body p-4">
                                <div class="flex justify-between items-center">
                                    <div class="flex-1 space-y-1 min-w-0 pr-2">
                                        <div class="flex items-center gap-2">
                                            <h3 class="font-semibold text-lg text-primary-focus truncate">
                                                {{ account.username }}
                                            </h3>
                                            <div v-if="account.is_active"
                                                class="badge badge-success badge-sm whitespace-nowrap">
                                                {{ t('settings.active') }}
                                            </div>
                                        </div>
                                        <p class="text-sm text-base-content/70 flex items-center gap-2">
                                            <span class="font-medium text-primary/80">{{ t('settings.tags') }}:</span>
                                            <span class="flex flex-wrap gap-1">
                                                <span v-for="tag in account.tags" :key="tag"
                                                    class="badge badge-outline badge-xs">
                                                    {{ tag }}
                                                </span>
                                            </span>
                                        </p>
                                        <p class="text-xs text-base-content/60 flex items-center gap-2">
                                            <span>{{ t('settings.created') }}:
                                                {{
                                                    formatDate(
                                                        account.created_at
                                                    )
                                                }}</span>
                                            <span v-if="account.last_used" class="border-l border-base-content/30 pl-2">
                                                {{ t('settings.last_used') }}:
                                                {{
                                                    formatDate(
                                                        account.last_used
                                                    )
                                                }}
                                            </span>
                                        </p>
                                    </div>
                                    <div class="flex items-center space-x-2">
                                        <button @click="setActiveAccount(account)" class="btn btn-sm btn-circle" :class="account.is_active
                                            ? 'btn-success text-success-content'
                                            : 'btn-outline btn-success hover:text-success-content'
                                            ">
                                            <User class="w-4 h-4" />
                                        </button>
                                        <button @click="editAccount(account)"
                                            class="btn btn-sm btn-circle btn-outline btn-warning hover:text-warning-content">
                                            <Edit3 class="w-4 h-4" />
                                        </button>
                                        <button @click="deleteAccount(account)"
                                            class="btn btn-sm btn-circle btn-outline btn-error hover:text-error-content">
                                            <Trash2 class="w-4 h-4" />
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </transition-group>

                    <div v-if="accounts.length === 0 && !isRefreshing" class="text-center py-10 text-base-content/60">
                        <div class="text-5xl mb-3 opacity-30">👤</div>
                        <h3 class="text-lg font-semibold mb-2">
                            {{ t('settings.no_accounts_title') }}
                        </h3>
                        <p class="text-sm">
                            {{ t('settings.no_accounts_description') }}
                        </p>
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
    border-radius: 4rem;
}

@keyframes slideIn {
    0% {
        opacity: 0;
        transform: translateY(10px);
    }

    100% {
        opacity: 1;
        transform: translateY(0);
    }
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

.confirm-dialog-enter-active {
    animation: scaleIn 0.3s ease-out forwards;
}

.confirm-dialog-leave-active {
    animation: scaleOut 0.2s ease-in forwards;
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

.account-list-enter-active,
.account-list-leave-active {
    transition:
        opacity 0.3s ease,
        transform 0.3s ease;
}

.account-list-enter-from,
.account-list-leave-to {
    opacity: 0;
    transform: translateY(10px);
}

.account-list-move {
    transition: transform 0.5s ease;
}

.account-list-enter-active {
    position: relative;
    z-index: 1;
    transition: all 0.3s ease-out;
}

.account-list-enter-from {
    opacity: 0;
    transform: translateY(10px);
}

.account-list-leave-active {
    position: absolute;
    transition: all 0.3s ease-in;
    width: 100%;
}

.account-list-leave-to {
    opacity: 0;
    transform: translateY(-10px);
}

html[data-theme='dark'] .discord-icon {
    filter: invert(100%) sepia(15%) saturate(1%) hue-rotate(282deg) brightness(102%) contrast(101%);
}

html[data-theme='light'] .discord-icon {
    filter: invert(0%) sepia(15%) saturate(17%) hue-rotate(253deg) brightness(95%) contrast(103%);
}
</style>

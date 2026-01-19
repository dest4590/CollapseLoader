<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { onBeforeUnmount, onMounted, ref, computed, watch, shallowRef, nextTick } from 'vue';
import {
    RefreshCcw,
    Folder,
    Copy,
    Trash2,
    Star,
    Download,
    Newspaper,
    StopCircle,
    FileText,
    X
} from 'lucide-vue-next';
import SearchBar from '../components/common/SearchBar.vue';
import ClientCard from '../components/features/clients/ClientCard.vue';
import FiltersMenu from '../components/common/FiltersMenu.vue';
import { useToast } from '../services/toastService';
import { useModal } from '../services/modalService';
import { syncService } from '../services/syncService';
import { useI18n } from 'vue-i18n';
import type { Client, InstallProgress } from '../types/ui';
import type { CustomClient } from '../types/ui';
import LogViewerModal from '../components/modals/clients/LogViewerModal.vue';
import InsecureClientWarningModal from '../components/modals/clients/InsecureClientWarningModal.vue';
import InlineIRCChat from '../components/features/social/InlineIRCChat.vue';
import { isHalloweenEvent } from '../utils/events';

interface Account {
    id: string;
    username: string;
    tags: string[];
    created_at: string;
    last_used?: string;
    is_active: boolean;
}

interface Filters {
    fabric: boolean;
    vanilla: boolean;
    forge: boolean;
    installed: boolean;
}

const { t } = useI18n();
const halloweenActive = ref(isHalloweenEvent());

const props = defineProps<{
    isOnline: boolean;
    userId?: number | null;
    unreadNewsCount?: number;
}>();

defineEmits<{
    'logged-out': [];
    'logged-in': [];
    registered: [];
    'change-view': [view: string];
    'show-user-profile': [userId: number];
    'back-to-friends': [];
    'unread-count-updated': [count: number];
}>();

const clients = shallowRef<Client[]>([]);
const customClients = shallowRef<CustomClient[]>([]);
const customClientsDisplayMode = ref<'global' | 'separate'>('separate');
const favoriteClients = shallowRef<number[]>([]);
const error = ref('');
const clientsLoaded = ref(false);
const runningClients = shallowRef<number[]>([]);
const skipNextRunningCheck = ref<Set<number>>(new Set());
const isLeaving = ref(false);
const viewVisible = ref(false);
const { addToast } = useToast();
const { showModal, hideModal } = useModal();
const statusInterval = ref<number | null>(null);
const searchBarRef = ref<any>(null);

const HOME_ANIM_KEY = 'homeAnimPlayed';
const hasAnimatedBefore = ref<boolean>(false);
try {
    hasAnimatedBefore.value = sessionStorage.getItem(HOME_ANIM_KEY) === '1';
} catch (e) {
    console.error('Failed to read sessionStorage:', e);
    hasAnimatedBefore.value = false;
}
if (hasAnimatedBefore.value) {
    viewVisible.value = true;
}

const playClientSlideAnim = ref(!hasAnimatedBefore.value);

const STAGGER_KEY = 'staggerCardsPlayed';
const hasStaggerPlayed = ref<boolean>(false);
try {
    hasStaggerPlayed.value = sessionStorage.getItem(STAGGER_KEY) === '1';
} catch (e) {
    console.error('Failed to read sessionStorage:', e);
    hasStaggerPlayed.value = false;
}

const accounts = ref<Account[]>([]);
const selectedAccountId = ref<string>('');
const searchQuery = ref('');
let searchDebounceTimer: number | null = null;
const debouncedSearchQuery = ref('');
const ACTIVE_FILTERS_KEY = 'homeActiveFilters';
const CLIENT_SORT_KEY = 'homeClientSortKey';
const CLIENT_SORT_ORDER_KEY = 'homeClientSortOrder';

let initialFilters: Filters = {
    fabric: false,
    vanilla: false,
    forge: false,
    installed: false
};

try {
    const stored = localStorage.getItem(ACTIVE_FILTERS_KEY);
    if (stored) {
        initialFilters = JSON.parse(stored);
    }
} catch (e) {
    console.error('Failed to read active filters from localStorage:', e);
}

const activeFilters = ref<Filters>(initialFilters);

let initialSortKey: 'popularity' | 'name' | 'newest' | 'version' | 'rating' = 'popularity';

try {
    const stored = localStorage.getItem(CLIENT_SORT_KEY);
    if (stored) initialSortKey = stored as any;
} catch (e) {
    console.error('Failed to read client sort key from localStorage:', e);
}

const clientSortKey = ref<'popularity' | 'name' | 'newest' | 'version' | 'rating'>(initialSortKey);

let initialSortOrder: 'asc' | 'desc' = 'desc';
try {
    const stored = localStorage.getItem(CLIENT_SORT_ORDER_KEY);
    if (stored) initialSortOrder = stored as any;
} catch (e) {
    console.error('Failed to read client sort order from localStorage:', e);
}
const clientSortOrder = ref<'asc' | 'desc'>(initialSortOrder);

const clearAllFilters = () => {
    searchQuery.value = '';
    debouncedSearchQuery.value = '';
    activeFilters.value = {
        fabric: false,
        vanilla: false,
        forge: false,
        installed: false
    };
};

watch(
    activeFilters,
    (val) => {
        try {
            localStorage.setItem(ACTIVE_FILTERS_KEY, JSON.stringify(val));
        } catch (e) {
            console.error('Failed to save active filters to localStorage:', e);
        }
    },
    { deep: true, flush: 'post' }
);

watch(clientSortKey, (val) => {
    try {
        localStorage.setItem(CLIENT_SORT_KEY, val);
    } catch (e) {
        console.error('Failed to save client sort key to localStorage:', e);
    }
});

watch(clientSortOrder, (val) => {
    try {
        localStorage.setItem(CLIENT_SORT_ORDER_KEY, val);
    } catch (e) {
        console.error('Failed to save client sort order to localStorage:', e);
    }
});

const installationStatus = ref<Map<string, InstallProgress>>(new Map());
const eventListeners = ref<(() => void)[]>([]);
const requirementsInProgress = ref<boolean>(false);
const warnedInsecureClients = ref<Set<number>>(new Set());
const hashVerifyingClients = ref<Set<number>>(new Set());

const progressTargets = ref<Map<string, number>>(new Map());
const progressAnimHandles = ref<Map<string, number>>(new Map());

const easeOutQuad = (t: number) => 1 - (1 - t) * (1 - t);

const smoothUpdateProgress = (file: string, targetPercentage: number, action: string) => {
    const current = installationStatus.value.get(file)?.percentage ?? 0;
    const safeTarget = Math.max(current, Math.min(100, Math.floor(targetPercentage)));

    progressTargets.value.set(file, safeTarget);

    if (progressAnimHandles.value.has(file)) return;

    const startTime = performance.now();
    const startValue = current;

    const step = (now: number) => {
        const target = progressTargets.value.get(file) ?? safeTarget;
        const elapsed = now - startTime;
        const duration = Math.max(120, (target - startValue) * 5);
        const t = Math.min(1, elapsed / duration);
        const eased = easeOutQuad(t);
        const value = Math.floor(startValue + (target - startValue) * eased);

        installationStatus.value.set(file, {
            percentage: Math.min(100, value),
            action,
            isComplete: false,
        });

        if (value >= target && target >= 100) {
            progressAnimHandles.value.delete(file);
            return;
        }

        if (value >= target) {
            progressAnimHandles.value.delete(file);
            window.setTimeout(() => {
                if ((progressTargets.value.get(file) ?? target) > value) {
                    smoothUpdateProgress(file, progressTargets.value.get(file) ?? target, action);
                }
            }, 10);
            return;
        }

        const handle = window.requestAnimationFrame(step);
        progressAnimHandles.value.set(file, handle);
    };

    const handle = window.requestAnimationFrame(step);
    progressAnimHandles.value.set(file, handle);
};

const contextMenu = ref({
    visible: false,
    x: 0,
    y: 0,
    client: null as Client | null,
    isAnimating: false,
    animationClass: '',
    showAccountsDropdown: false,
});

const selectedClients = shallowRef<Set<number>>(new Set());
const isCtrlPressed = ref(false);
let ctrlPressTimer: number | null = null;

const blurHandler = () => {
    if (ctrlPressTimer !== null) {
        clearTimeout(ctrlPressTimer);
        ctrlPressTimer = null;
    }
    isCtrlPressed.value = false;
};

const visibilityHandler = () => {
    if (document.hidden) {
        if (ctrlPressTimer !== null) {
            clearTimeout(ctrlPressTimer);
            ctrlPressTimer = null;
        }
        isCtrlPressed.value = false;
    }
};
const expandedClientId = ref<number | null>(null);

const isAnyCardExpanded = computed(() => {
    return expandedClientId.value !== null;
});

const handleLaunchClick = (client: Client) => {
    if (client.meta.is_custom) {
        handleLaunchCustomClient(client);
        return;
    }

    if (isClientRunning(client.id)) {
        stopClient(client.id);
        return;
    }

    if (client.insecure && !warnedInsecureClients.value.has(client.id)) {
        showInsecureClientWarning(client);
        return;
    }

    launchClient(client.id);
};

const handleLaunchCustomClient = async (client: Client) => {
    try {
        if (isClientRunning(client.id)) {
            await stopCustomClient(client.id);
            return;
        }

        const userToken = localStorage.getItem('authToken') || 'null';
        addToast(t('home.launching', { client: client.name }), 'info', 2000);

        if (!runningClients.value.includes(client.id)) {
            runningClients.value = [...runningClients.value, client.id];
        }

        await invoke('launch_custom_client', {
            id: client.id,
            userToken,
        });

        await new Promise((resolve) => setTimeout(resolve, 500));
        await checkRunningStatus();
    } catch (err) {
        runningClients.value = runningClients.value.filter((i) => i !== client.id);
        addToast(`Failed to launch ${client.name}: ${err}`, 'error');
    }
};

const handleSearch = (query: string) => {
    searchQuery.value = query;

    if (searchDebounceTimer !== null) {
        clearTimeout(searchDebounceTimer);
    }

    searchDebounceTimer = window.setTimeout(() => {
        debouncedSearchQuery.value = query;
        searchDebounceTimer = null;
    }, 150);
};

const isAnyClientDownloading = computed(() => {
    if (installationStatus.value.size === 0) return false;
    const filenames = new Set(clients.value.map(c => c.filename));

    for (const [filename, status] of installationStatus.value.entries()) {
        if (!status.isComplete && filenames.has(filename)) {
            return true;
        }
    }
    return false;
});

const customClientsAsClients = computed(() => {
    return customClients.value.map(customClient => ({
        id: customClient.id,
        name: customClient.name,
        version: customClient.version.replace(/^V1_/, '1.').replace(/_/g, '.'),
        filename: customClient.filename,
        md5_hash: '',
        main_class: customClient.main_class,
        show: true,
        working: true,
        insecure: false,
        launches: 0,
        downloads: 0,
        size: 0,
        meta: {
            is_new: false,
            asset_index: '',
            installed: customClient.is_installed,
            is_custom: true,
            size: '0',
        },
    }));
});

const allClients = computed<Client[]>(() => {
    if (customClientsDisplayMode.value === 'global') {
        return [...customClientsAsClients.value, ...clients.value];
    }
    return clients.value;
});

const filteredClients = computed(() => {
    if (allClients.value.length === 0) {
        return [];
    }

    const query = debouncedSearchQuery.value.trim();
    const queryLower = query ? query.toLowerCase() : '';
    const filters = activeFilters.value;

    const hasActiveFilters = filters.fabric || filters.vanilla || filters.forge || filters.installed;

    let clientsList = allClients.value;

    if (queryLower || hasActiveFilters) {
        clientsList = clientsList.filter((client) => {
            if (queryLower) {
                const nameMatch = client.name.toLowerCase().includes(queryLower);
                const versionMatch = client.version.toLowerCase().includes(queryLower);
                if (!nameMatch && !versionMatch) return false;
            }

            if (hasActiveFilters) {
                if (filters.installed && !(client.meta?.installed)) {
                    return false;
                }

                if (filters.fabric || filters.vanilla || filters.forge) {
                    const clientTypeRaw = (client as any).client_type || client.meta?.client_type || '';
                    const clientType = String(clientTypeRaw).toLowerCase();
                    const tags = Array.isArray(client.meta?.tags) ? client.meta.tags : [];

                    const isFabric = clientType === 'fabric' || tags.some((t: string) => t.toLowerCase().includes('fabric'));
                    const isForge = clientType === 'forge' || tags.some((t: string) => t.toLowerCase().includes('forge'));
                    const isVanilla = !isFabric && !isForge;

                    const allowedByType =
                        (isFabric && filters.fabric) ||
                        (isForge && filters.forge) ||
                        (isVanilla && filters.vanilla);

                    if (!allowedByType) return false;
                }
            }

            return true;
        });
    }

    if (clientsList.length <= 1) {
        return clientsList;
    }

    const sortKey = clientSortKey.value;
    const sortOrder = clientSortOrder.value;
    const sortMultiplier = sortOrder === 'desc' ? -1 : 1;

    clientsList = [...clientsList];

    if (sortKey === 'newest') {
        clientsList.sort((a, b) => {
            return (a.id - b.id) * sortMultiplier;
        });
    } else if (sortKey === 'version') {
        const parseVer = (v: string): number[] => {
            if (!v) return [];
            return v.split(/[^0-9]+/).map(s => parseInt(s, 10) || 0);
        };

        clientsList.sort((a, b) => {
            const av = parseVer(a.version || '');
            const bv = parseVer(b.version || '');
            const len = Math.max(av.length, bv.length);

            for (let i = 0; i < len; i++) {
                const na = av[i] || 0;
                const nb = bv[i] || 0;
                if (na !== nb) {
                    return (nb - na) * sortMultiplier;
                }
            }

            return b.name.localeCompare(a.name) * sortMultiplier;
        });
    } else if (sortKey === 'rating') {
        clientsList.sort((a, b) => {
            const aAvg = (a.rating_avg ?? 0) as number;
            const bAvg = (b.rating_avg ?? 0) as number;

            if (aAvg !== bAvg) {
                return (bAvg - aAvg) * sortMultiplier;
            }

            const aCount = (a.rating_count ?? 0) as number;
            const bCount = (b.rating_count ?? 0) as number;
            if (aCount !== bCount) {
                return (bCount - aCount) * sortMultiplier;
            }

            return b.name.localeCompare(a.name) * sortMultiplier;
        });
    } else if (sortKey === 'popularity') {
        clientsList.sort((a, b) => {
            const av = a.launches ?? 0;
            const bv = b.launches ?? 0;
            if (av !== bv) {
                return (bv - av) * sortMultiplier;
            }
            return b.name.localeCompare(a.name) * sortMultiplier;
        });
    } else {
        clientsList.sort((a, b) =>
            b.name.localeCompare(a.name) * sortMultiplier
        );
    }

    const favSet = new Set(favoriteClients.value);
    if (favSet.size > 0) {
        const favs: Client[] = [];
        const others: Client[] = [];

        for (const c of clientsList) {
            if (favSet.has(c.id)) {
                favs.push(c);
            } else {
                others.push(c);
            }
        }

        clientsList = [...favs, ...others];
    }

    return clientsList;
});

const loadFavorites = async () => {
    try {
        const favorites = await invoke<number[]>('get_favorite_clients');
        favoriteClients.value = favorites;
    } catch (err) {
        console.error('Error loading favorites:', err);
        favoriteClients.value = [];
    }
};

const toggleFavorite = async (client: Client) => {
    try {
        const isFavorite = favoriteClients.value.includes(client.id);

        if (isFavorite) {
            await invoke('remove_favorite_client', { clientId: client.id });
            favoriteClients.value = favoriteClients.value.filter(
                (id) => id !== client.id
            );
            addToast(t('home.favorite_removed'), 'info');
        } else {
            await invoke('add_favorite_client', { clientId: client.id });
            favoriteClients.value = [...favoriteClients.value, client.id];
            addToast(t('home.favorite_added'), 'success');
        }

        syncService.uploadToCloud().catch(err => {
            console.warn('Failed to sync favorites to cloud:', err);
        });

        hideContextMenu();
    } catch (err) {
        console.error('Error toggling favorite:', err);
        addToast(`Error updating favorite: ${err}`, 'error');
    }
};

const isClientFavorite = (clientId: number): boolean => {
    return favoriteClients.value.includes(clientId);
};

const prepareForUnmount = () => {
    isLeaving.value = true;
    return new Promise((resolve) => setTimeout(resolve, 300));
};

const loadAccounts = async () => {
    try {
        const fetchedAccounts = await invoke<Account[]>('get_accounts');
        accounts.value = fetchedAccounts;

        const activeAccount = accounts.value.find(
            (account) => account.is_active
        );
        if (activeAccount) {
            selectedAccountId.value = activeAccount.id;
        } else if (accounts.value.length > 0) {
            selectedAccountId.value = accounts.value[0].id;
        }
    } catch (err) {
        console.error('Error loading accounts:', err);
        addToast(t('errors.load_accounts_error', { error: err }), 'error');
    }
};

const getClients = async () => {
    try {
        error.value = '';
        clientsLoaded.value = false;
        const response = await invoke<Client[]>('get_clients');
        if (response && response.length > 0) {
            clients.value = response;
        } else {
            clients.value = [];
            if (response !== null && response.length === 0) {
                error.value = t('errors.clients_load_failed');
                addToast(t('errors.clients_unavailable'), 'error');
            }
        }
    } catch (err) {
        console.error('Error fetching clients:', err);
        error.value = t('errors.clients_load_failed');
        addToast(t('errors.clients_load_failed'), 'error');
        clients.value = [];
    } finally {
        clientsLoaded.value = true;
    }
};

const downloadClient = async (id: number) => {
    try {
        const client = clients.value.find((c) => c.id === id);
        if (client) {
            if (props.isOnline) {
                addToast(
                    t('home.starting_download', { name: client.name }),
                    'info'
                );
                await invoke('increment_client_counter', { id, counterType: 'download' });
                await getClients();

                await invoke('download_client_only', { id });

                await new Promise((resolve) => setTimeout(resolve, 1000));
                await getClients();

                const updatedClient = clients.value.find((c) => c.id === id);
                if (updatedClient && !updatedClient.meta.installed) {
                    setTimeout(async () => {
                        await getClients();
                    }, 2000);
                }

            } else {
                addToast(t('home.no_internet'), 'error');
            }
        }
    } catch (err) {
        console.error('Error downloading client:', err);
        let errorMessage = String(err);

        if (
            errorMessage.includes('Hash verification failed') ||
            errorMessage.includes('corrupted')
        ) {
            errorMessage = t('errors.hash_verification_failed', {
                name: clients.value.find((c) => c.id === id)?.name || 'Client'
            });
        } else if (
            errorMessage.includes('Network read error') ||
            errorMessage.includes('error decoding response body')
        ) {
            errorMessage = t('errors.download_network_error', { error: err });
        } else if (errorMessage.includes('timeout')) {
            errorMessage = t('errors.download_timeout_error', { error: err });
        } else if (errorMessage.includes('Failed to create file')) {
            errorMessage = t('errors.download_disk_error', { error: err });
        }

        addToast(errorMessage, 'error');

        hashVerifyingClients.value.delete(id);
        await getClients();
    }
};

const launchClient = async (id: number) => {
    try {
        const client = clients.value.find((c) => c.id === id);
        if (client && !client.meta.installed) {
            addToast(t('home.not_installed', { name: client.name }), 'error');
            return;
        }

        addToast(
            t('home.launching', { client: client?.name || 'Client' }),
            'info',
            2000
        );

        const userToken = localStorage.getItem('authToken') || 'null';

        if (!runningClients.value.includes(id)) {
            runningClients.value = [...runningClients.value, id];
        }

        await invoke('increment_client_counter', { id, counterType: 'launch' });
        await getClients();

        try {
            await invoke('launch_client', { id, userToken });
        } catch (invokeErr) {
            runningClients.value = runningClients.value.filter((i) => i !== id);
            throw invokeErr;
        }

        await new Promise((resolve) => setTimeout(resolve, 500));
        await checkRunningStatus();
    } catch (err) {
        console.error('Error launching client:', err);
        hashVerifyingClients.value.delete(id);

        let errorMessage = String(err);
        if (errorMessage.includes('Hash verification failed')) {
            errorMessage = t('errors.hash_verification_launch_failed', {
                name: clients.value.find((c) => c.id === id)?.name || 'Client'
            });
        }

        addToast(errorMessage, 'error');
    }
};

const stopClient = async (id: number) => {
    try {
        const client = clients.value.find((c) => c.id === id);
        if (client) {
            addToast(t('home.stopping', { name: client.name }), 'info', 2000);
        }
        await invoke('stop_client', { id });

        await new Promise((resolve) => setTimeout(resolve, 1000));
        await checkRunningStatus();
    } catch (err) {
        console.error('Error stopping client:', err);
        addToast(t('errors.stop_error', { error: err }), 'error');
    }
};

const stopCustomClient = async (id: number) => {
    try {
        const client = customClients.value.find(c => c.id === id);
        if (client) {
            addToast(t('home.stopping', { client: client.name }), 'info', 2000);
        }
        await invoke('stop_custom_client', { id });

        await new Promise((resolve) => setTimeout(resolve, 1000));
        await checkRunningStatus();
    } catch (err) {
        console.error('Error stopping custom client:', err);
        addToast(`Error stopping client: ${err}`, 'error');
    }
};

const checkRunningStatus = async () => {
    if (isLeaving.value) return;
    if (skipNextRunningCheck.value.size > 0) {
        const idsToKeep = Array.from(skipNextRunningCheck.value);
        if (idsToKeep.length > 0) {
            runningClients.value = Array.from(new Set([...runningClients.value, ...idsToKeep]));
        }
        skipNextRunningCheck.value.clear();
        return;
    }
    try {
        const response = await invoke<number[]>('get_running_client_ids');
        let currentRunning: number[] = response || [];

        if (customClientsDisplayMode.value === 'global') {
            try {
                const customResponse = await invoke<number[]>('get_running_custom_client_ids');
                if (Array.isArray(customResponse)) {
                    currentRunning = Array.from(new Set([...currentRunning, ...customResponse]));
                }
            } catch (err) {
                console.error('Error checking custom client running status:', err);
            }
        }

        runningClients.value = currentRunning;
    } catch (err) {
        console.error('Error checking running status:', err);
    }
};

const isClientRunning = (id: number): boolean => {
    return runningClients.value.includes(id);
};

const openLogViewer = (client: Client) => {
    showModal(
        `log-viewer-${client.id}`,
        LogViewerModal,
        {
            title: t('logs.title', { client: client.name }),
            contentClass: 'wide',
        },
        {
            clientId: client.id,
            clientName: client.name,
        },
        {
            close: () => hideModal(`log-viewer-${client.id}`),
        }
    );
};

const showInsecureClientWarning = (client: Client) => {
    showModal(
        `insecure-warning-${client.id}`,
        InsecureClientWarningModal,
        {
            title: t('modals.insecure_client_warning.modal_title'),
        },
        { client },
        {
            proceed: (data: { client: Client; dontShowAgain: boolean }) => {
                if (data.dontShowAgain) {
                    warnedInsecureClients.value.add(data.client.id);
                    saveWarnedInsecureClients();
                }
                hideModal(`insecure-warning-${data.client.id}`);
                launchClient(data.client.id);
            },
            cancel: () => {
                hideModal(`insecure-warning-${client.id}`);
            },
            close: () => hideModal(`insecure-warning-${client.id}`),
        }
    );
};

const setupEventListeners = async () => {
    eventListeners.value.push(
        await listen('download-start', (event: any) => {
            const filename = event.payload as string;
            installationStatus.value.set(filename, {
                percentage: 0,
                action: t('installation.downloading'),
                isComplete: false,
            });
        })
    );

    eventListeners.value.push(
        await listen('download-progress', (event: any) => {
            const data = event.payload as { file: string; percentage: number };
            smoothUpdateProgress(data.file, data.percentage, t('installation.downloading'));
            updateClientInstallStatus(data.file);
        })
    );

    eventListeners.value.push(
        await listen('download-complete', (event: any) => {
            const filename = event.payload as string;
            const handle = progressAnimHandles.value.get(filename);
            if (handle != null) {
                window.cancelAnimationFrame(handle);
                progressAnimHandles.value.delete(filename);
            }
            progressTargets.value.delete(filename);
            installationStatus.value.set(filename, {
                percentage: 100,
                action: t('installation.download_complete'),
                isComplete: true,
            });

            if (!filename.endsWith('.zip')) {
                markClientAsInstalled(filename);
            }
        })
    );

    eventListeners.value.push(
        await listen('unzip-start', (event: any) => {
            const filename = event.payload as string;
            installationStatus.value.set(filename, {
                percentage: 0,
                action: t('installation.extracting'),
                isComplete: false,
            });
        })
    );

    eventListeners.value.push(
        await listen('unzip-progress', (event: any) => {
            const data = event.payload as {
                file: string;
                percentage: number;
                action: string;
            };
            smoothUpdateProgress(data.file, data.percentage, t('installation.extracting'));
            updateClientInstallStatus(data.file);
        })
    );

    eventListeners.value.push(
        await listen('unzip-complete', (event: any) => {
            const filename = event.payload as string;
            const handle = progressAnimHandles.value.get(filename);
            if (handle != null) {
                window.cancelAnimationFrame(handle);
                progressAnimHandles.value.delete(filename);
            }
            progressTargets.value.delete(filename);
            installationStatus.value.set(filename, {
                percentage: 100,
                action: t('installation.installation_complete'),
                isComplete: true,
            });

            markClientAsInstalled(filename);
        })
    );

    eventListeners.value.push(
        await listen('requirements-status', (event: any) => {
            requirementsInProgress.value = event.payload as boolean;
        })
    );

    eventListeners.value.push(
        await listen('client-needs-reinstall', async (event: any) => {
            const payload = event.payload as { id: number; name: string };
            addToast(
                t('toast.client.crashed_incomplete', { name: payload.name }),
                'warning',
                7000
            );

            try {
                await invoke('reinstall_client', { id: payload.id });
                addToast(
                    t('toast.client.reinstall_success', { name: payload.name }),
                    'success'
                );
                await getClients();
            } catch (reinstallError) {
                addToast(
                    t('toast.client.reinstall_failed', {
                        name: payload.name,
                        error: reinstallError,
                    }),
                    'error'
                );
            }
        })
    );

    const hashVerificationStartListener = await listen('client-hash-verification-start', (event: any) => {
        const { id } = event.payload;
        hashVerifyingClients.value.add(id);
    });

    const hashVerificationDoneListener = await listen('client-hash-verification-done', (event: any) => {
        const { id } = event.payload;

        hashVerifyingClients.value.delete(id);
        setTimeout(async () => {
            if (!runningClients.value.includes(id)) {
                await checkRunningStatus();
            }
        }, 800);
    });

    const hashVerificationFailedListener = await listen('client-hash-verification-failed', (event: any) => {
        const { id } = event.payload;
        hashVerifyingClients.value.add(id);
    });

    const redownloadCompleteListener = await listen('client-redownload-complete', async (event: any) => {
        const { id, name } = event.payload;
        hashVerifyingClients.value.delete(id);
        addToast(
            t('toast.client.redownload_success', { name }),
            'success'
        );

        await getClients();
    });

    eventListeners.value.push(hashVerificationStartListener);
    eventListeners.value.push(hashVerificationDoneListener);
    eventListeners.value.push(hashVerificationFailedListener);
    eventListeners.value.push(redownloadCompleteListener);

    const clientLaunchedListener = await listen('client-launched', (event: any) => {
        try {
            const { id } = event.payload as { id: number };
            skipNextRunningCheck.value.add(id);
        } catch (e) {
            console.error('Error handling client-launched event:', e);
        }
    });

    const customClientLaunchedListener = await listen('custom-client-launched', (event: any) => {
        try {
            const payload = event.payload as any;
            if (payload && typeof payload.id === 'number') {
                skipNextRunningCheck.value.add(payload.id);
            }
        } catch (e) {
            console.error('Error handling custom-client-launched event:', e);
        }
    });

    eventListeners.value.push(clientLaunchedListener);
    eventListeners.value.push(customClientLaunchedListener);
};

const updateClientInstallStatus = (filename: string) => {
    const client = clients.value.find((c) => c.filename === filename);
    if (client) {
        const status = installationStatus.value.get(filename);
        if (status && status.isComplete) {
            client.meta.installed = true;
        }
    }
};

const markClientAsInstalled = async (filename: string) => {
    const client = clients.value.find((c) => c.filename === filename);
    if (client) {
        client.meta.installed = true;

        try {
            await invoke('update_client_installed_status', {
                id: client.id,
                installed: true,
            });
        } catch (err) {
            console.error('Failed to update client installation status:', err);
            addToast(`Failed to update ${client.name} status: ${err}`, 'error');
        }
    }
};

const isClientInstalling = (client: Client): boolean => {
    const status = installationStatus.value.get(client.filename);
    return !!status && !status.isComplete;
};


const showContextMenu = (event: MouseEvent, client: Client) => {
    event.preventDefault();

    if (contextMenu.value.visible) {
        hideContextMenu();
        return;
    }

    const menuWidth = 224;
    const menuHeight = 150;
    const padding = 16;

    const viewportWidth = window.innerWidth;
    const viewportHeight = window.innerHeight;

    let x = event.clientX;
    let y = event.clientY;

    if (x + menuWidth + padding > viewportWidth) {
        x = viewportWidth - menuWidth - padding;
    }

    if (x < padding) {
        x = padding;
    }

    if (y + menuHeight + padding > viewportHeight) {
        y = viewportHeight - menuHeight - padding;
    }

    if (y < padding) {
        y = padding;
    }

    contextMenu.value = {
        visible: true,
        x: x,
        y: y,
        client: client,
        isAnimating: true,
        animationClass: 'context-menu-open-animation',
        showAccountsDropdown: false,
    };

    setTimeout(() => {
        contextMenu.value.isAnimating = false;
        contextMenu.value.animationClass = '';
    }, 150);

    document.addEventListener('click', hideContextMenu);
};

const hideContextMenu = () => {
    if (!contextMenu.value.visible) return;

    contextMenu.value.isAnimating = true;
    contextMenu.value.animationClass = 'context-menu-close-animation';
    setTimeout(() => {
        contextMenu.value.visible = false;
        contextMenu.value.client = null;
        contextMenu.value.isAnimating = false;
        contextMenu.value.animationClass = '';
        contextMenu.value.showAccountsDropdown = false;
        document.removeEventListener('click', hideContextMenu);
    }, 150);
};

const openClientFolder = async (client: Client) => {
    try {
        await invoke('open_client_folder', { id: client.id });
        addToast(t('home.opened_folder', { name: client.name }), 'success');
    } catch (err) {
        console.error('Error opening client folder:', err);
        addToast(t('errors.folder_error', { error: err }), 'error');
    }
    hideContextMenu();
};

const reinstallClient = async (client: Client) => {
    try {
        if (props.isOnline) {
            addToast(t('home.reinstalling', { name: client.name }), 'info');
            await invoke('reinstall_client', { id: client.id });
            await getClients();
        } else {
            addToast(t('home.no_internet_reinstall'), 'error');
        }
    } catch (err) {
        console.error('Error reinstalling client:', err);
        addToast(t('errors.reinstall_error', { error: err }), 'error');
    }
    hideContextMenu();
};

const copyClientLogs = async (client: Client) => {
    try {
        hideContextMenu();
        const logs = await invoke<string>('get_latest_client_logs', {
            id: client.id,
        });
        await navigator.clipboard.writeText(logs);
        addToast(t('home.logs_copied', { name: client.name }), 'success');
    } catch (err) {
        console.error('Error copying client logs:', err);
        addToast(t('errors.logs_error', { error: err }), 'error');
    }
};

const deleteClient = async (client: Client) => {
    try {
        await invoke('delete_client', { id: client.id });
        await getClients();
        addToast(t('home.deleted_success', { name: client.name }), 'success');
    } catch (err) {
        console.error('Error deleting client:', err);
        addToast(t('errors.delete_error', { error: err }), 'error');
    }
    hideContextMenu();
};

const handleClientClick = (client: Client, event: MouseEvent) => {
    if (expandedClientId.value !== null) {
        return;
    }

    if (isCtrlPressed.value) {
        event.preventDefault();
        event.stopPropagation();

        const newSelection = new Set(selectedClients.value);
        if (newSelection.has(client.id)) {
            newSelection.delete(client.id);
        } else {
            newSelection.add(client.id);
        }
        selectedClients.value = newSelection;
    } else {
        clearSelection();
    }
};

const handleExpandedStateChanged = (clientId: number, isExpanded: boolean) => {
    expandedClientId.value = isExpanded ? clientId : null;
};

const clearSelection = () => {
    if (selectedClients.value.size > 0) {
        selectedClients.value = new Set();
    }
};

const isClientSelected = (clientId: number): boolean => {
    return selectedClients.value.has(clientId);
};

const selectedClientsData = computed(() => {
    return clients.value.filter((client) =>
        selectedClients.value.has(client.id)
    );
});

const canDownloadSelected = computed(() => {
    return selectedClientsData.value.some(
        (client) => !client.meta.installed && client.working
    );
});

const canReinstallSelected = computed(() => {
    return selectedClientsData.value.some((client) => client.meta.installed);
});

const canDeleteSelected = computed(() => {
    return selectedClientsData.value.some((client) => client.meta.installed);
});

const canStopSelected = computed(() => {
    return selectedClientsData.value.some((client: Client) => isClientRunning(client.id));
});

const downloadMultipleClients = async () => {
    const selectedData = selectedClientsData.value;
    const validClients = selectedData.filter(
        (client) => !client.meta.installed && client.working
    );

    if (validClients.length === 0) {
        addToast(t('home.no_clients_to_download'), 'warning');
        hideContextMenu();
        return;
    }

    if (!props.isOnline) {
        addToast(t('home.no_internet'), 'error');
        hideContextMenu();
        return;
    }

    addToast(
        t('home.downloading_multiple', { count: validClients.length }),
        'info'
    );

    const downloadPromises = validClients.map(async (client) => {
        try {
            await invoke('increment_client_counter', { id: client.id, counterType: 'download' });
            await invoke('download_client_only', { id: client.id });
        } catch (err) {
            console.error(`Error downloading client ${client.name}:`, err);
            addToast(
                t('errors.download_error', { error: `${client.name}: ${err}` }),
                'error'
            );
        }
    });

    await Promise.all(downloadPromises);

    hideContextMenu();
    clearSelection();
};

const stopMultipleClients = async () => {
    const selectedData = selectedClientsData.value;
    const runningClients = selectedData.filter(
        (client) => isClientRunning(client.id)
    );

    if (runningClients.length === 0) {
        hideContextMenu();
        return;
    }

    addToast(
        t('home.stopping_multiple', { count: runningClients.length }),
        'info'
    );

    for (const client of runningClients) {
        try {
            await stopClient(client.id);
            await new Promise((resolve) => setTimeout(resolve, 100));
        } catch (err) {
            console.error(`Error stopping client ${client.name}:`, err);
            addToast(
                t('errors.stop_error', { error: `${client.name}: ${err}` }),
                'error'
            );
        }
    }

    await checkRunningStatus();
    addToast(
        t('home.multiple_clients_stopped', { count: runningClients.length }),
        'success'
    );
    hideContextMenu();
    clearSelection();
};

const deleteMultipleClients = async () => {
    const selectedData = selectedClientsData.value;
    const installedClients = selectedData.filter(
        (client) => client.meta.installed
    );

    if (installedClients.length === 0) {
        addToast(t('home.no_clients_to_delete'), 'warning');
        hideContextMenu();
        return;
    }

    addToast(
        t('home.deleting_multiple', { count: installedClients.length }),
        'info'
    );

    for (const client of installedClients) {
        try {
            await invoke('delete_client', { id: client.id });
            await new Promise((resolve) => setTimeout(resolve, 100));
        } catch (err) {
            console.error(`Error deleting client ${client.name}:`, err);
            addToast(
                t('errors.delete_error', { error: `${client.name}: ${err}` }),
                'error'
            );
        }
    }

    await getClients();
    addToast(
        t('home.multiple_clients_deleted', { count: installedClients.length }),
        'success'
    );
    hideContextMenu();
    clearSelection();
};

const reinstallMultipleClients = async () => {
    const selectedData = selectedClientsData.value;
    const installedClients = selectedData.filter(
        (client) => client.meta.installed
    );

    if (installedClients.length === 0) {
        addToast(t('home.no_clients_to_reinstall'), 'warning');
        hideContextMenu();
        return;
    }

    if (!props.isOnline) {
        addToast(t('home.no_internet_reinstall'), 'error');
        hideContextMenu();
        return;
    }

    addToast(
        t('home.reinstalling_multiple', { count: installedClients.length }),
        'info'
    );

    for (const client of installedClients) {
        try {
            await invoke('reinstall_client', { id: client.id });
            await new Promise((resolve) => setTimeout(resolve, 300));
        } catch (err) {
            console.error(`Error reinstalling client ${client.name}:`, err);
            addToast(
                t('errors.reinstall_error', {
                    error: `${client.name}: ${err}`,
                }),
                'error'
            );
        }
    }

    await getClients();
    addToast(
        t('home.multiple_clients_reinstalled', {
            count: installedClients.length,
        }),
        'success'
    );
    hideContextMenu();
    clearSelection();
};

const selectAllClients = () => {
    selectedClients.value = new Set(filteredClients.value.map((client) => client.id));
};

const handleKeyDown = (event: KeyboardEvent) => {
    if ((event.ctrlKey || event.metaKey) && event.key === 'f') {
        event.preventDefault();
        searchBarRef.value?.focus();
        return;
    }

    if (event.key === 'Control' && expandedClientId.value === null) {
        if (isCtrlPressed.value) return;

        if (ctrlPressTimer !== null) {
            clearTimeout(ctrlPressTimer);
        }

        ctrlPressTimer = window.setTimeout(() => {
            isCtrlPressed.value = true;
            ctrlPressTimer = null;
        }, 50);
        return;
    }

    if (event.key === 'Escape') {
        clearSelection();
    }

    if (
        event.ctrlKey &&
        (event.key === 'a' || event.key === 'ф') &&
        !(event.target as HTMLElement).matches('input, textarea, [contenteditable]') &&
        expandedClientId.value === null
    ) {
        if (!event.repeat) {
            event.preventDefault();
            selectAllClients();
        }
    }
};

const handleKeyUp = (event: KeyboardEvent) => {
    if (event.key === 'Control') {
        if (ctrlPressTimer !== null) {
            clearTimeout(ctrlPressTimer);
            ctrlPressTimer = null;
        }
        isCtrlPressed.value = false;
    }
};

const handleDocumentClick = (event: MouseEvent) => {
    const target = event.target as HTMLElement;
    if (!target.closest('.client-card')) {
        clearSelection();
    }
};

const loadWarnedInsecureClients = () => {
    try {
        const stored = localStorage.getItem('warnedInsecureClients');
        if (stored) {
            const clientIds = JSON.parse(stored);
            warnedInsecureClients.value = new Set(clientIds);
        }
    } catch (error) {
        console.error('Failed to load warned insecure clients:', error);
        warnedInsecureClients.value = new Set();
    }
};

const saveWarnedInsecureClients = () => {
    try {
        const clientIds = Array.from(warnedInsecureClients.value);
        localStorage.setItem(
            'warnedInsecureClients',
            JSON.stringify(clientIds)
        );
    } catch (error) {
        console.error('Failed to save warned insecure clients:', error);
    }
};

const loadCustomClients = async () => {
    try {
        const clients = await invoke<CustomClient[]>('get_custom_clients');
        customClients.value = clients;
    } catch (err) {
        console.error('Error loading custom clients:', err);
    }
};

const loadCustomClientsDisplayMode = async () => {
    try {
        const flags = await invoke('get_flags');
        const typedFlags = flags as any;

        customClientsDisplayMode.value = typedFlags.custom_clients_display.value;
        return typedFlags;
    } catch (err) {
        console.error('Error loading flags:', err);
        addToast(`Failed to load flags: ${err}`, 'error');
        return {};
    }
};

onMounted(async () => {
    debouncedSearchQuery.value = searchQuery.value;

    await getClients();
    await loadFavorites();
    await loadCustomClients();
    await loadCustomClientsDisplayMode();
    await checkRunningStatus();
    await loadAccounts();
    loadWarnedInsecureClients();

    if (statusInterval.value !== null) {
        clearInterval(statusInterval.value);
    }

    await setupEventListeners();

    setTimeout(() => {
        if (statusInterval.value === null && !isLeaving.value) {
            statusInterval.value = setInterval(
                checkRunningStatus,
                5000
            ) as unknown as number;
        }
    }, 500);

    document.addEventListener('keydown', handleKeyDown);
    document.addEventListener('keyup', handleKeyUp);
    document.addEventListener('click', handleDocumentClick);
    window.addEventListener('blur', blurHandler);
    document.addEventListener('visibilitychange', visibilityHandler);

    if (!hasAnimatedBefore.value) {
        await nextTick();
        viewVisible.value = true;
        if (filteredClients.value.length > 0) {
            const totalDelay = Math.min(Math.max(0, filteredClients.value.length) * 45 + 500, 2000);
            setTimeout(() => {
                playClientSlideAnim.value = false;
                try {
                    sessionStorage.setItem(HOME_ANIM_KEY, '1');
                    hasAnimatedBefore.value = true;
                } catch (e) {
                    console.error('Failed to set sessionStorage item:', e);
                }
            }, totalDelay);
        } else {
            try {
                sessionStorage.setItem(HOME_ANIM_KEY, '1');
                hasAnimatedBefore.value = true;
            } catch (e) {
                console.error('Failed to set sessionStorage item:', e);
            }
        }
    }

    try {
        if (!hasStaggerPlayed.value) {
            const maxIndex = Math.max(0, filteredClients.value.length - 1);
            const perItemDelay = 70;
            const animDuration = 400;
            const total = maxIndex * perItemDelay + animDuration + 80;
            setTimeout(() => {
                try {
                    sessionStorage.setItem(STAGGER_KEY, '1');
                    hasStaggerPlayed.value = true;
                } catch (e) {
                    console.error('Failed to set stagger session key:', e);
                }
            }, total);
        }
    } catch (e) {
        console.error('Error scheduling stagger flag:', e);
    }


});

onBeforeUnmount(() => {

    if (statusInterval.value !== null) {
        clearInterval(statusInterval.value);
        statusInterval.value = null;
    }

    if (searchDebounceTimer !== null) {
        clearTimeout(searchDebounceTimer);
        searchDebounceTimer = null;
    }
    if (ctrlPressTimer !== null) {
        clearTimeout(ctrlPressTimer);
        ctrlPressTimer = null;
    }

    eventListeners.value.forEach((unlisten) => unlisten());
    eventListeners.value = [];
    hideContextMenu();
    prepareForUnmount();

    document.removeEventListener('keydown', handleKeyDown);
    document.removeEventListener('keyup', handleKeyUp);
    document.removeEventListener('click', handleDocumentClick);
    window.removeEventListener('blur', blurHandler);
    document.removeEventListener('visibilitychange', visibilityHandler);

    progressAnimHandles.value.forEach((handle) => window.cancelAnimationFrame(handle));
    progressAnimHandles.value.clear();
});
</script>

<template>
    <div :class="['flex items-center gap-2 mb-6 top-menu', viewVisible ? 'home-entered' : 'home-hidden']">
        <SearchBar ref="searchBarRef" @search="handleSearch" class="flex-1 mr-2 home-search"
            :initial-value="searchQuery" :placeholder="t('home.search_placeholder')" />
        <FiltersMenu v-model:activeFilters="activeFilters" v-model:clientSortKey="clientSortKey"
            v-model:clientSortOrder="clientSortOrder" />
        <div v-if="halloweenActive" class="tooltip tooltip-bottom" :data-tip="t('events.halloween.tooltip')">
            <div class="px-3 py-2 bg-warning/10 border border-warning/30 rounded-lg text-warning">
                <span class="text-xl">🎃</span>
            </div>
        </div>
        <div class="tooltip tooltip-bottom" :data-tip="t('navigation.custom_clients')">
            <button @click="$emit('change-view', 'custom_clients')"
                class="btn btn-ghost border-base-300 btn-primary gap-2 home-action-btn"
                :style="{ border: 'var(--border) solid #0000', transitionDelay: '0.5s' }">
                <FileText class="w-4 h-4" />
            </button>
        </div>
        <div class="tooltip tooltip-bottom" :data-tip="t('navigation.news')">
            <button @click="$emit('change-view', 'news')"
                class="btn btn-ghost border-base-300 btn-primary gap-2 relative home-action-btn"
                :style="{ border: 'var(--border) solid #0000', transitionDelay: '1s' }">
                <Newspaper class="w-4 h-4" />
                <span v-if="props.unreadNewsCount && props.unreadNewsCount > 0"
                    class="absolute -top-2 -right-2 bg-primary text-primary-content text-xs font-bold rounded-full min-w-5 h-5 flex items-center justify-center border-2 border-base-100 px-1">
                    {{ props.unreadNewsCount > 9 ? '9+' : props.unreadNewsCount }}
                </span>
            </button>
        </div>
    </div>

    <InlineIRCChat class="mb-6" @show-user-profile="$emit('show-user-profile', $event)" />

    <div v-if="filteredClients.length === 0 && !error && clientsLoaded"
        class="text-center py-10 text-base-content/70 animate-fadeIn flex flex-col items-center">
        <div class="text-lg font-semibold mb-2">{{ t('home.no_clients') }}</div>
        <div class="text-sm mb-4">{{ t('home.adjust_search') }}</div>
        <button
            v-if="searchQuery || activeFilters.fabric || activeFilters.vanilla || activeFilters.forge || activeFilters.installed"
            @click="clearAllFilters" class="btn btn-sm btn-primary">
            {{ t('home.clear_filters') }}
        </button>
    </div>

    <div v-if="error" class="my-4 text-red-500 animate-fadeIn flex flex-col items-center justify-center h-full">
        <span>{{ error }}</span>
    </div>

    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4 relative overflow-hidden"
        :class="{ 'multi-select-mode': isCtrlPressed && expandedClientId === null }">
        <div v-for="(client, idx) in filteredClients" :key="client.id"
            :class="['client-card-item', { 'slide-in-animate': playClientSlideAnim }]"
            :style="playClientSlideAnim ? { animationDelay: `${Math.min(idx * 100, 600)}ms` } : {}" v-memo="[
                playClientSlideAnim,
                client.id,
                isClientRunning(client.id),
                isClientInstalling(client),
                installationStatus.get(client.filename),
                requirementsInProgress,
                isAnyClientDownloading,
                isClientFavorite(client.id),
                isClientSelected(client.id),
                isCtrlPressed,
                expandedClientId,
                hashVerifyingClients.has(client.id),
                isAnyCardExpanded
            ]">
            <ClientCard :client="client" :isClientRunning="isClientRunning(client.id)"
                :isClientInstalling="isClientInstalling(client)"
                :installationStatus="installationStatus.get(client.filename)"
                :isRequirementsInProgress="requirementsInProgress" :isAnyClientDownloading="isAnyClientDownloading"
                :isFavorite="isClientFavorite(client.id)" :isSelected="isClientSelected(client.id)"
                :isMultiSelectMode="isCtrlPressed && expandedClientId === null"
                :isHashVerifying="hashVerifyingClients.has(client.id)" :isAnyCardExpanded="isAnyCardExpanded"
                @launch="handleLaunchClick" @download="downloadClient" @open-log-viewer="openLogViewer"
                @show-context-menu="showContextMenu" @client-click="handleClientClick"
                @expanded-state-changed="handleExpandedStateChanged"
                @show-user-profile="$emit('show-user-profile', $event)" />
        </div>
    </div>

    <div v-if="contextMenu.visible" :style="{ top: `${contextMenu.y}px`, left: `${contextMenu.x}px` }"
        class="fixed z-40 menu p-0 bg-base-200 w-56 rounded-box shadow-xl border border-base-300 dropdown-content"
        :class="contextMenu.animationClass">
        <h3 v-if="selectedClients.size <= 1"
            class="font-medium text-sm px-4 py-2 border-b border-base-300 text-base-content/80 bg-base-300/30">
            {{ contextMenu.client?.name }}
        </h3>

        <h3 v-else class="font-medium text-sm px-4 py-2 border-b border-base-300 text-base-content/80 bg-base-300/30">
            {{
                t('home.multiple_clients_selected', {
                    count: selectedClients.size,
                })
            }}
        </h3>

        <ul v-if="selectedClients.size <= 1">
            <li>
                <a @click="toggleFavorite(contextMenu.client!)"
                    class="flex items-center gap-2 text-sm active:bg-primary/30">
                    <Star class="w-4 h-4" :class="{
                        'fill-yellow-400 text-yellow-400': isClientFavorite(
                            contextMenu.client?.id || 0
                        ),
                    }" />
                    {{
                        isClientFavorite(contextMenu.client?.id || 0)
                            ? t('theme.actions.remove_favorite')
                            : t('theme.actions.add_favorite')
                    }}
                </a>
            </li>
            <li v-if="contextMenu.client?.meta.installed">
                <a @click="reinstallClient(contextMenu.client!)"
                    class="flex items-center gap-2 text-sm active:bg-primary/30">
                    <RefreshCcw class="w-4 h-4" />
                    {{ t('common.reinstall') }}
                </a>
            </li>
            <li v-if="contextMenu.client?.meta.installed">
                <a @click="deleteClient(contextMenu.client!)"
                    class="flex items-center gap-2 text-sm active:bg-primary/30">
                    <Trash2 class="w-4 h-4" />
                    {{ t('common.delete') }}
                </a>
            </li>
            <li v-if="contextMenu.client?.meta.installed">
                <a @click="openClientFolder(contextMenu.client!)"
                    class="flex items-center gap-2 text-sm active:bg-primary/30">
                    <Folder class="w-4 h-4" />
                    {{ t('theme.actions.open_folder') }}
                </a>
            </li>
            <li v-if="contextMenu.client?.meta.installed">
                <a @click="copyClientLogs(contextMenu.client!)"
                    class="flex items-center gap-2 text-sm active:bg-primary/30">
                    <Copy class="w-4 h-4" />
                    {{ t('logs.copy_logs') }}
                </a>
            </li>
        </ul>
    </div>

    <transition name="slide-up-bottom">
        <div v-if="selectedClients.size > 0"
            class="fixed bottom-4 left-1/2 transform -translate-x-1/2 w-auto max-w-[calc(100%-2rem)] bg-neutral text-neutral-content px-4 py-3 rounded-lg shadow-xl z-30 flex items-center gap-3 sm:gap-4">
            <span class="font-medium text-xs sm:text-sm whitespace-nowrap">{{
                t('home.selected_clients', {
                    count: selectedClients.size,
                })
            }}</span>

            <div class="flex items-center gap-1 sm:gap-2">
                <transition name="button-fade" mode="out-in">
                    <button v-if="canStopSelected" @click="stopMultipleClients" :title="t('home.stop_selected')"
                        class="btn btn-sm btn-ghost hover:bg-neutral-focus p-2 aspect-square">
                        <StopCircle class="w-4 h-4 sm:w-5 sm:h-5" />
                    </button>
                </transition>

                <transition name="button-fade" mode="out-in">
                    <button v-if="canDownloadSelected" @click="downloadMultipleClients"
                        :title="t('home.download_selected')"
                        class="btn btn-sm btn-ghost hover:bg-neutral-focus p-2 aspect-square">
                        <Download class="w-4 h-4 sm:w-5 sm:h-5" />
                    </button>
                </transition>

                <transition name="button-fade" mode="out-in">
                    <button v-if="canReinstallSelected" @click="reinstallMultipleClients"
                        :title="t('home.reinstall_selected')"
                        class="btn btn-sm btn-ghost hover:bg-neutral-focus p-2 aspect-square">
                        <RefreshCcw class="w-4 h-4 sm:w-5 sm:h-5" />
                    </button>
                </transition>

                <transition name="button-fade" mode="out-in">
                    <button v-if="canDeleteSelected" @click="deleteMultipleClients" :title="t('home.delete_selected')"
                        class="btn btn-sm btn-ghost hover:bg-neutral-focus p-2 aspect-square">
                        <Trash2 class="w-4 h-4 sm:w-5 sm:h-5" />
                    </button>
                </transition>
            </div>

            <div v-if="
                canStopSelected ||
                canDownloadSelected ||
                canReinstallSelected ||
                canDeleteSelected
            " class="border-l border-neutral-content/30 h-5 sm:h-6 mx-1"></div>

            <button @click="clearSelection" class="btn btn-sm btn-ghost hover:bg-neutral-focus p-2 aspect-square">
                <X class="w-4 h-4 sm:w-5 sm:h-5" />
            </button>
        </div>
    </transition>
</template>

<style>
.client-card-item {
    transition:
        transform 0.2s ease-out,
        box-shadow 0.2s ease-out;
    opacity: 1;
}

@keyframes fadeInUp {
    from {
        opacity: 0;
        transform: translateY(15px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.animate-fadeIn {
    animation: fadeInGeneral 0.5s ease-in-out;
}

@keyframes fadeInGeneral {
    from {
        opacity: 0;
    }

    to {
        opacity: 1;
    }
}

@keyframes scaleIn {
    from {
        opacity: 0;
        transform: scale(0.95);
    }

    to {
        opacity: 1;
        transform: scale(1);
    }
}

@keyframes scaleOut {
    from {
        opacity: 1;
        transform: scale(1);
    }

    to {
        opacity: 0;
        transform: scale(0.95);
    }
}

.context-menu {
    position: fixed;
    left: var(--x);
    top: var(--y);
}

.context-menu-open-animation {
    animation: scaleIn 0.15s ease-out forwards;
}

.context-menu-close-animation {
    animation: scaleOut 0.15s ease-in forwards;
}

.menu.dropdown-content li a {
    transition: all 0.2s ease;
}

.menu.dropdown-content li a:hover {
    background-color: hsl(var(--p) / 0.1);
    padding-left: 1rem;
}

.search-results-enter-active,
.search-results-leave-active {
    transition: all 0.3s ease;
}

.search-results-enter-from,
.search-results-leave-to {
    opacity: 0;
    transform: translateY(20px);
}

.search-results-move {
    transition: transform 0.5s ease;
}

.empty-search-state {
    animation: fadeInUp 0.3s ease-out forwards;
}

.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.3s ease;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}

.menu.dropdown-content .relative .absolute {
    box-shadow:
        0 10px 25px -3px rgba(0, 0, 0, 0.1),
        0 4px 6px -2px rgba(0, 0, 0, 0.05);
}

.menu.dropdown-content .relative .absolute button {
    border: none;
    background: transparent;
    cursor: pointer;
}

.menu.dropdown-content .relative .absolute button:first-child {
    border-top-left-radius: 0.5rem;
    border-top-right-radius: 0.5rem;
}

.menu.dropdown-content .relative .absolute button:last-child {
    border-bottom-left-radius: 0.5rem;
    border-bottom-right-radius: 0.5rem;
}

.slide-in-enter-active,
.slide-in-leave-active {
    transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.slide-in-enter-from {
    opacity: 0;
    transform: translateX(90px);
}

.slide-in-leave-to {
    opacity: 0;
    transform: translateX(90px);
}

.slide-up-bottom-enter-active,
.slide-up-bottom-leave-active {
    transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.slide-up-bottom-enter-from,
.slide-up-bottom-leave-to {
    opacity: 0;
    transform: translateY(100px);
}

.button-fade-enter-active,
.button-fade-leave-active {
    transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.button-fade-enter-from {
    opacity: 0;
    transform: scale(0.8) translateY(10px);
}

.button-fade-leave-to {
    opacity: 0;
    transform: scale(0.8) translateY(-10px);
}

.button-fade-enter-to,
.button-fade-leave-from {
    opacity: 1;
    transform: scale(1) translateY(0);
}

.top-menu {
    position: relative;
    z-index: 10;
}

.home-hidden {
    opacity: 0;
    transform: translateY(-100vh);
}

.home-entered {
    opacity: 1;
    transform: translateY(0);
    transition: opacity 0.5s cubic-bezier(0.2, 0.9, 0.2, 1), transform 0.6s cubic-bezier(0.2, 0.9, 0.2, 1);
}

.home-search {
    opacity: 0;
    transform: translateY(-30px) scale(0.995);
    transition: transform 0.6s cubic-bezier(0.2, 0.9, 0.2, 1), opacity 0.6s ease;
}

.home-entered .home-search {
    opacity: 1;
    transform: translateY(0) scale(1);
    transition-delay: 0.08s;
}

.home-action-btn {
    opacity: 0;
    transform: translateY(-30px) scale(0.995);
    transition: transform 0.56s cubic-bezier(0.2, 0.9, 0.2, 1), opacity 0.56s ease;
}

.home-entered .home-action-btn {
    opacity: 1;
    transform: translateY(0) scale(1);
}

.home-entered .home-action-btn:nth-child(1) {
    transition-delay: 0.12s;
}

.home-entered .home-action-btn:nth-child(2) {
    transition-delay: 0.16s;
}


.client-card-item {
    opacity: 1;
}

.client-card-item.stagger-animate {
    opacity: 0;
    transform: translateY(15px);
    animation: fadeInUp 0.4s ease-out forwards;
}

.client-card-item.slide-in-animate {
    opacity: 0;
    transform: translateX(80px);
    animation: slideInFromRight 0.42s cubic-bezier(0.2, 0.9, 0.2, 1) forwards;
    will-change: transform, opacity;
}

@keyframes slideInFromRight {
    from {
        opacity: 0;
        transform: translateX(80px);
    }

    to {
        opacity: 1;
        transform: translateX(0);
    }
}

@media (prefers-reduced-motion: reduce) {
    .client-card-item.slide-in-animate {
        animation: none !important;
        transform: none !important;
        opacity: 1 !important;
    }
}

.client-list-move,
.client-list-enter-active,
.client-list-leave-active {
    transition: transform 0.4s cubic-bezier(0.55, 0, 0.1, 1), opacity 0.2s ease;
}

.client-list-leave-active {
    position: absolute;
    z-index: 0;
}

.client-list-enter-from,
.client-list-leave-to {
    opacity: 0;
}
</style>

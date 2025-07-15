<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { onBeforeUnmount, onMounted, ref, computed } from 'vue';
import {
    RefreshCcw,
    Folder,
    Copy,
    Trash2,
    Star,
    Play,
    ChevronDown,
    Download,
    Newspaper,
    StopCircle,
} from 'lucide-vue-next';
import SearchBar from '../components/common/SearchBar.vue';
import ClientCard from '../components/features/clients/ClientCard.vue';
import { useToast } from '../services/toastService';
import { useModal } from '../services/modalService';
import { useI18n } from 'vue-i18n';
import type { Client, InstallProgress } from '../types/ui';
import LogViewerModal from '../components/modals/LogViewerModal.vue';
import InsecureClientWarningModal from '../components/modals/InsecureClientWarningModal.vue';

interface Account {
    id: string;
    username: string;
    tags: string[];
    created_at: string;
    last_used?: string;
    is_active: boolean;
}

const { t } = useI18n();

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

const clients = ref<Client[]>([]);
const favoriteClients = ref<number[]>([]);
const error = ref('');
const runningClients = ref<number[]>([]);
const isLeaving = ref(false);
const { addToast } = useToast();
const { showModal, hideModal } = useModal();
const statusInterval = ref<number | null>(null);

const accounts = ref<Account[]>([]);
const selectedAccountId = ref<string>('');

const searchQuery = ref('');
const installationStatus = ref<Map<string, InstallProgress>>(new Map());
const eventListeners = ref<(() => void)[]>([]);
const requirementsInProgress = ref<boolean>(false);
const warnedInsecureClients = ref<Set<number>>(new Set());
const hashVerifyingClients = ref<Set<number>>(new Set());

const contextMenu = ref({
    visible: false,
    x: 0,
    y: 0,
    client: null as Client | null,
    isAnimating: false,
    animationClass: '',
    showAccountsDropdown: false,
});

const selectedClients = ref<Set<number>>(new Set());
const isCtrlPressed = ref(false);
const expandedClientId = ref<number | null>(null);

const handleLaunchClick = (client: Client) => {
    if (isClientRunning(client.id)) {
        stopClient(client.id);
        return;
    }

    if (hashVerifyingClients.value.has(client.id)) {
        return;
    }

    if (client.insecure && !warnedInsecureClients.value.has(client.id)) {
        showInsecureClientWarning(client);
        return;
    }

    launchClient(client.id);
};

const handleSearch = (query: string) => {
    searchQuery.value = query;
};

const isRequirementsInProgress = computed(() => {
    return requirementsInProgress.value;
});

const isAnyClientDownloading = computed(() => {
    return Array.from(installationStatus.value.entries()).some(
        ([filename, status]) => {
            const isClientFile = clients.value.some(
                (client) => client.filename === filename
            );
            return isClientFile && !status.isComplete;
        }
    );
});

const filteredClients = computed(() => {
    let clientsList = clients.value;

    if (searchQuery.value.trim()) {
        const query = searchQuery.value.toLowerCase().trim();
        clientsList = clientsList.filter(
            (client) =>
                client.name.toLowerCase().includes(query) ||
                client.version.toLowerCase().includes(query) ||
                (client.meta &&
                    client.meta.tags &&
                    Array.isArray(client.meta.tags) &&
                    client.meta.tags.some((tag: string) =>
                        tag.toLowerCase().includes(query)
                    ))
        );
    }

    return clientsList.sort((a, b) => {
        const aIsFavorite = favoriteClients.value.includes(a.id);
        const bIsFavorite = favoriteClients.value.includes(b.id);

        if (aIsFavorite && !bIsFavorite) return -1;
        if (!aIsFavorite && bIsFavorite) return 1;
        return 0;
    });
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
            favoriteClients.value.push(client.id);
            addToast(t('home.favorite_added'), 'success');
        }

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

const launchClientWithAccount = async (client: Client, accountId: string) => {
    try {
        const previousActiveAccount = accounts.value.find(
            (account) => account.is_active
        );

        await invoke('set_active_account', { id: accountId });

        await launchClient(client.id);

        if (previousActiveAccount && previousActiveAccount.id !== accountId) {
            await invoke('set_active_account', {
                id: previousActiveAccount.id,
            });
        }
    } catch (err) {
        console.error('Error launching client with account:', err);
        addToast(
            t('errors.launch_with_account_error', { error: err }),
            'error'
        );
    }

    hideContextMenu();
};

const getClients = async () => {
    try {
        error.value = '';
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

                await new Promise((resolve) => setTimeout(resolve, 500));

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

        await invoke('increment_client_counter', { id, counterType: 'launch' });
        await getClients();

        await invoke('launch_client', { id, userToken });

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

const checkRunningStatus = async () => {
    if (isLeaving.value) return;
    try {
        const response = await invoke<number[]>('get_running_client_ids');
        runningClients.value = response;
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
            installationStatus.value.set(data.file, {
                percentage: data.percentage,
                action: t('installation.downloading'),
                isComplete: false,
            });
            updateClientInstallStatus(data.file);
        })
    );

    eventListeners.value.push(
        await listen('download-complete', (event: any) => {
            const filename = event.payload as string;
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
            installationStatus.value.set(data.file, {
                percentage: data.percentage,
                action: t('installation.extracting'),
                isComplete: false,
            });
            updateClientInstallStatus(data.file);
        })
    );

    eventListeners.value.push(
        await listen('unzip-complete', (event: any) => {
            const filename = event.payload as string;
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

        setTimeout(() => {
            hashVerifyingClients.value.delete(id);
        }, 1000);
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

const isClientHashVerifying = (clientId: number): boolean => {
    return hashVerifyingClients.value.has(clientId);
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

const toggleAccountsDropdown = (event?: Event) => {
    if (event) {
        event.stopPropagation();
    }
    contextMenu.value.showAccountsDropdown =
        !contextMenu.value.showAccountsDropdown;
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

        if (selectedClients.value.has(client.id)) {
            selectedClients.value.delete(client.id);
        } else {
            selectedClients.value.add(client.id);
        }
        selectedClients.value = new Set(selectedClients.value);
    } else {
        selectedClients.value.clear();
    }
};

const handleExpandedStateChanged = (clientId: number, isExpanded: boolean) => {
    expandedClientId.value = isExpanded ? clientId : null;
};

const clearSelection = () => {
    selectedClients.value.clear();
};

const isClientSelected = (clientId: number): boolean => {
    return selectedClients.value.has(clientId);
};

const getSelectedClientsData = (): Client[] => {
    return clients.value.filter((client) =>
        selectedClients.value.has(client.id)
    );
};

const canDownloadSelected = computed(() => {
    const selectedClientsData = getSelectedClientsData();
    return selectedClientsData.some(
        (client) => !client.meta.installed && client.working
    );
});

const canReinstallSelected = computed(() => {
    const selectedClientsData = getSelectedClientsData();
    return selectedClientsData.some((client) => client.meta.installed);
});

const canDeleteSelected = computed(() => {
    const selectedClientsData = getSelectedClientsData();
    return selectedClientsData.some((client) => client.meta.installed);
});

const canStopSelected = computed(() => {
    const selectedClientsData = getSelectedClientsData();
    return selectedClientsData.some((client) => isClientRunning(client.id));
});

const downloadMultipleClients = async () => {
    const selectedClientsData = getSelectedClientsData();
    const validClients = selectedClientsData.filter(
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
    const selectedClientsData = getSelectedClientsData();
    const runningClients = selectedClientsData.filter(
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
    const selectedClientsData = getSelectedClientsData();
    const installedClients = selectedClientsData.filter(
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
    const selectedClientsData = getSelectedClientsData();
    const installedClients = selectedClientsData.filter(
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
    const allClientIds = filteredClients.value.map((client) => client.id);
    selectedClients.value = new Set(allClientIds);
};

const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Control' && expandedClientId.value === null) {
        isCtrlPressed.value = true;
    }
    if (event.key === 'Escape') {
        clearSelection();
    }

    if (
        event.ctrlKey &&
        (event.key === 'a' || event.key === 'ф') &&
        !(event.target as HTMLElement).matches(
            'input, textarea, [contenteditable]'
        ) &&
        expandedClientId.value === null
    ) {
        event.preventDefault();
        selectAllClients();
    }
};

const handleKeyUp = (event: KeyboardEvent) => {
    if (event.key === 'Control') {
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

onMounted(async () => {
    await getClients();
    await loadFavorites();
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
});

onBeforeUnmount(() => {
    if (statusInterval.value !== null) {
        clearInterval(statusInterval.value);
        statusInterval.value = null;
    }

    eventListeners.value.forEach((unlisten) => unlisten());
    eventListeners.value = [];
    hideContextMenu();
    prepareForUnmount();

    document.removeEventListener('keydown', handleKeyDown);
    document.removeEventListener('keyup', handleKeyUp);
    document.removeEventListener('click', handleDocumentClick);
});
</script>

<template>
    <!-- <h1 class="text-error text-center mb-5 font-semibold">
        {{ t('home.alpha') }}
    </h1> -->

    <div class="flex items-center gap-4 mb-6">
        <SearchBar @search="handleSearch" class="flex-1" :initial-value="searchQuery"
            :placeholder="t('home.search_placeholder')" />
        <div class="tooltip tooltip-bottom" :data-tip="t('navigation.news')">
            <button @click="$emit('change-view', 'news')"
                class="btn btn-ghost border-base-300 btn-primary gap-2 flex-shrink-0 relative"
                style="border: var(--border) solid #0000">
                <Newspaper class="w-4 h-4" />
                <span v-if="props.unreadNewsCount && props.unreadNewsCount > 0"
                    class="absolute -top-2 -right-2 bg-primary text-primary-content text-xs font-bold rounded-full min-w-5 h-5 flex items-center justify-center border-2 border-base-100 px-1">
                    {{ props.unreadNewsCount > 9 ? '9+' : props.unreadNewsCount }}
                </span>
            </button>
        </div>
    </div>

    <div v-if="filteredClients.length === 0 && !error" class="text-center py-10 text-base-content/70 animate-fadeIn">
        <div class="text-lg font-semibold mb-2">{{ t('home.no_clients') }}</div>
        <div class="text-sm">{{ t('home.adjust_search') }}</div>
    </div>

    <div v-if="error" class="my-4 text-red-500 animate-fadeIn flex flex-col items-center justify-center h-full">
        <span>{{ error }}</span>
    </div>

    <div class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4">
        <ClientCard v-for="(client, index) in filteredClients" :key="client.id" :client="client"
            :isClientRunning="isClientRunning" :isClientInstalling="isClientInstalling"
            :installationStatus="installationStatus" :isRequirementsInProgress="isRequirementsInProgress"
            :isAnyClientDownloading="isAnyClientDownloading" :isFavorite="isClientFavorite(client.id)"
            :isSelected="isClientSelected(client.id)" :isMultiSelectMode="isCtrlPressed && expandedClientId === null"
            :isHashVerifying="isClientHashVerifying(client.id)" @launch="handleLaunchClick" @download="downloadClient"
            @open-log-viewer="openLogViewer" @show-context-menu="showContextMenu" @client-click="handleClientClick"
            @expanded-state-changed="handleExpandedStateChanged" class="client-card-item"
            :style="{ 'animation-delay': index * 0.07 + 's' }" />
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
                    class="flex items-center gap-2 text-sm active:bg-primary/30 focus:bg-primary/20">
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
            <li v-if="accounts.length > 1 && contextMenu.client?.meta.installed" class="relative">
                <a @click="toggleAccountsDropdown"
                    class="flex items-center gap-2 text-sm active:bg-primary/30 focus:bg-primary/20 justify-between">
                    <div class="flex items-center gap-2">
                        <Play class="w-4 h-4" />
                        {{ t('home.start_with') }}
                    </div>
                    <ChevronDown class="w-4 h-4 transition-transform duration-200" :class="{
                        'rotate-180': contextMenu.showAccountsDropdown,
                    }" />
                </a>
                <div v-if="contextMenu.showAccountsDropdown"
                    class="absolute left-0 top-full mt-1 w-56 bg-base-200 rounded-box shadow-xl border border-base-300 z-50 max-h-48 overflow-y-auto">
                    <div class="py-1">
                        <button v-for="account in accounts" :key="account.id" @click="
                            launchClientWithAccount(
                                contextMenu.client!,
                                account.id
                            )
                            "
                            class="w-full hover:scale-105 text-left px-3 py-2 text-sm hover:bg-primary/10 active:bg-primary/20 flex items-center justify-between transition-all duration-200 group">
                            <div class="flex flex-col gap-1 min-w-0 flex-1">
                                <span
                                    class="font-medium text-base-content truncate group-hover:text-primary transition-colors duration-200">
                                    {{ account.username }}
                                </span>
                                <div v-if="
                                    account.tags && account.tags.length > 0
                                " class="flex flex-wrap gap-1">
                                    <span v-for="tag in account.tags.slice(0, 2)" :key="tag"
                                        class="text-xs px-1.5 py-0.5 bg-base-300 text-base-content/70 rounded-full">
                                        {{ tag }}
                                    </span>
                                    <span v-if="account.tags.length > 2" class="text-xs text-base-content/50">
                                        +{{ account.tags.length - 2 }}
                                    </span>
                                </div>
                            </div>
                        </button>
                    </div>
                </div>
            </li>
            <li v-if="contextMenu.client?.meta.installed">
                <a @click="reinstallClient(contextMenu.client!)"
                    class="flex items-center gap-2 text-sm active:bg-primary/30 focus:bg-primary/20">
                    <RefreshCcw class="w-4 h-4" />
                    {{ t('common.reinstall') }}
                </a>
            </li>
            <li v-if="contextMenu.client?.meta.installed">
                <a @click="deleteClient(contextMenu.client!)"
                    class="flex items-center gap-2 text-sm active:bg-primary/30 focus:bg-primary/20">
                    <Trash2 class="w-4 h-4" />
                    {{ t('common.delete') }}
                </a>
            </li>
            <li v-if="contextMenu.client?.meta.installed">
                <a @click="openClientFolder(contextMenu.client!)"
                    class="flex items-center gap-2 text-sm active:bg-primary/30 focus:bg-primary/20">
                    <Folder class="w-4 h-4" />
                    {{ t('theme.actions.open_folder') }}
                </a>
            </li>
            <li v-if="contextMenu.client?.meta.installed">
                <a @click="copyClientLogs(contextMenu.client!)"
                    class="flex items-center gap-2 text-sm active:bg-primary/30 focus:bg-primary/20">
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
                <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none"
                    stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"
                    class="w-4 h-4 sm:w-5 sm:h-5">
                    <line x1="18" y1="6" x2="6" y2="18"></line>
                    <line x1="6" y1="6" x2="18" y2="18"></line>
                </svg>
            </button>
        </div>
    </transition>
</template>

<style>
.client-card-item {
    animation: fadeInUp 0.4s ease-out forwards;
    transition:
        transform 0.2s ease-out,
        box-shadow 0.2s ease-out;
    opacity: 0;
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

.menu.dropdown-content {
    transform-origin: top left;
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
</style>

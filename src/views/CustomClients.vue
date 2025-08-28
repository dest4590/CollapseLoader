<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { onMounted, onBeforeUnmount, ref, computed } from 'vue';
import {
    Plus,
    Edit3,
    Trash2,
    Play,
    StopCircle,
    AlertTriangle,
    FileText,
    Calendar,
    Settings,
} from 'lucide-vue-next';
import SearchBar from '../components/common/SearchBar.vue';
import { useToast } from '../services/toastService';
import { useModal } from '../services/modalService';
import { useI18n } from 'vue-i18n';
import type { CustomClient } from '../types/ui';
import AddCustomClientModal from '../components/modals/AddCustomClientModal.vue';
import EditCustomClientModal from '../components/modals/EditCustomClientModal.vue';
import DeleteCustomClientConfirmModal from '../components/modals/DeleteCustomClientConfirmModal.vue';
import CustomClientDisplaySettingsModal from '../components/modals/CustomClientDisplaySettingsModal.vue';

const { t } = useI18n();

const customClients = ref<CustomClient[]>([]);
const error = ref('');
const loading = ref(true);
const searchQuery = ref('');
const displayMode = ref<'global' | 'separate'>('separate');

const filteredClients = computed(() => {
    if (!searchQuery.value.trim()) return customClients.value;

    const query = searchQuery.value.trim().toLowerCase();
    return customClients.value.filter(client =>
        (client.name && client.name.toLowerCase().includes(query)) ||
        (client.version && client.version.toLowerCase().includes(query))
    );
});

const runningCustomClients = ref<number[]>([]);
const statusInterval = ref<number | null>(null);
const { addToast } = useToast();
const { showModal } = useModal();


const loadCustomClients = async () => {
    try {
        loading.value = true;
        const clients = await invoke<CustomClient[]>('get_custom_clients');
        customClients.value = clients.map(client => ({
            ...client,
            version: client.version.replace(/^V1_/, '1.').replace(/_/g, '.')
        }));
        error.value = '';
    } catch (err) {
        error.value = `Failed to load custom clients: ${err}`;
        addToast(`Failed to load custom clients: ${err}`, 'error');
    } finally {
        loading.value = false;
    }
};

const checkRunningStatus = async () => {
    try {
        const response = await invoke<number[]>('get_running_custom_client_ids');
        runningCustomClients.value = response;
    } catch (err) {
        console.error('Error checking custom client running status:', err);
    }
};

const isCustomClientRunning = (id: number): boolean => {
    return runningCustomClients.value.includes(id);
};

const handleLaunchClick = async (client: CustomClient) => {
    if (isCustomClientRunning(client.id)) {
        await stopCustomClient(client.id);
        return;
    }

    await handleLaunchClient(client);
};

const handleLaunchClient = async (client: CustomClient) => {
    try {
        const userToken = localStorage.getItem('authToken') || 'null';

        addToast(t('home.launching', { client: client.name }), 'info', 2000);

        await invoke('launch_custom_client', {
            id: client.id, userToken,
        });

        await new Promise((resolve) => setTimeout(resolve, 500));
        await checkRunningStatus();
    } catch (err) {
        addToast(`Failed to launch ${client.name}: ${err}`, 'error');
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

const handleAddClient = () => {
    showModal('add-custom-client', AddCustomClientModal, { title: t('modals.add_custom_client') }, {}, {
        'client-added': () => {
            addToast(t('modals.client_added'), 'success');
            loadCustomClients();
        }
    });
};

const handleEditClient = (client: CustomClient) => {
    showModal('edit-custom-client', EditCustomClientModal, { title: t('modals.edit_custom_client') }, { client }, {
        'client-edited': () => {
            addToast(t('modals.client_edited'), 'success');
            loadCustomClients();
        }
    });
};

const handleDeleteClient = (client: CustomClient) => {
    showModal('delete-custom-client-confirm', DeleteCustomClientConfirmModal, { title: t('modals.delete_custom_client') }, { client }, {
        'client-deleted': () => {
            addToast(t('modals.client_deleted'), 'success');
            loadCustomClients();
        }
    });
};

const loadDisplayMode = async () => {
    try {
        const flags = await invoke('get_flags');
        const typedFlags = flags as { custom_clients_display?: string };
        displayMode.value = (typedFlags.custom_clients_display === 'global' || typedFlags.custom_clients_display === 'separate')
            ? typedFlags.custom_clients_display
            : 'separate';
        return typedFlags;
    } catch (err) {
        console.error('Error loading flags:', err);
        addToast(`Failed to load flags: ${err}`, 'error');
        return {};
    }
};

const handleDisplaySettings = () => {
    showModal('custom-client-display-settings', CustomClientDisplaySettingsModal, { title: t("custom_clients.display_settings") }, {}, {});
}

const formatDate = (dateString: string): string => {
    try {
        const date = new Date(dateString);
        if (isNaN(date.getTime())) return 'Unknown';
        const day = String(date.getDate()).padStart(2, '0');
        const month = String(date.getMonth() + 1).padStart(2, '0');
        const year = String(date.getFullYear());
        return `${day}/${month}/${year}`;
    } catch {
        return 'Unknown';
    }
};

onMounted(async () => {
    await loadCustomClients();
    await loadDisplayMode();
    await checkRunningStatus();

    if (statusInterval.value !== null) {
        clearInterval(statusInterval.value);
    }

    statusInterval.value = setInterval(checkRunningStatus, 5000) as unknown as number;
});

onBeforeUnmount(() => {
    if (statusInterval.value !== null) {
        clearInterval(statusInterval.value);
        statusInterval.value = null;
    }
});


const handleSearch = (query: string) => {
    searchQuery.value = query;
};
</script>

<template>
    <div class="max-w-6xl mx-auto slide-up">
        <div class="flex justify-between items-center mb-4">
            <div>
                <h1 class="text-3xl font-bold text-primary mb-2">
                    {{ t('navigation.custom_clients') }}
                </h1>
            </div>
            <div class="flex gap-2">
                <button @click="handleAddClient" class="btn btn-primary gap-2">
                    <Plus class="w-4 h-4" />
                    {{ $t("custom_clients.add") }}
                </button>
                <button @click="handleDisplaySettings()" class="btn btn-secondary gap-2">
                    <Edit3 class="w-4 h-4" />
                    {{ $t("custom_clients.display_settings") }}
                </button>
            </div>
        </div>

        <div class="mb-6">
            <SearchBar @search="handleSearch" :initial-value="searchQuery" placeholder="Search custom clients..." />
        </div>

        <div v-if="loading" class="flex justify-center items-center py-12">
            <div class="loading loading-spinner loading-lg text-primary"></div>
        </div>

        <div v-else-if="error" class="alert alert-error">
            <AlertTriangle class="w-5 h-5" />
            <span>{{ error }}</span>
        </div>

        <div v-else-if="filteredClients.length === 0" class="text-center py-12">
            <div class="max-w-md mx-auto">
                <FileText class="w-16 h-16 text-base-content/30 mx-auto mb-4" />
                <h3 class="text-xl font-semibold mb-2">
                    {{ searchQuery ? 'No custom clients found' : 'No custom clients yet' }}
                </h3>
                <p class="text-base-content/70 mb-6">
                    {{ searchQuery
                        ? 'Try adjusting your search query.'
                        : 'Add your first custom client to get started.'
                    }}
                </p>
                <button v-if="!searchQuery" @click="handleAddClient" class="btn btn-primary">
                    <Plus class="w-4 h-4 mr-2" />
                    {{ $t('custom_clients.add') }}
                </button>
            </div>
        </div>

        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
            <div v-for="client in filteredClients" :key="client.id"
                class="card bg-base-200 shadow-md border border-base-300 hover:shadow-lg transition-all duration-300">
                <div class="card-body">
                    <div class="flex justify-between items-start mb-4">
                        <div class="flex-1">
                            <h3 class="card-title text-lg font-semibold mb-1">
                                {{ client.name }}
                                <div v-if="isCustomClientRunning(client.id)" class="badge badge-success badge-sm ml-2">
                                    {{ $t('custom_clients.running') }}
                                </div>
                            </h3>
                            <div class="badge badge-outline badge-sm">
                                {{ client.version }}
                            </div>
                        </div>
                        <div class="dropdown dropdown-end">
                            <button class="btn btn-ghost btn-sm btn-square">
                                <Settings class="w-4 h-4" />
                            </button>
                            <ul class="dropdown-content menu p-2 shadow bg-base-100 rounded-box w-52">
                                <li>
                                    <button @click="handleEditClient(client)" class="gap-2">
                                        <Edit3 class="w-4 h-4" />
                                        {{ $t('custom_clients.edit') }}
                                    </button>
                                </li>
                                <li>
                                    <button @click="handleDeleteClient(client)" class="gap-2 text-error">
                                        <Trash2 class="w-4 h-4" />
                                        {{ $t('custom_clients.delete') }}
                                    </button>
                                </li>
                            </ul>
                        </div>
                    </div>

                    <div class="space-y-3">
                        <div v-if="client.description" class="text-sm text-base-content/70">
                            {{ client.description }}
                        </div>

                        <div class="text-sm space-y-1">
                            <div class="flex items-center gap-2">
                                <Calendar class="w-4 h-4 text-primary" />
                                <span class="font-medium">{{ $t('custom_clients.added') }}:</span>
                                <span>{{ formatDate(client.created_at) }}</span>
                            </div>
                        </div>

                        <div class="card-actions justify-end">
                            <button @click="handleLaunchClick(client)" class="btn btn-sm gap-2"
                                :class="isCustomClientRunning(client.id) ? 'btn-error' : 'btn-primary'"
                                :disabled="!client.is_installed">
                                <StopCircle v-if="isCustomClientRunning(client.id)" class="w-4 h-4" />
                                <Play v-else class="w-4 h-4" />
                                {{ isCustomClientRunning(client.id) ? $t('custom_clients.stop') :
                                    $t('custom_clients.launch') }}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>
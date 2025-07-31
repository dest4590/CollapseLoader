<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { onMounted, ref, computed } from 'vue';
import {
    Plus,
    Edit3,
    Trash2,
    Play,
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

const { t } = useI18n();

const customClients = ref<CustomClient[]>([]);
const error = ref('');
const loading = ref(true);
const searchQuery = ref('');
const { addToast } = useToast();
const { showModal } = useModal();

const filteredClients = computed(() => {
    if (!searchQuery.value.trim()) return customClients.value;

    const query = searchQuery.value.trim().toLowerCase();
    return customClients.value.filter(client =>
        (client.name && client.name.toLowerCase().includes(query)) ||
        (client.version && client.version.toLowerCase().includes(query))
    );
});

const loadCustomClients = async () => {
    try {
        loading.value = true;
        const clients = await invoke<CustomClient[]>('get_custom_clients');
        customClients.value = clients;
        error.value = '';
    } catch (err) {
        error.value = `Failed to load custom clients: ${err}`;
        addToast(`Failed to load custom clients: ${err}`, 'error');
    } finally {
        loading.value = false;
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

const handleLaunchClient = async (client: CustomClient) => {
    try {
        const userToken = localStorage.getItem('authToken') || 'null';

        await invoke('launch_custom_client', {
            id: client.id, userToken,
        });

        addToast(`Launched ${client.name}`, 'success');
    } catch (err) {
        addToast(`Failed to launch ${client.name}: ${err}`, 'error');
    }
};

const formatDate = (dateString: string) => {
    return new Date(dateString).toLocaleDateString();
};

onMounted(async () => {
    await loadCustomClients();

    const unlisten = await listen('custom-client-launched', (event) => {
        const { name } = event.payload as { name: string };
        addToast(`${name} has been launched successfully!`, 'success');
    });

    return () => {
        unlisten();
    };
});
</script>

<template>
    <div class="max-w-6xl mx-auto slide-up">
        <div class="flex justify-between items-center mb-6">
            <div>
                <h1 class="text-3xl font-bold text-primary mb-2">
                    {{ t('navigation.custom_clients') }}
                </h1>
                <p class="text-base-content/70">
                    Manage your custom Minecraft clients
                </p>
            </div>
            <button @click="handleAddClient" class="btn btn-primary gap-2">
                <Plus class="w-4 h-4" />
                Add Custom Client
            </button>
        </div>

        <div class="mb-6">
            <SearchBar v-model="searchQuery" placeholder="Search custom clients..." class="max-w-md" />
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
                    Add Custom Client
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
                                        Edit
                                    </button>
                                </li>
                                <li>
                                    <button @click="handleDeleteClient(client)" class="gap-2 text-error">
                                        <Trash2 class="w-4 h-4" />
                                        Delete
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
                                <FileText class="w-4 h-4 text-primary" />
                                <span class="font-medium">Main Class:</span>
                                <code class="text-xs bg-base-300 px-2 py-1 rounded">
                                    {{ client.main_class }}
                                </code>
                            </div>

                            <div class="flex items-center gap-2">
                                <Calendar class="w-4 h-4 text-primary" />
                                <span class="font-medium">Added:</span>
                                <span>{{ formatDate(client.created_at) }}</span>
                            </div>

                            <div class="flex items-center gap-2">
                                <Play class="w-4 h-4 text-primary" />
                                <span class="font-medium">Launches:</span>
                                <span>{{ client.launches }}</span>
                            </div>
                        </div>

                        <div v-if="client.insecure" class="alert alert-warning py-2">
                            <AlertTriangle class="w-4 h-4" />
                            <span class="text-sm">This client is marked as insecure</span>
                        </div>

                        <div class="card-actions justify-end">
                            <button @click="handleLaunchClient(client)" class="btn btn-primary btn-sm gap-2"
                                :disabled="!client.is_installed">
                                <Play class="w-4 h-4" />
                                Launch
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>
<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import {
    AlertTriangle,
    Calendar,
    Copy,
    Edit3,
    FileText,
    Folder,
    Package,
    Play,
    Plus,
    Settings,
    StopCircle,
    Trash2,
} from "lucide-vue-next";
import SearchBar from "../components/common/SearchBar.vue";
import { useToast } from "../services/toastService";
import { useModal } from "../services/modalService";
import { useI18n } from "vue-i18n";
import { formatDate } from "../utils/utils";
import type { CustomClient } from "../types/ui";
import AddCustomClientModal from "../components/modals/clients/AddCustomClientModal.vue";
import EditCustomClientModal from "../components/modals/clients/EditCustomClientModal.vue";
import DeleteCustomClientConfirmModal from "../components/modals/clients/DeleteCustomClientConfirmModal.vue";
import CustomClientDisplaySettingsModal from "../components/modals/clients/CustomClientDisplaySettingsModal.vue";
import LogViewerModal from "../components/modals/clients/LogViewerModal.vue";
import CustomClientModsModal from "../components/modals/clients/CustomClientModsModal.vue";

const { t } = useI18n();

const customClients = ref<CustomClient[]>([]);
const error = ref("");
const loading = ref(true);
const searchQuery = ref("");
const displayMode = ref<"global" | "separate">("separate");

const filteredClients = computed(() => {
    if (!searchQuery.value.trim()) return customClients.value;
    const query = searchQuery.value.trim().toLowerCase();
    return customClients.value.filter(
        (client) =>
            (client.name && client.name.toLowerCase().includes(query)) ||
            (client.version && client.version.toLowerCase().includes(query))
    );
});

const runningCustomClients = ref<number[]>([]);
const statusInterval = ref<number | null>(null);
const { addToast } = useToast();
const { showModal } = useModal();

const contextMenu = ref({
    visible: false,
    x: 0,
    y: 0,
    client: null as CustomClient | null,
    animationClass: "",
});

const showContextMenu = (event: MouseEvent, client: CustomClient) => {
    event.preventDefault();
    event.stopPropagation();

    if (contextMenu.value.visible) {
        hideContextMenu();
        return;
    }

    const menuWidth = 224;
    const menuHeight = 280;
    let x = event.clientX;
    let y = event.clientY;

    if (x + menuWidth > window.innerWidth)
        x = window.innerWidth - menuWidth - 8;
    if (y + menuHeight > window.innerHeight)
        y = window.innerHeight - menuHeight - 8;

    contextMenu.value = {
        visible: true,
        x,
        y,
        client,
        animationClass: "context-menu-open-animation",
    };

    setTimeout(() => {
        contextMenu.value.animationClass = "";
    }, 150);

    document.addEventListener("click", hideContextMenu);
};

const hideContextMenu = () => {
    if (!contextMenu.value.visible) return;

    contextMenu.value.animationClass = "context-menu-close-animation";

    setTimeout(() => {
        contextMenu.value.visible = false;
        contextMenu.value.client = null;
        contextMenu.value.animationClass = "";
        document.removeEventListener("click", hideContextMenu);
    }, 150);
};

const loadCustomClients = async () => {
    try {
        loading.value = true;
        customClients.value =
            await invoke<CustomClient[]>("get_custom_clients");
        error.value = "";
    } catch (err) {
        error.value = t("custom_clients.load_failed", { error: err });
        addToast(t("custom_clients.load_failed", { error: err }), "error");
    } finally {
        loading.value = false;
    }
};

const checkCustomClientRunningStatus = async () => {
    try {
        runningCustomClients.value = await invoke<number[]>(
            "get_running_custom_client_ids"
        );
    } catch (err) {
        console.error("Error checking custom client running status:", err);
    }
};

const isCustomClientRunning = (id: number): boolean => {
    return runningCustomClients.value.includes(id);
};

const handleLaunchClient = async (client: CustomClient) => {
    hideContextMenu();
    try {
        const userToken = localStorage.getItem("authToken") || "null";
        addToast(t("home.launching", { client: client.name }), "info", 2000);
        await invoke("launch_custom_client", { id: client.id, userToken });
        await new Promise((resolve) => setTimeout(resolve, 500));
        await checkCustomClientRunningStatus();
    } catch (err) {
        addToast(
            t("custom_clients.launch_failed", {
                name: client.name,
                error: err,
            }),
            "error"
        );
    }
};

const stopCustomClient = async (id: number) => {
    hideContextMenu();
    try {
        const client = customClients.value.find((c) => c.id === id);
        if (client)
            addToast(t("home.stopping", { client: client.name }), "info", 2000);
        await invoke("stop_custom_client", { id });
        await new Promise((resolve) => setTimeout(resolve, 1000));
        await checkCustomClientRunningStatus();
    } catch (err) {
        addToast(t("custom_clients.stop_failed", { error: err }), "error");
    }
};

const handleAddClient = () => {
    showModal(
        "add-custom-client",
        AddCustomClientModal,
        { title: t("modals.add_custom_client") },
        {},
        {
            "client-added": () => {
                addToast(t("modals.client_added"), "success");
                loadCustomClients();
            },
        }
    );
};

const handleEditClient = (client: CustomClient) => {
    hideContextMenu();
    showModal(
        "edit-custom-client",
        EditCustomClientModal,
        { title: t("modals.edit_custom_client") },
        { client },
        {
            "client-edited": () => {
                addToast(t("modals.client_edited"), "success");
                loadCustomClients();
            },
        }
    );
};

const handleDeleteClient = (client: CustomClient) => {
    hideContextMenu();
    showModal(
        "delete-custom-client-confirm",
        DeleteCustomClientConfirmModal,
        { title: t("modals.delete_custom_client") },
        { client },
        {
            "client-deleted": () => {
                addToast(t("modals.client_deleted"), "success");
                loadCustomClients();
            },
        }
    );
};

const openLogViewer = (client: CustomClient) => {
    hideContextMenu();
    showModal(
        `log-viewer-${client.id}`,
        LogViewerModal,
        {
            title: t("logs.title", { client: client.name }),
            contentClass: "wide",
        },
        { clientId: client.id, clientName: client.name },
        { close: () => {} }
    );
};

const openClientFolder = async (client: CustomClient) => {
    hideContextMenu();
    try {
        await invoke("open_custom_client_folder", { id: client.id });
    } catch (err) {
        addToast(
            t("custom_clients.open_folder_failed", { error: err }),
            "error"
        );
    }
};

const openModsManager = (client: CustomClient) => {
    hideContextMenu();
    showModal(
        `mods-manager-custom-${client.id}`,
        CustomClientModsModal,
        { title: t("mods.manager_title"), contentClass: "wide" },
        { client },
        { close: () => {} }
    );
};

const copyClientLogs = async (client: CustomClient) => {
    hideContextMenu();
    try {
        const logs = await invoke<string>("get_latest_client_logs", {
            id: client.id,
        });
        await navigator.clipboard.writeText(logs);
        addToast(t("logs.copied"), "success");
    } catch (err) {
        addToast(t("logs.copy_failed", { error: err }), "error");
    }
};

const loadDisplayMode = async () => {
    try {
        const flags = await invoke("get_flags");
        const typedFlags = flags as { custom_clients_display?: { value: string } };
        displayMode.value =
            typedFlags.custom_clients_display?.value === "global" ||
            typedFlags.custom_clients_display?.value === "separate"
                ? typedFlags.custom_clients_display.value
                : "separate";
        return typedFlags;
    } catch (err) {
        addToast(t("custom_clients.flags_failed", { error: err }), "error");
        return {};
    }
};

const handleDisplaySettings = () => {
    showModal(
        "custom-client-display-settings",
        CustomClientDisplaySettingsModal,
        { title: t("custom_clients.display_settings") },
        {},
        {}
    );
};

onMounted(async () => {
    await loadCustomClients();
    await loadDisplayMode();
    await checkCustomClientRunningStatus();

    if (statusInterval.value !== null) clearInterval(statusInterval.value);

    statusInterval.value = setInterval(
        checkCustomClientRunningStatus,
        5000
    ) as unknown as number;
});

onBeforeUnmount(() => {
    if (statusInterval.value !== null) {
        clearInterval(statusInterval.value);
        statusInterval.value = null;
    }
    hideContextMenu();
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
                    {{ t("navigation.custom_clients") }}
                </h1>
            </div>
            <div class="flex gap-2">
                <button @click="handleAddClient" class="btn btn-primary gap-2">
                    <Plus class="w-4 h-4" />
                    {{ $t("custom_clients.add") }}
                </button>
                <button
                    @click="handleDisplaySettings()"
                    class="btn btn-secondary gap-2"
                >
                    <Edit3 class="w-4 h-4" />
                    {{ $t("custom_clients.display_settings") }}
                </button>
            </div>
        </div>

        <div class="mb-6">
            <SearchBar
                @search="handleSearch"
                :initial-value="searchQuery"
                :placeholder="t('custom_clients.search_placeholder')"
            />
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
                    {{
                        searchQuery
                            ? t("custom_clients.no_results")
                            : t("custom_clients.no_clients_yet")
                    }}
                </h3>
                <button
                    v-if="!searchQuery"
                    @click="handleAddClient"
                    class="btn btn-primary"
                >
                    <Plus class="w-4 h-4 mr-2" />
                    {{ $t("custom_clients.add") }}
                </button>
            </div>
        </div>

        <div
            v-else
            class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6"
        >
            <div
                v-for="client in filteredClients"
                :key="client.id"
                class="card bg-base-200 shadow-md border border-base-300 hover:shadow-lg transition-all duration-300"
            >
                <div class="card-body">
                    <div class="flex justify-between items-start mb-4">
                        <div class="flex-1">
                            <h3 class="card-title text-lg font-semibold mb-1">
                                {{ client.name }}
                                <div
                                    v-if="isCustomClientRunning(client.id)"
                                    class="badge badge-success badge-sm ml-2"
                                >
                                    {{ $t("custom_clients.running") }}
                                </div>
                            </h3>
                            <div class="badge badge-outline badge-sm">
                                {{ client.version }}
                            </div>
                        </div>
                        <button
                            @click.stop="showContextMenu($event, client)"
                            class="btn btn-ghost btn-sm btn-square"
                        >
                            <Settings class="w-4 h-4" />
                        </button>
                    </div>

                    <div class="space-y-3">
                        <div
                            v-if="client.description"
                            class="text-sm text-base-content/70"
                        >
                            {{ client.description }}
                        </div>

                        <div class="text-sm space-y-1">
                            <div class="flex items-center gap-2">
                                <Calendar class="w-4 h-4 text-primary" />
                                <span class="font-medium"
                                    >{{ $t("custom_clients.added") }}:</span
                                >
                                <span>{{ formatDate(client.created_at) }}</span>
                            </div>
                        </div>

                        <div class="card-actions justify-end">
                            <button
                                @click="
                                    isCustomClientRunning(client.id)
                                        ? stopCustomClient(client.id)
                                        : handleLaunchClient(client)
                                "
                                class="btn btn-sm gap-2"
                                :class="
                                    isCustomClientRunning(client.id)
                                        ? 'btn-error'
                                        : 'btn-primary'
                                "
                                :disabled="!client.is_installed"
                            >
                                <StopCircle
                                    v-if="isCustomClientRunning(client.id)"
                                    class="w-4 h-4"
                                />
                                <Play v-else class="w-4 h-4" />
                                {{
                                    isCustomClientRunning(client.id)
                                        ? $t("custom_clients.stop")
                                        : $t("custom_clients.launch")
                                }}
                            </button>
                            <button
                                @click="openLogViewer(client)"
                                class="btn btn-sm btn-ghost gap-2"
                            >
                                <FileText class="w-4 h-4" />
                                {{ $t("logs.view") }}
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>

    <div
        v-if="contextMenu.visible"
        :style="{ top: `${contextMenu.y}px`, left: `${contextMenu.x}px` }"
        class="fixed z-100 menu p-0 bg-base-200 w-56 rounded-box shadow-xl border border-base-300 dropdown-content"
        :class="contextMenu.animationClass"
    >
        <h3
            class="font-medium text-sm px-4 py-2 border-b border-base-300 text-base-content/80 bg-base-300/30"
        >
            {{ contextMenu.client?.name }}
        </h3>
        <ul>
            <li
                v-if="
                    contextMenu.client?.is_installed &&
                    isCustomClientRunning(contextMenu.client.id)
                "
            >
                <a
                    @click="handleLaunchClient(contextMenu.client!)"
                    class="flex items-center gap-2 text-sm active:bg-primary/30"
                >
                    <Plus class="w-4 h-4" />
                    {{ t("home.launch_another") }}
                </a>
            </li>
            <li v-if="contextMenu.client?.is_installed">
                <a
                    @click="openClientFolder(contextMenu.client!)"
                    class="flex items-center gap-2 text-sm active:bg-primary/30"
                >
                    <Folder class="w-4 h-4" />
                    {{ t("theme.actions.open_folder") }}
                </a>
            </li>
            <li
                v-if="
                    contextMenu.client?.is_installed &&
                    contextMenu.client?.client_type?.toLowerCase() === 'fabric'
                "
            >
                <a
                    @click="openModsManager(contextMenu.client!)"
                    class="flex items-center gap-2 text-sm active:bg-primary/30 text-primary font-medium"
                >
                    <Package class="w-4 h-4" />
                    {{ t("mods.manage_mods") }}
                </a>
            </li>
            <li v-if="contextMenu.client?.is_installed">
                <a
                    @click="copyClientLogs(contextMenu.client!)"
                    class="flex items-center gap-2 text-sm active:bg-primary/30"
                >
                    <Copy class="w-4 h-4" />
                    {{ t("logs.copy_logs") }}
                </a>
            </li>
            <li>
                <a
                    @click="handleEditClient(contextMenu.client!)"
                    class="flex items-center gap-2 text-sm active:bg-primary/30"
                >
                    <Edit3 class="w-4 h-4" />
                    {{ t("custom_clients.edit") }}
                </a>
            </li>
            <li>
                <a
                    @click="handleDeleteClient(contextMenu.client!)"
                    class="flex items-center gap-2 text-sm active:bg-primary/30 text-error"
                >
                    <Trash2 class="w-4 h-4" />
                    {{ t("custom_clients.delete") }}
                </a>
            </li>
        </ul>
    </div>
</template>

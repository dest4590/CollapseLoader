<script setup lang="ts">
import type { Client, InstallProgress } from "@shared/types/ui";
import ClientCard from "@features/clients/components/ClientCard.vue";

defineProps<{
    filteredClients: Client[];
    playClientSlideAnim: boolean;
    isCtrlPressed: boolean;
    expandedClientId: number | null;
    requirementsInProgress: boolean;
    isAnyClientDownloading: boolean;
    hashVerifyingClients: Set<number>;
    isAnyCardExpanded: boolean;
    isShiftHeld: boolean;
    installationStatus: Map<string, InstallProgress>;
    selectedClientsSize: number;
    getFileBasename: (filename: string) => string;
    isClientRunning: (clientId: number) => boolean;
    isClientInstalling: (client: Client) => boolean;
    isClientFavorite: (clientId: number) => boolean;
    isClientSelected: (clientId: number) => boolean;
}>();

const emit = defineEmits<{
    launch: [Client];
    download: [number];
    "open-log-viewer": [Client];
    "open-ram-viewer": [Client];
    "show-context-menu": [MouseEvent, Client];
    "client-click": [Client, MouseEvent];
    "expanded-state-changed": [number, boolean];
    "show-user-profile": [number];
}>();

const handleLaunch = (client: Client) => emit("launch", client);
const handleDownload = (clientId: number) => emit("download", clientId);
const handleOpenLogViewer = (client: Client) => emit("open-log-viewer", client);
const handleOpenRamViewer = (client: Client) => emit("open-ram-viewer", client);
const handleShowContextMenu = (event: MouseEvent, client: Client) =>
    emit("show-context-menu", event, client);
const handleClientClick = (client: Client, event: MouseEvent) =>
    emit("client-click", client, event);
const handleExpandedStateChanged = (clientId: number, isExpanded: boolean) =>
    emit("expanded-state-changed", clientId, isExpanded);
const handleShowUserProfile = (userId: number) =>
    emit("show-user-profile", userId);
</script>

<template>
    <div
        class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 gap-4 relative overflow-hidden"
        :class="{
            'multi-select-mode': isCtrlPressed && expandedClientId === null,
        }"
        :style="{ paddingBottom: selectedClientsSize > 0 ? '80px' : '0px' }"
    >
        <div
            v-for="(client, idx) in filteredClients"
            :key="`${client.id}-${idx}`"
            :class="[
                'client-card-item',
                { 'slide-in-animate': playClientSlideAnim },
            ]"
            :style="
                playClientSlideAnim
                    ? { animationDelay: `${Math.min(idx * 100, 600)}ms` }
                    : {}
            "
        >
            <ClientCard
                :client="client"
                :isClientRunning="isClientRunning(client.id)"
                :isClientInstalling="isClientInstalling(client)"
                :installationStatus="
                    installationStatus.get(getFileBasename(client.filename))
                "
                :isRequirementsInProgress="requirementsInProgress"
                :isAnyClientDownloading="isAnyClientDownloading"
                :isFavorite="isClientFavorite(client.id)"
                :isSelected="isClientSelected(client.id)"
                :isMultiSelectMode="isCtrlPressed && expandedClientId === null"
                :isHashVerifying="hashVerifyingClients.has(client.id)"
                :isAnyCardExpanded="isAnyCardExpanded"
                :isShiftHeld="isShiftHeld"
                @launch="handleLaunch"
                @download="handleDownload"
                @open-log-viewer="handleOpenLogViewer"
                @open-ram-viewer="handleOpenRamViewer"
                @show-context-menu="handleShowContextMenu"
                @client-click="handleClientClick"
                @expanded-state-changed="handleExpandedStateChanged"
                @show-user-profile="handleShowUserProfile"
            />
        </div>
    </div>
</template>

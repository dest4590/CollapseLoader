<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import {
    Copy,
    Folder,
    Link,
    Package,
    Plus,
    RefreshCcw,
    Star,
    Trash2,
} from "lucide-vue-next";
import type { Client } from "@shared/types/ui";

const { t } = useI18n();

const props = defineProps<{
    visible: boolean;
    x: number;
    y: number;
    client: Client | null;
    selectedClientsSize: number;
    isFavorite: boolean;
    isRunning: boolean;
    isMacOS: boolean;
    animationClass: string;
}>();

const emit = defineEmits<{
    "open-mods-manager": [Client];
    "toggle-favorite": [Client];
    "launch-another-instance": [Client];
    "reinstall-client": [Client];
    "delete-client": [Client];
    "open-client-folder": [Client];
    "copy-client-logs": [Client];
    "open-create-shortcut": [Client];
}>();

const client = computed(() => props.client);

const handleOpenModsManager = () => {
    if (!client.value) return;
    emit("open-mods-manager", client.value);
};

const handleToggleFavorite = () => {
    if (!client.value) return;
    emit("toggle-favorite", client.value);
};

const handleLaunchAnotherInstance = () => {
    if (!client.value) return;
    emit("launch-another-instance", client.value);
};

const handleReinstallClient = () => {
    if (!client.value) return;
    emit("reinstall-client", client.value);
};

const handleDeleteClient = () => {
    if (!client.value) return;
    emit("delete-client", client.value);
};

const handleOpenClientFolder = () => {
    if (!client.value) return;
    emit("open-client-folder", client.value);
};

const handleCopyClientLogs = () => {
    if (!client.value) return;
    emit("copy-client-logs", client.value);
};

const handleOpenCreateShortcut = () => {
    if (!client.value) return;
    emit("open-create-shortcut", client.value);
};
</script>

<template>
    <div
        v-if="visible"
        class="fixed z-100 menu p-0 bg-base-200 w-56 rounded-box shadow-xl border border-base-300 dropdown-content context-menu"
        :style="{ top: `${y}px`, left: `${x}px` }"
        :class="animationClass"
    >
        <h3
            v-if="selectedClientsSize <= 1"
            class="context-menu-header font-medium text-sm px-4 py-2 border-b border-base-300 text-base-content/80 bg-base-300/30"
        >
            {{ client?.name }}
        </h3>

        <h3
            v-else
            class="context-menu-header font-medium text-sm px-4 py-2 border-b border-base-300 text-base-content/80 bg-base-300/30"
        >
            {{
                t("home.multiple_clients_selected", {
                    count: selectedClientsSize,
                })
            }}
        </h3>

        <ul v-if="selectedClientsSize <= 1" class="menu-items">
            <li
                v-if="
                    client?.client_type?.toLowerCase() === 'fabric' &&
                    client?.meta.installed
                "
            >
                <a
                    @click.prevent="handleOpenModsManager"
                    class="flex items-center gap-2 text-sm active:bg-primary/30 text-primary font-medium"
                >
                    <Package class="w-4 h-4" />
                    {{ t("mods.manage_mods") }}
                </a>
            </li>
            <li>
                <a
                    @click.prevent="handleToggleFavorite"
                    class="flex items-center gap-2 text-sm active:bg-primary/30"
                >
                    <Star
                        class="w-4 h-4"
                        :class="{
                            'fill-yellow-400 text-yellow-400': isFavorite,
                        }"
                    />
                    {{
                        isFavorite
                            ? t("theme.actions.remove_favorite")
                            : t("theme.actions.add_favorite")
                    }}
                </a>
            </li>
            <li v-if="client?.meta.installed && isRunning">
                <a
                    @click.prevent="handleLaunchAnotherInstance"
                    class="flex items-center gap-2 text-sm active:bg-primary/30"
                >
                    <Plus class="w-4 h-4" />
                    {{ t("home.launch_another") }}
                </a>
            </li>
            <li v-if="client?.meta.installed">
                <a
                    @click.prevent="handleReinstallClient"
                    class="flex items-center gap-2 text-sm active:bg-primary/30"
                >
                    <RefreshCcw class="w-4 h-4" />
                    {{ t("common.reinstall") }}
                </a>
            </li>
            <li v-if="client?.meta.installed">
                <a
                    @click.prevent="handleDeleteClient"
                    class="flex items-center gap-2 text-sm active:bg-primary/30"
                >
                    <Trash2 class="w-4 h-4" />
                    {{ t("common.delete") }}
                </a>
            </li>
            <li v-if="client?.meta.installed">
                <a
                    @click.prevent="handleOpenClientFolder"
                    class="flex items-center gap-2 text-sm active:bg-primary/30"
                >
                    <Folder class="w-4 h-4" />
                    {{ t("theme.actions.open_folder") }}
                </a>
            </li>
            <li v-if="client?.meta.installed">
                <a
                    @click.prevent="handleCopyClientLogs"
                    class="flex items-center gap-2 text-sm active:bg-primary/30"
                >
                    <Copy class="w-4 h-4" />
                    {{ t("logs.copy_logs") }}
                </a>
            </li>
            <li v-if="client?.meta.installed && !isMacOS">
                <a
                    @click.prevent="handleOpenCreateShortcut"
                    class="flex items-center gap-2 text-sm active:bg-primary/30"
                >
                    <Link class="w-4 h-4" />
                    {{ t("modals.create_shortcut.menu_item") }}
                </a>
            </li>
        </ul>
    </div>
</template>

<style scoped>
.context-menu {
    position: fixed;
    transform-origin: top left;
    will-change: transform, opacity;
}

@keyframes context-menu-scale-in {
    from {
        opacity: 0;
        transform: scale(0.94);
    }
    to {
        opacity: 1;
        transform: scale(1);
    }
}

@keyframes context-menu-scale-out {
    from {
        opacity: 1;
        transform: scale(1);
    }
    to {
        opacity: 0;
        transform: scale(0.94);
    }
}

@keyframes context-menu-item-in {
    from {
        opacity: 0;
        transform: translateY(10px);
    }
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.context-menu-open-animation {
    animation: context-menu-scale-in 260ms cubic-bezier(0.16, 1, 0.3, 1) forwards;
}

.context-menu-close-animation {
    animation: context-menu-scale-out 220ms cubic-bezier(0.34, 0.84, 0.42, 0.96) forwards;
}

.context-menu-open-animation .context-menu-header,
.context-menu-open-animation .menu-items > li {
    opacity: 0;
    animation: context-menu-item-in 220ms cubic-bezier(0.22, 1, 0.36, 1) both;
}

.context-menu-open-animation .context-menu-header {
    animation-delay: 100ms;
}

.context-menu-open-animation .menu-items > li:nth-of-type(1) {
    animation-delay: 140ms;
}

.context-menu-open-animation .menu-items > li:nth-of-type(2) {
    animation-delay: 180ms;
}

.context-menu-open-animation .menu-items > li:nth-of-type(3) {
    animation-delay: 220ms;
}

.context-menu-open-animation .menu-items > li:nth-of-type(4) {
    animation-delay: 260ms;
}

.context-menu-open-animation .menu-items > li:nth-of-type(5) {
    animation-delay: 300ms;
}

.context-menu-open-animation .menu-items > li:nth-of-type(6) {
    animation-delay: 340ms;
}

.context-menu-open-animation .menu-items > li:nth-of-type(7) {
    animation-delay: 380ms;
}

.context-menu-open-animation .menu-items > li:nth-of-type(8) {
    animation-delay: 420ms;
}

.menu.dropdown-content li a {
    transition: background-color 0.2s ease, color 0.2s ease, transform 0.2s ease;
}

.menu.dropdown-content li a:hover {
    transform: translateX(2px);
}

@media (prefers-reduced-motion: reduce) {
    .context-menu-open-animation,
    .context-menu-close-animation,
    .context-menu-open-animation .context-menu-header,
    .context-menu-open-animation .menu-items > li {
        animation: none !important;
    }
}
</style>

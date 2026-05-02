<script setup lang="ts">
import { Download, StopCircle, Terminal } from "lucide-vue-next";
import { useI18n } from "vue-i18n";
import type { Client, InstallProgress } from "@shared/types/ui";

const { t } = useI18n();

defineProps<{
    client: Client;
    clientIsInstalling: boolean;
    currentInstallStatus: InstallProgress | undefined;
    clientIsRunning: boolean;
    isRequirementsInProgress: boolean;
    isAnimatingOrExpanded: boolean;
}>();

const emit = defineEmits(["launch", "download", "open-log-viewer"]);

const handleLaunchClick = () => {
    emit("launch");
};

const handleDownloadClick = () => {
    emit("download");
};

const handleOpenLogViewer = () => {
    emit("open-log-viewer");
};
</script>

<template>
    <transition-group
        name="action-list"
        tag="div"
        class="card-actions justify-end mt-2 relative"
        :class="[isAnimatingOrExpanded ? 'mr-4' : '']"
    >
        <div v-if="clientIsInstalling" key="installing" class="w-full">
            <div class="flex justify-between mb-1 text-xs text-base-content">
                <span class="truncate max-w-[90%]">
                    {{ currentInstallStatus?.action }}
                    {{ client.name }}
                </span>
                <span>{{ currentInstallStatus?.percentage }}%</span>
            </div>
            <div class="progress-bar-container">
                <div
                    class="progress-bar"
                    :style="{
                        width: `${currentInstallStatus?.percentage}%`,
                    }"
                ></div>
            </div>
        </div>

        <div v-else key="standard-actions" class="flex items-center space-x-2">
            <button
                v-if="!client.meta.installed"
                @click="handleDownloadClick"
                class="btn btn-sm btn-primary relative overflow-hidden group"
                :disabled="isRequirementsInProgress || !client.working"
            >
                <span
                    class="flex items-center justify-center w-full transition-all duration-300 group-hover:opacity-0 group-hover:-translate-y-3"
                >
                    <Download v-if="client.working" class="w-4 h-4 mr-1" />
                    <span v-if="client.working">{{ t("home.download") }}</span>
                    <span v-else-if="!client.working">{{
                        t("home.unavailable")
                    }}</span>
                </span>
                <span
                    class="absolute inset-0 flex items-center justify-center opacity-0 translate-y-3 transition-all duration-300 group-hover:opacity-100 group-hover:translate-y-0"
                >
                    {{ client.meta.size || "0" }} MB
                </span>
            </button>
            <button
                v-else
                @click="handleLaunchClick"
                class="btn btn-sm min-w-20 transition-all duration-300 active:scale-90"
                :disabled="isRequirementsInProgress"
                :class="[
                    clientIsRunning
                        ? 'btn-error animate-pulse-subtle'
                        : 'btn-primary',
                    'hover:shadow-lg hover:shadow-primary/20',
                ]"
            >
                <StopCircle class="w-4 h-4 mr-1" v-if="clientIsRunning" />
                {{ clientIsRunning ? t("home.stop") : t("home.launch") }}
            </button>
        </div>

        <button
            v-if="clientIsRunning && !clientIsInstalling"
            key="terminal"
            @click.stop="handleOpenLogViewer"
            class="btn btn-sm btn-ghost btn-circle text-info hover:bg-info/20 ml-2 transition-transform hover:rotate-12 active:scale-90"
        >
            <Terminal class="w-4 h-4" />
        </button>
    </transition-group>
</template>

<style scoped>
@reference 'tailwindcss';
@plugin 'daisyui';

.progress-bar-container {
    @apply w-full h-1.5 bg-base-content/10 rounded-full overflow-hidden relative;
}

.progress-bar {
    @apply h-full bg-primary transition-all duration-300 ease-out relative;
}

.progress-bar::after {
    content: "";
    @apply absolute inset-0 bg-linear-to-r from-transparent via-white/20 to-transparent;
    animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
    from {
        transform: translateX(-100%);
    }
    to {
        transform: translateX(100%);
    }
}

@keyframes pulse-subtle {
    0%,
    100% {
        opacity: 1;
        transform: scale(1);
    }
    50% {
        opacity: 0.85;
        transform: scale(0.98);
    }
}

.animate-pulse-subtle {
    animation: pulse-subtle 2s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

.action-list-enter-active,
.action-list-leave-active {
    transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.action-list-enter-from {
    opacity: 0;
    transform: scale(0.8) translateY(10px);
}

.action-list-leave-to {
    opacity: 0;
    transform: scale(0.8) translateY(-10px);
}
</style>

<style scoped>
.card-actions {
    min-height: 2rem;
    display: flex;
    align-items: center;
}

.progress-bar-container {
    height: 6px;
    background-color: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    overflow: hidden;
}

.progress-bar {
    height: 100%;
    background: linear-gradient(90deg, #3b3b3b, #777777);
    border-radius: 2px;
    transition: width 0.2s ease-out;
}

.action-list-move,
.action-list-enter-active,
.action-list-leave-active {
    transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

.action-list-enter-from,
.action-list-leave-to {
    opacity: 0;
    transform: translateX(20px);
}

.action-list-leave-active {
    position: absolute;
}
</style>

<script setup lang="ts">
import { computed } from 'vue';
import { CircleDot, Download, Play } from 'lucide-vue-next';
import { Client } from '../../../types/ui';

const props = defineProps<{
    client: Client;
    expanded: boolean;
}>();

const formattedDownloads = computed(() => {
    if (props.client.downloads >= 1000000) {
        return `${(props.client.downloads / 1000000).toFixed(1)}M`;
    } else if (props.client.downloads >= 1000) {
        return `${(props.client.downloads / 1000).toFixed(1)}K`;
    }
    return props.client.downloads.toString();
});

const formattedLaunches = computed(() => {
    if (props.client.launches >= 1000000) {
        return `${(props.client.launches / 1000000).toFixed(1)}M`;
    } else if (props.client.launches >= 1000) {
        return `${(props.client.launches / 1000).toFixed(1)}K`;
    }
    return props.client.launches.toString();
});
</script>

<template>
    <div class="client-stats stats-compact">
        <div class="stats">
            <CircleDot class="w-3 h-3 opacity-60" />
            <span class="text-xs font-semibold opacity-70">{{ client.version }}</span>

        </div>
        <transition name="fade">
            <div v-if="expanded" class="expanded-stats stats">
                <span class="stat-separator">•</span>

                <div class="stats">
                    <Download class="w-3 h-3 opacity-60" />
                    <span class="text-xs font-medium">{{ formattedDownloads }}</span>
                </div>

                <span class="stat-separator">•</span>

                <div class="stats">
                    <Play class="w-3 h-3 opacity-60" />
                    <span class="text-xs font-medium">{{ formattedLaunches }}</span>
                </div>
            </div>
        </transition>
    </div>
</template>

<style scoped>
.client-stats {
    display: flex;
    align-items: center;
    font-size: 0.75rem;
    color: rgba(var(--bc), 0.6);
    gap: 0.375rem;
}

.stats {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    transition: color 0.2s ease;
}

.stats:hover {
    color: rgba(var(--bc), 0.8);
}

.stat-separator {
    font-size: 0.625rem;
    opacity: 0.4;
}

.fade-enter-active,
.fade-leave-active {
    transition: opacity 0.3s;
}

.fade-enter-from,
.fade-leave-to {
    opacity: 0;
}

.fade-enter-to,
.fade-leave-from {
    opacity: 1;
}

.expanded-stats {
    display: flex;
    align-items: center;
}
</style>

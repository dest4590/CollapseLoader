<script setup lang="ts">
import { ref, onMounted, nextTick, watch } from 'vue';
import { bootLogService } from '../../services/bootLogService';

interface LogEntry {
    id: number;
    timestamp: string;
    level: 'OK' | 'INFO' | 'WARN' | 'ERROR';
    service: string;
    message: string;
}

const props = defineProps<{
    currentProgress: number;
    loadingState: string;
}>();

const logContainer = ref<HTMLElement | null>(null);
const displayedLogs = ref<LogEntry[]>([]);
const activeTimeouts = ref<NodeJS.Timeout[]>([]);

const logs = bootLogService.getLogs();

const addLogToDisplay = async (logEntry: LogEntry) => {
    displayedLogs.value.push(logEntry);
    await nextTick();

    if (logContainer.value) {
        logContainer.value.scrollTop = logContainer.value.scrollHeight;
    }
};

const clearTimeouts = () => {
    activeTimeouts.value.forEach((timeoutId) => clearTimeout(timeoutId));
    activeTimeouts.value = [];
};

const updateDisplayedLogs = () => {
    clearTimeouts();

    const maxLogsToShow = Math.floor(props.currentProgress * logs.value.length);

    if (maxLogsToShow < displayedLogs.value.length) {
        displayedLogs.value = [];
    }

    let cumulativeDelay = 0;
    for (
        let i = displayedLogs.value.length;
        i < maxLogsToShow && i < logs.value.length;
        i++
    ) {
        const log = logs.value[i];
        if (!log) break;

        cumulativeDelay += 50;

        const timeoutId = setTimeout(async () => {
            await addLogToDisplay(log);
        }, cumulativeDelay);

        activeTimeouts.value.push(timeoutId);
    }
};

const getLevelColor = (level: string) => {
    switch (level) {
        case 'OK':
            return 'text-green-400';
        case 'INFO':
            return 'text-blue-400';
        case 'WARN':
            return 'text-yellow-400';
        case 'ERROR':
            return 'text-red-400';
        default:
            return 'text-gray-400';
    }
};

const getLevelSymbol = (level: string) => {
    switch (level) {
        case 'OK':
            return '[  OK  ]';
        case 'INFO':
            return '[ INFO ]';
        case 'WARN':
            return '[ WARN ]';
        case 'ERROR':
            return '[ERROR ]';
        default:
            return '[     ]';
    }
};

watch(
    () => props.currentProgress,
    () => {
        updateDisplayedLogs();
    },
    { immediate: true }
);

watch(
    logs,
    () => {
        updateDisplayedLogs();
    },
    { deep: true }
);

onMounted(() => {
    updateDisplayedLogs();
});
</script>

<template>
    <div class="boot-logs-container">
        <div ref="logContainer" class="log-output">
            <div v-for="(log, _) in displayedLogs" :key="log.id" class="log-entry" :class="getLevelColor(log.level)">
                <span class="timestamp">{{ log.timestamp }}</span>
                <span class="level-badge" :class="getLevelColor(log.level)">{{
                    getLevelSymbol(log.level)
                    }}</span>
                <span class="service">[{{ log.service }}]</span>
                <span class="message">{{ log.message }}</span>
            </div>
        </div>
    </div>
</template>

<style scoped>
.boot-logs-container {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    z-index: 1;
    overflow: hidden;
    pointer-events: none;
}

.log-output {
    height: 100%;
    overflow-y: auto;
    padding: 20px;
    font-family: 'Courier New', monospace;
    font-size: 12px;
    line-height: 1.4;
    color: #00ff00;
    background: rgba(0, 0, 0, 0.1);
    position: relative;

    scrollbar-width: none;
    -ms-overflow-style: none;
}

.log-output::after {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(to bottom,
            rgba(0, 0, 0, 0.8) 0%,
            rgba(0, 0, 0, 0.7) 20%,
            rgba(0, 0, 0, 0.6) 50%,
            rgba(0, 0, 0, 0.5) 80%,
            rgba(0, 0, 0, 0.3) 100%);
    pointer-events: none;
    z-index: 1;
}

.log-entry {
    display: flex;
    align-items: center;
    margin-bottom: 1px;
    opacity: 0;
    transform: translateY(10px);
    animation: logAppear 0.3s ease forwards;
    white-space: nowrap;
}

.timestamp {
    color: #666;
    margin-right: 10px;
}

.level-badge {
    font-weight: bold;
    margin-right: 5px;
    min-width: 60px;
}

.service {
    color: #888;
    min-width: 70px;
}

.message {
    flex: 1;
}

@keyframes logAppear {
    from {
        opacity: 0;
        transform: translateY(10px);
    }

    to {
        opacity: 0.8;
        transform: translateY(0);
    }
}

@keyframes blink {

    0%,
    50% {
        opacity: 1;
    }

    51%,
    100% {
        opacity: 0;
    }
}

:root[data-theme='dark'] .log-output {
    color: #00ff00;
    background: rgba(0, 0, 0, 0.15);
}

:root[data-theme='light'] .log-output {
    color: #003300;
    background: rgba(255, 255, 255, 0.05);
}

:root[data-theme='light'] .timestamp {
    color: #999;
}

:root[data-theme='light'] .service {
    color: #666;
}

:root[data-theme='light'] .text-green-400 {
    color: #16a34a !important;
}

:root[data-theme='light'] .text-blue-400 {
    color: #2563eb !important;
}

:root[data-theme='light'] .text-yellow-400 {
    color: #ca8a04 !important;
}

:root[data-theme='light'] .text-red-400 {
    color: #dc2626 !important;
}
</style>
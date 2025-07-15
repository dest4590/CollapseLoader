<template>
    <div class="toast-notification" :class="`toast-${toast.type}`" @mouseenter="handleMouseEnter"
        @mouseleave="handleMouseLeave">
        <div class="toast-content">
            <div class="toast-icon">
                <component :is="currentIcon" v-if="currentIcon" class="w-5 h-5" />
            </div>
            <div class="toast-message">{{ displayMessage }}</div>
            <button class="toast-close-btn" @click.stop="closeToast">
                <X class="w-4 h-4" />
            </button>
        </div>
        <div class="toast-progress-container">
            <div class="toast-progress" :style="{ animationDuration: `${toast.duration}ms` }"
                :class="{ 'toast-progress-paused': isPaused }"></div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue';
import type { ToastMessage, ToastType } from '../../types/toast';
import {
    CheckCircle,
    XCircle,
    AlertCircle,
    AlertTriangle,
    X,
} from 'lucide-vue-next';
import { useToast } from '../../services/toastService';

interface Props {
    toast: ToastMessage;
}

const props = defineProps<Props>();
const emit = defineEmits(['close']);

const { pauseToast, resumeToast, getDisplayMessage } = useToast();
const isPaused = ref(false);

const iconMap: Record<ToastType, any> = {
    success: CheckCircle,
    error: XCircle,
    info: AlertCircle,
    warning: AlertTriangle,
};

const currentIcon = computed(() => iconMap[props.toast.type]);
const displayMessage = computed(() => getDisplayMessage(props.toast));

const closeToast = () => {
    emit('close', props.toast.id);
};

const handleMouseEnter = () => {
    if (
        props.toast.duration &&
        props.toast.duration > 0 &&
        props.toast.timeoutId
    ) {
        isPaused.value = true;
        pauseToast(props.toast.id);
    }
};

const handleMouseLeave = () => {
    if (
        props.toast.duration &&
        props.toast.duration > 0 &&
        typeof props.toast.remainingDuration === 'number'
    ) {
        isPaused.value = false;
        resumeToast(props.toast.id);
    }
};
</script>

<style scoped>
.toast-notification {
    display: flex;
    flex-direction: column;
    margin-bottom: 0.75rem;
    border-radius: var(--radius-box, 0.5rem);
    background-color: var(--color-base-200, oklch(20% 0 0));
    box-shadow:
        0 4px 15px -1px rgba(0, 0, 0, 0.25),
        0 2px 6px -1px rgba(0, 0, 0, 0.15);
    color: var(--color-base-content, white);
    transition: all 0.3s ease-in-out;
    overflow: hidden;
    position: relative;
    backdrop-filter: blur(10px);
}

.toast-content {
    display: flex;
    align-items: center;
    width: 100%;
    padding: 0.75rem 1rem;
}

.toast-success {
    border-left-color: var(--color-success, #4caf50);
}

.toast-error {
    border-left-color: var(--color-error, #f44336);
}

.toast-info {
    border-left-color: var(--color-info, #2196f3);
}

.toast-warning {
    border-left-color: var(--color-warning, #ff9800);
}

.toast-icon {
    margin-right: 0.75rem;
    flex-shrink: 0;
}

.toast-success .toast-icon {
    color: var(--color-success, #4caf50);
}

.toast-error .toast-icon {
    color: var(--color-error, #f44336);
}

.toast-info .toast-icon {
    color: var(--color-info, #2196f3);
}

.toast-warning .toast-icon {
    color: var(--color-warning, #ff9800);
}

.toast-message {
    flex-grow: 1;
    font-size: 0.875rem;
    line-height: 1.4;
    word-break: break-word;
}

.toast-close-btn {
    background: none;
    border: none;
    color: inherit;
    opacity: 0.7;
    cursor: pointer;
    padding: 0.25rem;
    margin-left: 0.75rem;
    transition: all 0.2s ease;
    border-radius: 50%;
}

.toast-close-btn:hover {
    opacity: 1;
    background-color: rgba(255, 255, 255, 0.1);
    transform: scale(1.1);
}

.toast-progress-container {
    height: 3px;
    width: 100%;
    background-color: rgba(255, 255, 255, 0.1);
}

.toast-progress {
    height: 100%;
    width: 100%;
    transform-origin: left;
    animation: progress-shrink linear forwards;
}

.toast-success .toast-progress {
    background-color: var(--color-success, #4caf50);
}

.toast-error .toast-progress {
    background-color: var(--color-error, #f44336);
}

.toast-info .toast-progress {
    background-color: var(--color-info, #2196f3);
}

.toast-warning .toast-progress {
    background-color: var(--color-warning, #ff9800);
}

.toast-progress-paused {
    animation-play-state: paused;
}

@keyframes progress-shrink {
    0% {
        transform: scaleX(1);
    }

    100% {
        transform: scaleX(0);
    }
}
</style>

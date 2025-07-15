import { ref } from 'vue';
import type { ToastMessage, ToastType } from '../types/toast';

const toasts = ref<ToastMessage[]>([]);
let nextId = 0;
let DUPLICATE_THRESHOLD = 2000;

export type ToastPosition = 'bottom-right' | 'bottom-left' | 'top-right' | 'top-left' | 'bottom-center' | 'top-center';

const toastPosition = ref<ToastPosition>('bottom-right');

function addToast(message: string, type: ToastType, duration: number = 5000) {
    const now = Date.now();
    const existingToast = toasts.value.find(toast =>
        toast.message === message &&
        toast.type === type &&
        toast.startTime &&
        (now - toast.startTime) < DUPLICATE_THRESHOLD
    );

    if (existingToast) {
        existingToast.count = (existingToast.count || 1) + 1;

        if (existingToast.timeoutId) {
            clearTimeout(existingToast.timeoutId);
        }

        existingToast.remainingDuration = duration;
        existingToast.startTime = now;

        if (duration > 0) {
            startToastTimer(existingToast);
        }

        return;
    }

    const id = nextId++;
    const toast: ToastMessage = {
        id,
        message,
        type,
        duration,
        remainingDuration: duration,
        count: 1
    };

    toasts.value.push(toast);

    if (duration > 0) {
        startToastTimer(toast);
    }
}

function startToastTimer(toast: ToastMessage) {
    if (!toast.duration) return;

    toast.startTime = Date.now();
    toast.timeoutId = setTimeout(() => {
        removeToast(toast.id);
    }, toast.remainingDuration) as unknown as number;
}

function removeToast(id: number) {
    const toast = toasts.value.find(t => t.id === id);
    if (toast?.timeoutId) {
        clearTimeout(toast.timeoutId);
    }
    toasts.value = toasts.value.filter((toast) => toast.id !== id);
}

function pauseToast(id: number) {
    const toast = toasts.value.find(t => t.id === id);
    if (toast?.timeoutId && toast.startTime) {
        clearTimeout(toast.timeoutId);
        const elapsed = Date.now() - toast.startTime;
        toast.remainingDuration = Math.max(0, (toast.remainingDuration || 0) - elapsed);
        toast.timeoutId = undefined;
    }
}

function resumeToast(id: number) {
    const toast = toasts.value.find(t => t.id === id);
    if (toast && toast.remainingDuration && toast.remainingDuration > 0) {
        startToastTimer(toast);
    }
}

function getDisplayMessage(toast: ToastMessage): string {
    if (!toast.count || toast.count <= 1) {
        return toast.message;
    }
    return `${toast.message} (x${toast.count})`;
}

function setDuplicateThreshold(threshold: number) {
    DUPLICATE_THRESHOLD = threshold;
}

function setToastPosition(position: ToastPosition) {
    toastPosition.value = position;
    localStorage.setItem('toastPosition', position);
}

function getToastPosition(): ToastPosition {
    const stored = localStorage.getItem('toastPosition');
    if (stored && ['bottom-right', 'bottom-left', 'top-right', 'top-left', 'bottom-center', 'top-center'].includes(stored)) {
        toastPosition.value = stored as ToastPosition;
        return stored as ToastPosition;
    }
    toastPosition.value = 'bottom-right';
    return 'bottom-right';
}

export function useToast() {
    return {
        toasts,
        addToast,
        removeToast,
        pauseToast,
        resumeToast,
        getDisplayMessage,
        setDuplicateThreshold,
        toastPosition,
        setToastPosition,
        getToastPosition,
    };
}

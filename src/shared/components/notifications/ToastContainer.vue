<template>
    <div class="toast-container" :class="positionClass">
        <transition-group name="toast-list" tag="div">
            <ToastNotification
                v-for="toast in toasts"
                :key="toast.id"
                :toast="toast"
                @close="removeToast"
            />
        </transition-group>
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import ToastNotification from "./ToastNotification.vue";
import { useToast } from "@shared/composables/useToast";

const { toasts, removeToast, toastPosition } = useToast();

const positionClass = computed(() => {
    return `toast-position-${toastPosition.value}`;
});
</script>

<style scoped>
.toast-container {
    position: fixed;
    z-index: 9999;
    width: 320px;
}

.toast-position-bottom-right {
    bottom: var(--toast-bottom-offset, 1rem);
    right: var(--toast-right-offset, 1rem);
}

.toast-position-bottom-left {
    bottom: var(--toast-bottom-offset, 1rem);
    left: var(--toast-left-offset, 1rem);
}

.toast-position-top-right {
    top: var(--toast-top-offset, 1rem);
    right: var(--toast-right-offset, 1rem);
}

.toast-position-top-left {
    top: var(--toast-top-offset, 1rem);
    left: var(--toast-left-offset, 1rem);
}

.toast-position-bottom-center {
    bottom: var(--toast-bottom-offset, 1rem);
    left: 50%;
    transform: translateX(-50%);
}

.toast-position-top-center {
    top: var(--toast-top-offset, 1rem);
    left: 50%;
    transform: translateX(-50%);
}

.toast-list-move,
.toast-list-enter-active,
.toast-list-leave-active {
    transition: all 0.5s ease;
}

.toast-list-enter-from {
    opacity: 0;
}

.toast-list-leave-to {
    opacity: 0;
}

.toast-position-bottom-right .toast-list-enter-from,
.toast-position-top-right .toast-list-enter-from {
    transform: translateX(100%);
}

.toast-position-bottom-right .toast-list-leave-to,
.toast-position-top-right .toast-list-leave-to {
    transform: translateX(100%);
}

.toast-position-bottom-left .toast-list-enter-from,
.toast-position-top-left .toast-list-enter-from {
    transform: translateX(-100%);
}

.toast-position-bottom-left .toast-list-leave-to,
.toast-position-top-left .toast-list-leave-to {
    transform: translateX(-100%);
}

.toast-position-bottom-center .toast-list-enter-from {
    transform: translateY(100%);
}

.toast-position-bottom-center .toast-list-leave-to {
    transform: translateY(100%);
}

.toast-position-top-center .toast-list-enter-from {
    transform: translateY(-100%);
}

.toast-position-top-center .toast-list-leave-to {
    transform: translateY(-100%);
}
</style>

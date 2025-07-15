<template>
    <transition name="modal-fade">
        <div v-if="isOpen" class="custom-modal-overlay" @click.self="closeModal">
            <div v-if="showContent" class="custom-modal-content" :class="[contentClass]">
                <div class="custom-modal-header">
                    <slot name="header">
                        <h2 class="font-bold text-lg text-primary-focus">
                            {{ title }}
                        </h2>
                    </slot>
                    <button @click="closeModal" class="custom-modal-close-button">
                        ×
                    </button>
                </div>
                <div class="custom-modal-body">
                    <slot name="body"></slot>
                </div>
                <div class="custom-modal-footer">
                    <slot name="footer"></slot>
                </div>
            </div>
        </div>
    </transition>
</template>

<script setup lang="ts">
import { computed, watch, onUnmounted, ref } from 'vue';

const props = defineProps<{
    modelValue: boolean;
    title?: string;
    contentClass?: string;
}>();

const emit = defineEmits(['update:modelValue', 'close']);

const showContent = ref(false);

const isOpen = computed({
    get: () => props.modelValue,
    set: (value) => emit('update:modelValue', value),
});

const closeModal = () => {
    showContent.value = false;
    setTimeout(() => {
        isOpen.value = false;
        emit('close');
    }, 200);
};

const handleEscape = (e: KeyboardEvent) => {
    if (e.key === 'Escape' && isOpen.value) {
        closeModal();
    }
};

watch(
    isOpen,
    (newVal) => {
        if (newVal) {
            document.body.style.overflow = 'hidden';
            document.addEventListener('keydown', handleEscape);
            setTimeout(() => {
                showContent.value = true;
            }, 150);
        } else {
            document.body.style.overflow = '';
            document.removeEventListener('keydown', handleEscape);
            showContent.value = false;
        }
    },
    { immediate: true }
);

onUnmounted(() => {
    document.removeEventListener('keydown', handleEscape);
    document.body.style.overflow = '';
});
</script>

<style scoped>
.custom-modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.75);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
    backdrop-filter: blur(0px);
    animation: overlay-enter 0.4s ease-out forwards;
    padding: 1rem;
}

@keyframes overlay-enter {
    0% {
        background-color: rgba(0, 0, 0, 0);
        backdrop-filter: blur(0px);
    }

    50% {
        background-color: rgba(0, 0, 0, 0.4);
        backdrop-filter: blur(1px);
    }

    100% {
        background-color: rgba(0, 0, 0, 0.75);
        backdrop-filter: blur(2px);
    }
}

@keyframes overlay-exit {
    0% {
        background-color: rgba(0, 0, 0, 0.75);
        backdrop-filter: blur(2px);
    }

    50% {
        background-color: rgba(0, 0, 0, 0.4);
        backdrop-filter: blur(1px);
    }

    100% {
        background-color: rgba(0, 0, 0, 0);
        backdrop-filter: blur(0px);
    }
}

.custom-modal-content {
    background-color: var(--color-base-200);
    padding: 1.5rem;
    border-radius: 0.5rem;
    box-shadow:
        0 10px 25px -5px rgba(0, 0, 0, 0.3),
        0 8px 10px -6px rgba(0, 0, 0, 0.2);
    width: 100%;
    max-width: min(28rem, calc(100vw - 2rem));
    max-height: calc(100vh - 2rem);
    position: relative;
    box-sizing: border-box;
    animation: slide-down 0.3s ease-out forwards;
    border: 1px solid rgba(255, 255, 255, 0.1);
    display: flex;
    flex-direction: column;
    overflow: hidden;
}

.custom-modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
    flex-shrink: 0;
}

.custom-modal-body {
    flex: 1;
    overflow-y: hidden;
    overflow-x: hidden;
    word-wrap: break-word;
    word-break: break-word;
}

.custom-modal-close-button {
    background: none;
    border: none;
    font-size: 1.5rem;
    cursor: pointer;
    color: var(--fallback-bc, oklch(var(--bc)));
    padding: 0;
    line-height: 1;
    opacity: 0.7;
    transition: opacity 0.2s ease;
    flex-shrink: 0;
}

.custom-modal-close-button:hover {
    opacity: 1;
}

.custom-modal-footer {
    display: flex;
    justify-content: flex-end;
    gap: 0.75rem;
    margin-top: 1rem;
    flex-shrink: 0;
    flex-wrap: wrap;
}

.modal-fade-enter-active {
    transition: opacity 0.4s ease;
}

.modal-fade-leave-active {
    transition: opacity 0.3s ease;
    animation: overlay-exit 0.3s ease-out forwards;
}

.modal-fade-enter-from {
    opacity: 0;
}

.modal-fade-leave-to {
    opacity: 0;
}

@keyframes slide-down {
    from {
        transform: translateY(-20px);
        opacity: 0;
    }

    to {
        transform: translateY(0);
        opacity: 1;
    }
}

@keyframes slide-up {
    from {
        transform: translateY(20px);
        opacity: 0;
    }

    to {
        transform: translateY(0);
        opacity: 1;
    }
}

.wide {
    max-width: min(90vw, 80rem);
    width: 90vw;
    max-height: 90vh;
}

.tall {
    max-height: 90vh;
    height: auto;
}

.compact {
    max-width: min(24rem, calc(100vw - 2rem));
}

.medium {
    max-width: min(36rem, calc(100vw - 2rem));
}

.large {
    max-width: min(48rem, calc(100vw - 2rem));
}

.full-mobile {
    @media (max-width: 768px) {
        max-width: calc(100vw - 4rem);
        max-height: calc(100vh - 1rem);
        padding: 1rem;
    }
}

@media (max-width: 768px) {
    .custom-modal-overlay {
        padding: 0.5rem;
    }

    .custom-modal-content {
        max-width: calc(100vw - 1rem);
        max-height: calc(100vh - 1rem);
        padding: 1rem;
    }

    .custom-modal-header {
        margin-bottom: 0.75rem;
    }

    .custom-modal-footer {
        flex-direction: column;
        gap: 0.5rem;
    }

    .custom-modal-footer>* {
        width: 100%;
    }
}

@media (max-width: 480px) {
    .custom-modal-overlay {
        padding: 0.25rem;
    }

    .custom-modal-content {
        max-width: calc(100vw - 0.5rem);
        max-height: calc(100vh - 0.5rem);
        padding: 0.75rem;
    }
}
</style>

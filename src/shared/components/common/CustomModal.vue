<template>
    <transition name="modal-fade">
        <div v-if="isOpen" class="modal-overlay" @click.self="closeModal">
            <div
                v-if="showContent"
                :class="[
                    'modal-content bg-base-200 rounded-lg shadow-2xl w-full flex flex-col max-h-[90vh] border border-base-content/5',
                    sizeClasses[size || 'md'],
                    contentClass,
                ]"
            >
                <div
                    class="flex justify-between items-center p-4 border-b border-base-content/10 shrink-0"
                >
                    <slot name="header">
                        <h2 class="font-bold text-lg text-primary">
                            {{ title }}
                        </h2>
                    </slot>
                    <button
                        @click="closeModal"
                        class="btn btn-ghost btn-sm btn-square text-lg opacity-70 hover:opacity-100 transition-opacity"
                    >
                        <span class="sr-only">Close</span>
                        ×
                    </button>
                </div>

                <div
                    class="flex-1 overflow-y-auto overflow-x-hidden min-h-0 p-4 custom-scrollbar"
                >
                    <slot name="body"></slot>
                </div>

                <div
                    v-if="$slots.footer"
                    class="p-4 border-t border-base-content/10 shrink-0"
                >
                    <slot name="footer"></slot>
                </div>
            </div>
        </div>
    </transition>
</template>

<script setup lang="ts">
import { computed, watch, onUnmounted, ref } from "vue";
import { lockScroll, unlockScroll } from "@shared/utils/scrollLock";

const props = defineProps<{
    modelValue: boolean;
    title?: string;
    contentClass?: string;
    size?: "sm" | "md" | "lg" | "xl" | "full";
}>();

const emit = defineEmits(["update:modelValue", "close"]);

const showContent = ref(false);

const sizeClasses = {
    sm: "max-w-md",
    md: "max-w-2xl",
    lg: "max-w-4xl",
    xl: "max-w-6xl",
    full: "max-w-full h-full m-4",
};

const isOpen = computed({
    get: () => props.modelValue,
    set: (value) => emit("update:modelValue", value),
});

const closeModal = () => {
    showContent.value = false;
    setTimeout(() => {
        isOpen.value = false;
        emit("close");
    }, 250);
};

const handleEscape = (e: KeyboardEvent) => {
    if (e.key === "Escape" && isOpen.value) {
        closeModal();
    }
};

watch(
    isOpen,
    (newVal) => {
        if (newVal) {
            lockScroll();
            document.addEventListener("keydown", handleEscape);
            setTimeout(() => {
                showContent.value = true;
            }, 50);
        } else {
            unlockScroll();
            document.removeEventListener("keydown", handleEscape);
            showContent.value = false;
        }
    },
    { immediate: true }
);

onUnmounted(() => {
    document.removeEventListener("keydown", handleEscape);
    if (isOpen.value) {
        unlockScroll();
    }
});
</script>

<style scoped>
.modal-overlay {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background-color: rgba(0, 0, 0, 0.7);
    display: flex;
    justify-content: center;
    align-items: center;
    z-index: 1000;
    backdrop-filter: blur(0px);
    animation: overlay-enter 0.6s cubic-bezier(0.16, 1, 0.3, 1) forwards;
    padding: 1rem;
    overflow: hidden;
    overscroll-behavior: contain;
}

@keyframes overlay-enter {
    0% {
        background-color: rgba(0, 0, 0, 0);
        backdrop-filter: blur(0px);
    }

    100% {
        background-color: rgba(0, 0, 0, 0.7);
        backdrop-filter: blur(1.5px);
    }
}

@keyframes overlay-exit {
    0% {
        background-color: rgba(0, 0, 0, 0.7);
        backdrop-filter: blur(1.5px);
    }

    100% {
        background-color: rgba(0, 0, 0, 0);
        backdrop-filter: blur(0px);
    }
}

.modal-content {
    position: relative;
    box-sizing: border-box;
    animation: content-enter 0.6s cubic-bezier(0.22, 1, 0.36, 1) forwards;
}

@keyframes content-enter {
    0% {
        opacity: 0;
        transform: scale(0.96) translateY(20px);
    }

    100% {
        opacity: 1;
        transform: scale(1) translateY(0);
    }
}

.modal-fade-enter-active {
    transition: opacity 0.5s ease;
}

.modal-fade-leave-active {
    transition: opacity 0.4s ease;
    animation: overlay-exit 0.4s ease-out forwards;
}

.modal-fade-enter-from {
    opacity: 0;
}

.modal-fade-leave-to {
    opacity: 0;
}
</style>

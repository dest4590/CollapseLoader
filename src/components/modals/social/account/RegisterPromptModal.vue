<template>
    <div v-if="modelValue" class="fixed inset-0 flex items-center justify-center z-[2000] slide-up backdrop-blur-sm">
        <div class="custom-modal-overlay" @click.self="closeModal">
            <div class="custom-modal-content">
                <div class="custom-modal-header">
                    <div></div>
                    <button @click="closeModal" class="custom-modal-close-button">
                        ×
                    </button>
                </div>
                <div class="custom-modal-body">
                    <div class="flex flex-col items-center mb-6">
                        <div class="p-3 bg-primary/20 rounded-full mb-3">
                            <UserRound class="w-7 h-7" />
                        </div>
                        <h3 class="text-xl font-bold mb-2">
                            {{ $t('modals.register_prompt.title') }}
                        </h3>
                    </div>

                    <p class="text-base-content/80 mb-4">
                        {{ $t('modals.register_prompt.description') }}
                    </p>

                    <ul class="list-disc list-inside space-y-2 mb-6 text-sm">
                        <li>
                            {{ $t('modals.register_prompt.benefits.sync') }}
                        </li>
                        <li>
                            {{
                                $t(
                                    'modals.register_prompt.benefits.notifications'
                                )
                            }}
                        </li>
                    </ul>

                    <div class="flex flex-col sm:flex-row gap-3 mt-4">
                        <button @click="proceedToRegister" class="btn btn-primary flex-1">
                            {{ $t('modals.register_prompt.register_now') }}
                        </button>
                        <button @click="cancel" class="btn btn-outline flex-1">
                            {{ $t('modals.register_prompt.maybe_later') }}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { UserRound } from 'lucide-vue-next';
import { computed, watch, onUnmounted } from 'vue';

const props = defineProps<{
    modelValue: boolean;
}>();

const emit = defineEmits(['update:modelValue', 'register', 'cancel']);

const isOpen = computed({
    get: () => props.modelValue,
    set: (value) => emit('update:modelValue', value),
});

const closeModal = () => {
    isOpen.value = false;
    emit('cancel');
};

const proceedToRegister = () => {
    isOpen.value = false;
    emit('register');
};

const cancel = () => {
    isOpen.value = false;
    emit('cancel');
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
        } else {
            document.body.style.overflow = '';
            document.removeEventListener('keydown', handleEscape);
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
    animation: blur 0.3s ease-out forwards;
    transition: 1s backdrop-filter;
}

@keyframes blur {
    from {
        backdrop-filter: blur(0px);
    }

    to {
        backdrop-filter: blur(5px);
    }
}

.custom-modal-content {
    background-color: var(--color-base-200, oklch(20% 0 0));
    padding: 1.5rem;
    border-radius: 0.5rem;
    box-shadow:
        0 10px 25px -5px rgba(0, 0, 0, 0.3),
        0 8px 10px -6px rgba(0, 0, 0, 0.2);
    width: 100%;
    max-width: 28rem;
    position: relative;
    box-sizing: border-box;
    animation: slide-down 0.3s ease-out forwards;
    border: 1px solid rgba(255, 255, 255, 0.1);
}

.custom-modal-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
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
}

.custom-modal-close-button:hover {
    opacity: 1;
}

@keyframes slide-down {
    from {
        opacity: 0;
        transform: translateY(-20px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}
</style>

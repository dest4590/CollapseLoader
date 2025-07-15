<template>
    <transition name="disclaimer-fade">
        <div v-if="showDevMenu"
            class="fixed inset-0 bg-black/80 z-[1400] p-9 flex items-center justify-center backdrop-blur-md">
            <transition name="disclaimer-scale">
                <div v-if="showDevMenu"
                    class="bg-base-200 rounded-xl shadow-2xl border border-base-300 w-full h-full flex flex-col">
                    <div class="flex flex-col items-center text-center pt-4 flex-grow">
                        <h2 class="text-2xl font-bold text-primary mb-4">
                            Developer Menu
                        </h2>
                        <button class="btn btn-primary mb-4" @click="resetFlags">
                            Reset Flags
                        </button>
                    </div>
                    <div class="p-4 border-t border-base-300 flex justify-center">
                        <button @click="closeDevMenu" class="btn btn-primary w-full">
                            Close Menu
                        </button>
                    </div>
                </div>
            </transition>
        </div>
    </transition>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { useToast } from '../../services/toastService';
import { useI18n } from 'vue-i18n';

const { addToast } = useToast();
const { t } = useI18n();

defineProps<{
    showDevMenu: boolean;
}>();

const emit = defineEmits(['close', 'update:registerPrompt']);

const closeDevMenu = () => {
    emit('close');
};

const resetFlags = async () => {
    await invoke('reset_flags');
    addToast(t('toast.dev.flags_reset'), 'success');
};
</script>

<style scoped>
.disclaimer-fade-enter-active,
.disclaimer-fade-leave-active {
    transition: opacity 0.3s ease;
}

.disclaimer-fade-enter-from,
.disclaimer-fade-leave-to {
    opacity: 0;
}

.disclaimer-scale-enter-active,
.disclaimer-scale-leave-active {
    transition:
        opacity 0.3s ease,
        transform 0.3s ease;
}

.disclaimer-scale-enter-from,
.disclaimer-scale-leave-to {
    opacity: 0;
    transform: scale(0.95) translateY(10px);
}

.disclaimer-scale-enter-to,
.disclaimer-scale-leave-from {
    opacity: 1;
    transform: scale(1) translateY(0);
}
</style>

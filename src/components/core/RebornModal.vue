<script setup lang="ts">
import { ref } from "vue";
import { openUrl } from "@tauri-apps/plugin-opener";

const props = defineProps<{ show: boolean }>();
const emit = defineEmits<{
    (event: "update:show", value: boolean): void;
}>();

const close = () => {
    emit("update:show", false);
};

const openGithub = async () => {
    try {
        await openUrl("https://github.com/W1xced-io");
    } catch (error) {
        console.error("Failed to open GitHub:", error);
    }
};
</script>

<template>
    <div
        v-if="props.show"
        class="fixed inset-0 z-[9999] flex items-center justify-center bg-black/60 backdrop-blur-md p-6"
        @click.self="close"
    >
        <div
            class="max-w-2xl w-full rounded-3xl border border-primary/20 bg-base-100 p-1 shadow-2xl overflow-hidden animate-reborn-appear"
        >
            <div class="bg-linear-to-br from-primary/10 via-base-100 to-base-100 p-8 rounded-[1.4rem]">
                <div class="flex flex-col items-center text-center gap-6">
                    <!-- Logo / Icon -->
                    <div class="relative">
                        <div class="absolute inset-0 bg-primary/20 blur-2xl rounded-full"></div>
                        <img src="../../assets/images/logo.svg" class="w-24 h-24 relative z-10 animate-pulse-slow" alt="Logo" />
                    </div>

                    <div class="space-y-2">
                        <h2 class="text-4xl font-black tracking-tighter uppercase italic">
                            CollapseLoader <span class="text-primary">Reborn</span>
                        </h2>
                        <div class="h-1 w-24 bg-primary mx-auto rounded-full"></div>
                    </div>

                    <p class="text-lg text-base-content/80 leading-relaxed max-w-md">
                        Проект официально перезапущен и теперь поддерживается 
                        <span class="text-primary font-bold">W1xced-io</span>. 
                        Мы продолжаем развивать лучший лоадер вместе с вами!
                    </p>

                    <div class="flex flex-col sm:flex-row gap-4 w-full mt-4">
                        <button 
                            @click="openGithub"
                            class="btn btn-outline btn-primary grow gap-2 group"
                        >
                            <svg xmlns="http://www.w3.org/2000/svg" width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="group-hover:rotate-12 transition-transform"><path d="M15 22v-4a4.8 4.8 0 0 0-1-3.5c3 0 6-2 6-5.5.08-1.25-.27-2.48-1-3.5.28-1.15.28-2.35 0-3.5 0 0-1 0-3 1.5-2.64-.5-5.36-.5-8 0C6 2 5 2 5 2c-.3 1.15-.3 2.35 0 3.5A5.403 5.403 0 0 0 4 9c0 3.5 3 5.5 6 5.5-.39.49-.68 1.05-.85 1.65-.17.6-.22 1.23-.15 1.85v4"></path><path d="M9 18c-4.51 2-5-2-7-2"></path></svg>
                            GitHub @W1xced-io
                        </button>
                        <button 
                            @click="close"
                            class="btn btn-primary grow px-12"
                        >
                            Поехали!
                        </button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
@keyframes reborn-appear {
    from {
        opacity: 0;
        transform: scale(0.9) translateY(20px);
    }
    to {
        opacity: 1;
        transform: scale(1) translateY(0);
    }
}

.animate-reborn-appear {
    animation: reborn-appear 0.5s cubic-bezier(0.34, 1.56, 0.64, 1) forwards;
}

.animate-pulse-slow {
    animation: pulse 3s cubic-bezier(0.4, 0, 0.6, 1) infinite;
}

@keyframes pulse {
    0%, 100% {
        opacity: 1;
        transform: scale(1);
    }
    50% {
        opacity: 0.8;
        transform: scale(1.05);
    }
}
</style>

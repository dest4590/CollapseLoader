<template>
    <div class="space-y-4 mb-6">
        <div class="flex justify-center py-4">
            <div class="bg-primary/10 p-4 rounded-2xl">
                <Zap class="w-12 h-12 text-primary animate-pulse" />
            </div>
        </div>

        <p class="text-base text-base-content/80 leading-relaxed text-center">
            {{ t("modals.spotlight_tip.description") }}
        </p>

        <div
            class="bg-base-200 rounded-xl p-4 flex items-center justify-center gap-3 border border-base-300"
        >
            <kbd class="kbd kbd-md">Ctrl</kbd>
            <span class="text-base-content/40 font-bold">+</span>
            <kbd class="kbd kbd-md">Space</kbd>
        </div>
    </div>

    <div class="flex justify-center">
        <button @click="handleClose" class="btn btn-primary px-8">
            {{ t("common.ok") }}
        </button>
    </div>
</template>

<script setup lang="ts">
import { Zap } from "lucide-vue-next";
import { useI18n } from "vue-i18n";

const emit = defineEmits(["close"]);
const { t } = useI18n();

const handleClose = () => {
    emit("close");
};
</script>

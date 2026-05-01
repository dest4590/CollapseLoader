<template>
    <div class="flex items-center gap-2">
        <div class="flex gap-2 h-2 rounded-full flex-1">
            <div
                v-for="color in colors"
                :key="color"
                class="flex-1 h-full rounded-full shadow-sm"
                :style="{ backgroundColor: color }"
            ></div>
        </div>
        <div v-if="hasBackground" class="flex-none">
            <Image class="w-3 h-3 text-primary" />
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import type { MarketplacePreset } from "@features/presets/types";
import { Image } from "lucide-vue-next";

const props = defineProps<{
    preset: MarketplacePreset;
}>();

const colors = computed(() => {
    const theme = props.preset.theme ?? props.preset.preset_data ?? {};
    const keys = ["primary", "secondary", "accent", "neutral", "base100"];
    return keys
        .map((k) => (theme as any)[k])
        .filter((c) => c && typeof c === "string" && c.startsWith("#"));
});

const hasBackground = computed(() => {
    const theme = props.preset.theme ?? props.preset.preset_data ?? {};
    return !!(theme as any).backgroundImage;
});
</script>

<template>
    <div class="space-y-5">
        <div
            v-if="theme.backgroundImage"
            class="w-full h-48 rounded-xl bg-cover bg-center border border-white/5 overflow-hidden"
            :style="{ backgroundImage: `url(${theme.backgroundImage})` }"
        ></div>

        <div>
            <p class="text-xs text-base-content/50 uppercase tracking-widest font-bold mb-2">
                {{ t("marketplace.tg_colors") }}
            </p>
            <div class="flex flex-wrap gap-2">
                <template v-for="[label, value] in colorEntries" :key="label">
                    <div
                        v-if="value"
                        class="flex items-center gap-1.5 px-2 py-1 rounded-lg bg-base-300/50 text-xs"
                    >
                        <div
                            class="w-3 h-3 rounded-full border border-white/10 shrink-0"
                            :style="{ backgroundColor: value }"
                        ></div>
                        <span class="text-base-content/60">{{ label }}</span>
                    </div>
                </template>
            </div>
        </div>

        <div v-if="theme.backgroundImage" class="space-y-1">
            <p class="text-xs text-base-content/50 uppercase tracking-widest font-bold mb-2">
                {{ t("customization.background_title") }}
            </p>
            <div class="flex flex-wrap gap-3 text-sm text-base-content/70">
                <span v-if="theme.backgroundBlur != null" class="flex items-center gap-1">
                    <Blend class="w-3.5 h-3.5" />
                    {{ t("customization.background_blur") }}: {{ theme.backgroundBlur }}px
                </span>
                <span v-if="theme.backgroundOpacity != null" class="flex items-center gap-1">
                    <SlidersHorizontal class="w-3.5 h-3.5" />
                    {{ t("customization.background_opacity") }}: {{ theme.backgroundOpacity }}%
                </span>
            </div>
        </div>

        <div v-if="theme.enableCustomCSS && theme.customCSS" class="flex items-center gap-2">
            <Code class="w-4 h-4 text-primary" />
            <span class="text-sm text-base-content/70">{{ t("marketplace.tg_has_css") }}</span>
        </div>

        <div class="flex gap-3 pt-2">
            <button class="btn btn-primary flex-1 gap-2" @click="$emit('apply')">
                <PaintBucket class="w-4 h-4" />
                {{ t("marketplace.apply") }}
            </button>
            <button class="btn btn-outline flex-1 gap-2" @click="$emit('save')">
                <Download class="w-4 h-4" />
                {{ t("theme.presets.save_current") }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import {
    PaintBucket,
    Download,
    Blend,
    SlidersHorizontal,
    Code,
} from "lucide-vue-next";
import type { MarketplacePreset } from "../../../types/presets";

const props = defineProps<{ preset: MarketplacePreset }>();
defineEmits(["apply", "save", "close"]);

const { t } = useI18n();

const theme = computed(() => props.preset.theme ?? {});

const colorEntries = computed(() => {
    const t = theme.value as Record<string, any>;
    return [
        ["Primary", t.primary],
        ["Secondary", t.secondary],
        ["Accent", t.accent],
        ["Neutral", t.neutral],
        ["Base 100", t.base100],
        ["Base 200", t.base200],
        ["Base 300", t.base300],
        ["Success", t.success],
        ["Warning", t.warning],
        ["Error", t.error],
    ].filter(([, v]) => v && typeof v === "string" && v.startsWith("#"));
});
</script>

<template>
    <div
        class="card bg-base-200 shadow-md border border-base-300 p-4 transition-all duration-300"
        :class="themeMode === 'schedule' ? 'lg:col-span-5' : 'lg:col-span-8'"
    >
        <div class="flex items-center justify-between mb-4">
            <h2 class="card-title text-sm flex items-center gap-2">
                <Palette class="w-4 h-4 text-primary" />
                {{ t("theme.color_accent") }}
            </h2>
            <button
                @click="$emit('reset')"
                class="btn btn-ghost btn-xs text-base-content/20 hover:text-primary"
                :title="t('theme.reset_button')"
            >
                <RotateCcw class="w-3 h-3" />
            </button>
        </div>

        <div class="flex flex-col gap-2">
            <div
                v-for="(row, rowIndex) in accentRows"
                :key="rowIndex"
                class="grid grid-cols-5 gap-1"
            >
                <button
                    v-for="accent in row"
                    :key="accent.color"
                    class="flex flex-col items-center justify-center gap-1.5 py-2 px-1 rounded-lg transition-colors"
                    :class="
                        primaryColor === accent.color
                            ? 'bg-primary/10'
                            : 'hover:bg-base-content/5'
                    "
                    @click="$emit('update:primaryColor', accent.color)"
                >
                    <div class="relative">
                        <div
                            class="w-7 h-7 rounded-full"
                            :style="{ backgroundColor: accent.color }"
                            :class="
                                primaryColor === accent.color
                                    ? 'ring-2 ring-primary ring-offset-2 ring-offset-base-200'
                                    : ''
                            "
                        ></div>
                        <div
                            v-if="primaryColor === accent.color"
                            class="absolute inset-0 flex items-center justify-center"
                        >
                            <Check
                                class="w-3.5 h-3.5 text-white drop-shadow-sm"
                                stroke-width="3"
                            />
                        </div>
                    </div>
                    <span
                        class="text-[10px] font-medium uppercase tracking-wide"
                        :class="
                            primaryColor === accent.color
                                ? 'text-primary'
                                : 'text-base-content/40'
                        "
                    >
                        {{ t(accent.name).split(".").pop() }}
                    </span>
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { Palette, RotateCcw, Check } from "@lucide/vue";

const { t } = useI18n();

const props = defineProps<{
    primaryColor: string | null;
    themeMode: "dark" | "light" | "schedule";
}>();

defineEmits<{
    "update:primaryColor": [color: string];
    reset: [];
}>();

const accentColors = [
    { name: "theme.accent_colors.blue", color: "#3b82f6" },
    { name: "theme.accent_colors.red", color: "#ef4444" },
    { name: "theme.accent_colors.green", color: "#22c55e" },
    { name: "theme.accent_colors.purple", color: "#a855f7" },
    { name: "theme.accent_colors.orange", color: "#f97316" },
    { name: "theme.accent_colors.pink", color: "#ec4899" },
    { name: "theme.accent_colors.cyan", color: "#06b6d4" },
    { name: "theme.accent_colors.yellow", color: "#eab308" },
    { name: "theme.accent_colors.rose", color: "#f43f5e" },
    { name: "theme.accent_colors.lime", color: "#84cc16" },
];

const accentRows = computed(() => {
    const rows = [];
    for (let i = 0; i < accentColors.length; i += 5) {
        rows.push(accentColors.slice(i, i + 5));
    }
    return rows;
});
</script>

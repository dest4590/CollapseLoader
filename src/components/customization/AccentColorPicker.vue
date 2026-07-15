<template>
    <div
        class="card bg-base-200 shadow-md border border-base-300 p-4 overflow-hidden transition-all duration-300"
        :class="themeMode === 'schedule' ? 'lg:col-span-5' : 'lg:col-span-8'"
    >
        <div class="flex items-center justify-between mb-4">
            <h2 class="card-title text-sm flex items-center gap-2">
                <Palette class="w-4 h-4 text-primary" />
                {{ t("theme.color_accent") }}
            </h2>
            <button
                @click="$emit('reset')"
                class="btn btn-ghost btn-xs text-base-content/20 hover:text-primary transition-all duration-300 group/reset"
                :title="t('theme.reset_button')"
            >
                <RotateCcw
                    class="w-3 h-3 group-hover/reset:rotate-[-90deg] transition-transform duration-500"
                />
            </button>
        </div>

        <div class="flex flex-col gap-2">
            <div
                v-for="(row, rowIndex) in accentRows"
                :key="rowIndex"
                class="bg-base-300/50 p-1 rounded-xl border border-base-content/5 relative"
                :class="{ 'mt-2': rowIndex > 0 }"
            >
                <div
                    class="absolute top-1 bottom-1 transition-all duration-500 ease-in-out bg-primary/20 rounded-lg z-0 pointer-events-none"
                    :style="{
                        width: 'calc((100% - 0.5rem) / 5)',
                        left:
                            selectedAccentColumn >= 0
                                ? `calc(0.25rem + (100% - 0.5rem) / 5 * ${selectedAccentColumn})`
                                : '0.25rem',
                        opacity:
                            selectedAccentRow === rowIndex &&
                            selectedAccentColumn >= 0
                                ? 1
                                : 0,
                    }"
                ></div>
                <div class="grid grid-cols-5 gap-1 relative z-10">
                    <button
                        v-for="accent in row"
                        :key="accent.color"
                        class="flex flex-col items-center justify-center gap-2 py-2.5 px-1 w-full rounded-lg transition-all duration-300 group outline-none"
                        :class="
                            primaryColor === accent.color
                                ? ''
                                : 'hover:bg-base-content/5'
                        "
                        @click="$emit('update:primaryColor', accent.color)"
                    >
                        <div
                            class="relative flex items-center justify-center transition-all duration-500"
                            :class="
                                primaryColor === accent.color
                                    ? 'scale-110'
                                    : 'scale-100 group-hover:scale-105'
                            "
                        >
                            <div
                                class="w-7 h-7 rounded-full shadow-sm transition-all duration-500"
                                :style="{
                                    backgroundColor: accent.color,
                                }"
                                :class="
                                    primaryColor === accent.color
                                        ? 'ring-2 ring-white/60 ring-offset-1 ring-offset-transparent'
                                        : ''
                                "
                            ></div>
                            <transition name="check-pop">
                                <div
                                    v-if="primaryColor === accent.color"
                                    class="absolute inset-0 flex items-center justify-center"
                                >
                                    <div
                                        class="w-full h-full rounded-full bg-black/20 flex items-center justify-center backdrop-blur-[1px]"
                                    >
                                        <Check
                                            class="w-4 h-4 text-white drop-shadow-md"
                                            stroke-width="3"
                                        />
                                    </div>
                                </div>
                            </transition>
                        </div>
                        <span
                            class="text-[9px] font-bold uppercase tracking-wider text-center w-full transition-all duration-200"
                            :class="
                                primaryColor === accent.color
                                    ? 'text-primary'
                                    : 'text-base-content/30 group-hover:text-base-content/60'
                            "
                        >
                            {{ t(accent.name).split(".").pop() }}
                        </span>
                    </button>
                </div>
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

const selectedAccentIndex = computed(() =>
    accentColors.findIndex((a) => a.color === props.primaryColor)
);

const selectedAccentColumn = computed(() =>
    selectedAccentIndex.value >= 0 ? selectedAccentIndex.value % 5 : -1
);

const selectedAccentRow = computed(() =>
    selectedAccentIndex.value >= 0
        ? Math.floor(selectedAccentIndex.value / 5)
        : -1
);

const accentRows = computed(() => {
    const rows = [];
    for (let i = 0; i < accentColors.length; i += 5) {
        rows.push(accentColors.slice(i, i + 5));
    }
    return rows;
});
</script>

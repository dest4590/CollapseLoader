<template>
    <div class="space-y-3">
        <div class="flex items-center justify-between p-3 bg-base-300/60 rounded-lg mb-4">
            <div class="flex items-center gap-2 text-base-content/70 text-sm">
                <Timer class="w-4 h-4 text-primary" />
                <span>{{ t("account.playtime_total") }}</span>
            </div>
            <span class="font-bold text-primary">{{ formatPlaytime(totalMinutes) }}</span>
        </div>

        <div
            v-for="([clientName, stats], index) in entries"
            :key="clientName"
            class="flex items-center gap-3 p-3 bg-base-300/40 rounded-lg"
        >
            <span class="text-base-content/30 text-xs font-mono w-5 text-right shrink-0">
                {{ index + 1 }}
            </span>
            <Gamepad2 class="w-5 h-5 text-primary shrink-0" />
            <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between gap-2">
                    <span class="font-medium truncate">{{ clientName }}</span>
                    <span class="text-primary font-semibold shrink-0">{{ formatPlaytime(stats.playtimeMinutes) }}</span>
                </div>
                <div class="flex items-center gap-3 mt-1">
                    <div class="flex-1 bg-base-content/10 rounded-full h-1.5">
                        <div
                            class="bg-primary h-1.5 rounded-full transition-all"
                            :style="{ width: getPercent(stats.playtimeMinutes) + '%' }"
                        ></div>
                    </div>
                    <span class="text-xs text-base-content/50 shrink-0">
                        {{ t("account.playtime_launches", { count: stats.launches }) }}
                    </span>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import { Timer, Gamepad2 } from "lucide-vue-next";
import { localTrackerService } from "../../../services/localTrackerService";

const { t } = useI18n();

const stats = computed(() => localTrackerService.getStats());

const totalMinutes = computed(() => stats.value.totalPlaytimeMinutes);

const entries = computed(() => {
    return Object.entries(stats.value.clientStats).sort(
        ([, a], [, b]) => b.playtimeMinutes - a.playtimeMinutes
    );
});

const maxMinutes = computed(() =>
    Math.max(...entries.value.map(([, s]) => s.playtimeMinutes), 1)
);

const getPercent = (minutes: number) =>
    Math.round((minutes / maxMinutes.value) * 100);

const formatPlaytime = (minutes: number): string => {
    if (!minutes) return "0m";
    const h = Math.floor(minutes / 60);
    const m = minutes % 60;
    if (h > 0 && m > 0) return `${h}h ${m}m`;
    if (h > 0) return `${h}h`;
    return `${m}m`;
};
</script>

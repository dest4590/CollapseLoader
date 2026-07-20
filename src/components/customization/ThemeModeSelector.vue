<template>
    <div
        class="card bg-base-200 shadow-md border border-base-300 p-6 transition-all duration-300"
        :class="themeMode === 'schedule' ? 'lg:col-span-7' : 'lg:col-span-4'"
    >
        <h2 class="card-title flex items-center gap-2">
            <SunMoon class="w-5 h-5 text-primary" />
            {{ t("theme.select_theme") }}
        </h2>
        <p class="text-base-content/70 mb-4">
            {{ t("theme.description") }}
        </p>

        <div
            class="flex flex-col sm:flex-row gap-4 items-start relative overflow-hidden flex-nowrap"
        >
            <div
                class="flex flex-col gap-3 w-full shrink-0 transition-all duration-500 ease-[cubic-bezier(0.34,1.56,0.64,1)] z-10"
                :class="
                    themeMode === 'schedule'
                        ? 'sm:w-[calc(50%-0.5rem)]'
                        : 'sm:w-full'
                "
            >
                <button
                    @click="$emit('select-mode', 'dark')"
                    class="btn border flex items-center justify-between px-6 py-3 transition-all duration-300"
                    :class="{
                        'border-primary/50 bg-primary/10':
                            themeMode === 'dark',
                        'border-base-content/10': themeMode !== 'dark',
                    }"
                >
                    <div class="flex items-center gap-2">
                        <Moon
                            class="w-5 h-5 text-indigo-400 transition-transform duration-300"
                            :class="
                                themeMode === 'dark'
                                    ? 'scale-110'
                                    : 'scale-100'
                            "
                        />
                        <span class="font-medium">{{
                            t("theme.dark")
                        }}</span>
                    </div>
                    <transition name="badge-pop">
                        <div
                            v-if="themeMode === 'dark'"
                            class="badge badge-primary"
                        >
                            {{ t("theme.selected") }}
                        </div>
                    </transition>
                </button>

                <button
                    @click="$emit('select-mode', 'light')"
                    class="btn border flex items-center justify-between px-6 py-3 transition-all duration-300"
                    :class="{
                        'border-primary/50 bg-primary/10':
                            themeMode === 'light',
                        'border-base-content/10': themeMode !== 'light',
                    }"
                >
                    <div class="flex items-center gap-2">
                        <Sun
                            class="w-5 h-5 text-amber-400 transition-transform duration-300"
                            :class="
                                themeMode === 'light'
                                    ? 'scale-110'
                                    : 'scale-100'
                            "
                        />
                        <span class="font-medium">{{
                            t("theme.light")
                        }}</span>
                    </div>
                    <transition name="badge-pop">
                        <div
                            v-if="themeMode === 'light'"
                            class="badge badge-primary"
                        >
                            {{ t("theme.selected") }}
                        </div>
                    </transition>
                </button>

                <button
                    @click="$emit('select-mode', 'system')"
                    class="btn border flex items-center justify-between px-6 py-3 transition-all duration-300"
                    :class="{
                        'border-primary/50 bg-primary/10':
                            themeMode === 'system',
                        'border-base-content/10': themeMode !== 'system',
                    }"
                >
                    <div class="flex items-center gap-2">
                        <Monitor
                            class="w-5 h-5 text-emerald-400 transition-transform duration-300"
                            :class="
                                themeMode === 'system'
                                    ? 'scale-110'
                                    : 'scale-100'
                            "
                        />
                        <span class="font-medium">{{
                            t("theme.system")
                        }}</span>
                    </div>
                    <transition name="badge-pop">
                        <div
                            v-if="themeMode === 'system'"
                            class="badge badge-primary"
                        >
                            {{ t("theme.selected") }}
                        </div>
                    </transition>
                </button>

                <button
                    @click="$emit('select-mode', 'schedule')"
                    class="btn border flex items-center justify-between px-6 py-3 transition-all duration-300"
                    :class="{
                        'border-primary/50 bg-primary/10':
                            themeMode === 'schedule',
                        'border-base-content/10':
                            themeMode !== 'schedule',
                    }"
                >
                    <div class="flex items-center gap-2">
                        <div class="relative w-5 h-5 shrink-0">
                            <Sun
                                class="absolute inset-0 w-5 h-5 text-amber-400 transition-all duration-500"
                                :class="
                                    themeMode === 'schedule'
                                        ? 'opacity-100 scale-100'
                                        : 'opacity-60 scale-90'
                                "
                            />
                            <Moon
                                class="absolute inset-0 w-3 h-3 text-indigo-400 transition-all duration-500"
                                :class="
                                    themeMode === 'schedule'
                                        ? 'opacity-100 translate-x-2.5 translate-y-2.5'
                                        : 'opacity-0 translate-x-1 translate-y-1'
                                "
                            />
                        </div>
                        <span class="font-medium">{{
                            t("theme.schedule.title")
                        }}</span>
                    </div>
                    <transition name="badge-pop">
                        <div
                            v-if="themeMode === 'schedule'"
                            class="badge badge-primary"
                        >
                            {{ t("theme.selected") }}
                        </div>
                    </transition>
                </button>
            </div>

            <transition name="schedule-slide">
                <div
                    v-if="themeMode === 'system'"
                    class="w-full sm:w-[calc(50%-0.5rem)] shrink-0 z-0 flex flex-col gap-3 border border-base-300 bg-base-200 rounded-lg p-4"
                >
                    <div
                        class="flex items-center gap-2 font-medium text-sm text-primary"
                    >
                        <Monitor class="w-4 h-4 shrink-0" />
                        <span>{{ t("theme.system") }}</span>
                    </div>

                    <p class="text-xs text-base-content/60">
                        {{ t("theme.system_description") }}
                    </p>

                    <div
                        class="flex items-center justify-between gap-2 p-2 mt-1 rounded-lg bg-base-100 border border-base-content/10"
                    >
                        <div class="flex items-center gap-2">
                            <div
                                class="w-2 h-2 rounded-full shrink-0 transition-colors duration-500"
                                :class="
                                    currentSystemTheme === 'light'
                                        ? 'bg-amber-400'
                                        : 'bg-indigo-400'
                                "
                            ></div>
                            <span class="text-xs text-base-content/60">
                                {{ t("theme.system_detected") }}
                            </span>
                        </div>
                        <span
                            class="text-xs font-bold uppercase text-primary"
                        >
                            {{ t(`theme.${currentSystemTheme}`) }}
                        </span>
                    </div>
                </div>
            </transition>

            <transition name="schedule-slide">
                <div
                    v-if="themeMode === 'schedule'"
                    class="w-full sm:w-[calc(50%-0.5rem)] shrink-0 z-0 flex flex-col gap-3 border border-base-300 bg-base-200 rounded-lg p-4"
                >
                    <div
                        class="flex items-center gap-2 font-medium text-sm text-primary"
                    >
                        <Clock class="w-4 h-4 shrink-0" />
                        <span>{{
                            t("theme.schedule.light_window")
                        }}</span>
                    </div>

                    <div class="grid grid-cols-2 gap-3">
                        <div class="flex flex-col gap-1">
                            <label
                                class="text-sm text-base-content/60"
                                >{{ t("theme.schedule.from") }}</label
                            >
                            <input
                                type="time"
                                class="input input-bordered w-full"
                                :value="scheduleLightStart"
                                @change="
                                    $emit(
                                        'update-schedule',
                                        'lightStart',
                                        ($event.target as HTMLInputElement).value
                                    )
                                "
                            />
                        </div>
                        <div class="flex flex-col gap-1">
                            <label
                                class="text-sm text-base-content/60"
                                >{{ t("theme.schedule.to") }}</label
                            >
                            <input
                                type="time"
                                class="input input-bordered w-full"
                                :value="scheduleLightEnd"
                                @change="
                                    $emit(
                                        'update-schedule',
                                        'lightEnd',
                                        ($event.target as HTMLInputElement).value
                                    )
                                "
                            />
                        </div>
                    </div>

                    <div
                        class="flex items-center justify-between gap-2 p-2 mt-1 rounded-lg bg-base-100 border border-base-content/10"
                    >
                        <div class="flex items-center gap-2">
                            <div
                                class="w-2 h-2 rounded-full shrink-0 transition-colors duration-500"
                                :class="
                                    schedulePreviewTheme === 'light'
                                        ? 'bg-amber-400'
                                        : 'bg-indigo-400'
                                "
                            ></div>
                            <span class="text-xs text-base-content/60">
                                {{ t("theme.schedule.now_active") }}
                            </span>
                        </div>
                        <span
                            class="text-xs font-bold uppercase text-primary"
                        >
                            {{ t(`theme.${schedulePreviewTheme}`) }}
                        </span>
                    </div>
                </div>
            </transition>
        </div>
    </div>
</template>

<script setup lang="ts">
import { SunMoon, Moon, Sun, Clock, Monitor } from "@lucide/vue";
import { useI18n } from "vue-i18n";

const { t } = useI18n();

defineProps<{
    themeMode: "dark" | "light" | "system" | "schedule";
    scheduleLightStart: string;
    scheduleLightEnd: string;
    schedulePreviewTheme: string;
    currentSystemTheme: "dark" | "light";
}>();

defineEmits<{
    "select-mode": [
        mode: "dark" | "light" | "system" | "schedule",
    ];
    "update-schedule": [field: "lightStart" | "lightEnd", value: string];
}>();
</script>

<template>
    <div
        class="card bg-base-200 shadow-md border border-base-300 p-6 transition-all duration-300 lg:col-span-12"
    >
        <h2 class="card-title flex items-center gap-2">
            <SunMoon class="w-5 h-5 text-primary" />
            {{ t("theme.select_theme") }}
        </h2>
        <p class="text-base-content/70 mb-4">
            {{ t("theme.description") }}
        </p>

        <div class="flex flex-col lg:flex-row gap-4 items-start">
            <div class="w-full lg:flex-1">
                <div class="grid grid-cols-2 sm:grid-cols-4 gap-3">
                    <button
                        type="button"
                        class="group flex flex-col gap-2 rounded-xl border p-2.5 text-left transition-all duration-200 outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
                        :class="
                            themeMode === 'light'
                                ? 'border-primary bg-primary/5 shadow-sm'
                                : 'border-base-300 bg-base-100/40 hover:border-base-content/20 hover:bg-base-100/70'
                        "
                        @click="$emit('select-mode', 'light')"
                    >
                        <div
                            data-theme="light"
                           
                            class="w-full aspect-[16/10] rounded-lg border border-base-300 overflow-hidden bg-base-100 shadow-sm"
                        >
                            <div class="h-full w-full flex" :class="frameClass">
                                <div :class="sidebarClass">
                                    <div :class="navIconActiveClass"></div>
                                    <div :class="navIconClass"></div>
                                    <div :class="navIconClass"></div>
                                    <div :class="spacerClass"></div>
                                    <div :class="navIconClass"></div>
                                    <div :class="navIconClass"></div>
                                </div>
                                <div class="flex-1 p-1.5 flex flex-col gap-1.5 min-w-0 bg-base-100">
                                    <div class="flex items-center gap-1">
                                        <div class="h-4 flex-1 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-4 w-4 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-4 w-4 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-4 w-4 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-4 w-4 rounded border border-base-300 bg-base-200/70"></div>
                                    </div>
                                    <div class="flex-1 grid grid-cols-3 gap-1 min-h-0">
                                        <div
                                            v-for="i in 6"
                                            :key="'l' + i"
                                            class="rounded-md border border-base-content/10 bg-base-300 shadow-sm p-1 flex flex-col"
                                        >
                                            <div class="h-1 rounded bg-base-content/30" :class="i % 3 === 1 ? 'w-3/4' : i % 3 === 2 ? 'w-2/3' : 'w-4/5'"></div>
                                            <div class="mt-0.5 flex items-center gap-0.5">
                                                <div class="w-1 h-1 rounded-full bg-base-content/20"></div>
                                                <div class="h-0.5 rounded bg-base-content/20" :class="i % 2 ? 'w-4' : 'w-3'"></div>
                                            </div>
                                            <div class="mt-auto flex justify-end pt-0.5">
                                                <div
                                                    class="h-2.5 w-7 rounded"
                                                    :class="i === 4 ? 'bg-base-300 border border-base-content/10' : 'bg-primary'"
                                                ></div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="flex items-center justify-center gap-1.5">
                            <Sun class="w-4 h-4 text-warning" />
                            <span class="text-sm font-medium">{{ t("theme.light") }}</span>
                        </div>
                    </button>

                    <button
                        type="button"
                        class="group flex flex-col gap-2 rounded-xl border p-2.5 text-left transition-all duration-200 outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
                        :class="
                            themeMode === 'dark'
                                ? 'border-primary bg-primary/5 shadow-sm'
                                : 'border-base-300 bg-base-100/40 hover:border-base-content/20 hover:bg-base-100/70'
                        "
                        @click="$emit('select-mode', 'dark')"
                    >
                        <div
                            data-theme="dark"
                           
                            class="w-full aspect-[16/10] rounded-lg border border-base-300 overflow-hidden bg-base-100 shadow-sm"
                        >
                            <div class="h-full w-full flex" :class="frameClass">
                                <div :class="sidebarClass">
                                    <div :class="navIconActiveClass"></div>
                                    <div :class="navIconClass"></div>
                                    <div :class="navIconClass"></div>
                                    <div :class="spacerClass"></div>
                                    <div :class="navIconClass"></div>
                                    <div :class="navIconClass"></div>
                                </div>
                                <div class="flex-1 p-1.5 flex flex-col gap-1.5 min-w-0 bg-base-100">
                                    <div class="flex items-center gap-1">
                                        <div class="h-4 flex-1 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-4 w-4 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-4 w-4 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-4 w-4 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-4 w-4 rounded border border-base-300 bg-base-200/70"></div>
                                    </div>
                                    <div class="flex-1 grid grid-cols-3 gap-1 min-h-0">
                                        <div
                                            v-for="i in 6"
                                            :key="'d' + i"
                                            class="rounded-md border border-base-content/10 bg-base-300 shadow-sm p-1 flex flex-col"
                                        >
                                            <div class="h-1 rounded bg-base-content/30" :class="i % 3 === 1 ? 'w-3/4' : i % 3 === 2 ? 'w-2/3' : 'w-4/5'"></div>
                                            <div class="mt-0.5 flex items-center gap-0.5">
                                                <div class="w-1 h-1 rounded-full bg-base-content/20"></div>
                                                <div class="h-0.5 rounded bg-base-content/20" :class="i % 2 ? 'w-4' : 'w-3'"></div>
                                            </div>
                                            <div class="mt-auto flex justify-end pt-0.5">
                                                <div
                                                    class="h-2.5 w-7 rounded"
                                                    :class="i === 4 ? 'bg-base-300 border border-base-content/10' : 'bg-primary'"
                                                ></div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="flex items-center justify-center gap-1.5">
                            <Moon class="w-4 h-4 text-info" />
                            <span class="text-sm font-medium">{{ t("theme.dark") }}</span>
                        </div>
                    </button>

                    <button
                        type="button"
                        class="group flex flex-col gap-2 rounded-xl border p-2.5 text-left transition-all duration-200 outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
                        :class="
                            themeMode === 'system'
                                ? 'border-primary bg-primary/5 shadow-sm'
                                : 'border-base-300 bg-base-100/40 hover:border-base-content/20 hover:bg-base-100/70'
                        "
                        @click="$emit('select-mode', 'system')"
                    >
                        <div class="w-full aspect-[16/10] rounded-lg border border-base-300 overflow-hidden bg-base-100 shadow-sm flex">
                            <div data-theme="light" class="w-1/2 h-full flex" :class="frameClass">
                                <div :class="sidebarClassMini">
                                    <div :class="navIconActiveClassMini"></div>
                                    <div :class="navIconClassMini"></div>
                                    <div :class="navIconClassMini"></div>
                                    <div :class="spacerClass"></div>
                                    <div :class="navIconClassMini"></div>
                                    <div :class="navIconClassMini"></div>
                                </div>
                                <div class="flex-1 p-1 flex flex-col gap-1 min-w-0 bg-base-100">
                                    <div class="flex gap-0.5">
                                        <div class="h-3 flex-1 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-3 w-3 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-3 w-3 rounded border border-base-300 bg-base-200/70"></div>
                                    </div>
                                    <div class="flex-1 grid grid-cols-2 gap-0.5 min-h-0">
                                        <div
                                            v-for="i in 4"
                                            :key="'sl' + i"
                                            class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col"
                                        >
                                            <div class="h-0.5 w-3/4 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div
                                                    class="h-2 w-5 rounded"
                                                    :class="i === 4 ? 'bg-base-300 border border-base-content/10' : 'bg-primary'"
                                                ></div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div data-theme="dark" class="w-1/2 h-full flex border-l border-base-300" :class="frameClass">
                                <div :class="sidebarClassMini">
                                    <div :class="navIconActiveClassMini"></div>
                                    <div :class="navIconClassMini"></div>
                                    <div :class="navIconClassMini"></div>
                                    <div :class="spacerClass"></div>
                                    <div :class="navIconClassMini"></div>
                                    <div :class="navIconClassMini"></div>
                                </div>
                                <div class="flex-1 p-1 flex flex-col gap-1 min-w-0 bg-base-100">
                                    <div class="flex gap-0.5">
                                        <div class="h-3 flex-1 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-3 w-3 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-3 w-3 rounded border border-base-300 bg-base-200/70"></div>
                                    </div>
                                    <div class="flex-1 grid grid-cols-2 gap-0.5 min-h-0">
                                        <div
                                            v-for="i in 4"
                                            :key="'sd' + i"
                                            class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col"
                                        >
                                            <div class="h-0.5 w-3/4 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div
                                                    class="h-2 w-5 rounded"
                                                    :class="i === 4 ? 'bg-base-300 border border-base-content/10' : 'bg-primary'"
                                                ></div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                        <div class="flex items-center justify-center gap-1.5">
                            <Monitor class="w-4 h-4 text-success" />
                            <span class="text-sm font-medium">{{ t("theme.system") }}</span>
                        </div>
                    </button>

                    <button
                        type="button"
                        class="group flex flex-col gap-2 rounded-xl border p-2.5 text-left transition-all duration-200 outline-none focus-visible:ring-2 focus-visible:ring-primary/50"
                        :class="
                            themeMode === 'schedule'
                                ? 'border-primary bg-primary/5 shadow-sm'
                                : 'border-base-300 bg-base-100/40 hover:border-base-content/20 hover:bg-base-100/70'
                        "
                        @click="$emit('select-mode', 'schedule')"
                    >
                        <div class="w-full aspect-[16/10] rounded-lg border border-base-300 overflow-hidden bg-base-100 shadow-sm flex relative">
                            <div data-theme="light" class="w-1/2 h-full flex" :class="frameClass">
                                <div :class="sidebarClassMini">
                                    <div :class="navIconActiveClassMini"></div>
                                    <div :class="navIconClassMini"></div>
                                    <div :class="navIconClassMini"></div>
                                    <div :class="spacerClass"></div>
                                    <div :class="navIconClassMini"></div>
                                    <div :class="navIconClassMini"></div>
                                </div>
                                <div class="flex-1 p-1 flex flex-col gap-1 min-w-0 bg-base-100">
                                    <div class="flex gap-0.5">
                                        <div class="h-3 flex-1 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-3 w-3 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-3 w-3 rounded border border-base-300 bg-base-200/70"></div>
                                    </div>
                                    <div class="flex-1 grid grid-cols-2 gap-0.5 min-h-0">
                                        <div
                                            v-for="i in 4"
                                            :key="'cl' + i"
                                            class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col"
                                        >
                                            <div class="h-0.5 w-3/4 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div class="h-2 w-5 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div data-theme="dark" class="w-1/2 h-full flex border-l border-base-300" :class="frameClass">
                                <div :class="sidebarClassMini">
                                    <div :class="navIconActiveClassMini"></div>
                                    <div :class="navIconClassMini"></div>
                                    <div :class="navIconClassMini"></div>
                                    <div :class="spacerClass"></div>
                                    <div :class="navIconClassMini"></div>
                                    <div :class="navIconClassMini"></div>
                                </div>
                                <div class="flex-1 p-1 flex flex-col gap-1 min-w-0 bg-base-100">
                                    <div class="flex gap-0.5">
                                        <div class="h-3 flex-1 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-3 w-3 rounded border border-base-300 bg-base-200/70"></div>
                                        <div class="h-3 w-3 rounded border border-base-300 bg-base-200/70"></div>
                                    </div>
                                    <div class="flex-1 grid grid-cols-2 gap-0.5 min-h-0">
                                        <div
                                            v-for="i in 4"
                                            :key="'cd' + i"
                                            class="rounded border border-base-content/10 bg-base-300 p-0.5 flex flex-col"
                                        >
                                            <div class="h-0.5 w-3/4 rounded bg-base-content/25"></div>
                                            <div class="mt-auto flex justify-end">
                                                <div class="h-2 w-5 rounded bg-primary"></div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                            <div class="absolute inset-0 flex items-center justify-center pointer-events-none">
                                <div class="rounded-full bg-base-100 border border-base-300 p-1 shadow-sm">
                                    <Clock class="w-3.5 h-3.5 text-primary" />
                                </div>
                            </div>
                        </div>
                        <div class="flex items-center justify-center gap-1.5">
                            <Clock class="w-4 h-4 text-primary" />
                            <span class="text-sm font-medium">{{ t("theme.schedule.title") }}</span>
                        </div>
                    </button>
                </div>
            </div>

            <transition name="schedule-slide">
                <div
                    v-if="themeMode === 'system'"
                    class="w-full lg:w-80 shrink-0 flex flex-col gap-3 border border-base-300 bg-base-100/50 rounded-xl p-4"
                >
                    <div
                        class="flex items-center gap-2 font-medium text-sm text-base-content"
                    >
                        <Monitor class="w-4 h-4 shrink-0 text-primary" />
                        <span>{{ t("theme.system") }}</span>
                    </div>

                    <p class="text-xs text-base-content/60">
                        {{ t("theme.system_description") }}
                    </p>

                    <div
                        class="flex items-center justify-between gap-2 p-2 mt-1 rounded-lg bg-base-200 border border-base-300"
                    >
                        <div class="flex items-center gap-2">
                            <div
                                class="w-2 h-2 rounded-full shrink-0 transition-colors duration-500"
                                :class="
                                    currentSystemTheme === 'light'
                                        ? 'bg-warning'
                                        : 'bg-info'
                                "
                            ></div>
                            <span class="text-xs text-base-content/60">
                                {{ t("theme.system_detected") }}
                            </span>
                        </div>
                        <span class="text-xs font-semibold text-base-content">
                            {{ t(`theme.${currentSystemTheme}`) }}
                        </span>
                    </div>
                </div>
            </transition>

            <transition name="schedule-slide">
                <div
                    v-if="themeMode === 'schedule'"
                    class="w-full lg:w-80 shrink-0 flex flex-col gap-3 border border-base-300 bg-base-100/50 rounded-xl p-4"
                >
                    <div
                        class="flex items-center gap-2 font-medium text-sm text-base-content"
                    >
                        <Clock class="w-4 h-4 shrink-0 text-primary" />
                        <span>{{ t("theme.schedule.light_window") }}</span>
                    </div>

                    <div class="grid grid-cols-2 gap-3">
                        <div class="flex flex-col gap-1">
                            <label class="text-sm text-base-content/60">{{
                                t("theme.schedule.from")
                            }}</label>
                            <input
                                type="time"
                                class="input input-bordered input-sm w-full"
                                :value="scheduleLightStart"
                                @change="
                                    $emit(
                                        'update-schedule',
                                        'lightStart',
                                        ($event.target as HTMLInputElement)
                                            .value
                                    )
                                "
                            />
                        </div>
                        <div class="flex flex-col gap-1">
                            <label class="text-sm text-base-content/60">{{
                                t("theme.schedule.to")
                            }}</label>
                            <input
                                type="time"
                                class="input input-bordered input-sm w-full"
                                :value="scheduleLightEnd"
                                @change="
                                    $emit(
                                        'update-schedule',
                                        'lightEnd',
                                        ($event.target as HTMLInputElement)
                                            .value
                                    )
                                "
                            />
                        </div>
                    </div>

                    <div
                        class="flex items-center justify-between gap-2 p-2 mt-1 rounded-lg bg-base-200 border border-base-300"
                    >
                        <div class="flex items-center gap-2">
                            <div
                                class="w-2 h-2 rounded-full shrink-0 transition-colors duration-500"
                                :class="
                                    schedulePreviewTheme === 'light'
                                        ? 'bg-warning'
                                        : 'bg-info'
                                "
                            ></div>
                            <span class="text-xs text-base-content/60">
                                {{ t("theme.schedule.now_active") }}
                            </span>
                        </div>
                        <span class="text-xs font-semibold text-base-content">
                            {{ t(`theme.${schedulePreviewTheme}`) }}
                        </span>
                    </div>
                </div>
            </transition>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref } from "vue";
import { SunMoon, Moon, Sun, Clock, Monitor } from "@lucide/vue";
import { useI18n } from "vue-i18n";

type SidebarPosition = "left" | "right" | "top" | "bottom";

const { t } = useI18n();

defineProps<{
    themeMode: "dark" | "light" | "system" | "schedule";
    scheduleLightStart: string;
    scheduleLightEnd: string;
    schedulePreviewTheme: string;
    currentSystemTheme: "dark" | "light";
}>();

defineEmits<{
    "select-mode": [mode: "dark" | "light" | "system" | "schedule"];
    "update-schedule": [field: "lightStart" | "lightEnd", value: string];
}>();

const resolveSidebarPosition = (): SidebarPosition => {
    const v = localStorage.getItem("sidebarPosition") as SidebarPosition | null;
    return v === "right" || v === "top" || v === "bottom" ? v : "left";
};

const sidebarPosition = ref<SidebarPosition>(resolveSidebarPosition());

const syncSidebarPosition = () => {
    sidebarPosition.value = resolveSidebarPosition();
};

const handleSidebarPositionChanged = (event: Event) => {
    const detail = (event as CustomEvent<SidebarPosition>).detail;
    if (
        detail === "left" ||
        detail === "right" ||
        detail === "top" ||
        detail === "bottom"
    ) {
        sidebarPosition.value = detail;
    } else {
        syncSidebarPosition();
    }
};

const handleStorage = (event: StorageEvent) => {
    if (event.key === "sidebarPosition") {
        syncSidebarPosition();
    }
};

onMounted(() => {
    syncSidebarPosition();
    window.addEventListener(
        "sidebar-position-changed",
        handleSidebarPositionChanged as EventListener
    );
    window.addEventListener("storage", handleStorage);
});

onUnmounted(() => {
    window.removeEventListener(
        "sidebar-position-changed",
        handleSidebarPositionChanged as EventListener
    );
    window.removeEventListener("storage", handleStorage);
});

const frameClass = computed(() => {
    if (sidebarPosition.value === "right") return "flex-row-reverse";
    if (sidebarPosition.value === "top") return "flex-col";
    if (sidebarPosition.value === "bottom") return "flex-col-reverse";
    return "flex-row";
});

const isVertical = computed(
    () => sidebarPosition.value === "left" || sidebarPosition.value === "right"
);

const sidebarClass = computed(() => {
    if (isVertical.value) {
        const border =
            sidebarPosition.value === "left"
                ? "border-r border-base-content/10"
                : "border-l border-base-content/10";
        return `w-[18%] h-full ${border} bg-base-300 flex flex-col items-center py-1.5 gap-1.5 shrink-0`;
    }
    const border =
        sidebarPosition.value === "top"
            ? "border-b border-base-content/10"
            : "border-t border-base-content/10";
    return `h-[22%] w-full ${border} bg-base-300 flex flex-row items-center px-1.5 gap-1.5 shrink-0`;
});

const sidebarClassMini = computed(() => {
    if (isVertical.value) {
        const border =
            sidebarPosition.value === "left"
                ? "border-r border-base-content/10"
                : "border-l border-base-content/10";
        return `w-[22%] h-full ${border} bg-base-300 flex flex-col items-center py-1 gap-1 shrink-0`;
    }
    const border =
        sidebarPosition.value === "top"
            ? "border-b border-base-content/10"
            : "border-t border-base-content/10";
    return `h-[24%] w-full ${border} bg-base-300 flex flex-row items-center px-1 gap-1 shrink-0`;
});

const navIconActiveClass =
    "w-4 h-4 rounded-md bg-primary text-primary-content shadow-sm shrink-0";
const navIconClass = "w-4 h-4 rounded-md bg-base-content/10 shrink-0";
const navIconActiveClassMini = "w-3 h-3 rounded-sm bg-primary shrink-0";
const navIconClassMini = "w-3 h-3 rounded-sm bg-base-content/10 shrink-0";
const spacerClass = computed(() => (isVertical.value ? "mt-auto" : "ml-auto"));
</script>

<style scoped>
.schedule-slide-enter-active,
.schedule-slide-leave-active {
    transition: all 0.25s ease;
}

.schedule-slide-enter-from,
.schedule-slide-leave-to {
    opacity: 0;
    transform: translateX(12px);
}
</style>

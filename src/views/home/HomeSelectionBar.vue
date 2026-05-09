<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { Download, RefreshCcw, StopCircle, Trash2, X } from "lucide-vue-next";

const { t } = useI18n();

defineProps<{
    selectedClientsSize: number;
    canStopSelected: boolean;
    canDownloadSelected: boolean;
    canReinstallSelected: boolean;
    canDeleteSelected: boolean;
}>();

const emit = defineEmits<{
    "stop-multiple": [];
    "download-multiple": [];
    "reinstall-multiple": [];
    "delete-multiple": [];
    "clear-selection": [];
}>();
</script>

<template>
    <transition name="slide-up-bottom">
        <div
            v-if="selectedClientsSize > 0"
            class="fixed left-1/2 transform -translate-x-1/2 w-auto max-w-[calc(100%-2rem)] bg-neutral text-neutral-content px-4 py-3 rounded-lg shadow-xl z-30 flex items-center gap-3 sm:gap-4"
            :style="{
                bottom: `calc(1rem + var(--sidebar-bottom-height, 0px))`,
            }"
        >
            <span class="font-medium text-xs sm:text-sm whitespace-nowrap">
                {{
                    t("home.selected_clients", {
                        count: selectedClientsSize,
                    })
                }}
            </span>

            <div class="flex items-center gap-1 sm:gap-2">
                <transition name="button-fade" mode="out-in">
                    <button
                        v-if="canStopSelected"
                        @click="emit('stop-multiple')"
                        :title="t('home.stop_selected')"
                        class="btn btn-sm btn-ghost hover:bg-neutral-focus p-2 aspect-square"
                    >
                        <StopCircle class="w-4 h-4 sm:w-5 sm:h-5" />
                    </button>
                </transition>

                <transition name="button-fade" mode="out-in">
                    <button
                        v-if="canDownloadSelected"
                        @click="emit('download-multiple')"
                        :title="t('home.download_selected')"
                        class="btn btn-sm btn-ghost hover:bg-neutral-focus p-2 aspect-square"
                    >
                        <Download class="w-4 h-4 sm:w-5 sm:h-5" />
                    </button>
                </transition>

                <transition name="button-fade" mode="out-in">
                    <button
                        v-if="canReinstallSelected"
                        @click="emit('reinstall-multiple')"
                        :title="t('home.reinstall_selected')"
                        class="btn btn-sm btn-ghost hover:bg-neutral-focus p-2 aspect-square"
                    >
                        <RefreshCcw class="w-4 h-4 sm:w-5 sm:h-5" />
                    </button>
                </transition>

                <transition name="button-fade" mode="out-in">
                    <button
                        v-if="canDeleteSelected"
                        @click="emit('delete-multiple')"
                        :title="t('home.delete_selected')"
                        class="btn btn-sm btn-ghost hover:bg-neutral-focus p-2 aspect-square"
                    >
                        <Trash2 class="w-4 h-4 sm:w-5 sm:h-5" />
                    </button>
                </transition>
            </div>

            <div
                v-if="
                    canStopSelected ||
                    canDownloadSelected ||
                    canReinstallSelected ||
                    canDeleteSelected
                "
                class="border-l border-neutral-content/30 h-5 sm:h-6 mx-1"
            ></div>

            <button
                @click="emit('clear-selection')"
                class="btn btn-sm btn-ghost hover:bg-neutral-focus p-2 aspect-square"
            >
                <X class="w-4 h-4 sm:w-5 sm:h-5" />
            </button>
        </div>
    </transition>
</template>

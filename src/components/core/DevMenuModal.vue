<template>
    <div
        v-if="showDevMenu"
        class="fixed inset-0 bg-black/80 z-1400 p-9 flex items-center justify-center backdrop-blur-md"
    >
        <div
            v-if="showDevMenu"
            class="bg-base-200 rounded-xl shadow-2xl border border-base-300 w-full h-full flex flex-col"
        >
            <div class="flex flex-col items-center text-center pt-4 grow">
                <h2 class="text-2xl font-bold text-primary mb-4">
                    Developer Menu
                </h2>

                <button class="btn btn-primary mb-4" @click="openNetworkDebug">
                    Network Debug
                </button>
                <button class="btn btn-primary mb-4" @click="resetFlags">
                    Reset Flags
                </button>
            </div>
            <div class="p-4 border-t border-base-300 flex justify-center">
                <button @click="closeDevMenu" class="btn btn-primary w-full">
                    Close Menu
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "@shared/composables/useToast";
import { useI18n } from "vue-i18n";
import { useRouter } from "../../services/router";

const { addToast } = useToast();
const { t } = useI18n();
const { push } = useRouter();

defineProps<{
    showDevMenu: boolean;
}>();

const emit = defineEmits(["close"]);

const closeDevMenu = () => {
    emit("close");
};

const openNetworkDebug = () => {
    push("network_debug");
    closeDevMenu();
};

const resetFlags = async () => {
    await invoke("reset_flags");
    addToast(t("toast.dev.flags_reset"), "success");
};
</script>

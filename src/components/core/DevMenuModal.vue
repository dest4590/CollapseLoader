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
                <button
                    class="btn btn-secondary mb-4"
                    :disabled="isFaking"
                    @click="startFakeDownloads"
                >
                    {{
                        isFaking
                            ? "Fake download in progress…"
                            : "Fake download progress"
                    }}
                </button>
                <p class="text-sm text-base-content/70 mt-2">
                    Simulate download + unzip progress without affecting network
                    activity.
                </p>
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
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { emit as emitAppEvent } from "@tauri-apps/api/event";
import { useToast } from "@shared/composables/useToast";
import { useI18n } from "vue-i18n";
import { useRouter } from "@router";

const { addToast } = useToast();
const { t } = useI18n();
const { push } = useRouter();
const isFaking = ref(false);

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

const delay = (ms: number) =>
    new Promise<void>((resolve) => setTimeout(resolve, ms));

const startFakeDownloads = async () => {
    if (isFaking.value) return;
    isFaking.value = true;

    const fakeFiles = [
        { filename: "fake-client-alpha.zip", size: 640 * 1024 * 1024 },
        { filename: "fake-client-beta.zip", size: 520 * 1024 * 1024 },
        { filename: "fake-client-gamma.zip", size: 760 * 1024 * 1024 },
        { filename: "fake-client-delta.zip", size: 880 * 1024 * 1024 },
    ];

    const simulateFakeDownload = async (
        filename: string,
        totalBytes: number,
        startDelay: number
    ) => {
        await delay(startDelay);
        await emitAppEvent("download-start", filename);

        const downloadSteps = 18;
        const downloadDuration = 1000 + Math.random() * 1500;
        const stepDelay = downloadDuration / downloadSteps;

        for (let i = 1; i <= downloadSteps; i++) {
            await delay(stepDelay + Math.random() * 120);
            const percentage = Math.min(
                100,
                Math.round((i / downloadSteps) * 100)
            );
            const downloaded = Math.round((percentage / 100) * totalBytes);
            await emitAppEvent("download-progress", {
                file: filename,
                percentage,
                downloaded,
                total: totalBytes,
                speed_bps: 3_500_000 + Math.round(Math.random() * 2_500_000),
            });
        }

        await delay(500);
        await emitAppEvent("download-complete", filename);
        await delay(400 + Math.random() * 300);
        await emitAppEvent("unzip-start", filename);

        const unzipSteps = 8;
        for (let i = 1; i <= unzipSteps; i++) {
            await delay(180);
            await emitAppEvent("unzip-progress", {
                file: filename,
                percentage: Math.min(100, Math.round((i / unzipSteps) * 100)),
                files_extracted: Math.round((i / unzipSteps) * 100),
                total_files: 100,
            });
        }

        await delay(300 + Math.random() * 200);
        await emitAppEvent("unzip-complete", filename);
    };

    await Promise.all(
        fakeFiles.map((item, index) =>
            simulateFakeDownload(item.filename, item.size, index * 1500)
        )
    );

    isFaking.value = false;
    addToast("Fake downloads finished", "success");
};
</script>

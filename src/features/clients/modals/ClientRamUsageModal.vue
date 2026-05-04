<template>
    <div class="space-y-5">
        <div class="rounded-2xl border border-base-content/10 bg-base-200/70 p-4">
            <div class="flex flex-wrap items-start justify-between gap-4">
                <div>
                    <p class="text-xs uppercase tracking-[0.25em] text-base-content/50">
                        {{ t("modals.client_ram_usage.subtitle") }}
                    </p>
                    <h3 class="mt-1 text-xl font-black text-base-content">
                        {{ clientName }}
                    </h3>
                    <p class="mt-1 text-sm text-base-content/60">
                        {{ t("modals.client_ram_usage.description") }}
                    </p>
                </div>

                <div
                    class="badge badge-lg gap-2"
                    :class="ramUsage?.is_running ? 'badge-success' : 'badge-warning'"
                >
                    <span class="h-2 w-2 rounded-full bg-current animate-pulse"></span>
                    {{
                        ramUsage?.is_running
                            ? t("modals.client_ram_usage.running")
                            : t("modals.client_ram_usage.not_running")
                    }}
                </div>
            </div>

            <div class="mt-4 grid gap-3 md:grid-cols-3">
                <div class="rounded-xl border border-base-content/10 bg-base-100/70 p-4">
                    <p class="text-xs uppercase tracking-[0.2em] text-base-content/40">
                        {{ t("modals.client_ram_usage.client_ram") }}
                    </p>
                    <p class="mt-2 text-3xl font-black text-primary">
                        {{ ramUsage ? clientRamService.formatMiB(ramUsage.total_memory_mib) : "--" }}
                    </p>
                    <p class="mt-1 text-sm text-base-content/60">
                        {{ ramUsage ? clientRamService.formatBytes(ramUsage.total_memory_bytes) : "Waiting for data" }}
                    </p>
                </div>

                <div class="rounded-xl border border-base-content/10 bg-base-100/70 p-4">
                    <p class="text-xs uppercase tracking-[0.2em] text-base-content/40">
                        {{ t("modals.client_ram_usage.system_share") }}
                    </p>
                    <p class="mt-2 text-3xl font-black text-secondary">
                        {{ ramUsage ? clientRamService.formatPercent(ramUsage.system_memory_percent) : "--" }}
                    </p>
                    <p class="mt-1 text-sm text-base-content/60">
                        of {{ ramUsage ? clientRamService.formatMiB(ramUsage.system_total_memory_mib) : "system memory" }}
                    </p>
                </div>

                <div class="rounded-xl border border-base-content/10 bg-base-100/70 p-4">
                    <p class="text-xs uppercase tracking-[0.2em] text-base-content/40">
                        {{ t("modals.client_ram_usage.processes") }}
                    </p>
                    <p class="mt-2 text-3xl font-black text-accent">
                        {{ ramUsage?.process_count ?? 0 }}
                    </p>
                    <p class="mt-1 text-sm text-base-content/60">
                        {{
                            ramUsage?.pids?.length
                                ? `PID ${ramUsage.pids.join(", ")}`
                                : t("modals.client_ram_usage.no_process_found")
                        }}
                    </p>
                </div>
            </div>

            <div class="mt-4 space-y-2">
                <div class="flex items-center justify-between text-xs uppercase tracking-[0.18em] text-base-content/40">
                    <span>{{ t("modals.client_ram_usage.usage_bar") }}</span>
                    <span>{{ ramUsage ? clientRamService.formatPercent(ramUsage.system_memory_percent) : "0%" }}</span>
                </div>
                <progress
                    class="progress progress-primary w-full"
                    :value="ramUsage?.system_memory_percent ?? 0"
                    max="100"
                ></progress>
            </div>
        </div>

        <div
            v-if="errorMessage"
            class="alert alert-warning"
        >
            <span>{{ errorMessage }}</span>
        </div>

        <div class="flex flex-wrap items-center justify-between gap-3 rounded-xl border border-base-content/10 bg-base-200/50 px-4 py-3">
            <div class="text-sm text-base-content/70">
                <p class="font-semibold text-base-content">
                    {{ t("modals.client_ram_usage.last_refresh") }}: {{ lastUpdatedLabel }}
                </p>
                <p>
                    {{ t("modals.client_ram_usage.backend_description") }}
                </p>
            </div>

            <div class="flex items-center gap-2">
                <button class="btn btn-ghost btn-sm" @click="refresh" :disabled="isLoading">
                    {{ t("modals.client_ram_usage.refresh") }}
                </button>
                <button class="btn btn-primary btn-sm" @click="emit('close')">
                    {{ t("modals.client_ram_usage.close") }}
                </button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { clientRamService } from "@services/clientRamService";
import type { ClientRamUsage } from "@shared/types/ui";

const props = defineProps<{
    clientId: number;
    clientName: string;
}>();

const emit = defineEmits(["close"]);
const { t } = useI18n();

const ramUsage = ref<ClientRamUsage | null>(null);
const isLoading = ref(false);
const errorMessage = ref("");
const lastUpdatedAt = ref<number | null>(null);
let refreshHandle: number | null = null;

const lastUpdatedLabel = computed(() => {
    if (!lastUpdatedAt.value) {
        return "Waiting for first sample";
    }

    return new Date(lastUpdatedAt.value).toLocaleTimeString();
});

const refresh = async () => {
    if (isLoading.value) {
        return;
    }

    isLoading.value = true;
    errorMessage.value = "";

    try {
        ramUsage.value = await clientRamService.getClientRamUsage(props.clientId);
        lastUpdatedAt.value = Date.now();
    } catch (error) {
        console.error("Failed to fetch client RAM usage:", error);
        errorMessage.value = "Unable to read RAM usage for this client right now.";
    } finally {
        isLoading.value = false;
    }
};

onMounted(async () => {
    await refresh();
    refreshHandle = window.setInterval(refresh, 2000);
});

onBeforeUnmount(() => {
    if (refreshHandle !== null) {
        clearInterval(refreshHandle);
        refreshHandle = null;
    }
});
</script>
<template>
    <div class="slide-up">
        <div
            class="card bg-base-200 border border-base-300 rounded-3xl p-5 mb-6 shadow-sm"
        >
            <div
                class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4"
            >
                <div>
                    <div class="flex items-center gap-3 mb-1">
                        <h1 class="text-2xl font-semibold text-primary-focus">
                            HF API
                        </h1>
                        <span
                            class="flex items-center gap-1.5 px-2.5 py-0.5 rounded-full border text-xs font-semibold bg-green-500/10 border-green-500/20 text-green-500"
                        >
                            <span
                                class="w-1.5 h-1.5 rounded-full bg-green-400 animate-pulse"
                            ></span>
                            {{ t("hfapi.live") }}
                        </span>
                    </div>
                    <p class="text-sm text-base-content/50">
                        {{ t("hfapi.subtitle") }}
                    </p>
                </div>
                <div class="flex items-center gap-2">
                    <button
                        @click="fetchData"
                        :disabled="loading"
                        class="btn btn-sm btn-ghost"
                        :title="t('hfapi.refresh')"
                    >
                        <RefreshCcw
                            class="w-4 h-4"
                            :class="{ 'animate-spin': loading }"
                        />
                    </button>
                </div>
            </div>
        </div>

        <div v-if="loading && !clients.length" class="flex justify-center items-center py-12">
            <div class="text-center space-y-3">
                <span
                    class="loading loading-spinner loading-md text-primary"
                ></span>
                <p class="text-base-content/70">{{ t("hfapi.loading") }}</p>
            </div>
        </div>

        <div
            v-else-if="error"
            class="bg-error/10 border border-error/20 rounded-xl p-6"
        >
            <div class="flex items-center gap-3 mb-3">
                <div class="text-error text-xl">⚠️</div>
                <h3 class="text-lg font-semibold text-error">
                    {{ t("hfapi.error") }}
                </h3>
            </div>
            <p class="text-base-content/70 mb-4">{{ error }}</p>
            <button
                @click="fetchData"
                class="btn btn-primary btn-sm"
                :disabled="loading"
            >
                {{ t("hfapi.retry") }}
            </button>
        </div>

        <template v-else>
            <div class="mb-8">
                <h2 class="text-lg font-semibold text-primary-focus mb-3">
                    {{ t("hfapi.latest_changes") }}
                </h2>
                <div v-if="latest.length" class="space-y-2">
                    <div
                        v-for="(client, idx) in latest"
                        :key="String(client.md5_hash) + idx"
                        class="card bg-base-200 border border-base-300 hover:border-primary/30 transition-all duration-200"
                    >
                        <div class="card-body p-4 flex-row items-center gap-4">
                            <span
                                class="shrink-0 px-2 py-0.5 rounded-md text-[10px] font-bold uppercase tracking-wider border"
                                :class="typeBadgeClass(client.client_type)"
                            >
                                {{ typeLabel(client.client_type) }}
                            </span>
                            <div class="flex-1 min-w-0">
                                <span
                                    class="font-semibold text-sm text-base-content"
                                    >{{ client.name }}</span
                                >
                                <span
                                    class="text-xs text-base-content/40 ml-2"
                                    >{{ client.version || "—" }}</span
                                >
                            </div>
                            <code
                                class="text-[11px] font-mono text-base-content/30 shrink-0 hidden sm:block"
                                >{{ String(client.md5_hash).slice(0, 10) }}</code
                            >
                            <span class="text-xs text-base-content/30 shrink-0">{{
                                formatDate(client.created_at)
                            }}</span>
                        </div>
                    </div>
                </div>
                <div
                    v-else
                    class="bg-base-200 rounded-xl border border-base-300 p-6 text-center text-sm text-base-content/40"
                >
                    {{ t("hfapi.no_changes") }}
                </div>
            </div>

            <h2 class="text-lg font-semibold text-primary-focus mb-3">
                {{ t("hfapi.all_clients") }}
            </h2>

            <div class="flex gap-1 p-1 bg-base-300/50 rounded-xl mb-4 w-fit">
                <button
                    v-for="tab in tabs"
                    :key="tab"
                    @click="activeTab = tab"
                    class="flex items-center gap-2 px-4 py-1.5 rounded-lg text-sm font-medium transition-all duration-200 capitalize"
                    :class="
                        activeTab === tab
                            ? 'bg-primary text-primary-content'
                            : 'text-base-content/50 hover:text-base-content'
                    "
                >
                    {{ typeLabel(tab) }}
                    <span
                        class="text-[10px] px-1.5 py-0.5 rounded-md"
                        :class="
                            activeTab === tab
                                ? 'bg-black/15'
                                : 'bg-base-content/10'
                        "
                        >{{ counts[tab] }}</span
                    >
                </button>
            </div>

            <div class="overflow-x-auto">
                <table class="table table-zebra w-full text-sm">
                    <thead>
                        <tr
                            class="text-base-content/50 text-xs uppercase tracking-wider"
                        >
                            <th>{{ t("hfapi.table_name") }}</th>
                            <th>{{ t("hfapi.table_version") }}</th>
                            <th class="hidden sm:table-cell">
                                {{ t("hfapi.table_hash") }}
                            </th>
                            <th class="hidden sm:table-cell">
                                {{ t("hfapi.table_size") }}
                            </th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr
                            v-for="client in filteredClients"
                            :key="client.id"
                        >
                            <td>
                                <div class="flex items-center gap-2">
                                    <span class="font-semibold">{{
                                        client.name
                                    }}</span>
                                    <span
                                        v-if="!client.working"
                                        class="badge badge-error badge-sm"
                                        >{{ t("hfapi.status_down") }}</span
                                    >
                                </div>
                            </td>
                            <td>
                                <span class="badge badge-outline badge-sm">{{
                                    client.version || "—"
                                }}</span>
                            </td>
                            <td class="hidden sm:table-cell">
                                <code class="text-xs font-mono text-base-content/40">{{
                                    client.md5_hash
                                }}</code>
                            </td>
                            <td class="hidden sm:table-cell">
                                <span class="text-xs text-base-content/40"
                                    >{{ client.size }} MB</span
                                >
                            </td>
                        </tr>
                    </tbody>
                </table>
            </div>

            <div class="mt-4 pt-4 border-t border-base-300/50">
                <div
                    class="flex flex-wrap items-center gap-4 text-xs text-base-content/40"
                >
                    <span
                        >{{ t("hfapi.total") }}:
                        <strong class="text-base-content/60">{{
                            counts.total
                        }}</strong></span
                    >
                    <span
                        >{{ t("hfapi.fabric") }}:
                        <strong class="text-base-content/60">{{
                            counts.fabric
                        }}</strong></span
                    >
                    <span
                        >{{ t("hfapi.forge") }}:
                        <strong class="text-base-content/60">{{
                            counts.forge
                        }}</strong></span
                    >
                    <span
                        >{{ t("hfapi.default") }}:
                        <strong class="text-base-content/60">{{
                            counts.default
                        }}</strong></span
                    >
                </div>
            </div>
        </template>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@shared/composables/useToast";
import { formatDate } from "@shared/utils/utils";
import { RefreshCcw } from "@lucide/vue";
import {
    hfClientsService,
    type HfClient,
} from "@services/hfClientsService";

const { t } = useI18n();
const { addToast } = useToast();

const clients = ref<HfClient[]>([]);
const latest = ref<HfClient[]>([]);
const counts = ref({
    total: 0,
    fabric: 0,
    forge: 0,
    default: 0,
});
const loading = ref(false);
const error = ref<string | null>(null);
const activeTab = ref<"fabric" | "forge" | "default">("fabric");

const tabs = ["fabric", "forge", "default"] as const;

const filteredClients = computed(() =>
    clients.value.filter((c) => c.client_type?.toLowerCase() === activeTab.value)
);

const typeLabel = (type: string): string => {
    const map: Record<string, string> = {
        fabric: t("hfapi.fabric"),
        forge: t("hfapi.forge"),
        default: t("hfapi.default"),
    };
    return map[type?.toLowerCase()] || type;
};

const typeBadgeClass = (type: string): string => {
    const t = type?.toLowerCase();
    if (t === "fabric")
        return "bg-purple-500/10 border-purple-500/20 text-purple-400";
    if (t === "forge")
        return "bg-orange-500/10 border-orange-500/20 text-orange-400";
    return "bg-blue-500/10 border-blue-500/20 text-blue-400";
};

const fetchData = async () => {
    loading.value = true;
    error.value = null;

    try {
        const result = await hfClientsService.fetchClients();
        clients.value = result.all;
        latest.value = result.latest;
        counts.value = result.counts;
    } catch (err: any) {
        console.error("Failed to fetch HF clients:", err);
        error.value = err.message || "Unknown error";
        addToast(t("hfapi.fetch_failed"), "error");
    } finally {
        loading.value = false;
    }
};

onMounted(() => {
    fetchData();
});
</script>

<style scoped>
.slide-up {
    animation: slideUp 0.6s ease-out forwards;
}

@keyframes slideUp {
    from {
        opacity: 0;
        transform: translateY(20px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}
</style>

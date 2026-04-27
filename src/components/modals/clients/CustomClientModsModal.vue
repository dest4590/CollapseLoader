<template>
    <div class="space-y-4 flex flex-col h-full overflow-hidden">
        <div v-if="!selectedVersionsMod" class="flex items-center gap-2">
            <input
                v-model="searchQuery"
                @keyup.enter="handleSearch"
                type="text"
                :placeholder="t('mods.search_placeholder')"
                class="input input-bordered w-full"
                :disabled="isLoading"
            />
            <button
                @click="handleSearch"
                class="btn btn-primary"
                :disabled="isLoading"
            >
                <Search class="w-4 h-4" />
            </button>
        </div>

        <div
            v-else
            class="flex items-center gap-4 border-b border-base-content/10 pb-2"
        >
            <button
                @click="selectedVersionsMod = null"
                class="btn btn-sm btn-ghost btn-circle"
            >
                <ChevronLeft class="w-4 h-4" />
            </button>
            <h3 class="font-bold truncate text-sm">
                {{ t("mods.versions_for", { mod: selectedVersionsMod.title }) }}
            </h3>
        </div>

        <div class="flex-1 overflow-y-auto custom-scrollbar pr-2 min-h-0">
            <div v-if="!selectedVersionsMod">
                <div
                    v-if="isLoading"
                    class="flex justify-center items-center h-40"
                >
                    <span class="loading loading-spinner loading-lg text-primary"></span>
                </div>

                <div v-else-if="error" class="alert alert-error">
                    <AlertCircle class="w-4 h-4" />
                    <span>{{ error }}</span>
                </div>

                <div
                    v-else-if="mods.length === 0 && hasSearched"
                    class="text-center py-8 text-base-content/60"
                >
                    <p>{{ t("mods.no_results") }}</p>
                </div>

                <div v-else class="space-y-2">
                    <div
                        v-for="mod in mods"
                        :key="mod.project_id"
                        class="bg-base-200/50 p-3 rounded-lg flex items-center gap-3 hover:bg-base-200 transition-colors"
                    >
                        <img
                            :src="mod.icon_url || 'https://cdn.modrinth.com/assets/unknown_server.png'"
                            :alt="mod.title"
                            class="w-10 h-10 rounded shadow-sm object-cover"
                        />
                        <div class="flex-1 min-w-0">
                            <div class="flex items-center gap-2">
                                <h4 class="font-bold text-sm truncate">{{ mod.title }}</h4>
                                <span class="text-[10px] bg-base-300 px-1.5 py-0.5 rounded text-base-content/60">{{ mod.author }}</span>
                                <span
                                    v-if="isModInstalled(mod)"
                                    class="badge badge-success badge-xs text-[10px] py-2 px-2 uppercase font-bold tracking-tighter opacity-70"
                                >{{ t("mods.installed") }}</span>
                            </div>
                            <p class="text-xs text-base-content/70 truncate">{{ mod.description }}</p>
                        </div>
                        <button
                            @click="showVersions(mod)"
                            class="btn btn-sm btn-primary"
                            :disabled="installingMods.has(mod.project_id)"
                        >
                            <span
                                v-if="installingMods.has(mod.project_id)"
                                class="loading loading-spinner loading-xs"
                            ></span>
                            <Download v-else class="w-4 h-4" />
                        </button>
                    </div>
                </div>
            </div>

            <div v-else class="space-y-2">
                <div
                    v-if="isLoadingVersions"
                    class="flex justify-center items-center h-40"
                >
                    <span class="loading loading-spinner loading-lg text-primary"></span>
                </div>
                <div
                    v-else-if="versions.length === 0"
                    class="text-center py-8 text-base-content/60"
                >
                    <p>{{ t("mods.no_compatible_version", { version: client.version }) }}</p>
                </div>
                <div
                    v-else
                    v-for="version in versions"
                    :key="version.id"
                    class="bg-base-200/50 p-3 rounded-lg flex items-center justify-between gap-3 border border-transparent hover:border-primary/30 transition-all"
                >
                    <div class="flex-1 min-w-0">
                        <div class="flex items-center gap-2 font-bold text-sm">
                            {{ version.version_number }}
                            <span v-if="version.version_type === 'release'" class="badge badge-success badge-xs">Release</span>
                            <span v-else-if="version.version_type === 'beta'" class="badge badge-warning badge-xs">Beta</span>
                        </div>
                        <div class="text-[10px] text-base-content/50 mt-1">
                            {{ new Date(version.date_published).toLocaleDateString() }}
                            • {{ version.loaders.join(", ") }}
                        </div>
                    </div>
                    <button
                        @click="installVersion(selectedVersionsMod, version)"
                        class="btn btn-sm btn-primary"
                    >
                        {{ t("mods.install") }}
                    </button>
                </div>
            </div>
        </div>

        <div class="flex justify-end gap-2 pt-2 border-t border-base-content/10">
            <button @click="emit('close')" class="btn btn-ghost">
                {{ t("common.close") }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { Search, Download, AlertCircle, ChevronLeft } from "lucide-vue-next";
import {
    ModrinthService,
    type ModrinthSearchResult,
    type ModrinthVersion,
} from "../../../services/modrinthService";
import { invoke } from "@tauri-apps/api/core";
import type { CustomClient } from "../../../types/ui";
import { useToast } from "../../../services/toastService";

const props = defineProps<{
    client: CustomClient;
}>();

const emit = defineEmits(["close"]);
const { t } = useI18n();
const { addToast } = useToast();

const searchQuery = ref("");
const mods = ref<ModrinthSearchResult[]>([]);
const isLoading = ref(false);
const error = ref<string | null>(null);
const hasSearched = ref(false);
const installingMods = ref(new Set<string>());
const installedModFiles = ref<string[]>([]);
const selectedVersionsMod = ref<ModrinthSearchResult | null>(null);
const versions = ref<ModrinthVersion[]>([]);
const isLoadingVersions = ref(false);

const fetchInstalledMods = async () => {
    try {
        installedModFiles.value = await invoke<string[]>("list_installed_mods_custom", {
            id: props.client.id,
        });
    } catch (e) {
        console.error("Failed to fetch installed mods", e);
    }
};

const isModInstalled = (mod: ModrinthSearchResult) => {
    const slugLower = mod.slug.toLowerCase();
    const titleLower = mod.title.toLowerCase().replace(/\s+/g, "-");
    return installedModFiles.value.some((file) => {
        const fileLower = file.toLowerCase();
        return fileLower.includes(slugLower) || fileLower.includes(titleLower);
    });
};

onMounted(() => {
    fetchInstalledMods();
});

const handleSearch = async () => {
    if (!searchQuery.value.trim()) return;

    isLoading.value = true;
    error.value = null;
    mods.value = [];
    hasSearched.value = true;

    try {
        const loaders = props.client.client_type === "Forge" ? ["forge"] : ["fabric"];
        const result = await ModrinthService.searchMods(searchQuery.value, {
            limit: 20,
            facets: JSON.stringify([[`project_type:mod`]]),
        });
        mods.value = result.hits;
    } catch (e: any) {
        console.error("Search failed", e);
        error.value = e.message || "Failed to search mods";
    } finally {
        isLoading.value = false;
    }
};

const showVersions = async (mod: ModrinthSearchResult) => {
    selectedVersionsMod.value = mod;
    isLoadingVersions.value = true;
    try {
        const loaders = props.client.client_type === "Forge" ? ["forge"] : ["fabric"];
        versions.value = await ModrinthService.getModVersions(
            mod.slug,
            loaders,
            [props.client.version]
        );
    } catch (e: any) {
        console.error("Failed to fetch versions", e);
        addToast(t("mods.install_failed", { error: e.message }), "error", 5000);
        selectedVersionsMod.value = null;
    } finally {
        isLoadingVersions.value = false;
    }
};

const installVersion = async (
    mod: ModrinthSearchResult,
    version: ModrinthVersion
) => {
    if (installingMods.value.has(mod.project_id)) return;

    installingMods.value.add(mod.project_id);

    try {
        const file = version.files.find((f: any) => f.primary) || version.files[0];
        if (!file) {
            throw new Error("No file found in version");
        }

        await invoke("install_mod_for_custom_client", {
            id: props.client.id,
            url: file.url,
            filename: file.filename,
        });

        addToast(t("mods.install_success", { mod: mod.title }), "success", 3000);
        await fetchInstalledMods();
    } catch (e: any) {
        console.error("Install failed", e);
        addToast(t("mods.install_failed", { error: e.message }), "error", 5000);
    } finally {
        installingMods.value.delete(mod.project_id);
    }
};
</script>

<template>
    <div class="space-y-4 flex flex-col h-full overflow-hidden">
        <div v-if="!selectedVersionsMod" class="flex flex-col gap-2">
            <div class="flex items-center gap-2">
                <input
                    v-model="searchQuery"
                    @keyup.enter="handleSearch"
                    type="text"
                    :placeholder="t('mods.search_placeholder')"
                    class="input input-sm input-bordered w-full"
                    :disabled="isLoading"
                />
                <button
                    @click="handleSearch"
                    class="btn btn-sm btn-primary"
                    :disabled="isLoading"
                >
                    <Search class="w-4 h-4" />
                </button>
            </div>

            <div
                class="flex flex-wrap items-center justify-between gap-4 px-1 pb-1"
            >
                <div class="flex items-center gap-2">
                    <span class="text-xs font-semibold text-base-content/70"
                        >{{ t("mods.sort_by") }}:</span
                    >
                    <select
                        v-model="sortBy"
                        class="select select-sm select-bordered bg-base-200 h-8 min-h-0 text-xs"
                        @change="hasSearched && handleSearch()"
                    >
                        <option value="relevance">
                            {{ t("mods.sort_relevance") }}
                        </option>
                        <option value="downloads">
                            {{ t("mods.sort_downloads") }}
                        </option>
                        <option value="newest">
                            {{ t("mods.sort_newest") }}
                        </option>
                        <option value="updated">
                            {{ t("mods.sort_updated") }}
                        </option>
                    </select>
                </div>
                <label class="cursor-pointer label px-0 py-0 gap-2">
                    <span class="label-text text-xs font-semibold">{{
                        t("mods.strict_compatibility")
                    }}</span>
                    <input
                        type="checkbox"
                        v-model="enforceCompatibility"
                        class="checkbox checkbox-xs rounded checkbox-primary"
                        @change="hasSearched && handleSearch()"
                    />
                </label>
            </div>
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
                    <span
                        class="loading loading-spinner loading-lg text-primary"
                    ></span>
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
                            :src="
                                mod.icon_url ||
                                'https://cdn.modrinth.com/assets/unknown_server.png'
                            "
                            :alt="mod.title"
                            class="w-10 h-10 rounded shadow-sm object-cover"
                        />
                        <div class="flex-1 min-w-0">
                            <div class="flex items-center gap-2">
                                <h4
                                    class="font-bold text-sm truncate cursor-pointer hover:underline text-primary"
                                    @click="openInModrinth(mod)"
                                    :title="t('mods.open_in_modrinth')"
                                >
                                    {{ mod.title }}
                                </h4>
                                <span
                                    class="text-[10px] bg-base-300 px-1.5 py-0.5 rounded text-base-content/60"
                                    >{{ mod.author }}</span
                                >
                                <span
                                    v-if="isModInstalled(mod)"
                                    class="badge badge-success badge-xs text-[10px] py-2 px-2 uppercase font-bold tracking-tighter opacity-70"
                                    >{{ t("mods.installed") }}</span
                                >
                            </div>
                            <p class="text-xs text-base-content/70 truncate">
                                {{ mod.description }}
                            </p>
                        </div>
                        <div class="flex items-center gap-2">
                            <button
                                v-if="isModInstalled(mod)"
                                @click="uninstallMod(mod)"
                                class="btn btn-sm btn-error btn-outline"
                                :disabled="installingMods.has(mod.project_id)"
                            >
                                <span
                                    v-if="installingMods.has(mod.project_id)"
                                    class="loading loading-spinner loading-xs"
                                ></span>
                                <span v-else>{{ t("mods.uninstall") }}</span>
                            </button>
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
            </div>

            <div v-else class="space-y-2">
                <div
                    v-if="isLoadingVersions"
                    class="flex justify-center items-center h-40"
                >
                    <span
                        class="loading loading-spinner loading-lg text-primary"
                    ></span>
                </div>
                <div
                    v-else-if="versions.length === 0"
                    class="text-center py-8 text-base-content/60"
                >
                    <p>
                        {{
                            t("mods.no_compatible_version", {
                                version: client.version,
                            })
                        }}
                    </p>
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
                            <span
                                v-if="version.version_type === 'release'"
                                class="badge badge-success badge-xs"
                                >Release</span
                            >
                            <span
                                v-else-if="version.version_type === 'beta'"
                                class="badge badge-warning badge-xs"
                                >Beta</span
                            >
                        </div>
                        <div class="text-[10px] text-base-content/50 mt-1">
                            {{
                                new Date(
                                    version.date_published
                                ).toLocaleDateString()
                            }}
                            • {{ version.loaders.join(", ") }}
                        </div>
                    </div>
                    <button
                        v-if="!isVersionInstalled(version)"
                        @click="installVersion(selectedVersionsMod, version)"
                        class="btn btn-sm btn-primary"
                        :disabled="
                            installingMods.has(selectedVersionsMod.project_id)
                        "
                    >
                        <span
                            v-if="
                                installingMods.has(
                                    selectedVersionsMod.project_id
                                )
                            "
                            class="loading loading-spinner loading-xs"
                        ></span>
                        <span v-else>{{ t("mods.install") }}</span>
                    </button>
                    <button
                        v-else
                        class="btn btn-sm btn-success opacity-80 cursor-default"
                        disabled
                    >
                        {{ t("mods.installed") }}
                    </button>
                </div>
            </div>
        </div>

        <div
            class="flex justify-end gap-2 pt-2 border-t border-base-content/10"
        >
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
} from "@features/clients/modrinthService";
import { openUrl } from "@tauri-apps/plugin-opener";
import { invoke } from "@tauri-apps/api/core";
import type { CustomClient } from "@shared/types/ui";
import { useToast } from "@shared/composables/useToast";

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

const sortBy = ref<"relevance" | "downloads" | "newest" | "updated">(
    "relevance"
);
const enforceCompatibility = ref(true);

const fetchInstalledMods = async () => {
    try {
        installedModFiles.value = await invoke<string[]>(
            "list_installed_mods_custom",
            {
                id: props.client.id,
            }
        );
    } catch (e) {
        console.error("Failed to fetch installed mods", e);
    }
};

const isModInstalled = (mod: ModrinthSearchResult) => {
    const slugLower = mod.slug.toLowerCase().replace(/-/g, "");
    const titleLower = mod.title.toLowerCase().replace(/[^a-z0-9]/g, "");
    return installedModFiles.value.some((file) => {
        const fileLower = file.toLowerCase().replace(/[^a-z0-9]/g, "");
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
        const facetsArray = [["project_type:mod"]];

        if (enforceCompatibility.value && props.client.version) {
            facetsArray.push([`versions:${props.client.version}`]);

            if (props.client.client_type) {
                const loader =
                    props.client.client_type.toLowerCase() === "forge"
                        ? "forge"
                        : "fabric";
                facetsArray.push([`categories:${loader}`]);
            }
        }

        const result = await ModrinthService.searchMods(searchQuery.value, {
            limit: 20,
            facets: JSON.stringify(facetsArray),
            index: sortBy.value,
        });
        mods.value = result.hits;
    } catch (e: any) {
        console.error("Search failed", e);
        error.value = e.message || "Failed to search mods";
    } finally {
        isLoading.value = false;
    }
};

const isVersionInstalled = (version: ModrinthVersion) => {
    return version.files.some((file) =>
        installedModFiles.value.includes(file.filename)
    );
};

const showVersions = async (mod: ModrinthSearchResult) => {
    selectedVersionsMod.value = mod;
    isLoadingVersions.value = true;
    try {
        const loaders =
            props.client.client_type === "Forge" ? ["forge"] : ["fabric"];
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
        const file =
            version.files.find((f: any) => f.primary) || version.files[0];
        if (!file) {
            throw new Error("No file found in version");
        }

        await invoke("install_mod_for_custom_client", {
            id: props.client.id,
            url: file.url,
            filename: file.filename,
        });

        addToast(
            t("mods.install_success", { mod: mod.title }),
            "success",
            3000
        );
        await fetchInstalledMods();
    } catch (e: any) {
        console.error("Install failed", e);
        addToast(t("mods.install_failed", { error: e.message }), "error", 5000);
    } finally {
        installingMods.value.delete(mod.project_id);
    }
};

const uninstallMod = async (mod: ModrinthSearchResult) => {
    if (installingMods.value.has(mod.project_id)) return;

    installingMods.value.add(mod.project_id);
    try {
        const slugLower = mod.slug.toLowerCase().replace(/-/g, "");
        const titleLower = mod.title.toLowerCase().replace(/[^a-z0-9]/g, "");
        const filename = installedModFiles.value.find((file) => {
            const fileLower = file.toLowerCase().replace(/[^a-z0-9]/g, "");
            return (
                fileLower.includes(slugLower) || fileLower.includes(titleLower)
            );
        });

        if (!filename) throw new Error("Mod file not found locally");

        await invoke("uninstall_mod_custom", {
            id: props.client.id,
            filename,
        });

        addToast(
            t("mods.uninstall_success", { mod: mod.title }),
            "success",
            3000
        );
        await fetchInstalledMods();
    } catch (e: any) {
        console.error("Failed to uninstall", e);
        addToast(
            t("mods.uninstall_failed", { error: e.message }),
            "error",
            5000
        );
    } finally {
        installingMods.value.delete(mod.project_id);
    }
};

const openInModrinth = async (mod: ModrinthSearchResult) => {
    try {
        await openUrl(`https://modrinth.com/mod/${mod.slug}`);
    } catch (e) {
        console.error("Failed to open URL", e);
    }
};
</script>

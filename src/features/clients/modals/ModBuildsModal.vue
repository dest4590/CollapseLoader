<template>
    <div class="flex flex-col h-full overflow-hidden">
        <div v-if="isEditing" class="flex flex-col gap-3 overflow-y-auto flex-1 p-1">
            <div class="flex items-center gap-2">
                <button
                    class="btn btn-ghost btn-sm"
                    @click="cancelEdit"
                >
                    <ArrowLeft class="w-4 h-4" />
                </button>
                <h3 class="font-bold text-sm">
                    {{ editingBuild.id ? t("mod_builds.edit_build") : t("mod_builds.create_build") }}
                </h3>
            </div>

            <div class="form-control gap-1">
                <label class="label py-0">
                    <span class="label-text text-xs">{{ t("mod_builds.build_name") }}</span>
                </label>
                <input
                    v-model="editingBuild.name"
                    type="text"
                    class="input input-sm input-bordered"
                    :placeholder="t('mod_builds.build_name_placeholder')"
                />
            </div>

            <div class="form-control gap-1">
                <label class="label py-0">
                    <span class="label-text text-xs">{{ t("mod_builds.build_description") }}</span>
                </label>
                <textarea
                    v-model="editingBuild.description"
                    class="textarea textarea-sm textarea-bordered h-16"
                    :placeholder="t('mod_builds.build_description_placeholder')"
                ></textarea>
            </div>

            <div class="flex gap-2">
                <div class="form-control gap-1 flex-1">
                    <label class="label py-0">
                        <span class="label-text text-xs">{{ t("mod_builds.loader") }}</span>
                    </label>
                    <select
                        v-model="editingBuild.loader"
                        class="select select-sm select-bordered"
                        @change="onLoaderChange"
                    >
                        <option value="fabric">Fabric</option>
                        <option value="forge">Forge</option>
                    </select>
                </div>
                <div class="form-control gap-1 flex-1">
                    <label class="label py-0">
                        <span class="label-text text-xs">{{ t("mod_builds.mc_version") }}</span>
                    </label>
                    <select
                        v-model="editingBuild.mc_version"
                        class="select select-sm select-bordered"
                        :disabled="versionsLoading"
                    >
                        <option v-if="versionsLoading" value="" disabled>
                            {{ t("common.loading") }}...
                        </option>
                        <option v-for="v in availableVersions" :key="v" :value="v">
                            {{ v }}
                        </option>
                    </select>
                </div>
            </div>

            <div class="divider text-xs opacity-50">{{ t("mod_builds.mods_in_build") }} ({{ editingBuild.mods.length }})</div>

            <div class="flex gap-2">
                <div class="relative flex-1">
                    <input
                        v-model="modSearchQuery"
                        @input="onSearchInput"
                        @keyup.enter="searchModsToAdd"
                        type="text"
                        class="input input-sm input-bordered w-full pr-8"
                        :placeholder="t('mod_builds.search_mods')"
                    />
                    <button
                        v-if="modSearchQuery"
                        class="absolute right-2 top-1/2 -translate-y-1/2 btn btn-ghost btn-xs"
                        @click="clearSearch"
                    >
                        <X class="w-3 h-3" />
                    </button>
                </div>
                <button
                    class="btn btn-sm btn-primary shrink-0"
                    :disabled="isSearchingMods || !modSearchQuery.trim()"
                    @click="searchModsToAdd"
                >
                    <Search v-if="!isSearchingMods" class="w-4 h-4" />
                    <span v-else class="loading loading-spinner loading-xs"></span>
                </button>
            </div>

            <div v-if="isSearchingMods" class="flex justify-center py-2">
                <span class="loading loading-spinner loading-sm text-primary"></span>
            </div>

            <div v-else-if="modSearchResults.length > 0" class="flex flex-col gap-1 max-h-48 overflow-y-auto">
                <div
                    v-for="mod in modSearchResults"
                    :key="mod.project_id"
                    class="flex items-center gap-2 p-2 bg-base-200 rounded hover:bg-base-300 transition-colors"
                >
                    <img
                        v-if="mod.icon_url"
                        :src="mod.icon_url"
                        class="w-8 h-8 rounded"
                        @error="($event.target as HTMLImageElement).style.display = 'none'"
                    />
                    <div class="flex-1 min-w-0">
                        <div class="text-xs font-medium truncate">{{ mod.title }}</div>
                        <div class="text-[10px] opacity-50 line-clamp-1">{{ mod.description }}</div>
                        <div class="text-[10px] opacity-40 mt-0.5">{{ formatDownloads(mod.downloads) }}</div>
                    </div>
                    <button
                        v-if="isModInBuild(mod.slug)"
                        class="btn btn-xs btn-ghost text-success"
                        disabled
                    >
                        <Check class="w-3 h-3" />
                    </button>
                    <button
                        v-else
                        class="btn btn-xs btn-primary"
                        @click="addModToBuild(mod)"
                    >
                        <Plus class="w-3 h-3" />
                    </button>
                </div>
            </div>

            <div v-else-if="modSearchQuery && !isSearchingMods" class="text-center text-xs opacity-50 py-2">
                {{ t("mod_builds.no_results") }}
            </div>

            <div
                v-for="(mod, idx) in editingBuild.mods"
                :key="mod.slug"
                class="flex items-center gap-2 p-2 bg-base-200 rounded"
            >
                <img
                    v-if="mod.icon_url"
                    :src="mod.icon_url"
                    class="w-6 h-6 rounded"
                    @error="($event.target as HTMLImageElement).style.display = 'none'"
                />
                <div class="flex-1 min-w-0">
                    <div class="text-xs font-medium truncate">{{ mod.title }}</div>
                    <div class="text-[10px] opacity-50 truncate">{{ mod.version_number }}</div>
                </div>
                <button
                    class="btn btn-ghost btn-xs text-error"
                    @click="editingBuild.mods.splice(idx, 1)"
                >
                    <X class="w-3 h-3" />
                </button>
            </div>

            <div class="flex justify-end gap-2 mt-2">
                <button
                    class="btn btn-primary btn-sm"
                    :disabled="!editingBuild.name.trim() || !editingBuild.mc_version.trim()"
                    @click="saveBuild"
                >
                    {{ t("common.save") }}
                </button>
                <button class="btn btn-ghost btn-sm" @click="cancelEdit">
                    {{ t("common.cancel") }}
                </button>
            </div>
        </div>

        <div v-else class="flex flex-col gap-3 overflow-hidden flex-1">
            <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                    <button
                        class="btn btn-ghost btn-sm"
                        @click="emit('close')"
                    >
                        <ArrowLeft class="w-4 h-4" />
                    </button>
                    <h3 class="font-bold text-sm">{{ t("mod_builds.title") }}</h3>
                </div>
                <div class="flex gap-2">
                    <button
                        class="btn btn-ghost btn-sm border border-base-200"
                        @click="importBuildFromFile"
                    >
                        <Upload class="w-4 h-4" />
                        {{ t("mod_builds.import") }}
                    </button>
                    <button
                        class="btn btn-primary btn-sm"
                        @click="startCreate"
                    >
                        <Plus class="w-4 h-4" />
                        {{ t("mod_builds.create") }}
                    </button>
                </div>
            </div>

            <div v-if="builds.length === 0" class="flex flex-col items-center justify-center flex-1 gap-2 opacity-50">
                <PackageOpen class="w-10 h-10" />
                <span class="text-sm">{{ t("mod_builds.no_builds") }}</span>
            </div>

            <div v-else class="flex flex-col gap-2 overflow-y-auto flex-1">
                <div
                    v-for="build in builds"
                    :key="build.id"
                    class="card bg-base-200 shadow-sm"
                >
                    <div class="card-body p-3">
                        <div class="flex items-start justify-between">
                            <div class="flex-1 min-w-0">
                                <h4 class="font-medium text-sm truncate">{{ build.name }}</h4>
                                <p v-if="build.description" class="text-xs opacity-60 mt-0.5 line-clamp-2">
                                    {{ build.description }}
                                </p>
                                <div class="flex items-center gap-2 mt-1.5 flex-wrap">
                                    <span class="badge badge-xs badge-outline">{{ build.mc_version }}</span>
                                    <span class="badge badge-xs badge-primary">{{ build.loader }}</span>
                                    <span class="badge badge-xs badge-ghost">{{ build.mods.length }} {{ t("mod_builds.mods") }}</span>
                                </div>
                            </div>
                        </div>

                        <div class="card-actions justify-end mt-2">
                            <button
                                class="btn btn-primary btn-xs"
                                :disabled="isInstalling"
                                @click="installBuild(build)"
                            >
                                <Download class="w-3 h-3" />
                                {{ t("mod_builds.install_all") }}
                            </button>
                            <button
                                class="btn btn-ghost btn-xs"
                                @click="startEdit(build)"
                            >
                                <Pencil class="w-3 h-3" />
                            </button>
                            <button
                                class="btn btn-ghost btn-xs"
                                @click="exportBuild(build)"
                            >
                                <Share class="w-3 h-3" />
                            </button>
                            <button
                                class="btn btn-ghost btn-xs text-error"
                                @click="confirmDelete(build)"
                            >
                                <Trash2 class="w-3 h-3" />
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { useI18n } from "vue-i18n";
import { useToast } from "@shared/composables/useToast";
import { invoke } from "@tauri-apps/api/core";
import {
    ArrowLeft, Plus, Pencil, Trash2, Download, Share, PackageOpen, X, Search, Check, Upload,
} from "@lucide/vue";
import {
    ModBuildService,
    type ModBuild,
} from "@features/clients/modBuildService";
import {
    ModrinthService,
    type ModrinthSearchResult,
} from "@features/clients/modrinthService";
import { useCustomClientVersions } from "@features/clients/composables/useCustomClientVersions";
import type { Client } from "@shared/types/ui";

const props = defineProps<{
    client: Client;
}>();

const emit = defineEmits(["close"]);
const { t } = useI18n();
const { addToast } = useToast();

const builds = ref<ModBuild[]>([]);
const isEditing = ref(false);
const isInstalling = ref(false);
const editingBuild = ref<ModBuild>(createEmptyBuild());

const modSearchQuery = ref("");
const modSearchResults = ref<ModrinthSearchResult[]>([]);
const isSearchingMods = ref(false);

const { fetchVersions, getAvailableVersions, versionsLoading } = useCustomClientVersions();

const availableVersions = computed(() => {
    return getAvailableVersions(editingBuild.value.loader);
});

const onLoaderChange = () => {
    const versions = availableVersions.value;
    if (versions.length > 0 && !versions.includes(editingBuild.value.mc_version)) {
        editingBuild.value.mc_version = versions[versions.length - 1];
    }
};

function createEmptyBuild(): ModBuild {
    return {
        id: "",
        name: "",
        description: "",
        mc_version: props.client.version || "",
        loader: props.client.client_type?.toLowerCase() || "fabric",
        mods: [],
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString(),
    };
}

const loadBuilds = async () => {
    try {
        builds.value = await ModBuildService.getAll();
    } catch (e) {
        console.error("Failed to load builds", e);
    }
};

const startCreate = () => {
    editingBuild.value = createEmptyBuild();
    isEditing.value = true;
};

const startEdit = (build: ModBuild) => {
    editingBuild.value = JSON.parse(JSON.stringify(build));
    isEditing.value = true;
};

const cancelEdit = () => {
    isEditing.value = false;
    modSearchQuery.value = "";
    modSearchResults.value = [];
    editingBuild.value = createEmptyBuild();
};

let searchDebounce: ReturnType<typeof setTimeout> | null = null;

const onSearchInput = () => {
    if (searchDebounce) clearTimeout(searchDebounce);
    searchDebounce = setTimeout(() => {
        if (modSearchQuery.value.trim().length >= 2) {
            searchModsToAdd();
        }
    }, 400);
};

const clearSearch = () => {
    modSearchQuery.value = "";
    modSearchResults.value = [];
};

const formatDownloads = (n: number): string => {
    if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M downloads`;
    if (n >= 1_000) return `${(n / 1_000).toFixed(1)}K downloads`;
    return `${n} downloads`;
};

const importBuildFromFile = async () => {
    try {
        const { open } = await import("@tauri-apps/plugin-dialog");
        const path = await open({
            multiple: false,
            filters: [{ name: "JSON", extensions: ["json"] }],
        });
        if (!path) return;

        const { readTextFile } = await import("@tauri-apps/plugin-fs");
        const content = await readTextFile(path as string);
        const parsed = JSON.parse(content) as ModBuild;

        if (!parsed.name || !parsed.mc_version || !Array.isArray(parsed.mods)) {
            addToast(t("mod_builds.invalid_file"), "error");
            return;
        }

        parsed.id = ModBuildService.generateId();
        parsed.created_at = new Date().toISOString();
        parsed.updated_at = new Date().toISOString();
        await ModBuildService.create(parsed);
        await loadBuilds();
        addToast(t("mod_builds.import_success"), "success");
    } catch (e) {
        console.error("Failed to import build", e);
        addToast(t("mod_builds.import_failed"), "error");
    }
};

const searchModsToAdd = async () => {
    if (!modSearchQuery.value.trim()) return;
    isSearchingMods.value = true;
    try {
        const facets = JSON.stringify([
            ["project_type:mod"],
            [`versions:${editingBuild.value.mc_version}`],
            [`categories:${editingBuild.value.loader}`],
        ]);
        const result = await ModrinthService.searchMods(modSearchQuery.value, {
            facets,
            limit: 10,
        });
        modSearchResults.value = result.hits;
    } catch (e) {
        console.error("Failed to search mods", e);
        modSearchResults.value = [];
    } finally {
        isSearchingMods.value = false;
    }
};

const isModInBuild = (slug: string): boolean => {
    return editingBuild.value.mods.some((m) => m.slug === slug);
};

const addModToBuild = async (mod: ModrinthSearchResult) => {
    try {
        const versions = await ModrinthService.getModVersions(
            mod.slug,
            [editingBuild.value.loader],
            [editingBuild.value.mc_version]
        );
        if (versions.length === 0) {
            addToast(t("mod_builds.no_version_found"), "warning");
            return;
        }
        const version = versions[0];
        const file = version.files.find((f) => f.primary) || version.files[0];
        if (!file) return;

        editingBuild.value.mods.push({
            slug: mod.slug,
            title: mod.title,
            description: mod.description,
            icon_url: mod.icon_url || "",
            modrinth_url: `https://modrinth.com/mod/${mod.slug}`,
            version_id: version.id,
            version_number: version.version_number,
            filename: file.filename,
            download_url: file.url,
            file_size: file.size,
        });
        addToast(t("mod_builds.mod_added", { name: mod.title }), "success");
    } catch (e) {
        console.error("Failed to add mod", e);
    }
};

const saveBuild = async () => {
    try {
        const now = new Date().toISOString();
        if (editingBuild.value.id) {
            editingBuild.value.updated_at = now;
            await ModBuildService.update(editingBuild.value);
        } else {
            editingBuild.value.id = ModBuildService.generateId();
            editingBuild.value.created_at = now;
            editingBuild.value.updated_at = now;
            await ModBuildService.create(editingBuild.value);
        }
        isEditing.value = false;
        editingBuild.value = createEmptyBuild();
        await loadBuilds();
        addToast(t("mod_builds.build_saved"), "success");
    } catch (e) {
        console.error("Failed to save build", e);
        addToast(t("mod_builds.save_failed"), "error");
    }
};

const confirmDelete = async (build: ModBuild) => {
    try {
        await ModBuildService.delete(build.id);
        await loadBuilds();
        addToast(t("mod_builds.build_deleted"), "success");
    } catch (e) {
        console.error("Failed to delete build", e);
    }
};

const exportBuild = async (build: ModBuild) => {
    try {
        const json = JSON.stringify(build, null, 2);
        const blob = new Blob([json], { type: "application/json" });
        const url = URL.createObjectURL(blob);
        const a = document.createElement("a");
        a.href = url;
        a.download = `${build.name.replace(/[^a-z0-9]/gi, "_").toLowerCase()}.json`;
        a.click();
        URL.revokeObjectURL(url);
        addToast(t("mod_builds.build_exported"), "success");
    } catch (e) {
        console.error("Failed to export build", e);
    }
};

const installBuild = async (build: ModBuild) => {
    if (build.mods.length === 0) {
        addToast(t("mod_builds.no_mods_to_install"), "warning");
        return;
    }

    isInstalling.value = true;
    let installed = 0;
    let failed = 0;

    for (const mod of build.mods) {
        try {
            await invoke("install_mod_from_url", {
                id: props.client.id,
                url: mod.download_url,
                filename: mod.filename,
            });
            installed++;
        } catch (e) {
            console.error(`Failed to install mod ${mod.title}:`, e);
            failed++;
        }
    }

    isInstalling.value = false;

    if (failed === 0) {
        addToast(
            t("mod_builds.all_mods_installed", { count: installed }),
            "success"
        );
    } else {
        addToast(
            t("mod_builds.some_mods_failed", { installed, failed }),
            "warning"
        );
    }
};

onMounted(() => {
    loadBuilds();
    fetchVersions();
});
</script>

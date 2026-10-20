<script setup lang="ts">
import { computed, onMounted, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { useI18n } from "vue-i18n";
import { LibraryBig, Search, Shirt, Trash2 } from "@lucide/vue";
import { useToast } from "@shared/composables/useToast";
import { useModal } from "@shared/composables/useModal";
import type { SavedSkin, SkinProfile, SkinVariant } from "@shared/types/skin";
import { MINECRAFT_SKINS } from "@shared/data/minecraftSkins";
import SkinThumbnail from "./SkinThumbnail.vue";

interface CatalogSkin {
    key: string;
    name: string;
    variant: SkinVariant;
    textureUrl: string;
    section?: string;
    savedId?: string;
    textureData?: string;
    textureKey?: string;
}

interface CatalogGroup {
    section: string;
    skins: CatalogSkin[];
}

const props = defineProps<{
    accountId: string;
    refreshKey: number;
}>();

const emit = defineEmits<{ (e: "applied", profile: SkinProfile): void }>();

const { t } = useI18n();
const { addToast } = useToast();
const { showConfirm } = useModal();

const tab = ref<"saved" | "defaults">("saved");
const search = ref("");
const savedSkins = ref<SavedSkin[]>([]);
const loading = ref(false);
const loadError = ref("");
const applyingKey = ref<string | null>(null);
const deletingId = ref<string | null>(null);

const filteredSkins = computed<CatalogSkin[]>(() => {
    const source: CatalogSkin[] =
        tab.value === "saved"
            ? savedSkins.value.map((skin) => ({
                  key: skin.id,
                  name: skin.name,
                  variant: skin.variant,
                  textureUrl: skin.textureUrl,
                  savedId: skin.id,
              }))
            : MINECRAFT_SKINS.map((skin) => ({
                  key: skin.id,
                  name: skin.name,
                  variant: skin.variant,
                  textureUrl: skin.textureData,
                  textureData: skin.textureData,
                  section: skin.section,
              }));

    const query = search.value.trim().toLowerCase();
    if (!query) {
        return source;
    }
    return source.filter((skin) =>
        `${skin.name} ${skin.variant} ${skin.section ?? ""}`
            .toLowerCase()
            .includes(query)
    );
});

const filteredGroups = computed<CatalogGroup[]>(() => {
    if (tab.value === "saved") {
        return [{ section: "", skins: filteredSkins.value }];
    }
    const sections = new Map<string, CatalogSkin[]>();
    for (const skin of filteredSkins.value) {
        const section = skin.section ?? "";
        const skins = sections.get(section);
        if (skins) {
            skins.push(skin);
        } else {
            sections.set(section, [skin]);
        }
    }
    return [...sections].map(([section, skins]) => ({ section, skins }));
});

const loadSaved = async () => {
    loading.value = true;
    loadError.value = "";
    try {
        savedSkins.value = await invoke<SavedSkin[]>(
            "list_saved_minecraft_skins"
        );
    } catch (error) {
        loadError.value = String(error);
        addToast(
            t("settings.skin_catalog_load_failed", { error: String(error) }),
            "error"
        );
    } finally {
        loading.value = false;
    }
};

const applySkin = async (skin: CatalogSkin) => {
    if (!props.accountId || applyingKey.value !== null) {
        return;
    }
    applyingKey.value = skin.key;
    try {
        const profile = skin.savedId
            ? await invoke<SkinProfile>("apply_saved_minecraft_skin", {
                  accountId: props.accountId,
                  id: skin.savedId,
              })
            : await invoke<SkinProfile>("apply_default_minecraft_skin", {
                  accountId: props.accountId,
                  textureKey: skin.textureKey ?? null,
                  textureData: skin.textureData ?? null,
                  variant: skin.variant,
              });
        emit("applied", profile);
        addToast(
            t("settings.skin_catalog_applied", { name: skin.name }),
            "success"
        );
    } catch (error) {
        addToast(
            t("settings.skin_catalog_apply_failed", { error: String(error) }),
            "error"
        );
    } finally {
        applyingKey.value = null;
    }
};

const deleteSkin = async (skin: CatalogSkin) => {
    if (!skin.savedId || deletingId.value !== null) {
        return;
    }
    const confirmed = await showConfirm({
        title: t("settings.skin_catalog_delete"),
        message: t("settings.skin_catalog_delete_confirm", {
            name: skin.name,
        }),
        confirmLabel: t("common.delete"),
        cancelLabel: t("common.cancel"),
    });
    if (!confirmed) {
        return;
    }
    deletingId.value = skin.savedId;
    try {
        await invoke("delete_saved_minecraft_skin", { id: skin.savedId });
        savedSkins.value = savedSkins.value.filter(
            (saved) => saved.id !== skin.savedId
        );
        addToast(t("settings.skin_catalog_deleted"), "success");
    } catch (error) {
        addToast(
            t("settings.skin_catalog_delete_failed", { error: String(error) }),
            "error"
        );
    } finally {
        deletingId.value = null;
    }
};

onMounted(loadSaved);
watch(() => props.refreshKey, loadSaved);
</script>

<template>
    <div class="card bg-base-200 shadow-md border border-base-300">
        <div class="card-body p-4 space-y-4">
            <div
                class="flex flex-col lg:flex-row lg:items-center lg:justify-between gap-3"
            >
                <div class="flex items-start gap-3">
                    <div
                        class="w-10 h-10 rounded-xl bg-primary/10 flex items-center justify-center shrink-0"
                    >
                        <LibraryBig class="w-5 h-5 text-primary" />
                    </div>
                    <div>
                        <h3 class="font-semibold text-sm">
                            {{ t("settings.skin_catalog") }}
                        </h3>
                        <p class="text-xs text-base-content/60 mt-0.5">
                            {{ t("settings.skin_catalog_description") }}
                        </p>
                    </div>
                </div>

                <div class="flex flex-wrap items-center gap-2">
                    <div class="tabs tabs-box tabs-xs bg-base-300">
                        <button
                            class="tab"
                            :class="{ 'tab-active': tab === 'saved' }"
                            @click="tab = 'saved'"
                        >
                            {{ t("settings.skin_catalog_saved") }}
                            <span class="badge badge-xs badge-ghost ml-1">
                                {{ savedSkins.length }}
                            </span>
                        </button>
                        <button
                            class="tab"
                            :class="{ 'tab-active': tab === 'defaults' }"
                            @click="tab = 'defaults'"
                        >
                            {{ t("settings.skin_catalog_defaults") }}
                            <span class="badge badge-xs badge-ghost ml-1">
                                {{ MINECRAFT_SKINS.length }}
                            </span>
                        </button>
                    </div>
                    <label
                        class="input input-sm input-bordered flex items-center gap-2 bg-base-300/40 w-full sm:w-56"
                    >
                        <Search class="w-4 h-4 text-base-content/40" />
                        <input
                            v-model="search"
                            type="search"
                            class="grow"
                            :placeholder="
                                t('settings.skin_catalog_search_placeholder')
                            "
                            :aria-label="
                                t('settings.skin_catalog_search_placeholder')
                            "
                        />
                    </label>
                </div>
            </div>

            <div v-if="loading" class="flex items-center justify-center py-10">
                <span
                    class="loading loading-spinner loading-lg text-primary"
                ></span>
            </div>

            <div
                v-else-if="filteredSkins.length === 0"
                class="rounded-box border border-dashed border-base-300 py-10 px-4 text-center"
            >
                <Shirt class="w-8 h-8 mx-auto text-base-content/30" />
                <p class="text-sm text-base-content/60 mt-2">
                    {{
                        search.trim()
                            ? t("settings.skin_catalog_no_results")
                            : t("settings.skin_catalog_empty")
                    }}
                </p>
            </div>

            <div v-else class="space-y-5">
                <section
                    v-for="group in filteredGroups"
                    :key="group.section || 'saved'"
                >
                    <div
                        v-if="group.section"
                        class="flex items-center gap-2 mb-2"
                    >
                        <h4
                            class="text-xs font-semibold uppercase tracking-wide text-base-content/60"
                        >
                            {{ group.section }}
                        </h4>
                        <span class="badge badge-xs badge-ghost">
                            {{ group.skins.length }}
                        </span>
                        <div class="h-px flex-1 bg-base-300"></div>
                    </div>
                    <div
                        class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 xl:grid-cols-6 gap-2.5"
                    >
                        <article
                            v-for="skin in group.skins"
                            :key="skin.key"
                            class="group relative rounded-box border border-base-300 bg-base-300/40 overflow-hidden transition duration-150 hover:border-primary/50 hover:-translate-y-0.5"
                        >
                            <button
                                class="w-full text-left disabled:opacity-60"
                                :disabled="
                                    applyingKey !== null ||
                                    deletingId !== null ||
                                    !accountId
                                "
                                @click="applySkin(skin)"
                            >
                                <div
                                    class="skin-card-preview relative h-44 border-b border-base-300/60 p-2"
                                >
                                    <SkinThumbnail
                                        :texture-url="skin.textureUrl"
                                        :model="skin.variant"
                                    />
                                    <div
                                        v-if="applyingKey === skin.key"
                                        class="absolute inset-0 bg-base-100/60 backdrop-blur-[1px] flex items-center justify-center"
                                    >
                                        <span
                                            class="loading loading-spinner loading-md text-primary"
                                        ></span>
                                    </div>
                                    <div
                                        v-else
                                        class="absolute inset-x-2 bottom-2 opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none"
                                    >
                                        <span
                                            class="btn btn-primary btn-xs w-full shadow-sm"
                                        >
                                            {{
                                                t("settings.skin_catalog_apply")
                                            }}
                                        </span>
                                    </div>
                                </div>
                                <div
                                    class="px-2.5 pb-2 flex items-start justify-between gap-1.5"
                                >
                                    <span
                                        class="text-xs font-medium leading-tight line-clamp-2 min-h-8"
                                        :title="skin.name"
                                    >
                                        {{ skin.name }}
                                    </span>
                                    <span
                                        class="badge badge-xs badge-ghost uppercase shrink-0"
                                    >
                                        {{
                                            skin.variant === "slim"
                                                ? t("settings.skin_model_slim")
                                                : t(
                                                      "settings.skin_model_classic"
                                                  )
                                        }}
                                    </span>
                                </div>
                            </button>
                            <button
                                v-if="skin.savedId"
                                class="btn btn-circle btn-xs btn-ghost absolute top-1.5 right-1.5 bg-base-100/70 backdrop-blur opacity-0 group-hover:opacity-100"
                                :disabled="deletingId !== null"
                                :aria-label="t('settings.skin_catalog_delete')"
                                :title="t('settings.skin_catalog_delete')"
                                @click.stop="deleteSkin(skin)"
                            >
                                <span
                                    v-if="deletingId === skin.savedId"
                                    class="loading loading-spinner loading-xs"
                                ></span>
                                <Trash2 v-else class="w-3.5 h-3.5 text-error" />
                            </button>
                        </article>
                    </div>
                </section>
            </div>

            <div
                v-if="loadError"
                class="alert alert-error alert-sm rounded-md text-xs p-2"
            >
                {{ loadError }}
            </div>
        </div>
    </div>
</template>

<style scoped>
.skin-card-preview {
    background-image:
        radial-gradient(
            circle at 50% 38%,
            color-mix(in srgb, var(--color-primary) 10%, transparent),
            transparent 62%
        ),
        linear-gradient(
            to bottom,
            color-mix(in srgb, var(--color-base-100) 70%, transparent),
            transparent
        );
}
</style>

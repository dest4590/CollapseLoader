<template>
    <div>
        <div class="flex items-center justify-between mb-4">
            <div class="flex items-center gap-2">
                <TelegramIcon class="w-4 h-4 text-primary" />
                <span class="text-sm font-medium text-base-content/70">
                    {{ t("marketplace.tg_source") }}
                    <a
                        :href="`https://t.me/${TG_CHANNEL}`"
                        target="_blank"
                        rel="noreferrer"
                        class="text-primary hover:underline ml-1"
                        >@{{ TG_CHANNEL }}</a
                    >
                </span>
            </div>
            <button
                class="btn btn-ghost btn-xs gap-1"
                :class="{ 'loading loading-spinner': refreshing }"
                @click="refresh"
                :disabled="refreshing"
            >
                <RefreshCw v-if="!refreshing" class="w-3.5 h-3.5" />
                {{ t("marketplace.tg_refresh") }}
            </button>
        </div>

        <div class="relative mb-4">
            <input
                type="text"
                class="input input-sm input-bordered w-full pl-9 bg-base-100/10 border-base-content/10 focus:border-primary/50"
                :placeholder="t('marketplace.search_placeholder')"
                v-model="search"
            />
            <Search
                class="absolute left-3 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-base-content/60 pointer-events-none"
            />
        </div>

        <div
            v-if="loading"
            class="flex items-center gap-2 text-base-content/70 py-8 justify-center"
        >
            <span class="loading loading-spinner loading-md"></span>
            <span>{{ t("common.loading") }}</span>
        </div>

        <div v-else-if="error" class="text-center py-8">
            <WifiOff class="w-10 h-10 mx-auto mb-2 text-base-content/30" />
            <p class="text-sm text-base-content/50 mb-3">{{ error }}</p>
            <button class="btn btn-sm btn-outline" @click="refresh">
                {{ t("marketplace.tg_retry") }}
            </button>
        </div>

        <div
            v-else-if="!filteredThemes.length"
            class="text-center py-8 text-base-content/50"
        >
            <Palette class="w-10 h-10 mx-auto mb-2 opacity-30" />
            <p class="text-sm">{{ t("marketplace.no_items") }}</p>
        </div>

        <div
            v-else
            class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-3"
        >
            <div
                v-for="preset in filteredThemes"
                :key="preset.id"
                class="card bg-base-100/50 border border-white/5 shadow-xl hover:bg-base-300 transition-all duration-300 rounded-2xl group cursor-pointer"
                @click="openDetails(preset)"
            >
                <div class="card-body p-3.5">
                    <div class="flex flex-col h-full">
                        <div class="flex flex-col gap-0.5 mb-2.5">
                            <h4
                                class="text-base font-bold text-base-content tracking-tight line-clamp-1"
                            >
                                {{ preset.name }}
                            </h4>
                            <p
                                class="text-[9px] font-black text-base-content/30 uppercase tracking-widest"
                            >
                                {{
                                    t("marketplace.by_author", {
                                        name: `@${TG_CHANNEL}`.toUpperCase(),
                                    })
                                }}
                            </p>
                        </div>

                        <div
                            v-if="preset.theme?.backgroundImage"
                            class="w-full h-24 rounded-xl bg-base-300/50 border border-white/5 overflow-hidden relative group-hover:border-primary/20 transition-colors mb-2.5"
                        >
                            <div
                                class="w-full h-full bg-cover bg-center transition-transform duration-500 group-hover:scale-105"
                                :style="{
                                    backgroundImage: `url(${preset.theme.backgroundImage})`,
                                }"
                            ></div>
                        </div>
                        <div
                            v-else
                            class="w-full h-24 rounded-xl bg-base-300/20 border border-white/5 flex items-center justify-center mb-2.5 overflow-hidden group-hover:border-primary/5 transition-all duration-500"
                        >
                            <Image
                                class="w-5 h-5 text-base-content/10 group-hover:text-base-content/20 transition-colors duration-500"
                            />
                        </div>

                        <PresetColorPreview :preset="preset" class="mt-0!" />

                        <p
                            v-if="preset.description"
                            class="text-[11px] text-base-content/50 line-clamp-2 mt-2 leading-snug min-h-8"
                        >
                            {{ preset.description }}
                        </p>
                        <div v-else class="min-h-8"></div>
                    </div>

                    <div class="card-actions justify-end mt-2">
                        <div class="flex items-center gap-1">
                            <button
                                class="btn btn-circle btn-xs btn-primary border-none"
                                :title="t('marketplace.apply')"
                                @click.stop="applyTheme(preset)"
                            >
                                <PaintBucket class="w-3.5 h-3.5" />
                            </button>
                            <button
                                class="btn btn-circle btn-xs border-none"
                                :title="t('theme.presets.save_current')"
                                @click.stop="saveLocal(preset)"
                            >
                                <Download class="w-3.5 h-3.5" />
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
import {
    Search,
    RefreshCw,
    WifiOff,
    Image,
    PaintBucket,
    Download,
    Palette,
} from "lucide-vue-next";
import TelegramIcon from "../ui/icons/TelegramIcon.vue";
import PresetColorPreview from "./PresetColorPreview.vue";
import { telegramThemeService } from "../../services/telegramThemeService";
import { presetService } from "../../services/presetService";
import { useToast } from "../../services/toastService";
import { useModal } from "../../services/modalService";
import { buildPresetCreatePayload } from "../../utils/presetPayload";
import { usePresets } from "../../composables/usePresets";
import type { MarketplacePreset } from "../../types/presets";
import TelegramThemeDetailsModal from "../modals/common/TelegramThemeDetailsModal.vue";

const TG_CHANNEL = "CollapseTheme";

const { t } = useI18n();
const { addToast } = useToast();
const { showModal, hideModal } = useModal();
const { createPreset, loadPresets } = usePresets();

const themes = ref<MarketplacePreset[]>([]);
const loading = ref(true);
const refreshing = ref(false);
const error = ref<string | null>(null);
const search = ref("");

const filteredThemes = computed(() => {
    const q = search.value.trim().toLowerCase();
    if (!q) return themes.value;
    return themes.value.filter(
        (p) =>
            p.name?.toLowerCase().includes(q) ||
            p.description?.toLowerCase().includes(q)
    );
});

async function load(force = false) {
    error.value = null;
    try {
        const data = await telegramThemeService.fetchThemes(force);
        if (data.length === 0 && !force) {
            const fresh = await telegramThemeService.fetchThemes(true);
            themes.value = fresh;
        } else {
            themes.value = data;
        }
    } catch (e: any) {
        error.value = t("marketplace.tg_load_failed");
        console.error(e);
    }
}

async function refresh() {
    refreshing.value = true;
    telegramThemeService.clearCache();
    await load(true);
    refreshing.value = false;
}

function applyTheme(preset: MarketplacePreset) {
    const theme = preset.theme ?? {};
    presetService.applyPresetToTheme({
        customCSS: theme.customCSS ?? "",
        enableCustomCSS: theme.enableCustomCSS ?? false,
        base100: theme.base100,
        base200: theme.base200,
        base300: theme.base300,
        baseContent: theme.baseContent,
        primary: theme.primary,
        primaryContent: theme.primaryContent,
        secondary: theme.secondary,
        secondaryContent: theme.secondaryContent,
        accent: theme.accent,
        accentContent: theme.accentContent,
        neutral: theme.neutral,
        neutralContent: theme.neutralContent,
        info: theme.info,
        infoContent: theme.infoContent,
        success: theme.success,
        successContent: theme.successContent,
        warning: theme.warning,
        warningContent: theme.warningContent,
        error: theme.error,
        errorContent: theme.errorContent,
        backgroundImage: theme.backgroundImage,
        backgroundBlur: theme.backgroundBlur,
        backgroundOpacity: theme.backgroundOpacity,
    } as any);
    addToast(
        t("theme.presets.messages.apply_success", { name: preset.name }),
        "success"
    );
}

async function saveLocal(preset: MarketplacePreset) {
    const theme = preset.theme ?? {};
    const raw = buildPresetCreatePayload(
        preset.name ?? "Imported",
        preset.description,
        theme
    );
    const input = JSON.parse(
        JSON.stringify(raw, (_k, v) => (v === undefined ? null : v))
    );
    const result = await createPreset(input);
    if (result) {
        await loadPresets();
    }
}

function openDetails(preset: MarketplacePreset) {
    const id = `tg-details-${preset.id}`;
    showModal(
        id,
        TelegramThemeDetailsModal,
        { title: preset.name },
        { preset },
        {
            apply: () => {
                applyTheme(preset);
                hideModal(id);
            },
            save: () => {
                saveLocal(preset);
                hideModal(id);
            },
            close: () => hideModal(id),
        }
    );
}

onMounted(async () => {
    loading.value = true;
    await load();
    loading.value = false;
});
</script>

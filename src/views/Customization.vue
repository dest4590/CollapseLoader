<template>
    <div class="container mx-auto mt-4">
        <div key="theme" class="grid grid-cols-1 lg:grid-cols-12 gap-8">
            <ThemeModeSelector
                :theme-mode="themeMode"
                :schedule-light-start="scheduleLightStart"
                :schedule-light-end="scheduleLightEnd"
                :schedule-preview-theme="schedulePreviewTheme"
                :current-system-theme="currentSystemTheme"
                @select-mode="selectThemeMode"
                @update-schedule="updateScheduleTime"
            />
            <AccentColorPicker
                :primary-color="primaryColor"
                :theme-mode="themeMode"
                @update:primary-color="(v: string) => (primaryColor = v)"
                @reset="primaryColor = null"
            />
        </div>

        <div
            class="card bg-base-200 shadow-md border border-base-300 mb-6 mt-6"
        >
            <div class="card-body">
                <div
                    class="flex flex-col md:flex-row md:items-center md:justify-between gap-4"
                >
                    <h1 class="card-title text-2xl flex items-center gap-3">
                        <Save class="w-6 h-6 text-primary" />
                        {{ t("theme.preset") }}
                    </h1>

                    <div class="flex flex-wrap items-center gap-2 justify-end">
                        <div class="flex gap-2 flex-wrap">
                            <button
                                v-if="!isExternalWindow"
                                class="btn btn-accent btn-sm flex items-center gap-2"
                                @click="$emit('change-view', 'marketplace')"
                            >
                                <Store class="w-4 h-4" />
                                <span class="hidden sm:inline">{{
                                    t("marketplace.open_marketplace")
                                }}</span>
                            </button>

                            <button
                                class="btn btn-outline btn-sm flex items-center gap-2"
                                @click="resetStyles"
                            >
                                <RotateCcw class="w-4 h-4" />
                                <span class="hidden sm:inline">{{
                                    t("theme.reset_button")
                                }}</span>
                            </button>

                            <button
                                v-if="!isExternalWindow"
                                class="btn btn-outline btn-primary btn-sm flex items-center gap-2"
                                @click="openInNewWindow"
                            >
                                <ExternalLink class="w-4 h-4" />
                                <span class="hidden sm:inline">{{
                                    $t("theme.actions.open_inspector")
                                }}</span>
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div class="card bg-base-200 shadow-md border border-base-300 mb-6">
            <div class="card-body p-6">
                <PresetManager />
            </div>
        </div>

        <div class="card bg-base-200 shadow-md border border-base-300 mb-6">
            <div class="card-body p-6">
                <ColorEditors
                    :base100="base100 ?? null"
                    :base200="base200 ?? null"
                    :base300="base300 ?? null"
                    :base-content="baseContent ?? null"
                    :primary-color-override="primaryColor ?? null"
                    :primary-content="primaryContent ?? null"
                    :secondary="secondary ?? null"
                    :secondary-content="secondaryContent ?? null"
                    :accent="accent ?? null"
                    :neutral="neutral ?? null"
                    :neutral-content="neutralContent ?? null"
                    :info="info ?? null"
                    :info-content="infoContent ?? null"
                    :success="success ?? null"
                    :success-content="successContent ?? null"
                    :warning="warning ?? null"
                    :warning-content="warningContent ?? null"
                    :error="error ?? null"
                    :error-content="errorContent ?? null"
                    @update:base100="(v: string | null) => (base100 = v)"
                    @update:base200="(v: string | null) => (base200 = v)"
                    @update:base300="(v: string | null) => (base300 = v)"
                    @update:base-content="(v: string | null) => (baseContent = v)"
                    @update:primary-color-override="(v: string | null) => (primaryColor = v)"
                    @update:primary-content="(v: string | null) => (primaryContent = v)"
                    @update:secondary="(v: string | null) => (secondary = v)"
                    @update:secondary-content="(v: string | null) => (secondaryContent = v)"
                    @update:accent="(v: string | null) => (accent = v)"
                    @update:neutral="(v: string | null) => (neutral = v)"
                    @update:neutral-content="(v: string | null) => (neutralContent = v)"
                    @update:info="(v: string | null) => (info = v)"
                    @update:info-content="(v: string | null) => (infoContent = v)"
                    @update:success="(v: string | null) => (success = v)"
                    @update:success-content="(v: string | null) => (successContent = v)"
                    @update:warning="(v: string | null) => (warning = v)"
                    @update:warning-content="(v: string | null) => (warningContent = v)"
                    @update:error="(v: string | null) => (error = v)"
                    @update:error-content="(v: string | null) => (errorContent = v)"
                />

                <BackgroundSettings
                    :background-image="backgroundImage ?? null"
                    :background-blur="backgroundBlur ?? null"
                    :background-opacity="backgroundOpacity ?? null"
                    @update:background-image="(v: string) => (backgroundImage = v)"
                    @update:background-blur="(v: number) => (backgroundBlur = v)"
                    @update:background-opacity="(v: number) => (backgroundOpacity = v)"
                />
            </div>
        </div>

        <PanelBlurSettings
            :spotlight-blur="spotlightBlur ?? 0"
            :history-blur="historyBlur ?? 0"
            :notifications-blur="notificationsBlur ?? 0"
            :disable-blur="disableBlur ?? false"
            @update:spotlight-blur="(v: number) => (spotlightBlur = v)"
            @update:history-blur="(v: number) => (historyBlur = v)"
            @update:notifications-blur="(v: number) => (notificationsBlur = v)"
            @update:disable-blur="(v: boolean) => (disableBlur = v)"
        />

        <ExpertCssEditor
            :enable-custom-css="enableCustomCSS"
            :custom-css="customCSS"
            :selected-theme="selectedTheme"
            :css-examples="cssExamples"
            @update:enable-custom-css="(v: boolean) => (enableCustomCSS = v)"
            @update:custom-css="(v: string) => (customCSS = v)"
            @add-example="addExample"
            @insert-example="insertExample"
            @open-export-modal="openExportModal"
            @open-import-modal="openImportModal"
        />

        <ClientCardPreview
            :is-external-window="isExternalWindow"
            @trigger-notification="triggerNotification"
        />
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, toRefs, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { emit as emitAppEvent, listen } from "@tauri-apps/api/event";
import { useI18n } from "vue-i18n";
import { Save, Store, RotateCcw, ExternalLink } from "@lucide/vue";
import { useToast } from "@shared/composables/useToast";
import { settingsService } from "@services/settings/settingsService";
import { themeService } from "@services/theme/themeService";
import PresetManager from "@features/presets/components/PresetManager.vue";
import ImportExportCssModal from "@features/presets/modals/ImportExportCssModal.vue";
import { useModal } from "@shared/composables/useModal";
import { themeScheduler } from "@services/theme/themeScheduler";

import ThemeModeSelector from "@components/customization/ThemeModeSelector.vue";
import AccentColorPicker from "@components/customization/AccentColorPicker.vue";
import ColorEditors from "@components/customization/ColorEditors.vue";
import BackgroundSettings from "@components/customization/BackgroundSettings.vue";
import PanelBlurSettings from "@components/customization/PanelBlurSettings.vue";
import ExpertCssEditor from "@components/customization/ExpertCssEditor.vue";
import ClientCardPreview from "@components/customization/ClientCardPreview.vue";

defineEmits(["change-view"]);

const i18n = useI18n();

defineOptions({
    name: "Customization",
});
const { t } = i18n;
const { addToast } = useToast();
const { showModal } = useModal();

type ThemeMode = "dark" | "light" | "system" | "schedule";

const currentSystemTheme = ref<"dark" | "light">(
    themeService.getSystemTheme()
);

const _getInitialThemeMode = (): ThemeMode => {
    const storedMode = themeService.getStoredThemeMode();
    if (storedMode === "system") return "system";
    if (themeScheduler.schedule.value.enabled) return "schedule";
    return (
        (document.documentElement.getAttribute("data-theme") as ThemeMode) ||
        "dark"
    );
};

const themeMode = ref<ThemeMode>(_getInitialThemeMode());
const selectedTheme = ref(
    document.documentElement.getAttribute("data-theme") || "dark"
);

const scheduleLightStart = computed(
    () => themeScheduler.schedule.value.lightStart
);
const scheduleLightEnd = computed(() => themeScheduler.schedule.value.lightEnd);
const schedulePreviewTheme = themeScheduler.previewTheme;

const applySystemTheme = async () => {
    const systemTheme = themeService.getSystemTheme();
    currentSystemTheme.value = systemTheme;
    await changeTheme(systemTheme);
};

const selectThemeMode = async (mode: ThemeMode) => {
    themeMode.value = mode;
    themeService.setStoredThemeMode(mode);

    if (mode === "schedule") {
        themeScheduler.updateSchedule({ enabled: true });
        selectedTheme.value = themeScheduler.previewTheme.value;
        themeService.stopSystemThemeListener();
    } else if (mode === "system") {
        themeScheduler.updateSchedule({ enabled: false });
        themeService.startSystemThemeListener(async (newTheme) => {
            currentSystemTheme.value = newTheme;
            await changeTheme(newTheme);
        });
        await applySystemTheme();
    } else {
        themeScheduler.updateSchedule({ enabled: false });
        themeService.stopSystemThemeListener();
        await changeTheme(mode);
    }
};

const updateScheduleTime = (
    field: "lightStart" | "lightEnd",
    value: string
) => {
    themeScheduler.updateSchedule({ [field]: value });
};

const isExternalWindow = window.location.search.includes(
    "window=customization"
);

const {
    customCSS,
    enableCustomCSS,
    primary: primaryColor,
    base100,
    base200,
    base300,
    baseContent,
    primaryContent,
    secondary,
    secondaryContent,
    accent,
    neutral,
    neutralContent,
    info,
    infoContent,
    success,
    successContent,
    warning,
    warningContent,
    error,
    errorContent,
    backgroundImage,
    backgroundBlur,
    backgroundOpacity,
    spotlightBlur,
    historyBlur,
    notificationsBlur,
    disableBlur,
} = toRefs(themeService.presetSettings);

watch(
    themeService.presetSettings,
    () => {
        themeService.saveCardSettings();
    },
    { deep: true }
);

const cssExamples = [
    {
        title: t("theme.example_1"),
        code: `.client-card {
  backdrop-filter: blur(5px);
  background-color: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
}`,
    },
    {
        title: t("theme.example_2"),
        code: `.client-card {
  /* Use --card-padding instead of padding
     to avoid breaking card layout */
  --card-padding: 12px;
}`,
    },
];

const changeTheme = async (theme: string) => {
    try {
        selectedTheme.value = theme;
        document.documentElement.setAttribute("data-theme", theme);
        localStorage.setItem("theme", theme);
        await invoke("set_window_theme", { theme });

        await settingsService.editSetting("theme", theme, false);
        await emitAppEvent("theme-mode-update", theme);

        addToast(t("theme.change_success"), "success");
    } catch (error) {
        console.error("Failed to save theme:", error);
        addToast(t("theme.save_failed", { error }), "error");
    }
};

const insertExample = (code: string) => {
    if (!enableCustomCSS.value) {
        addToast(t("theme.enable_custom_css_first"), "warning");
        return;
    }

    customCSS.value = customCSS.value
        ? `${customCSS.value.trim()}\n\n${code}`
        : code;

    addToast(t("theme.example_inserted"), "success");
};

const addExample = (className: string) => {
    if (!enableCustomCSS.value) {
        addToast(t("theme.enable_custom_css_first"), "warning");
        return;
    }

    const exampleCode = `${className} {\n  \n}`;
    customCSS.value = customCSS.value
        ? `${customCSS.value.trim()}\n\n${exampleCode}`
        : exampleCode;
};

const handleKeyDown = (event: KeyboardEvent) => {
    if ((event.ctrlKey || event.metaKey) && event.key === "s") {
        event.preventDefault();
        themeService.saveCardSettings();
    }
};

const resetStyles = () => {
    themeService.resetPresetSettings();
};

const triggerNotification = (
    type: "success" | "error" | "info" | "warning"
) => {
    const messages: Record<string, string> = {
        success: t("theme.notifications.success_message"),
        error: t("theme.notifications.error_message"),
        info: t("theme.notifications.info_message"),
        warning: t("theme.notifications.warning_message"),
    };

    const toastTypeMap: Record<
        string,
        "success" | "error" | "info" | "warning"
    > = {
        success: "success",
        error: "error",
        info: "info",
        warning: "warning",
    };

    addToast(messages[type], toastTypeMap[type]);
};

const openInNewWindow = async () => {
    try {
        const { WebviewWindow } = await import("@tauri-apps/api/webviewWindow");
        const webview = new WebviewWindow("customization-inspector", {
            url: "index.html?window=customization",
            title: "Theme Inspector",
            width: 1000,
            height: 800,
            resizable: true,
            decorations: true,
        });

        webview.once("tauri://created", function () {
            console.log("Customization window created");
        });

        webview.once("tauri://error", function (e) {
            console.error("Error creating window:", e);
        });
    } catch (e) {
        console.error("Failed to open new window:", e);
    }
};

const openExportModal = async () => {
    showModal(
        "export-css",
        ImportExportCssModal,
        { title: t("theme.export_css_title") },
        { mode: "export", css: customCSS.value },
        {}
    );
};

const openImportModal = () => {
    showModal(
        "import-css",
        ImportExportCssModal,
        { title: t("theme.import_css_title") },
        { mode: "import" },
        {
            import: (css: string) => {
                if (
                    /script|@import|url\(|expression|<|>|javascript:/i.test(css)
                ) {
                    addToast(t("theme.import_invalid"), "error");
                    return;
                }
                customCSS.value = css;
            },
        }
    );
};

onMounted(() => {
    document.addEventListener("keydown", handleKeyDown);
    listen<string>("theme-mode-update", (event) => {
        if (event.payload) {
            selectedTheme.value = event.payload;
            if (
                !themeScheduler.schedule.value.enabled &&
                themeMode.value !== "system"
            ) {
                themeMode.value = event.payload as
                    | "dark"
                    | "light"
                    | "system";
            }
        }
    }).then((unlisten) => {
        _unlistenThemeMode = unlisten;
    });

    if (themeMode.value === "system") {
        themeService.startSystemThemeListener(async (newTheme) => {
            currentSystemTheme.value = newTheme;
            await changeTheme(newTheme);
        });
    }
});

let _unlistenThemeMode: (() => void) | null = null;

onUnmounted(() => {
    document.removeEventListener("keydown", handleKeyDown);
    _unlistenThemeMode?.();
    themeService.stopSystemThemeListener();
});
</script>

<style scoped>
.check-pop-enter-active {
    transition: all 0.2s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.check-pop-leave-active {
    transition: all 0.15s ease-in;
}

.check-pop-enter-from,
.check-pop-leave-to {
    opacity: 0;
    transform: scale(0);
}

.check-pop-enter-to,
.check-pop-leave-from {
    opacity: 1;
    transform: scale(1);
}

.badge-pop-enter-active {
    transition: all 0.25s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.badge-pop-leave-active {
    transition: all 0.15s ease-in;
}

.badge-pop-enter-from,
.badge-pop-leave-to {
    opacity: 0;
    transform: scale(0.6) translateX(6px);
}

.badge-pop-enter-to,
.badge-pop-leave-from {
    opacity: 1;
    transform: scale(1) translateX(0);
}

.animate-fadeInUp {
    animation: fadeInUp 0.5s ease-out forwards;
    opacity: 0;
}

@keyframes fadeInUp {
    from {
        opacity: 0;
        transform: translateY(10px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.settings-card {
    opacity: 0;
    transform: translateY(10px);
    animation: fadeInUp 0.4s ease-out forwards;
}

.schedule-slide-enter-active {
    transition:
        opacity 0.4s ease-out,
        transform 0.4s ease-out;
}

.schedule-slide-leave-active {
}

.schedule-slide-enter-from,
.schedule-slide-leave-to {
    opacity: 0;
    transform: scale(0.96) translateX(10px);
}

.schedule-slide-enter-to,
.schedule-slide-leave-from {
    opacity: 1;
    transform: scale(1) translateX(0);
}

.blur-preview-wrapper {
    border-radius: 10px;
    border: 1px solid hsl(var(--b3));
    display: flex;
    gap: 8px;
    padding: 10px;
    background: hsl(var(--b3) / 0.4);
    min-height: 140px;
}

.blur-preview-spotlight,
.blur-preview-history {
    flex: 1;
    border-radius: 8px;
    border: 1px solid hsl(var(--b3));
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 10px 12px;
    position: relative;
    overflow: hidden;
}

.blur-preview-spotlight-bg,
.blur-preview-history-bg {
    position: absolute;
    inset: 0;
    background:
        radial-gradient(ellipse at 20% 50%, hsl(var(--p)) 0%, transparent 60%),
        radial-gradient(ellipse at 80% 30%, hsl(var(--a)) 0%, transparent 60%),
        radial-gradient(ellipse at 60% 80%, hsl(var(--s)) 0%, transparent 50%);
    opacity: 0.5;
    transition: filter 0.1s;
}

.blur-preview-spotlight-content,
.blur-preview-history-content {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    gap: 6px;
    background: hsl(var(--b2) / 0.5);
    border-radius: 6px;
    padding: 6px 8px;
}

.blur-preview-label {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: hsl(var(--bc) / 0.7);
    margin-bottom: 2px;
}

.blur-preview-row {
    display: flex;
    align-items: center;
    gap: 6px;
}

.blur-preview-icon {
    width: 20px;
    height: 20px;
    border-radius: 5px;
    background: hsl(var(--p) / 0.5);
    flex-shrink: 0;
}

.blur-preview-icon--sm {
    width: 16px;
    height: 16px;
}

.blur-preview-text {
    display: flex;
    flex-direction: column;
    gap: 3px;
    flex: 1;
}

.blur-preview-line {
    height: 6px;
    border-radius: 3px;
    background: hsl(var(--bc) / 0.3);
}
</style>

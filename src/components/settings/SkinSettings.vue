<script setup lang="ts">
import { computed, ref, watch } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useI18n } from "vue-i18n";
import {
    Check,
    ImageUp,
    RefreshCcw,
    RotateCcw,
    Shirt,
    Upload,
    UserRound,
    X,
} from "@lucide/vue";
import { useToast } from "@shared/composables/useToast";
import { useModal } from "@shared/composables/useModal";
import type { SkinProfile, SkinVariant } from "@shared/types/skin";
import SkinCatalog from "./SkinCatalog.vue";
import SkinPreview from "./SkinPreview.vue";

interface Account {
    id: string;
    username: string;
    is_active: boolean;
    account_type?: "offline" | "microsoft";
}

const props = defineProps<{ accounts: Account[] }>();
const emit = defineEmits<{ (e: "open-accounts"): void }>();

const { t } = useI18n();
const { addToast } = useToast();
const { showConfirm } = useModal();

const microsoftAccounts = computed(() =>
    props.accounts.filter((account) => account.account_type === "microsoft")
);

const selectedAccountId = ref("");
const profile = ref<SkinProfile | null>(null);
const loading = ref(false);
const uploading = ref(false);
const resetting = ref(false);
const capeUpdating = ref<string | null>(null);
const selectedFile = ref<string | null>(null);
const variant = ref<SkinVariant>("classic");
const loadError = ref("");
const catalogVersion = ref(0);

const modelOptions = computed(() => [
    { value: "classic" as const, label: t("settings.skin_model_classic") },
    { value: "slim" as const, label: t("settings.skin_model_slim") },
]);

const fileLabel = computed(() =>
    selectedFile.value
        ? selectedFile.value.split(/[\\/]/).pop() || selectedFile.value
        : ""
);

const activeCape = computed(
    () => profile.value?.capes.find((cape) => cape.active) ?? null
);

const applyProfile = (loaded: SkinProfile) => {
    profile.value = loaded;
    if (loaded.skinVariant === "slim" || loaded.skinVariant === "classic") {
        variant.value = loaded.skinVariant;
    }
};

const syncSelection = () => {
    const stillAvailable = microsoftAccounts.value.some(
        (account) => account.id === selectedAccountId.value
    );
    if (stillAvailable) {
        return;
    }
    const next =
        microsoftAccounts.value.find((account) => account.is_active) ??
        microsoftAccounts.value[0];
    selectedAccountId.value = next?.id ?? "";
};

const loadProfile = async () => {
    if (!selectedAccountId.value) {
        profile.value = null;
        return;
    }
    loading.value = true;
    loadError.value = "";
    try {
        const loaded = await invoke<SkinProfile>("get_minecraft_skin_profile", {
            accountId: selectedAccountId.value,
        });
        applyProfile(loaded);
    } catch (error) {
        profile.value = null;
        loadError.value = String(error);
        addToast(
            t("settings.skin_load_failed", { error: String(error) }),
            "error"
        );
    } finally {
        loading.value = false;
    }
};

watch(selectedAccountId, () => {
    selectedFile.value = null;
    void loadProfile();
});

watch(() => props.accounts, syncSelection, { immediate: true, deep: true });

const chooseFile = async () => {
    try {
        const selected = await open({
            multiple: false,
            directory: false,
            filters: [{ name: "PNG", extensions: ["png"] }],
        });
        if (typeof selected === "string") {
            selectedFile.value = selected;
        }
    } catch (error) {
        addToast(
            t("settings.skin_choose_file_failed", { error: String(error) }),
            "error"
        );
    }
};

const applySkin = async () => {
    if (!selectedFile.value) {
        addToast(t("settings.skin_choose_file_first"), "warning");
        return;
    }
    uploading.value = true;
    try {
        const updated = await invoke<SkinProfile>("upload_minecraft_skin", {
            accountId: selectedAccountId.value,
            filePath: selectedFile.value,
            variant: variant.value,
        });
        applyProfile(updated);
        selectedFile.value = null;
        catalogVersion.value += 1;
        addToast(t("settings.skin_upload_success"), "success");
    } catch (error) {
        addToast(
            t("settings.skin_upload_failed", { error: String(error) }),
            "error"
        );
    } finally {
        uploading.value = false;
    }
};

const resetSkin = async () => {
    const confirmed = await showConfirm({
        title: t("settings.skin_reset"),
        message: t("settings.skin_reset_confirm"),
        confirmLabel: t("settings.skin_reset"),
        cancelLabel: t("common.cancel"),
    });
    if (!confirmed) {
        return;
    }
    resetting.value = true;
    try {
        const updated = await invoke<SkinProfile>("reset_minecraft_skin", {
            accountId: selectedAccountId.value,
        });
        applyProfile(updated);
        addToast(t("settings.skin_reset_success"), "success");
    } catch (error) {
        addToast(
            t("settings.skin_reset_failed", { error: String(error) }),
            "error"
        );
    } finally {
        resetting.value = false;
    }
};

const setCape = async (capeId: string | null) => {
    if (!selectedAccountId.value || capeUpdating.value !== null) {
        return;
    }
    capeUpdating.value = capeId ?? "none";
    try {
        const updated = await invoke<SkinProfile>("set_minecraft_cape", {
            accountId: selectedAccountId.value,
            capeId,
        });
        applyProfile(updated);
        addToast(t("settings.skin_cape_updated"), "success");
    } catch (error) {
        addToast(
            t("settings.skin_cape_update_failed", { error: String(error) }),
            "error"
        );
    } finally {
        capeUpdating.value = null;
    }
};
</script>

<template>
    <div class="space-y-3">
        <div
            v-if="microsoftAccounts.length === 0"
            class="card bg-base-200 shadow-md border border-base-300"
        >
            <div
                class="card-body items-center text-center py-10 px-6 space-y-2"
            >
                <div
                    class="w-16 h-16 rounded-2xl bg-primary/10 flex items-center justify-center"
                >
                    <Shirt class="w-8 h-8 text-primary" />
                </div>
                <h2 class="card-title text-base font-semibold">
                    {{ t("settings.skin_no_microsoft_account") }}
                </h2>
                <p class="text-sm text-base-content/70 max-w-lg">
                    {{ t("settings.skin_no_microsoft_account_hint") }}
                </p>
                <button
                    class="btn btn-primary btn-sm mt-2"
                    @click="emit('open-accounts')"
                >
                    <UserRound class="w-4 h-4 mr-2" />
                    {{ t("settings.skin_open_accounts") }}
                </button>
            </div>
        </div>

        <template v-else>
            <div class="card bg-base-200 shadow-md border border-base-300">
                <div class="card-body p-4">
                    <div
                        class="flex flex-col md:flex-row justify-between items-start md:items-center gap-3"
                    >
                        <div>
                            <h2
                                class="card-title text-base font-semibold text-primary-focus flex items-center gap-2"
                            >
                                <Shirt class="w-5 h-5" />
                                {{ t("settings.skin_title") }}
                            </h2>
                            <p class="text-sm text-base-content/70 mt-1">
                                {{ t("settings.skin_description") }}
                            </p>
                        </div>
                        <label class="flex items-center gap-2 shrink-0">
                            <span class="text-sm text-base-content/70">{{
                                t("settings.skin_account")
                            }}</span>
                            <select
                                v-model="selectedAccountId"
                                class="select select-bordered select-sm"
                            >
                                <option
                                    v-for="account in microsoftAccounts"
                                    :key="account.id"
                                    :value="account.id"
                                >
                                    {{ account.username }}
                                </option>
                            </select>
                        </label>
                    </div>
                </div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-3 gap-3">
                <div class="card bg-base-200 shadow-md border border-base-300">
                    <div class="card-body p-4 items-center text-center">
                        <span
                            class="text-xs font-semibold uppercase tracking-wide text-base-content/50"
                            >{{ t("settings.skin_current") }}</span
                        >
                        <div
                            class="min-h-[210px] flex items-center justify-center py-2"
                        >
                            <span
                                v-if="loading"
                                class="loading loading-spinner loading-lg text-primary"
                            ></span>
                            <SkinPreview
                                v-else-if="profile?.skinUrl"
                                :texture-url="profile.skinUrl"
                                :model="variant"
                                :cape-url="activeCape?.url"
                            />
                            <div
                                v-else
                                class="flex flex-col items-center gap-2 text-base-content/40"
                            >
                                <UserRound class="w-16 h-16" />
                                <span class="text-sm">{{
                                    t("settings.skin_default_skin")
                                }}</span>
                            </div>
                        </div>
                        <p
                            v-if="profile?.skinUrl"
                            class="text-[11px] leading-tight text-base-content/50"
                        >
                            {{ t("settings.skin_drag_hint") }}
                        </p>
                        <div v-if="profile" class="space-y-1">
                            <div class="font-semibold text-base">
                                {{ profile.name }}
                            </div>
                            <div
                                class="text-xs text-base-content/50 font-mono truncate max-w-[220px]"
                            >
                                {{ profile.id }}
                            </div>
                            <div
                                class="badge badge-sm badge-ghost mt-1 uppercase"
                            >
                                {{
                                    variant === "slim"
                                        ? t("settings.skin_model_slim")
                                        : t("settings.skin_model_classic")
                                }}
                            </div>
                        </div>
                        <button
                            class="btn btn-ghost btn-xs mt-1"
                            :disabled="loading"
                            @click="loadProfile"
                        >
                            <RefreshCcw
                                class="w-3.5 h-3.5 mr-1"
                                :class="{ 'animate-spin': loading }"
                            />
                            {{ t("settings.skin_refresh") }}
                        </button>
                    </div>
                </div>

                <div class="lg:col-span-2 space-y-3">
                    <div
                        class="card bg-base-200 shadow-md border border-base-300"
                    >
                        <div class="card-body p-4 space-y-3">
                            <h3
                                class="font-semibold text-sm flex items-center gap-2"
                            >
                                <ImageUp class="w-5 h-5 text-primary" />
                                {{ t("settings.skin_model") }}
                            </h3>
                            <div class="flex flex-wrap gap-2">
                                <button
                                    v-for="option in modelOptions"
                                    :key="option.value"
                                    class="btn btn-sm"
                                    :class="
                                        variant === option.value
                                            ? 'btn-primary'
                                            : 'btn-ghost bg-base-300'
                                    "
                                    :disabled="uploading"
                                    @click="variant = option.value"
                                >
                                    {{ option.label }}
                                </button>
                            </div>

                            <div class="divider my-0"></div>

                            <div
                                class="flex flex-col sm:flex-row sm:items-center gap-2"
                            >
                                <button
                                    class="btn btn-outline btn-sm shrink-0"
                                    :disabled="uploading"
                                    @click="chooseFile"
                                >
                                    <Upload class="w-4 h-4 mr-2" />
                                    {{ t("settings.skin_choose_file") }}
                                </button>
                                <span
                                    class="text-sm text-base-content/70 truncate"
                                >
                                    {{
                                        fileLabel || t("settings.skin_no_file")
                                    }}
                                </span>
                            </div>
                            <p class="text-xs text-base-content/50">
                                {{ t("settings.skin_file_hint") }}
                            </p>

                            <div class="flex flex-wrap gap-2 pt-1">
                                <button
                                    class="btn btn-primary btn-sm"
                                    :disabled="
                                        !selectedFile ||
                                        uploading ||
                                        !profile ||
                                        loading
                                    "
                                    @click="applySkin"
                                >
                                    <span
                                        v-if="uploading"
                                        class="loading loading-spinner loading-xs mr-2"
                                    ></span>
                                    <Check v-else class="w-4 h-4 mr-2" />
                                    {{ t("settings.skin_apply") }}
                                </button>
                                <button
                                    class="btn btn-ghost btn-sm text-error hover:bg-error/10"
                                    :disabled="resetting || !profile || loading"
                                    @click="resetSkin"
                                >
                                    <span
                                        v-if="resetting"
                                        class="loading loading-spinner loading-xs mr-2"
                                    ></span>
                                    <RotateCcw v-else class="w-4 h-4 mr-2" />
                                    {{ t("settings.skin_reset") }}
                                </button>
                            </div>

                            <div
                                v-if="loadError"
                                class="alert alert-error alert-sm rounded-md text-xs p-2"
                            >
                                {{ loadError }}
                            </div>
                        </div>
                    </div>

                    <div
                        class="card bg-base-200 shadow-md border border-base-300"
                    >
                        <div class="card-body p-4 space-y-3">
                            <h3
                                class="font-semibold text-sm flex items-center gap-2"
                            >
                                <Shirt class="w-5 h-5 text-primary" />
                                {{ t("settings.skin_capes") }}
                            </h3>
                            <div
                                v-if="profile && profile.capes.length > 0"
                                class="flex flex-wrap gap-2"
                            >
                                <button
                                    class="cape-option"
                                    :class="{
                                        'cape-option-active': !activeCape,
                                    }"
                                    :disabled="capeUpdating !== null || loading"
                                    @click="setCape(null)"
                                >
                                    <div
                                        class="cape-thumb flex items-center justify-center"
                                    >
                                        <X
                                            class="w-4 h-4 text-base-content/40"
                                        />
                                    </div>
                                    <span class="truncate">{{
                                        t("settings.skin_cape_none")
                                    }}</span>
                                    <span
                                        v-if="capeUpdating === 'none'"
                                        class="loading loading-spinner loading-xs"
                                    ></span>
                                    <Check
                                        v-else-if="!activeCape"
                                        class="w-3.5 h-3.5 text-primary"
                                    />
                                </button>
                                <button
                                    v-for="cape in profile.capes"
                                    :key="cape.id"
                                    class="cape-option"
                                    :class="{
                                        'cape-option-active': cape.active,
                                    }"
                                    :disabled="capeUpdating !== null || loading"
                                    @click="setCape(cape.id)"
                                >
                                    <div
                                        class="cape-thumb"
                                        :style="{
                                            backgroundImage: `url(${cape.url})`,
                                        }"
                                    ></div>
                                    <span class="truncate">{{
                                        cape.alias || cape.id
                                    }}</span>
                                    <span
                                        v-if="capeUpdating === cape.id"
                                        class="loading loading-spinner loading-xs"
                                    ></span>
                                    <Check
                                        v-else-if="cape.active"
                                        class="w-3.5 h-3.5 text-primary"
                                    />
                                </button>
                            </div>
                            <p
                                v-else-if="profile"
                                class="text-sm text-base-content/60"
                            >
                                {{ t("settings.skin_capes_empty") }}
                            </p>
                        </div>
                    </div>
                </div>
            </div>

            <SkinCatalog
                :account-id="selectedAccountId"
                :refresh-key="catalogVersion"
                @applied="applyProfile"
            />
        </template>
    </div>
</template>

<style scoped>
.cape-option {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    padding: 0.4rem 0.6rem;
    max-width: 190px;
    border-radius: var(--radius-box, 0.5rem);
    border: 1px solid var(--color-base-300);
    background: var(--color-base-300);
    font-size: 0.8rem;
    transition:
        border-color 0.15s ease,
        background 0.15s ease,
        transform 0.15s ease;
}

.cape-option:hover:not(:disabled) {
    border-color: var(--color-primary);
    transform: translateY(-1px);
}

.cape-option:disabled {
    opacity: 0.6;
}

.cape-option-active {
    border-color: var(--color-primary);
    background: color-mix(in srgb, var(--color-primary) 12%, transparent);
}

.cape-thumb {
    width: 2rem;
    height: 2.6rem;
    border-radius: 0.25rem;
    background-color: rgb(0 0 0 / 0.15);
    background-size: 640% 200%;
    background-position: 0 0;
    background-repeat: no-repeat;
    image-rendering: pixelated;
    flex-shrink: 0;
}
</style>

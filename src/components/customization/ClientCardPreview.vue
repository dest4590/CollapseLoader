<template>
    <div
        v-if="isExternalWindow"
        class="card bg-base-200 shadow-md border border-base-300 mt-6"
    >
        <div class="card-body">
            <h2 class="card-title flex items-center gap-2 mb-2">
                <LayoutGrid class="w-5 h-5 text-primary" />
                {{ t("theme.client_card_preview.title") }}
            </h2>
            <p class="text-sm text-base-content/60 mb-4">
                {{ t("theme.client_card_preview.description") }}
            </p>

            <div class="flex flex-row gap-6 items-start">
                <div class="w-[420px] shrink-0">
                    <ClientCard
                        :client="mockClient"
                        :isClientRunning="previewRunning"
                        :isClientInstalling="previewInstalling"
                        :isRequirementsInProgress="previewRequirements"
                        :isFavorite="previewFavorite"
                        :isSelected="previewSelected"
                    />
                </div>

                <div class="flex-1 flex flex-col gap-5 min-w-0">
                    <div>
                        <h3
                            class="text-xs font-bold uppercase tracking-widest opacity-60 mb-2"
                        >
                            {{ t("theme.ui_preview.card_state") }}
                        </h3>
                        <div class="flex flex-wrap gap-2">
                            <label
                                class="flex items-center gap-2 cursor-pointer"
                            >
                                <input
                                    type="checkbox"
                                    class="toggle toggle-primary toggle-sm"
                                    v-model="previewRunning"
                                />
                                <span class="text-sm">{{
                                    t("theme.client_card_preview.controls.running")
                                }}</span>
                            </label>
                            <label
                                class="flex items-center gap-2 cursor-pointer"
                            >
                                <input
                                    type="checkbox"
                                    class="toggle toggle-primary toggle-sm"
                                    v-model="previewInstalling"
                                />
                                <span class="text-sm">{{
                                    t("theme.client_card_preview.controls.installing")
                                }}</span>
                            </label>
                            <label
                                class="flex items-center gap-2 cursor-pointer"
                            >
                                <input
                                    type="checkbox"
                                    class="toggle toggle-primary toggle-sm"
                                    v-model="previewRequirements"
                                />
                                <span class="text-sm">{{
                                    t("theme.client_card_preview.controls.requirements")
                                }}</span>
                            </label>
                            <label
                                class="flex items-center gap-2 cursor-pointer"
                            >
                                <input
                                    type="checkbox"
                                    class="toggle toggle-primary toggle-sm"
                                    v-model="previewFavorite"
                                />
                                <span class="text-sm">{{
                                    t("theme.client_card_preview.controls.favorite")
                                }}</span>
                            </label>
                            <label
                                class="flex items-center gap-2 cursor-pointer"
                            >
                                <input
                                    type="checkbox"
                                    class="toggle toggle-primary toggle-sm"
                                    v-model="previewSelected"
                                />
                                <span class="text-sm">{{
                                    t("theme.client_card_preview.controls.selected")
                                }}</span>
                            </label>
                        </div>
                    </div>

                    <div class="divider my-0"></div>

                    <div>
                        <h3
                            class="text-xs font-bold uppercase tracking-widest opacity-60 mb-2"
                        >
                            {{ t("theme.ui_preview.buttons") }}
                        </h3>
                        <div class="flex flex-wrap gap-2">
                            <button class="btn btn-primary btn-sm">
                                {{ t("theme.ui_preview.primary") }}
                            </button>
                            <button class="btn btn-secondary btn-sm">
                                {{ t("theme.ui_preview.secondary") }}
                            </button>
                            <button class="btn btn-accent btn-sm">
                                {{ t("theme.ui_preview.accent") }}
                            </button>
                            <button class="btn btn-ghost btn-sm">
                                {{ t("theme.ui_preview.ghost") }}
                            </button>
                            <button class="btn btn-outline btn-sm">
                                {{ t("theme.ui_preview.outline") }}
                            </button>
                            <button class="btn btn-success btn-sm">
                                {{ t("theme.ui_preview.success") }}
                            </button>
                            <button class="btn btn-warning btn-sm">
                                {{ t("theme.ui_preview.warning") }}
                            </button>
                            <button class="btn btn-error btn-sm">
                                {{ t("theme.ui_preview.error") }}
                            </button>
                            <button class="btn btn-info btn-sm">
                                {{ t("theme.ui_preview.info") }}
                            </button>
                            <button class="btn btn-neutral btn-sm">
                                {{ t("theme.ui_preview.neutral") }}
                            </button>
                        </div>
                        <div class="flex flex-wrap gap-2 mt-2">
                            <button class="btn btn-primary btn-sm gap-1">
                                <Download class="w-3.5 h-3.5" />
                                {{ t("theme.ui_preview.with_icon") }}
                            </button>
                            <button class="btn btn-error btn-sm gap-1">
                                <Trash2 class="w-3.5 h-3.5" />
                                {{ t("theme.ui_preview.delete") }}
                            </button>
                            <button class="btn btn-success btn-sm gap-1">
                                <Check class="w-3.5 h-3.5" />
                                {{ t("theme.ui_preview.confirm") }}
                            </button>
                            <button class="btn btn-ghost btn-sm btn-circle">
                                <Star class="w-4 h-4" />
                            </button>
                            <button
                                class="btn btn-outline btn-sm btn-circle"
                            >
                                <Search class="w-4 h-4" />
                            </button>
                            <button class="btn btn-disabled btn-sm">
                                {{ t("theme.ui_preview.disabled") }}
                            </button>
                        </div>
                    </div>

                    <div class="divider my-0"></div>

                    <div>
                        <h3
                            class="text-xs font-bold uppercase tracking-widest opacity-60 mb-2"
                        >
                            {{ t("theme.ui_preview.forms") }}
                        </h3>
                        <div class="flex flex-wrap items-center gap-3">
                            <input
                                type="text"
                                :placeholder="
                                    t('theme.ui_preview.input_placeholder')
                                "
                                class="input input-bordered input-sm w-48"
                            />
                            <select
                                class="select select-bordered select-sm w-36"
                            >
                                <option disabled selected>
                                    {{ t("theme.ui_preview.select_label") }}
                                </option>
                                <option>Option 1</option>
                                <option>Option 2</option>
                            </select>
                            <label class="cursor-pointer">
                                <input
                                    type="checkbox"
                                    class="checkbox checkbox-primary checkbox-sm"
                                />
                            </label>
                            <label
                                class="flex items-center gap-2 cursor-pointer"
                            >
                                <input
                                    type="checkbox"
                                    class="toggle toggle-secondary toggle-sm"
                                />
                                <span class="text-sm">Toggle</span>
                            </label>
                        </div>
                    </div>

                    <div class="divider my-0"></div>

                    <div>
                        <h3
                            class="text-xs font-bold uppercase tracking-widest opacity-60 mb-2"
                        >
                            {{ t("theme.ui_preview.badges") }}
                        </h3>
                        <div class="flex flex-wrap gap-2">
                            <span class="badge badge-primary badge-sm">{{
                                t("theme.ui_preview.primary")
                            }}</span>
                            <span class="badge badge-secondary badge-sm">{{
                                t("theme.ui_preview.secondary")
                            }}</span>
                            <span class="badge badge-accent badge-sm">{{
                                t("theme.ui_preview.accent")
                            }}</span>
                            <span class="badge badge-success badge-sm">{{
                                t("theme.ui_preview.success")
                            }}</span>
                            <span class="badge badge-warning badge-sm">{{
                                t("theme.ui_preview.warning")
                            }}</span>
                            <span class="badge badge-error badge-sm">{{
                                t("theme.ui_preview.error")
                            }}</span>
                            <span class="badge badge-info badge-sm">{{
                                t("theme.ui_preview.info")
                            }}</span>
                            <span class="badge badge-ghost badge-sm">{{
                                t("theme.ui_preview.ghost")
                            }}</span>
                            <span class="badge badge-outline badge-sm">{{
                                t("theme.ui_preview.outline")
                            }}</span>
                        </div>
                    </div>

                    <div class="divider my-0"></div>

                    <div>
                        <h3
                            class="text-xs font-bold uppercase tracking-widest opacity-60 mb-2"
                        >
                            {{ t("theme.ui_preview.alerts") }}
                        </h3>
                        <div class="flex flex-col gap-2">
                            <div
                                class="alert alert-success py-2 text-sm cursor-pointer hover:brightness-110 transition-all"
                                @click="$emit('trigger-notification', 'success')"
                            >
                                <CheckCircle class="w-4 h-4 shrink-0" />
                                {{ t("theme.ui_preview.alert_success") }}
                            </div>
                            <div
                                class="alert alert-error py-2 text-sm cursor-pointer hover:brightness-110 transition-all"
                                @click="$emit('trigger-notification', 'error')"
                            >
                                <XCircle class="w-4 h-4 shrink-0" />
                                {{ t("theme.ui_preview.alert_error") }}
                            </div>
                            <div
                                class="alert alert-warning py-2 text-sm cursor-pointer hover:brightness-110 transition-all"
                                @click="$emit('trigger-notification', 'warning')"
                            >
                                <AlertTriangle class="w-4 h-4 shrink-0" />
                                {{ t("theme.ui_preview.alert_warning") }}
                            </div>
                            <div
                                class="alert alert-info py-2 text-sm cursor-pointer hover:brightness-110 transition-all"
                                @click="$emit('trigger-notification', 'info')"
                            >
                                <Info class="w-4 h-4 shrink-0" />
                                {{ t("theme.ui_preview.alert_info") }}
                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import {
    LayoutGrid,
    Download,
    Trash2,
    Check,
    Star,
    Search,
    CheckCircle,
    XCircle,
    AlertTriangle,
    Info,
} from "@lucide/vue";
import ClientCard from "@features/clients/components/ClientCard.vue";
import type { Client } from "@shared/types/ui";

const { t } = useI18n();

defineProps<{
    isExternalWindow: boolean;
}>();

defineEmits<{
    "trigger-notification": [type: "success" | "error" | "info" | "warning"];
}>();

const mockClient: Client = {
    id: 9999,
    name: "Inspector Preview",
    version: "1.20.4",
    filename: "preview.jar",
    md5_hash: "mockhash",
    main_class: "net.minecraft.client.main.Main",
    show: true,
    working: true,
    insecure: false,
    rating_avg: 4.5,
    rating_count: 128,
    launches: 1337,
    downloads: 9001,
    client_type: "fabric",
    meta: {
        is_custom: false,
        size: "100 MB",
    },
};

const previewRunning = ref(false);
const previewInstalling = ref(false);
const previewRequirements = ref(false);
const previewFavorite = ref(false);
const previewSelected = ref(false);
</script>

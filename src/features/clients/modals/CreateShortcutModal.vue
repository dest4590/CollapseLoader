<script setup lang="ts">
import { ref, reactive } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useToast } from "@shared/composables/useToast";
import { useI18n } from "vue-i18n";
import { ImagePlus, X } from "@lucide/vue";
import type { Client, CustomClient } from "@shared/types/ui";

const { addToast } = useToast();
const { t } = useI18n();

const props = defineProps<{
    client: Client | CustomClient;
}>();

const emit = defineEmits<{
    close: [];
}>();

const isCustom = (c: Client | CustomClient): c is CustomClient =>
    !!(c as CustomClient).file_path;

const form = reactive({
    name: props.client.name,
    iconPath: "",
    iconFileName: "",
});

const loading = ref(false);

const selectIcon = async () => {
    try {
        const selected = await open({
            multiple: false,
            filters: [
                {
                    name: t("modals.create_shortcut.icon_filter_name"),
                    extensions: ["ico", "png"],
                },
            ],
        });
        if (selected) {
            form.iconPath = selected;
            const parts = selected.split(/[/\\]/);
            form.iconFileName = parts[parts.length - 1];
        }
    } catch {
        // user cancelled
    }
};

const clearIcon = () => {
    form.iconPath = "";
    form.iconFileName = "";
};

const handleSubmit = async () => {
    try {
        loading.value = true;

        const client = props.client;
        const custom = isCustom(client);

        await invoke("create_client_shortcut", {
            id: custom ? 0 : (client as Client).id,
            customId: custom ? client.id : null,
            isCustom: custom,
            shortcutName: form.name.trim() || null,
            iconPath: form.iconPath || null,
        });

        addToast(
            t("modals.create_shortcut.success", {
                name: form.name || client.name,
            }),
            "success"
        );
        emit("close");
    } catch (err) {
        addToast(t("modals.create_shortcut.error", { error: err }), "error");
    } finally {
        loading.value = false;
    }
};
</script>

<template>
    <form @submit.prevent="handleSubmit" class="space-y-4">
        <!-- Shortcut name -->
        <div class="form-control">
            <label class="label">
                <span class="label-text">{{
                    t("modals.create_shortcut.name_label")
                }}</span>
            </label>
            <input
                v-model="form.name"
                type="text"
                :placeholder="client.name"
                class="input input-bordered"
            />
            <label class="label">
                <span class="label-text-alt opacity-60">
                    {{ t("modals.create_shortcut.name_hint") }}
                </span>
            </label>
        </div>

        <!-- Icon picker -->
        <div class="form-control">
            <label class="label">
                <span class="label-text">{{
                    t("modals.create_shortcut.icon_label")
                }}</span>
            </label>

            <div
                v-if="!form.iconFileName"
                class="border-2 border-dashed border-base-300 hover:border-primary/50 rounded-lg p-4 text-center cursor-pointer transition-colors"
                @click="selectIcon"
            >
                <ImagePlus class="w-6 h-6 mx-auto mb-1 opacity-40" />
                <p class="text-sm text-base-content/60">
                    {{ t("modals.create_shortcut.icon_placeholder") }}
                </p>
                <p class="text-xs text-base-content/40 mt-0.5">
                    {{ t("modals.create_shortcut.icon_formats") }}
                </p>
            </div>

            <div
                v-else
                class="flex items-center gap-3 border border-base-300 rounded-lg px-3 py-2"
            >
                <ImagePlus class="w-4 h-4 text-success shrink-0" />
                <span class="text-sm truncate flex-1 text-success font-medium">
                    {{ form.iconFileName }}
                </span>
                <button
                    type="button"
                    class="btn btn-ghost btn-xs p-1"
                    @click="clearIcon"
                    :title="t('common.remove')"
                >
                    <X class="w-3 h-3" />
                </button>
            </div>

            <label class="label">
                <span class="label-text-alt opacity-60">
                    {{ t("modals.create_shortcut.icon_hint") }}
                </span>
            </label>
        </div>

        <div class="modal-action">
            <button
                type="button"
                class="btn"
                @click="$emit('close')"
                :disabled="loading"
            >
                {{ t("common.cancel") }}
            </button>
            <button type="submit" class="btn btn-primary" :disabled="loading">
                <span
                    v-if="loading"
                    class="loading loading-spinner loading-sm"
                ></span>
                {{
                    loading
                        ? t("modals.create_shortcut.creating")
                        : t("modals.create_shortcut.create")
                }}
            </button>
        </div>
    </form>
</template>

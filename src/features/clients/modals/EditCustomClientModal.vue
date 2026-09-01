<script setup lang="ts">
import { ref, reactive, watch, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useToast } from "@shared/composables/useToast";
import { useModal } from "@shared/composables/useModal";
import { useI18n } from "vue-i18n";
import type { CustomClient } from "@shared/types/ui";
import { useCustomClientVersions } from "../composables/useCustomClientVersions";

const { addToast } = useToast();
const { getModals } = useModal();
const { t } = useI18n();

const emit = defineEmits<{
    "client-edited": [];
    close: [];
}>();

const { fetchVersions, getAvailableVersions } = useCustomClientVersions();

const form = reactive({
    name: "",
    version: "",
    mainClass: "",
    javaPath: "",
    javaArgs: "",
    librariesPath: "",
    nativesPath: "",
    clientType: "default",
});

const loading = ref(false);
const errors = ref<Record<string, string>>({});
const currentClient = ref<CustomClient | null>(null);

const availableVersions = computed(() => {
    return getAvailableVersions(form.clientType);
});

watch(
    () => form.clientType,
    (newType) => {
        if (
            newType === "fabric" &&
            form.mainClass !== "net.fabricmc.loader.impl.launch.knot.KnotClient"
        ) {
            form.mainClass = "net.fabricmc.loader.impl.launch.knot.KnotClient";
        }

        const versions = getAvailableVersions(newType);
        if (
            versions &&
            versions.length > 0 &&
            !versions.includes(form.version)
        ) {
            form.version = versions[0];
        }
    }
);

onMounted(async () => {
    await fetchVersions();
});

const validateForm = () => {
    errors.value = {};

    if (!form.name.trim()) {
        errors.value.name = t("modals.edit_custom_client_modal.validate_name");
    }

    if (!form.version.trim()) {
        errors.value.version = t(
            "modals.edit_custom_client_modal.validate_version"
        );
    }

    if (!form.mainClass.trim()) {
        errors.value.mainClass = t(
            "modals.edit_custom_client_modal.validate_main_class"
        );
    }

    return Object.keys(errors.value).length === 0;
};

const selectJavaExecutable = async () => {
    try {
        const selected = await open({
            multiple: false,
        });

        if (selected) {
            form.javaPath = selected;
        }
    } catch (error) {
        console.log("Java executable selection cancelled or failed", error);
    }
};

const selectLibrariesDir = async () => {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
        });

        if (selected) {
            form.librariesPath = selected;
        }
    } catch (error) {
        console.log("Library directory selection cancelled or failed", error);
    }
};

const selectNativesDir = async () => {
    try {
        const selected = await open({
            directory: true,
            multiple: false,
        });

        if (selected) {
            form.nativesPath = selected;
        }
    } catch (error) {
        console.log("Natives directory selection cancelled or failed", error);
    }
};

const handleSubmit = async () => {
    if (!validateForm() || !currentClient.value) {
        return;
    }

    try {
        loading.value = true;

        await invoke("update_custom_client", {
            id: currentClient.value.id,
            name: form.name.trim(),
            version: form.version,
            mainClass: form.mainClass.trim(),
            javaPath: form.javaPath.trim() || null,
            javaArgs: form.javaArgs.trim() || null,
            librariesPath: form.librariesPath.trim() || null,
            nativesPath: form.nativesPath.trim() || null,
            client_type: form.clientType,
        });

        emit("client-edited");
        emit("close");
    } catch (err) {
        addToast(
            t("modals.edit_custom_client_modal.update_failed", { error: err }),
            "error"
        );
    } finally {
        loading.value = false;
    }
};

const modals = getModals();

watch(
    () => modals["edit-custom-client"]?.props?.client,
    (client: CustomClient | undefined) => {
        if (client) {
            currentClient.value = client;
            form.name = client.name;
            form.version = client.version;
            form.mainClass = client.main_class;
            form.javaPath = client.java_path || "";
            form.javaArgs = client.java_args || "";
            form.librariesPath = client.libraries_path || "";
            form.nativesPath = client.natives_path || "";
            form.clientType = client.client_type || "default";
        }
    },
    { immediate: true }
);
</script>

<template>
    <form @submit.prevent="handleSubmit" class="space-y-4">
        <div class="form-control">
            <label class="label">
                <span class="label-text"
                    >{{
                        $t("modals.edit_custom_client_modal.client_name")
                    }}
                    *</span
                >
            </label>
            <input
                v-model="form.name"
                type="text"
                :placeholder="
                    $t('modals.edit_custom_client_modal.enter_client_name')
                "
                class="input input-bordered"
                :class="{ 'input-error': errors.name }"
            />
            <label v-if="errors.name" class="label">
                <span class="label-text-alt text-error">{{ errors.name }}</span>
            </label>
        </div>

        <div class="form-control">
            <label class="label">
                <span class="label-text"
                    >{{
                        $t("modals.edit_custom_client_modal.minecraft_version")
                    }}
                    *</span
                >
            </label>
            <select
                v-model="form.version"
                class="select select-bordered w-full"
                :class="{ 'select-error': errors.version }"
            >
                <option v-for="v in availableVersions" :key="v" :value="v">
                    {{ v }}
                </option>
            </select>
            <label v-if="errors.version" class="label">
                <span class="label-text-alt text-error">{{
                    errors.version
                }}</span>
            </label>
        </div>

        <div class="form-control" v-if="form.clientType === 'default'">
            <label class="label">
                <span class="label-text"
                    >{{
                        $t("modals.edit_custom_client_modal.main_class")
                    }}
                    *</span
                >
            </label>
            <input
                v-model="form.mainClass"
                type="text"
                :placeholder="
                    $t('modals.edit_custom_client_modal.main_class_placeholder')
                "
                class="input input-bordered"
                :class="{ 'input-error': errors.mainClass }"
            />
            <label v-if="errors.mainClass" class="label">
                <span class="label-text-alt text-error">{{
                    errors.mainClass
                }}</span>
            </label>
        </div>

        <div class="form-control">
            <label class="label">
                <span class="label-text">{{
                    $t("modals.edit_custom_client_modal.client_type")
                }}</span>
            </label>
            <select
                v-model="form.clientType"
                class="select select-bordered w-full"
            >
                <option value="default">
                    {{
                        $t(
                            "modals.edit_custom_client_modal.client_type_vanilla"
                        )
                    }}
                </option>
                <option value="fabric">Fabric</option>
                <option value="forge">Forge</option>
            </select>
            <label class="label">
                <span class="label-text-alt opacity-60">{{
                    $t("modals.edit_custom_client_modal.client_type_hint")
                }}</span>
            </label>
        </div>

        <div class="divider text-xs opacity-50 uppercase tracking-widest">
            {{ $t("common.advanced") || "Advanced" }}
        </div>

        <div class="form-control" v-if="form.clientType === 'default'">
            <label class="label">
                <span class="label-text">{{
                    $t("modals.edit_custom_client_modal.java_path")
                }}</span>
            </label>
            <div class="join w-full">
                <input
                    v-model="form.javaPath"
                    type="text"
                    placeholder="C:\Path\To\bin\java.exe"
                    class="input input-bordered join-item w-full"
                />
                <button
                    type="button"
                    class="btn btn-primary join-item"
                    @click="selectJavaExecutable"
                >
                    {{ $t("common.select") }}
                </button>
            </div>
        </div>

        <div class="form-control">
            <label class="label">
                <span class="label-text">{{
                    $t("modals.edit_custom_client_modal.libraries_path")
                }}</span>
            </label>
            <div class="join w-full">
                <input
                    v-model="form.librariesPath"
                    type="text"
                    placeholder="/path/to/libraries"
                    class="input input-bordered join-item w-full"
                />
                <button
                    type="button"
                    class="btn btn-primary join-item"
                    @click="selectLibrariesDir"
                >
                    {{ $t("common.select") }}
                </button>
            </div>
        </div>

        <div class="form-control">
            <label class="label">
                <span class="label-text">{{
                    $t("modals.edit_custom_client_modal.natives_path")
                }}</span>
            </label>
            <div class="join w-full">
                <input
                    v-model="form.nativesPath"
                    type="text"
                    placeholder="/path/to/natives"
                    class="input input-bordered join-item w-full"
                />
                <button
                    type="button"
                    class="btn btn-primary join-item"
                    @click="selectNativesDir"
                >
                    {{ $t("common.select") }}
                </button>
            </div>
        </div>

        <div class="form-control">
            <label class="label">
                <span class="label-text">{{
                    $t("modals.edit_custom_client_modal.java_args")
                }}</span>
            </label>
            <textarea
                v-model="form.javaArgs"
                class="textarea textarea-bordered h-20"
                placeholder="-Xms512M -XX:+UseG1GC"
            ></textarea>
        </div>

        <div class="modal-action">
            <button
                type="button"
                class="btn"
                @click="$emit('close')"
                :disabled="loading"
            >
                {{ $t("common.cancel") }}
            </button>
            <button type="submit" class="btn btn-primary" :disabled="loading">
                <div
                    v-if="loading"
                    class="loading loading-spinner loading-sm"
                ></div>
                {{
                    loading
                        ? $t("modals.edit_custom_client_modal.updating")
                        : $t("modals.edit_custom_client_modal.update_client")
                }}
            </button>
        </div>
    </form>
</template>

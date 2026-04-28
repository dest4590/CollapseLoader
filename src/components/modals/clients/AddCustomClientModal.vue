<script setup lang="ts">
import { ref, reactive, watch, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { useToast } from "../../../services/toastService";
import { useI18n } from "vue-i18n";

const { addToast } = useToast();
const { t } = useI18n();

const emit = defineEmits<{
    "client-added": [];
    close: [];
}>();

const VERSION_MAP = {
    default: ["1.16.5"],
    forge: ["1.8.9"],
    fabric: ["1.21.4", "1.21.8", "1.21.11"]
};

const form = reactive({
    name: "",
    version: "",
    mainClass: "net.minecraft.client.main.Main",
    filePath: "",
    fileName: "",
    javaPath: "",
    javaArgs: "",
    clientType: "default",
});

const loading = ref(false);
const errors = ref<Record<string, string>>({});

const availableVersions = computed(() => {
    return VERSION_MAP[form.clientType as keyof typeof VERSION_MAP] || [];
});

watch(() => form.clientType, (newType) => {
    if (newType === 'fabric') {
        form.mainClass = "net.fabricmc.loader.impl.launch.knot.KnotClient";
    } else if (newType === 'default') {
        form.mainClass = "net.minecraft.client.main.Main";
    }
    
    // Auto-set version
    const versions = VERSION_MAP[newType as keyof typeof VERSION_MAP];
    if (versions && versions.length > 0) {
        form.version = versions[0];
    }
});

const validateForm = () => {
    errors.value = {};

    if (!form.name.trim()) {
        errors.value.name = t("modals.add_custom_client_modal.validate_name");
    }

    if (!form.version.trim()) {
        errors.value.version = t("modals.add_custom_client_modal.validate_version");
    }

    if (!form.mainClass.trim()) {
        errors.value.mainClass = t("modals.add_custom_client_modal.validate_main_class");
    }

    if (!form.filePath) {
        errors.value.filePath = t("modals.add_custom_client_modal.validate_file");
    }

    return Object.keys(errors.value).length === 0;
};

const selectFile = async () => {
    try {
        const selected = await open({
            multiple: false,
            filters: [
                {
                    name: "JAR Files",
                    extensions: ["jar"],
                },
            ],
        });

        if (selected) {
            form.filePath = selected;
            const pathParts = selected.split(/[/\\]/);
            form.fileName = pathParts[pathParts.length - 1];

            try {
                const mainClass = await invoke<string>("detect_main_class", {
                    filePath: selected,
                });
                if (mainClass) {
                    form.mainClass = mainClass;
                    addToast(t("modals.add_custom_client_modal.main_class_detected"), "success");
                }
            } catch (e) {}
        }
    } catch (error) {
        addToast(t("modals.add_custom_client_modal.file_select_failed"), "error");
    }
};

const handleSubmit = async () => {
    if (!validateForm()) {
        return;
    }

    try {
        loading.value = true;

        await invoke("add_custom_client", {
            name: form.name.trim(),
            version: form.version,
            filename: form.fileName,
            filePath: form.filePath,
            mainClass: form.mainClass.trim(),
            javaPath: form.javaPath.trim() || null,
            javaArgs: form.javaArgs.trim() || null,
            clientType: form.clientType,
        });

        Object.assign(form, {
            name: "",
            version: "",
            mainClass: "net.minecraft.client.main.Main",
            filePath: "",
            fileName: "",
            javaPath: "",
            javaArgs: "",
            clientType: "default",
        });

        emit("client-added");
        emit("close");
    } catch (err) {
        addToast(t("modals.add_custom_client_modal.add_failed", { error: err }), "error");
    } finally {
        loading.value = false;
    }
};
</script>

<template>
    <form @submit.prevent="handleSubmit" class="space-y-4">
        <div class="form-scroll-area">
            <div class="form-control">
                <label class="label">
                    <span class="label-text"
                        >{{
                            $t("modals.add_custom_client_modal.client_name")
                        }}
                        *</span
                    >
                </label>
                <input
                    v-model="form.name"
                    type="text"
                    :placeholder="
                        $t('modals.add_custom_client_modal.enter_client_name')
                    "
                    class="input input-bordered"
                    :class="{ 'input-error': errors.name }"
                />
                <label v-if="errors.name" class="label">
                    <span class="label-text-alt text-error">{{
                        errors.name
                    }}</span>
                </label>
            </div>

            <div class="form-control">
                <label class="label">
                    <span class="label-text"
                        >{{
                            $t(
                                "modals.add_custom_client_modal.minecraft_version"
                            )
                        }}
                        *</span
                    >
                </label>
                <select
                    v-model="form.version"
                    class="select select-bordered w-full"
                    :class="{ 'select-error': errors.version }"
                >
                    <option v-for="v in availableVersions" :key="v" :value="v">{{ v }}</option>
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
                            $t("modals.add_custom_client_modal.main_class")
                        }}
                        *</span
                    >
                </label>
                <input
                    v-model="form.mainClass"
                    type="text"
                    :placeholder="
                        $t(
                            'modals.add_custom_client_modal.main_class_placeholder'
                        )
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
                    <span class="label-text"
                        >{{
                            $t("modals.add_custom_client_modal.jar_file")
                        }}
                        *</span
                    >
                </label>
                <div class="flex gap-2">
                    <input
                        :value="
                            form.fileName ||
                            $t(
                                'modals.add_custom_client_modal.no_file_selected'
                            )
                        "
                        type="text"
                        :placeholder="
                            $t('modals.add_custom_client_modal.select_jar_file')
                        "
                        class="input input-bordered flex-1"
                        readonly
                        :class="{ 'input-error': errors.filePath }"
                    />
                    <button
                        type="button"
                        @click="selectFile"
                        class="btn btn-outline"
                    >
                        {{ $t("common.browse") }}
                    </button>
                </div>
                <label v-if="errors.filePath" class="label">
                    <span class="label-text-alt text-error">{{
                        errors.filePath
                    }}</span>
                </label>
                <label v-if="form.fileName" class="label">
                    <span class="label-text-alt text-success"
                        >{{ $t("modals.add_custom_client_modal.selected") }}:
                        {{ form.fileName }}</span
                    >
                </label>
            </div>

            <div class="form-control">
                <label class="label">
                    <span class="label-text">{{ $t("modals.add_custom_client_modal.client_type") }}</span>
                </label>
                <select v-model="form.clientType" class="select select-bordered w-full">
                    <option value="default">{{ $t("modals.add_custom_client_modal.client_type_vanilla") }}</option>
                    <option value="fabric">Fabric</option>
                    <option value="forge">Forge</option>
                </select>
                <label class="label">
                    <span class="label-text-alt opacity-60">{{ $t("modals.add_custom_client_modal.client_type_hint") }}</span>
                </label>
            </div>

            <div class="divider text-xs opacity-50 uppercase tracking-widest">
                {{ $t("common.advanced") }}
            </div>

            <div class="form-control" v-if="form.clientType === 'default'">
                <label class="label">
                    <span class="label-text">{{ $t("modals.add_custom_client_modal.java_path") }}</span>
                </label>
                <input
                    v-model="form.javaPath"
                    type="text"
                    placeholder="C:\Path\To\bin\java.exe"
                    class="input input-bordered"
                />
            </div>

            <div class="form-control">
                <label class="label">
                    <span class="label-text">{{ $t("modals.add_custom_client_modal.java_args") }}</span>
                </label>
                <textarea
                    v-model="form.javaArgs"
                    class="textarea textarea-bordered h-20"
                    placeholder="-Xms512M -XX:+UseG1GC"
                ></textarea>
            </div>
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
                <span
                    v-if="loading"
                    class="loading loading-spinner loading-sm"
                ></span>
                {{
                    loading
                        ? $t("modals.add_custom_client_modal.adding")
                        : $t("modals.add_custom_client_modal.add_client")
                }}
            </button>
        </div>
    </form>
</template>

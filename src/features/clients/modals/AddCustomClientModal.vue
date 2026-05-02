<script setup lang="ts">
import { ref, reactive, watch, computed, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { useToast } from "@shared/composables/useToast";
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
    fabric: ["1.21.4", "1.21.8", "1.21.11"],
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

const isDragging = ref(false);
let unlistenDrop: (() => void) | null = null;

const applyJarFile = async (filePath: string) => {
    if (!filePath.endsWith(".jar")) {
        addToast(
            t("modals.add_custom_client_modal.file_select_failed"),
            "error"
        );
        return;
    }

    form.filePath = filePath;
    const parts = filePath.split(/[/\\]/);
    form.fileName = parts[parts.length - 1];

    if (!form.name) {
        form.name = form.fileName.replace(".jar", "");
    }

    try {
        const mainClass = await invoke<string>("detect_main_class", {
            filePath,
        });
        if (mainClass) {
            form.mainClass = mainClass;
            addToast(
                t("modals.add_custom_client_modal.main_class_detected"),
                "success"
            );
        }
    } catch (e) {
        console.log("Failed to detect main class:", e);
    }
};

onMounted(async () => {
    unlistenDrop = await getCurrentWebview().onDragDropEvent(async (event) => {
        if (event.payload.type === "over") {
            isDragging.value = true;
        } else if (event.payload.type === "drop") {
            isDragging.value = false;
            const paths = event.payload.paths;
            if (paths.length > 0) {
                await applyJarFile(paths[0]);
            }
        } else {
            isDragging.value = false;
        }
    });
});

onUnmounted(() => {
    if (unlistenDrop) unlistenDrop();
});
const errors = ref<Record<string, string>>({});
const loading = ref(false);

const availableVersions = computed(() => {
    return VERSION_MAP[form.clientType as keyof typeof VERSION_MAP] || [];
});

watch(
    () => form.clientType,
    (newType) => {
        if (newType === "fabric") {
            form.mainClass = "net.fabricmc.loader.impl.launch.knot.KnotClient";
        } else if (newType === "default") {
            form.mainClass = "net.minecraft.client.main.Main";
        }

        const versions = VERSION_MAP[newType as keyof typeof VERSION_MAP];
        if (versions && versions.length > 0) {
            form.version = versions[0];
        }
    }
);

const validateForm = () => {
    errors.value = {};

    if (!form.name.trim()) {
        errors.value.name = t("modals.add_custom_client_modal.validate_name");
    }

    if (!form.version.trim()) {
        errors.value.version = t(
            "modals.add_custom_client_modal.validate_version"
        );
    }

    if (!form.mainClass.trim()) {
        errors.value.mainClass = t(
            "modals.add_custom_client_modal.validate_main_class"
        );
    }

    if (!form.filePath) {
        errors.value.filePath = t(
            "modals.add_custom_client_modal.validate_file"
        );
    }

    return Object.keys(errors.value).length === 0;
};

const selectFile = async () => {
    try {
        const selected = await open({
            multiple: false,
            filters: [{ name: "JAR Files", extensions: ["jar"] }],
        });

        if (selected) {
            await applyJarFile(selected);
        }
    } catch (error) {
        console.log("File selection cancelled or failed", error);
        addToast(
            t("modals.add_custom_client_modal.file_select_failed"),
            "error"
        );
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
        addToast(
            t("modals.add_custom_client_modal.add_failed", { error: err }),
            "error"
        );
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
                <div
                    class="border-2 border-dashed rounded-lg p-4 text-center transition-colors cursor-pointer mb-2"
                    :class="
                        isDragging
                            ? 'border-primary bg-primary/10'
                            : 'border-base-300 hover:border-primary/50'
                    "
                    @click="selectFile"
                >
                    <div v-if="!form.fileName" class="space-y-1">
                        <p class="text-sm text-base-content/60">
                            {{
                                $t(
                                    "modals.add_custom_client_modal.drop_jar_here"
                                )
                            }}
                        </p>
                        <p class="text-xs text-base-content/40">
                            {{
                                $t(
                                    "modals.add_custom_client_modal.or_click_browse"
                                )
                            }}
                        </p>
                    </div>
                    <div
                        v-else
                        class="flex items-center justify-center gap-2 text-success text-sm"
                    >
                        <span>✓</span>
                        <span class="font-medium truncate max-w-xs">{{
                            form.fileName
                        }}</span>
                    </div>
                </div>
                <label v-if="errors.filePath" class="label">
                    <span class="label-text-alt text-error">{{
                        errors.filePath
                    }}</span>
                </label>
            </div>

            <div class="form-control">
                <label class="label">
                    <span class="label-text">{{
                        $t("modals.add_custom_client_modal.client_type")
                    }}</span>
                </label>
                <select
                    v-model="form.clientType"
                    class="select select-bordered w-full"
                >
                    <option value="default">
                        {{
                            $t(
                                "modals.add_custom_client_modal.client_type_vanilla"
                            )
                        }}
                    </option>
                    <option value="fabric">Fabric</option>
                    <option value="forge">Forge</option>
                </select>
                <label class="label">
                    <span class="label-text-alt opacity-60">{{
                        $t("modals.add_custom_client_modal.client_type_hint")
                    }}</span>
                </label>
            </div>

            <div class="divider text-xs opacity-50 uppercase tracking-widest">
                {{ $t("common.advanced") }}
            </div>

            <div class="form-control" v-if="form.clientType === 'default'">
                <label class="label">
                    <span class="label-text">{{
                        $t("modals.add_custom_client_modal.java_path")
                    }}</span>
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
                    <span class="label-text">{{
                        $t("modals.add_custom_client_modal.java_args")
                    }}</span>
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

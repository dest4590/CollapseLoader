<template>
    <div class="card bg-base-200 shadow-md border border-base-300 mt-6">
        <div class="card-body">
            <div
                @click="toggleExpertMode"
                class="cursor-pointer flex items-center justify-between"
            >
                <h2 class="card-title flex items-center gap-2">
                    <Code class="w-5 h-5 text-primary" />
                    {{ t("theme.expert_css_title") }}
                </h2>
                <button class="btn btn-sm btn-ghost">
                    <ChevronDown v-if="!showExpertOptions" class="w-5 h-5" />
                    <ChevronUp v-else class="w-5 h-5" />
                    {{
                        showExpertOptions
                            ? t("theme.hide_expert")
                            : t("theme.show_expert")
                    }}
                </button>
            </div>

            <transition
                name="expert-fade"
                @before-enter="expertAnimationActive = true"
                @after-leave="expertAnimationActive = false"
            >
                <div v-if="showExpertOptions" class="mt-4">
                    <div
                        class="bg-warning/10 border border-warning/20 rounded-lg p-4 mb-4"
                    >
                        <div class="flex items-start gap-2">
                            <HelpCircle
                                class="w-5 h-5 text-warning shrink-0 mt-0.5"
                            />
                            <p class="text-sm text-warning">
                                {{ t("theme.expert_warning") }}
                            </p>
                        </div>
                    </div>

                    <div class="flex items-center justify-between mb-2">
                        <label class="flex items-center gap-2">
                            <input
                                type="checkbox"
                                class="checkbox"
                                :checked="enableCustomCss"
                                @change="
                                    $emit(
                                        'update:enableCustomCss',
                                        ($event.target as HTMLInputElement)
                                            ?.checked ?? false
                                    )
                                "
                            />
                            <span>{{ t("theme.enable_custom_css") }}</span>
                        </label>
                    </div>

                    <div class="flex flex-col gap-2 mb-4">
                        <label class="font-medium mb-1">{{
                            t("theme.available_classes_label")
                        }}</label>
                        <div class="flex flex-wrap gap-2">
                            <span
                                class="bg-base-300 text-xs px-3 py-1 rounded-full font-mono text-base-content/80 border border-base-200 tooltip tooltip-right cursor-pointer"
                                :data-tip="t('theme.tooltip_client_card')"
                                @click="$emit('add-example', '.client-card')"
                                >client-card</span
                            >
                            <span
                                class="bg-base-300 text-xs px-3 py-1 rounded-full font-mono text-base-content/80 border border-base-200 tooltip tooltip-right cursor-pointer"
                                :data-tip="t('theme.tooltip_sidebar_btn')"
                                @click="$emit('add-example', '.sidebar-btn')"
                                >sidebar-btn</span
                            >
                            <span
                                class="bg-base-300 text-xs px-3 py-1 rounded-full font-mono text-base-content/80 border border-base-200 tooltip tooltip-right cursor-pointer"
                                :data-tip="
                                    t('theme.tooltip_launch_download_btn')
                                "
                                @click="
                                    $emit(
                                        'add-example',
                                        '.launch-btn, .download-btn'
                                    )
                                "
                                >download-btn | launch-btn</span
                            >
                        </div>
                    </div>

                    <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 mt-4">
                        <div>
                            <label class="block mb-2 font-medium">{{
                                t("theme.custom_css_label")
                            }}</label>
                            <VueMonacoEditor
                                :value="customCss"
                                @update:value="
                                    $emit('update:customCss', $event)
                                "
                                language="css"
                                :theme="
                                    selectedTheme === 'dark' ? 'vs-dark' : 'vs'
                                "
                                :options="{
                                    readOnly: !enableCustomCss,
                                    minimap: { enabled: false },
                                    fontSize: 14,
                                    lineNumbers: 'on',
                                    wordWrap: 'on',
                                    automaticLayout: true,
                                    scrollBeyondLastLine: false,
                                }"
                                style="
                                    height: 300px;
                                    border-radius: 0.5rem;
                                    border: 1px solid rgba(255, 255, 255, 0.1);
                                "
                            />
                        </div>
                    </div>

                    <div class="flex gap-2 mt-4">
                        <button
                            class="btn btn-primary btn-sm flex items-center gap-2"
                            @click="$emit('open-export-modal')"
                        >
                            <ClipboardCopy class="w-4 h-4" />
                            {{ t("theme.export_css_btn") }}
                        </button>
                        <button
                            class="btn btn-secondary btn-sm flex items-center gap-2"
                            @click="$emit('open-import-modal')"
                        >
                            <ClipboardPaste class="w-4 h-4" />
                            {{ t("theme.import_css_btn") }}
                        </button>
                    </div>

                    <div class="mt-6">
                        <h3 class="font-medium text-sm mb-3">
                            {{ t("theme.css_examples_title") }}
                        </h3>
                        <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                            <div
                                v-for="(example, index) in cssExamples"
                                :key="index"
                                class="card shadow-md border border-base-300"
                            >
                                <div class="card-body p-4">
                                    <h4 class="card-title text-sm">
                                        {{ example.title }}
                                    </h4>
                                    <pre
                                        class="text-xs bg-base-300 p-2 rounded overflow-x-auto mt-2"
                                    ><code>{{ example.code }}</code></pre>
                                    <button
                                        @click="
                                            $emit(
                                                'insert-example',
                                                example.code
                                            )
                                        "
                                        class="btn btn-xs btn-primary mt-2"
                                        :disabled="!enableCustomCss"
                                    >
                                        <ClipboardPaste class="w-4 h-4" />
                                        {{ t("theme.insert_example") }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </transition>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { useI18n } from "vue-i18n";
import {
    Code,
    HelpCircle,
    ChevronDown,
    ChevronUp,
    ClipboardCopy,
    ClipboardPaste,
} from "@lucide/vue";
import { VueMonacoEditor } from "@guolao/vue-monaco-editor";

const { t } = useI18n();

defineProps<{
    enableCustomCss: boolean;
    customCss: string;
    selectedTheme: string;
    cssExamples: { title: string; code: string }[];
}>();

defineEmits<{
    "update:enableCustomCss": [value: boolean];
    "update:customCss": [value: string];
    "add-example": [className: string];
    "insert-example": [code: string];
    "open-export-modal": [];
    "open-import-modal": [];
}>();

const showExpertOptions = ref(false);
const expertAnimationActive = ref(false);

const toggleExpertMode = () => {
    expertAnimationActive.value = true;
    showExpertOptions.value = !showExpertOptions.value;

    setTimeout(() => {
        expertAnimationActive.value = false;
    }, 300);
};
</script>

<style scoped>
.expert-fade-enter-active,
.expert-fade-leave-active {
    transition:
        opacity 0.3s cubic-bezier(0.4, 0, 0.2, 1),
        transform 0.3s cubic-bezier(0.4, 0, 0.2, 1),
        max-height 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    overflow: hidden;
    max-height: 2000px;
}

.expert-fade-enter-from,
.expert-fade-leave-to {
    opacity: 0;
    transform: translateY(12px) scale(0.98);
    max-height: 0;
}
</style>

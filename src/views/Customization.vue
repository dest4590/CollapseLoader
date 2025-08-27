<template>
    <div class="container mx-auto mt-4">
        <div class="card bg-base-200 shadow-md border border-base-300 mb-6">
            <div class="card-body">
                <div class="flex flex-col md:flex-row md:items-center md:justify-between gap-4">
                    <h1 class="card-title text-2xl flex items-center gap-3">
                        <Save class="w-6 h-6 text-primary" />
                        {{ t('theme.preset') }}
                    </h1>

                    <div class="flex flex-wrap items-center gap-2 justify-end">
                        <div class="flex gap-2">
                            <button class="btn btn-primary btn-sm flex items-center gap-2"
                                @click="importPresetFromClipboard">
                                <ClipboardPaste class="w-4 h-4" />
                                <span class="hidden sm:inline">{{ t('theme.import_preset') }}</span>
                            </button>

                            <button class="btn btn-secondary btn-sm flex items-center gap-2"
                                @click="exportPresetToClipboard">
                                <ClipboardCopy class="w-4 h-4" />
                                <span class="hidden sm:inline">{{ t('theme.export_preset') }}</span>
                            </button>

                            <button class="btn btn-outline btn-sm flex items-center gap-2" @click="resetStyles">
                                <RotateCcw class="w-4 h-4" />
                                <span class="hidden sm:inline">{{ t('theme.reset_button') }}</span>
                            </button>

                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div key="theme" class="grid grid-cols-1 lg:grid-cols-12 gap-8">
            <div class="card bg-base-200 shadow-md border border-base-300 lg:col-span-4 p-6">
                <h2 class="text-xl font-semibold mb-4">{{ t('theme.select_theme') }}</h2>
                <p class="text-base-content/70 mb-4">{{ t('theme.description') }}</p>

                <div class="flex flex-col gap-4">
                    <button v-for="theme in themes" :key="theme" @click="changeTheme(theme)"
                        class="btn border flex items-center justify-between px-6 py-3" :class="{
                            'border-primary/50 bg-primary/10': selectedTheme === theme,
                            'border-base-content/10': selectedTheme !== theme
                        }">
                        <div class="flex items-center gap-2">
                            <Sun v-if="theme === 'light'" class="w-5 h-5 text-amber-400" />
                            <Moon v-else class="w-5 h-5 text-indigo-400" />
                            <span class="font-medium capitalize">{{ t(`theme.${theme}`) }}</span>
                        </div>
                        <div v-if="selectedTheme === theme" class="badge badge-primary">
                            {{ t('theme.selected') }}
                        </div>
                    </button>
                </div>
            </div>

            <div class="lg:col-span-8">
                <div class="card bg-base-200 shadow-md border border-base-300 mb-6">
                    <div class="card-body">
                        <h2 class="card-title flex items-center gap-2">
                            <LayoutPanelLeft class="w-5 h-5 text-primary" />
                            {{ t('theme.card_configure_title') }}
                        </h2>

                        <div class="grid grid-cols-1 md:grid-cols-2 gap-8 mt-4">
                            <div>
                                <div class="space-y-6">
                                    <div>
                                        <div class="flex items-center justify-between mb-2">
                                            <label class="text-sm font-medium flex items-center gap-1">
                                                <Radius class="w-4 h-4" />
                                                {{ t('theme.card_radius') }}
                                            </label>
                                            <span class="text-xs text-base-content/70">
                                                {{ getRadiusLabel(radiusIndex) }}
                                            </span>
                                        </div>
                                        <AnimatedSlider v-model="radiusIndex" :min="0" :max="radiusOptions.length - 1"
                                            @update:modelValue="handleRadiusChange" />
                                    </div>

                                    <div>
                                        <div class="flex items-center justify-between mb-2">
                                            <label class="text-sm font-medium flex items-center gap-1">
                                                <Square class="w-4 h-4" />
                                                {{ t('theme.card_shadow_label') }}
                                            </label>
                                            <span class="text-xs text-base-content/70">
                                                {{ getShadowLabel(shadowIndex) }}
                                            </span>
                                        </div>
                                        <AnimatedSlider v-model="shadowIndex" :min="0" :max="shadowOptions.length - 1"
                                            @update:modelValue="handleShadowChange" />
                                    </div>

                                    <div>
                                        <div class="flex items-center justify-between mb-2">
                                            <label class="text-sm font-medium flex items-center gap-1">
                                                <GripVertical class="w-4 h-4" />
                                                {{ t('theme.card_padding_label') }}
                                            </label>
                                            <span class="text-xs text-base-content/70">
                                                {{ getPaddingLabel(paddingIndex) }}
                                            </span>
                                        </div>
                                        <AnimatedSlider v-model="paddingIndex" :min="0" :max="paddingOptions.length - 1"
                                            @update:modelValue="handlePaddingChange" />
                                    </div>
                                    <div>
                                        <div class="flex items-center justify-between mb-2">
                                            <label class="text-sm font-medium flex items-center gap-1">
                                                <Radius class="w-4 h-4" />
                                                {{ t('theme.global_radius') }}
                                            </label>
                                            <span class="text-xs text-base-content/70">
                                                {{ getRadiusLabel(globalRadiusIndex) }}
                                            </span>
                                        </div>
                                        <AnimatedSlider v-model="globalRadiusIndex" :min="0"
                                            :max="radiusOptions.length - 1"
                                            @update:modelValue="handleGlobalRadiusChange" class="mb-7" />
                                    </div>
                                </div>

                                <div class="flex items-center justify-between">
                                    <label class="text-sm font-medium flex items-center gap-2">
                                        <GripVertical class="w-4 h-4" />
                                        {{ t('theme.remove_animations') }}
                                    </label>
                                    <input type="checkbox" class="toggle toggle-primary" v-model="reduceMotion"
                                        @change="handleReduceMotionChange" />
                                </div>
                            </div>

                            <div>
                                <h3 class="font-medium text-base mb-3">
                                    {{ t('theme.card_preview') }}
                                </h3>
                                <p class="text-sm text-base-content/70 mb-4">
                                    {{ t('theme.card_preview_desc') }}
                                </p>
                                <div class="card-preview">
                                    <ClientCard :client="demoClient" :is-client-running="() => false"
                                        :is-client-installing="() => false" :installation-status="new Map()"
                                        :is-requirements-in-progress="false" :is-favorite="true" />
                                </div>
                            </div>
                        </div>
                    </div>
                </div>

                <div class="card bg-base-200 shadow-md border border-base-300">
                    <div class="card-body p-6">
                        <h2 class="card-title flex items-center gap-3 text-2xl font-bold text-base-content">
                            <svg class="w-6 h-6 text-primary" fill="none" stroke="currentColor" viewBox="0 0 24 24"
                                xmlns="http://www.w3.org/2000/svg">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                    d="M7 21a4 4 0 01-4-4V5a2 2 0 012-2h4a2 2 0 012 2v12a4 4 0 01-4 4zm0 0h12a2 2 0 002-2v-4a2 2 0 00-2-2h-2.343M11 7.343l1.657-1.657a2 2 0 012.828 0l2.829 2.829a2 2 0 010 2.828l-8.486 8.485M7 17h.01">
                                </path>
                            </svg>
                            {{ t('theme.colors') }}
                        </h2>

                        <div class="mb-8">
                            <h3 class="text-xl font-semibold mb-4 text-base-content">{{ t('theme.base_colors') }}</h3>
                            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{ t('theme.base100')
                                        }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="base100"
                                        @input="handleColorInput('base100', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{ t('theme.base200')
                                        }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="base200"
                                        @input="handleColorInput('base200', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{ t('theme.base300')
                                        }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="base300"
                                        @input="handleColorInput('base300', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{
                                        t('theme.base_content') }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="baseContent"
                                        @input="handleColorInput('baseContent', ($event.target as HTMLInputElement).value)" />
                                </div>
                            </div>
                        </div>

                        <div class="mb-8">
                            <h3 class="text-xl font-semibold mb-4 text-base-content">{{
                                t('theme.primary_secondary_accent') }}</h3>
                            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4">
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{
                                        t('theme.primary_color_override') }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="primaryColor"
                                        @input="handleColorInput('primaryColorOverride', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{
                                        t('theme.primary_content') }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="primaryContent"
                                        @input="handleColorInput('primaryContent', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{ t('theme.secondary')
                                        }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="secondary"
                                        @input="handleColorInput('secondary', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{
                                        t('theme.secondary_content') }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="secondaryContent"
                                        @input="handleColorInput('secondaryContent', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{ t('theme.accent')
                                        }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="accent"
                                        @input="handleColorInput('accent', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{
                                        t('theme.accent_content') }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="accentContent"
                                        @input="handleColorInput('accentContent', ($event.target as HTMLInputElement).value)" />
                                </div>
                            </div>
                        </div>

                        <div>
                            <h3 class="text-xl font-semibold mb-4 text-base-content">{{ t('theme.semantic_colors') }}
                            </h3>
                            <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{ t('theme.neutral')
                                        }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="neutral"
                                        @input="handleColorInput('neutral', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{
                                        t('theme.neutral_content') }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="neutralContent"
                                        @input="handleColorInput('neutralContent', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{ t('theme.info')
                                        }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="info"
                                        @input="handleColorInput('info', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{
                                        t('theme.info_content') }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="infoContent"
                                        @input="handleColorInput('infoContent', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{ t('theme.success')
                                        }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="success"
                                        @input="handleColorInput('success', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{
                                        t('theme.success_content') }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="successContent"
                                        @input="handleColorInput('successContent', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{ t('theme.warning')
                                        }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="warning"
                                        @input="handleColorInput('warning', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{
                                        t('theme.warning_content') }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="warningContent"
                                        @input="handleColorInput('warningContent', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{ t('theme.error')
                                        }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="error"
                                        @input="handleColorInput('error', ($event.target as HTMLInputElement).value)" />
                                </div>
                                <div class="form-control">
                                    <label class="label text-sm font-medium text-base-content">{{
                                        t('theme.error_content') }}</label>
                                    <input type="color"
                                        class="input input-bordered w-full h-10 p-0 rounded-md border-base-300 focus:border-primary focus:ring-1 focus:ring-primary"
                                        :value="errorContent"
                                        @input="handleColorInput('errorContent', ($event.target as HTMLInputElement).value)" />
                                </div>
                            </div>
                        </div>
                    </div>
                </div>


                <div class="card bg-base-200 shadow-md border border-base-300 mt-6">
                    <div class="card-body">
                        <div @click="toggleExpertMode" class="cursor-pointer flex items-center justify-between">
                            <h2 class="card-title flex items-center gap-2">
                                <Code class="w-5 h-5 text-primary" />
                                {{ t('theme.expert_css_title') }}
                            </h2>
                            <button class="btn btn-sm btn-ghost">
                                <ChevronDown v-if="!showExpertOptions" class="w-5 h-5" />
                                <ChevronUp v-else class="w-5 h-5" />
                                {{ showExpertOptions ? t('theme.hide_expert') : t('theme.show_expert') }}
                            </button>
                        </div>

                        <transition name="expert-fade" @before-enter="expertAnimationActive = true"
                            @after-leave="expertAnimationActive = false">
                            <div v-if="showExpertOptions" class="mt-4">
                                <div class="bg-warning/10 border border-warning/20 rounded-lg p-4 mb-4">
                                    <div class="flex items-start gap-2">
                                        <HelpCircle class="w-5 h-5 text-warning flex-shrink-0 mt-0.5" />
                                        <p class="text-sm text-warning">
                                            {{ t('theme.expert_warning') }}
                                        </p>
                                    </div>
                                </div>

                                <div class="flex items-center justify-between mb-2">
                                    <label class="flex items-center gap-2">
                                        <input type="checkbox" class="checkbox" v-model="enableCustomCSS"
                                            @change="handleEnableCustomCSS(($event.target as HTMLInputElement)?.checked ?? false)" />
                                        <span>{{ t('theme.enable_custom_css') }}</span>
                                    </label>
                                </div>
                                <div class="flex flex-col gap-2 mb-4">
                                    <label class="font-medium mb-1">{{ t('theme.available_classes_label') }}</label>
                                    <div class="flex flex-wrap gap-2">
                                        <span
                                            class="bg-base-300 text-xs px-3 py-1 rounded-full font-mono text-base-content/80 border border-base-200 tooltip tooltip-right cursor-pointer"
                                            :data-tip="t('theme.tooltip_client_card')"
                                            @click="addExample('.client-card')">
                                            client-card
                                        </span>
                                        <span
                                            class="bg-base-300 text-xs px-3 py-1 rounded-full font-mono text-base-content/80 border border-base-200 tooltip tooltip-right cursor-pointer"
                                            :data-tip="t('theme.tooltip_sidebar_btn')"
                                            @click="addExample('.sidebar-btn')">
                                            sidebar-btn
                                        </span>
                                        <span
                                            class="bg-base-300 text-xs px-3 py-1 rounded-full font-mono text-base-content/80 border border-base-200 tooltip tooltip-right cursor-pointer"
                                            :data-tip="t('theme.tooltip_launch_download_btn')"
                                            @click="addExample('.launch-btn, .download-btn')">
                                            download-btn | launch-btn
                                        </span>
                                    </div>
                                </div>

                                <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 mt-4">
                                    <div>
                                        <label class="block mb-2 font-medium">{{ t('theme.custom_css_label')
                                            }}</label>
                                        <VueMonacoEditor v-model:value="customCSS" language="css"
                                            :theme="selectedTheme === 'dark' ? 'vs-dark' : 'vs'" :options="{
                                                readOnly: !enableCustomCSS,
                                                minimap: { enabled: false },
                                                fontSize: 14,
                                                lineNumbers: 'on',
                                                wordWrap: 'on',
                                                automaticLayout: true,
                                                scrollBeyondLastLine: false
                                            }"
                                            style="height: 300px; border-radius: 0.5rem; border: 1px solid rgba(255, 255, 255, 0.1);" />
                                    </div>
                                </div>
                                <div class="flex gap-2 mt-4">
                                    <button class="btn btn-primary btn-sm flex items-center gap-2"
                                        @click="openExportModal">
                                        <ClipboardCopy class="w-4 h-4" />
                                        {{ t('theme.export_css_btn') }}
                                    </button>
                                    <button class="btn btn-secondary btn-sm flex items-center gap-2"
                                        @click="openImportModal">
                                        <ClipboardPaste class="w-4 h-4" />
                                        {{ t('theme.import_css_btn') }}
                                    </button>
                                </div>
                                <div class="mt-6">
                                    <h3 class="font-medium text-sm mb-3">{{ t('theme.css_examples_title') }}</h3>
                                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                                        <div v-for="(example, index) in cssExamples" :key="index"
                                            class="card shadow-md border border-base-300">
                                            <div class="card-body p-4">
                                                <h4 class="card-title text-sm">{{ example.title }}</h4>
                                                <pre
                                                    class="text-xs bg-base-300 p-2 rounded overflow-x-auto mt-2"><code>{{ example.code }}</code></pre>
                                                <button @click="insertExample(example.code)"
                                                    class="btn btn-xs btn-primary mt-2" :disabled="!enableCustomCSS">
                                                    <ClipboardPaste class="w-4 h-4" />
                                                    {{ t('theme.insert_example') }}
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </transition>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, watch, reactive, type Ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { ClipboardCopy, ClipboardPaste, Save } from 'lucide-vue-next';
import { invoke } from '@tauri-apps/api/core';
import { useToast } from '../services/toastService';
import { themeService } from '../services/themeService';
import ClientCard from '../components/features/clients/ClientCard.vue';
import AnimatedSlider from '../components/ui/AnimatedSlider.vue';
import {
    Moon,
    Sun,
    RotateCcw,
    LayoutPanelLeft,
    Radius,
    Square,
    GripVertical,
    Code,
    HelpCircle,
    ChevronDown,
    ChevronUp,
} from 'lucide-vue-next';
import { VueMonacoEditor } from '@guolao/vue-monaco-editor'
import ImportExportCssModal from '../components/modals/ImportExportCssModal.vue';
import { useModal } from '../services/modalService';

const i18n = useI18n();
const { t } = i18n;
const { addToast } = useToast();
const { showModal } = useModal();

const themes = ['dark', 'light'];
const selectedTheme = ref(document.documentElement.getAttribute('data-theme') || 'dark');
const showExpertOptions = ref(false);
const expertAnimationActive = ref(false);

const customCSS = ref(themeService.presetSettings.customCSS);
const enableCustomCSS = ref(themeService.presetSettings.enableCustomCSS);

watch(customCSS, (val) => {
    if (enableCustomCSS.value) {
        themeService.updatePresetSettings({ customCSS: val });
    }
});

watch(enableCustomCSS, (val) => {
    themeService.updatePresetSettings({ enableCustomCSS: val });
});

const demoClient = reactive({
    id: 0,
    name: 'Demo Client',
    version: '1.0.0',
    meta: {
        size: '128',
        installed: true,
        category: 'demo'
    },
    working: true,
    required_files: [],
    url: '',
    filename: 'demo.zip',
    description: 'This is a demo client for preview purposes',
    insecure: false,
    md5_hash: 'demo-md5-hash',
    main_class: 'com.demo.Main',
    show: true,
    launches: 1337,
    downloads: 1337
});

const radiusOptions = [
    { value: '0', label: 'None' },
    { value: '0.375rem', label: 'Small' },
    { value: '0.5rem', label: 'Medium' },
    { value: '0.75rem', label: 'Large' },
    { value: '1rem', label: 'Extra Large' }
];

const shadowOptions = [
    { value: 'none', label: 'None' },
    { value: '0 1px 3px rgba(0,0,0,0.1)', label: 'Small' },
    { value: '0 4px 6px -1px rgba(0,0,0,0.1)', label: 'Medium' },
    { value: '0 10px 15px -3px rgba(0,0,0,0.1)', label: 'Large' },
    { value: '0 20px 25px -5px rgba(0,0,0,0.1)', label: 'Extra Large' }
];

const paddingOptions = [
    { value: '0.5rem', label: 'Small' },
    { value: '1rem', label: 'Medium' },
    { value: '1.5rem', label: 'Large' },
    { value: '2rem', label: 'Extra Large' }
];

const getRadiusLabel = (index: number) => {
    return t(`theme.card_radius_${radiusOptions[index]?.value === '0' ? 'none' :
        index === 1 ? 'sm' :
            index === 2 ? 'md' :
                index === 3 ? 'lg' :
                    index === 4 ? 'xl' : 'md'}`);
};

const getShadowLabel = (index: number) => {
    return t(`theme.card_shadow_${index === 0 ? 'none' :
        index === 1 ? 'sm' :
            index === 2 ? 'md' :
                index === 3 ? 'lg' :
                    index === 4 ? 'xl' : 'md'}`);
};

const getPaddingLabel = (index: number) => {
    return t(`theme.card_padding_${index === 0 ? 'sm' :
        index === 1 ? 'md' :
            index === 2 ? 'lg' :
                index === 3 ? 'xl' : 'md'}`);
};

const cssExamples = [
    {
        title: t('theme.example_1'),
        code: `.client-card {
  backdrop-filter: blur(5px);
  background-color: rgba(0, 0, 0, 0.3);
  border: 1px solid rgba(255, 255, 255, 0.1);
}`
    },
];

const findOptionIndex = (options: { value: string }[], value: string | undefined, defaultIndex: number): number => {
    if (!value) return defaultIndex;
    const index = options.findIndex(opt => opt.value === value);
    return index >= 0 ? index : defaultIndex;
};

const radiusIndex = ref(findOptionIndex(radiusOptions, themeService.presetSettings.borderRadius, 2));
const shadowIndex = ref(findOptionIndex(shadowOptions, themeService.presetSettings.shadow, 2));
const paddingIndex = ref(findOptionIndex(paddingOptions, themeService.presetSettings.padding, 1));

const globalRadiusIndex = ref(findOptionIndex(radiusOptions, themeService.presetSettings.globalRadius, 2));
const primaryColor = ref(themeService.presetSettings.primaryColorOverride || '#000000');
const reduceMotion = ref<boolean>(themeService.presetSettings.reduceMotion);

const base100 = ref<string>(themeService.presetSettings.base100 || '');
const base200 = ref<string>(themeService.presetSettings.base200 || '');
const base300 = ref<string>(themeService.presetSettings.base300 || '');
const baseContent = ref<string>(themeService.presetSettings.baseContent || '');

const primaryContent = ref<string>(themeService.presetSettings.primaryContent || '');
const secondary = ref<string>(themeService.presetSettings.secondary || '');
const secondaryContent = ref<string>(themeService.presetSettings.secondaryContent || '');
const accent = ref<string>(themeService.presetSettings.accent || '');
const accentContent = ref<string>(themeService.presetSettings.accentContent || '');

const neutral = ref<string>(themeService.presetSettings.neutral || '');
const neutralContent = ref<string>(themeService.presetSettings.neutralContent || '');
const info = ref<string>(themeService.presetSettings.info || '');
const infoContent = ref<string>(themeService.presetSettings.infoContent || '');
const success = ref<string>(themeService.presetSettings.success || '');
const successContent = ref<string>(themeService.presetSettings.successContent || '');
const warning = ref<string>(themeService.presetSettings.warning || '');
const warningContent = ref<string>(themeService.presetSettings.warningContent || '');
const error = ref<string>(themeService.presetSettings.error || '');
const errorContent = ref<string>(themeService.presetSettings.errorContent || '');

const _colorRefs: Record<string, Ref<string>> = {
    base100,
    base200,
    base300,
    baseContent,
    primaryColorOverride: primaryColor,
    primaryContent,
    secondary,
    secondaryContent,
    accent,
    accentContent,
    neutral,
    neutralContent,
    info,
    infoContent,
    success,
    successContent,
    warning,
    warningContent,
    error,
    errorContent
};

const handleColorInput = (settingKey: string, color: string): void => {
    const r = _colorRefs[settingKey];
    if (r) {
        r.value = color;
    }

    const payload: Record<string, string | null> = {};
    payload[settingKey] = color && color.trim().length > 0 ? color : null;
    themeService.updatePresetSettings(payload);
};

watch(
    [
        base100, base200, base300, baseContent,
        primaryContent, secondary, secondaryContent, accent, accentContent,
        neutral, neutralContent, info, infoContent, success, successContent,
        warning, warningContent, error, errorContent
    ],
    () => {
        themeService.updatePresetSettings({
            base100: base100.value ? base100.value : null,
            base200: base200.value ? base200.value : null,
            base300: base300.value ? base300.value : null,
            baseContent: baseContent.value ? baseContent.value : null,

            primaryContent: primaryContent.value ? primaryContent.value : null,
            secondary: secondary.value ? secondary.value : null,
            secondaryContent: secondaryContent.value ? secondaryContent.value : null,
            accent: accent.value ? accent.value : null,
            accentContent: accentContent.value ? accentContent.value : null,

            neutral: neutral.value ? neutral.value : null,
            neutralContent: neutralContent.value ? neutralContent.value : null,
            info: info.value ? info.value : null,
            infoContent: infoContent.value ? infoContent.value : null,
            success: success.value ? success.value : null,
            successContent: successContent.value ? successContent.value : null,
            warning: warning.value ? warning.value : null,
            warningContent: warningContent.value ? warningContent.value : null,
            error: error.value ? error.value : null,
            errorContent: errorContent.value ? errorContent.value : null,
        });
    },
    { deep: false }
);

const changeTheme = async (theme: string) => {
    try {
        selectedTheme.value = theme;
        document.documentElement.setAttribute('data-theme', theme);

        const currentSettings = await invoke('get_settings');

        const settingsObj = typeof currentSettings === 'object' && currentSettings !== null
            ? { ...currentSettings }
            : {};

        const inputSettings = {
            ...settingsObj,
            theme: { value: theme, show: false }
        };
        if ('config_path' in inputSettings) {
            delete inputSettings.config_path;
        }

        await invoke('save_settings', { inputSettings: inputSettings });

        addToast(t('theme.change_success'), 'success');
    } catch (error) {
        console.error('Failed to save theme:', error);
        addToast(t('theme.save_failed', { error }), 'error');
    }
};

const handleRadiusChange = () => {
    const index = radiusIndex.value >= 0 && radiusIndex.value < radiusOptions.length ? radiusIndex.value : 2;
    themeService.updatePresetSettings({
        borderRadius: radiusOptions[index].value
    });
};

const handleShadowChange = () => {
    const index = shadowIndex.value >= 0 && shadowIndex.value < shadowOptions.length ? shadowIndex.value : 2;
    themeService.updatePresetSettings({
        shadow: shadowOptions[index].value
    });
};

const handlePaddingChange = () => {
    const index = paddingIndex.value >= 0 && paddingIndex.value < paddingOptions.length ? paddingIndex.value : 1;
    themeService.updatePresetSettings({
        padding: paddingOptions[index].value
    });
};

const handleGlobalRadiusChange = () => {
    const idx = globalRadiusIndex.value >= 0 && globalRadiusIndex.value < radiusOptions.length ? globalRadiusIndex.value : 2;
    themeService.updatePresetSettings({ globalRadius: radiusOptions[idx].value });
};

const handleReduceMotionChange = () => {
    themeService.updatePresetSettings({ reduceMotion: !!reduceMotion.value });
};

const exportPresetToClipboard = async () => {
    try {
        const json = themeService.exportPreset();
        await navigator.clipboard.writeText(json);
        addToast(t('theme.export_success'), 'success');
    } catch (e) {
        addToast(t('theme.export_failed'), 'error');
    }
};

const importPresetFromClipboard = async () => {
    try {
        const txt = await navigator.clipboard.readText();
        if (!txt || txt.trim().length === 0) {
            addToast(t('theme.import_empty'), 'warning');
            return;
        }
        if (txt.length > 20000) {
            addToast(t('theme.import_too_large'), 'error');
            return;
        }

        themeService.importPreset(txt);

        radiusIndex.value = findOptionIndex(radiusOptions, themeService.presetSettings.borderRadius, 2);
        shadowIndex.value = findOptionIndex(shadowOptions, themeService.presetSettings.shadow, 2);
        paddingIndex.value = findOptionIndex(paddingOptions, themeService.presetSettings.padding, 1);
        globalRadiusIndex.value = findOptionIndex(radiusOptions, themeService.presetSettings.globalRadius, 2);
        primaryColor.value = themeService.presetSettings.primaryColorOverride || '#000000';
        reduceMotion.value = themeService.presetSettings.reduceMotion;

        addToast(t('theme.import_success'), 'success');
    } catch (e) {
        console.error('Import preset error:', e);
        addToast(t('theme.import_invalid'), 'error');
    }
};

const toggleExpertMode = () => {
    expertAnimationActive.value = true;
    showExpertOptions.value = !showExpertOptions.value;

    setTimeout(() => {
        expertAnimationActive.value = false;
    }, 300);
};

const handleEnableCustomCSS = (val: boolean) => {
    enableCustomCSS.value = val;
};

const insertExample = (code: string) => {
    if (!enableCustomCSS.value) {
        addToast(t('theme.enable_custom_css_first'), 'warning');
        return;
    }

    customCSS.value = customCSS.value
        ? `${customCSS.value.trim()}\n\n${code}`
        : code;

    applyCustomCSS();
    addToast(t('theme.example_inserted'), 'success');
};

const addExample = (className: string) => {
    if (!enableCustomCSS.value) {
        addToast(t('theme.enable_custom_css_first'), 'warning');
        return;
    }

    const exampleCode = `${className} {\n  \n}`;
    customCSS.value = customCSS.value
        ? `${customCSS.value.trim()}\n\n${exampleCode}`
        : exampleCode;

    applyCustomCSS();
};

const handleKeyDown = (event: KeyboardEvent) => {
    if ((event.ctrlKey || event.metaKey) && event.key === 's') {
        event.preventDefault();
        applyCustomCSS();
    }
};

const applyCustomCSS = () => {
    themeService.updatePresetSettings({
        customCSS: customCSS.value,
        enableCustomCSS: enableCustomCSS.value
    });
};

const resetStyles = () => {
    themeService.resetPresetSettings();

    const newRadiusIndex = radiusOptions.findIndex(opt => opt.value === themeService.presetSettings.borderRadius);
    const newShadowIndex = shadowOptions.findIndex(opt => opt.value === themeService.presetSettings.shadow);
    const newPaddingIndex = paddingOptions.findIndex(opt => opt.value === themeService.presetSettings.padding);

    radiusIndex.value = newRadiusIndex >= 0 ? newRadiusIndex : 2;
    shadowIndex.value = newShadowIndex >= 0 ? newShadowIndex : 2;
    paddingIndex.value = newPaddingIndex >= 0 ? newPaddingIndex : 1;

    customCSS.value = themeService.presetSettings.customCSS;
    enableCustomCSS.value = themeService.presetSettings.enableCustomCSS;

    base100.value = themeService.presetSettings.base100 || '';
    base200.value = themeService.presetSettings.base200 || '';
    base300.value = themeService.presetSettings.base300 || '';
    baseContent.value = themeService.presetSettings.baseContent || '';

    primaryContent.value = themeService.presetSettings.primaryContent || '';
    secondary.value = themeService.presetSettings.secondary || '';
    secondaryContent.value = themeService.presetSettings.secondaryContent || '';
    accent.value = themeService.presetSettings.accent || '';
    accentContent.value = themeService.presetSettings.accentContent || '';

    neutral.value = themeService.presetSettings.neutral || '';
    neutralContent.value = themeService.presetSettings.neutralContent || '';
    info.value = themeService.presetSettings.info || '';
    infoContent.value = themeService.presetSettings.infoContent || '';
    success.value = themeService.presetSettings.success || '';
    successContent.value = themeService.presetSettings.successContent || '';
    warning.value = themeService.presetSettings.warning || '';
    warningContent.value = themeService.presetSettings.warningContent || '';
    error.value = themeService.presetSettings.error || '';
    errorContent.value = themeService.presetSettings.errorContent || '';

    primaryColor.value = themeService.presetSettings.primaryColorOverride || '#000000';
    reduceMotion.value = !!themeService.presetSettings.reduceMotion;
};

const openExportModal = async () => {
    showModal(
        'export-css',
        ImportExportCssModal,
        { title: t('theme.export_css_title') },
        { mode: 'export', css: customCSS.value },
        {}
    );
};

const openImportModal = () => {
    showModal(
        'import-css',
        ImportExportCssModal,
        { title: t('theme.import_css_title') },
        { mode: 'import' },
        {
            import: (css: string) => {
                if (/script|@import|url\(|expression|<|>|javascript:/i.test(css)) {
                    addToast(t('theme.import_invalid'), 'error');
                    return;
                }
                customCSS.value = css;
                applyCustomCSS();
            }
        }
    );
};


onMounted(() => {
    document.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
    document.removeEventListener('keydown', handleKeyDown);
});
</script>

<style scoped>
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

.expert-fade-enter-active,
.expert-fade-leave-active {
    transition: opacity 0.3s cubic-bezier(.4, 0, .2, 1),
        transform 0.3s cubic-bezier(.4, 0, .2, 1),
        max-height 0.3s cubic-bezier(.4, 0, .2, 1);
    overflow: hidden;
    max-height: 2000px;
}

.expert-fade-enter-from,
.expert-fade-leave-to {
    opacity: 0;
    transform: translateY(12px) scale(0.98);
    max-height: 0;
}

textarea.textarea-bordered {
    font-family: 'Fira Code', 'Menlo', 'Monaco', 'Courier New', monospace;
    line-height: 1.5;
    tab-size: 2;
}
</style>
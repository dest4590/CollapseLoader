<template>
    <div class="container mx-auto mt-4">
        <h1 class="text-2xl font-bold mb-6">{{ t('theme.title') }}</h1>

        <div class="grid grid-cols-1 lg:grid-cols-12 gap-8">
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

                                    <button class="btn btn-outline btn-sm" @click="resetCardStyles">
                                        <RotateCcw class="w-4 h-4 mr-1" />
                                        {{ t('theme.reset_button') }}
                                    </button>
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
                                        <label class="block mb-2 font-medium">{{ t('theme.custom_css_label') }}</label>
                                        <VueMonacoEditor v-model:value="customCSS" language="css" theme="vs-dark"
                                            :options="{
                                                minimap: { enabled: false },
                                                fontSize: 14,
                                                readOnly: !enableCustomCSS,
                                                automaticLayout: true,
                                                lineNumbers: 'on',
                                                scrollBeyondLastLine: false,
                                                wordWrap: 'on',
                                                roundedSelection: false,
                                                scrollbar: { vertical: 'auto' }
                                            }" style="height: 300px; border-radius: 0.5rem; border: 1px solid #333;" />
                                    </div>
                                </div>
                                <div class="flex gap-2 mt-4">
                                    <button class="btn btn-primary btn-sm flex items-center gap-2"
                                        @click="openExportModal">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none"
                                            viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                                d="M12 4v16m8-8H4" />
                                        </svg>
                                        {{ t('theme.export_css_btn') }}
                                    </button>
                                    <button class="btn btn-secondary btn-sm flex items-center gap-2"
                                        @click="openImportModal">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none"
                                            viewBox="0 0 24 24" stroke="currentColor">
                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2"
                                                d="M4 12h16m-8 8V4" />
                                        </svg>
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
import { ref, reactive, onMounted, watch } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { useI18n } from 'vue-i18n';
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

const { t } = useI18n();
const { addToast } = useToast();
const { showModal } = useModal();

const themes = ['dark', 'light'];
const selectedTheme = ref(document.documentElement.getAttribute('data-theme') || 'dark');
const showExpertOptions = ref(false);
const expertAnimationActive = ref(false);

const customCSS = ref(themeService.settings.customCSS);
const enableCustomCSS = ref(themeService.settings.enableCustomCSS);

watch(customCSS, (val) => {
    if (enableCustomCSS.value) {
        themeService.updateCardSettings({ customCSS: val });
    }
});

watch(enableCustomCSS, (val) => {
    themeService.updateCardSettings({ enableCustomCSS: val });
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

const radiusIndex = ref(findOptionIndex(radiusOptions, themeService.settings.borderRadius, 2));
const shadowIndex = ref(findOptionIndex(shadowOptions, themeService.settings.shadow, 2));
const paddingIndex = ref(findOptionIndex(paddingOptions, themeService.settings.padding, 1));

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
    themeService.updateCardSettings({
        borderRadius: radiusOptions[index].value
    });
};

const handleShadowChange = () => {
    const index = shadowIndex.value >= 0 && shadowIndex.value < shadowOptions.length ? shadowIndex.value : 2;
    themeService.updateCardSettings({
        shadow: shadowOptions[index].value
    });
};

const handlePaddingChange = () => {
    const index = paddingIndex.value >= 0 && paddingIndex.value < paddingOptions.length ? paddingIndex.value : 1;
    themeService.updateCardSettings({
        padding: paddingOptions[index].value
    });
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
    themeService.updateCardSettings({
        customCSS: customCSS.value,
        enableCustomCSS: enableCustomCSS.value
    });
};

const resetCardStyles = () => {
    themeService.resetCardSettings();

    const newRadiusIndex = radiusOptions.findIndex(opt => opt.value === themeService.settings.borderRadius);
    const newShadowIndex = shadowOptions.findIndex(opt => opt.value === themeService.settings.shadow);
    const newPaddingIndex = paddingOptions.findIndex(opt => opt.value === themeService.settings.padding);

    radiusIndex.value = newRadiusIndex >= 0 ? newRadiusIndex : 2;
    shadowIndex.value = newShadowIndex >= 0 ? newShadowIndex : 2;
    paddingIndex.value = newPaddingIndex >= 0 ? newPaddingIndex : 1;

    customCSS.value = themeService.settings.customCSS;
    enableCustomCSS.value = themeService.settings.enableCustomCSS;
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

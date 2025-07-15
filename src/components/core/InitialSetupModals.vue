<script setup lang="ts">
import { openUrl } from '@tauri-apps/plugin-opener';
import { useToast } from '../../services/toastService';
import { useI18n } from 'vue-i18n';
import { ref, watch, onMounted } from 'vue';
import HoldButton from '../ui/HoldButton.vue';
import {
    changeLanguage,
    getAvailableLanguages,
    getCurrentLanguage,
} from '../../i18n';
import {
    Languages,
    MemoryStick,
    UserPlus,
    ChartNoAxesCombined,
    DoorOpen,
    Cpu,
    Handshake,
    Headset,
    NotebookPen,
    HeartHandshake
} from 'lucide-vue-next';
import AnimatedSlider from '../ui/AnimatedSlider.vue';
import { invoke } from '@tauri-apps/api/core';
import RegistrationForm from '../forms/RegistrationForm.vue';

const props = defineProps({
    showFirstRun: Boolean,
    showDisclaimer: Boolean,
    currentTheme: String,
});

const emit = defineEmits([
    'firstRunAccepted',
    'disclaimerAccepted',
    'themeChanged',
    'auto-login',
]);
const { t } = useI18n();
const { addToast } = useToast();

const availableThemes = ref(['dark', 'light']);
const selectedTheme = ref(props.currentTheme || 'dark');

const currentTutorialStep = ref(1);
const totalTutorialSteps = 6;

const tutorialContentWrapper = ref<HTMLElement | null>(null);

const availableLanguages = ref(getAvailableLanguages());
const selectedLanguage = ref(getCurrentLanguage());

const ramOptions = [
    { mb: 2048, label: '2 GB' },
    { mb: 4096, label: '4 GB' },
    { mb: 6144, label: '6 GB' },
    { mb: 8192, label: '8 GB' },
    { mb: 16384, label: '16 GB' },
    { mb: 32768, label: '32 GB' },
];
const ramOptionIndex = ref(0);

const showRegisterForm = ref(false);
const registrationFormRef = ref<InstanceType<typeof RegistrationForm> | null>(
    null
);

const goToStep = async (step: number) => {
    if (step >= 1 && step <= totalTutorialSteps) {
        if (currentTutorialStep.value === 2 && step > 2) {
            try {
                await saveRamSettings();
            } catch (error) {
                console.error('Failed to auto-save RAM settings:', error);
            }
        }
        currentTutorialStep.value = step;
    }
};

watch(
    () => props.currentTheme,
    (newVal) => {
        if (newVal && availableThemes.value.includes(newVal)) {
            selectedTheme.value = newVal;
        }
    }
);

const acceptFirstRun = () => {
    emit('firstRunAccepted');
};

const acceptDisclaimer = () => {
    emit('disclaimerAccepted');
};

const openTelegram = async () => {
    try {
        await openUrl('https://t.me/CollapseLoader');
    } catch (error) {
        console.error('Failed to open telegram:', error);
        addToast(t('about.open_failed', { platform: 'Telegram' }), 'error');
    }
};

const openDiscord = async () => {
    try {
        await openUrl('https://collapseloader.org/discord/');
    } catch (error) {
        console.error('Failed to open discord:', error);
        addToast(t('about.open_failed', { platform: 'Discord' }), 'error');
    }
};

const handleLanguageChange = async (languageCode: string) => {
    try {
        selectedLanguage.value = languageCode as 'en' | 'ru';
        await changeLanguage(languageCode);
        addToast(t('modals.initial_setup.language_changed'), 'success');
    } catch (error) {
        console.error('Failed to change language:', error);
        addToast(t('modals.initial_setup.language_change_failed'), 'error');
    }
};

const handleSliderChange = () => { };

const saveRamSettings = async () => {
    try {
        const currentSettings = await invoke('get_settings');
        const ramValue = ramOptions[ramOptionIndex.value].mb;
        const newSettings = {
            ...(currentSettings as any),
            ram: { value: ramValue, show: true },
        };
        await invoke('save_settings', { inputSettings: newSettings });
    } catch (error) {
        console.error('Failed to save RAM settings:', error);
    }
};

const handleRegistrationSuccess = () => {
    addToast(t('modals.initial_setup.registration.success'), 'success');
    showRegisterForm.value = false;
    goToStep(currentTutorialStep.value + 1);

    if (registrationFormRef.value) {
        registrationFormRef.value.clearForm();
    }
};

const handleAutoLogin = () => {
    showRegisterForm.value = false;
    emit('auto-login');

    if (registrationFormRef.value) {
        registrationFormRef.value.clearForm();
    }
};

const handleRegistrationCancel = () => {
    showRegisterForm.value = false;
    if (registrationFormRef.value) {
        registrationFormRef.value.clearForm();
    }
};

onMounted(() => {
    if (tutorialContentWrapper.value) {
        tutorialContentWrapper.value.scrollTop = 0;
    }
});
</script>

<template>
    <transition name="modal-fade">
        <div v-if="showFirstRun"
            class="fixed inset-0 bg-gradient-to-br from-black/90 to-black/95 flex items-center justify-center z-[1500] backdrop-blur-sm">
            <transition name="modal-scale">
                <div v-if="showFirstRun"
                    class="bg-base-100 rounded-xl shadow-2xl w-full h-full max-w-none max-h-none modal-container">
                    <div class="flex flex-col h-full">
                        <div class="bg-primary/5 px-8 py-3 border-b border-base-300 flex-shrink-0">
                            <div class="flex items-center justify-between">
                                <transition name="step-title" mode="out-in">
                                    <div v-if="currentTutorialStep === 1" key="step1" class="flex items-center gap-2">
                                        <DoorOpen class="w-6 h-6 text-primary" />
                                        <h3 class="text-2xl font-bold text-primary">
                                            {{
                                                t(
                                                    'modals.initial_setup.welcome.title'
                                                )
                                            }}
                                        </h3>
                                    </div>
                                    <div v-else-if="currentTutorialStep === 2" key="step2"
                                        class="flex items-center gap-2">
                                        <MemoryStick class="w-6 h-6 text-primary" />
                                        <h3 class="text-2xl font-bold text-primary">
                                            {{
                                                t(
                                                    'modals.initial_setup.ram.title'
                                                )
                                            }}
                                        </h3>
                                    </div>
                                    <div v-else-if="currentTutorialStep === 3" key="step3"
                                        class="flex items-center gap-2">
                                        <ChartNoAxesCombined class="w-6 h-6 text-primary" />
                                        <h3 class="text-2xl font-bold text-primary">
                                            {{
                                                t(
                                                    'modals.initial_setup.telemetry.title'
                                                )
                                            }}
                                        </h3>
                                    </div>
                                    <div v-else-if="currentTutorialStep === 4" key="step4"
                                        class="flex items-center gap-2">
                                        <UserPlus class="w-6 h-6 text-primary" />
                                        <h3 class="text-2xl font-bold text-primary">
                                            {{
                                                t(
                                                    'modals.initial_setup.registration.title'
                                                )
                                            }}
                                        </h3>
                                    </div>
                                    <div v-else-if="currentTutorialStep === 5" key="step5"
                                        class="flex items-center gap-2">
                                        <Headset class="w-6 h-6 text-primary" />
                                        <h3 class="text-2xl font-bold text-primary">
                                            {{
                                                t(
                                                    'modals.initial_setup.feedback.title'
                                                )
                                            }}
                                        </h3>
                                    </div>
                                    <div v-else-if="currentTutorialStep === 6" key="step6"
                                        class="flex items-center gap-2">
                                        <Handshake class="w-6 h-6 text-primary" />
                                        <h3 class="text-2xl font-bold text-primary">
                                            {{
                                                t(
                                                    'modals.initial_setup.appreciation.title'
                                                )
                                            }}
                                        </h3>
                                    </div>
                                </transition>
                                <div class="flex items-center gap-3 ml-auto animate-slide-in-right">
                                    <Languages class="w-5 h-5 text-primary" />
                                    <select v-model="selectedLanguage" @change="
                                        handleLanguageChange(
                                            selectedLanguage
                                        )
                                        "
                                        class="select select-bordered select-sm bg-base-100 min-w-0 w-auto transition-all duration-300 hover:scale-105">
                                        <option v-for="lang in availableLanguages" :key="lang.code" :value="lang.code">
                                            {{ lang.nativeName }}
                                        </option>
                                    </select>
                                </div>
                            </div>
                        </div>

                        <div ref="tutorialContentWrapper" class="tutorial-content-wrapper flex-grow">
                            <div class="tutorial-content">
                                <transition name="step-height" mode="out-in">
                                    <div v-if="currentTutorialStep === 1" key="step1" class="step-content">
                                        <div class="text-center">
                                            <img src="../../assets/images/logo.svg" alt="CollapseLoader Logo"
                                                class="w-24 h-24 mx-auto mb-6" />
                                            <p
                                                class="text-lg text-base-content/70 mb-6 animate-fade-in-up animation-delay-200">
                                                {{
                                                    t(
                                                        'modals.initial_setup.welcome.description'
                                                    )
                                                }}
                                            </p>
                                        </div>
                                    </div>
                                </transition>
                                <transition name="step-height" mode="out-in">
                                    <div v-if="currentTutorialStep === 2" key="step3" class="step-content">
                                        <div class="text-center">
                                            <Cpu class="w-16 h-16 mx-auto mb-4 text-primary animate-bounce-gentle" />
                                            <p
                                                class="text-lg text-base-content/70 mb-6 animate-fade-in-up animation-delay-200">
                                                {{
                                                    t(
                                                        'modals.initial_setup.ram.description'
                                                    )
                                                }}
                                            </p>

                                            <div
                                                class="space-y-4 animate-fade-in-up animation-delay-400 max-w-xl mx-auto">
                                                <div class="flex items-center gap-4">
                                                    <AnimatedSlider v-model="ramOptionIndex" :min="0" :max="ramOptions.length -
                                                        1
                                                        " @update:modelValue="
                                                            handleSliderChange
                                                        " class="flex-grow" />
                                                    <div
                                                        class="flex items-center gap-2 rounded-lg p-3 bg-base-200 min-w-fit shadow-md">
                                                        <span class="text-lg font-bold">{{
                                                            ramOptions[
                                                                ramOptionIndex
                                                            ].label
                                                        }}</span>
                                                        <div class="badge badge-primary">
                                                            {{
                                                                ramOptions[
                                                                    ramOptionIndex
                                                                ].mb
                                                            }}
                                                            MB
                                                        </div>
                                                    </div>
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </transition>
                                <transition name="step-height" mode="out-in">
                                    <div v-if="currentTutorialStep === 3" key="step3" class="step-content">
                                        <div class="text-center">
                                            <p
                                                class="text-base text-base-content/70 mb-6 animate-fade-in-up animation-delay-200 max-w-lg mx-auto">
                                                {{
                                                    t(
                                                        'modals.initial_setup.telemetry.description'
                                                    )
                                                }}
                                            </p>
                                            <div
                                                class="bg-base-200 rounded-lg p-4 space-y-3 mb-6 max-w-xl mx-auto text-left">
                                                <h4 class="font-semibold text-primary text-center">
                                                    {{
                                                        t(
                                                            'modals.initial_setup.telemetry.data_collected_title'
                                                        )
                                                    }}
                                                </h4>
                                                <ul class="text-sm text-base-content/70 space-y-2">
                                                    <li class="flex items-start">
                                                        <span class="text-primary mr-2">•</span>
                                                        {{
                                                            t(
                                                                'modals.initial_setup.telemetry.data_collected.performance'
                                                            )
                                                        }}
                                                    </li>
                                                    <li class="flex items-start">
                                                        <span class="text-primary mr-2">•</span>
                                                        {{
                                                            t(
                                                                'modals.initial_setup.telemetry.data_collected.usage_patterns'
                                                            )
                                                        }}
                                                    </li>
                                                    <li class="flex items-start">
                                                        <span class="text-primary mr-2">•</span>
                                                        {{
                                                            t(
                                                                'modals.initial_setup.telemetry.data_collected.crash_reports'
                                                            )
                                                        }}
                                                    </li>
                                                </ul>
                                            </div>
                                        </div>
                                    </div>
                                </transition>
                                <transition name="step-height" mode="out-in">
                                    <div v-if="currentTutorialStep === 4" key="step4" class="step-content">
                                        <div class="text-center">
                                            <div v-if="!showRegisterForm">
                                                <UserPlus class="w-16 h-16 mx-auto mb-4 text-primary" />
                                                <p
                                                    class="text-lg text-base-content/70 mb-6 animate-fade-in-up animation-delay-200">
                                                    {{
                                                        t(
                                                            'modals.initial_setup.registration.description'
                                                        )
                                                    }}
                                                </p>
                                            </div>

                                            <div v-if="!showRegisterForm"
                                                class="space-y-4 animate-fade-in-up animation-delay-400 flex content-center justify-center gap-4">
                                                <button @click="
                                                    showRegisterForm = true
                                                    " class="btn btn-primary">
                                                    {{
                                                        t(
                                                            'modals.initial_setup.registration.create_account'
                                                        )
                                                    }}
                                                </button>
                                            </div>

                                            <div v-else class="max-w-sm mx-auto animate-fade-in-up">
                                                <RegistrationForm ref="registrationFormRef" :show-cancel-button="true"
                                                    :compact="true" @registered="
                                                        handleRegistrationSuccess
                                                    " @logged-in="handleAutoLogin" @cancel="
                                                        handleRegistrationCancel
                                                    " />
                                            </div>
                                        </div>
                                    </div>
                                </transition>
                                <transition name="step-height" mode="out-in">
                                    <div v-if="currentTutorialStep === 5" key="step5" class="step-content">
                                        <div class="text-center">
                                            <DoorOpen class="w-16 h-16 mx-auto mb-4 text-primary" />

                                            <p
                                                class="text-lg text-base-content/70 mb-6 animate-fade-in-up animation-delay-200">
                                                {{
                                                    t(
                                                        'modals.initial_setup.feedback.description'
                                                    )
                                                }}
                                            </p>

                                            <div
                                                class="flex gap-4 justify-center animate-fade-in-up animation-delay-400">
                                                <button @click="openTelegram"
                                                    class="btn btn-ghost bg-info text-white hover:scale-110 transition-transform duration-300">
                                                    <img src="@/assets/icons/telegram.svg"
                                                        class="w-6 h-6 mr-2 telegram-icon" />
                                                    Telegram
                                                </button>
                                                <button @click="openDiscord"
                                                    class="btn btn-ghost bg-indigo-500 text-white hover:scale-110 transition-transform duration-300">
                                                    <img src="@/assets/icons/discord.svg"
                                                        class="w-6 h-6 mr-2 discord-icon" />
                                                    Discord
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                </transition>
                                <transition name="step-height" mode="out-in">
                                    <div v-if="currentTutorialStep === 6" key="step6" class="step-content">
                                        <div class="text-center">
                                            <HeartHandshake class="w-16 h-16 mx-auto mb-4 text-primary" />
                                            <p
                                                class="text-lg text-base-content/70 animate-fade-in-up animation-delay-200">
                                                {{
                                                    t(
                                                        'modals.initial_setup.appreciation.description'
                                                    )
                                                }}
                                            </p>

                                            <HoldButton @start="acceptFirstRun" class="mt-5" />
                                        </div>
                                    </div>
                                </transition>
                            </div>
                        </div>

                        <div class="bg-primary/5 px-8 py-4 border-t border-base-300 animate-slide-up flex-shrink-0">
                            <div class="flex items-center justify-between max-w-4xl mx-auto">
                                <div class="flex items-center space-x-3">
                                    <button v-for="step in totalTutorialSteps" :key="step" @click="goToStep(step)"
                                        class="w-3 h-3 rounded-full transition-all cursor-pointer duration-500 hover:scale-150"
                                        :class="step === currentTutorialStep
                                            ? 'bg-primary shadow-lg'
                                            : 'bg-base-content/30 hover:bg-base-content/50'
                                            "></button>
                                </div>

                                <div class="flex gap-3">
                                    <button v-if="currentTutorialStep > 1" @click="
                                        goToStep(currentTutorialStep - 1)
                                        " class="btn btn-sm btn-outline hover:scale-105 transition-all duration-300">
                                        {{ t('common.previous') }}
                                    </button>
                                    <button v-if="
                                        currentTutorialStep <
                                        totalTutorialSteps
                                    " @click="
                                        goToStep(currentTutorialStep + 1)
                                        " class="btn btn-sm btn-primary hover:scale-105 transition-all duration-300">
                                        {{ t('common.next') }}
                                    </button>
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            </transition>
        </div>
    </transition>

    <transition name="modal-fade">
        <div v-if="showDisclaimer"
            class="fixed inset-0 bg-gradient-to-br from-black/90 to-black/95 flex items-center justify-center z-[1500] backdrop-blur-sm">
            <transition name="modal-scale">
                <div class="bg-base-100 rounded-xl shadow-2xl w-full h-full max-w-none max-h-none modal-container">
                    <div class="flex flex-col h-full">
                        <div class="bg-error/10 px-8 py-6 border-b border-error/20 flex-shrink-0">
                            <h2 class="text-xl font-bold text-error animate-slide-in-left">
                                <NotebookPen class="inline w-6 h-6 mr-2 text-error" />
                                {{ t('modals.initial_setup.disclaimer.title') }}
                            </h2>
                        </div>

                        <div
                            class="disclaimer-content flex-grow flex items-center justify-center px-8 py-8 animate-fade-in-up animation-delay-200">
                            <div class="space-y-4 max-w-2xl mx-auto text-center">
                                <p class="text-base text-base-content/80 mb-4">
                                    {{
                                        t(
                                            'modals.initial_setup.disclaimer.responsibility'
                                        )
                                    }}
                                </p>
                                <p class="text-base font-semibold text-warning">
                                    {{
                                        t(
                                            'modals.initial_setup.disclaimer.acknowledgment'
                                        )
                                    }}
                                </p>
                            </div>
                        </div>

                        <div class="animate-slide-up animation-delay-400 flex-shrink-0">
                            <div class="bg-base-200 px-8 py-4 border-t border-base-300">
                                <button @click="acceptDisclaimer"
                                    class="btn btn-error w-full hover:scale-105 transition-all duration-300">
                                    {{
                                        t(
                                            'modals.initial_setup.disclaimer.accept'
                                        )
                                    }}
                                </button>
                            </div>
                        </div>
                    </div>
                </div>
            </transition>
        </div>
    </transition>
</template>

<style scoped>
.modal-container {
    transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
    margin: 0;
    border-radius: 0;
}

.modal-fade-enter-active,
.modal-fade-leave-active {
    transition: opacity 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.modal-fade-enter-from,
.modal-fade-leave-to {
    opacity: 0;
}

.modal-scale-enter-active {
    transition: all 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.modal-scale-leave-active {
    transition: all 0.3s cubic-bezier(0.55, 0.085, 0.68, 0.53);
}

.modal-scale-enter-from,
.modal-scale-leave-to {
    opacity: 0;
    transform: scale(0.85) translateY(20px);
}

.modal-scale-enter-to,
.modal-scale-leave-from {
    opacity: 1;
    transform: scale(1) translateY(0);
}

.step-height-enter-active,
.step-height-leave-active {
    transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.step-height-enter-from {
    opacity: 0;
    transform: translateY(30px);
}

.step-height-leave-to {
    opacity: 0;
    transform: translateY(-30px);
}

.step-height-enter-to,
.step-height-leave-from {
    opacity: 1;
    transform: translateY(0);
}

.step-content {
    position: absolute;
    display: flex;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    padding: 2rem;
    width: 100%;
    justify-content: center;
    align-items: center;
}

.tutorial-content-wrapper {
    overflow: hidden;
    position: relative;
    flex: 1;
}

.tutorial-content {
    position: relative;
    height: 100%;
    transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

@keyframes fadeInUp {
    from {
        opacity: 0;
        transform: translateY(20px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}

@keyframes slideInLeft {
    from {
        opacity: 0;
        transform: translateX(-20px);
    }

    to {
        opacity: 1;
        transform: translateX(0);
    }
}

@keyframes slideInRight {
    from {
        opacity: 0;
        transform: translateX(20px);
    }

    to {
        opacity: 1;
        transform: translateX(0);
    }
}

@keyframes slideUp {
    from {
        opacity: 0;
        transform: translateY(20px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}

@keyframes bounceGentle {

    0%,
    20%,
    50%,
    80%,
    100% {
        transform: translateY(0);
    }

    40% {
        transform: translateY(-8px);
    }

    60% {
        transform: translateY(-4px);
    }
}

@keyframes pulseGentle {

    0%,
    100% {
        transform: scale(1);
        opacity: 1;
    }

    50% {
        transform: scale(1.02);
        opacity: 0.9;
    }
}

.animate-fade-in-up {
    animation: fadeInUp 0.6s cubic-bezier(0.25, 0.8, 0.25, 1) forwards;
    opacity: 0;
}

.animate-slide-in-left {
    animation: slideInLeft 0.6s cubic-bezier(0.25, 0.8, 0.25, 1) forwards;
    opacity: 0;
}

.animate-slide-in-right {
    animation: slideInRight 0.6s cubic-bezier(0.25, 0.8, 0.25, 1) forwards;
    opacity: 0;
}

.animate-slide-up {
    animation: slideUp 0.6s cubic-bezier(0.25, 0.8, 0.25, 1) forwards;
    opacity: 0;
}

.animate-bounce-gentle {
    animation: bounceGentle 2s infinite;
}

.animate-pulse-gentle {
    animation: pulseGentle 2s infinite;
}

.animation-delay-200 {
    animation-delay: 0.2s;
}

.animation-delay-400 {
    animation-delay: 0.4s;
}

.animation-delay-600 {
    animation-delay: 0.6s;
}

.btn {
    transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.btn:hover {
    transform: translateY(-2px);
    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
}

button[class*='w-2 h-2'] {
    position: relative;
    overflow: hidden;
}

button[class*='w-2 h-2']:before {
    content: '';
    position: absolute;
    top: 50%;
    left: 50%;
    width: 0;
    height: 0;
    background: rgba(255, 255, 255, 0.3);
    border-radius: 50%;
    transform: translate(-50%, -50%);
    transition: all 0.3s ease;
}

button[class*='w-2 h-2']:hover:before {
    width: 200%;
    height: 200%;
}

/* Анимация для заголовков шагов в верхней панели */
.step-title-enter-active,
.step-title-leave-active {
    transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.step-title-enter-from {
    opacity: 0;
    transform: translateX(20px) scale(0.95);
}

.step-title-leave-to {
    opacity: 0;
    transform: translateX(-20px) scale(0.95);
}

.step-title-enter-to,
.step-title-leave-from {
    opacity: 1;
    transform: translateX(0) scale(1);
}

@media (max-width: 768px) {
    .modal-container {
        border-radius: 0.5rem;
        max-width: calc(100vw - 1rem);
        max-height: calc(100vh - 1rem);
    }

    .step-content {
        padding: 1rem;
    }

    .tutorial-content-wrapper {
        padding: 0.5rem;
    }
}


@media (max-width: 768px) {
    .step-content {
        padding: 1rem;
    }

    .modal-container {
        border-radius: 0;
    }
}

html[data-theme='dark'] .telegram-icon,
html[data-theme='dark'] .discord-icon {
    filter: invert(100%) sepia(15%) saturate(1%) hue-rotate(282deg) brightness(102%) contrast(101%);
}

html[data-theme='light'] .telegram-icon,
html[data-theme='light'] .discord-icon {
    filter: invert(0%) sepia(15%) saturate(17%) hue-rotate(253deg) brightness(95%) contrast(103%);
}
</style>

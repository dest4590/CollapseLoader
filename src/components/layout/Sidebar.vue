<script setup lang="ts">
import {
    Home,
    Info,
    Settings,
    Terminal,
    LogIn,
    User,
    Users,
    ShieldAlert,
} from 'lucide-vue-next';
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useFriends } from '../../composables/useFriends';
import { useUser } from '../../composables/useUser';
import { Vue3Lottie } from 'vue3-lottie';
import customization_idle from '../../assets/misc/customization_idle.json';
import customization_animated from '../../assets/misc/customization_animated.json';

const { t } = useI18n();
const { adminStatus } = useUser();

const props = defineProps<{
    activeTab: string;
    isOnline: boolean;
    isAuthenticated: boolean;
}>();

const emit = defineEmits(['changeTab', 'open-dev-menu']);
const isAltPressed = ref(false);
const isSlideOutActive = ref(false);
const terminalButtonRef = ref<HTMLButtonElement | null>(null);

const isAdmin = computed(() => adminStatus.value?.is_admin || false);

const { onlineFriendsCount } = useFriends();

let homeClickCount = 0;
const homeClickTimeout = ref<number | null>(null);
const slideOutTimeout = ref<number | null>(null);

const changeTab = (tab: string) => {
    if (tab === 'home') {
        homeClickCount++;
        if (homeClickTimeout.value) {
            clearTimeout(homeClickTimeout.value);
        }
        homeClickTimeout.value = setTimeout(() => {
            homeClickCount = 0;
        }, 1500) as unknown as number;

        if (homeClickCount === 5) {
            homeClickCount = 0;
            if (homeClickTimeout.value) {
                clearTimeout(homeClickTimeout.value);
            }
            emit('open-dev-menu');
        }
    } else {
        homeClickCount = 0;
        if (homeClickTimeout.value) {
            clearTimeout(homeClickTimeout.value);
        }
    }

    if (tab != 'app_logs') {
        isSlideOutActive.value = true;
    }

    emit('changeTab', tab);
};

const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Alt') {
        isAltPressed.value = true;
        isSlideOutActive.value = false;

        if (slideOutTimeout.value) {
            clearTimeout(slideOutTimeout.value);
            slideOutTimeout.value = null;
        }
    }
};

const handleKeyUp = (event: KeyboardEvent) => {
    if (event.key === 'Alt' && props.activeTab !== 'app_logs') {
        isSlideOutActive.value = true;

        slideOutTimeout.value = setTimeout(() => {
            isAltPressed.value = false;
            isSlideOutActive.value = false;
        }, 300) as unknown as number;
    }
};
onMounted(async () => {
    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
});

onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyDown);
    window.removeEventListener('keyup', handleKeyUp);

    if (homeClickTimeout.value) {
        clearTimeout(homeClickTimeout.value);
    }

    if (slideOutTimeout.value) {
        clearTimeout(slideOutTimeout.value);
    }
});
</script>

<template>
    <div
        class="w-20 h-screen fixed left-0 top-0 bg-base-300 flex flex-col items-center py-6 shadow-md border-r border-base-content/10 z-50">
        <div class="flex flex-col gap-4">
            <div class="tooltip tooltip-right tooltip-accent" :data-tip="t('navigation.home')">
                <button class="btn btn-ghost btn-square rounded-lg relative sidebar-btn" :class="{
                    'bg-primary text-primary-content shadow-lg scale-110':
                        activeTab === 'home',
                }" @click="changeTab('home')">
                    <Home class="w-5 h-5" />
                    <span v-if="!isOnline"
                        class="absolute top-0 right-0 w-3 h-3 bg-error rounded-full border-2 border-base-300"></span>
                </button>
            </div>

            <div v-if="isAuthenticated" class="tooltip tooltip-right tooltip-accent"
                :data-tip="t('navigation.friends')">
                <button class="btn btn-ghost btn-square rounded-lg transition-all relative sidebar-btn"
                    @click="changeTab('friends')" :class="{
                        'bg-primary text-primary-content shadow-lg scale-110':
                            activeTab === 'friends',
                    }">
                    <Users class="w-5 h-5" />
                    <span v-if="onlineFriendsCount > 0"
                        class="absolute -top-1 -right-1 bg-success text-success-content text-xs font-bold rounded-full w-5 h-5 flex items-center justify-center border-2 border-base-300">
                        {{ onlineFriendsCount }}
                    </span>
                </button>
            </div>

            <div class="tooltip tooltip-right tooltip-accent" :data-tip="t('navigation.settings')">
                <button class="btn btn-ghost btn-square rounded-lg sidebar-btn" :class="{
                    'bg-primary text-primary-content shadow-lg scale-110':
                        activeTab === 'settings',
                }" @click="changeTab('settings')">
                    <Settings class="w-5 h-5 settings-icon" :style="{
                        transform:
                            activeTab === 'settings'
                                ? 'rotate(180deg)'
                                : 'rotate(0deg)',
                    }" />
                </button>
            </div>
            <div class="tooltip tooltip-right tooltip-accent" :data-tip="t('navigation.customization')">
                <button class="btn btn-ghost btn-square rounded-lg transition-all sidebar-btn" :class="{
                    'bg-primary text-primary-content shadow-lg scale-110':
                        activeTab === 'customization',
                }" @click="changeTab('customization')">
                    <Vue3Lottie :animation-data="customization_animated" class="w-5 h-5 customization-light-selected"
                        v-if="activeTab === 'customization'" :class="{
                            'invert': activeTab === 'customization'
                        }" />
                    <Vue3Lottie :animation-data="customization_idle" class="w-5 h-5 customization-light" v-else />
                </button>
            </div>

            <div v-show="isAltPressed" class="tooltip tooltip-right tooltip-accent" :data-tip="t('navigation.logs')">
                <button ref="terminalButtonRef" class="btn btn-ghost btn-square rounded-lg terminal-button sidebar-btn"
                    :class="{
                        'bg-primary text-primary-content shadow-lg scale-110':
                            activeTab === 'app_logs',
                        'slide-out': isSlideOutActive,
                    }" @click="changeTab('app_logs')">
                    <img src="/src/assets/images/sidebar/terminal-blink.svg" alt="Terminal Icon" class="w-5 h-5"
                        v-if="activeTab === 'app_logs'" />
                    <Terminal class="w-5 h-5" v-if="activeTab !== 'app_logs'" />
                </button>
            </div>
        </div>

        <div class="mt-auto flex flex-col gap-4">
            <div v-if="isAuthenticated && isAdmin" class="tooltip tooltip-right tooltip-accent"
                :data-tip="t('navigation.admin')">
                <button class="btn btn-ghost btn-square rounded-lg transition-all relative sidebar-btn"
                    @click="changeTab('admin')" :class="{
                        'bg-primary text-primary-content shadow-lg scale-110':
                            activeTab === 'admin',
                    }">
                    <ShieldAlert class="w-5 h-5" />
                </button>
            </div>

            <div class="tooltip tooltip-right tooltip-accent" :data-tip="isAuthenticated
                ? t('navigation.account')
                : t('auth.login.title')
                ">
                <button class="btn btn-ghost btn-square rounded-lg relative sidebar-btn" :class="{
                    'bg-primary text-primary-content shadow-lg scale-110': [
                        'login',
                        'register',
                        'account',
                    ].includes(activeTab),
                }" @click="changeTab(isAuthenticated ? 'account' : 'login')">
                    <LogIn v-if="!isAuthenticated" class="w-5 h-5" />
                    <User v-else class="w-5 h-5" />
                </button>
            </div>

            <div class="tooltip tooltip-right tooltip-accent" :data-tip="t('navigation.about')">
                <button class="btn btn-ghost btn-square rounded-lg transition-all sidebar-btn"
                    @click="changeTab('about')" :class="{
                        'bg-primary text-primary-content shadow-lg scale-110':
                            activeTab === 'about',
                    }">
                    <Info class="w-5 h-5" />
                </button>
            </div>
        </div>
    </div>
</template>

<style scoped>
.btn-square {
    transition:
        all 0.2s cubic-bezier(0.4, 0, 0.2, 1),
        transform 0.15s ease-out;
}

.btn-square:hover {
    transform: scale(1.05);
}

.btn-square:active {
    transform: scale(1.02);
}

.settings-icon {
    transition: transform 1s ease;
}

.terminal-button {
    animation: fadeIn 0.3s ease-out;
    opacity: 0;
    animation-fill-mode: forwards;
    transform: translateY(10px);
}

@keyframes fadeIn {
    from {
        opacity: 0;
        transform: translateY(10px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.slide-out {
    animation: slideOut 0.3s ease-out forwards;
}

@keyframes slideOut {
    from {
        opacity: 1;
        transform: translateY(0px);
    }

    to {
        opacity: 0;
        transform: translateY(10px);
    }
}

.invert {
    filter: invert(1);
}

html[data-theme="light"] .customization-light {
    filter: invert(1);
}

html[data-theme="light"] .customization-light-selected {
    filter: invert(0);
}

.sidebar-btn,
.btn-square.sidebar-btn,
.btn-square.sidebar-btn>* {
    border-radius: var(--radius-box, 0.5rem) !important;
    overflow: visible !important;
}

.sidebar-btn img,
.sidebar-btn .Vue3Lottie,
.sidebar-btn svg {
    border-radius: inherit !important;
}

.sidebar-btn {
    will-change: transform, box-shadow;
}

html[data-reduce-motion='true'] .sidebar-btn,
html[data-reduce-motion='true'] .btn-square.sidebar-btn {
    animation: none !important;
    transition: none !important;
    transform: none !important;
    opacity: 1 !important;
}
</style>

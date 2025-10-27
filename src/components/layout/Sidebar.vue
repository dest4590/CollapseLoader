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
    SlidersVertical,
    UserCog,
} from 'lucide-vue-next';
import { computed, onMounted, onUnmounted, ref } from 'vue';
import { useI18n } from 'vue-i18n';
import { useFriends } from '../../composables/useFriends';
import { useUser } from '../../composables/useUser';
import { getIsDevelopment } from '../../utils/isDevelopment';
import { isHalloweenEvent, getEventGreeting } from '../../utils/events';

const { t } = useI18n();
const { adminStatus } = useUser();
const halloweenActive = ref(isHalloweenEvent());
const halloweenGreeting = ref(getEventGreeting());

defineProps<{
    activeTab: string;
    isOnline: boolean;
    isAuthenticated: boolean;
}>();

const emit = defineEmits(['changeTab', 'open-dev-menu']);
const visible = ref(false);
const isAltPressed = ref(false);
const altPressCount = ref(0);
const altPressTimeout = ref<number | null>(null);

const isDev = ref(false);
const isAdmin = computed(() => adminStatus.value?.is_admin || false);

const { onlineFriendsCount, friendRequests } = useFriends();
const incomingRequestsCount = computed(() => (friendRequests.value?.received?.length) || 0);

let homeClickCount = 0;
const homeClickTimeout = ref<number | null>(null);

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


    emit('changeTab', tab);
};

const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Alt') {
        altPressCount.value++;

        if (altPressTimeout.value) {
            clearTimeout(altPressTimeout.value);
        }

        altPressTimeout.value = setTimeout(() => {
            altPressCount.value = 0;
            altPressTimeout.value = null;
        }, 600) as unknown as number;

        if (altPressCount.value === 2) {
            altPressCount.value = 0;
            if (altPressTimeout.value) {
                clearTimeout(altPressTimeout.value);
                altPressTimeout.value = null;
            }
            isAltPressed.value = !isAltPressed.value;
        }
    }
};

onMounted(async () => {
    window.addEventListener('keydown', handleKeyDown);

    visible.value = false;
    requestAnimationFrame(() => {
        setTimeout(() => {
            visible.value = true;
        }, 40);
    });


    isDev.value = await getIsDevelopment();
});

onUnmounted(() => {
    window.removeEventListener('keydown', handleKeyDown);

    if (homeClickTimeout.value) {
        clearTimeout(homeClickTimeout.value);
    }

    if (altPressTimeout.value) {
        clearTimeout(altPressTimeout.value);
    }
});
</script>

<template>
    <div
        :class="['w-20 h-screen fixed left-0 top-0 bg-base-300 flex flex-col items-center py-6 shadow-md border-r border-base-content/10 z-50', visible ? 'sidebar-entered' : 'sidebar-hidden']">
        <div class="flex flex-col gap-4">
            <div class="tooltip tooltip-right" :class="halloweenActive ? 'tooltip-warning' : 'tooltip-accent'" :data-tip="t('navigation.home') + (halloweenActive ? ' — ' + halloweenGreeting : '')">
                <button class="btn btn-ghost btn-square rounded-lg transition-all relative sidebar-btn" :class="{
                    'bg-primary text-primary-content shadow-lg scale-110':
                        activeTab === 'home',
                }" @click="changeTab('home')">
                    <Home class="w-5 h-5" />
                    <span v-if="halloweenActive" class="absolute -top-1 -right-1 text-xl">🎃</span>
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

                    <span v-if="incomingRequestsCount > 0"
                        class="absolute -bottom-1 -right-1 bg-info text-info-content text-xs font-bold rounded-full w-5 h-5 flex items-center justify-center border-2 border-base-300">
                        {{ incomingRequestsCount }}
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
                    <SlidersVertical class="w-5 h-5" />
                </button>
            </div>

            <div v-show="isAltPressed" class="tooltip tooltip-right tooltip-accent" :data-tip="t('navigation.logs')">
                <button class="btn btn-ghost btn-square rounded-lg sidebar-btn" :class="{
                    'bg-primary text-primary-content shadow-lg scale-110':
                        activeTab === 'app_logs',
                }" @click="changeTab('app_logs')">
                    <Terminal class="w-5 h-5" />
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
                    <UserCog v-if="isAuthenticated && isDev" class="w-5 h-5" />
                    <User v-if="isAuthenticated && !isDev" class="w-5 h-5" />
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
    outline: none;
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

.invert {
    filter: invert(1);
}

.sidebar-btn,
.btn-square.sidebar-btn,
.btn-square.sidebar-btn>* {
    border-radius: var(--radius-box, 0.5rem) !important;
}

.sidebar-btn {
    will-change: transform, box-shadow;
}


.sidebar-hidden {
    transform: translateX(-28px);
    opacity: 0;
    pointer-events: none;
}

.sidebar-entered {
    transform: translateX(0);
    opacity: 1;
    transition: transform 1.6s cubic-bezier(0.2, 0.9, 0.2, 1), opacity 0.5s ease;
}

.sidebar-hidden .flex>*,
.sidebar-hidden .mt-auto>* {
    opacity: 0;
    transform: translateY(8px) scale(0.995);
}

.sidebar-entered .flex>*,
.sidebar-entered .mt-auto>* {
    opacity: 1;
    transform: translateY(0) scale(1);
    transition: transform 0.42s cubic-bezier(0.2, 0.9, 0.2, 1), opacity 0.42s ease;
}

.sidebar-entered .flex>*:nth-child(1) {
    transition-delay: 0.06s;
}

.sidebar-entered .flex>*:nth-child(2) {
    transition-delay: 0.10s;
}

.sidebar-entered .flex>*:nth-child(3) {
    transition-delay: 0.14s;
}

.sidebar-entered .flex>*:nth-child(4) {
    transition-delay: 0.18s;
}

.sidebar-entered .mt-auto>*:nth-child(1) {
    transition-delay: 0.22s;
}

.sidebar-entered .mt-auto>*:nth-child(2) {
    transition-delay: 0.26s;
}

.sidebar-entered .mt-auto>*:nth-child(3) {
    transition-delay: 0.30s;
}
</style>

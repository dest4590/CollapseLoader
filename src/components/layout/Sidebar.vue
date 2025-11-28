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

const props = defineProps<{
    activeTab: string;
    isOnline: boolean;
    isAuthenticated: boolean;
    position?: 'left' | 'right' | 'top' | 'bottom';
}>();

const emit = defineEmits(['changeTab', 'open-dev-menu', 'update:position']);
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

const isDragging = ref(false);
const dragTarget = ref<string | null>(null);

const onDrag = (event: MouseEvent) => {
    const { clientX, clientY } = event;
    const { innerWidth, innerHeight } = window;

    const distLeft = clientX;
    const distRight = innerWidth - clientX;
    const distTop = clientY;
    const distBottom = innerHeight - clientY;

    const min = Math.min(distLeft, distRight, distTop, distBottom);
    let newPos = 'left';

    if (min === distLeft) newPos = 'left';
    else if (min === distRight) newPos = 'right';
    else if (min === distTop) newPos = 'top';
    else if (min === distBottom) newPos = 'bottom';

    dragTarget.value = newPos;

    if (newPos !== props.position) {
        emit('update:position', newPos);
    }
};

const startDrag = (event: MouseEvent) => {
    const target = event.target as HTMLElement;
    if (target.closest('button')) return;

    isDragging.value = true;
    document.body.style.cursor = 'grabbing';
    document.body.style.userSelect = 'none';
    document.addEventListener('mouseup', stopDrag);
    document.addEventListener('mousemove', onDrag);
};

const stopDrag = () => {
    if (!isDragging.value) return;
    isDragging.value = false;
    document.body.style.cursor = '';
    document.body.style.userSelect = '';
    document.removeEventListener('mouseup', stopDrag);
    document.removeEventListener('mousemove', onDrag);

    dragTarget.value = null;
};

const currentPosition = computed(() => props.position || 'left');

const sidebarClasses = computed(() => {
    const base = 'fixed bg-base-300 flex items-center shadow-md border-base-content/10 z-50 transition-all duration-500 ease-in-out active:cursor-grabbing';
    const pos = currentPosition.value;

    if (pos === 'left') return `${base} w-20 h-screen left-0 top-0 flex-col py-6 border-r`;
    if (pos === 'right') return `${base} w-20 h-screen right-0 top-0 flex-col py-6 border-l`;
    if (pos === 'top') return `${base} w-screen h-20 left-0 top-0 flex-row px-6 border-b`;
    if (pos === 'bottom') return `${base} w-screen h-20 left-0 bottom-0 flex-row px-6 border-t`;
    return '';
});

const tooltipClass = computed(() => {
    const pos = currentPosition.value;
    if (pos === 'left') return 'tooltip-right';
    if (pos === 'right') return 'tooltip-left';
    if (pos === 'top') return 'tooltip-bottom';
    if (pos === 'bottom') return 'tooltip-top';
    return 'tooltip-right';
});

const containerClasses = computed(() => {
    return ['flex', 'gap-4', ['top', 'bottom'].includes(currentPosition.value) ? 'flex-row' : 'flex-col'];
});

const footerClasses = computed(() => {
    return [['top', 'bottom'].includes(currentPosition.value) ? 'ml-auto flex-row' : 'mt-auto flex-col', 'flex', 'gap-4'];
});

const animationClass = computed(() => {
    if (!visible.value) {
        if (currentPosition.value === 'left') return 'sidebar-hidden-left';
        if (currentPosition.value === 'right') return 'sidebar-hidden-right';
        if (currentPosition.value === 'top') return 'sidebar-hidden-top';
        if (currentPosition.value === 'bottom') return 'sidebar-hidden-bottom';
    }
    return 'sidebar-entered';
});

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
    <div v-if="isDragging" class="fixed inset-0 z-40 pointer-events-none">
        <div class="absolute left-0 top-0 w-20 h-full border-2 border-dashed transition-all duration-200"
            :class="[dragTarget === 'left' ? 'border-primary  scale-105 opacity-100' : 'border-base-content/10 scale-100 opacity-50']">
        </div>
        <div class="absolute right-0 top-0 w-20 h-full border-2 border-dashed transition-all duration-200"
            :class="[dragTarget === 'right' ? 'border-primary  scale-105 opacity-100' : 'border-base-content/10 scale-100 opacity-50']">
        </div>
        <div class="absolute left-0 top-0 w-full h-20 border-2 border-dashed transition-all duration-200"
            :class="[dragTarget === 'top' ? 'border-primary  scale-105 opacity-100' : 'border-base-content/10 scale-100 opacity-50']">
        </div>
        <div class="absolute left-0 bottom-0 w-full h-20 border-2 border-dashed transition-all duration-200"
            :class="[dragTarget === 'bottom' ? 'border-primary  scale-105 opacity-100' : 'border-base-content/10 scale-100 opacity-50']">
        </div>
    </div>

    <div :class="[sidebarClasses, animationClass]" @mousedown="startDrag">
        <div :class="containerClasses">
            <div class="tooltip" :class="[tooltipClass, halloweenActive ? 'tooltip-warning' : 'tooltip-accent']"
                :data-tip="t('navigation.home') + (halloweenActive ? ' — ' + halloweenGreeting : '')">
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


            <div v-if="isAuthenticated" class="tooltip tooltip-accent" :class="tooltipClass"
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

            <div class="tooltip tooltip-accent" :class="tooltipClass" :data-tip="t('navigation.settings')">
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
            <div class="tooltip tooltip-accent" :class="tooltipClass" :data-tip="t('navigation.customization')">
                <button class="btn btn-ghost btn-square rounded-lg transition-all sidebar-btn" :class="{
                    'bg-primary text-primary-content shadow-lg scale-110':
                        activeTab === 'customization',
                }" @click="changeTab('customization')">
                    <SlidersVertical class="w-5 h-5" />
                </button>
            </div>

            <div v-show="isAltPressed" class="tooltip tooltip-accent" :class="tooltipClass"
                :data-tip="t('navigation.logs')">
                <button class="btn btn-ghost btn-square rounded-lg sidebar-btn" :class="{
                    'bg-primary text-primary-content shadow-lg scale-110':
                        activeTab === 'app_logs',
                }" @click="changeTab('app_logs')">
                    <Terminal class="w-5 h-5" />
                </button>
            </div>
        </div>

        <div :class="footerClasses">
            <div v-if="isAuthenticated && isAdmin" class="tooltip tooltip-accent" :class="tooltipClass"
                :data-tip="t('navigation.admin')">
                <button class="btn btn-ghost btn-square rounded-lg transition-all relative sidebar-btn"
                    @click="changeTab('admin')" :class="{
                        'bg-primary text-primary-content shadow-lg scale-110':
                            activeTab === 'admin',
                    }">
                    <ShieldAlert class="w-5 h-5" />
                </button>
            </div>

            <div class="tooltip tooltip-accent" :class="tooltipClass" :data-tip="isAuthenticated
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

            <div class="tooltip tooltip-accent" :class="tooltipClass" :data-tip="t('navigation.about')">
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


.sidebar-hidden-left {
    transform: translateX(-28px);
    opacity: 0;
    pointer-events: none;
}

.sidebar-hidden-right {
    transform: translateX(28px);
    opacity: 0;
    pointer-events: none;
}

.sidebar-hidden-top {
    transform: translateY(-28px);
    opacity: 0;
    pointer-events: none;
}

.sidebar-hidden-bottom {
    transform: translateY(28px);
    opacity: 0;
    pointer-events: none;
}

.sidebar-entered {
    transform: translateX(0) translateY(0);
    opacity: 1;
    transition: transform 1.6s cubic-bezier(0.2, 0.9, 0.2, 1), opacity 0.5s ease;
}

.sidebar-hidden-left .flex>*,
.sidebar-hidden-left .mt-auto>*,
.sidebar-hidden-right .flex>*,
.sidebar-hidden-right .mt-auto>*,
.sidebar-hidden-top .flex>*,
.sidebar-hidden-top .mt-auto>*,
.sidebar-hidden-bottom .flex>*,
.sidebar-hidden-bottom .mt-auto>* {
    opacity: 0;
    transform: scale(0.995);
}

.sidebar-entered .flex>*,
.sidebar-entered .mt-auto>* {
    opacity: 1;
    transform: scale(1);
    transition: transform 0.42s cubic-bezier(0.2, 0.9, 0.2, 1), opacity 0.42s ease;
}

/* Stagger delays */
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

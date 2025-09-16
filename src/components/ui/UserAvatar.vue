<template>
    <div class="avatar placeholder avatar-clickable"
        :class="{ relative: showStatus, clickable: isClickable || !!resolvedSrc }" @click="handleClick">
        <div class="bg-base-100 text-primary-content rounded-full flex content-center overflow-hidden"
            :class="[sizeClasses, backgroundClass, { 'avatar-inner': !resolvedSrc }]">
            <img v-if="resolvedSrc && !imageError" :src="resolvedSrc" alt="avatar" class="w-full h-full object-cover"
                @error="onImageError" />
            <template v-else-if="resolvedSrc && imageError">
                <div class="w-full h-full flex items-center justify-center bg-transparent">
                    <ImageOff class="text-base-content" width="48" height="48" />
                </div>
            </template>
            <span class="font-bold text-primary flex justify-center items-center avatar-text" :class="textSizeClass"
                v-else>
                {{ getInitials(displayName) }}
            </span>
        </div>

        <div v-if="showStatus"
            class="absolute -bottom-1 -right-1 w-4 h-4 rounded-full border-2 border-base-200 bg-base-200"
            :class="[statusClass, { 'status-indicator': isOnline }]"></div>
    </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { globalUserStatus } from '../../composables/useUserStatus';
import { ImageOff } from 'lucide-vue-next';

interface Props {
    name: string;
    size?: 'sm' | 'md' | 'lg' | 'xl';
    showStatus?: boolean;
    isOnline?: boolean;
    backgroundClass?: string;
    isClickable?: boolean;
    src?: string | null;
    originalSrc?: string | null;
}

const props = withDefaults(defineProps<Props>(), {
    size: 'md',
    showStatus: false,
    isOnline: false,
    backgroundClass: 'bg-base-100',
    isClickable: false,
    src: null,
    originalSrc: null,
});

const sizeClasses = computed(() => {
    switch (props.size) {
        case 'sm':
            return 'w-8 h-8';
        case 'md':
            return 'w-10 h-10';
        case 'lg':
            return 'w-16 h-16';
        case 'xl':
            return 'w-20 h-20';
        default:
            return 'w-10 h-10';
    }
});

const textSizeClass = computed(() => {
    switch (props.size) {
        case 'sm':
            return 'text-lg';
        case 'md':
            return 'text-xl';
        case 'lg':
            return 'text-2xl';
        case 'xl':
            return 'text-3xl';
        default:
            return 'text-xl';
    }
});

const statusClass = computed(() => {
    return props.isOnline ? 'bg-success' : 'bg-base-300';
});

const displayName = computed(() => {
    return globalUserStatus.isStreamer.value ? 'Streamer' : props.name;
});

const resolvedSrc = computed(() => {
    if (globalUserStatus.isStreamer.value) return null;
    return props.src || props.originalSrc || null;
});

const imageError = ref(false);

const onImageError = () => {
    imageError.value = true;
};

watch(
    () => resolvedSrc.value,
    (newVal, oldVal) => {
        if (newVal !== oldVal) {
            imageError.value = false;
        }
    }
);

watch(
    () => props.src,
    (newVal, oldVal) => {
        if (newVal !== oldVal) imageError.value = false;
    }
);

const getInitials = (name: string): string => {
    if (!name) return '?';
    return name.charAt(0).toUpperCase();
};

const emit = defineEmits(['click']);

const handleClick = () => {
    if (props.isClickable && resolvedSrc.value) {
        emit('click', resolvedSrc.value);
    }
};
</script>

<style scoped>
.avatar-clickable {
    transition: all 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94);
}

.avatar-clickable.clickable {
    cursor: pointer;
}

.avatar-clickable.clickable:hover {
    transform: scale(1.1);
}

.avatar-clickable.clickable:hover .avatar-inner {
    box-shadow: 0 8px 25px rgba(0, 0, 0, 0.15);
    border: 2px solid hsl(var(--p));
}

.avatar-clickable.clickable:active {
    transform: scale(0.95);
    transition: transform 0.1s ease;
}

.avatar-inner {
    transition: all 0.3s cubic-bezier(0.25, 0.46, 0.45, 0.94);
    position: relative;
    overflow: hidden;
}

.avatar-inner::before {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: linear-gradient(45deg,
            transparent 30%,
            rgba(255, 255, 255, 0.1) 50%,
            transparent 70%);
    transform: translateX(-100%);
    transition: transform 0.6s ease;
}

.avatar-clickable.clickable:hover .avatar-inner::before {
    transform: translateX(100%);
}

.avatar-text {
    transition: all 0.3s ease;
    z-index: 1;
}

.status-indicator {
    transition: all 0.3s ease;
    z-index: 2;
}

.avatar-clickable.clickable:hover .status-indicator {
    transform: scale(1.2);
    box-shadow: 0 0 10px rgba(0, 255, 0, 0.5);
}
</style>

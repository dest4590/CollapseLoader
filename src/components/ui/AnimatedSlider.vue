<template>
    <div class="relative w-full cursor-pointer select-none" @mousedown="startDrag" @mousemove="drag" @mouseup="stopDrag"
        @mouseleave="stopDrag" ref="slider">
        <div class="absolute top-1/2 left-0 w-full h-2 bg-base-300 rounded-full -translate-y-1/2" />

        <div class="absolute top-1/2 left-0 h-2 bg-primary rounded-full -translate-y-1/2 transition-all duration-200"
            :style="{ width: animatedValue + '%' }" />

        <div class="absolute top-1/2 bg-base-100 border-2 border-primary rounded-full w-4 h-4 -translate-y-1/2 -translate-x-1/2 shadow-md hover:shadow-lg focus:outline-none focus:ring-2 focus:ring-primary focus:ring-opacity-50 transition-all duration-200"
            :style="{ left: animatedValue + '%' }" />
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onUnmounted } from 'vue';

const props = defineProps<{
    modelValue: number;
    min: number;
    max: number;
}>();
const emit = defineEmits(['update:modelValue']);

const slider = ref<HTMLElement | null>(null);
const isDragging = ref(false);
const value = ref(props.modelValue);

watch(
    () => props.modelValue,
    (v) => (value.value = v)
);

const animatedValue = computed(() => {
    return ((value.value - props.min) / (props.max - props.min)) * 100;
});

const startDrag = (e: MouseEvent) => {
    isDragging.value = true;
    updateValue(e.clientX);
    window.addEventListener('mouseup', stopDrag);
    window.addEventListener('mousemove', drag);
};

const drag = (e: MouseEvent) => {
    if (isDragging.value) {
        updateValue(e.clientX);
    }
};

const stopDrag = () => {
    isDragging.value = false;
    window.removeEventListener('mouseup', stopDrag);
    window.removeEventListener('mousemove', drag);
};

onUnmounted(() => {
    window.removeEventListener('mouseup', stopDrag);
    window.removeEventListener('mousemove', drag);
});

const updateValue = (clientX: number) => {
    if (!slider.value) return;
    const rect = slider.value.getBoundingClientRect();
    const percent = Math.min(
        Math.max((clientX - rect.left) / rect.width, 0),
        1
    );
    const newValue = Math.round(props.min + percent * (props.max - props.min));
    value.value = newValue;
    emit('update:modelValue', newValue);
};
</script>

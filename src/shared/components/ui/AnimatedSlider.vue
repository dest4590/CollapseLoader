<template>
    <div
        ref="slider"
        class="relative w-full h-6 select-none touch-none"
        role="slider"
        tabindex="0"
        :aria-label="label"
        :aria-valuemin="min"
        :aria-valuemax="max"
        :aria-valuenow="internalValue"
        :aria-valuetext="`${internalValue} / ${max}`"
        @pointerdown="startDrag"
        @pointermove="drag"
        @pointerup="stopDrag"
        @pointercancel="stopDrag"
        @keydown="onKeydown"
    >
        <div
            class="absolute inset-x-0 top-1/2 h-2 -translate-y-1/2 rounded-full bg-base-300"
        />

        <div
            class="absolute left-0 top-1/2 h-2 -translate-y-1/2 rounded-full bg-primary transition-all duration-200"
            :style="fillStyle"
        />

        <div
            class="absolute top-1/2 h-4 w-4 -translate-x-1/2 -translate-y-1/2 rounded-full border-2 border-primary bg-base-100 shadow-md transition-all duration-200 hover:shadow-lg focus-visible:shadow-lg"
            :style="thumbStyle"
        />
    </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from "vue";

const props = defineProps<{
    modelValue: number;
    min?: number;
    max?: number;
    step?: number;
    label?: string;
}>();

const emit = defineEmits<{
    (event: "update:modelValue", value: number): void;
}>();

const slider = ref<HTMLElement | null>(null);
const isDragging = ref(false);
const internalValue = ref(props.modelValue ?? 0);

const min = computed(() => (Number.isFinite(props.min) ? props.min! : 0));
const max = computed(() => (Number.isFinite(props.max) ? props.max! : 100));
const step = computed(() => (props.step && props.step > 0 ? props.step : 1));
const label = computed(() => props.label ?? "Slider control");

const clampedValue = (value: number) => {
    const minValue = Math.min(min.value, max.value);
    const maxValue = Math.max(min.value, max.value);
    const stepped =
        minValue + Math.round((value - minValue) / step.value) * step.value;
    return Math.min(maxValue, Math.max(minValue, stepped));
};

watch(
    () => props.modelValue,
    (value) => {
        internalValue.value = clampedValue(value ?? internalValue.value);
    }
);

const percent = computed(() => {
    if (max.value === min.value) return 0;
    return ((internalValue.value - min.value) / (max.value - min.value)) * 100;
});

const fillStyle = computed(() => ({ width: `${percent.value}%` }));
const thumbStyle = computed(() => ({ left: `${percent.value}%` }));

const emitValue = (value: number) => {
    const nextValue = clampedValue(value);
    if (nextValue !== internalValue.value) {
        internalValue.value = nextValue;
        emit("update:modelValue", nextValue);
    }
};

const getValueFromPointer = (clientX: number) => {
    if (!slider.value) return internalValue.value;
    const rect = slider.value.getBoundingClientRect();
    const relativeX = Math.min(Math.max(clientX - rect.left, 0), rect.width);
    const ratio = rect.width > 0 ? relativeX / rect.width : 0;
    const raw = min.value + ratio * (max.value - min.value);
    return clampedValue(raw);
};

const startDrag = (event: PointerEvent) => {
    if (!slider.value) return;
    event.preventDefault();
    slider.value.setPointerCapture(event.pointerId);
    isDragging.value = true;
    emitValue(getValueFromPointer(event.clientX));
};

const drag = (event: PointerEvent) => {
    if (!isDragging.value) return;
    event.preventDefault();
    emitValue(getValueFromPointer(event.clientX));
};

const stopDrag = (event: PointerEvent) => {
    if (!slider.value) return;
    isDragging.value = false;
    slider.value.releasePointerCapture(event.pointerId);
};

const onKeydown = (event: KeyboardEvent) => {
    let delta = 0;

    switch (event.key) {
        case "ArrowLeft":
        case "ArrowDown":
            delta = -step.value;
            break;
        case "ArrowRight":
        case "ArrowUp":
            delta = step.value;
            break;
        case "PageDown":
            delta = -step.value * 5;
            break;
        case "PageUp":
            delta = step.value * 5;
            break;
        case "Home":
            emitValue(min.value);
            event.preventDefault();
            return;
        case "End":
            emitValue(max.value);
            event.preventDefault();
            return;
        default:
            return;
    }

    emitValue(internalValue.value + delta);
    event.preventDefault();
};
</script>

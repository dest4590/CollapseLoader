<template>
    <div>
        <div class="flex justify-between items-center mb-2">
            <label class="text-sm font-medium text-base-content">{{
                label
            }}</label>
            <div class="flex items-center gap-2">
                <span class="text-sm text-base-content/60"
                    >{{ modelValue }}px</span
                >
                <button
                    class="btn btn-ghost btn-xs opacity-50 hover:opacity-100"
                    @click="$emit('update:modelValue', defaultValue)"
                >
                    <RotateCcw class="w-3 h-3" />
                </button>
            </div>
        </div>
        <input
            type="range"
            min="0"
            max="60"
            step="1"
            class="range range-primary range-sm"
            :value="modelValue"
            @input="
                $emit(
                    'update:modelValue',
                    Number(($event.target as HTMLInputElement).value)
                )
            "
        />
    </div>
</template>

<script setup lang="ts">
import { RotateCcw } from "@lucide/vue";

defineProps<{
    label: string;
    modelValue: number;
    defaultValue: number;
}>();

defineEmits<{
    "update:modelValue": [value: number];
}>();
</script>

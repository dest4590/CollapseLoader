<template>
    <div class="form-control">
        <label class="label text-sm font-medium text-base-content">{{
            label
        }}</label>
        <input
            type="color"
            class="input input-bordered w-full h-10 p-0 rounded-md border-base-300"
            :value="modelValue"
            @input="
                $emit(
                    'update:modelValue',
                    ($event.target as HTMLInputElement).value || null
                )
            "
        />
    </div>
</template>

<script setup lang="ts">
defineProps<{
    label: string;
    modelValue: string | null;
}>();

defineEmits<{
    "update:modelValue": [value: string | null];
}>();
</script>

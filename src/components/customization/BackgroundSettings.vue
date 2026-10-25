<template>
    <div class="mt-8">
        <h3 class="text-xl font-semibold mb-4 text-base-content">
            {{ t("customization.background_title") }}
        </h3>
        <div class="grid grid-cols-1 lg:grid-cols-12 gap-6">
            <div class="lg:col-span-12">
                <label class="label text-sm font-medium text-base-content">{{
                    t("customization.background_image")
                }}</label>
                <div class="relative">
                    <input
                        type="text"
                        class="input input-bordered w-full pr-10"
                        :value="backgroundImage"
                        :placeholder="
                            t('customization.background_image_placeholder')
                        "
                        @input="
                            $emit(
                                'update:backgroundImage',
                                ($event.target as HTMLInputElement).value
                            )
                        "
                    />
                    <button
                        v-if="backgroundImage"
                        class="absolute right-2 top-1/2 -translate-y-1/2 btn btn-xs btn-ghost"
                        @click="$emit('update:backgroundImage', '')"
                    >
                        &times;
                    </button>
                </div>
                <p class="text-xs text-base-content/50 mt-1">
                    {{ t("customization.background_image_help") }}
                </p>
            </div>

            <div class="lg:col-span-6">
                <div class="flex justify-between mb-2">
                    <label class="text-sm font-medium text-base-content">{{
                        t("customization.background_blur")
                    }}</label>
                    <span class="text-xs font-mono"
                        >{{ backgroundBlur ?? 0 }}px</span
                    >
                </div>
                <input
                    type="range"
                    min="0"
                    max="20"
                    step="1"
                    class="range range-primary range-sm"
                    :value="backgroundBlur ?? 0"
                    @input="
                        $emit(
                            'update:backgroundBlur',
                            Number(($event.target as HTMLInputElement).value)
                        )
                    "
                />
            </div>

            <div class="lg:col-span-6">
                <div class="flex justify-between mb-2">
                    <label class="text-sm font-medium text-base-content">{{
                        t("customization.background_opacity")
                    }}</label>
                    <span class="text-xs font-mono"
                        >{{ backgroundOpacity ?? 100 }}%</span
                    >
                </div>
                <input
                    type="range"
                    min="0"
                    max="100"
                    step="1"
                    class="range range-primary range-sm"
                    :value="backgroundOpacity ?? 100"
                    @input="
                        $emit(
                            'update:backgroundOpacity',
                            Number(($event.target as HTMLInputElement).value)
                        )
                    "
                />
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";

const { t } = useI18n();

defineProps<{
    backgroundImage: string | null;
    backgroundBlur: number | null;
    backgroundOpacity: number | null;
}>();

defineEmits<{
    "update:backgroundImage": [value: string];
    "update:backgroundBlur": [value: number];
    "update:backgroundOpacity": [value: number];
}>();
</script>

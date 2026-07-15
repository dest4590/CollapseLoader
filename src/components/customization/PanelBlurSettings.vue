<template>
    <div class="card bg-base-200 shadow-md border border-base-300 mt-6">
        <div class="card-body">
            <h2 class="card-title flex items-center gap-2 mb-4">
                <Blend class="w-5 h-5 text-primary" />
                {{ t("customization.panel_blur_title") }}
                <button
                    class="btn btn-xs ml-auto"
                    :class="
                        disableBlur
                            ? 'btn-error'
                            : 'btn-ghost opacity-60 hover:opacity-100'
                    "
                    @click="$emit('update:disableBlur', !disableBlur)"
                >
                    <EyeOff v-if="disableBlur" class="w-3.5 h-3.5" />
                    <Eye v-else class="w-3.5 h-3.5" />
                    {{
                        disableBlur
                            ? t("customization.blur_disabled")
                            : t("customization.disable_blur")
                    }}
                </button>
            </h2>

            <div class="flex flex-col gap-4 max-w-md">
                <BlurSlider
                    :label="t('customization.spotlight_blur')"
                    :model-value="spotlightBlur"
                    :default-value="24"
                    @update:model-value="(v) => $emit('update:spotlightBlur', v)"
                />
                <BlurSlider
                    :label="t('customization.history_blur')"
                    :model-value="historyBlur"
                    :default-value="20"
                    @update:model-value="(v) => $emit('update:historyBlur', v)"
                />
                <BlurSlider
                    :label="t('customization.notifications_blur')"
                    :model-value="notificationsBlur"
                    :default-value="20"
                    @update:model-value="(v) => $emit('update:notificationsBlur', v)"
                />
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { Blend, Eye, EyeOff } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import BlurSlider from "./BlurSlider.vue";

const { t } = useI18n();

defineProps<{
    spotlightBlur: number;
    historyBlur: number;
    notificationsBlur: number;
    disableBlur: boolean;
}>();

defineEmits<{
    "update:spotlightBlur": [value: number];
    "update:historyBlur": [value: number];
    "update:notificationsBlur": [value: number];
    "update:disableBlur": [value: boolean];
}>();
</script>

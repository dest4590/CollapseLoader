<template>
    <div class="card bg-base-200 shadow-md border border-base-300 mb-6">
        <div class="card-body p-6">
            <h2 class="card-title flex items-center gap-2 mb-6">
                <Palette class="w-6 h-6 text-primary" />
                {{ t("theme.colors") }}
            </h2>

            <div class="mb-8">
                <h3 class="text-xl font-semibold mb-4 text-base-content">
                    {{ t("theme.base_colors") }}
                </h3>
                <div
                    class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4"
                >
                    <ColorInput
                        :label="t('theme.base100')"
                        :model-value="base100"
                        @update:model-value="(v) => $emit('update:base100', v)"
                    />
                    <ColorInput
                        :label="t('theme.base200')"
                        :model-value="base200"
                        @update:model-value="(v) => $emit('update:base200', v)"
                    />
                    <ColorInput
                        :label="t('theme.base300')"
                        :model-value="base300"
                        @update:model-value="(v) => $emit('update:base300', v)"
                    />
                    <ColorInput
                        :label="t('theme.base_content')"
                        :model-value="baseContent"
                        @update:model-value="
                            (v) => $emit('update:baseContent', v)
                        "
                    />
                </div>
            </div>

            <div class="mb-8">
                <h3 class="text-xl font-semibold mb-4 text-base-content">
                    {{ t("theme.primary_secondary_accent") }}
                </h3>
                <div
                    class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-5 gap-4"
                >
                    <ColorInput
                        :label="t('theme.primary')"
                        :model-value="primaryColorOverride"
                        @update:model-value="
                            (v) => $emit('update:primaryColorOverride', v)
                        "
                    />
                    <ColorInput
                        :label="t('theme.primary_content')"
                        :model-value="primaryContent"
                        @update:model-value="
                            (v) => $emit('update:primaryContent', v)
                        "
                    />
                    <ColorInput
                        :label="t('theme.secondary')"
                        :model-value="secondary"
                        @update:model-value="
                            (v) => $emit('update:secondary', v)
                        "
                    />
                    <ColorInput
                        :label="t('theme.secondary_content')"
                        :model-value="secondaryContent"
                        @update:model-value="
                            (v) => $emit('update:secondaryContent', v)
                        "
                    />
                    <ColorInput
                        :label="t('theme.accent')"
                        :model-value="accent"
                        @update:model-value="(v) => $emit('update:accent', v)"
                    />
                </div>
            </div>

            <div class="mb-8">
                <h3 class="text-xl font-semibold mb-4 text-base-content">
                    {{ t("theme.semantic_colors") }}
                </h3>
                <div
                    class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4"
                >
                    <ColorInput
                        :label="t('theme.neutral')"
                        :model-value="neutral"
                        @update:model-value="(v) => $emit('update:neutral', v)"
                    />
                    <ColorInput
                        :label="t('theme.neutral_content')"
                        :model-value="neutralContent"
                        @update:model-value="
                            (v) => $emit('update:neutralContent', v)
                        "
                    />
                    <ColorInput
                        :label="t('theme.info')"
                        :model-value="info"
                        @update:model-value="(v) => $emit('update:info', v)"
                    />
                    <ColorInput
                        :label="t('theme.info_content')"
                        :model-value="infoContent"
                        @update:model-value="
                            (v) => $emit('update:infoContent', v)
                        "
                    />
                    <ColorInput
                        :label="t('theme.success')"
                        :model-value="success"
                        @update:model-value="(v) => $emit('update:success', v)"
                    />
                    <ColorInput
                        :label="t('theme.success_content')"
                        :model-value="successContent"
                        @update:model-value="
                            (v) => $emit('update:successContent', v)
                        "
                    />
                    <ColorInput
                        :label="t('theme.warning')"
                        :model-value="warning"
                        @update:model-value="(v) => $emit('update:warning', v)"
                    />
                    <ColorInput
                        :label="t('theme.warning_content')"
                        :model-value="warningContent"
                        @update:model-value="
                            (v) => $emit('update:warningContent', v)
                        "
                    />
                    <ColorInput
                        :label="t('theme.error')"
                        :model-value="error"
                        @update:model-value="(v) => $emit('update:error', v)"
                    />
                    <ColorInput
                        :label="t('theme.error_content')"
                        :model-value="errorContent"
                        @update:model-value="
                            (v) => $emit('update:errorContent', v)
                        "
                    />
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { Palette } from "@lucide/vue";
import { useI18n } from "vue-i18n";
import ColorInput from "./ColorInput.vue";

const { t } = useI18n();

defineProps<{
    base100: string | null;
    base200: string | null;
    base300: string | null;
    baseContent: string | null;
    primaryColorOverride: string | null;
    primaryContent: string | null;
    secondary: string | null;
    secondaryContent: string | null;
    accent: string | null;
    neutral: string | null;
    neutralContent: string | null;
    info: string | null;
    infoContent: string | null;
    success: string | null;
    successContent: string | null;
    warning: string | null;
    warningContent: string | null;
    error: string | null;
    errorContent: string | null;
}>();

defineEmits<{
    [key: `update:${string}`]: [value: string | null];
}>();
</script>

<template>
    <div class="space-y-4">
        <p class="text-sm text-base-content/70">
            {{ t("marketplace.add_theme_info") }}
        </p>
        <a
            :href="telegramLink"
            target="_blank"
            rel="noreferrer"
            class="btn btn-primary w-full gap-2"
        >
            <Send class="w-4 h-4" />
            {{ t("marketplace.contact_channel") }}
        </a>
        <button class="btn btn-ghost w-full" @click="$emit('close')">
            {{ t("common.close") }}
        </button>
    </div>
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { Send } from "@lucide/vue";

defineEmits<{
    (e: "close"): void;
}>();

const { t } = useI18n();
const telegramLink = "https://t.me/CollapseTheme";
</script>
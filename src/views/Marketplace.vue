<template>
    <div class="max-w-6xl mx-auto p-2 md:p-4">
        <div class="flex items-center mb-4">
            <button class="btn btn-ghost btn-sm mr-2" @click="goBack">
                <ArrowLeft class="w-5 h-5" />
            </button>
            <h1 class="text-2xl font-bold">{{ t("marketplace.title") }}</h1>
        </div>

        <TelegramThemeGallery />
    </div>
</template>

<script setup lang="ts">
import { useI18n } from "vue-i18n";
import TelegramThemeGallery from "../components/presets/TelegramThemeGallery.vue";
import { ArrowLeft } from "lucide-vue-next";
import { useRouter } from "../services/router";

defineEmits(["show-user-profile"]);

const { t } = useI18n();
const router = useRouter();

function goBack() {
    router.back();
}
</script>

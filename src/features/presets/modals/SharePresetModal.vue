<template>
    <div class="space-y-4">
        <div class="form-control">
            <label class="label"
                ><span class="label-text">{{
                    t("marketplace.title_label")
                }}</span></label
            >
            <input
                v-model="title"
                class="input input-bordered w-full"
                :placeholder="t('marketplace.title_placeholder')"
            />
        </div>
        <div class="form-control">
            <label class="label"
                ><span class="label-text">{{
                    t("marketplace.description_label")
                }}</span></label
            >
            <textarea
                v-model="description"
                class="textarea textarea-bordered w-full"
                rows="4"
                :placeholder="t('marketplace.description_placeholder')"
            ></textarea>
        </div>
        <div class="form-control">
            <label class="cursor-pointer label">
                <span class="label-text block">{{
                    t("marketplace.public_label")
                }}</span>
                <input type="checkbox" class="toggle" v-model="isPublic" />
            </label>
        </div>
        <div class="flex justify-end gap-2">
            <button class="btn" @click="$emit('close')">
                {{ t("common.cancel") }}
            </button>
            <button
                class="btn btn-primary"
                :disabled="loading || !title"
                @click="share"
            >
                {{
                    loading ? t("marketplace.sharing") : t("marketplace.share")
                }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from "vue";
import { themeService } from "@services/theme/themeService";
import { marketplaceService } from "@features/marketplace/marketplaceService";
import { useToast } from "@shared/composables/useToast";
import { useI18n } from "vue-i18n";

const emit = defineEmits(["shared", "close"]);

const { addToast } = useToast();
const { t } = useI18n();
const title = ref("");
const description = ref("");
const isPublic = ref(true);
const loading = ref(false);

async function share() {
    try {
        loading.value = true;
        const s = themeService.presetSettings;
        const payload = {
            name: title.value.trim(),
            description: description.value.trim(),
            customCSS: s.customCSS,
            enableCustomCSS: s.enableCustomCSS,
            base100: s.base100 || undefined,
            base200: s.base200 || undefined,
            base300: s.base300 || undefined,
            baseContent: s.baseContent || undefined,
            primary: s.primary || undefined,
            primaryContent: s.primaryContent || undefined,
            secondary: s.secondary || undefined,
            secondaryContent: s.secondaryContent || undefined,
            accent: s.accent || undefined,
            accentContent: s.accentContent || undefined,
            neutral: s.neutral || undefined,
            neutralContent: s.neutralContent || undefined,
            info: s.info || undefined,
            infoContent: s.infoContent || undefined,
            success: s.success || undefined,
            successContent: s.successContent || undefined,
            warning: s.warning || undefined,
            warningContent: s.warningContent || undefined,
            error: s.error || undefined,
            errorContent: s.errorContent || undefined,
            backgroundImage: s.backgroundImage || undefined,
            backgroundBlur: s.backgroundBlur,
            backgroundOpacity: s.backgroundOpacity,
            is_public: isPublic.value,
        };
        await marketplaceService.createPreset(payload);
        addToast(t("marketplace.shared_success"), "success");
        emit("shared");
        emit("close");
    } catch (e) {
        console.error("Failed to share preset:", e);
        addToast(t("marketplace.shared_failed"), "error");
    } finally {
        loading.value = false;
    }
}
</script>

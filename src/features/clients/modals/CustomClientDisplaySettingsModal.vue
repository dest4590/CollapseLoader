<script lang="ts" setup>
import { invoke } from "@tauri-apps/api/core";
import { useToast } from "@shared/composables/useToast";
import { onMounted, ref, watch } from "vue";
import { useI18n } from "vue-i18n";
import AnimatedDropdown from "@shared/components/ui/AnimatedDropdown.vue";

defineEmits(["close"]);

const { addToast } = useToast();
const { t } = useI18n();
const selectedOption = ref<string>("separate");

const getFlags = async () => {
    try {
        const flags = await invoke("get_flags");
        const typedFlags = flags as {
            custom_clients_display?: { value: string };
        };
        selectedOption.value =
            typedFlags.custom_clients_display?.value || "separate";
        return typedFlags;
    } catch (err) {
        console.error("Error loading flags:", err);
        addToast(`Failed to load flags: ${err}`, "error");
        return {};
    }
};

const setCustomClientsDisplay = async () => {
    try {
        await invoke("set_custom_clients_display", {
            display: selectedOption.value,
        });
    } catch (error) {
        console.error("Failed to save flags:", error);
        addToast(t("settings.save_flags_failed", { error }), "error");
    }
};

onMounted(async () => {
    await getFlags();
});

watch(selectedOption, async () => {
    await setCustomClientsDisplay();
});
</script>
<template>
    <div class="form-control w-full mb-6">
        <label class="label mb-2">
            <span class="label-text font-medium">{{
                $t("custom_clients.display_mode")
            }}</span>
        </label>
        <AnimatedDropdown
            v-model="selectedOption"
            :options="[
                {
                    value: 'separate',
                    label: $t('custom_clients.display_mode_separate'),
                },
                {
                    value: 'global',
                    label: $t('custom_clients.display_mode_global'),
                },
            ]"
            width="100%"
            field-size
        />
    </div>
    <div class="flex justify-end">
        <button class="btn btn-primary" @click="$emit('close')">
            {{ $t("common.save") }}
        </button>
    </div>
</template>

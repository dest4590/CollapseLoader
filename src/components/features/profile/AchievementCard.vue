<template>
    <div
        class="card bg-base-100 shadow-sm border border-base-200 transition-all hover:shadow-md"
        :class="{ 'opacity-60 grayscale': locked }"
    >
        <div class="card-body p-4 flex flex-row items-center gap-4">
            <div
                class="p-3 rounded-full bg-base-200 text-primary"
                :class="{ 'bg-base-300 text-base-content/50': locked }"
            >
                <component :is="iconComponent" class="w-6 h-6" />
            </div>
            <div class="flex-1">
                <h3
                    class="font-bold text-sm"
                    :class="{ 'text-base-content/50': locked }"
                >
                    {{ title }}
                </h3>
                <p class="text-xs text-base-content/70">
                    {{ description }}
                </p>
                <p
                    v-if="!locked && unlockedAt"
                    class="text-[10px] text-primary mt-1"
                >
                    {{
                        t("achievements.unlocked_at", {
                            date: formatDate(unlockedAt),
                        })
                    }}
                </p>
                <p
                    v-if="receivePercentage !== undefined"
                    class="text-[10px] text-base-content/40 mt-0.5"
                >
                    {{
                        t("achievements.owned_percentage", {
                            percent: Number(receivePercentage).toFixed(1),
                        })
                    }}
                </p>
            </div>
            <div
                v-if="locked"
                class="text-xs font-semibold px-2 py-1 bg-base-300 rounded text-base-content/60"
            >
                <Lock class="w-3 h-3" />
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from "vue";
import { useI18n } from "vue-i18n";
import * as LucideIcons from "lucide-vue-next";
import { Lock } from "lucide-vue-next";
import { formatDate } from "../../../utils/utils";

const props = defineProps<{
    achievementKey: string;
    iconName: string;
    locked: boolean;
    unlockedAt?: string | null;
    hidden?: boolean;
    receivePercentage?: number;
}>();

const { t } = useI18n();

const iconComponent = computed(() => {
    // @ts-ignore
    return LucideIcons[props.iconName] || LucideIcons.Trophy;
});

const title = computed(() => {
    if (props.locked && props.hidden) return t("achievements.secret_title");
    return t(`achievements.list.${props.achievementKey}.name`);
});

const description = computed(() => {
    if (props.locked && props.hidden)
        return t("achievements.secret_description");
    return t(`achievements.list.${props.achievementKey}.description`);
});
</script>

<template>
    <div class="flex gap-2 h-2 rounded-full mt-2">
        <div v-for="color in colors" :key="color" 
             class="flex-1 h-full rounded-full shadow-sm"
             :style="{ backgroundColor: color }">
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import type { MarketplacePreset } from '../../types/presets';

const props = defineProps<{
    preset: MarketplacePreset;
}>();

const colors = computed(() => {
    const theme = props.preset.theme ?? props.preset.preset_data ?? {};
    const keys = ['primary', 'secondary', 'accent', 'neutral', 'base100'];
    return keys
        .map(k => (theme as any)[k])
        .filter(c => c && typeof c === 'string' && c.startsWith('#'));
});
</script>

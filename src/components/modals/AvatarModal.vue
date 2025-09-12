<template>
    <div class="p-4 flex items-center justify-center">
        <div class="max-w-full max-h-[80vh] w-auto h-auto">
            <img
                v-if="displaySrc"
                :src="displaySrc"
                :alt="alt || 'avatar'"
                class="max-w-full max-h-[80vh] object-contain rounded"
            />
            <div v-else class="text-center text-sm text-base-content/60">No image available</div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
interface Props {
    src?: string | null;
    originalSrc?: string | null;
    alt?: string | null;
}

const props = withDefaults(defineProps<Props>(), {
    src: null,
    originalSrc: null,
    alt: null,
});

const displaySrc = computed(() => props.originalSrc || props.src || null);
</script>

<style scoped>
.rounded {
    border-radius: 8px;
}
</style>

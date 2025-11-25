<script setup lang="ts">
const props = defineProps<{
    title?: string;
    icon?: any;
    description?: string;
    delay?: number;
    field?: any;
}>();
</script>

<template>
    <div class="card bg-base-200 shadow-md border border-base-300 settings-card"
        :style="{ 'animation-delay': `${props.delay || 0}s` }">
        <div class="card-body p-4">
            <h2 class="card-title text-base font-semibold text-primary-focus mb-2">
                <slot name="title">
                    <component :is="props.icon" v-if="props.icon" class="w-5 h-5 text-primary" />
                    {{ props.title }}
                    <slot name="info" />
                </slot>
            </h2>

            <slot />
            <p v-if="description" class="text-xs text-base-content/70 mt-2">{{ description }}</p>
        </div>
    </div>
</template>

<style scoped>
.settings-card {
    opacity: 0;
    transform: translateY(10px);
    animation: fadeInUp 0.4s ease-out forwards;
}
</style>
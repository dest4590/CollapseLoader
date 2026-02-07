<script setup lang="ts">
const props = defineProps<{
    title?: string;
    icon?: any;
    description?: string;
    delay?: number;
    field?: any;
    layout?: 'row' | 'col';
}>();
</script>

<template>
    <div class="card bg-base-200 shadow-sm border border-base-300 settings-card hover:border-primary/20 transition-colors"
        :style="{ 'animation-delay': `${props.delay || 0}s` }">
        <div class="card-body p-3 h-full"
            :class="[layout === 'col' ? 'flex-col items-start gap-3 justify-between' : 'flex-row items-center justify-between gap-3']">
            <div class="flex-1 min-w-0">
                <h2 class="card-title text-sm font-semibold text-base-content flex items-center gap-2">
                    <slot name="title">
                        <div v-if="props.icon" class="p-1.5 rounded-md bg-base-300 text-primary">
                            <component :is="props.icon" class="w-4 h-4" />
                        </div>
                        {{ props.title }}
                        <slot name="info" />
                    </slot>
                </h2>
                <p v-if="description" class="text-xs text-base-content/60 mt-1 font-medium leading-relaxed">{{
                    description }}</p>
            </div>

            <div :class="[layout === 'col' ? 'w-full' : 'shrink-0']">
                <slot />
            </div>
        </div>
    </div>
</template>

<style scoped>
.settings-card {
    opacity: 0;
    transform: translateY(10px);
    animation: fadeInUp 0.4s cubic-bezier(0.2, 0.8, 0.2, 1) forwards;
    height: 100%;
    width: 100%;
    display: flex;
}

.settings-card .card-body {
    flex: 1;
    display: flex;
    width: 100%;
}
</style>
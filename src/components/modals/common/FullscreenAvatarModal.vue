<template>
    <div class="w-full h-full flex items-center justify-center p-6">
        <div class="relative max-w-[95%] max-h-[95%] w-full">
            <div class="w-full h-full flex items-center justify-center overflow-hidden rounded">
                <img :src="src" :alt="alt || 'avatar'" class="object-contain w-full h-full bg-base-200" />
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
defineProps<{ src: string; alt?: string }>();
defineEmits(['close']);
</script>

<style scoped>
.modal-fullscreen-image {
    display: flex;
    align-items: center;
    justify-content: center;
}
</style>

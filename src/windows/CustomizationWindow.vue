<script setup lang="ts">
import Customization from "../views/Customization.vue";
import { onMounted } from "vue";
import { themeService } from "@core/theme/themeService";

onMounted(() => {
    themeService.loadSettings();
});
</script>

<template>
    <div
        class="h-screen w-screen bg-base-100 overflow-y-auto overflow-x-hidden p-6"
    >
        <Customization />
    </div>
</template>

<style>
html,
body,
#app {
    height: 100%;
    margin: 0;
    padding: 0;
}
</style>

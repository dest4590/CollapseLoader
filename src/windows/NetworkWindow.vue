<script setup lang="ts">
import NetworkDebug from "../views/NetworkDebug.vue";
import { onMounted } from "vue";
import { initNetworkDebug } from "../services/networkDebugService";

onMounted(() => {
    initNetworkDebug();
});
</script>

<template>
    <div class="h-screen w-screen bg-base-100 overflow-hidden">
        <NetworkDebug />
    </div>
</template>

<style>
html,
body,
#app {
    height: 100%;
    margin: 0;
    padding: 0;
    overflow: hidden;
}
</style>

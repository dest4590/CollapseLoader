<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { ref, onMounted } from 'vue';
import { Minus, Square, X } from 'lucide-vue-next';

const appWindow = getCurrentWindow();
const minimize = () => appWindow.minimize();
const maximize = () => appWindow.toggleMaximize();
const close = () => appWindow.close();

const version = ref('');
const codename = ref('');
const commitFull = ref('');
const commitShort = ref('');
const isVisible = ref(false);

const fetchVersion = async () => {
    try {
        const result = await invoke('get_version');
        const data = typeof result === 'string' ? JSON.parse(result) : result;
        version.value = data.version || '';
        codename.value = data.codename || '';
        commitFull.value = String(data.commitHash || '');
        commitShort.value = commitFull.value ? String(commitFull.value).slice(0, 7) : '';
    } catch (e) {
        console.error('Titlebar: failed to fetch version info', e);
    }
};

onMounted(() => {
    fetchVersion();
    setTimeout(() => {
        isVisible.value = true;
    }, 300);
});
</script>
// TODO: improve titlebar animations
<template>
    <div
      data-tauri-drag-region
      :class="[
        'fixed top-0 left-0 right-0 flex justify-between items-center h-10 select-none z-[100]',
        'bg-base-100/95 backdrop-blur-md border-b border-base-300/50',
        'transition-all duration-1000 ease-out',
        isVisible ? 'opacity-100 translate-y-0 scale-100' : 'opacity-0 -translate-y-8 scale-95'
      ]"
    >
        <div class="flex items-center pl-3 gap-2 pointer-events-none">
            <span
              :class="[
                'text-[12px] font-semibold tracking-tight text-base-content/80 uppercase',
                'transition-all duration-1000 delay-200 ease-out',
                isVisible ? 'opacity-100 translate-x-0 scale-100' : 'opacity-0 -translate-x-8 scale-90'
              ]"
            >
                CollapseLoader
            </span>
            <span
              v-if="version || codename || commitShort"
              :title="(version ? `v${version}` : '') + (codename ? ` · ${codename}` : '') + (commitFull ? ` (commit ${commitFull})` : '')"
              :class="[
                'text-[10px] font-medium tracking-tight text-base-content/50 uppercase select-none',
                'transition-all duration-1000 delay-400 ease-out',
                isVisible ? 'opacity-100 translate-x-0' : 'opacity-0 -translate-x-6'
              ]"
            >
              <template v-if="version">v{{ version }}</template>
              <template v-if="codename"> <span class="mx-1">·</span> {{ codename }}</template>
              <template v-if="commitShort"> <span class="mx-1">·</span> {{ commitShort }}</template>
            </span>
        </div>
        <div
          :class="[
            'flex h-full',
            'transition-all duration-1000 delay-300 ease-out',
            isVisible ? 'opacity-100 translate-x-0 scale-100' : 'opacity-0 translate-x-8 scale-90'
          ]"
        >
            <button
              :class="[
                'w-12 h-full flex items-center justify-center border-none bg-transparent cursor-pointer',
                'text-base-content/70 transition-all duration-200',
                'hover:bg-base-300/80 hover:text-base-content hover:scale-110 active:scale-95'
              ]"
              @click="minimize"
              title="Minimize"
            >
                <Minus :size="14" :stroke-width="2.5"/>
            </button>
            <button
              :class="[
                'w-12 h-full flex items-center justify-center border-none bg-transparent cursor-pointer',
                'text-base-content/70 transition-all duration-200',
                'hover:bg-base-300/80 hover:text-base-content hover:scale-110 active:scale-95'
              ]"
              @click="maximize"
              title="Maximize"
            >
                <Square :size="12" :stroke-width="2.5"/>
            </button>
            <button
              :class="[
                'w-12 h-full flex items-center justify-center border-none bg-transparent cursor-pointer',
                'text-base-content/70 transition-all duration-200',
                'hover:bg-error hover:text-error-content hover:scale-110 hover:rotate-90 active:scale-95'
              ]"
              @click="close"
              title="Close"
            >
                <X :size="14" :stroke-width="2.5"/>
            </button>
        </div>
    </div>
</template>
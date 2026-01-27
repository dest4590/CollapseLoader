<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window';
import { invoke } from '@tauri-apps/api/core';
import { ref, onMounted, onUnmounted } from 'vue';
import { Minus, Square, X } from 'lucide-vue-next';

const appWindow = getCurrentWindow();
const minimize = async () => appWindow.minimize();
const maximize = async () => {
    await appWindow.toggleMaximize();
    await checkMax();
};
const close = async () => appWindow.close();

const isMaximized = ref(false);

const checkMax = async () => {
    try {
        isMaximized.value = await appWindow.isMaximized();
    } catch {
        isMaximized.value = false;
    }
};

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

let resizeUnlisten: any;
onMounted(() => {
    fetchVersion();
    checkMax();
    resizeUnlisten = appWindow.listen('tauri://resize', checkMax);
    setTimeout(() => {
        isVisible.value = true;
    }, 300);
});

onUnmounted(() => {
    if (resizeUnlisten && typeof resizeUnlisten.then === 'function') {
        resizeUnlisten.then((un: () => void) => un());
    }
});
</script>

<template>
    <div data-tauri-drag-region :class="[
        'titlebar fixed top-0 left-0 right-0 flex justify-between items-center h-10 select-none z-100',
        isVisible ? 'opacity-100' : 'opacity-0'
    ]">
        <div class="flex items-center pl-3 gap-2 pointer-events-none">
            <span :class="[
                'text-[12px] font-semibold tracking-tight text-base-content/80 uppercase',
                'transition-all duration-1000 delay-200 ease-out',
                isVisible ? 'opacity-100 translate-x-0 scale-100' : 'opacity-0 -translate-x-8 scale-90'
            ]">
                CollapseLoader
            </span>
            <span v-if="version || codename || commitShort"
                :title="(version ? `v${version}` : '') + (codename ? ` · ${codename}` : '') + (commitFull ? ` (commit ${commitFull})` : '')"
                :class="[
                    'text-[10px] font-medium tracking-tight text-base-content/50 uppercase select-none',
                    'transition-all duration-1000 delay-400 ease-out',
                    isVisible ? 'opacity-100 translate-x-0' : 'opacity-0 -translate-x-6'
                ]">
                <template v-if="version">v{{ version }}</template>
                <template v-if="codename"> <span class="mx-1">·</span> {{ codename }}</template>
                <template v-if="commitShort"> <span class="mx-1">·</span> {{ commitShort }}</template>
            </span>
        </div>
        <div :class="[
            'flex h-full',
            'transition-all duration-1000 delay-300 ease-out',
            isVisible ? 'opacity-100 translate-x-0 scale-100' : 'opacity-0 translate-x-8 scale-90'
        ]">
            <button class="titlebar-btn" @click="minimize" title="Minimize" aria-label="Minimize window">
                <Minus :size="14" :stroke-width="2.5" />
            </button>
            <button class="titlebar-btn" @click="maximize" :title="isMaximized ? 'Restore' : 'Maximize'"
                :aria-label="isMaximized ? 'Restore window' : 'Maximize window'">
                <Square :size="12" :stroke-width="2.5" />
            </button>
            <button class="titlebar-btn titlebar-close" @click="close" title="Close" aria-label="Close window">
                <X :size="14" :stroke-width="2.5" />
            </button>
        </div>
    </div>
</template>

<style scoped>
.titlebar {
    background: linear-gradient(180deg, rgba(255, 255, 255, 0.02), rgba(0, 0, 0, 0.02));
    backdrop-filter: blur(6px);
    border-bottom: 1px solid rgba(148, 163, 184, 0.06);
    transition: opacity .25s ease, transform .35s cubic-bezier(.2, .9, .2, 1);
}

.titlebar-btn {
    width: 48px;
    height: 100%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    transition: transform .18s ease, background-color .12s ease, color .12s ease, box-shadow .12s ease;
    cursor: pointer;
    outline: none;
}

.titlebar-btn:hover {
    transform: scale(1.08);
    background-color: rgba(148, 163, 184, 0.06);
    box-shadow: 0 6px 18px rgba(2, 6, 23, 0.06);
}

.titlebar-btn:active {
    transform: scale(.96);
}

.titlebar-btn:focus {
    box-shadow: 0 0 0 3px rgba(99, 102, 241, 0.12);
    border-radius: 6px;
}

.titlebar-close:hover {
    background-color: rgba(220, 38, 38, 0.08);
    color: rgba(220, 38, 38, 0.95);
}

@media (prefers-reduced-motion: reduce) {

    .titlebar,
    .titlebar-btn {
        transition: none !important;
        transform: none !important;
    }
}
</style>
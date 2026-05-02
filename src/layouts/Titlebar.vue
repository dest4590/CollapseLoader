<script setup lang="ts">
import { getCurrentWindow } from "@tauri-apps/api/window";
import { invoke } from "@tauri-apps/api/core";
import { ref, onMounted, onUnmounted, computed } from "vue";
import { Minus, Square, X } from "lucide-vue-next";

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

const version = ref("");
const codename = ref("");
const commitFull = ref("");
const commitShort = ref("");
const commitMessage = ref("");
const apiUrl = ref("");
const cdnUrl = ref("");
const isVisible = ref(false);

const tooltipContent = computed(() => {
    const lines: string[] = [];
    if (version.value)
        lines.push(
            `BUILD: v${version.value}${codename.value ? ` (${codename.value})` : ""}`
        );
    if (apiUrl.value) lines.push(`SERVER: ${apiUrl.value}`);
    if (cdnUrl.value) lines.push(`CDN: ${cdnUrl.value}`);
    if (commitFull.value) lines.push(`COMMIT: ${commitFull.value}`);
    if (commitMessage.value) lines.push(`\nMSG: ${commitMessage.value}`);
    return lines.length > 0 ? lines.join("\n") : "No version info";
});

const fetchVersion = async () => {
    try {
        const result = await invoke("get_version");
        const data = typeof result === "string" ? JSON.parse(result) : result;
        version.value = data.version || "";
        codename.value = data.codename || "";
        commitFull.value = String(data.commitHash || "");
        commitShort.value = commitFull.value
            ? String(commitFull.value).slice(0, 7)
            : "";
        commitMessage.value = data.commitMessage || "";
    } catch (e) {
        console.error("Titlebar: failed to fetch version info", e);
    }

    try {
        const s = await invoke("get_api_url");
        apiUrl.value = typeof s === "string" ? s : String(s);
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (e) {
        apiUrl.value = "";
    }
    try {
        const c = await invoke("get_cdn_url");
        cdnUrl.value = typeof c === "string" ? c : String(c);
        // eslint-disable-next-line @typescript-eslint/no-unused-vars
    } catch (e) {
        cdnUrl.value = "";
    }
};

let resizeUnlisten: any;
onMounted(() => {
    fetchVersion();
    checkMax();
    resizeUnlisten = appWindow.listen("tauri://resize", checkMax);
    setTimeout(() => {
        isVisible.value = true;
    }, 300);
});

onUnmounted(() => {
    if (resizeUnlisten && typeof resizeUnlisten.then === "function") {
        resizeUnlisten.then((un: () => void) => un());
    }
});
</script>

<template>
    <div
        data-tauri-drag-region
        :class="[
            'titlebar fixed top-0 left-0 right-0 flex justify-between items-center h-10 select-none z-100',
            isVisible ? 'opacity-100' : 'opacity-0',
        ]"
    >
        <div
            data-tauri-drag-region
            class="flex items-center pl-3 gap-2 h-full grow"
        >
            <span
                data-tauri-drag-region
                class="flex overflow-hidden"
                :class="[
                    'text-[14px] font-bold tracking-tight text-base-content/80 cursor-default px-0.5',
                    'transition-all duration-1000 delay-200 ease-out',
                    isVisible
                        ? 'opacity-100 translate-x-0 scale-100'
                        : 'opacity-0 -translate-x-8 scale-90',
                ]"
            >
                <span
                    v-for="(char, index) in 'CollapseLoader'.split('')"
                    :key="index"
                    :style="{ transitionDelay: `${200 + index * 50}ms` }"
                    :class="[
                        'inline-block transition-all duration-500 ease-out',
                        isVisible
                            ? 'opacity-100 translate-y-0'
                            : 'opacity-0 -translate-y-2',
                    ]"
                >
                    {{ char }}
                </span>
            </span>

            <div
                v-if="version || codename || commitShort"
                data-tauri-drag-region
                :data-tip="tooltipContent"
                :class="[
                    'tooltip tooltip-bottom tooltip-multiline cursor-default pointer-events-auto',
                    'text-[10px] font-medium tracking-tight text-base-content/50 select-none',
                    'transition-all duration-1000 delay-400 ease-out',
                    'animate-in fade-in slide-in-from-top-2',
                    isVisible
                        ? 'opacity-100 translate-y-0'
                        : 'opacity-0 -translate-y-4',
                ]"
            >
                <span data-tauri-drag-region v-if="version"
                    >v{{ version }}</span
                >
                <span data-tauri-drag-region v-if="codename">
                    <span class="mx-1">·</span> {{ codename }}</span
                >
                <span data-tauri-drag-region v-if="commitShort">
                    <span class="mx-1">·</span>
                    <span data-tauri-drag-region>{{ commitShort }}</span>
                </span>
            </div>
        </div>

        <div
            :class="[
                'flex h-full relative z-10',
                'transition-all duration-1000 delay-300 ease-out',
                isVisible
                    ? 'opacity-100 translate-y-0 scale-100'
                    : 'opacity-0 -translate-y-4 scale-95',
            ]"
        >
            <button class="titlebar-btn" @click="minimize" title="Minimize">
                <Minus :size="14" :stroke-width="2.5" />
            </button>
            <button
                class="titlebar-btn"
                @click="maximize"
                :title="isMaximized ? 'Restore' : 'Maximize'"
            >
                <Square :size="12" :stroke-width="2.5" />
            </button>
            <button
                class="titlebar-btn titlebar-close"
                @click="close"
                title="Close"
            >
                <X :size="14" :stroke-width="2.5" />
            </button>
        </div>
    </div>
</template>

<style scoped>
.titlebar {
    background: linear-gradient(
        180deg,
        rgba(255, 255, 255, 0.02),
        rgba(0, 0, 0, 0.02)
    );
    backdrop-filter: blur(6px);
    border-bottom: 1px solid rgba(148, 163, 184, 0.06);
    transition:
        opacity 0.25s ease,
        transform 0.35s cubic-bezier(0.2, 0.9, 0.2, 1);
}

.tooltip-multiline:before {
    white-space: pre-wrap;
    word-break: break-all;
    text-align: left;
    width: 320px;
    font-size: 11px;
    line-height: 1.5;
    padding: 10px 12px;
    background-color: #161616;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    box-shadow: 0 8px 24px rgba(0, 0, 0, 0.5);
    left: 0;
    transform: translateX(0);
}

.tooltip-multiline:after {
    border-bottom-color: #161616;
    left: 20px;
    transform: translateX(0);
}

.titlebar-btn {
    width: 48px;
    height: 100%;
    display: inline-flex;
    align-items: center;
    justify-content: center;
    background: transparent;
    border: none;
    transition:
        transform 0.18s ease,
        background-color 0.12s ease,
        color 0.12s ease,
        box-shadow 0.12s ease;
    cursor: pointer;
    outline: none;
}

.titlebar-btn:hover {
    transform: scale(1.08);
    background-color: rgba(148, 163, 184, 0.06);
    box-shadow: 0 6px 18px rgba(2, 6, 23, 0.06);
}

.titlebar-btn:active {
    transform: scale(0.96);
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

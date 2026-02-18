<template>
    <div
        id="preloader"
        v-if="visible"
        :class="{ 'animate-out': animateOut }"
        class="fixed inset-0 bg-base-300 flex items-center justify-center"
    >
        <BootLogs
            v-if="isDev"
            :current-progress="currentProgress / 100"
            :loading-state="loadingState"
        />

        <div
            class="flex flex-col items-center justify-center h-full w-screen relative z-10"
        >
            <div
                v-if="!halloweenActive"
                class="logo-wrapper mb-8 relative"
                :style="maskStyle"
            >
                <canvas ref="matrixCanvas" class="matrix-canvas"></canvas>
            </div>
            <div v-else class="w-48 h-48 mb-8">
                <img
                    src="../../assets/misc/ghosts.gif"
                    class="w-full h-full object-contain"
                />
            </div>

            <div class="loading-status mt-2">
                <OdometerText
                    :text="loadingState"
                    :stagger="15"
                    :duration="500"
                    class="text-lg font-bold tracking-wider uppercase font-mono"
                    :class="{
                        'text-gray-800': currentTheme === 'light',
                        'text-white': currentTheme !== 'light',
                    }"
                />
            </div>

            <div
                class="w-96 progress-container mt-6 relative"
                :class="{ invert: currentTheme === 'light' }"
            >
                <div
                    class="bg-white/5 border border-white/10 rounded-full h-1.5 overflow-hidden progress-track relative shadow-inner"
                >
                    <div
                        class="bg-white h-full progress-fill relative"
                        :style="{ width: `${displayProgress}%` }"
                    >
                        <div
                            class="absolute inset-0 bg-linear-to-r from-transparent via-white/30 to-transparent"
                        ></div>
                    </div>
                </div>

                <div
                    class="flex justify-between mt-3 text-[10px] font-mono tracking-[0.2em] text-gray-500"
                >
                    <span class="opacity-70">{{
                        $t("preloader.initializing_progress")
                    }}</span>
                    <span class="percentage-number font-bold text-gray-400">
                        {{ Math.round(displayProgress) }}%
                    </span>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import {
    computed,
    ref,
    watch,
    onBeforeUnmount,
    onMounted,
    nextTick,
} from "vue";
import BootLogs from "./BootLogs.vue";
import OdometerText from "./OdometerText.vue";

const props = defineProps({
    show: { type: Boolean, required: true },
    isDev: { type: Boolean, default: false },
    loadingState: { type: String, required: true },
    currentProgress: { type: Number, required: true },
    halloweenActive: { type: Boolean, default: false },
    currentTheme: { type: String, default: "dark" },
});

const displayProgress = ref(0);

let _progressRaf: number | null = null;
const LERP_FACTOR = 0.12;
const LERP_EPS = 0.05;

const stepProgress = () => {
    const target = props.currentProgress;
    const current = displayProgress.value;
    const next = current + (target - current) * LERP_FACTOR;
    displayProgress.value = Math.abs(target - next) < LERP_EPS ? target : next;

    if (displayProgress.value !== target) {
        _progressRaf = requestAnimationFrame(stepProgress);
    } else {
        _progressRaf = null;
    }
};

watch(
    () => props.currentProgress,
    () => {
        if (_progressRaf) cancelAnimationFrame(_progressRaf);
        _progressRaf = requestAnimationFrame(stepProgress);
    },
    { immediate: true }
);

const svgString = `
<svg xmlns="http://www.w3.org/2000/svg" width="435" height="365" fill="none" viewBox="0 0 435 365">
    <path fill="#000" d="M182.028 36.5L272.733 127.44L309.138 127.44L182.028 2.22629e-05L-7.97733e-06 182.5L182.028 365L309.138 237.56L272.733 237.56L182.028 328.5L36.4056 182.5L182.028 36.5Z" />
    <path fill="#000" d="M182.028 72.81L236.731 127.655L182.028 182.5L236.731 237.345L182.028 292.19L72.6217 182.5L182.028 72.81Z" />
    <path fill="#000" d="M254.65 36.5L345.354 127.44L381.76 127.44L254.65 1.90885e-05L236.447 18.25L236.446 18.2508L254.649 36.5008L254.65 36.5Z" />
    <path fill="#000" d="M381.76 237.56L345.44 237.56L236.49 346.793L254.65 365L381.76 237.56Z" />
    <path fill="#000" d="M249.602 148.733L216.137 182.285L249.602 215.837L249.653 215.786L249.653 215.888L401.535 215.888L401.535 215.837L435 182.285L401.535 148.733L401.484 148.783L249.653 148.783L249.602 148.733Z" />
</svg>
`;

const maskStyle = computed(() => {
    const encodedSvg = encodeURIComponent(svgString);
    const dataUrl = `url("data:image/svg+xml,${encodedSvg}")`;
    return {
        maskImage: dataUrl,
        WebkitMaskImage: dataUrl,
        maskSize: "contain",
        WebkitMaskSize: "contain",
        maskRepeat: "no-repeat",
        WebkitMaskRepeat: "no-repeat",
        maskPosition: "center",
        WebkitMaskPosition: "center",
    };
});

const matrixCanvas = ref<HTMLCanvasElement | null>(null);
let _matrixInterval: number | null = null;

const initMatrix = () => {
    const canvas = matrixCanvas.value;
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const width = 217.5;
    const height = 182.5;
    canvas.width = width * 2;
    canvas.height = height * 2;
    ctx.scale(2, 2);

    const fontSize = 7;
    const columns = Math.ceil(width / fontSize);
    const drops: number[] = [];
    for (let x = 0; x < columns; x++) drops[x] = Math.random() * -50;
    const chars = "01XYZ_<>[]!@#";

    const draw = () => {
        if (props.currentTheme === "light") {
            ctx.fillStyle = "rgba(0, 0, 0, 0.3)";
        } else {
            ctx.fillStyle = "rgba(255, 255, 255, 1)";
        }

        ctx.font = `${fontSize}px monospace`;
        if (displayProgress.value > 50) {
            ctx.shadowColor = "white";
            ctx.shadowBlur = 8;
        }

        for (let i = 0; i < drops.length; i++) {
            const text = chars[Math.floor(Math.random() * chars.length)];
            const x = i * fontSize;
            const y = drops[i] * fontSize;
            ctx.fillText(text, x, y);
            if (y > height && Math.random() > 0.95) drops[i] = 0;
            drops[i] += 1.8;
        }
    };

    if (_matrixInterval) clearInterval(_matrixInterval);
    _matrixInterval = window.setInterval(draw, 16);
};

const visible = ref(props.show);
const animateOut = ref(false);
const ANIMATE_OUT_MS = 500;

watch(
    () => props.show,
    (val) => {
        if (val) {
            animateOut.value = false;
            visible.value = true;
            if (!props.halloweenActive) nextTick(() => initMatrix());
        } else {
            animateOut.value = true;
            window.setTimeout(() => {
                animateOut.value = false;
                visible.value = false;
            }, ANIMATE_OUT_MS);
        }
    }
);

onMounted(() => {
    if (!props.halloweenActive) nextTick(() => initMatrix());
});
watch(
    () => props.halloweenActive,
    (val) => {
        if (!val) nextTick(() => initMatrix());
    }
);
onBeforeUnmount(() => {
    if (_matrixInterval) clearInterval(_matrixInterval);
    if (_progressRaf) cancelAnimationFrame(_progressRaf);
});
</script>

<style scoped>
#preloader {
    position: fixed;
    width: 100%;
    height: 100%;
    left: 0;
    top: 0;
    background-color: #050505;
    z-index: 1337;
    transition: opacity 0.3s ease;
    color: white;
}

[data-theme="light"] #preloader {
    background-color: #f0f0f0;
}

#preloader.animate-out {
    opacity: 0;
    pointer-events: none;
}

.logo-wrapper {
    width: 217.5px;
    height: 182.5px;
    position: relative;
    filter: drop-shadow(0 0 15px rgba(255, 255, 255, 0.15));
}

.matrix-canvas {
    width: 100%;
    height: 100%;
    display: block;
}
.invert {
    filter: invert(1);
}

.progress-fill {
    transition: width 1.2s cubic-bezier(0.22, 1, 0.36, 1);
    box-shadow:
        0 0 12px rgba(255, 255, 255, 0.6),
        0 0 4px rgba(255, 255, 255, 0.8);
}
</style>

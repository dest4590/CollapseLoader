<template>
    <div
        id="preloader"
        v-if="visible"
        :class="{ 'animate-out': animateOut }"
        role="status"
        aria-live="polite"
        :aria-label="loadingState"
        :aria-busy="visible"
        class="fixed inset-0 bg-base-300 flex items-center justify-center"
    >
        <BootLogs
            v-if="isDev"
            :current-progress="currentProgress / totalSteps"
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

            <span class="sr-only">{{ loadingState }}</span>

            <div class="loading-status mt-2">
                <transition name="slide-fade" mode="out-in">
                    <span
                        :key="loadingState"
                        class="text-lg font-bold tracking-wider uppercase text-white"
                        :class="{ invert: currentTheme === 'light' }"
                    >
                        {{ loadingState }}
                    </span>
                </transition>
            </div>

            <div
                class="w-96 progress-container mt-6"
                :class="{ invert: currentTheme === 'light' }"
                role="progressbar"
                :aria-valuenow="animatedProgressRounded"
                aria-valuemin="0"
                aria-valuemax="100"
                :aria-valuetext="`${animatedProgressRounded}%`"
            >
                <div
                    class="bg-gray-900 rounded-sm h-1 overflow-hidden progress-track relative"
                >
                    <div
                        class="bg-white h-full progress-fill shadow-[0_0_15px_rgba(255,255,255,0.7)]"
                        :style="{
                            transform: `scaleX(${animatedProgress / 100})`,
                            transformOrigin: 'left',
                        }"
                    ></div>
                </div>
                <div
                    class="flex justify-between mt-2 text-xs font-mono text-gray-500"
                >
                    <span>LOADING</span>
                    <span
                        :class="['percentage-number', { bump: percentageBump }]"
                    >
                        {{ animatedProgressRounded }}%
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

const props = defineProps({
    show: { type: Boolean, required: true },
    isDev: { type: Boolean, default: false },
    loadingState: { type: String, required: true },
    currentProgress: { type: Number, required: true },
    totalSteps: { type: Number, required: true },
    halloweenActive: { type: Boolean, default: false },
    currentTheme: { type: String, default: "dark" },
});

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

const progressPercent = computed(() =>
    Math.min(
        100,
        Math.max(0, (props.currentProgress / (props.totalSteps || 1)) * 100)
    )
);
const animatedProgress = ref(progressPercent.value);
const animatedProgressRounded = computed(() =>
    Math.round(animatedProgress.value)
);

let _progressRaf: number | null = null;
const animateTo = (target: number) => {
    if (_progressRaf) cancelAnimationFrame(_progressRaf);
    const step = () => {
        const current = animatedProgress.value;
        const delta = target - current;
        const ease = 0.12;
        const snapThreshold = 0.1;
        animatedProgress.value =
            Math.abs(delta) < snapThreshold ? target : current + delta * ease;
        if (Math.abs(delta) >= snapThreshold) {
            _progressRaf = requestAnimationFrame(step);
        } else {
            _progressRaf = null;
        }
    };
    if (!Number.isFinite(animatedProgress.value)) animatedProgress.value = 0;
    _progressRaf = requestAnimationFrame(step);
};
watch(progressPercent, (newVal) => animateTo(newVal));

const percentageBump = ref(false);
let _bumpTimer: number | null = null;
watch(animatedProgressRounded, () => {
    percentageBump.value = true;
    if (_bumpTimer) window.clearTimeout(_bumpTimer);
    _bumpTimer = window.setTimeout(() => (percentageBump.value = false), 150);
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

    for (let x = 0; x < columns; x++) {
        drops[x] = Math.random() * -50;
    }

    const chars = "01XYZ_<>[]!@#";

    const draw = () => {
        const progress = animatedProgress.value;

        ctx.fillStyle = "#FFFFFF";
        ctx.font = `bold ${fontSize}px monospace`;

        if (progress > 85) {
            ctx.shadowColor = "white";
            ctx.shadowBlur = 8;
        }

        for (let i = 0; i < drops.length; i++) {
            const text = chars[Math.floor(Math.random() * chars.length)];
            const x = i * fontSize;
            const y = drops[i] * fontSize;

            ctx.fillText(text, x, y);

            if (y > height && Math.random() > 0.95) {
                drops[i] = 0;
            }
            drops[i] += 1.8;
        }
    };

    if (_matrixInterval) clearInterval(_matrixInterval);
    _matrixInterval = window.setInterval(draw, 16);
};

onMounted(() => {
    if (!props.halloweenActive) nextTick(() => initMatrix());
});

watch(
    () => props.halloweenActive,
    (val) => {
        if (!val) nextTick(() => initMatrix());
    }
);

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

onBeforeUnmount(() => {
    if (_progressRaf) cancelAnimationFrame(_progressRaf);
    if (_bumpTimer) window.clearTimeout(_bumpTimer);
    if (_matrixInterval) clearInterval(_matrixInterval);
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
    transition: opacity 0.3s ease; /* Быстрее затухание */
    color: white;
}

#preloader.animate-out {
    opacity: 0;
    pointer-events: none;
}

.logo-wrapper {
    width: 217.5px;
    height: 182.5px;
    position: relative;
    filter: drop-shadow(0 0 10px rgba(255, 255, 255, 0.2));
}

.matrix-canvas {
    width: 100%;
    height: 100%;
    display: block;
}

.invert {
    filter: invert(1);
}

.bump {
    animation: bump 0.15s cubic-bezier(0.2, 0.8, 0.2, 1);
}

@keyframes bump {
    0% {
        transform: scale(1);
    }
    50% {
        transform: scale(1.08);
        color: #fff;
    }
    100% {
        transform: scale(1);
    }
}

.progress-fill {
    will-change: transform;
    transition: none;
}
</style>

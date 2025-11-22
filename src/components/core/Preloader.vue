<template>
  <div id="preloader" v-if="visible" :class="{ 'animate-out': animateOut }" role="status" aria-live="polite"
    :aria-label="loadingState" :aria-busy="visible" class="fixed inset-0 bg-base-300 flex items-center justify-center">
    <BootLogs v-if="isDev" :current-progress="currentProgress / totalSteps" :loading-state="loadingState" />

    <div class="flex flex-col items-center justify-center h-full w-screen relative z-10">
      <div v-if="!halloweenActive" class="w-48 h-48 lottie-animate">
        <Vue3Lottie :animation-data="preloader" :height="200" :width="200" />
      </div>
      <div v-else class="w-48 h-48">
        <img src="../../assets/misc/ghosts.gif" />
      </div>

      <span class="sr-only">{{ loadingState }}</span>

      <div class="loading-status mt-6">
        <transition name="slide-fade" mode="out-in">
          <span :key="loadingState" class="text-lg font-medium" :class="{ invert: currentTheme === 'light' }">{{
            loadingState }}</span>
        </transition>
      </div>

      <div class="w-80 progress-container mt-4" :class="{ invert: currentTheme === 'light' }" role="progressbar"
        :aria-valuenow="animatedProgressRounded" aria-valuemin="0" aria-valuemax="100"
        :aria-valuetext="`${animatedProgressRounded}%`">
        <div class="bg-base-100 rounded-full h-3 overflow-hidden shadow-inner progress-track" aria-hidden="true">
          <div class="bg-primary h-full rounded-full progress-fill"
            :style="{ transform: `scaleX(${(animatedProgress / 100)})` }"></div>
        </div>
        <div class="text-center mt-3 text-sm opacity-75">
          <span :class="['percentage-number', { bump: percentageBump }]">{{ animatedProgressRounded }}%</span>
        </div>
      </div>

      <button v-if="isDev" @click="skipIntro" type="button" class="btn btn-sm btn-ghost mt-6">
        Skip intro
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onBeforeUnmount } from 'vue';
import { Vue3Lottie } from 'vue3-lottie';
import preloader from '../../assets/misc/preloader.json';
import BootLogs from './BootLogs.vue';

const props = defineProps({
  show: { type: Boolean, required: true },
  isDev: { type: Boolean, default: false },
  loadingState: { type: String, required: true },
  currentProgress: { type: Number, required: true },
  totalSteps: { type: Number, required: true },
  halloweenActive: { type: Boolean, default: false },
  currentTheme: { type: String, default: 'dark' },
});

const emit = defineEmits(['update:show']);

const progressPercent = computed(() =>
  Math.min(100, Math.max(0, (props.currentProgress / (props.totalSteps || 1)) * 100))
);
const animatedProgress = ref(progressPercent.value);
const animatedProgressRounded = computed(() => Math.round(animatedProgress.value));
let _progressRaf: number | null = null;
const animateTo = (target: number) => {
  if (_progressRaf) cancelAnimationFrame(_progressRaf);
  const step = () => {
    const current = animatedProgress.value;
    const delta = target - current;
    const ease = 0.08;
    const snapThreshold = 0.15;
    animatedProgress.value = Math.abs(delta) < snapThreshold ? target : current + delta * ease;
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
  _bumpTimer = window.setTimeout(() => (percentageBump.value = false), 360);
});

const visible = ref(props.show);
const animateOut = ref(false);
const ANIMATE_OUT_MS = 600;
watch(() => props.show, (val) => {
  if (val) {
    animateOut.value = false;
    visible.value = true;
  } else {
    animateOut.value = true;
    window.setTimeout(() => {
      animateOut.value = false;
      visible.value = false;
    }, ANIMATE_OUT_MS);
  }
});

const skipIntro = () => emit('update:show', false);

onBeforeUnmount(() => {
  if (_progressRaf) cancelAnimationFrame(_progressRaf);
  if (_bumpTimer) window.clearTimeout(_bumpTimer);
});
</script>

<style scoped>
#preloader {
  position: fixed;
  width: 100%;
  height: 100%;
  left: 0;
  top: 0;
  background-color: rgba(0, 0, 0, 1);
  z-index: 1337;
  transition:
    opacity 0.4s ease,
    transform 0.6s ease,
    background-color 0.6s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  pointer-events: auto;
}

#preloader.animate-out {
  background-color: rgba(0, 0, 0, 0);
  opacity: 0;
  transform: translateY(-10px);
  pointer-events: none;
  transition: opacity 0.5s ease, transform 0.6s ease, background-color 0.6s ease;
}
</style>
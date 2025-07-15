<!-- BUTTON FROM DAISYUI THEME GENERATOR -->
<!-- https://github.com/saadeghi/daisyui/blob/b4196cce349e1b09e1eeca01a180ed5bcabe2d01/packages/docs/src/routes/(routes)/theme-generator/%2Bpage.svelte#L545 -->
<template>
    <button ref="btnElement" class="btn btn-neutral btn-lg group" @mousedown="handleMouseDown"
        @mouseup="handleMouseUpOrLeave" @mouseleave="handleMouseUpOrLeave" @blur="handleMouseUpOrLeave"
        @touchstart.prevent="handleMouseDown" @touchend="handleMouseUpOrLeave" @keydown="handleKeyDown"
        @keyup="handleKeyUp">
        <Gamepad2 ref="svgElement" class="w-7 h-7  transition-transform duration-300 ease-in-out" />
        <span class="font-normal text-lg">{{ $t('hold_button.label') }}</span>
    </button>
</template>

<script setup>
import { ref, onUnmounted } from 'vue';
import { Gamepad2 } from 'lucide-vue-next';

const emit = defineEmits(['start']);

const btnElement = ref(null);
const svgElement = ref(null);

let holdTimeout = null;
let holdInterval = null;
let startTime = null;

const handleMouseDown = () => {
    if (holdTimeout || holdInterval) return;

    startTime = Date.now();

    if (btnElement.value && svgElement.value) {
        svgElement.value.style.transition = 'rotate 3s linear';
        btnElement.value.style.transition = 'box-shadow 3s ease-out';

        btnElement.value.style.boxShadow = '0 0 0 -0.1rem color-mix(in oklab, var(--color-base-content) 30%, transparent) inset';

        holdInterval = setInterval(() => {
            const elapsedTime = Date.now() - startTime;
            const rotation = 90 * (elapsedTime / 1000);
            const boxShadowOffset = -2 * (elapsedTime / 600) - 3.5;

            if (rotation >= 90) {
                svgElement.value.style.rotate = '90deg';
                clearInterval(holdInterval);
                holdInterval = null;
            } else {
                svgElement.value.style.rotate = `${rotation}deg`;
                btnElement.value.style.boxShadow = `0 ${boxShadowOffset}rem 0 -3rem color-mix(in oklab, var(--color-base-content) 30%, transparent) inset`;
            }
        }, 10);
    }

    holdTimeout = setTimeout(() => {
        emit('start');
        handleMouseUpOrLeave();
    }, 3000);
};

const handleMouseUpOrLeave = () => {
    clearTimeout(holdTimeout);
    clearInterval(holdInterval);
    holdTimeout = null;
    holdInterval = null;

    if (svgElement.value) {
        svgElement.value.style.removeProperty('transition');
        svgElement.value.style.removeProperty('rotate');
    }
    if (btnElement.value) {
        btnElement.value.style.removeProperty('transition');
        btnElement.value.style.removeProperty('box-shadow');
    }
};

const handleKeyDown = (event) => {
    if ((event.key === 'Enter' || event.key === ' ') && !holdTimeout && !holdInterval) {
        event.preventDefault();
        handleMouseDown();
    }
};

const handleKeyUp = (event) => {
    if (event.key === 'Enter' || event.key === ' ') {
        handleMouseUpOrLeave();
    }
};

onUnmounted(() => {
    handleMouseUpOrLeave();
});
</script>

<style scoped>
.start-btn {
    background-image:
        radial-gradient(ellipse at 50% 270%, #00ff4747, transparent 60%),
        radial-gradient(ellipse at 20% 150%, #00ff8847, transparent 60%),
        radial-gradient(ellipse at 70% 200%, #22ff0047, transparent 60%);
}
</style>
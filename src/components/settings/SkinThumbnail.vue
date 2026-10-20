<script setup lang="ts">
import { onMounted, ref, watch } from "vue";
import { UserRound } from "@lucide/vue";

const props = defineProps<{
    textureUrl: string;
    model: string;
}>();

const canvas = ref<HTMLCanvasElement | null>(null);
const failed = ref(false);
let requestId = 0;

const paintFace = (
    context: CanvasRenderingContext2D,
    image: HTMLImageElement,
    sourceX: number,
    sourceY: number,
    sourceWidth: number,
    sourceHeight: number,
    destinationX: number,
    destinationY: number,
    unit: number
) => {
    context.drawImage(
        image,
        sourceX,
        sourceY,
        sourceWidth,
        sourceHeight,
        destinationX * unit,
        destinationY * unit,
        sourceWidth * unit,
        sourceHeight * unit
    );
};

const render = (image: HTMLImageElement) => {
    const element = canvas.value;
    if (!element) {
        return;
    }
    const context = element.getContext("2d");
    if (!context) {
        return;
    }

    const unit = element.width / 18;
    const slim = props.model === "slim";
    const armWidth = slim ? 3 : 4;
    const rightArmX = slim ? 2 : 1;
    const leftArmX = 13;

    context.clearRect(0, 0, element.width, element.height);
    context.imageSmoothingEnabled = false;

    context.save();
    context.fillStyle = "rgba(0, 0, 0, 0.18)";
    context.beginPath();
    context.ellipse(
        9 * unit,
        33.6 * unit,
        6.2 * unit,
        0.9 * unit,
        0,
        0,
        Math.PI * 2
    );
    context.fill();
    context.restore();

    paintFace(context, image, 4, 20, 4, 12, 5, 21, unit);
    paintFace(context, image, 4, 36, 4, 12, 5, 21, unit);

    paintFace(context, image, 20, 52, 4, 12, 9, 21, unit);
    paintFace(context, image, 4, 52, 4, 12, 9, 21, unit);

    paintFace(context, image, 20, 20, 8, 12, 5, 9, unit);
    paintFace(context, image, 20, 36, 8, 12, 5, 9, unit);

    paintFace(context, image, 44, 20, armWidth, 12, rightArmX, 9, unit);
    paintFace(context, image, 44, 36, armWidth, 12, rightArmX, 9, unit);

    paintFace(context, image, 36, 52, armWidth, 12, leftArmX, 9, unit);
    paintFace(context, image, 52, 52, armWidth, 12, leftArmX, 9, unit);

    paintFace(context, image, 8, 8, 8, 8, 5, 1, unit);
    paintFace(context, image, 40, 8, 8, 8, 5, 1, unit);
};

const draw = () => {
    const currentRequest = ++requestId;
    failed.value = false;
    if (!props.textureUrl) {
        failed.value = true;
        return;
    }
    const image = new Image();
    image.onload = () => {
        if (currentRequest === requestId) {
            render(image);
        }
    };
    image.onerror = () => {
        if (currentRequest === requestId) {
            failed.value = true;
        }
    };
    image.src = props.textureUrl;
};

onMounted(draw);
watch(() => [props.textureUrl, props.model], draw);
</script>

<template>
    <div class="relative h-full w-full flex items-center justify-center">
        <canvas
            ref="canvas"
            width="180"
            height="340"
            class="skin-thumbnail"
            :class="{ 'opacity-0': failed }"
        ></canvas>
        <UserRound v-if="failed" class="w-10 h-10 text-base-content/30" />
    </div>
</template>

<style scoped>
.skin-thumbnail {
    height: 100%;
    width: auto;
    max-width: 100%;
    image-rendering: pixelated;
}
</style>

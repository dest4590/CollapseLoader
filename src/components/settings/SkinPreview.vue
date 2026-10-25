<script setup lang="ts">
import { onBeforeUnmount, onMounted, ref, watch } from "vue";
import { ImageOff } from "@lucide/vue";
import { IdleAnimation, SkinViewer } from "skinview3d";

const props = withDefaults(
    defineProps<{
        textureUrl: string | null;
        model?: "classic" | "slim";
        capeUrl?: string | null;
    }>(),
    {
        model: "classic",
        capeUrl: null,
    }
);

const container = ref<HTMLDivElement | null>(null);
const canvas = ref<HTMLCanvasElement | null>(null);
const loading = ref(true);
const failed = ref(false);

let viewer: SkinViewer | null = null;
let resizeObserver: ResizeObserver | null = null;
let loadVersion = 0;

const setInitialView = () => {
    if (!viewer) {
        return;
    }
    viewer.resetCameraPose();
    const distance = viewer.camera.position.length();
    const angle = -0.32;
    viewer.camera.position.set(
        Math.sin(angle) * distance,
        0,
        Math.cos(angle) * distance
    );
    viewer.camera.lookAt(0, 0, 0);
    viewer.controls.update();
};

const loadAppearance = async () => {
    if (!viewer) {
        return;
    }
    const version = ++loadVersion;
    if (!props.textureUrl) {
        viewer.resetSkin();
        viewer.resetCape();
        loading.value = false;
        return;
    }
    loading.value = true;
    failed.value = false;
    try {
        await viewer.loadSkin(props.textureUrl, {
            model: props.model === "slim" ? "slim" : "default",
        });
        if (version !== loadVersion) {
            return;
        }
        if (props.capeUrl) {
            await viewer.loadCape(props.capeUrl);
        } else {
            viewer.resetCape();
        }
        if (version !== loadVersion) {
            return;
        }
    } catch {
        if (version === loadVersion) {
            failed.value = true;
        }
    } finally {
        if (version === loadVersion) {
            loading.value = false;
        }
    }
};

const resetView = () => {
    setInitialView();
};

onMounted(() => {
    const element = container.value;
    if (!element || !canvas.value) {
        return;
    }

    viewer = new SkinViewer({
        canvas: canvas.value,
        width: Math.max(element.clientWidth, 1),
        height: Math.max(element.clientHeight, 1),
        zoom: 0.86,
        fov: 48,
        animation: new IdleAnimation(),
    });
    if (viewer.animation) {
        viewer.animation.speed = 0.65;
    }
    viewer.controls.enablePan = false;
    viewer.controls.enableDamping = true;
    viewer.controls.dampingFactor = 0.08;
    viewer.controls.minPolarAngle = Math.PI * 0.25;
    viewer.controls.maxPolarAngle = Math.PI * 0.75;
    viewer.globalLight.intensity = 2.1;
    viewer.cameraLight.intensity = 0.7;
    setInitialView();
    void loadAppearance();

    resizeObserver = new ResizeObserver(() => {
        if (!viewer || !element) {
            return;
        }
        const width = Math.max(element.clientWidth, 1);
        const height = Math.max(element.clientHeight, 1);
        viewer.setSize(width, height);
    });
    resizeObserver.observe(element);
});

onBeforeUnmount(() => {
    resizeObserver?.disconnect();
    resizeObserver = null;
    loadVersion += 1;
    viewer?.dispose();
    viewer = null;
});

watch(
    () => [props.textureUrl, props.model, props.capeUrl] as const,
    () => {
        void loadAppearance();
    }
);
</script>

<template>
    <div ref="container" class="skin-viewer" @dblclick="resetView">
        <canvas ref="canvas" class="skin-canvas"></canvas>
        <div v-if="loading" class="skin-loading">
            <span
                class="loading loading-spinner loading-md text-primary"
            ></span>
        </div>
        <div v-else-if="failed" class="skin-loading">
            <ImageOff class="w-8 h-8 text-base-content/30" />
        </div>
    </div>
</template>

<style scoped>
.skin-viewer {
    position: relative;
    width: 100%;
    height: 230px;
    overflow: hidden;
    isolation: isolate;
}

.skin-viewer::before {
    content: "";
    position: absolute;
    inset: 8% 4% 4%;
    background: radial-gradient(
        ellipse at 50% 38%,
        color-mix(in srgb, var(--color-primary) 18%, transparent),
        transparent 68%
    );
    filter: blur(8px);
    pointer-events: none;
}

.skin-viewer::after {
    content: "";
    position: absolute;
    left: 50%;
    bottom: 2.5%;
    width: 44%;
    height: 12px;
    border-radius: 50%;
    background: rgb(0 0 0 / 0.28);
    filter: blur(9px);
    transform: translateX(-50%);
    pointer-events: none;
}

.skin-canvas {
    position: relative;
    z-index: 1;
    display: block;
    width: 100%;
    height: 100%;
    cursor: grab;
    touch-action: none;
}

.skin-canvas:active {
    cursor: grabbing;
}

.skin-loading {
    position: absolute;
    inset: 0;
    z-index: 2;
    display: flex;
    align-items: center;
    justify-content: center;
    pointer-events: none;
}
</style>

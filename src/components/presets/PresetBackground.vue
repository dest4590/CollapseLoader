<template>
    <div ref="element" :style="backgroundStyle"></div>
</template>

<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from "vue";
import { resolveImageSource } from "@services/marketplaceImageService";

const props = defineProps<{ src: string }>();

const element = ref<HTMLElement | null>(null);
const resolved = ref<string | null>(null);

const backgroundStyle = computed(() =>
    resolved.value ? { backgroundImage: `url("${resolved.value}")` } : undefined
);

let observer: IntersectionObserver | null = null;
let visible = false;
let currentSrc = "";

async function load() {
    const src = props.src;
    if (!src || (src === currentSrc && resolved.value)) return;

    currentSrc = src;

    try {
        const resolvedSrc = await resolveImageSource(src);
        if (currentSrc === src) {
            resolved.value = resolvedSrc;
        }
    } catch (e) {
        if (currentSrc === src) {
            currentSrc = "";
        }
        console.warn("[PresetBackground] Failed to load image:", e);
    }
}

function observe() {
    observer?.disconnect();
    observer = null;

    if (!element.value) return;

    if (typeof IntersectionObserver === "undefined") {
        visible = true;
        void load();
        return;
    }

    observer = new IntersectionObserver(
        (entries) => {
            if (!entries.some((entry) => entry.isIntersecting)) return;
            visible = true;
            observer?.disconnect();
            observer = null;
            void load();
        },
        { rootMargin: "200px" }
    );
    observer.observe(element.value);
}

onMounted(observe);

onBeforeUnmount(() => {
    observer?.disconnect();
    observer = null;
});

watch(
    () => props.src,
    () => {
        resolved.value = null;
        currentSrc = "";
        if (visible) {
            void load();
        } else {
            observe();
        }
    }
);
</script>

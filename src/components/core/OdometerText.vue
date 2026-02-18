<template>
    <div class="odometer-container" aria-hidden="true">
        <span class="sr-only">{{ text }}</span>
        <TransitionGroup name="odometer-list" tag="div" class="odometer-row">
            <div v-for="(slot, i) in slots" :key="i" class="char-slot">
                <span
                    v-if="slot.prevChar !== null"
                    :key="`ex-${i}-${slot.gen}`"
                    class="char char-exit"
                    :style="{
                        animationDuration: `${duration}ms`,
                        animationDelay: `${i * stagger}ms`,
                    }"
                >
                    {{ slot.prevChar }}
                </span>

                <span
                    :key="`en-${i}-${slot.gen}`"
                    class="char"
                    :class="{ 'char-enter': slot.entering }"
                    :style="
                        slot.entering
                            ? {
                                  animationDuration: `${duration}ms`,
                                  animationDelay: `${i * stagger}ms`,
                              }
                            : {}
                    "
                >
                    {{ slot.current }}
                </span>
            </div>
        </TransitionGroup>
    </div>
</template>

<script setup lang="ts">
import { ref, watch } from "vue";

const props = defineProps({
    text: {
        type: String,
        default: "",
    },
    duration: {
        type: Number,
        default: 450,
    },
    stagger: {
        type: Number,
        default: 20,
    },
});

interface Slot {
    current: string;
    prevChar: string | null;
    entering: boolean;
    gen: number;
}

const slots = ref<Slot[]>([]);
let maxLen = 0;
let currentText = "";
const queue = ref<string[]>([]);
const busy = ref(false);

function initSlots(text: string): void {
    maxLen = text.length;
    currentText = text;
    slots.value = Array.from({ length: maxLen }, (_, i) => ({
        current: text[i] ?? "\u00A0",
        prevChar: null,
        entering: false,
        gen: 0,
    }));
}

function applyText(newText: string): void {
    const len = newText.length;

    if (slots.value.length > len) {
        slots.value.splice(len);
    }

    while (slots.value.length < len) {
        slots.value.push({
            current: "\u00A0",
            prevChar: null,
            entering: false,
            gen: 0,
        });
    }

    maxLen = len;
    currentText = newText;

    for (let i = 0; i < len; i++) {
        const ch = newText[i] ?? "\u00A0";
        const slot = slots.value[i];

        slot.prevChar = slot.current;
        slot.current = ch;
        slot.entering = true;
        slot.gen++;

        const thisGen = slot.gen;
        const cleanupMs = props.duration + i * props.stagger + 60;

        setTimeout(() => {
            const s = slots.value[i];
            if (s && s.gen === thisGen) {
                s.prevChar = null;
                s.entering = false;
            }
        }, cleanupMs);
    }
}

function processQueue(): void {
    if (busy.value || queue.value.length === 0) return;

    const next = queue.value.shift()!;
    busy.value = true;
    applyText(next);

    const lockMs = props.duration + maxLen * props.stagger + 100;
    setTimeout(() => {
        busy.value = false;
        processQueue();
    }, lockMs);
}

initSlots(props.text);

watch(
    () => props.text,
    (newVal) => {
        const last = queue.value[queue.value.length - 1] ?? currentText;
        if (newVal !== last) {
            queue.value.push(newVal);
            processQueue();
        }
    }
);
</script>

<style scoped>
.odometer-container {
    display: inline-flex;
    font-family: ui-monospace, SFMono-Regular, "SF Mono", Menlo, Consolas,
        "Liberation Mono", monospace;
    font-variant-numeric: tabular-nums;
    line-height: 1.5;
    white-space: pre;

    justify-content: center;
    min-width: 16ch;
}

.odometer-row {
    display: flex;
    align-items: center;
    justify-content: center;
}

.odometer-list-enter-active,
.odometer-list-leave-active,
.odometer-list-move {
    transition: all 0.5s cubic-bezier(0.22, 0.9, 0.3, 1);
}

.odometer-list-enter-from,
.odometer-list-leave-to {
    opacity: 0;
    width: 0 !important;
    transform: scaleX(0);
}

.odometer-list-leave-active {
    position: absolute;
}

.char-slot {
    position: relative;
    width: 1ch;
    height: 1.5em;
    overflow: hidden;
}

.char {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    will-change: transform, opacity;
}

@keyframes charEnter {
    0% {
        transform: translateY(-100%);
        opacity: 0;
    }
    60% {
        transform: translateY(4%);
        opacity: 1;
    }
    85% {
        transform: translateY(-1.5%);
    }
    100% {
        transform: translateY(0);
        opacity: 1;
    }
}

@keyframes charExit {
    0% {
        transform: translateY(0);
        opacity: 1;
    }
    30% {
        transform: translateY(-3%);
    }
    100% {
        transform: translateY(100%);
        opacity: 0;
    }
}

.char-enter {
    animation-name: charEnter;
    animation-timing-function: cubic-bezier(0.2, 0.9, 0.2, 1);
    animation-fill-mode: both;
}

.char-exit {
    animation-name: charExit;
    animation-timing-function: cubic-bezier(0.2, 0.9, 0.2, 1);
    animation-fill-mode: both;
}
</style>

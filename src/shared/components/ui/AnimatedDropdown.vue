<script setup lang="ts">
import { ref, computed, onMounted, onBeforeUnmount, nextTick } from "vue";
import { ChevronDown, Check } from "lucide-vue-next";

interface Option {
    value: string;
    label: string;
}

const props = defineProps<{
    modelValue: string;
    options: Option[];
    width?: string;
    direction?: "up" | "down";
}>();

const emit = defineEmits<{
    (e: "update:modelValue", value: string): void;
    (e: "change", value: string): void;
}>();

const isOpen = ref(false);
const triggerRef = ref<HTMLElement | null>(null);
const menuStyle = ref<Record<string, string>>({});
let intersectionObserver: IntersectionObserver | null = null;

const selectedLabel = computed(
    () => props.options.find((o) => o.value === props.modelValue)?.label ?? ""
);

const updateMenuPosition = () => {
    if (!triggerRef.value) return;
    const rect = triggerRef.value.getBoundingClientRect();

    const style: Record<string, string> = {
        position: "fixed",
        width: `${rect.width}px`,
        zIndex: "99999",
        right: `${window.innerWidth - rect.right}px`,
    };

    if (props.direction === "up") {
        style.bottom = `${window.innerHeight - rect.top + 6}px`;
    } else {
        style.top = `${rect.bottom + 6}px`;
    }

    menuStyle.value = style;
};

const toggle = async () => {
    if (!isOpen.value) {
        updateMenuPosition();
        await nextTick();
    }
    isOpen.value = !isOpen.value;
};

const select = (option: Option) => {
    emit("update:modelValue", option.value);
    emit("change", option.value);
    isOpen.value = false;
};

const onClickOutside = (e: MouseEvent) => {
    if (triggerRef.value && !triggerRef.value.contains(e.target as Node)) {
        isOpen.value = false;
    }
};

const onScroll = () => {
    if (isOpen.value) updateMenuPosition();
};

onMounted(() => {
    document.addEventListener("mousedown", onClickOutside);
    window.addEventListener("scroll", onScroll, true);

    if (triggerRef.value) {
        intersectionObserver = new IntersectionObserver(
            ([entry]) => {
                if (!entry.isIntersecting && isOpen.value) {
                    isOpen.value = false;
                }
            },
            { threshold: 0.5 }
        );
        intersectionObserver.observe(triggerRef.value);
    }
});

onBeforeUnmount(() => {
    document.removeEventListener("mousedown", onClickOutside);
    window.removeEventListener("scroll", onScroll, true);
    intersectionObserver?.disconnect();
});
</script>

<template>
    <div class="animated-dropdown" :style="{ width: width ?? '12rem' }">
        <button
            ref="triggerRef"
            type="button"
            class="animated-dropdown-trigger"
            :class="{ open: isOpen }"
            @click="toggle"
        >
            <span class="trigger-label">{{ selectedLabel }}</span>
            <ChevronDown class="trigger-chevron" :class="{ rotated: isOpen }" />
        </button>

        <Teleport to="body">
            <Transition name="dropdown">
                <div
                    v-if="isOpen"
                    class="animated-dropdown-menu-inner"
                    :style="menuStyle"
                    :data-direction="direction ?? 'down'"
                >
                    <button
                        v-for="option in options"
                        :key="option.value"
                        type="button"
                        class="animated-dropdown-item"
                        :class="{ active: option.value === modelValue }"
                        @click="select(option)"
                    >
                        <Check
                            class="item-check"
                            :class="{ visible: option.value === modelValue }"
                        />
                        {{ option.label }}
                    </button>
                </div>
            </Transition>
        </Teleport>
    </div>
</template>

<style scoped>
.animated-dropdown {
    position: relative;
    display: inline-block;
}

.trigger-chevron {
    width: 1rem;
    height: 1rem;
    opacity: 0.5;
    flex-shrink: 0;
    transition:
        transform 0.25s cubic-bezier(0.34, 1.56, 0.64, 1),
        opacity 0.15s ease;
}

.trigger-chevron.rotated {
    transform: rotate(180deg);
    opacity: 0.8;
}

.dropdown-enter-active {
    transition:
        opacity 0.2s ease,
        transform 0.25s cubic-bezier(0.34, 1.4, 0.64, 1);
}
.dropdown-leave-active {
    transition:
        opacity 0.15s ease,
        transform 0.15s ease;
}
.dropdown-enter-from {
    opacity: 0;
    transform: translateY(-6px) scaleY(0.92);
    transform-origin: top center;
}
.dropdown-leave-to {
    opacity: 0;
    transform: translateY(-4px) scaleY(0.95);
    transform-origin: top center;
}
</style>

<style>
.animated-dropdown-trigger {
    width: 100%;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 0.5rem;
    padding: 0.25rem 0.625rem;
    border-radius: 0.5rem;
    border: 1.5px solid rgba(128, 128, 128, 0.25);
    background: rgba(128, 128, 128, 0.1);
    color: inherit;
    font-size: 0.8125rem;
    cursor: pointer;
    transition:
        border-color 0.15s ease,
        box-shadow 0.15s ease,
        background 0.15s ease;
    outline: none;
}

[data-theme="light"] .animated-dropdown-trigger {
    border-color: rgba(0, 0, 0, 0.15);
    background: rgba(0, 0, 0, 0.05);
}

.animated-dropdown-trigger:hover {
    border-color: rgba(128, 128, 128, 0.45);
    background: rgba(128, 128, 128, 0.15);
}

.animated-dropdown-trigger.open {
    border-color: oklch(var(--p) / 0.6);
    box-shadow: 0 0 0 2px oklch(var(--p) / 0.15);
    background: rgba(128, 128, 128, 0.12);
}

.animated-dropdown-trigger .trigger-label {
    flex: 1;
    text-align: left;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
}

.animated-dropdown-menu-inner {
    background-color: var(--color-base-200, #1d232a);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 0.625rem;
    padding: 0.25rem;
    box-shadow:
        0 16px 40px rgba(0, 0, 0, 0.35),
        0 2px 8px rgba(0, 0, 0, 0.2);
    overflow: hidden;
}

[data-theme="light"] .animated-dropdown-menu-inner {
    background-color: #ffffff;
    border-color: rgba(0, 0, 0, 0.1);
    box-shadow:
        0 8px 24px rgba(0, 0, 0, 0.12),
        0 2px 6px rgba(0, 0, 0, 0.06);
}

.animated-dropdown-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 0.375rem;
    padding: 0.3rem 0.5rem;
    border-radius: 0.375rem;
    border: none;
    background: transparent;
    color: rgba(255, 255, 255, 0.6);
    font-size: 0.75rem;
    text-align: left;
    cursor: pointer;
    transition:
        background 0.1s ease,
        color 0.1s ease;
}

[data-theme="light"] .animated-dropdown-item {
    color: rgba(0, 0, 0, 0.6);
}

.animated-dropdown-item:hover {
    background: rgba(255, 255, 255, 0.07);
    color: rgba(255, 255, 255, 0.95);
}

[data-theme="light"] .animated-dropdown-item:hover {
    background: rgba(0, 0, 0, 0.05);
    color: rgba(0, 0, 0, 0.9);
}

.animated-dropdown-item.active {
    color: oklch(var(--p));
    font-weight: 500;
}

.animated-dropdown-item .item-check {
    width: 0.875rem;
    height: 0.875rem;
    flex-shrink: 0;
    opacity: 0;
    color: oklch(var(--p));
    transition: opacity 0.15s ease;
}

.animated-dropdown-item .item-check.visible {
    opacity: 1;
}
</style>

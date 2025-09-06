<template>
    <div class="search-container">
        <div class="relative">
            <div class="relative">
                <div class="absolute left-10 top-1/2 -translate-y-1/2 pointer-events-none z-15 flex text-base-content">
                    <span v-for="(item, index) in animatedChars" :key="item.id"
                        :class="['letter-wrapper', { 'letter-slide-in': !item.static }]"
                        :style="!item.static ? { animationDelay: (index * 20) + 'ms' } : undefined">
                        {{ item.char === ' ' ? '\u00A0' : item.char }}
                    </span>
                    <span v-if="animatedChars.length === 0" class="text-gray-400">{{ t('home.search_placeholder')
                    }}</span>
                </div>

                <input type="text"
                    class="input input-bordered w-full pl-10 pr-10 transition-colors duration-300 z-20 relative text-transparent caret-transparent selection:bg-primary selection:text-primary-content"
                    style="background-color: transparent;" v-model="searchTerm" @input="onSearch" />
            </div>

            <div class="absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none z-30">
                <Search class="w-5 h-5 text-gray-400" :class="{ 'search-icon-active': searchTerm }" />
            </div>
            <div v-if="searchTerm" class="absolute right-3 top-1/2 -translate-y-1/2 z-30 cursor-pointer"
                @click="clearSearch">
                <X class="w-4 h-4 text-gray-400 hover:text-gray-600" />
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue';
import { Search, X } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

const props = defineProps<{ initialValue?: string }>();
const emit = defineEmits(['search']);
const { t } = useI18n();

const searchTerm = ref(props.initialValue || '');

interface AnimatedChar { id: number; char: string; static?: boolean }
const animatedChars = ref<AnimatedChar[]>([]);
let charId = 0;

if (searchTerm.value) {
    animatedChars.value = searchTerm.value.split('').map(c => ({ id: ++charId, char: c, static: true }));
}

const onSearch = () => emit('search', searchTerm.value);
const clearSearch = () => { searchTerm.value = ''; animatedChars.value = []; onSearch(); };

watch(() => props.initialValue, (newValue) => { if (newValue !== undefined) searchTerm.value = newValue; });

watch(searchTerm, (newVal, oldVal) => {
    if (newVal.length > oldVal.length && newVal.startsWith(oldVal)) {
        const added = newVal.slice(oldVal.length);
        for (const c of added) {
            animatedChars.value.push({ id: ++charId, char: c });
        }
    } else if (oldVal.length > newVal.length && oldVal.startsWith(newVal)) {
        animatedChars.value.splice(newVal.length);
    } else if (newVal !== oldVal) {
        animatedChars.value = newVal.split('').map(c => ({ id: ++charId, char: c, static: true }));
    }
});
</script>

<style scoped>
.search-container {
    animation: fadeIn 0.3s ease-in;
}

@keyframes fadeIn {
    from {
        opacity: 0;
        transform: translateY(-5px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.letter-wrapper {
    display: inline-block;
    font-family: inherit;
    font-size: inherit;
    letter-spacing: inherit;
}

.letter-slide-in {
    opacity: 0;
    transform: translateY(5px);
    animation: letterSlideIn 0.2s ease-out forwards;
}

@keyframes letterSlideIn {
    from {
        opacity: 0;
        transform: translateY(5px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.search-icon-active {
    color: var(--color-primary);
}

input {
    background-color: transparent;
}

input:focus {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 1px rgba(var(--color-primary-rgb), 0.2);
}

input.text-transparent {
    color: transparent !important;
    caret-color: transparent !important;
}

.caret-transparent {
    caret-color: transparent !important;
}

input.text-transparent::selection {
    background-color: rgba(var(--color-primary-rgb, 59, 130, 246), 0.3);
}
</style>
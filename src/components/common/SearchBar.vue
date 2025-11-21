<template>
    <div class="search-container group">
        <div class="relative">
            <input ref="inputRef" type="text"
                class="input input-bordered w-full pl-10 pr-12 transition-all duration-300 z-10 relative bg-base-200/50 focus:bg-base-100 focus:shadow-lg"
                :placeholder="t('home.search_placeholder')" v-model="searchTerm" @input="onSearch" />
            <div class="absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none z-20 transition-colors duration-300"
                :class="{ 'text-primary': isFocused || searchTerm }">
                <Search class="w-5 h-5" :class="[isFocused || searchTerm ? 'text-primary' : 'text-base-content/50']" />
            </div>

            <div class="absolute right-3 top-1/2 -translate-y-1/2 z-20 flex items-center gap-2">
                <Transition name="scale">
                    <button v-if="searchTerm" @click="clearSearch"
                        class="btn btn-circle btn-ghost btn-xs hover:bg-base-content/10">
                        <X class="w-4 h-4 text-base-content/70" />
                    </button>
                </Transition>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, onBeforeUnmount } from 'vue';
import { Search, X } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

const props = defineProps<{
    initialValue?: string;
}>();

const emit = defineEmits(['search']);

const { t } = useI18n();
const searchTerm = ref(props.initialValue || '');
const inputRef = ref<HTMLInputElement | null>(null);
const isFocused = ref(false);

const onSearch = () => {
    emit('search', searchTerm.value);
};

const clearSearch = () => {
    searchTerm.value = '';
    onSearch();
    inputRef.value?.focus();
};

const focus = () => {
    inputRef.value?.focus();
};

defineExpose({ focus });

watch(
    () => props.initialValue,
    (newValue) => {
        if (newValue !== undefined) {
            searchTerm.value = newValue;
        }
    }
);

const handleFocus = () => { isFocused.value = true; };
const handleBlur = () => { isFocused.value = false; };

onMounted(() => {
    inputRef.value?.addEventListener('focus', handleFocus);
    inputRef.value?.addEventListener('blur', handleBlur);
});

onBeforeUnmount(() => {
    inputRef.value?.removeEventListener('focus', handleFocus);
    inputRef.value?.removeEventListener('blur', handleBlur);
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

.scale-enter-active,
.scale-leave-active {
    transition: all 0.2s ease;
}

.scale-enter-from,
.scale-leave-to {
    opacity: 0;
    transform: scale(0.8);
}
</style>
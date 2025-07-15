<template>
    <div class="search-container">
        <div class="relative">
            <input type="text"
                class="input input-bordered w-full pl-10 pr-10 transition-colors duration-300 z-10 relative"
                :placeholder="t('home.search_placeholder')" v-model="searchTerm" @input="onSearch" />
            <div class="absolute left-3 top-1/2 -translate-y-1/2 pointer-events-none z-20">
                <Search class="w-5 h-5 text-gray-400" :class="{ 'search-icon-active': searchTerm }" />
            </div>
            <div v-if="searchTerm" class="absolute right-3 top-1/2 -translate-y-1/2 z-20 cursor-pointer"
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

const props = defineProps<{
    initialValue?: string;
}>();

const emit = defineEmits(['search']);

const { t } = useI18n();
const searchTerm = ref(props.initialValue || '');

const onSearch = () => {
    emit('search', searchTerm.value);
};

const clearSearch = () => {
    searchTerm.value = '';
    onSearch();
};

watch(
    () => props.initialValue,
    (newValue) => {
        if (newValue !== undefined) {
            searchTerm.value = newValue;
        }
    }
);
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

.search-icon-active {
    color: var(--color-primary);
}

input {
    background-color: transparent;
    backdrop-filter: blur(10px);
}

input:focus {
    border-color: var(--color-primary);
    box-shadow: 0 0 0 1px rgba(var(--color-primary-rgb), 0.2);
}
</style>

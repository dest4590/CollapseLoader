<script setup lang="ts">
import { computed, ref, watch } from 'vue';
import { ChevronDown, Funnel } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

interface Filters {
    fabric: boolean;
    vanilla: boolean;
    installed: boolean;
}

type SortKey = 'popularity' | 'name' | 'newest' | 'version' | 'rating';
type SortOrder = 'asc' | 'desc';

const props = defineProps<{
    activeFilters: Filters;
    clientSortKey: SortKey;
    clientSortOrder: SortOrder;
}>();

const emit = defineEmits<{
    'update:activeFilters': [filters: Filters];
    'update:clientSortKey': [sortKey: SortKey];
    'update:clientSortOrder': [sortOrder: SortOrder];
}>();

const { t } = useI18n();

const showFiltersMenu = ref(false);

const localFilters = computed({
    get: () => props.activeFilters,
    set: (value) => emit('update:activeFilters', value),
});

const localSortKey = computed({
    get: () => props.clientSortKey,
    set: (value) => emit('update:clientSortKey', value),
});

const localSortOrder = computed({
    get: () => props.clientSortOrder,
    set: (value) => emit('update:clientSortOrder', value),
});

const toggleFiltersMenu = (event?: Event) => {
    if (event) event.stopPropagation();
    showFiltersMenu.value = !showFiltersMenu.value;
};

const closeFiltersMenu = () => {
    showFiltersMenu.value = false;
};

const onDocumentClick = (e: MouseEvent) => {
    const target = e.target as HTMLElement;
    if (!target.closest('.filters-container')) {
        closeFiltersMenu();
    }
};

watch(showFiltersMenu, (isOpen) => {
    if (isOpen) {
        document.addEventListener('click', onDocumentClick);
    } else {
        document.removeEventListener('click', onDocumentClick);
    }
});

import { onBeforeUnmount } from 'vue';
onBeforeUnmount(() => {
    document.removeEventListener('click', onDocumentClick);
});
</script>

<template>
    <div class="relative filters-container">
        <button @click.stop="toggleFiltersMenu" class="btn btn-ghost btn-sm ml-2">
            <Funnel class="w-4 h-4 mr-1" />
            {{ t('home.filters_title') }}
            <ChevronDown class="w-4 h-4 ml-1 transition-all duration-300" :class="{ 'rotate-180 ': showFiltersMenu }" />
        </button>

        <transition name="slide-up-filter">
            <div v-if="showFiltersMenu"
                class="absolute right-0 mt-2 w-56 bg-base-200 rounded-box shadow-xl border border-base-300 z-50 p-3">
                <div class="flex flex-col gap-2 text-sm">
                    <select class="select select-sm" v-model="localSortKey">
                        <option value="newest">{{ t('home.sort.newest') }}</option>
                        <option value="popularity">{{ t('home.sort.popularity') }}</option>
                        <option value="rating">{{ t('home.sort.rating') }}</option>
                        <option value="name">{{ t('home.sort.name') }}</option>
                        <option value="version">{{ t('home.sort.version') }}</option>
                    </select>

                    <select class="select select-sm" v-model="localSortOrder">
                        <option value="asc">{{ t('home.sort.order.asc') }}</option>
                        <option value="desc">{{ t('home.sort.order.desc') }}</option>
                    </select>

                    <label class="flex items-center gap-2">
                        <input type="checkbox" class="checkbox" v-model="localFilters.fabric" />
                        <span>Fabric</span>
                    </label>

                    <label class="flex items-center gap-2">
                        <input type="checkbox" class="checkbox" v-model="localFilters.vanilla" />
                        <span>Vanilla</span>
                    </label>

                    <label class="flex items-center gap-2">
                        <input type="checkbox" class="checkbox" v-model="localFilters.installed" />
                        <span>{{ t('home.filters.installed') }}</span>
                    </label>
                </div>
            </div>
        </transition>
    </div>
</template>

<style scoped>
.slide-up-filter-enter-active,
.slide-up-filter-leave-active {
    transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.slide-up-filter-enter-from,
.slide-up-filter-leave-to {
    opacity: 0;
    transform: translateY(10px);
}
</style>
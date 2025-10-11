<script setup lang="ts">
import { ref, watch } from 'vue';
import { ChevronDown } from 'lucide-vue-next';
import { useI18n } from 'vue-i18n';

interface Filters {
    fabric: boolean;
    vanilla: boolean;
    installed: boolean;
    new: boolean;
}

type SortKey = 'popularity' | 'name' | 'newest' | 'version';
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

const localFilters = ref<Filters>({ ...props.activeFilters });
const localSortKey = ref<SortKey>(props.clientSortKey);
const localSortOrder = ref<SortOrder>(props.clientSortOrder);

watch(() => props.activeFilters, (newFilters) => {
    localFilters.value = { ...newFilters };
}, { deep: true });

watch(() => props.clientSortKey, (newSortKey) => {
    localSortKey.value = newSortKey;
});

watch(() => props.clientSortOrder, (newSortOrder) => {
    localSortOrder.value = newSortOrder;
});

watch(localFilters, (newFilters) => {
    emit('update:activeFilters', { ...newFilters });
}, { deep: true });

watch(localSortKey, (newSortKey) => {
    emit('update:clientSortKey', newSortKey);
});

watch(localSortOrder, (newSortOrder) => {
    emit('update:clientSortOrder', newSortOrder);
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
            {{ t('home.filters_title') }}
            <ChevronDown class="w-4 h-4 ml-1" />
        </button>

        <transition name="slide-up-filter">
            <div v-if="showFiltersMenu"
                class="absolute right-0 mt-2 w-56 bg-base-200 rounded-box shadow-xl border border-base-300 z-50 p-3">
                <div class="flex flex-col gap-2 text-sm">
                    <select class="select select-sm" v-model="localSortKey">
                        <option value="newest">{{ t('home.sort.newest') }}</option>
                        <option value="popularity">{{ t('home.sort.popularity') }}</option>
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

                    <label class="flex items-center gap-2">
                        <input type="checkbox" class="checkbox" v-model="localFilters.new" />
                        <span>{{ t('home.filters.new') }}</span>
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
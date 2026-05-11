<script setup lang="ts">
import type { Ref } from "vue";
import { computed, onBeforeUnmount, onMounted, ref } from "vue";
import { useI18n } from "vue-i18n";
import { FileText, History, Newspaper } from "lucide-vue-next";
import SearchBar from "@shared/components/common/SearchBar.vue";
import FiltersMenu from "@shared/components/common/FiltersMenu.vue";
import LaunchHistoryPanel from "@features/clients/components/LaunchHistoryPanel.vue";

interface Filters {
    fabric: boolean;
    vanilla: boolean;
    forge: boolean;
    installed: boolean;
}

const { t } = useI18n();

const props = defineProps<{
    searchQuery: string;
    activeFilters: Filters;
    clientSortKey: "popularity" | "name" | "newest" | "version" | "rating";
    clientSortOrder: "asc" | "desc";
    unreadNewsCount?: number | null;
    viewVisible: boolean;
    showHistory: boolean;
    searchBarRef: Ref<any>;
}>();

const emit = defineEmits<{
    search: [string];
    "update:activeFilters": [Filters];
    "update:clientSortKey": ["popularity" | "name" | "newest" | "version" | "rating"];
    "update:clientSortOrder": ["asc" | "desc"];
    "update:showHistory": [boolean];
    "change-view": [string];
    "launch-client": [number];
}>();

const rootRef = ref<HTMLElement | null>(null);

const activeFiltersLocal = computed<Filters>({
    get: () => props.activeFilters,
    set: (value) => emit("update:activeFilters", value),
});

const clientSortKeyLocal = computed<
    "popularity" | "name" | "newest" | "version" | "rating"
>({
    get: () => props.clientSortKey,
    set: (value) => emit("update:clientSortKey", value),
});

const clientSortOrderLocal = computed<"asc" | "desc">({
    get: () => props.clientSortOrder,
    set: (value) => emit("update:clientSortOrder", value),
});

const handleSearch = (value: string) => {
    emit("search", value);
};

const toggleHistory = () => {
    emit("update:showHistory", !props.showHistory);
};

const handleLaunchFromHistory = (id: number) => {
    emit("launch-client", id);
    emit("update:showHistory", false);
};

const onDocumentClick = (event: MouseEvent) => {
    const target = event.target as HTMLElement;
    if (
        props.showHistory &&
        rootRef.value &&
        !rootRef.value.contains(target)
    ) {
        emit("update:showHistory", false);
    }
};

onMounted(() => {
    document.addEventListener("click", onDocumentClick);
});

onBeforeUnmount(() => {
    document.removeEventListener("click", onDocumentClick);
});
</script>

<template>
    <div
        ref="rootRef"
        :class="[
            'flex items-center gap-2 mb-6 top-menu',
            viewVisible ? 'home-entered' : 'home-hidden',
        ]"
    >
        <SearchBar
            :ref="searchBarRef"
            @search="handleSearch"
            class="flex-1 mr-2 home-search"
            :initial-value="searchQuery"
            :placeholder="t('home.search_placeholder')"
        />
        <div class="home-action-btn">
            <FiltersMenu
                v-model:activeFilters="activeFiltersLocal"
                v-model:clientSortKey="clientSortKeyLocal"
                v-model:clientSortOrder="clientSortOrderLocal"
            />
        </div>
        <div
            class="tooltip tooltip-bottom home-action-btn"
            :data-tip="t('navigation.custom_clients')"
        >
            <button
                @click="$emit('change-view', 'custom_clients')"
                class="btn btn-ghost border-base-300 btn-primary gap-2"
                :style="{
                    border: 'var(--border) solid #0000',
                }"
            >
                <FileText class="w-4 h-4" />
            </button>
        </div>
        <div
            class="tooltip tooltip-bottom home-action-btn"
            :data-tip="t('navigation.news')"
        >
            <button
                @click="$emit('change-view', 'news')"
                class="btn btn-ghost border-base-300 btn-primary gap-2 relative"
                :style="{
                    border: 'var(--border) solid #0000',
                }"
            >
                <Newspaper class="w-4 h-4" />
                <span
                    v-if="unreadNewsCount && unreadNewsCount > 0"
                    class="absolute -top-2 -right-2 bg-primary text-primary-content text-xs font-bold rounded-full min-w-5 h-5 flex items-center justify-center border-2 border-base-100 px-1"
                >
                    {{ unreadNewsCount > 9 ? '9+' : unreadNewsCount }}
                </span>
            </button>
        </div>
        <div
            class="home-action-btn relative"
            style="isolation: isolate"
        >
            <button
                @click.stop="toggleHistory"
                class="btn btn-ghost border-base-300 gap-2 btn-primary"
                :class="{ 'tooltip tooltip-bottom': !showHistory }"
                :data-tip="!showHistory ? t('history.title') : undefined"
                :style="{
                    border: 'var(--border) solid #0000',
                }"
            >
                <History class="w-4 h-4" />
            </button>
            <LaunchHistoryPanel
                v-if="showHistory"
                @close="emit('update:showHistory', false)"
                @launch="handleLaunchFromHistory"
            />
        </div>
    </div>
</template>

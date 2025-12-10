<template>
    <div class="flex flex-col h-full max-h-[80vh]">
        <div class="text-center space-y-4 shrink-0 pb-4">
            <div class="space-y-2">
                <div class="flex items-center justify-center gap-4 text-sm">
                    <div class="flex items-center gap-2 px-3 py-1 bg-base-200 rounded-full">
                        <span class="text-base-content/60">{{ t('updater.current') }}:</span>
                        <code class="text-primary font-mono">v{{ updateInfo.current_version }}</code>
                    </div>
                    <ChevronRight class="w-4 h-4 text-base-content/40" />
                    <div class="flex items-center gap-2 px-3 py-1 bg-primary/10 rounded-full">
                        <span class="text-base-content/60">{{ t('updater.latest') }}:</span>
                        <code class="text-primary font-mono font-bold">{{ updateInfo.latest_version }}</code>
                    </div>
                </div>
            </div>
        </div>

        <div class="flex-1 overflow-y-auto min-h-0 space-y-6" style="max-height: calc(80vh - 160px);">
            <div v-if="updateInfo.changelog && updateInfo.changelog.length > 0" class="space-y-4">
                <div class="divider">
                    <h4 class="font-semibold flex items-center gap-2 text-primary">
                        <History class="w-4 h-4" />
                        {{ t('updater.changelog') }}
                    </h4>
                </div>

                <div class="space-y-6">
                    <div v-for="(entry, index) in displayedChangelogEntries" :key="entry.version"
                        class="relative timeline-entry" :class="{ 'opacity-60': index > 0 }">

                        <div v-if="index < displayedChangelogEntries.length - 1"
                            class="absolute left-6 top-14 w-0.5 h-full bg-linear-to-b from-primary/40 to-transparent">
                        </div>

                        <div class="flex items-center gap-4 mb-4">
                            <div class="flex-1">
                                <div class="flex items-center gap-3 mb-1">
                                    <h5 class="text-lg font-bold">{{ entry.version }}</h5>
                                    <span class="text-xs text-base-content/50">
                                        {{ entry.date }}
                                    </span>
                                </div>

                                <div v-if="entry.highlights && entry.highlights.length > 0"
                                    class="flex flex-wrap gap-2">
                                    <span v-for="highlight in entry.highlights.slice(0, 3)" :key="highlight"
                                        class="inline-flex items-center gap-1 px-2 py-1 bg-primary/10 text-primary text-xs rounded-full">
                                        <Star class="w-3 h-3" />
                                        {{ highlight }}
                                    </span>
                                    <span v-if="entry.highlights.length > 3"
                                        class="text-xs text-base-content/50 self-center">
                                        +{{ entry.highlights.length - 3 }} more
                                    </span>
                                </div>
                            </div>
                        </div>

                        <div class="ml-16 space-y-3">
                            <div v-for="(categoryGroup, category) in groupChangesByCategory(entry.changes)"
                                :key="category" class="space-y-2">

                                <h6 class="text-sm font-medium flex items-center gap-2"
                                    :class="getCategoryHeaderClass(category)">
                                    <component :is="getCategoryIcon(category)" class="w-4 h-4" />
                                    {{ t(`updater.categories.${category}`) }}
                                    <span class="badge badge-xs">{{ categoryGroup.length }}</span>
                                </h6>

                                <div class="space-y-2">
                                    <div v-for="change in categoryGroup" :key="change.description_key"
                                        class="flex items-start gap-3 p-3 rounded-lg transition-all duration-200 hover:bg-base-200/50"
                                        :class="getCategoryBgClass(category)">
                                        <span class="text-lg shrink-0 mt-0.5">{{ change.icon }}</span>
                                        <span class="text-sm text-base-content/80 leading-relaxed">
                                            {{ resolveDescription(change, entry) }}
                                        </span>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div v-if="updateInfo.changelog.length > maxDisplayedEntries" class="text-center pb-4">
                        <button @click="toggleShowAllChangelog" class="btn btn-sm btn-ghost btn-outline">
                            <ChevronDown class="w-4 h-4 transition-transform duration-200"
                                :class="{ 'rotate-180': showAllChangelog }" />
                            {{ showAllChangelog ? t('updater.show_less') : t('updater.show_more', {
                                count: updateInfo.changelog.length - maxDisplayedEntries
                            }) }}
                        </button>
                    </div>
                </div>
            </div>
        </div>

        <div class="flex gap-3 justify-end mt-4">
            <button @click="$emit('close')" class="btn btn-ghost">
                <X class="w-4 h-4 mr-2" />
                {{ t('updater.skip') }}
            </button>
            <button @click="handleDownload" :disabled="isDownloading" class="btn btn-primary gap-2"
                :class="{ 'btn-error': updateInfo.is_critical }">
                <span v-if="isDownloading" class="loading loading-spinner loading-sm"></span>
                <Download v-else class="w-4 h-4" />
                {{ isDownloading ? t('updater.downloading') : t('updater.download_install') }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import { useI18n } from 'vue-i18n';
import {
    Download,
    FileText,
    History,
    ChevronRight,
    ChevronDown,
    Star,
    X,
    AlertTriangle,
    Zap,
    Shield,
    Plus,
    Bug,
} from 'lucide-vue-next';

interface ChangeItem {
    category: string;
    description_key: string;
    icon: string;
}

interface ChangelogEntry {
    version: string;
    changes: ChangeItem[];
    date: string;
    highlights: string[];
}

interface UpdateInfo {
    available: boolean;
    current_version: string;
    latest_version: string;
    download_url: string;
    changelog: ChangelogEntry[];
    release_date: string;
    is_critical: boolean;
}

const { t } = useI18n();

const props = defineProps<{
    updateInfo: UpdateInfo;
}>();

const emit = defineEmits<{
    download: [];
    close: [];
}>();

const isDownloading = ref(false);
const showAllChangelog = ref(false);
const maxDisplayedEntries = 3;

const displayedChangelogEntries = computed(() => {
    if (showAllChangelog.value) {
        return props.updateInfo.changelog;
    }
    return props.updateInfo.changelog.slice(0, maxDisplayedEntries);
});

const toggleShowAllChangelog = () => {
    showAllChangelog.value = !showAllChangelog.value;
};

const groupChangesByCategory = (changes: ChangeItem[]) => {
    return changes.reduce((groups, change) => {
        const category = change.category;
        if (!groups[category]) {
            groups[category] = [];
        }
        groups[category].push(change);
        return groups;
    }, {} as Record<string, ChangeItem[]>);
};

const getCategoryIcon = (category: string) => {
    switch (category) {
        case 'feature':
            return Plus;
        case 'improvement':
            return Zap;
        case 'bugfix':
            return Bug;
        case 'security':
            return Shield;
        case 'breaking':
            return AlertTriangle;
        default:
            return FileText;
    }
};

const getCategoryHeaderClass = (category: string): string => {
    switch (category) {
        case 'feature':
            return 'text-primary';
        case 'improvement':
            return 'text-info';
        case 'bugfix':
            return 'text-warning';
        case 'security':
            return 'text-error';
        case 'breaking':
            return 'text-error';
        default:
            return 'text-base-content/70';
    }
};

const getCategoryBgClass = (category: string): string => {
    switch (category) {
        case 'feature':
            return 'bg-primary/5 border border-primary/10';
        case 'improvement':
            return 'bg-info/5 border border-info/10';
        case 'bugfix':
            return 'bg-warning/5 border border-warning/10';
        case 'security':
            return 'bg-error/5 border border-error/10';
        case 'breaking':
            return 'bg-error/5 border border-error/10';
        default:
            return 'bg-base-200/30 border border-base-300/50';
    }
};

const handleDownload = async () => {
    isDownloading.value = true;
    try {
        emit('download');
    } finally {
    }
};

const resolveDescription = (change: ChangeItem, entry: ChangelogEntry) => {
    const raw = change.description_key || '';
    if (raw.includes('.')) {
        const val = t(raw as any) as string;
        if (val && val !== raw) return val;
    }

    const versionKey = `v${entry.version.replace(/\./g, '_')}`;
    const scoped = `updater.changelogs.${change.category}.${versionKey}.${raw}`;
    const scopedVal = t(scoped as any) as string;
    if (scopedVal && scopedVal !== scoped) return scopedVal;

    return raw;
};
</script>

<style scoped>
@reference "tailwindcss";
@plugin "daisyui";

.timeline-entry {
    position: relative;
}

.prose code {
    @apply bg-base-300 px-1 rounded text-sm;
}

.overflow-y-auto::-webkit-scrollbar {
    width: 6px;
}

.overflow-y-auto::-webkit-scrollbar-track {
    background: transparent;
}

.overflow-y-auto::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.2);
    border-radius: 3px;
}

.overflow-y-auto::-webkit-scrollbar-thumb:hover {
    background: rgba(255, 255, 255, 0.3);
}
</style>

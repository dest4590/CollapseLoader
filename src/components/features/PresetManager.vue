<template>
    <div class="space-y-6">
        <div class="flex items-center justify-between">
            <div>
                <h2 class="text-xl font-semibold text-primary-focus">{{ $t('theme.presets.title') }}</h2>
                <p class="text-sm text-base-content/70 mt-1">
                    {{ $t('theme.presets.description') }}
                </p>
            </div>
            <div class="flex gap-2">
                <button @click="openCreateModal" class="btn btn-primary btn-sm">
                    <Plus class="w-4 h-4 mr-2" />
                    {{ $t('theme.presets.save_current') }}
                </button>
                <button @click="openImportModal" class="btn btn-outline btn-sm">
                    <Upload class="w-4 h-4 mr-2" />
                    {{ $t('theme.presets.import') }}
                </button>
            </div>
        </div>

        <div v-if="loading" class="text-center py-8">
            <div class="loading loading-spinner loading-md"></div>
            <p class="text-sm text-base-content/70 mt-2">{{ $t('theme.presets.loading') }}</p>
        </div>

        <div v-else-if="presets.length === 0" class="text-center py-12">
            <Palette class="w-16 h-16 mx-auto text-base-content/30 mb-4" />
            <h3 class="text-lg font-medium text-base-content/70 mb-2">{{ $t('theme.presets.no_presets_title') }}</h3>
            <p class="text-sm text-base-content/50 mb-4">
                {{ $t('theme.presets.no_presets_description') }}
            </p>
            <button @click="openCreateModal" class="btn btn-primary btn-sm">
                <Plus class="w-4 h-4 mr-2" />
                {{ $t('theme.presets.create_first_preset') }}
            </button>
        </div>

        <div v-else>
            <div class="flex items-center justify-between mb-3">
                <div class="text-sm text-base-content/70">
                    {{ $t('theme.presets.pagination.page_of', { current: currentPage, total: totalPages }) }}
                </div>
                <div class="flex items-center gap-2">
                    <label class="text-xs text-base-content/60 flex-shrink-0">
                        {{ $t('theme.presets.pagination.per_page') }}
                    </label>
                    <select v-model.number="pageSize" class="select select-bordered select-xs">
                        <option :value="6">6</option>
                        <option :value="9">9</option>
                        <option :value="12">12</option>
                        <option :value="24">24</option>
                    </select>
                </div>
            </div>

            <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                <div v-for="preset in paginatedPresets" :key="preset.id"
                    class="preset-card card bg-base-200 shadow-md border border-base-300 hover:shadow-lg transition-all duration-200 cursor-pointer"
                    :class="{ 'ring ring-primary ring-offset-1 ring-offset-base-100 border-primary/60': isPresetSelected(preset.id) }"
                    @click="handlePresetClick(preset, $event)">
                    <div class="card-body p-4">
                        <div class="flex items-start justify-between mb-3">
                            <div class="flex-1">
                                <h3 class="card-title text-base">{{ preset.name }}</h3>
                                <p v-if="preset.description" class="text-xs text-base-content/70 mt-1 line-clamp-2">
                                    {{ preset.description }}
                                </p>
                            </div>
                            <div class="dropdown dropdown-end">
                                <button tabindex="0" class="btn btn-ghost btn-xs btn-circle" @click.stop>
                                    <MoreVertical class="w-4 h-4" />
                                </button>
                                <ul tabindex="0" @click.stop
                                    class="dropdown-content menu p-2 shadow bg-base-200 rounded-box w-52 border border-base-300">
                                    <li>
                                        <a @click="applyPreset(preset)">
                                            <Palette class="w-4 h-4" />
                                            {{ $t('theme.presets.apply_preset') }}
                                        </a>
                                    </li>
                                    <li>
                                        <a @click="openEditModal(preset)">
                                            <Edit class="w-4 h-4" />
                                            {{ $t('theme.presets.edit') }}
                                        </a>
                                    </li>
                                    <li>
                                        <a @click="duplicatePreset(preset)">
                                            <Copy class="w-4 h-4" />
                                            {{ $t('theme.presets.duplicate') }}
                                        </a>
                                    </li>
                                    <li>
                                        <a @click="exportPreset(preset)">
                                            <Download class="w-4 h-4" />
                                            {{ $t('theme.presets.export') }}
                                        </a>
                                    </li>
                                    <li>
                                        <a @click="openDeleteModal(preset)" class="text-error">
                                            <Trash2 class="w-4 h-4" />
                                            {{ $t('theme.presets.delete') }}
                                        </a>
                                    </li>
                                </ul>
                            </div>
                        </div>

                        <div class="text-xs text-base-content/50">
                            {{ $t('theme.presets.created', { date: formatDate(preset.created_at) }) }}
                        </div>

                        <div class="card-actions justify-end mt-auto">
                            <button @click="applyPreset(preset)" class="btn btn-primary btn-xs">
                                {{ $t('theme.presets.apply') }}
                            </button>
                        </div>
                    </div>
                </div>
            </div>

            <div class="mt-4 flex items-center justify-between">
                <button class="btn btn-sm" :disabled="currentPage === 1" @click="prevPage">{{ $t('common.previous')
                }}</button>
                <div class="text-sm text-base-content/70">{{ $t('theme.presets.pagination.page_of', {
                    current:
                        currentPage, total: totalPages
                }) }}</div>
                <button class="btn btn-sm" :disabled="currentPage === totalPages || totalPages === 0"
                    @click="nextPage">{{ $t('common.next') }}</button>
            </div>
        </div>

        <transition name="slide-up-bottom">
            <div v-if="selectedPresets.size > 0"
                class="fixed bottom-4 left-1/2 transform -translate-x-1/2 w-auto max-w-[calc(100%-2rem)] bg-neutral text-neutral-content px-4 py-3 rounded-lg shadow-xl z-30 flex items-center gap-3 sm:gap-4">
                <span class="font-medium text-xs sm:text-sm whitespace-nowrap">
                    {{ $t('theme.presets.selected_count', { count: selectedPresets.size }) }}
                </span>

                <div class="flex items-center gap-1 sm:gap-2">
                    <transition name="button-fade" mode="out-in">
                        <button v-if="canApplyFirst" @click="applyFirstSelected"
                            :title="$t('theme.presets.actions.apply_first')"
                            class="btn btn-sm btn-ghost hover:bg-neutral-focus p-2 aspect-square">
                            <Palette class="w-4 h-4 sm:w-5 sm:h-5" />
                        </button>
                    </transition>

                    <transition name="button-fade" mode="out-in">
                        <button v-if="canExportSelected" @click="exportSelectedPresets"
                            :title="$t('theme.presets.actions.export_selected')"
                            class="btn btn-sm btn-ghost hover:bg-neutral-focus p-2 aspect-square">
                            <Download class="w-4 h-4 sm:w-5 sm:h-5" />
                        </button>
                    </transition>

                    <transition name="button-fade" mode="out-in">
                        <button v-if="canDuplicateSelected" @click="duplicateSelectedPresets"
                            :title="$t('theme.presets.actions.duplicate_selected')"
                            class="btn btn-sm btn-ghost hover:bg-neutral-focus p-2 aspect-square">
                            <Copy class="w-4 h-4 sm:w-5 sm:h-5" />
                        </button>
                    </transition>

                    <transition name="button-fade" mode="out-in">
                        <button v-if="canDeleteSelected" @click="deleteSelectedPresets"
                            :title="$t('theme.presets.actions.delete_selected')"
                            class="btn btn-sm btn-ghost hover:bg-neutral-focus p-2 aspect-square">
                            <Trash2 class="w-4 h-4 sm:w-5 sm:h-5" />
                        </button>
                    </transition>
                </div>

                <div v-if="canApplyFirst || canExportSelected || canDuplicateSelected || canDeleteSelected"
                    class="border-l border-neutral-content/30 h-5 sm:h-6 mx-1"></div>

                <button @click="clearSelection" class="btn btn-sm btn-ghost hover:bg-neutral-focus p-2 aspect-square">
                    <X class="w-4 h-4 sm:w-5 sm:h-5" />
                </button>
            </div>
        </transition>
    </div>
</template>

<script setup lang="ts">
import { onMounted, onBeforeUnmount, ref, computed, watch } from 'vue';
import {
    Plus, Upload, Palette, MoreVertical, Edit, Copy, Download,
    Trash2, X
} from 'lucide-vue-next';
import type { ThemePreset } from '../../types/presets';
import { usePresets } from '../../composables/usePresets';
import { useModal } from '../../services/modalService';
import CreatePresetModal from '../modals/social/presets/CreatePresetModal.vue';
import ImportPresetModal from '../modals/social/presets/ImportPresetModal.vue';
import DeletePresetConfirmModal from '../modals/social/presets/DeletePresetConfirmModal.vue';
import { useI18n } from 'vue-i18n';
import { formatDate } from '../../utils/utils';
import { useToast } from '../../services/toastService';

const {
    presets,
    loading,
    loadPresets,
    updatePreset,
    deletePreset,
    duplicatePreset: duplicatePresetAction,
    applyPreset: applyPresetAction,
    saveCurrentAsPreset,
    exportPreset: exportPresetAction
} = usePresets();

const { showModal } = useModal();
const { t } = useI18n();
const { addToast } = useToast();

const selectedPresets = ref<Set<string>>(new Set());
const isCtrlPressed = ref(false);

const pageSize = ref<number>(9);
const currentPage = ref<number>(1);
const totalPages = computed(() => {
    const total = Math.ceil((presets.value?.length || 0) / pageSize.value);
    return Math.max(total, 1);
});
const paginatedPresets = computed(() => {
    const start = (currentPage.value - 1) * pageSize.value;
    const end = start + pageSize.value;
    return presets.value.slice(start, end);
});
function nextPage() {
    if (currentPage.value < totalPages.value) currentPage.value += 1;
}
function prevPage() {
    if (currentPage.value > 1) currentPage.value -= 1;
}
watch(pageSize, () => {
    currentPage.value = 1;
});
watch(presets, () => {
    if (currentPage.value > totalPages.value) currentPage.value = totalPages.value;
});


// formatDate imported from `src/utils/utils.ts`

const openCreateModal = () => {
    showModal(
        'create-preset',
        CreatePresetModal,
        {
            title: 'Create New Preset',
        },
        {},
        {
            'save': handleCreatePreset,
        }
    );
};

const openEditModal = (preset: ThemePreset) => {
    showModal(
        'edit-preset',
        CreatePresetModal,
        {
            title: 'Edit Preset',
        },
        { editingPreset: preset },
        {
            'update': handleUpdatePreset,
        }
    );
};

const openImportModal = () => {
    showModal(
        'import-preset',
        ImportPresetModal,
        {
            title: 'Import Preset',
        },
        {},
        {
            'import': handleImportPreset,
        }
    );
};

const openDeleteModal = (preset: ThemePreset) => {
    showModal(
        'delete-preset',
        DeletePresetConfirmModal,
        {
            title: 'Delete Preset',
        },
        { preset },
        {
            'preset-deleted': handleDeletePreset,
        }
    );
};

const handleCreatePreset = async (data: { name: string; description?: string }) => {
    await saveCurrentAsPreset(data.name, data.description);
};

const handleUpdatePreset = async (data: any) => {
    await updatePreset(data);
};

const handleDeletePreset = async (presetId: string) => {
    await deletePreset(presetId);
};

const handleImportPreset = () => {
    loadPresets();
};

const applyPreset = (preset: ThemePreset) => {
    applyPresetAction(preset);
};

const duplicatePreset = async (preset: ThemePreset) => {
    const newName = `${preset.name} (Copy)`;
    await duplicatePresetAction(preset.id, newName);
};

const exportPreset = async (preset: ThemePreset) => {
    try {
        const json = exportPresetAction(preset);
        await navigator.clipboard.writeText(json);
        addToast(t('theme.export_success'), 'success');
    } catch (e) {
        console.error('Failed to export preset:', e);
        addToast(t('theme.export_failed'), 'error');
    }
};

const handleKeyDown = (event: KeyboardEvent) => {
    if (event.key === 'Control') {
        isCtrlPressed.value = true;
    }
    if (
        event.ctrlKey &&
        (event.key === 'a' || event.key === 'ф') &&
        !(event.target as HTMLElement).matches('input, textarea, [contenteditable]')
    ) {
        event.preventDefault();
        selectAllPresets();
    }
    if (event.key === 'Escape') {
        clearSelection();
    }
};

const handleKeyUp = (event: KeyboardEvent) => {
    if (event.key === 'Control') {
        isCtrlPressed.value = false;
    }
};

const handleDocumentClick = (event: MouseEvent) => {
    const target = event.target as HTMLElement;
    if (!target.closest('.preset-card')) {
        clearSelection();
    }
};

onMounted(() => {
    loadPresets();
    document.addEventListener('keydown', handleKeyDown);
    document.addEventListener('keyup', handleKeyUp);
    document.addEventListener('click', handleDocumentClick);
});

onBeforeUnmount(() => {
    document.removeEventListener('keydown', handleKeyDown);
    document.removeEventListener('keyup', handleKeyUp);
    document.removeEventListener('click', handleDocumentClick);
});

const handlePresetClick = (preset: ThemePreset, event: MouseEvent) => {
    if (isCtrlPressed.value) {
        event.preventDefault();
        event.stopPropagation();
        if (selectedPresets.value.has(preset.id)) {
            selectedPresets.value.delete(preset.id);
        } else {
            selectedPresets.value.add(preset.id);
        }
        selectedPresets.value = new Set(selectedPresets.value);
    } else {
        selectedPresets.value.clear();
    }
};

const isPresetSelected = (id: string): boolean => {
    return selectedPresets.value.has(id);
};

const clearSelection = () => {
    selectedPresets.value.clear();
};

const selectAllPresets = () => {
    const allIds = presets.value.map(p => p.id);
    selectedPresets.value = new Set(allIds);
};

const getSelectedPresetsData = (): ThemePreset[] => {
    return presets.value.filter(p => selectedPresets.value.has(p.id));
};

const canDeleteSelected = computed(() => selectedPresets.value.size > 0);
const canDuplicateSelected = computed(() => selectedPresets.value.size > 0);
const canExportSelected = computed(() => selectedPresets.value.size > 0);
const canApplyFirst = computed(() => selectedPresets.value.size > 0);

const applyFirstSelected = () => {
    const ids = Array.from(selectedPresets.value);
    const first = presets.value.find(p => p.id === ids[0]);
    if (first) {
        applyPresetAction(first);
    }
};

const deleteSelectedPresets = async () => {
    const items = getSelectedPresetsData();
    if (items.length === 0) return;
    for (const p of items) {
        await deletePreset(p.id);
    }
    await loadPresets();
    addToast(t('theme.presets.messages.multiple_deleted', { count: items.length }), 'success');
    clearSelection();
};

const duplicateSelectedPresets = async () => {
    const items = getSelectedPresetsData();
    if (items.length === 0) return;
    for (const p of items) {
        const newName = `${p.name} (Copy)`;
        await duplicatePresetAction(p.id, newName);
    }
    await loadPresets();
    addToast(t('theme.presets.messages.multiple_duplicated', { count: items.length }), 'success');
    clearSelection();
};

const exportSelectedPresets = async () => {
    try {
        const items = getSelectedPresetsData();
        if (items.length === 0) return;
        const exported = items.map(p => {
            try {
                return JSON.parse(exportPresetAction(p));
            } catch {
                return null;
            }
        }).filter(Boolean);
        const json = JSON.stringify(exported, null, 2);
        await navigator.clipboard.writeText(json);
        addToast(t('theme.presets.messages.multiple_exported', { count: exported.length }), 'success');
    } catch (e) {
        console.error('Failed to export selected presets:', e);
        addToast(t('theme.export_failed'), 'error');
    }
};
</script>

<style>
.slide-up-bottom-enter-active,
.slide-up-bottom-leave-active {
    transition: all 0.4s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.slide-up-bottom-enter-from,
.slide-up-bottom-leave-to {
    opacity: 0;
    transform: translateY(100px);
}

.button-fade-enter-active,
.button-fade-leave-active {
    transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
}

.button-fade-enter-from {
    opacity: 0;
    transform: scale(0.8) translateY(10px);
}

.button-fade-leave-to {
    opacity: 0;
    transform: scale(0.8) translateY(-10px);
}

.button-fade-enter-to,
.button-fade-leave-from {
    opacity: 1;
    transform: scale(1) translateY(0);
}
</style>
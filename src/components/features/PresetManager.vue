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

        <div v-else class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
            <div v-for="preset in presets" :key="preset.id"
                class="card bg-base-200 shadow-md border border-base-300 hover:shadow-lg transition-all duration-200">
                <div class="card-body p-4">
                    <div class="flex items-start justify-between mb-3">
                        <div class="flex-1">
                            <h3 class="card-title text-base">{{ preset.name }}</h3>
                            <p v-if="preset.description" class="text-xs text-base-content/70 mt-1 line-clamp-2">
                                {{ preset.description }}
                            </p>
                        </div>
                        <div class="dropdown dropdown-end">
                            <button tabindex="0" class="btn btn-ghost btn-xs btn-circle">
                                <MoreVertical class="w-4 h-4" />
                            </button>
                            <ul tabindex="0"
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
    </div>
</template>

<script setup lang="ts">
import { onMounted } from 'vue';
import {
    Plus, Upload, Palette, MoreVertical, Edit, Copy, Download,
    Trash2
} from 'lucide-vue-next';
import type { ThemePreset } from '../../types/presets';
import { usePresets } from '../../composables/usePresets';
import { useModal } from '../../services/modalService';
import CreatePresetModal from '../modals/CreatePresetModal.vue';
import ImportPresetModal from '../modals/ImportPresetModal.vue';
import DeletePresetConfirmModal from '../modals/DeletePresetConfirmModal.vue';

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

const formatDate = (dateString: string): string => {
    try {
        const date = new Date(dateString);
        if (isNaN(date.getTime())) return 'Unknown';
        const day = String(date.getDate()).padStart(2, '0');
        const month = String(date.getMonth() + 1).padStart(2, '0');
        const year = String(date.getFullYear());
        return `${day}/${month}/${year}`;
    } catch {
        return 'Unknown';
    }
};

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

const exportPreset = (preset: ThemePreset) => {
    const dataStr = exportPresetAction(preset);
    const dataBlob = new Blob([dataStr], { type: 'application/json' });
    const url = URL.createObjectURL(dataBlob);
    const link = document.createElement('a');
    link.href = url;
    link.download = `${preset.name.replace(/[^a-z0-9]/gi, '_').toLowerCase()}_preset.json`;
    link.click();
    URL.revokeObjectURL(url);
};

onMounted(() => {
    loadPresets();
});
</script>
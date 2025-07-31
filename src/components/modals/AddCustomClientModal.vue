<script setup lang="ts">
import { ref, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useToast } from '../../services/toastService';
import { useI18n } from 'vue-i18n';

const { t } = useI18n();
const { addToast } = useToast();

const emit = defineEmits<{
    'client-added': [];
    'close': [];
}>();

const form = reactive({
    name: '',
    version: '1.16.5',
    mainClass: '',
    filePath: '',
    fileName: '',
});

const loading = ref(false);
const errors = ref<Record<string, string>>({});

const availableVersions = [
    '1.16.5',
    '1.12.2',
    'Custom'
];

const validateForm = () => {
    errors.value = {};

    if (!form.name.trim()) {
        errors.value.name = 'Name is required';
    }

    if (!form.mainClass.trim()) {
        errors.value.mainClass = 'Main class is required';
    }

    if (!form.filePath) {
        errors.value.filePath = 'Please select a .jar file';
    }

    return Object.keys(errors.value).length === 0;
};

const selectFile = async () => {
    try {
        const selected = await open({
            multiple: false,
            filters: [{
                name: 'JAR Files',
                extensions: ['jar']
            }]
        });

        if (selected && typeof selected === 'string') {
            form.filePath = selected;
            // Extract filename from path for display
            const pathParts = selected.split(/[/\\]/);
            form.fileName = pathParts[pathParts.length - 1];
        }
    } catch (error) {
        addToast('Failed to select file', 'error');
    }
};

const handleSubmit = async () => {
    if (!validateForm()) {
        return;
    }

    try {
        loading.value = true;

        await invoke('add_custom_client', {
            name: form.name.trim(),
            version: form.version,
            filename: form.fileName,
            filePath: form.filePath,
            mainClass: form.mainClass.trim(),
        });

        Object.assign(form, {
            name: '',
            version: '1.16.5',
            mainClass: '',
            filePath: '',
            fileName: '',
        });

        emit('client-added');
    } catch (err) {
        addToast(`Failed to add custom client: ${err}`, 'error');
    } finally {
        loading.value = false;
    }
};
</script>

<template>
    <form @submit.prevent="handleSubmit" class="space-y-4">
        <div class="form-control">
            <label class="label">
                <span class="label-text">Client Name *</span>
            </label>
            <input v-model="form.name" type="text" placeholder="Enter client name" class="input input-bordered"
                :class="{ 'input-error': errors.name }" />
            <label v-if="errors.name" class="label">
                <span class="label-text-alt text-error">{{ errors.name }}</span>
            </label>
        </div>

        <div class="form-control">
            <label class="label">
                <span class="label-text">Minecraft Version *</span>
            </label>
            <select v-model="form.version" class="select select-bordered">
                <option v-for="version in availableVersions" :key="version" :value="version">
                    {{ version }}
                </option>
            </select>
        </div>

        <div class="form-control">
            <label class="label">
                <span class="label-text">Main Class *</span>
            </label>
            <input v-model="form.mainClass" type="text" placeholder="e.g., net.minecraft.client.main.Main"
                class="input input-bordered" :class="{ 'input-error': errors.mainClass }" />
            <label v-if="errors.mainClass" class="label">
                <span class="label-text-alt text-error">{{ errors.mainClass }}</span>
            </label>
        </div>


        <div class="form-control">
            <label class="label">
                <span class="label-text">JAR File *</span>
            </label>
            <div class="flex gap-2">
                <input :value="form.fileName || 'No file selected'" type="text" placeholder="Select a .jar file"
                    class="input input-bordered flex-1" readonly :class="{ 'input-error': errors.filePath }" />
                <button type="button" @click="selectFile" class="btn btn-outline">
                    Browse
                </button>
            </div>
            <label v-if="errors.filePath" class="label">
                <span class="label-text-alt text-error">{{ errors.filePath }}</span>
            </label>
            <label v-if="form.fileName" class="label">
                <span class="label-text-alt text-success">Selected: {{ form.fileName }}</span>
            </label>
        </div>

        <div class="modal-action">
            <button type="button" class="btn" @click="$emit('close')" :disabled="loading">
                Cancel
            </button>
            <button type="submit" class="btn btn-primary" :disabled="loading">
                <div v-if="loading" class="loading loading-spinner loading-sm"></div>
                {{ loading ? 'Adding...' : 'Add Client' }}
            </button>
        </div>
    </form>
</template>
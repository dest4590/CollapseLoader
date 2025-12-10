<script setup lang="ts">
import { ref, reactive } from 'vue';
import { invoke } from '@tauri-apps/api/core';
import { open } from '@tauri-apps/plugin-dialog';
import { useToast } from '../../../services/toastService';

const { addToast } = useToast();

const emit = defineEmits<{
    'client-added': [];
    'close': [];
}>();

const form = reactive({
    name: '',
    version: '',
    mainClass: 'net.minecraft.client.main.Main',
    filePath: '',
    fileName: '',
});

const loading = ref(false);
const errors = ref<Record<string, string>>({});

const validateForm = () => {
    errors.value = {};

    if (!form.name.trim()) {
        errors.value.name = 'Name is required';
    }

    if (!form.version.trim()) {
        errors.value.version = 'Version is required';
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
            const pathParts = selected.split(/[/\\]/);
            form.fileName = pathParts[pathParts.length - 1];

            try {
                const mainClass = await invoke<string>('detect_main_class', { filePath: selected });
                if (mainClass) {
                    form.mainClass = mainClass;
                    addToast('Main class detected successfully', 'success');
                }
            } catch (e) {
                console.warn('Failed to detect main class:', e);
            }
        }
    } catch (error) {
        console.error('File selection error:', error);
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
            version: '',
            mainClass: 'net.minecraft.client.main.Main',
            filePath: '',
            fileName: '',
        });

        emit('client-added');
        emit('close');
    } catch (err) {
        addToast(`Failed to add custom client: ${err}`, 'error');
    } finally {
        loading.value = false;
    }
};
</script>

<template>
    <form @submit.prevent="handleSubmit" class="space-y-4 ">
        <div class="form-scroll-area">
            <div class="form-control">
                <label class="label">
                    <span class="label-text">{{ $t('modals.add_custom_client_modal.client_name') }} *</span>
                </label>
                <input v-model="form.name" type="text" :placeholder="$t('modals.add_custom_client_modal.enter_client_name')" class="input input-bordered"
                    :class="{ 'input-error': errors.name }" />
                <label v-if="errors.name" class="label">
                    <span class="label-text-alt text-error">{{ errors.name }}</span>
                </label>
            </div>

            <div class="form-control">
                <label class="label">
                    <span class="label-text">{{ $t('modals.add_custom_client_modal.minecraft_version') }} *</span>
                </label>
                <input v-model="form.version" type="text" placeholder="e.g. 1.16.5" class="input input-bordered"
                    :class="{ 'input-error': errors.version }" />
                <label v-if="errors.version" class="label">
                    <span class="label-text-alt text-error">{{ errors.version }}</span>
                </label>
            </div>

            <div class="form-control">
                <label class="label">
                    <span class="label-text">{{ $t('modals.add_custom_client_modal.main_class') }} *</span>
                </label>
                <input v-model="form.mainClass" type="text" :placeholder="$t('modals.add_custom_client_modal.main_class_placeholder')"
                    class="input input-bordered" :class="{ 'input-error': errors.mainClass }" />
                <label v-if="errors.mainClass" class="label">
                    <span class="label-text-alt text-error">{{ errors.mainClass }}</span>
                </label>
            </div>

            <div class="form-control">
                <label class="label">
                    <span class="label-text">{{ $t('modals.add_custom_client_modal.jar_file') }} *</span>
                </label>
                <div class="flex gap-2">
                    <input :value="form.fileName || $t('modals.add_custom_client_modal.no_file_selected')" type="text" :placeholder="$t('modals.add_custom_client_modal.select_jar_file')"
                        class="input input-bordered flex-1" readonly :class="{ 'input-error': errors.filePath }" />
                    <button type="button" @click="selectFile" class="btn btn-outline">
                        {{ $t('common.browse') }}
                    </button>
                </div>
                <label v-if="errors.filePath" class="label">
                    <span class="label-text-alt text-error">{{ errors.filePath }}</span>
                </label>
                <label v-if="form.fileName" class="label">
                    <span class="label-text-alt text-success">{{ $t('modals.add_custom_client_modal.selected') }}: {{ form.fileName }}</span>
                </label>
            </div>
        </div>
        <div class="modal-action">
            <button type="button" class="btn" @click="$emit('close')" :disabled="loading">
                {{ $t('common.cancel') }}
            </button>
            <button type="submit" class="btn btn-primary" :disabled="loading">
                <div v-if="loading" class="loading loading-spinner loading-sm"></div>
                {{ loading ? $t('modals.add_custom_client_modal.adding') : $t('modals.add_custom_client_modal.add_client') }}
            </button>
        </div>
    </form>
</template>
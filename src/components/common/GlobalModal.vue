<template>
    <div class="global-modals">
        <template v-for="(modal, id) in modals" :key="id">
            <CustomModal v-model="modal.open" :title="modal.title" :content-class="modal.contentClass"
                @close="closeModal(id)">
                <template #header>
                    <slot :name="`header-${id}`"></slot>
                </template>
                <template #body>
                    <component :is="modal.component" v-bind="modal.props || {}" v-on="modal.listeners || {}"
                        @close="closeModal(id)" :key="`${id}-${locale}`" />
                </template>
                <template #footer>
                    <slot :name="`footer-${id}`"></slot>
                </template>
            </CustomModal>
        </template>
    </div>
</template>

<script setup lang="ts">
import { useModal } from '../../services/modalService';
import { useI18n } from 'vue-i18n';
import CustomModal from '../ui/CustomModal.vue';

const { getModals, hideModal } = useModal();
const { locale } = useI18n();

const modals = getModals();

const closeModal = (id: string) => {
    hideModal(id);
};
</script>

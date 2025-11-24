<script setup lang="ts">
import { defineProps, defineEmits } from 'vue';

const props = defineProps<{
    account: {
        id: string;
        username: string;
        tags: string[];
        created_at: string;
        last_used?: string;
        is_active: boolean;
    };
    formatDate?: (v: string) => string;
}>();

const emit = defineEmits<{
    (e: 'set-active', account: any): void;
    (e: 'edit-account', account: any): void;
    (e: 'delete-account', account: any): void;
}>();

const handleSetActive = () => emit('set-active', props.account);
const handleEdit = () => emit('edit-account', props.account);
const handleDelete = () => emit('delete-account', props.account);
</script>

<template>
    <div class="card bg-base-200 shadow-md border border-base-300 account-card overflow-x-hidden">
        <div class="card-body p-4">
            <div class="flex justify-between items-center">
                <div class="flex-1 space-y-1 min-w-0 pr-2">
                    <div class="flex items-center gap-2">
                        <h3 class="font-semibold text-lg text-primary-focus truncate">{{ account.username }}</h3>
                        <div v-if="account.is_active" class="badge badge-success badge-sm whitespace-nowrap">Active
                        </div>
                    </div>
                    <p class="text-sm text-base-content/70 flex items-center gap-2">
                        <span class="font-medium text-primary/80">Tags:</span>
                        <span class="flex flex-wrap gap-1">
                            <span v-for="tag in account.tags" :key="tag" class="badge badge-outline badge-xs">{{ tag
                                }}</span>
                        </span>
                    </p>
                    <p class="text-xs text-base-content/60 flex items-center gap-2">
                        <span>Created: {{ formatDate ? formatDate(account.created_at) : account.created_at }}</span>
                        <span v-if="account.last_used" class="border-l border-base-content/30 pl-2">Last used: {{
                            formatDate ? formatDate(account.last_used) : account.last_used }}</span>
                    </p>
                </div>
                <div class="flex items-center space-x-2">
                    <button @click="handleSetActive" class="btn btn-sm btn-circle" :class="account.is_active
                        ? 'btn-success text-success-content'
                        : 'btn-outline btn-success hover:text-success-content'">
                        <slot name="user-icon"><svg width="16" height="16" viewBox="0 0 24 24" fill="none"
                                stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2" />
                                <circle cx="12" cy="7" r="4" />
                            </svg></slot>
                    </button>
                    <button @click="handleEdit"
                        class="btn btn-sm btn-circle btn-outline btn-warning hover:text-warning-content">
                        <slot name="edit-icon"><svg width="16" height="16" viewBox="0 0 24 24" fill="none"
                                stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <path d="M11 4H4a2 2 0 0 0-2 2v14l4-1 1 1h12a2 2 0 0 0 2-2v-7" />
                                <path d="M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4z" />
                            </svg></slot>
                    </button>
                    <button @click="handleDelete"
                        class="btn btn-sm btn-circle btn-outline btn-error hover:text-error-content">
                        <slot name="delete-icon"><svg width="16" height="16" viewBox="0 0 24 24" fill="none"
                                stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                <polyline points="3 6 5 6 21 6" />
                                <path d="M19 6l-1 14a2 2 0 0 1-2 2H8a2 2 0 0 1-2-2L5 6" />
                                <path d="M10 11v6" />
                                <path d="M14 11v6" />
                                <path d="M9 6V4a1 1 0 0 1 1-1h4a1 1 0 0 1 1 1v2" />
                            </svg></slot>
                    </button>
                </div>
            </div>
        </div>
    </div>
</template>

<style scoped>
.account-card {
    opacity: 0;
    transform: translateY(10px);
    animation: fadeInUp 0.4s ease-out forwards;
}



.account-list-enter-active,
.account-list-leave-active {
    transition:
        opacity 0.3s ease,
        transform 0.3s ease;
}
</style>
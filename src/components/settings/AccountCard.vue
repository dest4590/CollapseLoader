<script setup lang="ts">
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
    <div class="card bg-base-200 shadow-sm border border-base-300 account-card hover:border-primary/20 transition-colors">
        <div class="card-body p-4">
            <div class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4">
                <div class="flex-1 min-w-0 space-y-1.5">
                    <div class="flex items-center gap-3">
                        <h3 class="font-semibold text-base text-base-content truncate">{{ account.username }}</h3>
                        <div v-if="account.is_active" class="badge badge-success badge-sm gap-1">
                            <div class="w-1.5 h-1.5 rounded-full bg-current"></div>
                            Active
                        </div>
                    </div>
                    
                    <div class="flex flex-wrap gap-2 items-center text-xs text-base-content/60">
                        <span class="flex items-center gap-1.5 bg-base-300/50 px-2 py-1 rounded-md" v-if="account.tags.length">
                            <span v-for="tag in account.tags" :key="tag" class="font-medium text-primary">{{ tag }}</span>
                        </span>
                        <span v-if="account.tags.length" class="w-1 h-1 rounded-full bg-base-content/20"></span>
                        <span>Created {{ formatDate ? formatDate(account.created_at) : account.created_at }}</span>
                        <template v-if="account.last_used">
                            <span class="w-1 h-1 rounded-full bg-base-content/20"></span>
                            <span>Used {{ formatDate ? formatDate(account.last_used) : account.last_used }}</span>
                        </template>
                    </div>
                </div>

                <div class="flex items-center gap-2 w-full sm:w-auto justify-end">
                    <button @click="handleSetActive" class="btn btn-sm btn-square" :class="account.is_active
                        ? 'btn-success text-success-content'
                        : 'btn-ghost hover:bg-success/10 hover:text-success'">
                        <slot name="user-icon" />
                    </button>
                    <button @click="handleEdit"
                        class="btn btn-sm btn-square btn-ghost hover:bg-warning/10 hover:text-warning">
                        <slot name="edit-icon" />
                    </button>
                    <button @click="handleDelete"
                        class="btn btn-sm btn-square btn-ghost hover:bg-error/10 hover:text-error">
                        <slot name="delete-icon" />
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
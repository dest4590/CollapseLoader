<script setup lang="ts">
import { useI18n } from "vue-i18n";
import { GripVertical } from "lucide-vue-next";

const { t } = useI18n();

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
    isDragging?: boolean;
    isDragOver?: boolean;
    accountId?: string;
}>();

const emit = defineEmits<{
    (e: "set-active", account: any): void;
    (e: "edit-account", account: any): void;
    (e: "delete-account", account: any): void;
    (e: "drag-start", event: MouseEvent): void;
}>();

const handleSetActive = () => emit("set-active", props.account);
const handleEdit = () => emit("edit-account", props.account);
const handleDelete = () => emit("delete-account", props.account);
const handleDragStart = (e: MouseEvent) => emit("drag-start", e);
</script>

<template>
    <div
        class="card bg-base-200 shadow-sm border account-card transition-all duration-150 select-none"
        :data-account-id="account.id"
        :class="{
            'opacity-40 scale-[0.98] border-primary/30': isDragging,
            'border-primary shadow-md ring-2 ring-primary/30':
                isDragOver && !isDragging,
            'border-base-300 hover:border-primary/20':
                !isDragging && !isDragOver,
        }"
    >
        <div
            v-if="isDragOver && !isDragging"
            class="h-0.5 bg-primary rounded-full mx-3 mt-2"
        />

        <div class="card-body p-4">
            <div class="flex items-center gap-2">
                <div
                    class="drag-handle shrink-0 cursor-grab active:cursor-grabbing text-base-content/30 hover:text-base-content/70 transition-colors p-1 rounded"
                    @mousedown="handleDragStart"
                    title="Перетащить"
                >
                    <GripVertical class="w-4 h-4" />
                </div>

                <div
                    class="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-3 flex-1 min-w-0"
                >
                    <div class="flex-1 min-w-0 space-y-1.5">
                        <div class="flex items-center gap-3">
                            <h3
                                class="font-semibold text-base text-base-content truncate"
                            >
                                {{ account.username }}
                            </h3>
                            <div
                                v-if="account.is_active"
                                class="badge badge-success badge-sm gap-1 shrink-0"
                            >
                                <div
                                    class="w-1.5 h-1.5 rounded-full bg-current"
                                ></div>
                                {{ t("settings.active") }}
                            </div>
                        </div>

                        <div
                            class="flex flex-wrap gap-2 items-center text-xs text-base-content/60"
                        >
                            <span
                                class="flex items-center gap-1.5 bg-base-300/50 px-2 py-1 rounded-md"
                                v-if="account.tags.length"
                            >
                                <span
                                    v-for="tag in account.tags"
                                    :key="tag"
                                    class="font-medium text-primary"
                                    >{{ tag }}</span
                                >
                            </span>
                            <span
                                v-if="account.tags.length"
                                class="w-1 h-1 rounded-full bg-base-content/20"
                            ></span>
                            <span
                                >{{ t("settings.created") }}
                                {{
                                    formatDate
                                        ? formatDate(account.created_at)
                                        : account.created_at
                                }}</span
                            >
                            <template v-if="account.last_used">
                                <span
                                    class="w-1 h-1 rounded-full bg-base-content/20"
                                ></span>
                                <span
                                    >{{ t("settings.last_used") }}
                                    {{
                                        formatDate
                                            ? formatDate(account.last_used)
                                            : account.last_used
                                    }}</span
                                >
                            </template>
                        </div>
                    </div>

                    <div
                        class="flex items-center gap-2 w-full sm:w-auto justify-end shrink-0"
                    >
                        <button
                            @click="handleSetActive"
                            class="btn btn-sm btn-square"
                            :class="
                                account.is_active
                                    ? 'btn-success text-success-content'
                                    : 'btn-ghost hover:bg-success/10 hover:text-success'
                            "
                        >
                            <slot name="user-icon" />
                        </button>
                        <button
                            @click="handleEdit"
                            class="btn btn-sm btn-square btn-ghost hover:bg-warning/10 hover:text-warning"
                        >
                            <slot name="edit-icon" />
                        </button>
                        <button
                            @click="handleDelete"
                            class="btn btn-sm btn-square btn-ghost hover:bg-error/10 hover:text-error"
                        >
                            <slot name="delete-icon" />
                        </button>
                    </div>
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

.account-card.opacity-40 {
    animation: none;
    opacity: 0.4 !important;
    transform: scale(0.98) !important;
}

@keyframes fadeInUp {
    to {
        opacity: 1;
        transform: translateY(0);
    }
}

.drag-handle:hover {
    background: rgba(var(--color-base-300, 0 0 0) / 0.3);
}
</style>

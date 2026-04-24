<template>
    <div class="space-y-4">
        <p class="text-sm text-base-content/70 mb-4">
            {{ $t('modals.role_selection.description') || 'Выберите вашу роль для локального профиля' }}
        </p>
        
        <div class="grid gap-2">
            <button 
                v-for="role in availableRoles" 
                :key="role.id"
                @click="selectRole(role.id)"
                class="btn btn-ghost justify-between hover:bg-base-300"
                :class="{ 'bg-base-300 border-primary': currentRole === role.id }"
            >
                <span class="flex items-center gap-2">
                    <span :class="role.badgeClass">{{ role.name }}</span>
                </span>
                <Check v-if="currentRole === role.id" class="w-4 h-4 text-primary" />
            </button>
        </div>

        <div class="flex justify-end mt-4">
            <button @click="$emit('close')" class="btn btn-outline">
                {{ $t('common.close') }}
            </button>
        </div>
    </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useI18n } from 'vue-i18n';
import { Check } from 'lucide-vue-next';
import getRoleBadge from '../../../../utils/roleBadge';

const props = defineProps<{
    currentRole: string;
}>();

const emit = defineEmits(['close', 'role-selected']);
const { t } = useI18n();

const availableRoles = computed(() => {
    const roleKeys = ['OWNER', 'DEVELOPER', 'ADMIN', 'TESTER', 'USER', 'LOCAL_USER'];
    return roleKeys.map(key => {
        const badge = getRoleBadge(key, (k: string) => t(k));
        return {
            id: key,
            name: badge.text,
            badgeClass: badge.className
        };
    });
});

const selectRole = (roleId: string) => {
    emit('role-selected', roleId);
    emit('close');
};
</script>

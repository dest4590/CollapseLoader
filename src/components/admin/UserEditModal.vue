<template>
    <div class="p-4">
        <h3 class="text-lg font-semibold mb-2">{{ title }}</h3>

        <div class="space-y-3">
            <div>
                <label class="label">
                    <span class="label-text">Email</span>
                </label>
                <input v-model="localEmail" type="email" class="input input-bordered w-full" />
            </div>

            <div>
                <label class="label">
                    <span class="label-text">Nickname</span>
                </label>
                <input v-model="localNickname" type="text" class="input input-bordered w-full" />
            </div>

            <div class="user-flags">
                <label class="flex items-center gap-2">
                    <input type="checkbox" v-model="localIsActive" class="checkbox checkbox-sm" />
                    <span class="text-sm">Active</span>
                </label>

                <label class="flex items-center gap-2">
                    <input type="checkbox" v-model="localIsStaff" class="checkbox checkbox-sm" />
                    <span class="text-sm">Staff</span>
                </label>
            </div>

            <div>
                <label class="label">
                    <span class="label-text">Role</span>
                </label>
                <select v-model="localRole" class="select select-bordered w-full">
                    <option value="user">User</option>
                    <option value="tester">Tester</option>
                    <option value="admin">Admin</option>
                    <option value="developer">Developer</option>
                    <option value="owner">Owner</option>
                </select>
            </div>

            <div v-if="error" class="text-sm text-error">{{ error }}</div>

            <div class="flex justify-end gap-2 mt-3">
                <button class="btn btn-ghost btn-sm" @click="$emit('cancel')">Cancel</button>
                <button class="btn btn-primary btn-sm" :disabled="saving" @click="save">Save</button>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';

const props = defineProps({
    title: { type: String, default: 'Edit user' },
    email: { type: String, default: '' },
    nickname: { type: String, default: '' },
    is_active: { type: Boolean, default: true },
    is_staff: { type: Boolean, default: false },
    role: { type: String, default: 'user' },
});

const emit = defineEmits<{
    (e: 'save', payload: { email: string; nickname: string; is_active?: boolean; is_staff?: boolean; role?: string }): void;
    (e: 'cancel'): void;
}>();

const localEmail = ref(props.email || '');
const localNickname = ref(props.nickname || '');
const localIsActive = ref(props.is_active ?? true);
const localIsStaff = ref(props.is_staff ?? false);
const localRole = ref(props.role || 'user');
const saving = ref(false);
const error = ref('');

const save = () => {
    error.value = '';
    if (!localEmail.value || !localEmail.value.includes('@')) {
        error.value = 'Please enter a valid email';
        return;
    }

    saving.value = true;
    setTimeout(() => {
        saving.value = false;
    emit('save', { email: localEmail.value, nickname: localNickname.value, is_active: localIsActive.value, is_staff: localIsStaff.value, role: localRole.value });
    }, 250);
};
</script>

<style scoped>
.user-flags {
    display: flex;
    gap: 0.5rem;
    align-items: center
}
</style>

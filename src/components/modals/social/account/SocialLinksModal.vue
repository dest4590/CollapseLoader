<template>
    <div class="space-y-4 max-h-[70vh] overflow-hidden flex flex-col">
        <div v-if="loading" class="text-center py-6">
            <span>{{ t('common.loading') }}…</span>
        </div>

        <div v-else class="flex-1 min-h-0 overflow-y-auto">
            <div v-if="isStreamerMode" class="p-4 text-center text-base-content/70">
                {{ t('modals.social_links.hidden_in_streamer_mode') }}
            </div>

            <div v-else>
                <div v-if="links.length === 0" class="text-sm text-base-content/70 p-2">
                    {{ t('account.no_social_links') }}
                </div>

                <div class="space-y-2 p-2">
                    <div v-for="link in links" :key="link.platform"
                        class="social-link-row flex flex-col sm:flex-row sm:items-center sm:justify-between p-2 rounded-md bg-base-100">
                        <div class="flex-1 min-w-0">
                            <div class="font-medium truncate">{{ platformLabel(link.platform) }}</div>
                            <a v-if="link.platform !== 'DISCORD'" :href="platformHref(link.platform, link.url)"
                                target="_blank"
                                class="text-sm text-primary hover:underline truncate block wrap-break-word"
                                rel="noreferrer">{{ displayHref(link.platform, link.url) }}</a>
                            <div v-else class="text-sm truncate block">{{ displayHandle(link.platform, link.url) }}
                            </div>
                        </div>

                        <div class="flex items-center gap-2 mt-2 sm:mt-0 ml-0 sm:ml-4 shrink-0">
                            <button v-if="!isStreamerMode" type="button" @click="startEdit(link)"
                                class="btn btn-ghost btn-sm">{{ t('common.edit') }}</button>
                            <button v-if="!isStreamerMode" type="button" @click="confirmDelete(link)"
                                class="btn btn-ghost btn-sm text-error">{{ t('common.delete') }}</button>
                        </div>
                    </div>
                </div>
            </div>
        </div>

        <div class="pt-4 border-t border-base-300 mt-4 shrink-0">
            <div v-if="!editing && !isStreamerMode">
                <h4 class="font-medium mb-2">{{ t('modals.social_links.add_title') }}</h4>
                <div class="grid grid-cols-1 gap-2">
                    <select v-model="newPlatform" class="select select-bordered w-full bg-base-100">
                        <option value="">Select platform</option>
                        <option v-for="(label, key) in platformOptions" :key="key" :value="key">{{ label }}</option>
                    </select>
                    <input v-model="newUrl" type="text" class="input input-bordered w-full bg-base-100"
                        placeholder="username or full URL (e.g. @name or https://github.com/name)" />
                    <div class="flex justify-end gap-2">
                        <button @click="addLink" class="btn btn-primary">{{ t('common.add') }}</button>
                        <button @click="$emit('close')" class="btn btn-outline">{{ t('common.cancel') }}</button>
                    </div>
                </div>
            </div>

            <div v-else-if="!isStreamerMode" class="mt-2">
                <h4 class="font-medium mb-2">{{ t('modals.social_links.edit_title') }}</h4>
                <div class="grid grid-cols-1 gap-2">
                    <select v-model="editing.platform" class="select select-bordered w-full bg-base-100">
                        <option v-for="(label, key) in platformOptions" :key="key" :value="key">{{ label }}</option>
                    </select>
                    <input v-model="editing.url" type="text" class="input input-bordered w-full bg-base-100" />
                    <div class="flex justify-end gap-2">
                        <button @click="saveEdit" class="btn btn-primary">{{ t('common.save') }}</button>
                        <button @click="cancelEdit" class="btn btn-outline">{{ t('common.cancel') }}</button>
                    </div>
                </div>
            </div>
        </div>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { globalUserStatus } from '../../../../composables/useUserStatus';
import { useI18n } from 'vue-i18n';
import { useToast } from '../../../../services/toastService';
import { apiGet, apiPut } from '../../../../services/apiClient';

const { t } = useI18n();
const { addToast } = useToast();

const links = ref<any[]>([]);
const loading = ref(true);

const isStreamerMode = computed(() => globalUserStatus.isStreamer.value);

const platformOptions: Record<string, string> = {
    DISCORD: 'Discord',
    TELEGRAM: 'Telegram',
    GITHUB: 'GitHub',
    YOUTUBE: 'YouTube',
};

const newPlatform = ref('');
const newUrl = ref('');

const editing = ref<any | null>(null);

const loadLinks = async () => {
    loading.value = true;
    try {
        const resp = await apiGet<any[]>('/users/me/social-links');
        links.value = resp || [];
    } catch (error) {
        console.error('Failed to load social links', error);
        addToast(t('modals.social_links.load_failed'), 'error');
    } finally {
        loading.value = false;
    }
};

const platformLabel = (key: string) => platformOptions[key] || key;

const updateSocialLinks = async (updatedLinks: any[]) => {
    const payload = {
        links: updatedLinks.map(l => ({
            platform: l.platform,
            url: l.url
        }))
    };
    const resp = await apiPut<any[]>('/users/me/social-links', payload);
    links.value = resp || [];
};

const addLink = async () => {
    if (isStreamerMode.value) return;
    if (!newPlatform.value || !newUrl.value.trim()) {
        addToast(t('modals.social_links.fill_fields'), 'error');
        return;
    }

    // Prevent adding duplicate platforms
    if (links.value.some(l => l.platform === newPlatform.value)) {
        addToast(t('modals.social_links.platform_exists') || t('modals.social_links.add_failed'), 'error');
        return;
    }

    try {
        const newLinks = [...links.value, { platform: newPlatform.value, url: newUrl.value.trim() }];
        await updateSocialLinks(newLinks);
        newPlatform.value = '';
        newUrl.value = '';
        addToast(t('modals.social_links.added'), 'success');
    } catch (error: any) {
        console.error('Add social link failed', error);
        const msg = error?.response?.data?.error || t('modals.social_links.add_failed');
        addToast(msg, 'error');
    }
};

const confirmDelete = (link: any) => {
    deleteLink(link);
};

const deleteLink = async (link: any) => {
    if (isStreamerMode.value) return;
    try {
        const newLinks = links.value.filter((l) => l.platform !== link.platform);
        await updateSocialLinks(newLinks);
        addToast(t('modals.social_links.deleted'), 'success');
    } catch (error) {
        console.error('Delete social link failed', error);
        addToast(t('modals.social_links.delete_failed'), 'error');
    }
};

const startEdit = (link: any) => {
    if (isStreamerMode.value) return;
    editing.value = { ...link, originalPlatform: link.platform };
};

const cancelEdit = () => {
    editing.value = null;
};

const saveEdit = async () => {
    if (!editing.value.platform || !editing.value.url) {
        addToast(t('modals.social_links.fill_fields'), 'error');
        return;
    }
    // Prevent changing platform to one that already exists on another link
    if (
        editing.value.platform !== editing.value.originalPlatform &&
        links.value.some(l => l.platform === editing.value.platform)
    ) {
        addToast(t('modals.social_links.platform_exists') || t('modals.social_links.update_failed'), 'error');
        return;
    }
    try {
        const updatedLinks = links.value.map(l =>
            l.platform === editing.value.originalPlatform
                ? { platform: editing.value.platform, url: editing.value.url }
                : l
        );
        await updateSocialLinks(updatedLinks);
        editing.value = null;
        addToast(t('modals.social_links.updated'), 'success');
    } catch (error: any) {
        console.error('Update social link failed', error);
        const msg = error?.response?.data?.error || t('modals.social_links.update_failed');
        addToast(msg, 'error');
    }
};

const platformHref = (platform: string, handle: string) => {
    if (!handle) return '#';
    const h = handle.startsWith('@') ? handle.substring(1) : handle;
    switch (platform) {
        case 'GITHUB':
            return `https://github.com/${h}`;
        case 'TELEGRAM':
            return `https://t.me/${h}`;
        case 'YOUTUBE':
            if (handle.startsWith('@')) return `https://www.youtube.com/${h}`;
            return `https://www.youtube.com/@${h}`;
        default:
            return `#`;
    }
};

const displayHref = (platform: string, handle: string) => {
    if (!handle) return '';
    switch (platform) {
        case 'GITHUB':
            return `github.com/${handle.startsWith('@') ? handle.substring(1) : handle}`;
        case 'TELEGRAM':
            return `t.me/${handle.startsWith('@') ? handle.substring(1) : handle}`;
        case 'YOUTUBE':
            return handle;
        default:
            return handle;
    }
};

const displayHandle = (_platform: string, handle: string) => {
    if (!handle) return '';
    return handle.startsWith('@') ? handle : `@${handle}`;
};

onMounted(() => {
    loadLinks();
});
</script>

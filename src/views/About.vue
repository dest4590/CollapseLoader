<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core';
import { openUrl } from '@tauri-apps/plugin-opener';
import { onMounted, ref } from 'vue';
import { useToast } from '../services/toastService';
import { useI18n } from 'vue-i18n';
import { UpdateInfo, updaterService } from '../services/updaterService';
import Logo from '../assets/images/logo.svg';
import IconGitHub from '../assets/icons/github.svg';
import IconTelegram from '../assets/icons/telegram.svg';
import IconDiscord from '../assets/icons/discord.svg';
import { CircleFadingArrowUp } from 'lucide-vue-next';

const { t } = useI18n();
const LogoUrl = String(Logo);
const IconGitHubUrl = String(IconGitHub);
const IconTelegramUrl = String(IconTelegram);
const IconDiscordUrl = String(IconDiscord);
const { addToast } = useToast();

const version = ref('');
const codename = ref('');
const commitHash = ref('');
const branch = ref('');
const development = ref(false);
const isCheckingUpdates = ref(false);

const getVersion = async () => {
    try {
        const result = await invoke('get_version');
        const data = typeof result === 'string' ? JSON.parse(result) : result;
        version.value = data.version || '';
        codename.value = data.codename || '';
        commitHash.value = String(data.commitHash).slice(0, 7) || '';
        branch.value = data.branch || '';
        development.value = data.development === 'true' || false;
    } catch (error) {
        console.error('Failed to get version:', error);
    }
};

const openRepository = async () => {
    try {
        await openUrl('https://github.com/dest4590/CollapseLoader');
    } catch (error) {
        console.error('Failed to open repository:', error);
        addToast(t('about.open_failed', { platform: 'Github' }), 'error');
    }
};

const openTelegram = async () => {
    try {
        await openUrl('https://t.me/CollapseLoader');
    } catch (error) {
        console.error('Failed to open telegram:', error);
        addToast(t('about.open_failed', { platform: 'Telegram' }), 'error');
    }
};

const openDiscord = async () => {
    try {
        await openUrl('https://collapseloader.org/discord/');
    } catch (error) {
        console.error('Failed to open discord:', error);
        addToast(t('about.open_failed', { platform: 'Discord' }), 'error');
    }
};

const checkForUpdates = async () => {
    isCheckingUpdates.value = true;
    try {
        const updateInfo: UpdateInfo | null = await updaterService.checkForUpdates(false, t);
        if (!updateInfo?.available) {
            addToast(t('updater.no_update'), 'success');
        }
    } finally {
        isCheckingUpdates.value = false;
    }
};

onMounted(async () => {
    await getVersion();
});
</script>

<template>
    <div class="slide-up">
        <div class="flex flex-col items-center mb-4">
            <img :src="LogoUrl" alt="CollapseLoader Logo" class="w-36 h-36" />

            <div class="text-center">
                <h1 class="text-4xl font-bold mb-2">
                    CollapseLoader (GAMMA)
                </h1>
                <div class="tooltip tooltip-bottom hover:underline cursor-pointer" id="codename"
                    @click="openRepository">
                    <div class="tooltip-content flex flex-col">
                        <span class="text-sm font-semibold text-base-content">{{
                            codename
                            }}</span>
                        <span class="text-xs text-base-content/50">{{
                            commitHash
                            }}</span>
                    </div>
                    <p class="text-base-content/70">{{ version ? `v${version}` : '-' }}</p>
                </div>
            </div>
        </div>

        <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 max-w-xl mx-auto mb-5">
            <button type="button" @click="openRepository"
                class="social-link-btn btn btn-outline hover:bg-base-300 hover:text-primary border-base-content/20 h-auto py-4 flex flex-col items-center justify-center gap-2"
                aria-label="Open GitHub repository">
                <img :src="IconGitHubUrl" class="w-8 h-8 github-icon" alt="GitHub" />
                <span class="text-sm font-medium">GitHub</span>
            </button>

            <button type="button" @click="openTelegram"
                class="social-link-btn btn btn-outline hover:bg-base-300 hover:text-info border-base-content/20 h-auto py-4 flex flex-col items-center justify-center gap-2"
                aria-label="Open Telegram">
                <img :src="IconTelegramUrl" class="w-8 h-8 telegram-icon" alt="Telegram" />
                <span class="text-sm font-medium">Telegram</span>
            </button>

            <button type="button" @click="openDiscord"
                class="social-link-btn btn btn-outline hover:bg-base-300 hover:text-indigo-500 border-base-content/20 h-auto py-4 flex flex-col items-center justify-center gap-2"
                aria-label="Open Discord">
                <img :src="IconDiscordUrl" class="w-8 h-8 discord-icon" alt="Discord" />
                <span class="text-sm font-medium">Discord</span>
            </button>
        </div>

        <div class="flex justify-center max-w-xl mx-auto mb-5">
            <button @click="checkForUpdates" :disabled="isCheckingUpdates" class="btn btn-primary w-full">
                <span v-if="isCheckingUpdates" class="loading loading-spinner loading-sm"></span>
                <CircleFadingArrowUp class="w-5 h-5" v-if="!isCheckingUpdates" />
                {{ isCheckingUpdates ? t('updater.checking_updates') : t('updater.check_for_updates') }}
            </button>
        </div>

        <div class="bg-base-200 rounded-xl border border-base-300 p-6 max-w-xl mx-auto">
            <div class="flex flex-col gap-3">
                <h2 class="text-lg font-semibold text-primary-focus mb-2">
                    {{ t('about.title') }}
                </h2>
                <div class="flex justify-between items-center py-2 border-b border-base-300/50">
                    <span class="text-base-content/80">{{
                        t('about.version')
                        }}</span>
                    <span class="font-medium">{{ version }}</span>
                </div>

                <div class="flex justify-between items-center py-2 border-b border-base-300/50">
                    <span class="text-base-content/80">{{
                        t('about.codename')
                        }}</span>
                    <span class="font-medium">{{ codename }}</span>
                </div>

                <div class="flex justify-between items-center py-2 border-b border-base-300/50">
                    <span class="text-base-content/80">{{
                        t('about.commit')
                        }}</span>
                    <code class="bg-base-300 px-2 py-1 rounded text-xs">{{
                        commitHash
                    }}</code>
                </div>

                <div class="flex justify-between items-center py-2 border-b border-base-300/50">
                    <span class="text-base-content/80">{{ t('about.branch') }}</span>
                    <span class="font-medium"><code
                            class="bg-base-300 px-2 py-1 rounded text-xs">{{ branch }}</code></span>
                </div>

                <div class="flex justify-between items-center py-2 border-b border-base-300/50">
                    <span class="text-base-content/80">{{ t('about.development') }}</span>
                    <span class="font-medium"><code
                            class="bg-base-300 px-2 py-1 rounded text-xs">{{ development }}</code></span>
                </div>

                <p class="text-sm text-base-content/70 mt-4">
                    {{ t('about.description') }}
                </p>
            </div>
        </div>
    </div>
</template>

<style scoped>
.slide-up {
    animation: slideUp 0.6s ease-out forwards;
}

@keyframes slideUp {
    from {
        opacity: 0;
        transform: translateY(20px);
    }

    to {
        opacity: 1;
        transform: translateY(0);
    }
}

button {
    transition:
        transform 0.2s ease-out,
        box-shadow 0.2s ease-out;
}

button:hover {
    box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
}

.social-link-btn {
    position: relative;
    overflow: hidden;
    transition:
        color 0.2s ease-out,
        transform 0.3s ease;
    transform: scale(1);
}

.social-link-btn:hover {
    transform: scale(1.05);
}

.social-link-btn::after {
    content: '';
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    background: currentColor;
    opacity: 0.1;
    transform: scaleY(0);
    transform-origin: top;
    transition: transform 0.2s ease-out;
}

.social-link-btn:hover::after {
    transform: scaleY(1);
}

html[data-theme='dark'] .github-icon {
    filter: invert(100%) sepia(15%) saturate(1%) hue-rotate(282deg) brightness(102%) contrast(101%);
}

html[data-theme='light'] .github-icon {
    filter: invert(0%) sepia(15%) saturate(17%) hue-rotate(253deg) brightness(95%) contrast(103%);
}

.telegram-icon {
    filter: invert(60%) sepia(24%) saturate(3389%) hue-rotate(169deg) brightness(89%) contrast(94%);
}

.discord-icon {
    filter: invert(39%) sepia(99%) saturate(2691%) hue-rotate(226deg) brightness(97%) contrast(109%);
}
</style>
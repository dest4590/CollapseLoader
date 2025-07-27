import { invoke } from '@tauri-apps/api/core';
import { useModal } from './modalService';
import { useToast } from './toastService';
import UpdateModal from '../components/modals/UpdateModal.vue';

interface UpdateInfo {
    available: boolean;
    current_version: string;
    latest_version: string;
    download_url?: string;
    changelog?: string;
    release_notes?: string;
}

class UpdaterService {
    private checkInterval: number | null = null;
    private isChecking = false;
    private lastCheckTime = 0;
    private readonly CHECK_INTERVAL = 6 * 60 * 60 * 1000;
    private readonly MIN_CHECK_INTERVAL = 5 * 60 * 1000;

    async checkForUpdates(showNoUpdateToast = true, t: any): Promise<UpdateInfo | null> {
        if (this.isChecking) {
            return null;
        }

        const now = Date.now();
        if (now - this.lastCheckTime < this.MIN_CHECK_INTERVAL) {
            return null;
        }

        this.isChecking = true;
        this.lastCheckTime = now;

        try {
            console.log('Checking for updates...');
            const updateInfo = await invoke<UpdateInfo>('check_for_updates');
            console.log('Update check result:', updateInfo);

            if (updateInfo.available) {
                this.showUpdateNotification(updateInfo, t);
            } else if (showNoUpdateToast) {
                const { addToast } = useToast();
                addToast('updater.up_to_date', 'success');
            }

            return updateInfo;
        } catch (error) {
            console.error('Failed to check for updates:', error);
            if (showNoUpdateToast) {
                const { addToast } = useToast();
                addToast(`updater.check_failed|${error}`, 'error');
            }
            return null;
        } finally {
            this.isChecking = false;
        }
    }

    private showUpdateNotification(updateInfo: UpdateInfo, t: any): void {
        const { showModal } = useModal();

        showModal(
            'update-available',
            UpdateModal,
            {
                title: t('updater.update_available'),
                contentClass: "wide",
            },
            { updateInfo },
            {
                download: async () => {
                    await this.downloadAndInstallUpdate(updateInfo, t);
                },
                close: () => {
                    const { hideModal } = useModal();
                    hideModal('update-available');
                }
            }
        );
    }

    async downloadAndInstallUpdate(updateInfo: UpdateInfo, t: any): Promise<void> {
        const { addToast } = useToast();
        const { hideModal } = useModal();

        try {
            addToast(t('updater.starting_download'), 'info');

            await invoke('download_and_install_update', {
                downloadUrl: updateInfo.download_url
            });

            hideModal('update-available');
            addToast(t('updater.update_installed'), 'success');
        } catch (error) {
            console.error('Failed to download/install update:', error);
            addToast(`${t('updater.update_failed', { error })}`, 'error');
        }
    }

    startPeriodicCheck(t: any): void {
        if (this.checkInterval) {
            clearInterval(this.checkInterval);
        }

        setTimeout(() => {
            this.checkForUpdates(false, t);
        }, 10000);

        this.checkInterval = window.setInterval(() => {
            this.checkForUpdates(false, t);
        }, this.CHECK_INTERVAL);

        console.log('Started periodic update checking');
    }

    stopPeriodicCheck(): void {
        if (this.checkInterval) {
            clearInterval(this.checkInterval);
            this.checkInterval = null;
        }
        console.log('Stopped periodic update checking');
    }

    get isCheckingForUpdates(): boolean {
        return this.isChecking;
    }
}

export const updaterService = new UpdaterService();
export type { UpdateInfo };

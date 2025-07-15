import { invoke } from '@tauri-apps/api/core';
import { userService, type SyncData } from './userService';

export type ToastFunction = (message: string, type: string) => void;
export type TranslateFunction = (key: string, params?: Record<string, any>) => string;

interface SyncServiceState {
    isOnline: boolean;
    isSyncing: boolean;
    lastSyncTime: string | null;
    hasCloudData: boolean;
    autoSyncEnabled: boolean;
}

class SyncService {
    private state: SyncServiceState = {
        isOnline: navigator.onLine,
        isSyncing: false,
        lastSyncTime: null,
        hasCloudData: false,
        autoSyncEnabled: true
    };

    private listeners: ((state: SyncServiceState) => void)[] = [];
    private autoSyncInterval: number | null = null;

    constructor() {
        window.addEventListener('online', () => {
            this.state.isOnline = true;
            this.notifyListeners();
            if (this.state.autoSyncEnabled) {
                this.autoSync();
            }
        });

        window.addEventListener('offline', () => {
            this.state.isOnline = false;
            this.notifyListeners();
        });

        this.startAutoSync();
    }

    subscribe(listener: (state: SyncServiceState) => void) {
        this.listeners.push(listener);
        return () => {
            this.listeners = this.listeners.filter(l => l !== listener);
        };
    }

    private notifyListeners() {
        this.listeners.forEach(listener => listener({ ...this.state }));
    }

    private startAutoSync() {
        if (this.autoSyncInterval) return;

        this.autoSyncInterval = setInterval(() => {
            if (this.state.isOnline && this.state.autoSyncEnabled && !this.state.isSyncing) {
                this.autoSync();
            }
        }, 5 * 60 * 1000) as unknown as number;
    }

    private async autoSync() {
        try {
            await this.uploadToCloud();
        } catch (error) {
            console.warn('Auto-sync failed:', error);
        }
    }

    async initializeSyncStatus() {
        if (!this.state.isOnline) return;

        try {
            const status = await userService.getSyncStatus();
            this.state.lastSyncTime = status.last_sync_timestamp;
            this.state.hasCloudData = status.has_cloud_data;
            this.notifyListeners();
        } catch (error) {
            console.error('Failed to get sync status:', error);
        }
    }

    async checkAndRestoreOnStartup(): Promise<void> {
        if (!this.state.isOnline) {
            console.log('Offline - skipping startup sync check');
            return;
        }

        try {
            await this.initializeSyncStatus();

            if (!this.state.hasCloudData) {
                console.log('No cloud data available - skipping startup restore');
                return;
            }

            console.log('Checking for startup sync restoration...');

            const cloudData = await userService.downloadFromCloud();
            if (!cloudData) {
                console.log('No cloud data to restore');
                return;
            }

            let restored = false;

            if (cloudData.favorites_data && Array.isArray(cloudData.favorites_data) && cloudData.favorites_data.length > 0) {
                try {
                    const localFavorites = await invoke<number[]>('get_favorite_clients');

                    if (localFavorites.length === 0 ||
                        cloudData.favorites_data.some(fav => !localFavorites.includes(fav))) {

                        console.log('Restoring favorites from cloud sync');

                        for (const clientId of localFavorites) {
                            await invoke('remove_favorite_client', { clientId });
                        }
                        for (const clientId of cloudData.favorites_data) {
                            await invoke('add_favorite_client', { clientId });
                        }
                        restored = true;
                    }
                } catch (error) {
                    console.warn('Failed to restore favorites from cloud:', error);
                }
            }

            if (cloudData.accounts_data && Array.isArray(cloudData.accounts_data) && cloudData.accounts_data.length > 0) {
                try {
                    const localAccounts = await invoke<any[]>('get_accounts');
                    const localUsernames = new Set(localAccounts.map((acc: any) => acc.username));

                    for (const cloudAccount of cloudData.accounts_data) {
                        if (cloudAccount.username && !localUsernames.has(cloudAccount.username)) {
                            console.log(`Restoring account ${cloudAccount.username} from cloud sync`);
                            try {
                                await invoke('add_account', {
                                    username: cloudAccount.username,
                                    tags: cloudAccount.tags || ['cloud-sync']
                                });
                                restored = true;
                            } catch (error) {
                                console.warn('Failed to restore cloud account:', cloudAccount.username, error);
                            }
                        }
                    }
                } catch (error) {
                    console.warn('Failed to restore accounts from cloud:', error);
                }
            }

            if (restored) {
                console.log('Startup sync restoration completed');
            }

        } catch (error) {
            console.warn('Failed to check/restore synced data on startup:', error);
        }
    }

    async uploadToCloud(): Promise<boolean> {
        if (!this.state.isOnline || this.state.isSyncing) return false;

        this.state.isSyncing = true;
        this.notifyListeners();

        try {
            await new Promise(resolve => setTimeout(resolve, 1000));

            const [settings, favorites, accounts] = await Promise.all([
                invoke('get_settings'),
                invoke<number[]>('get_favorite_clients'),
                invoke<any[]>('get_accounts')
            ]);

            const syncData: SyncData = {
                settings_data: settings,
                favorites_data: favorites,
                accounts_data: accounts
            };

            const result = await userService.syncToCloud(syncData);

            this.state.lastSyncTime = result.last_sync_timestamp || new Date().toISOString();
            this.state.hasCloudData = true;

            return true;
        } catch (error) {
            console.error('Failed to upload to cloud:', error);
            throw error;
        } finally {
            this.state.isSyncing = false;
            this.notifyListeners();
        }
    }

    async downloadFromCloud(): Promise<boolean> {
        if (!this.state.isOnline || this.state.isSyncing) return false;

        this.state.isSyncing = true;
        this.notifyListeners();

        try {
            await new Promise(resolve => setTimeout(resolve, 1000));

            const cloudData = await userService.downloadFromCloud();

            if (!cloudData) return false;

            if (cloudData.settings_data && Object.keys(cloudData.settings_data).length > 0) {
                await invoke('save_settings', { input_settings: cloudData.settings_data });
            }

            if (cloudData.favorites_data && Array.isArray(cloudData.favorites_data)) {
                const currentFavorites = await invoke<number[]>('get_favorite_clients');
                for (const clientId of currentFavorites) {
                    await invoke('remove_favorite_client', { clientId });
                }
                for (const clientId of cloudData.favorites_data) {
                    await invoke('add_favorite_client', { clientId });
                }
            }

            if (cloudData.accounts_data && Array.isArray(cloudData.accounts_data)) {
                try {
                    const localAccounts = await invoke<any[]>('get_accounts');
                    const localUsernames = new Set(localAccounts.map((acc: any) => acc.username));

                    for (const cloudAccount of cloudData.accounts_data) {
                        if (cloudAccount.username && !localUsernames.has(cloudAccount.username)) {
                            try {
                                await invoke('add_account', {
                                    username: cloudAccount.username,
                                    tags: cloudAccount.tags || ['cloud-sync']
                                });
                            } catch (error) {
                                console.warn('Failed to add cloud account:', cloudAccount.username, error);
                            }
                        }
                    }
                } catch (error) {
                    console.warn('Failed to sync accounts from cloud:', error);
                }
            }

            this.state.lastSyncTime = cloudData.last_sync_timestamp || new Date().toISOString();
            this.state.hasCloudData = true;

            return true;
        } catch (error) {
            console.error('Failed to download from cloud:', error);
            throw error;
        } finally {
            this.state.isSyncing = false;
            this.notifyListeners();
        }
    }

    async manualSync(
        addToast: ToastFunction,
        t: TranslateFunction
    ): Promise<void> {
        if (typeof addToast !== 'function') {
            console.error('addToast is not a function:', addToast);
            return;
        }

        if (typeof t !== 'function') {
            console.error('t (translation function) is not a function:', t);
            return;
        }

        if (!this.state.isOnline) {
            addToast(t('toast.sync.offline_error'), 'error');
            return;
        }

        if (this.state.isSyncing) {
            addToast(t('toast.sync.already_in_progress'), 'info');
            return;
        }

        try {
            addToast(t('toast.sync.syncing'), 'info');
            await this.uploadToCloud();
            addToast(t('toast.sync.success'), 'success');
        } catch (error) {
            addToast(t('toast.sync.failed', { error }), 'error');
        }
    }

    getState(): SyncServiceState {
        return { ...this.state };
    }

    setAutoSyncEnabled(enabled: boolean) {
        this.state.autoSyncEnabled = enabled;
        this.notifyListeners();
    }

    destroy() {
        if (this.autoSyncInterval) {
            clearInterval(this.autoSyncInterval);
            this.autoSyncInterval = null;
        }
    }
}

export const syncService = new SyncService();
export type { SyncServiceState };
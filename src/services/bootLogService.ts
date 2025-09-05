import { ref } from 'vue';

export interface BootLogEntry {
    id: number;
    timestamp: string;
    level: 'OK' | 'INFO' | 'WARN' | 'ERROR';
    service: string;
    message: string;
}

class BootLogService {
    private logs = ref<BootLogEntry[]>([]);
    private logIdCounter = 0;
    private startTime = Date.now();

    private getTimestamp(): string {
        const elapsed = Date.now() - this.startTime;
        const seconds = Math.floor(elapsed / 1000);
        const milliseconds = elapsed % 1000;
        return `${seconds.toString().padStart(2, '0')}:${(milliseconds / 10).toFixed(0).padStart(2, '0')}.${(milliseconds % 10).toString()}`;
    }

    private addLog(level: BootLogEntry['level'], service: string, message: string) {
        this.logs.value.push({
            id: ++this.logIdCounter,
            timestamp: this.getTimestamp(),
            level,
            service,
            message
        });
    }

    start() {
        this.addLog('OK', 'init', 'CollapseLoader starting...');
    }

    systemInit() {
        this.addLog('INFO', 'init', 'Initializing system components');
    }

    themeApplied(theme: string) {
        this.addLog('OK', 'theme', `Theme applied: ${theme} [OK]`);
    }

    languageApplied(language: string) {
        this.addLog('OK', 'i18n', `Language system initialized: ${language} [OK]`);
    }

    eventListenersInit() {
        this.addLog('INFO', 'events', 'Registering event listeners...');
    }

    eventListenersReady() {
        this.addLog('OK', 'events', 'Event listeners registered [OK]');
    }

    serverConnectivityCheck() {
        this.addLog('INFO', 'network', 'Checking server connectivity...');
    }

    cdnOnline() {
        this.addLog('OK', 'cdn', 'CDN connection established [OK]');
    }

    cdnOffline() {
        this.addLog('ERROR', 'cdn', 'CDN connection failed [ERROR]');
    }

    webApiOnline() {
        this.addLog('OK', 'webapi', 'Web API connection verified [OK]');
    }

    webApiOffline() {
        this.addLog('ERROR', 'webapi', 'Web API connection failed [ERROR]');
    }

    apiInit() {
        this.addLog('INFO', 'api', 'Initializing API bridge...');
    }

    apiInitSuccess() {
        this.addLog('OK', 'tauri', 'Tauri API bridge initialized [OK]');
    }

    apiInitFailed() {
        this.addLog('ERROR', 'tauri', 'Tauri API bridge failed [ERROR]');
    }

    authCheck() {
        this.addLog('INFO', 'auth', 'Checking authentication status...');
    }

    authSuccess() {
        this.addLog('OK', 'auth', 'User authenticated [OK]');
    }

    authSkipped() {
        this.addLog('INFO', 'auth', 'No authentication token found');
    }

    userDataInit() {
        this.addLog('INFO', 'user', 'Loading user data...');
    }

    userDataSuccess() {
        this.addLog('OK', 'user', 'User data structures ready [OK]');
    }

    userDataFailed() {
        this.addLog('WARN', 'user', 'Failed to load user data [WARN]');
    }

    syncInit() {
        this.addLog('INFO', 'sync', 'Initializing synchronization service...');
    }

    syncReady() {
        this.addLog('OK', 'sync', 'Synchronization service loaded [OK]');
    }

    toastSystemReady() {
        this.addLog('OK', 'toast', 'Notification system active [OK]');
    }

    flagsLoaded() {
        this.addLog('OK', 'config', 'Configuration flags loaded [OK]');
    }

    flagsLoadFailed() {
        this.addLog('ERROR', 'config', 'Failed to load configuration flags [ERROR]');
    }

    systemReady() {
        this.addLog('INFO', 'system', 'All systems operational');
    }

    clear() {
        this.logs.value = [];
        this.logIdCounter = 0;
        this.startTime = Date.now();
    }

    getLogs() {
        return this.logs;
    }

    addCustomLog(level: BootLogEntry['level'], service: string, message: string) {
        this.addLog(level, service, message);
    }
}

export const bootLogService = new BootLogService();
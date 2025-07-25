export interface PluginMetadata {
    id: string;
    name: string;
    version: string;
    author: string;
    description: string;
    icon?: string;
    website?: string;
    configSchema?: Record<string, PluginConfigFieldSchema>;
}

export interface PluginConfigFieldSchema {
    type: 'string' | 'number' | 'boolean' | 'select' | 'range' | 'color';
    label: string;
    description?: string;
    default?: any;
    required?: boolean;
    min?: number;
    max?: number;
    step?: number;
    options?: Array<{ label: string; value: any }>;
}

export interface PluginConfig {
    [key: string]: any;
}

export interface PluginError {
    type: PluginErrorType;
    message: string;
    pluginId?: string;
    details?: any;
}

export enum PluginErrorType {
    SYNTAX_ERROR = 'syntax_error',
    RUNTIME_ERROR = 'runtime_error',
    VALIDATION_ERROR = 'validation_error',
    NETWORK_ERROR = 'network_error',
    PERMISSION_ERROR = 'permission_error'
}

export interface PluginState {
    id: string;
    enabled: boolean;
    metadata: PluginMetadata;
    config: PluginConfig;
    code: string;
    installedAt: string;
    errors: PluginError[];
    instance?: PluginInstance;
}

export interface ParsedPlugin {
    metadata: PluginMetadata;
    code: string;
    isValid: boolean;
    errors: string[];
}

export interface PluginInstance {
    onEnable?(context: PluginContext): Promise<void> | void;
    onDisable?(): Promise<void> | void;
    onUpdate?(context: PluginContext): Promise<void> | void;
    onConfigChange?(config: PluginConfig): Promise<void> | void;
}

export interface PluginAPI {
    invoke(command: string, args?: any): Promise<any>;
    addToast(message: string, type: 'info' | 'success' | 'warning' | 'error', duration?: number): void;
    getClients(): Promise<any[]>;
    getConfig(): PluginConfig;
    updateConfig(config: Partial<PluginConfig>): void;
    subscribe(event: string, callback: (data: any) => void): () => void;
    emit(event: string, data: any): void;
    getAppSettings(): Promise<any>;
    showModal?(modalId: string, component: any, props?: any, events?: any): void;
    hideModal?(modalId: string): void;
    getUserAccount?(): Promise<any>;
    getFavoriteClients?(): Promise<number[]>;
    addFavoriteClient?(clientId: number): Promise<void>;
    removeFavoriteClient?(clientId: number): Promise<void>;
    dom: {
        createElement(tag: string, className?: string): HTMLElement;
        addToBody(element: HTMLElement): void;
        removeFromBody(element: HTMLElement): void;
        querySelector(selector: string): HTMLElement | null;
        addStyles(css: string): void;
        removeStyles?(): void;
    };
    i18n: {
        getCurrentLanguage(): string;
        translate?(key: string, params?: any): string;
    };
}

export interface PluginContext {
    api: PluginAPI;
    metadata: PluginMetadata;
}

export interface PluginFileInfo {
    id: string;
    name: string;
    enabled: boolean;
    installedAt: string;
    size: number;
}

export interface PluginManifest {
    plugins: PluginFileInfo[];
    version: string;
    lastModified: string;
}
import { invoke } from '@tauri-apps/api/core';
import { reactive } from 'vue';
import { useToast } from './toastService';
import { useModal } from './modalService';
import {
    PluginState,
    PluginMetadata,
    ParsedPlugin,
    PluginConfig,
    PluginConfigFieldSchema,
    PluginError,
    PluginErrorType,
    PluginInstance,
    PluginAPI,
    PluginContext,
    PluginManifest
} from '../types/plugin';

interface PluginData {
    metadata: PluginMetadata;
    code: string;
    config: any;
    enabled: boolean;
    installed_at: string;
}

class PluginService {
    private plugins = reactive<PluginState[]>([]);
    private subscribers: ((states: PluginState[]) => void)[] = [];
    private eventListeners = new Map<string, ((data: any) => void)[]>();

    constructor() {
        this.loadPluginsFromFileSystem();
    }

    private async loadPluginsFromFileSystem(): Promise<void> {
        try {
            const manifest = await invoke<PluginManifest>('get_plugins_manifest');
            const pluginStates: PluginState[] = [];

            for (const pluginInfo of manifest.plugins) {
                try {
                    const pluginData = await invoke<PluginData>('get_plugin_data', {
                        pluginId: pluginInfo.id
                    });

                    const pluginState: PluginState = {
                        id: pluginData.metadata.id,
                        enabled: pluginData.enabled,
                        metadata: pluginData.metadata,
                        config: pluginData.config || {},
                        code: pluginData.code,
                        installedAt: pluginData.installed_at,
                        errors: []
                    };

                    pluginStates.push(pluginState);

                    if (pluginData.enabled) {
                        await this.enablePluginInstance(pluginState);
                    }
                } catch (error) {
                    console.error(`Failed to load plugin ${pluginInfo.id}:`, error);
                }
            }

            this.plugins.splice(0, this.plugins.length, ...pluginStates);
            this.notify();
        } catch (error) {
            console.error('Failed to load plugins from file system:', error);
        }
    }

    private async savePluginToFileSystem(plugin: PluginState): Promise<void> {
        const pluginData: PluginData = {
            metadata: plugin.metadata,
            code: plugin.code,
            config: plugin.config,
            enabled: plugin.enabled,
            installed_at: plugin.installedAt
        };

        await invoke('save_plugin_data', { pluginData });
    }

    private notify(): void {
        this.subscribers.forEach(callback => {
            try {
                callback([...this.plugins]);
            } catch (error) {
                console.error('Plugin service notification error:', error);
            }
        });
    }

    private createError(type: PluginErrorType, message: string, pluginId?: string): PluginError {
        return {
            type,
            message,
            pluginId,
            details: undefined
        };
    }

    getPluginStates(): PluginState[] {
        return [...this.plugins];
    }

    subscribe(callback: (states: PluginState[]) => void): () => void {
        this.subscribers.push(callback);
        callback([...this.plugins]);

        return () => {
            const index = this.subscribers.indexOf(callback);
            if (index > -1) {
                this.subscribers.splice(index, 1);
            }
        };
    }

    parsePluginFromText(text: string): ParsedPlugin {
        const errors: string[] = [];
        let metadata: Partial<PluginMetadata> = {};

        try {
            const metadataMatch = text.match(/\/\*\*([\s\S]*?)\*\//);
            if (metadataMatch) {
                const metadataText = metadataMatch[1];
                const fields = [
                    { key: 'name', regex: /@name\s+(.+)/ },
                    { key: 'version', regex: /@version\s+(.+)/ },
                    { key: 'author', regex: /@author\s+(.+)/ },
                    { key: 'description', regex: /@description\s+(.+)/ },
                    { key: 'icon', regex: /@icon\s+(.+)/ },
                    { key: 'website', regex: /@website\s+(.+)/ }
                ];

                fields.forEach(field => {
                    const match = metadataText.match(field.regex);
                    if (match) {
                        (metadata as any)[field.key] = match[1].trim();
                    }
                });
            }

            if (!metadata.name) errors.push('Plugin name is required');

            if (!metadata.version || !/^\d+\.\d+\.\d+$/.test(metadata.version)) {
                errors.push('Valid plugin version (x.y.z) is required');
            }

            if (!metadata.author) errors.push('Plugin author is required');

            if (!metadata.description) errors.push('Plugin description is required');

            if (!metadata.id && metadata.name) {
                metadata.id = metadata.name.toLowerCase().replace(/[^a-z0-9]/g, '-');
            }

            if (metadata.id && this.plugins.some(p => p.id === metadata.id)) {
                errors.push('A plugin with this ID already exists');
            }

        } catch (error) {
            errors.push('Failed to parse plugin metadata');
        }

        return {
            metadata: metadata as PluginMetadata,
            code: text,
            isValid: errors.length === 0 && !!metadata.name && !!metadata.version && !!metadata.author,
            errors
        };
    }

    async addPlugin(parsedPlugin: ParsedPlugin): Promise<boolean> {
        if (!parsedPlugin.isValid) {
            return false;
        }

        const newPlugin: PluginState = {
            id: parsedPlugin.metadata.id,
            enabled: false,
            metadata: parsedPlugin.metadata,
            config: this.getDefaultConfig(parsedPlugin.metadata.configSchema),
            code: parsedPlugin.code,
            installedAt: new Date().toISOString(),
            errors: []
        };

        await this.savePluginToFileSystem(newPlugin);

        this.plugins.push(newPlugin);
        this.notify();
        return true;
    }

    async removePlugin(id: string): Promise<boolean> {
        const index = this.plugins.findIndex(p => p.id === id);
        if (index === -1) return false;

        const plugin = this.plugins[index];
        if (plugin.enabled) {
            await this.disablePlugin(id);
        }

        await invoke('delete_plugin', { pluginId: id });

        this.plugins.splice(index, 1);
        this.notify();
        return true;
    }

    async togglePlugin(id: string): Promise<boolean> {
        const plugin = this.plugins.find(p => p.id === id);
        if (!plugin) return false;

        return plugin.enabled ? await this.disablePlugin(id) : await this.enablePlugin(id);
    }

    private createPluginAPI(plugin: PluginState): PluginAPI {
        const { addToast } = useToast();
        const { showModal, hideModal } = useModal();

        return {
            invoke: async (command: string, args?: any) => {
                try {
                    return await invoke(command, args);
                } catch (error) {
                    console.error(`Plugin ${plugin.id} invoke error:`, error);
                    throw error;
                }
            },
            addToast: (message: string, type: 'info' | 'success' | 'warning' | 'error', duration?: number) => {
                addToast(`[${plugin.metadata.name}] ${message}`, type, duration);
            },
            getClients: async () => {
                try {
                    return await invoke('get_clients');
                } catch (error) {
                    console.error('Plugin failed to get clients:', error);
                    return [];
                }
            },
            getConfig: () => ({ ...plugin.config }),
            updateConfig: (config: Partial<PluginConfig>) => {
                this.updatePluginConfig(plugin.id, config);
            },
            subscribe: (event: string, callback: (data: any) => void) => {
                if (!this.eventListeners.has(event)) {
                    this.eventListeners.set(event, []);
                }
                this.eventListeners.get(event)!.push(callback);

                return () => {
                    const listeners = this.eventListeners.get(event);
                    if (listeners) {
                        const index = listeners.indexOf(callback);
                        if (index > -1) {
                            listeners.splice(index, 1);
                        }
                    }
                };
            },
            emit: (event: string, data: any) => {
                const listeners = this.eventListeners.get(event);
                if (listeners) {
                    listeners.forEach(callback => {
                        try {
                            callback(data);
                        } catch (error) {
                            console.error('Plugin event callback error:', error);
                        }
                    });
                }
            },
            getAppSettings: async () => {
                try {
                    return await invoke('get_settings');
                } catch (error) {
                    console.error('Plugin failed to get app settings:', error);
                    return {};
                }
            },
            showModal: (modalId: string, component: any, props?: any, events?: any) => {
                showModal(`plugin-${plugin.id}-${modalId}`, component, props, events);
            },
            hideModal: (modalId: string) => {
                hideModal(`plugin-${plugin.id}-${modalId}`);
            },
            getUserAccount: async () => {
                try {
                    return await invoke('get_active_account');
                } catch (error) {
                    console.error('Plugin failed to get user account:', error);
                    return null;
                }
            },
            getFavoriteClients: async () => {
                try {
                    return await invoke('get_favorite_clients');
                } catch (error) {
                    console.error('Plugin failed to get favorite clients:', error);
                    return [];
                }
            },
            addFavoriteClient: async (clientId: number) => {
                try {
                    await invoke('add_favorite_client', { clientId });
                } catch (error) {
                    console.error('Plugin failed to add favorite client:', error);
                    throw error;
                }
            },
            removeFavoriteClient: async (clientId: number) => {
                try {
                    await invoke('remove_favorite_client', { clientId });
                } catch (error) {
                    console.error('Plugin failed to remove favorite client:', error);
                    throw error;
                }
            },
            dom: {
                createElement: (tag: string, className?: string): HTMLElement => {
                    const element = document.createElement(tag);
                    if (className) {
                        element.className = className;
                    }
                    element.setAttribute('data-plugin', plugin.id);
                    return element;
                },
                addToBody: (element: HTMLElement) => {
                    if (!element.hasAttribute('data-plugin')) {
                        element.setAttribute('data-plugin', plugin.id);
                    }
                    document.body.appendChild(element);
                },
                removeFromBody: (element: HTMLElement) => {
                    if (element.parentNode === document.body) {
                        document.body.removeChild(element);
                    }
                },
                querySelector: (selector: string) => {
                    return document.querySelector(selector) as HTMLElement | null;
                },
                addStyles: (css: string) => {
                    const scopedCSS = css.replace(/(\.|#)?([a-zA-Z][\w-]*)/g, (match, prefix, _) => {
                        if (prefix === '#' || prefix === '.') {
                            return match;
                        }
                        return `[data-plugin="${plugin.id}"] ${match}`;
                    });

                    const existingStyles = document.querySelectorAll(`style[data-plugin="${plugin.id}"]`);
                    existingStyles.forEach(style => style.remove());

                    const style = document.createElement('style');
                    style.setAttribute('data-plugin', plugin.id);
                    style.textContent = scopedCSS;
                    document.head.appendChild(style);
                },
                removeStyles: () => {
                    const styles = document.querySelectorAll(`style[data-plugin="${plugin.id}"]`);
                    styles.forEach(style => style.remove());
                }
            },
            i18n: {
                getCurrentLanguage: () => {
                    return localStorage.getItem('user-locale') || 'en';
                }
            }
        };
    }

    private async executePluginCode(plugin: PluginState): Promise<PluginInstance | null> {
        try {
            console.log(`Executing plugin code for: ${plugin.metadata.name}`);

            const context: PluginContext = {
                api: this.createPluginAPI(plugin),
                metadata: plugin.metadata
            };

            const pluginFunction = new Function('context', `
                try {
                    var module = { exports: {} };
                    var exports = module.exports;
                    (function(context, module, exports) {
                        ${plugin.code}
                    })(context, module, exports);
                    let PluginClass = null;
                    if (typeof module.exports === 'function') {
                        PluginClass = module.exports;
                    } else if (typeof module.exports.default === 'function') {
                        PluginClass = module.exports.default;
                    }
                    if (PluginClass) {
                        const instance = new PluginClass();
                        return instance;
                    } else {
                        return null;
                    }
                } catch (error) {
                    console.error('Error in plugin execution wrapper:', error);
                    throw error;
                }
            `);

            const instance = pluginFunction(context) as PluginInstance;
            console.log('Plugin execution result:', instance);
            return instance;
        } catch (error) {
            console.error('Plugin execution error:', error);
            throw error;
        }
    }

    async enablePlugin(id: string): Promise<boolean> {
        const plugin = this.plugins.find(p => p.id === id);
        if (!plugin || plugin.enabled) return false;

        const success = await this.enablePluginInstance(plugin);
        if (success) {
            plugin.enabled = true;
            await invoke('update_plugin_enabled_status', {
                pluginId: id,
                enabled: true
            });
            this.notify();
        }
        return success;
    }

    async disablePlugin(id: string): Promise<boolean> {
        const plugin = this.plugins.find(p => p.id === id);
        if (!plugin || !plugin.enabled) return false;

        try {
            if (plugin.instance?.onDisable) {
                await plugin.instance.onDisable();
            }

            const pluginElements = document.querySelectorAll(`[data-plugin="${id}"]`);
            pluginElements.forEach(element => {
                element.remove();
            });

            this.eventListeners.forEach((listeners, event) => {
                this.eventListeners.set(event, listeners.filter(_ => {
                    return true;
                }));
            });

            plugin.enabled = false;
            plugin.instance = undefined;

            await invoke('update_plugin_enabled_status', {
                pluginId: id,
                enabled: false
            });

            this.notify();
            return true;
        } catch (error) {
            console.error('Error disabling plugin:', error);
            plugin.enabled = false;
            plugin.instance = undefined;
            this.notify();
            return false;
        }
    }

    async updatePluginCode(id: string, code: string): Promise<boolean> {
        const plugin = this.plugins.find(p => p.id === id);
        if (!plugin) return false;

        try {
            plugin.code = code;

            await invoke('save_plugin_code', { pluginId: id, code });

            if (plugin.enabled) {
                await this.disablePlugin(id);
                await this.enablePlugin(id);
            }

            this.notify();
            return true;
        } catch (error) {
            console.error('Failed to update plugin code:', error);
            return false;
        }
    }

    async getPluginCode(id: string): Promise<string | null> {
        try {
            return await invoke<string>('get_plugin_code', { pluginId: id });
        } catch (error) {
            console.error('Failed to get plugin code:', error);
            return null;
        }
    }

    private async enablePluginInstance(plugin: PluginState): Promise<boolean> {
        console.log(`Enabling plugin: ${plugin.metadata.name}`);

        try {
            const instance = await this.executePluginCode(plugin);
            console.log('Plugin instance from execution:', instance);

            if (!instance) {
                throw new Error('Failed to create plugin instance');
            }

            plugin.instance = instance;

            if (instance.onEnable) {
                console.log('Calling onEnable...');
                await instance.onEnable({
                    api: this.createPluginAPI(plugin),
                    metadata: plugin.metadata
                });
                console.log('onEnable completed');
            }

            plugin.errors = [];
            console.log(`Plugin ${plugin.metadata.name} enabled successfully`);
            return true;
        } catch (error) {
            console.error('Error enabling plugin:', error);
            const errorMsg = error instanceof Error ? error.message : String(error);
            plugin.errors = [this.createError(PluginErrorType.RUNTIME_ERROR, errorMsg, plugin.id)];
            return false;
        }
    }

    async updatePluginConfig(id: string, config: Partial<PluginConfig>): Promise<boolean> {
        const plugin = this.plugins.find(p => p.id === id);
        if (!plugin) return false;

        try {
            const validatedConfig = this.validateConfig(plugin.metadata.configSchema, config);
            plugin.config = { ...plugin.config, ...validatedConfig };
            await this.savePluginToFileSystem(plugin);
            this.notify();
            return true;
        } catch (error) {
            const errorMsg = error instanceof Error ? error.message : String(error);
            plugin.errors = [this.createError(PluginErrorType.VALIDATION_ERROR, errorMsg, plugin.id)];
            this.notify();
            return false;
        }
    }

    private getDefaultConfig(schema?: Record<string, PluginConfigFieldSchema>): PluginConfig {
        if (!schema) return {};

        const defaultConfig: PluginConfig = {};
        Object.entries(schema).forEach(([key, field]) => {
            if (field.default !== undefined) {
                defaultConfig[key] = field.default;
            }
        });
        return defaultConfig;
    }

    private validateConfig(
        schema: Record<string, PluginConfigFieldSchema> | undefined,
        config: Partial<PluginConfig>
    ): PluginConfig {
        if (!schema) return config as PluginConfig;

        const validated: PluginConfig = {};

        Object.entries(schema).forEach(([key, field]) => {
            const value = config[key] !== undefined ? config[key] : field.default;

            if (field.required && value === undefined) {
                throw new Error(`Configuration field ${key} is required`);
            }

            if (value !== undefined) {
                switch (field.type) {
                    case 'number':
                        if (typeof value !== 'number') {
                            throw new Error(`Field ${key} must be a number`);
                        }
                        break;
                    case 'boolean':
                        if (typeof value !== 'boolean') {
                            throw new Error(`Field ${key} must be a boolean`);
                        }
                        break;
                    case 'string':
                    case 'color':
                        if (typeof value !== 'string') {
                            throw new Error(`Field ${key} must be a string`);
                        }
                        break;
                    case 'select':
                        if (field.options && !field.options.some(opt => opt.value === value)) {
                            throw new Error(`Field ${key} has invalid option`);
                        }
                        break;
                    case 'range':
                        if (typeof value !== 'number' ||
                            (field.min !== undefined && value < field.min) ||
                            (field.max !== undefined && value > field.max)) {
                            throw new Error(`Field ${key} out of range`);
                        }
                        break;
                }
                validated[key] = value;
            }
        });

        return validated;
    }

    async importPluginFromFile(file: File): Promise<boolean> {
        try {
            const text = await file.text();
            const parsedPlugin = this.parsePluginFromText(text);
            return await this.addPlugin(parsedPlugin);
        } catch (error) {
            console.error('Failed to import plugin from file:', error);
            return false;
        }
    }

    destroy(): void {
        this.subscribers.length = 0;
        this.plugins.forEach(plugin => {
            plugin.enabled = false;
            plugin.instance = undefined;
        });
    }
}

let pluginServiceInstance: PluginService | null = null;

export function getPluginService(): PluginService {
    if (!pluginServiceInstance) {
        pluginServiceInstance = new PluginService();
    }
    return pluginServiceInstance;
}

export function createPluginService(): PluginService {
    return getPluginService();
}
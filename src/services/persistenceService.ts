import { readFile, writeFile, mkdir } from "@tauri-apps/plugin-fs";
import { join } from "@tauri-apps/api/path";
import { invoke } from "@tauri-apps/api/core";

const PERSISTENCE_FILE = "aci.json";
const STORAGE_KEYS = [
    "local_profiles",
    "active_local_profile",
    "local_achievements",
    "local_user_stats",
    "local_unique_favorites",
    "userData"
];

class PersistenceService {
    private isInitialized = false;

    async init() {
        if (this.isInitialized) return;

        try {
            const dataDir = await invoke<string>("get_data_folder");
            const filePath = await join(dataDir, PERSISTENCE_FILE);

            try {
                const content = await readFile(filePath);
                const decoder = new TextDecoder();
                const json = JSON.parse(decoder.decode(content));

                console.log("Loading local data from aci.json...");

                for (const key of STORAGE_KEYS) {
                    if (json[key]) {
                        localStorage.setItem(key, typeof json[key] === 'string' ? json[key] : JSON.stringify(json[key]));
                    }
                }
                console.log("Local data restored successfully.");
            } catch (e) {
                console.log("aci.json not found or empty, starting fresh.");
                await this.saveToDisk();
            }

            this.isInitialized = true;
            this.setupWatchers();
        } catch (error) {
            console.error("Persistence initialization failed:", error);
        }
    }

    private setupWatchers() {
        const originalSetItem = localStorage.setItem;
        const self = this;

        localStorage.setItem = function (key: string, value: string) {
            originalSetItem.apply(this, [key, value]);
            if (STORAGE_KEYS.includes(key)) {
                self.saveToDisk();
            }
        };

        const originalRemoveItem = localStorage.removeItem;
        localStorage.removeItem = function (key: string) {
            originalRemoveItem.apply(this, [key]);
            if (STORAGE_KEYS.includes(key)) {
                self.saveToDisk();
            }
        };

        window.addEventListener('storage', (event) => {
            if (event.key && STORAGE_KEYS.includes(event.key)) {
                this.saveToDisk();
            }
        });
    }

    async saveToDisk() {
        try {
            const data: Record<string, any> = {};
            for (const key of STORAGE_KEYS) {
                const val = localStorage.getItem(key);
                if (val) {
                    try {
                        data[key] = JSON.parse(val);
                    } catch {
                        data[key] = val;
                    }
                }
            }

            const dataDir = await invoke<string>("get_data_folder");
            try {
                await mkdir(dataDir, { recursive: true });
            } catch { }

            const filePath = await join(dataDir, PERSISTENCE_FILE);
            const encoder = new TextEncoder();
            await writeFile(filePath, encoder.encode(JSON.stringify(data, null, 4)));
        } catch (error) {
            console.error("Failed to save data to disk:", error);
        }
    }
}

export const persistenceService = new PersistenceService();

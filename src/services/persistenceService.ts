import { readFile, writeFile, mkdir, BaseDirectory } from "@tauri-apps/plugin-fs";
import { appDataDir, join } from "@tauri-apps/api/path";

const PERSISTENCE_FILE = "aci.json";
const STORAGE_KEYS = [
    "local_profiles",
    "active_local_profile",
    "local_achievements",
    "local_user_stats",
    "local_unique_favorites",
    "userData" // Кэш текущего пользователя для быстрого входа
];

class PersistenceService {
    private isInitialized = false;

    async init() {
        if (this.isInitialized) return;
        
        try {
            const dataDir = await appDataDir();
            const filePath = await join(dataDir, PERSISTENCE_FILE);
            
            // Пытаемся прочитать файл
            try {
                const content = await readFile(filePath);
                const decoder = new TextDecoder();
                const json = JSON.parse(decoder.decode(content));
                
                console.log("Loading local data from aci.json...");
                
                // Загружаем данные в localStorage
                for (const key of STORAGE_KEYS) {
                    if (json[key]) {
                        localStorage.setItem(key, typeof json[key] === 'string' ? json[key] : JSON.stringify(json[key]));
                    }
                }
                console.log("Local data restored successfully.");
            } catch (e) {
                console.log("aci.json not found or empty, starting fresh.");
                // Если файла нет, создаем его из текущего localStorage
                await this.saveToDisk();
            }
            
            this.isInitialized = true;
            this.setupWatchers();
        } catch (error) {
            console.error("Persistence initialization failed:", error);
        }
    }

    private setupWatchers() {
        // Следим за изменениями в localStorage в текущем окне
        const originalSetItem = localStorage.setItem;
        const self = this;
        
        localStorage.setItem = function(key: string, value: string) {
            originalSetItem.apply(this, [key, value]);
            if (STORAGE_KEYS.includes(key)) {
                self.saveToDisk();
            }
        };

        const originalRemoveItem = localStorage.removeItem;
        localStorage.removeItem = function(key: string) {
            originalRemoveItem.apply(this, [key]);
            if (STORAGE_KEYS.includes(key)) {
                self.saveToDisk();
            }
        };

        // Также слушаем событие storage (для других вкладок/окон, если будут)
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

            const dataDir = await appDataDir();
            // Убеждаемся, что папка существует
            try {
                await mkdir(dataDir, { recursive: true });
            } catch {}

            const filePath = await join(dataDir, PERSISTENCE_FILE);
            const encoder = new TextEncoder();
            await writeFile(filePath, encoder.encode(JSON.stringify(data, null, 4)));
            // console.log("Persistent data saved to disk.");
        } catch (error) {
            console.error("Failed to save data to disk:", error);
        }
    }
}

export const persistenceService = new PersistenceService();

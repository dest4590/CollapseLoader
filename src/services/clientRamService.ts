import { invoke } from "@tauri-apps/api/core";
import type { ClientRamUsage } from "@shared/types/ui";

class ClientRamService {
    async getClientRamUsage(clientId: number): Promise<ClientRamUsage> {
        return invoke<ClientRamUsage>("get_client_ram_usage", {
            id: clientId,
        });
    }

    formatBytes(bytes: number): string {
        if (!Number.isFinite(bytes) || bytes <= 0) {
            return "0 B";
        }

        const units = ["B", "KB", "MB", "GB", "TB"];
        let value = bytes;
        let unitIndex = 0;

        while (value >= 1024 && unitIndex < units.length - 1) {
            value /= 1024;
            unitIndex += 1;
        }

        const digits = unitIndex === 0 || value >= 100 ? 0 : value >= 10 ? 1 : 2;
        return `${value.toFixed(digits)} ${units[unitIndex]}`;
    }

    formatMiB(value: number): string {
        if (!Number.isFinite(value) || value <= 0) {
            return "0 MiB";
        }

        return `${value.toFixed(value >= 10 ? 1 : 2)} MiB`;
    }

    formatPercent(value: number): string {
        if (!Number.isFinite(value) || value <= 0) {
            return "0%";
        }

        return `${value.toFixed(value >= 10 ? 1 : 2)}%`;
    }
}

export const clientRamService = new ClientRamService();
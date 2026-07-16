import { invoke } from "@tauri-apps/api/core";

export interface ModEntry {
    slug: string;
    title: string;
    description: string;
    icon_url: string;
    modrinth_url: string;
    version_id: string;
    version_number: string;
    filename: string;
    download_url: string;
    file_size: number;
}

export interface ModBuild {
    id: string;
    name: string;
    description: string;
    mc_version: string;
    loader: string;
    mods: ModEntry[];
    created_at: string;
    updated_at: string;
}

export class ModBuildService {
    static async getAll(): Promise<ModBuild[]> {
        return invoke("get_all_mod_builds");
    }

    static async get(id: string): Promise<ModBuild | null> {
        return invoke("get_mod_build", { id });
    }

    static async create(build: ModBuild): Promise<ModBuild> {
        return invoke("create_mod_build", { build });
    }

    static async update(build: ModBuild): Promise<ModBuild> {
        return invoke("update_mod_build", { build });
    }

    static async delete(id: string): Promise<void> {
        return invoke("delete_mod_build", { id });
    }

    static async exportBuild(id: string): Promise<ModBuild> {
        return invoke("export_mod_build", { id });
    }

    static async importBuild(build: ModBuild): Promise<ModBuild> {
        return invoke("import_mod_build", { build });
    }

    static generateId(): string {
        return `build_${Date.now()}_${Math.random().toString(36).slice(2, 8)}`;
    }

    static formatSize(bytes: number): string {
        if (bytes < 1024) return `${bytes} B`;
        if (bytes < 1048576) return `${(bytes / 1024).toFixed(1)} KB`;
        return `${(bytes / 1048576).toFixed(1)} MB`;
    }
}

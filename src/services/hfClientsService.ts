import { invoke } from "@tauri-apps/api/core";

export interface HfClient {
    id: string | number;
    name: string;
    version: string;
    client_type: string;
    filename: string;
    md5_hash: string;
    downloads: number;
    launches: number;
    show: boolean;
    working: boolean;
    size: number;
    created_at: string;
    dependencies?: string[];
}

const CDN_URLS = [
    "https://huggingface.co/datasets/Collapsecdn/collapsecdn/resolve/main/static/clients.json",
    "https://huggingface.co/datasets/Collapsecdn/collapsecdn/resolve/main/static/fabric-clients.json",
    "https://huggingface.co/datasets/Collapsecdn/collapsecdn/resolve/main/static/forge-clients.json",
];

function tryParse(d: any): any[] {
    if (Array.isArray(d)) return d;
    if (d == null) return [];
    if (typeof d === "string") {
        try {
            const parsed = JSON.parse(d);
            return Array.isArray(parsed)
                ? parsed
                : Array.isArray(parsed?.data)
                  ? parsed.data
                  : [];
        } catch {
            return [];
        }
    }
    if (Array.isArray(d?.data)) return d.data;
    return [];
}

export const hfClientsService = {
    async fetchClients(): Promise<{
        all: HfClient[];
        latest: HfClient[];
        counts: { total: number; fabric: number; forge: number; default: number };
    }> {
        const [allData, fabricData, forgeData] = await Promise.all(
            CDN_URLS.map((url) =>
                invoke<string>("api_request", {
                    method: "GET",
                    url,
                    headers: {},
                    body: null,
                })
            )
        );

        const rawAll = tryParse(allData);
        const fabric = tryParse(fabricData);
        const forge = tryParse(forgeData);

        const map = new Map<string | number, any>();
        rawAll.forEach((c: any) => map.set(c.id, c));
        fabric.forEach((c: any) => map.set(c.id, c));
        forge.forEach((c: any) => map.set(c.id, c));

        const all = Array.from(map.values()).filter(
            (c: any) => c.show !== false
        );

        const latest = [...all]
            .sort(
                (a, b) =>
                    new Date(b.created_at).getTime() -
                    new Date(a.created_at).getTime()
            )
            .slice(0, 5);

        const counts = {
            total: all.length,
            fabric: all.filter(
                (c) => c.client_type?.toLowerCase() === "fabric"
            ).length,
            forge: all.filter(
                (c) => c.client_type?.toLowerCase() === "forge"
            ).length,
            default: all.filter(
                (c) => c.client_type?.toLowerCase() === "default"
            ).length,
        };

        return { all, latest, counts };
    },
};

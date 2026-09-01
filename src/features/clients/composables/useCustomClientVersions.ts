import { ref } from "vue";

const FALLBACK_VERSIONS: Record<string, string[]> = {
    default: ["1.8.9", "1.16.5"],
    forge: ["1.8.9"],
    fabric: ["1.21.4", "1.21.8", "1.21.11"],
};

const serverVersions = ref<Record<string, string[]>>({});
const versionsLoading = ref(false);
const versionsLoaded = ref(false);

interface HFFile {
    type: string;
    path: string;
}

const sortVersions = (versions: Set<string>): string[] =>
    [...versions].sort((a, b) => {
        const toNum = (v: string) =>
            v
                .split(".")
                .map((n) => parseInt(n, 10) || 0)
                .reduce((acc, n, i) => acc + n * Math.pow(1000, 2 - i), 0);
        return toNum(a) - toNum(b);
    });

export function useCustomClientVersions() {
    const fetchVersions = async () => {
        if (versionsLoaded.value || versionsLoading.value) return;
        versionsLoading.value = true;

        try {
            const response = await fetch(
                "https://huggingface.co/api/datasets/Collapsecdn/collapsecdn/tree/main/misc/minecraft-versions"
            );
            if (!response.ok)
                throw new Error("Failed to fetch from HuggingFace");

            const data: HFFile[] = await response.json();
            const map: Record<string, Set<string>> = {
                fabric: new Set(),
                forge: new Set(),
            };

            for (const item of data) {
                if (item.type === "file" && item.path) {
                    const filename = item.path.split("/").pop();
                    if (filename) {
                        const match = filename.match(
                            /^(fabric|forge)_(.+)\.jar$/
                        );
                        if (match) {
                            const type = match[1];
                            const version = match[2];
                            if (map[type]) {
                                map[type].add(version);
                            }
                        }
                    }
                }
            }

            serverVersions.value = {
                default: FALLBACK_VERSIONS.default,
                fabric:
                    map.fabric.size > 0
                        ? sortVersions(map.fabric)
                        : FALLBACK_VERSIONS.fabric,
                forge:
                    map.forge.size > 0
                        ? sortVersions(map.forge)
                        : FALLBACK_VERSIONS.forge,
            };
            versionsLoaded.value = true;
        } catch (e) {
            console.warn(
                "Failed to load custom client versions from HuggingFace, using fallback:",
                e
            );
            serverVersions.value = { ...FALLBACK_VERSIONS };
        } finally {
            versionsLoading.value = false;
        }
    };

    const getAvailableVersions = (clientType: string): string[] => {
        const type = clientType as keyof typeof FALLBACK_VERSIONS;
        const fromServer = serverVersions.value[type];
        if (fromServer && fromServer.length > 0) return fromServer;
        return FALLBACK_VERSIONS[type] ?? [];
    };

    return {
        fetchVersions,
        getAvailableVersions,
        versionsLoading,
    };
}

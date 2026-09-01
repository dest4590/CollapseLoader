const fs = require("fs");
const crypto = require("crypto");
const path = require("path");
const https = require("https");

const VERSION_MANIFEST_URL =
    "https://launchermeta.mojang.com/mc/game/version_manifest_v2.json";
const FALLBACK_VERSIONS = ["1.21.4", "1.21.8", "1.21.10", "1.21.11"];

function fetchVersions() {
    return new Promise((resolve) => {
        https
            .get(
                VERSION_MANIFEST_URL,
                { headers: { "User-Agent": "CollapseLoader-Script/1.0" } },
                (res) => {
                    let data = "";
                    res.on("data", (chunk) => (data += chunk));
                    res.on("end", () => {
                        try {
                            const json = JSON.parse(data);
                            const releases = json.versions
                                .filter((v) => v.type === "release")
                                .map((v) => v.id);
                            resolve(releases);
                        } catch {
                            resolve(FALLBACK_VERSIONS);
                        }
                    });
                }
            )
            .on("error", () => resolve(FALLBACK_VERSIONS));
    });
}

async function main() {
    const args = process.argv.slice(2);
    const showVersions = args.includes("--versions");

    if (showVersions) {
        console.log("Fetching Minecraft versions from Mojang...");
        const versions = await fetchVersions();
        console.log(`\nAvailable release versions (${versions.length}):\n`);
        versions.forEach((v, i) => console.log(`  ${i + 1}) ${v}`));
        process.exit(0);
    }

    const filePath = args.find(
        (a) =>
            !a.startsWith("--") &&
            !a.includes("\\") &&
            !a.includes("/") &&
            a.endsWith(".jar")
    );
    const version = args.find(
        (a) => /^\d+\.\d+/.test(a) && !a.includes("\\") && !a.includes("/")
    );
    const clientType =
        args.find((a) => ["default", "fabric", "forge"].includes(a)) ||
        "default";
    const cdnRoot =
        args.find((a) => a.includes("\\") || a.includes("/")) || "E:\\hf-cdn";
    const extraFlags = args.filter(
        (a) =>
            !a.startsWith("--") &&
            !a.includes("\\") &&
            !a.includes("/") &&
            !a.endsWith(".jar") &&
            !["default", "fabric", "forge"].includes(a) &&
            !/^\d+\.\d+/.test(a)
    );

    if (!filePath || !version) {
        console.error(
            "Usage: node scripts/new_client.cjs <path-to.jar> <version> [default|fabric|forge] [flags...] [cdn-root]"
        );
        console.error(
            "       node scripts/new_client.cjs --versions                    # list available versions"
        );
        console.error("Flags: kotlin, satin, sodium");
        console.error(
            'Example: node scripts/new_client.cjs "E:\\hf-cdn\\clients\\fabric\\jars\\lambda.jar" "1.21.11" fabric kotlin'
        );
        process.exit(1);
    }

    const KOTLIN_DEP = {
        md5_hash: "964103287b72e606de845420d1a8cc57",
        name: "fabric-language-kotlin-1.13.8+kotlin.2.3.0",
        size: 7,
    };
    const SATIN_DEP = {
        md5_hash: "2cf1534f9e818bd567837979444557e9",
        name: "satin-3.0.0-alpha.1",
        size: 0,
    };
    const SODIUM_DEP = {
        md5_hash: "28922a78d1876ee062e3265f10abcc46",
        name: "sodium-fabric-0.6.13+mc1.21.4",
        size: 1,
    };

    const BARITONE_DEPS = {
        "1.21.4": [
            {
                md5_hash: "0f8e922606f64c422cafafc0ad887c0e",
                name: "baritone-api-fabric-1.13.1",
                size: 2,
            },
            {
                md5_hash: "56cc7fc0294adc92cbbefe6f456d8f68",
                name: "baritone-standalone-fabric-1.13.1",
                size: 1,
            },
            {
                md5_hash: "015e00b79c6ae76881373d367b88a565",
                name: "baritone-unoptimized-fabric-1.13.1",
                size: 2,
            },
        ],
    };

    const FABRIC_BASE_DEPS = {
        "1.21.4": [
            {
                md5_hash: "128a8d042180e7c92567342e21a21a6d",
                name: "fabric-api-0.119.4+1.21.4",
                size: 2,
            },
        ],
        "1.21.8": [
            {
                md5_hash: "85d76d57a7b5bb7043ea815133d2f6ba",
                name: "fabric-api-0.136.1+1.21.8",
                size: 2,
            },
        ],
        "1.21.10": [
            {
                md5_hash: "c9ebf1b300d813310d18115a7cc03f99",
                name: "fabric-api-0.138.4+1.21.10",
                size: 2,
            },
        ],
        "1.21.11": [
            {
                md5_hash: "e2a72b6c6aa2c6c4f74541394858c86a",
                name: "fabric-api-0.140.2+1.21.11",
                size: 2,
            },
        ],
    };

    const MAIN_CLASSES = {
        default: "net.minecraft.client.main.Main",
        fabric: "net.fabricmc.loader.launch.knot.KnotClient",
        forge: "net.minecraft.launchwrapper.Launch",
    };

    const JSON_FILES = {
        default: path.join(cdnRoot, "static", "clients.json"),
        fabric: path.join(cdnRoot, "static", "fabric-clients.json"),
        forge: path.join(cdnRoot, "static", "forge-clients.json"),
    };

    const FILENAME_PREFIXES = {
        default: "",
        fabric: "fabric/",
        forge: "forge/",
    };

    const jsonPath = JSON_FILES[clientType];
    const existing = fs.existsSync(jsonPath)
        ? JSON.parse(fs.readFileSync(jsonPath, "utf8"))
        : [];

    const file = path.basename(filePath);
    const filename = FILENAME_PREFIXES[clientType] + file;

    if (existing.find((c) => c.filename === filename)) {
        console.error(
            `Client "${filename}" already exists in ${path.basename(jsonPath)}`
        );
        process.exit(1);
    }

    const buf = fs.readFileSync(filePath);
    const md5 = crypto.createHash("md5").update(buf).digest("hex");
    const size = Math.round(fs.statSync(filePath).size / 1024 / 1024);
    const nextId =
        existing.length > 0 ? Math.max(...existing.map((c) => c.id)) + 1 : 1;

    const entry = {
        client_type: clientType,
        created_at: new Date().toISOString(),
        downloads: 0,
        filename,
        id: nextId,
        launches: 0,
        main_class: MAIN_CLASSES[clientType],
        md5_hash: md5,
        name: path.basename(file, ".jar"),
        show: true,
        size,
        version,
        working: true,
    };

    if (clientType === "fabric") {
        const deps = [...(FABRIC_BASE_DEPS[version] || [])];
        if (extraFlags.includes("kotlin")) deps.push(KOTLIN_DEP);
        if (extraFlags.includes("satin")) deps.push(SATIN_DEP);
        if (extraFlags.includes("sodium")) deps.push(SODIUM_DEP);
        if (extraFlags.includes("baritone")) {
            if (BARITONE_DEPS[version]) {
                for (const dep of BARITONE_DEPS[version]) {
                    deps.push(dep);
                }
            } else {
                console.warn(
                    `Warning: baritone requested but no definition for version ${version}`
                );
            }
        }
        entry.dependencies = deps;
        if (extraFlags.length)
            console.log(`Extra deps: ${extraFlags.join(", ")}`);
    }
    if (clientType === "forge") {
        entry.dependencies = [];
    }

    existing.unshift(entry);
    fs.mkdirSync(path.dirname(jsonPath), { recursive: true });
    fs.writeFileSync(jsonPath, JSON.stringify(existing, null, 2));

    console.log(
        `Added "${entry.name}" (id=${entry.id}) to ${path.basename(jsonPath)}`
    );
    console.log(`md5: ${md5} | size: ${size}MB | version: ${version}`);
}

main();

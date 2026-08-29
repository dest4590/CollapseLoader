#!/usr/bin/env python3
"""new_client.py – Add a new client entry to the CDN JSON.

Usage:
    python scripts/new_client.py                      # interactive menu
    python scripts/new_client.py <jar> <ver> [type]   # direct mode
"""

import json
import hashlib
import os
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


KOTLIN_DEP = {"md5_hash": "964103287b72e606de845420d1a8cc57", "name": "fabric-language-kotlin-1.13.8+kotlin.2.3.0", "size": 7}
SATIN_DEP = {"md5_hash": "2cf1534f9e818bd567837979444557e9", "name": "satin-3.0.0-alpha.1", "size": 0}
SODIUM_DEP = {"md5_hash": "28922a78d1876ee062e3265f10abcc46", "name": "sodium-fabric-0.6.13+mc1.21.4", "size": 1}

BARITONE_DEPS = {
    "1.21.4": [
        {"md5_hash": "0f8e922606f64c422cafafc0ad887c0e", "name": "baritone-api-fabric-1.13.1", "size": 2},
        {"md5_hash": "56cc7fc0294adc92cbbefe6f456d8f68", "name": "baritone-standalone-fabric-1.13.1", "size": 1},
        {"md5_hash": "015e00b79c6ae76881373d367b88a565", "name": "baritone-unoptimized-fabric-1.13.1", "size": 2},
    ],
}

MAIN_CLASSES = {
    "default": "net.minecraft.client.main.Main",
    "fabric": "net.fabricmc.loader.launch.knot.KnotClient",
    "forge": "net.minecraft.launchwrapper.Launch",
}

CLIENT_TYPES = ["default", "fabric", "forge"]

FILENAMES = {
    "default": "clients.json",
    "fabric": "fabric-clients.json",
    "forge": "forge-clients.json",
}

CDN_ROOT = os.environ.get("CDN_ROOT", "/media/w1xced/Disk/hf-cdn")
HF_VERSIONS_URL = "https://huggingface.co/api/datasets/Collapsecdn/collapsecdn/tree/main/misc/minecraft-versions"
FALLBACK_VERSIONS: dict[str, list[str]] = {
    "default": ["1.8.9", "1.16.5"],
    "fabric": ["1.21.4", "1.21.8", "1.21.11"],
    "forge": ["1.8.9"],
}


def compute_md5(filepath: Path) -> str:
    h = hashlib.md5()
    with open(filepath, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()


def json_path(client_type: str) -> Path:
    return Path(CDN_ROOT) / "static" / FILENAMES[client_type]


def load_clients(client_type: str) -> list:
    p = json_path(client_type)
    if p.exists():
        return json.loads(p.read_text(encoding="utf-8"))
    return []


def save_clients(client_type: str, data: list):
    p = json_path(client_type)
    p.parent.mkdir(parents=True, exist_ok=True)
    p.write_text(json.dumps(data, indent=2, ensure_ascii=False), encoding="utf-8")


def pick_jar() -> Path | None:
    """Find .jar files and let user pick one."""
    search_dirs = [Path("."), Path("..")]
    jar_files = []
    for d in search_dirs:
        jar_files.extend(d.rglob("*.jar"))

    if not jar_files:
        print("No .jar files found.")
        return None

    print("\nAvailable .jar files:\n")
    for i, f in enumerate(jar_files, 1):
        print(f"  {i}) {f}")

    print(f"\n  0) Cancel\n")
    try:
        choice = int(input("Select file: "))
    except (ValueError, EOFError):
        return None

    if choice == 0 or choice > len(jar_files):
        return None
    return jar_files[choice - 1]


def pick_client_type() -> str | None:
    print("\nClient type:\n")
    for i, t in enumerate(CLIENT_TYPES, 1):
        print(f"  {i}) {t}")

    print(f"\n  0) Cancel\n")
    try:
        choice = int(input("Select type: "))
    except (ValueError, EOFError):
        return None

    if choice == 0 or choice > len(CLIENT_TYPES):
        return None
    return CLIENT_TYPES[choice - 1]


def _sort_versions(versions: list[str]) -> list[str]:
    def to_num(v: str) -> int:
        parts = v.split(".")
        return sum(int(p) * (1000 ** (2 - i)) for i, p in enumerate(parts))
    return sorted(versions, key=to_num)


FABRIC_API_RE = re.compile(r"^fabric-api-([0-9.]+\+\d+\.\d+\.\d+)\.jar$")


def scan_local_deps(cdn_root: str) -> dict:
    """Scan {cdn_root}/clients/fabric/deps/jars/ and return parsed deps with MD5."""
    deps_dir = os.path.join(cdn_root, "clients", "fabric", "deps", "jars")
    result: dict[str, dict] = {}
    if not os.path.isdir(deps_dir):
        return result
    for fname in sorted(os.listdir(deps_dir)):
        if not fname.endswith(".jar"):
            continue
        fpath = os.path.join(deps_dir, fname)
        if not os.path.isfile(fpath):
            continue
        md5 = compute_md5(Path(fpath))
        size_mb = round(os.path.getsize(fpath) / 1024 / 1024)
        result[fname] = {
            "md5_hash": md5,
            "name": fname.replace(".jar", ""),
            "size": size_mb,
        }
    return result


def _find_dep(local_deps: dict, keyword: str) -> dict | None:
    """Find a dep by keyword in local scanned deps."""
    for fname, info in local_deps.items():
        if keyword.lower() in fname.lower():
            return {"md5_hash": info["md5_hash"], "name": info["name"], "size": info["size"]}
    return None


def pick_fabric_api(cdn_root: str, mc_version: str) -> dict | None:
    """Let user pick a fabric-api version from local deps, auto-compute MD5."""
    local = scan_local_deps(cdn_root)
    api_deps = {k: v for k, v in local.items() if FABRIC_API_RE.match(k)}
    if not api_deps:
        print("No fabric-api jars found in CDN deps folder.")
        return None

    matching = {k: v for k, v in api_deps.items() if f"+{mc_version}.jar" in k}
    if not matching:
        print(f"\nNo fabric-api for MC {mc_version}. Available:")
        matching = api_deps

    items = sorted(matching.items(), key=lambda x: x[1]["name"])
    print("\nFabric API version:\n")
    for i, (_, info) in enumerate(items, 1):
        print(f"  {i}) {info['name']}  (md5: {info['md5_hash'][:12]}...)")
    print(f"\n  0) None (skip fabric-api)\n")

    try:
        choice = int(input("Select fabric-api: "))
    except (ValueError, EOFError):
        return None

    if choice == 0 or choice > len(items):
        return None
    _, info = items[choice - 1]
    return {"md5_hash": info["md5_hash"], "name": info["name"], "size": info["size"]}


def fetch_cdn_versions() -> dict[str, list[str]]:
    """Parse jar filenames from HuggingFace CDN to get available versions per type."""
    try:
        result = subprocess.run(
            ["curl", "-s", "--max-time", "10", HF_VERSIONS_URL],
            capture_output=True, text=True, timeout=15,
        )
        if result.returncode == 0 and result.stdout:
            data = json.loads(result.stdout)
            map_fabric: set[str] = set()
            map_forge: set[str] = set()
            for item in data:
                if item.get("type") == "file" and item.get("path"):
                    filename = item["path"].rsplit("/", 1)[-1]
                    m = re.match(r"^(fabric|forge)_(.+)\.jar$", filename)
                    if m:
                        kind, ver = m.group(1), m.group(2)
                        if kind == "fabric":
                            map_fabric.add(ver)
                        elif kind == "forge":
                            map_forge.add(ver)
            return {
                "default": FALLBACK_VERSIONS["default"],
                "fabric": _sort_versions(list(map_fabric)) if map_fabric else FALLBACK_VERSIONS["fabric"],
                "forge": _sort_versions(list(map_forge)) if map_forge else FALLBACK_VERSIONS["forge"],
            }
    except Exception:
        pass
    return {**FALLBACK_VERSIONS}


def pick_version(client_type: str = "fabric") -> str | None:
    cdn = fetch_cdn_versions()
    versions = cdn.get(client_type, cdn.get("fabric", []))
    print("\nMinecraft version:\n")
    for i, v in enumerate(versions, 1):
        print(f"  {i}) {v}")

    print(f"\n  0) Cancel\n")
    try:
        choice = int(input("Select version: "))
    except (ValueError, EOFError):
        return None

    if choice == 0 or choice > len(versions):
        return None
    return versions[choice - 1]


def pick_flags() -> list[str]:
    available = ["kotlin", "satin", "sodium", "baritone"]
    print("\nExtra dependencies (comma-separated):\n")
    for i, f in enumerate(available, 1):
        print(f"  {i}) {f}")

    print(f"\n  0) None\n")
    try:
        raw = input("Select flags: ").strip()
    except EOFError:
        return []

    if raw == "0" or not raw:
        return []

    selected = []
    for part in raw.split(","):
        part = part.strip()
        if part.isdigit():
            idx = int(part) - 1
            if 0 <= idx < len(available):
                selected.append(available[idx])
        elif part in available:
            selected.append(part)

    return selected


def main():
    jar_path = None
    version = None
    client_type = None

    if len(sys.argv) >= 3:
        jar_path = Path(sys.argv[1])
        version = sys.argv[2]
        client_type = sys.argv[3] if len(sys.argv) > 3 else "default"
    else:
        print("=" * 50)
        print("  CollapseLoader – New Client")
        print("=" * 50)

        jar_path = pick_jar()
        if not jar_path:
            print("Cancelled.")
            sys.exit(0)

        client_type = pick_client_type()
        if not client_type:
            print("Cancelled.")
            sys.exit(0)

        version = pick_version(client_type)
        if not version:
            print("Cancelled.")
            sys.exit(0)

    if not jar_path.exists():
        print(f"File not found: {jar_path}")
        sys.exit(1)

    filename = jar_path.name
    if client_type != "default":
        filename = f"{client_type}/{filename}"

    existing = load_clients(client_type)
    if any(c.get("filename") == filename for c in existing):
        print(f'Client "{filename}" already exists in {FILENAMES[client_type]}')
        sys.exit(1)

    md5 = compute_md5(jar_path)
    size_mb = round(jar_path.stat().st_size / 1024 / 1024)
    next_id = max((c.get("id", 0) for c in existing), default=0) + 1

    entry = {
        "client_type": client_type,
        "created_at": datetime.now(timezone.utc).isoformat(),
        "downloads": 0,
        "filename": filename,
        "id": next_id,
        "launches": 0,
        "main_class": MAIN_CLASSES[client_type],
        "md5_hash": md5,
        "name": jar_path.stem,
        "show": True,
        "size": size_mb,
        "version": version,
        "working": True,
    }

    if client_type == "fabric":
        deps = []
        fabric_api = pick_fabric_api(CDN_ROOT, version)
        if fabric_api:
            deps.append(fabric_api)

        if len(sys.argv) > 4:
            flags = [a.lower() for a in sys.argv[4:] if not os.sep in a and "/" not in a]
        else:
            flags = pick_flags()

        local = scan_local_deps(CDN_ROOT)
        if "kotlin" in flags:
            dep = _find_dep(local, "kotlin")
            if dep: deps.append(dep)
            else: deps.append(KOTLIN_DEP)
        if "satin" in flags:
            dep = _find_dep(local, "satin")
            if dep: deps.append(dep)
            else: deps.append(SATIN_DEP)
        if "sodium" in flags:
            dep = _find_dep(local, "sodium")
            if dep: deps.append(dep)
            else: deps.append(SODIUM_DEP)
        if "baritone" in flags:
            known_names = {d["name"] for d in BARITONE_DEPS.get(version, [])}
            local_baritone = [v for k, v in local.items() if "baritone" in k.lower() and v["name"] in known_names]
            if local_baritone:
                for dep in local_baritone:
                    deps.append({"md5_hash": dep["md5_hash"], "name": dep["name"], "size": dep["size"]})
            elif version in BARITONE_DEPS:
                for dep in BARITONE_DEPS[version]:
                    deps.append(dep)
            else:
                print(f"Warning: baritone not available for {version}")

        entry["dependencies"] = deps
        if flags:
            print(f"Extra deps: {', '.join(flags)}")

    elif client_type == "forge":
        entry["dependencies"] = []

    existing.insert(0, entry)
    save_clients(client_type, existing)

    print(f'\nAdded "{entry["name"]}" (id={entry["id"]}) to {FILENAMES[client_type]}')
    print(f"md5: {md5} | size: {size_mb}MB | version: {version}")


if __name__ == "__main__":
    main()

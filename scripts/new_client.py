#!/usr/bin/env python3
"""new_client.py – Add a new client entry to the CDN JSON.

Usage:
    python scripts/new_client.py                      # interactive menu
    python scripts/new_client.py <jar> <ver> [type]   # direct mode
"""

import json
import hashlib
import os
import sys
from datetime import datetime, timezone
from pathlib import Path


KOTLIN_DEP = {"md5_hash": "964103287b72e606de845420d1a8cc57", "name": "fabric-language-kotlin-1.13.8+kotlin.2.3.0", "size": 7}
SATIN_DEP = {"md5_hash": "2cf1534f9e818bd567837979444557e9", "name": "satin-3.0.0-alpha.1", "size": 0}
SODIUM_DEP = {"md5_hash": "28922a78d1876ee062e3265f10abcc46", "name": "sodium-fabric-0.6.13+mc1.21.4", "size": 1}

BARITONE_DEPS = {
    "1.21.11": {"md5_hash": "dbd83c7de8426f2facdc73f0a3a1da48", "name": "baritone-1.21.11", "size": 2},
}

FABRIC_BASE_DEPS = {
    "1.21.4": [{"md5_hash": "128a8d042180e7c92567342e21a21a6d", "name": "fabric-api-0.119.4+1.21.4", "size": 2}],
    "1.21.8": [{"md5_hash": "85d76d57a7b5bb7043ea815133d2f6ba", "name": "fabric-api-0.136.1+1.21.8", "size": 2}],
    "1.21.10": [{"md5_hash": "c9ebf1b300d813310d18115a7cc03f99", "name": "fabric-api-0.138.4+1.21.10", "size": 2}],
    "1.21.11": [{"md5_hash": "e2a72b6c6aa2c6c4f74541394858c86a", "name": "fabric-api-0.140.2+1.21.11", "size": 2}],
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


def pick_version() -> str | None:
    versions = ["1.21.4", "1.21.8", "1.21.10", "1.21.11"]
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

        version = pick_version()
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
        deps = list(FABRIC_BASE_DEPS.get(version, []))
        if len(sys.argv) > 4:
            flags = [a.lower() for a in sys.argv[4:] if not os.sep in a and "/" not in a]
        else:
            flags = pick_flags()

        if "kotlin" in flags:
            deps.append(KOTLIN_DEP)
        if "satin" in flags:
            deps.append(SATIN_DEP)
        if "sodium" in flags:
            deps.append(SODIUM_DEP)
        if "baritone" in flags:
            if version in BARITONE_DEPS:
                deps.append(BARITONE_DEPS[version])
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

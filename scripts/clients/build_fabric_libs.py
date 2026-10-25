#!/usr/bin/env python3
"""build_fabric_libs.py – Build/patch Fabric library zips for the CDN.

Generates misc/libraries-fabric/<mc>.zip for Minecraft versions and can
update the common misc/libraries-fabric.zip ASM set to match the bundled
fabric-loader version (required by loader 0.19+ classpath verification).

Also supports downloading the latest Fabric API from Modrinth into
clients/fabric/deps/jars/ for each MC version.

Usage:
    python scripts/clients/build_fabric_libs.py --mc 26.1.2 --mc 26.2 \
        --loader 0.19.5 --update-common-asm --fetch-fabric-api
"""

import argparse
import hashlib
import io
import json
import os
import re
import urllib.request
import zipfile
from pathlib import Path

CDN_ROOT = os.environ.get("CDN_ROOT", "/media/w1xced/disk/collapsecdn")
MOJANG_MANIFEST = "https://piston-meta.mojang.com/mc/game/version_manifest_v2.json"
FABRIC_MAVEN = "https://maven.fabricmc.net/"
MODRINTH_API = "https://api.modrinth.com/v2"
WORK_DIR = Path("/tmp/opencode/fabric-libs-cache")
USER_AGENT = "CollapseLoader-CDN-Tools"


def fetch(url: str, dest: Path | None = None) -> bytes:
    if dest and dest.exists():
        return dest.read_bytes()
    req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
    data = urllib.request.urlopen(req, timeout=120).read()
    if dest:
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(data)
    return data


def sha1(data: bytes) -> str:
    return hashlib.sha1(data).hexdigest()


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def artifact_basename(name: str) -> str:
    group, art, ver = name.rsplit(":", 2)[0], name.split(":")[-2], name.split(":")[-1]
    return f"{art}-{ver}.jar"


def artifact_url(name: str, repo: str = FABRIC_MAVEN) -> str:
    group, art, ver, *classifier = name.split(":")
    path = group.replace(".", "/")
    suffix = f"-{classifier[0]}" if classifier else ""
    return f"{repo}{path}/{art}/{ver}/{art}-{ver}{suffix}.jar"


def artifact_id(jar_base: str) -> str:
    stem = jar_base[:-4] if jar_base.endswith(".jar") else jar_base
    m = re.match(r"^(.*?)-([0-9][0-9A-Za-z.\-]*)$", stem)
    return m.group(1) if m else stem


def mojang_version_json(mc: str, cache: Path) -> dict:
    manifest = json.loads(fetch(MOJANG_MANIFEST, cache / "manifest.json"))
    url = next(v["url"] for v in manifest["versions"] if v["id"] == mc)
    return json.loads(fetch(url, cache / f"{mc}.json"))


def loader_installer_libs(loader: str, cache: Path) -> list[dict]:
    jar = fetch(
        f"{FABRIC_MAVEN}net/fabricmc/fabric-loader/{loader}/fabric-loader-{loader}.jar",
        cache / f"fabric-loader-{loader}.jar",
    )
    with zipfile.ZipFile(io.BytesIO(jar)) as z:
        meta = json.loads(z.read("fabric-installer.json"))
    return meta["libraries"]["common"]


def load_common(cdn_root: str) -> tuple[zipfile.ZipFile, str]:
    path = Path(cdn_root) / "misc" / "libraries-fabric.zip"
    return zipfile.ZipFile(path), str(path)


def update_common_asm(cdn_root: str, installer_libs: list[dict]) -> None:
    src, path = load_common(cdn_root)
    entries = {n: src.read(n) for n in src.namelist() if not n.endswith("/")}
    src.close()

    asm_libs = [l for l in installer_libs if l["name"].startswith("org.ow2.asm:")]
    arts = {l["name"].split(":")[-2] for l in asm_libs}
    for name in list(entries):
        base = name.split("/")[-1]
        if any(re.match(rf"^{a}-\d.*\.jar$", base) for a in arts):
            del entries[name]
            print(f"  common: removed {base}")

    for lib in asm_libs:
        base = artifact_basename(lib["name"])
        data = fetch(artifact_url(lib["name"], lib.get("url", FABRIC_MAVEN)), WORK_DIR / base)
        entries[f"libraries-fabric/{base}"] = data
        print(f"  common: added {base}")

    manifest = "\n".join(
        f"{n.split('/', 1)[-1]}:{sha256(d)}" for n, d in sorted(entries.items())
    ) + "\n"
    entries["libraries-fabric/manifest.txt"] = manifest.encode()

    tmp = path + ".tmp"
    with zipfile.ZipFile(tmp, "w", zipfile.ZIP_DEFLATED, compresslevel=6) as z:
        for n in sorted(entries):
            z.writestr(n, entries[n])
    zipfile.ZipFile(tmp).testzip()
    os.replace(tmp, path)
    print(f"common zip updated: {path}")


def common_basenames(cdn_root: str) -> set[str]:
    src, _ = load_common(cdn_root)
    names = {n.split("/")[-1] for n in src.namelist() if n.endswith(".jar")}
    src.close()
    return names


def build_version_zip(cdn_root: str, mc: str, loader: str, installer_libs: list[dict]) -> None:
    cache = WORK_DIR
    meta = mojang_version_json(mc, cache)

    common = common_basenames(cdn_root)
    common_ids: dict[str, set[str]] = {}
    for b in common:
        common_ids.setdefault(artifact_id(b), set()).add(b)

    entries: dict[str, bytes] = {}
    skipped = []
    for lib in meta["libraries"]:
        art = lib.get("downloads", {}).get("artifact")
        if not art:
            continue
        base = art["path"].split("/")[-1]
        aid = artifact_id(base)
        # Skip only when common holds exactly this version and no other version
        # of the same artifact (a divergent copy must be shadowed by placing the
        # json version in the version dir, which precedes common on the classpath).
        if base in common and not (common_ids.get(aid, set()) - {base}):
            skipped.append(base)
            continue
        entries[base] = fetch(art["url"], cache / base)
        assert sha1(entries[base]) == art["sha1"], f"{mc}: sha1 mismatch {base}"

    # Loader-required libs that are not MC-version specific and not in common.
    for lib in installer_libs:
        if lib["name"].startswith("org.ow2.asm:"):
            continue  # ASM lives in the common zip only (loader forbids duplicates)
        base = artifact_basename(lib["name"])
        if base in common:
            continue
        entries[base] = fetch(artifact_url(lib["name"]), cache / base)

    # The loader jar itself is not part of fabric-installer.json libraries.
    loader_base = f"fabric-loader-{loader}.jar"
    entries[loader_base] = fetch(
        f"{FABRIC_MAVEN}net/fabricmc/fabric-loader/{loader}/fabric-loader-{loader}.jar",
        cache / loader_base,
    )

    inter = f"intermediary-{mc}.jar"
    entries[inter] = fetch(
        f"{FABRIC_MAVEN}net/fabricmc/intermediary/{mc}/{inter}", cache / inter
    )

    manifest = "\n".join(f"{n}:{sha256(d)}" for n, d in sorted(entries.items())) + "\n"

    out = Path(cdn_root) / "misc" / "libraries-fabric" / f"{mc}.zip"
    tmp = str(out) + ".tmp"
    with zipfile.ZipFile(tmp, "w", zipfile.ZIP_DEFLATED, compresslevel=6) as z:
        for n in sorted(entries):
            z.writestr(n, entries[n])
        z.writestr("manifest.txt", manifest)
    zipfile.ZipFile(tmp).testzip()
    os.replace(tmp, out)
    raw = sum(len(d) for d in entries.values())
    print(f"{mc}.zip: {len(entries)} jars, {raw / 1e6:.1f} MB raw ({len(skipped)} shared w/ common)")

    # Classpath completeness check.
    cp = set(entries) | common
    for lib in meta["libraries"]:
        art = lib.get("downloads", {}).get("artifact")
        if art:
            base = art["path"].split("/")[-1]
            assert base in cp, f"{mc}: missing on classpath: {base}"
    for lib in installer_libs:
        base = artifact_basename(lib["name"])
        assert base in cp, f"{mc}: missing loader lib: {base}"
    assert loader_base in cp, f"{mc}: missing {loader_base}"
    # Any json lib whose artifact exists in common under a different version
    # must live in the version dir to shadow it.
    for lib in meta["libraries"]:
        art = lib.get("downloads", {}).get("artifact")
        if not art:
            continue
        base = art["path"].split("/")[-1]
        aid = artifact_id(base)
        if any(b != base for b in common if artifact_id(b) == aid):
            assert base in entries, f"{mc}: {base} must shadow divergent common copy"
    asm_in_version = [n for n in entries if re.match(r"^asm.*\.jar$", n)]
    assert not asm_in_version, f"{mc}: asm must not be in version zip: {asm_in_version}"
    print(f"{mc}.zip: classpath verification passed")


def fetch_fabric_api(mc_versions: list[str], cdn_root: str) -> None:
    """Download the latest Fabric API for each MC version from Modrinth."""
    deps_dir = Path(cdn_root) / "clients" / "fabric" / "deps" / "jars"
    deps_dir.mkdir(parents=True, exist_ok=True)
    for mc in mc_versions:
        url = (
            f"{MODRINTH_API}/project/P7dR8mSH/version"
            f"?game_versions=%5B%22{mc}%22%5D&loaders=%5B%22fabric%22%5D"
        )
        req = urllib.request.Request(url, headers={"User-Agent": USER_AGENT})
        data = json.loads(urllib.request.urlopen(req, timeout=30).read())
        if not data:
            print(f"  fabric-api: no version found for {mc}")
            continue
        best = data[0]
        file_info = best["files"][0]
        filename = file_info["filename"]
        file_url = file_info["url"]
        dest = deps_dir / filename
        if dest.exists() and sha1(dest.read_bytes()) == file_info.get("hashes", {}).get("sha1", ""):
            print(f"  fabric-api: {filename} already present")
            continue
        raw = fetch(file_url, dest)
        print(f"  fabric-api: {filename} ({len(raw) / 1e6:.1f} MB)")
        print(f"    md5: {hashlib.md5(raw).hexdigest()}")
        print(f"    sha1: {file_info.get('hashes', {}).get('sha1', sha1(raw))}")


def main() -> None:
    ap = argparse.ArgumentParser()
    ap.add_argument("--mc", action="append", required=True, help="MC version (repeatable)")
    ap.add_argument("--loader", default="0.19.5", help="fabric-loader version")
    ap.add_argument("--cdn-root", default=CDN_ROOT)
    ap.add_argument("--update-common-asm", action="store_true",
                    help="align common zip ASM set with loader requirements")
    ap.add_argument("--fetch-fabric-api", action="store_true",
                    help="download latest fabric-api for each --mc from Modrinth")
    args = ap.parse_args()

    WORK_DIR.mkdir(parents=True, exist_ok=True)
    installer_libs = loader_installer_libs(args.loader, WORK_DIR)
    print(f"loader {args.loader}: "
          + ", ".join(l["name"] for l in installer_libs))

    if args.update_common_asm:
        update_common_asm(args.cdn_root, installer_libs)

    for mc in args.mc:
        build_version_zip(args.cdn_root, mc, args.loader, installer_libs)

    if args.fetch_fabric_api:
        fetch_fabric_api(args.mc, args.cdn_root)


if __name__ == "__main__":
    main()

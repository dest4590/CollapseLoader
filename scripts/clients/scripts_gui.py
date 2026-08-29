#!/usr/bin/env python3

import hashlib
import json
import os
import re

from datetime import datetime, timezone
from http.server import BaseHTTPRequestHandler, HTTPServer
from pathlib import Path
import webbrowser

CDN_ROOT = os.environ.get("CDN_ROOT", "/media/w1xced/disk/collapsecdn")

FALLBACK_VERSIONS: dict[str, list[str]] = {
    "default": ["1.8.9", "1.16.5"],
    "fabric": ["1.21.4", "1.21.8", "1.21.11"],
    "forge": ["1.8.9"],
}

FABRIC_API_RE = re.compile(r"^fabric-api-([0-9.]+\+\d+\.\d+\.\d+)\.jar$")
OTHER_DEP_RE = re.compile(r"^(.+)\.jar$")

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
FILENAMES = {"default": "clients.json", "fabric": "fabric-clients.json", "forge": "forge-clients.json"}
DEFAULT_VIAVERSION = "5.9.1"
VIA_VERSIONS = ["5.3.0", "5.9.1", "5.11.0"]

HTML_TEMPLATE_PATH = Path(__file__).parent / "gui_template.html"


def _sort_versions(versions: list[str]) -> list[str]:
    def to_num(v: str) -> int:
        parts = v.split(".")
        return sum(int(p) * (1000 ** (2 - i)) for i, p in enumerate(parts))
    return sorted(versions, key=to_num)


def compute_md5(filepath):
    h = hashlib.md5()
    with open(filepath, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()


def scan_cdn_client_versions(cdn_root: str) -> dict[str, list[str]]:
    result: dict[str, list[str]] = {t: [] for t in FALLBACK_VERSIONS}
    mv_dir = os.path.join(cdn_root, "misc", "minecraft-versions")
    if not os.path.isdir(mv_dir):
        return {"default": FALLBACK_VERSIONS["default"], "fabric": FALLBACK_VERSIONS["fabric"], "forge": FALLBACK_VERSIONS["forge"]}
    for client_type in ["fabric", "forge"]:
        versions: set[str] = set()
        for fname in os.listdir(mv_dir):
            if not fname.endswith(".jar"):
                continue
            m = re.match(r"^" + re.escape(client_type) + r"_(.+)\.jar$", fname)
            if m:
                versions.add(m.group(1))
        result[client_type] = _sort_versions(list(versions)) if versions else FALLBACK_VERSIONS.get(client_type, [])
    result["default"] = FALLBACK_VERSIONS["default"]
    return result


def _find_dep(local_other: dict, keyword: str) -> dict | None:
    for fname, info in local_other.items():
        if keyword.lower() in fname.lower():
            return {"md5_hash": info["md5_hash"], "name": info["name"], "size": info["size"]}
    return None


def _find_file_by_name(name: str) -> str | None:
    base = CDN_ROOT
    search_dirs = [
        os.path.join(base, "clients", "fabric", "deps", "jars"),
        os.path.join(base, "misc", "minecraft-versions"),
        os.path.join(base, "clients", "fabric"),
        base,
    ]
    for d in search_dirs:
        if not os.path.isdir(d):
            continue
        for root, dirs, files in os.walk(d):
            if name in files:
                return os.path.join(root, name)
    return None


def scan_local_deps(cdn_root: str) -> dict:
    deps_dir = os.path.join(cdn_root, "clients", "fabric", "deps", "jars")
    result = {"fabric_api": {}, "other": {}}
    if not os.path.isdir(deps_dir):
        return result
    for fname in sorted(os.listdir(deps_dir)):
        if not fname.endswith(".jar"):
            continue
        fpath = os.path.join(deps_dir, fname)
        if not os.path.isfile(fpath):
            continue
        md5 = compute_md5(fpath)
        size_mb = round(os.path.getsize(fpath) / 1024 / 1024)
        m_api = FABRIC_API_RE.match(fname)
        if m_api:
            api_ver = m_api.group(1)
            parts = api_ver.split("+")
            result["fabric_api"][fname] = {
                "md5_hash": md5,
                "name": fname.replace(".jar", ""),
                "size": size_mb,
                "api_version": parts[0] if parts else "",
                "mc_version": parts[1] if len(parts) > 1 else "",
            }
        else:
            m_other = OTHER_DEP_RE.match(fname)
            if m_other:
                result["other"][fname] = {
                    "md5_hash": md5,
                    "name": fname.replace(".jar", ""),
                    "size": size_mb,
                }
    return result


class Handler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        print(f"[SERVER] {format % args}")

    def _json(self, code, data):
        body = json.dumps(data).encode()
        self.send_response(code)
        self.send_header("Content-Type", "application/json")
        self.send_header("Content-Length", str(len(body)))
        self.end_headers()
        self.wfile.write(body)

    def _read_body(self):
        length = int(self.headers.get("Content-Length", 0))
        return json.loads(self.rfile.read(length)) if length else {}

    def do_GET(self):
        if self.path == "/" or self.path == "/index.html":
            versions = scan_cdn_client_versions(CDN_ROOT)
            deps = scan_local_deps(CDN_ROOT)
            payload = json.dumps({"versions": versions, "deps": deps, "viaversions": VIA_VERSIONS, "default_viaversion": DEFAULT_VIAVERSION}).replace("\\", "\\\\").replace("'", "\\'")
            html = HTML_TEMPLATE_PATH.read_text(encoding="utf-8")
            body = html.replace("CDN_ROOT_PLACEHOLDER", CDN_ROOT).replace("/*__INIT_DATA__*/", f"window.__INIT={payload};").encode()
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
        else:
            self.send_error(404)

    def do_POST(self):
        try:
            if self.path == "/api/load":
                data = self._read_body()
                root = data.get("cdn_root", CDN_ROOT)
                versions = scan_cdn_client_versions(root)
                deps = scan_local_deps(root)
                self._json(200, {"versions": versions, "deps": deps})

            elif self.path == "/api/deps":
                data = self._read_body()
                root = data.get("cdn_root", CDN_ROOT)
                deps = scan_local_deps(root)
                self._json(200, {"deps": deps})

            elif self.path == "/api/md5":
                data = self._read_body()
                filepath = data.get("path", "").strip()
                if not filepath:
                    self._json(200, {"error": "No file path provided"})
                    return
                if not os.path.isfile(filepath):
                    found = _find_file_by_name(filepath)
                    if found:
                        filepath = found
                    else:
                        self._json(200, {"error": f"File not found: {filepath}"})
                        return
                digest = compute_md5(filepath)
                self._json(200, {"name": os.path.basename(filepath), "hash": digest})

            elif self.path == "/api/client":
                data = self._read_body()
                jar = data.get("jar", "").strip()
                version = data.get("version", "1.21.11")
                client_type = data.get("client_type", "fabric")
                flags = data.get("flags", [])
                viaversion = data.get("viaversion", DEFAULT_VIAVERSION)
                if viaversion not in VIA_VERSIONS:
                    viaversion = DEFAULT_VIAVERSION
                cdn_root = data.get("cdn_root", CDN_ROOT)

                if not jar:
                    self._json(200, {"error": "No JAR file specified"})
                    return

                json_file = FILENAMES.get(client_type, "clients.json")
                json_path = os.path.join(cdn_root, "static", json_file)

                existing = []
                if os.path.exists(json_path):
                    with open(json_path, "r", encoding="utf-8") as f:
                        existing = json.load(f)

                filename = jar if client_type == "default" else f"{client_type}/{jar}"
                if any(c.get("filename") == filename for c in existing):
                    self._json(200, {"error": f'Client "{filename}" already exists in {json_file}'})
                    return

                md5 = compute_md5(jar) if os.path.isfile(jar) else "unknown"
                size_mb = 0
                if os.path.isfile(jar):
                    size_mb = round(os.path.getsize(jar) / 1024 / 1024)

                next_id = max((c.get("id", 0) for c in existing), default=0) + 1
                name = os.path.splitext(os.path.basename(jar))[0]

                entry = {
                    "client_type": client_type,
                    "created_at": datetime.now(timezone.utc).isoformat(),
                    "downloads": 0,
                    "filename": filename,
                    "id": next_id,
                    "launches": 0,
                    "main_class": MAIN_CLASSES.get(client_type, MAIN_CLASSES["default"]),
                    "md5_hash": md5,
                    "name": name,
                    "show": True,
                    "size": size_mb,
                    "version": version,
                    "working": True,
                }

                if client_type == "default" and viaversion:
                    entry["viaversion"] = viaversion

                if client_type == "fabric":
                    deps = []
                    fabric_api = data.get("fabric_api")
                    if fabric_api and fabric_api.get("md5_hash"):
                        deps.append({
                            "md5_hash": fabric_api["md5_hash"],
                            "name": fabric_api["name"],
                            "size": fabric_api.get("size", 0),
                        })
                    local = scan_local_deps(cdn_root)
                    local_other = local.get("other", {})
                    if "kotlin" in flags:
                        dep = _find_dep(local_other, "kotlin")
                        if dep: deps.append(dep)
                        else: deps.append(KOTLIN_DEP)
                    if "satin" in flags:
                        dep = _find_dep(local_other, "satin")
                        if dep: deps.append(dep)
                        else: deps.append(SATIN_DEP)
                    if "sodium" in flags:
                        dep = _find_dep(local_other, "sodium")
                        if dep: deps.append(dep)
                        else: deps.append(SODIUM_DEP)
                    if "baritone" in flags:
                        known_names = {d["name"] for d in BARITONE_DEPS.get(version, [])}
                        local_baritone = [v for k, v in local_other.items() if "baritone" in k.lower() and v["name"] in known_names]
                        if local_baritone:
                            for dep in local_baritone:
                                deps.append(dep)
                        elif version in BARITONE_DEPS:
                            for dep in BARITONE_DEPS[version]:
                                deps.append(dep)
                    entry["dependencies"] = deps
                elif client_type == "forge":
                    entry["dependencies"] = []

                existing.insert(0, entry)
                os.makedirs(os.path.dirname(json_path), exist_ok=True)
                with open(json_path, "w", encoding="utf-8") as f:
                    json.dump(existing, f, indent=2, ensure_ascii=False)

                self._json(200, {"name": name, "id": next_id, "md5": md5, "size": size_mb})
            else:
                self.send_error(404)
        except Exception as e:
            self._json(500, {"error": str(e)})


def main():
    port = 8765
    server = HTTPServer(("127.0.0.1", port), Handler)
    url = f"http://127.0.0.1:{port}"
    print(f"Starting GUI at {url}")
    print(f"CDN root: {CDN_ROOT}")

    webbrowser.open(url)

    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nStopped.")
        server.server_close()


if __name__ == "__main__":
    main()

#!/usr/bin/env python3
"""scripts_gui.py – Web GUI for CollapseLoader scripts (md5 + new_client)

Opens a browser with clickable interface.
No dependencies – uses only Python stdlib.

Usage:
    python3 scripts/scripts_gui.py
"""

import hashlib
import json
import os
import re
import webbrowser
from datetime import datetime, timezone
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path
from urllib.parse import parse_qs, urlparse

CDN_ROOT = os.environ.get("CDN_ROOT", "/media/w1xced/disk/collapsecdn")
FALLBACK_VERSIONS: dict[str, list[str]] = {
    "default": ["1.16.5"],
    "fabric": ["1.21.4", "1.21.8", "1.21.11"],
    "forge": ["1.8.9"],
}


def _sort_versions(versions: list[str]) -> list[str]:
    def to_num(v: str) -> int:
        parts = v.split(".")
        return sum(int(p) * (1000 ** (2 - i)) for i, p in enumerate(parts))
    return sorted(versions, key=to_num)


def scan_cdn_client_versions(cdn_root: str) -> dict[str, list[str]]:
    """Scan local CDN misc/minecraft-versions/ to find available MC versions per type."""
    result: dict[str, list[str]] = {t: [] for t in FALLBACK_VERSIONS}
    mv_dir = os.path.join(cdn_root, "misc", "minecraft-versions")
    if not os.path.isdir(mv_dir):
        return result
    for client_type in ["fabric", "forge"]:
        versions: set[str] = set()
        for fname in os.listdir(mv_dir):
            if not fname.endswith(".jar"):
                continue
            m = re.match(r"^" + re.escape(client_type) + r"_(.+)\.jar$", fname)
            if m:
                versions.add(m.group(1))
        result[client_type] = _sort_versions(list(versions)) if versions else FALLBACK_VERSIONS.get(client_type, [])
    return result

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
FILENAMES = {"default": "clients.json", "fabric": "fabric-clients.json", "forge": "forge-clients.json"}


def compute_md5(filepath):
    h = hashlib.md5()
    with open(filepath, "rb") as f:
        for chunk in iter(lambda: f.read(8192), b""):
            h.update(chunk)
    return h.hexdigest()


import re

def _find_dep(local_other: dict, keyword: str) -> dict | None:
    """Find a dep by keyword in local scanned deps."""
    for fname, info in local_other.items():
        if keyword.lower() in fname.lower():
            return {"md5_hash": info["md5_hash"], "name": info["name"], "size": info["size"]}
    return None


FABRIC_API_RE = re.compile(r"^fabric-api-([0-9.]+\+\d+\.\d+\.\d+)\.jar$")
OTHER_DEP_RE = re.compile(r"^(.+)\.jar$")


def _find_file_by_name(name: str) -> str | None:
    """Search CDN for a file by name in common locations."""
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
    """Scan {cdn_root}/clients/fabric/deps/jars/ and return parsed deps with MD5."""
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


HTML = r"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>CollapseLoader Scripts</title>
<style>
  :root {
    --bg: #1a1a2e; --surface: #16213e; --border: #0f3460;
    --primary: #e94560; --primary-hover: #ff6b81;
    --text: #eee; --text-dim: #999; --green: #2ecc71; --red: #e74c3c;
  }
  * { box-sizing: border-box; margin: 0; padding: 0; }
  body { font-family: 'Segoe UI', system-ui, sans-serif; background: var(--bg); color: var(--text); min-height: 100vh; display: flex; justify-content: center; padding: 32px 16px; }
  .container { width: 100%; max-width: 560px; }
  h1 { font-size: 1.5rem; margin-bottom: 24px; text-align: center; color: var(--primary); }
  .tabs { display: flex; gap: 4px; margin-bottom: 0; }
  .tab { flex: 1; padding: 12px; text-align: center; background: var(--surface); border: 1px solid var(--border); border-bottom: none; border-radius: 10px 10px 0 0; cursor: pointer; font-size: 0.95rem; color: var(--text-dim); transition: all 0.2s; }
  .tab.active { background: var(--border); color: var(--text); font-weight: 600; }
  .tab:hover:not(.active) { background: rgba(15,52,96,0.5); }
  .panel { display: none; background: var(--surface); border: 1px solid var(--border); border-radius: 0 0 12px 12px; padding: 24px; }
  .panel.active { display: block; }
  label { display: block; font-size: 0.85rem; color: var(--text-dim); margin-bottom: 4px; margin-top: 14px; }
  label:first-child { margin-top: 0; }
  input[type="text"], select { width: 100%; padding: 10px 12px; background: var(--bg); border: 1px solid var(--border); border-radius: 8px; color: var(--text); font-size: 0.95rem; outline: none; transition: border 0.2s; }
  input[type="text"]:focus, select:focus { border-color: var(--primary); }
  select { cursor: pointer; appearance: none; background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' fill='%23999'%3E%3Cpath d='M6 8L1 3h10z'/%3E%3C/svg%3E"); background-repeat: no-repeat; background-position: right 12px center; }
  .file-row { display: flex; gap: 8px; }
  .file-row input { flex: 1; }
  .btn { padding: 10px 20px; border: none; border-radius: 8px; font-size: 0.95rem; cursor: pointer; transition: all 0.2s; font-weight: 600; }
  .btn-primary { background: var(--primary); color: #fff; width: 100%; margin-top: 20px; padding: 12px; font-size: 1rem; }
  .btn-primary:hover { background: var(--primary-hover); transform: translateY(-1px); }
  .btn-secondary { background: var(--border); color: var(--text); }
  .btn-secondary:hover { background: #1a4a8a; }
  .btn:active { transform: scale(0.98); }
  .flags { display: flex; gap: 16px; margin-top: 10px; flex-wrap: wrap; }
  .flag { display: flex; align-items: center; gap: 6px; cursor: pointer; }
  .flag input { accent-color: var(--primary); width: 16px; height: 16px; cursor: pointer; }
  .result { margin-top: 16px; padding: 12px; border-radius: 8px; font-family: 'Consolas', monospace; font-size: 0.9rem; word-break: break-all; display: none; }
  .result.ok { display: block; background: rgba(46,204,113,0.15); border: 1px solid var(--green); color: var(--green); }
  .result.err { display: block; background: rgba(231,76,60,0.15); border: 1px solid var(--red); color: var(--red); }
  .result.info { display: block; background: rgba(233,69,96,0.1); border: 1px solid var(--primary); color: var(--text-dim); }
  .spinner { display: none; text-align: center; margin-top: 12px; color: var(--text-dim); }
  .spinner.on { display: block; }
  .sep { border: none; border-top: 1px solid var(--border); margin: 16px 0 0; }
  .flags-disabled input[type="checkbox"] { pointer-events: none; opacity: 0.3; }
  .flags-disabled .flag { pointer-events: none; opacity: 0.3; }
</style>
</head>
<body>
<div class="container">
  <h1>CollapseLoader Scripts</h1>

  <div class="tabs">
    <div class="tab active" onclick="switchTab('md5')">MD5 Hash</div>
    <div class="tab" onclick="switchTab('client')">New Client</div>
  </div>

  <!-- MD5 Panel -->
  <div id="panel-md5" class="panel active">
    <label>JAR file</label>
    <div class="file-row">
      <input type="text" id="md5-path" placeholder="Type full path or Browse">
      <button class="btn btn-secondary" onclick="pickMd5()">Browse (name only)</button>
    </div>
    <div id="md5-result" class="result"></div>
    <div id="md5-spinner" class="spinner">Computing...</div>
    <button class="btn btn-primary" onclick="computeMd5()">Compute MD5</button>
  </div>

  <!-- Client Panel -->
  <div id="panel-client" class="panel">
    <label>JAR file</label>
    <div class="file-row">
      <input type="text" id="c-jar" placeholder="Select .jar file">
      <button class="btn btn-secondary" onclick="pickJar()">Browse</button>
    </div>

    <label>Version</label>
    <select id="c-version" onchange="populateFabricApi()">
      <option value="" disabled selected>Loading versions...</option>
    </select>

    <label>Client type</label>
    <select id="c-type" onchange="onTypeChange()">
      <option value="fabric" selected>fabric</option>
      <option value="forge">forge</option>
      <option value="default">default</option>
    </select>

    <hr class="sep">
    <div id="c-deps-section">
      <label>Fabric API version</label>
      <select id="c-fabric-api">
        <option value="" disabled selected>Loading deps...</option>
      </select>

      <label>Extra dependencies</label>
      <div id="c-flags" class="flags">
        <label class="flag"><input type="checkbox" value="kotlin"> kotlin</label>
        <label class="flag"><input type="checkbox" value="satin"> satin</label>
        <label class="flag"><input type="checkbox" value="sodium"> sodium</label>
        <label class="flag"><input type="checkbox" value="baritone"> baritone</label>
      </div>

      <div id="c-deps-info" class="result info" style="display:block; margin-top:10px;"></div>
    </div>

    <label>CDN root</label>
    <input type="text" id="c-cdn" value="CDN_ROOT_PLACEHOLDER">

    <div id="c-result" class="result"></div>
    <div id="c-spinner" class="spinner">Processing...</div>
    <button class="btn btn-primary" onclick="addClient()">Add Client</button>
  </div>
</div>

<script>
/*__INIT_DATA__*/
const $ = id => document.getElementById(id);

function switchTab(name) {
  document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
  document.querySelectorAll('.panel').forEach(p => p.classList.remove('active'));
  document.querySelector(`.tab:${name === 'md5' ? 'first-child' : 'last-child'}`).classList.add('active');
  $('panel-' + name).classList.add('active');
}

function onTypeChange() {
  const isFabric = $('c-type').value === 'fabric';
  $('c-deps-section').style.display = isFabric ? 'block' : 'none';
  populateVersions($('c-type').value);
  if (isFabric) populateFabricApi();
}

function pickMd5() {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.jar';
  input.onchange = () => {
    if (input.files[0]) {
      $('md5-path').value = input.files[0].name;
      $('md5-path').dataset.path = input.files[0].name;
    }
  };
  input.click();
}

function pickJar() {
  const input = document.createElement('input');
  input.type = 'file';
  input.accept = '.jar';
  input.onchange = () => {
    if (input.files[0]) $('c-jar').value = input.files[0].name;
  };
  input.click();
}

async function computeMd5() {
  const path = $('md5-path').value;
  if (!path || path === 'No file selected') { showResult('md5', 'err', 'Select a file first'); return; }
  $('md5-spinner').classList.add('on');
  try {
    const res = await fetch('/api/md5', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify({path}) });
    const data = await res.json();
    $('md5-spinner').classList.remove('on');
    if (data.error) showResult('md5', 'err', data.error);
    else showResult('md5', 'ok', `MD5(${data.name}) = ${data.hash}`);
  } catch (e) {
    $('md5-spinner').classList.remove('on');
    showResult('md5', 'err', 'Request failed');
  }
}

async function addClient() {
  const jar = $('c-jar').value.trim();
  if (!jar) { showResult('c', 'err', 'Select a JAR file'); return; }

  const flags = [];
  $('c-flags').querySelectorAll('input:checked').forEach(cb => flags.push(cb.value));

  const fabricApiSelect = $('c-fabric-api');
  const fabricApiOpt = fabricApiSelect.selectedOptions[0];

  const body = {
    jar,
    version: $('c-version').value,
    client_type: $('c-type').value,
    flags,
    cdn_root: $('c-cdn').value.trim(),
    fabric_api: fabricApiOpt ? {
      filename: fabricApiSelect.value,
      md5_hash: fabricApiOpt.dataset.md5,
      name: fabricApiOpt.dataset.name,
      size: parseInt(fabricApiOpt.dataset.size) || 0
    } : null
  };

  $('c-spinner').classList.add('on');
  try {
    const res = await fetch('/api/client', { method: 'POST', headers: {'Content-Type': 'application/json'}, body: JSON.stringify(body) });
    const data = await res.json();
    $('c-spinner').classList.remove('on');
    if (data.error) showResult('c', 'err', data.error);
    else showResult('c', 'ok', `Added "${data.name}" id=${data.id} | ${data.md5.slice(0, 12)}... | ${data.size}MB`);
  } catch (e) {
    $('c-spinner').classList.remove('on');
    showResult('c', 'err', 'Request failed');
  }
}

function showResult(prefix, type, msg) {
  const el = $(prefix + '-result');
  el.className = 'result ' + type;
  el.textContent = msg;
}

let cdnVersions = {};
let localDeps = { fabric_api: {}, other: {} };

try {
  cdnVersions = window.__INIT.versions || {};
  localDeps = window.__INIT.deps || { fabric_api: {}, other: {} };
} catch(e) {}

onTypeChange();
populateFabricApi();
showDepsInfo();

function populateVersions(type) {
  const select = $('c-version');
  const versions = cdnVersions[type] || [];
  select.innerHTML = '';
  if (versions.length === 0) {
    select.innerHTML = '<option value="" disabled selected>No versions</option>';
    return;
  }
  versions.forEach((v, i) => {
    const opt = document.createElement('option');
    opt.value = v;
    opt.textContent = v;
    if (i === 0) opt.selected = true;
    select.appendChild(opt);
  });
  populateFabricApi();
}

function populateFabricApi() {
  const select = $('c-fabric-api');
  const mcVer = $('c-version').value;
  select.innerHTML = '';
  const entries = Object.entries(localDeps.fabric_api)
    .filter(([_, v]) => !mcVer || v.mc_version === mcVer)
    .sort((a, b) => a[1].api_version.localeCompare(b[1].api_version, undefined, { numeric: true }));
  if (entries.length === 0) {
    select.innerHTML = '<option value="" disabled>No fabric-api for this version</option>';
    return;
  }
  entries.forEach(([fname, info]) => {
    const opt = document.createElement('option');
    opt.value = fname;
    opt.textContent = `${info.api_version} (MC ${info.mc_version})`;
    opt.dataset.md5 = info.md5_hash;
    opt.dataset.size = info.size;
    opt.dataset.name = info.name;
    select.appendChild(opt);
  });
}

function showDepsInfo() {
  const other = Object.values(localDeps.other);
  const el = $('c-deps-info');
  if (other.length === 0) {
    el.textContent = 'No extra deps found in CDN.';
    return;
  }
  el.textContent = `Found ${other.length} extra deps: ${other.map(d => d.name).join(', ')}`;
}
</script>
</body>
</html>"""


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
            payload = json.dumps({"versions": versions, "deps": deps}).replace("\\", "\\\\").replace("'", "\\'")
            body = HTML.replace("CDN_ROOT_PLACEHOLDER", CDN_ROOT).replace("/*__INIT_DATA__*/", f"window.__INIT={payload};").encode()
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
                        dep = _find_dep(local_other, "baritone")
                        if dep: deps.append(dep)
                        elif version in BARITONE_DEPS:
                            deps.append(BARITONE_DEPS[version])
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
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nStopped.")
        server.server_close()


if __name__ == "__main__":
    main()

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
import sys
import webbrowser
from datetime import datetime, timezone
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path
from urllib.parse import parse_qs, urlparse

CDN_ROOT = os.environ.get("CDN_ROOT", "/media/w1xced/Disk/hf-cdn")

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
      <input type="text" id="md5-path" placeholder="No file selected" readonly>
      <button class="btn btn-secondary" onclick="pickMd5()">Browse</button>
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
    <select id="c-version">
      <option value="1.21.4">1.21.4</option>
      <option value="1.21.8">1.21.8</option>
      <option value="1.21.10">1.21.10</option>
      <option value="1.21.11" selected>1.21.11</option>
    </select>

    <label>Client type</label>
    <select id="c-type" onchange="onTypeChange()">
      <option value="fabric" selected>fabric</option>
      <option value="forge">forge</option>
      <option value="default">default</option>
    </select>

    <hr class="sep">
    <label>Extra dependencies (fabric only)</label>
    <div id="c-flags" class="flags">
      <label class="flag"><input type="checkbox" value="kotlin"> kotlin</label>
      <label class="flag"><input type="checkbox" value="satin"> satin</label>
      <label class="flag"><input type="checkbox" value="sodium"> sodium</label>
      <label class="flag"><input type="checkbox" value="baritone"> baritone</label>
    </div>

    <label>CDN root</label>
    <input type="text" id="c-cdn" value="CDN_ROOT_PLACEHOLDER">

    <div id="c-result" class="result"></div>
    <div id="c-spinner" class="spinner">Processing...</div>
    <button class="btn btn-primary" onclick="addClient()">Add Client</button>
  </div>
</div>

<script>
const $ = id => document.getElementById(id);

function switchTab(name) {
  document.querySelectorAll('.tab').forEach(t => t.classList.remove('active'));
  document.querySelectorAll('.panel').forEach(p => p.classList.remove('active'));
  document.querySelector(`.tab:${name === 'md5' ? 'first-child' : 'last-child'}`).classList.add('active');
  $('panel-' + name).classList.add('active');
}

function onTypeChange() {
  const isFabric = $('c-type').value === 'fabric';
  const flags = $('c-flags');
  flags.classList.toggle('flags-disabled', !isFabric);
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

  const body = {
    jar,
    version: $('c-version').value,
    client_type: $('c-type').value,
    flags,
    cdn_root: $('c-cdn').value.trim()
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

onTypeChange();
</script>
</body>
</html>"""


class Handler(BaseHTTPRequestHandler):
    def log_message(self, format, *args):
        pass

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
            body = HTML.replace("CDN_ROOT_PLACEHOLDER", CDN_ROOT).encode()
            self.send_response(200)
            self.send_header("Content-Type", "text/html; charset=utf-8")
            self.send_header("Content-Length", str(len(body)))
            self.end_headers()
            self.wfile.write(body)
        else:
            self.send_error(404)

    def do_POST(self):
        try:
            if self.path == "/api/md5":
                data = self._read_body()
                filepath = data.get("path", "").strip()
                if not filepath or not os.path.isfile(filepath):
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
                    deps = list(FABRIC_BASE_DEPS.get(version, []))
                    if "kotlin" in flags: deps.append(KOTLIN_DEP)
                    if "satin" in flags: deps.append(SATIN_DEP)
                    if "sodium" in flags: deps.append(SODIUM_DEP)
                    if "baritone" in flags and version in BARITONE_DEPS:
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
    webbrowser.open(url)
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nStopped.")
        server.server_close()


if __name__ == "__main__":
    main()

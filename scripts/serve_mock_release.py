#!/usr/bin/env python3
"""Simple mock GitHub Releases endpoint for local testing.
Run: python scripts/serve_mock_release.py
It serves the file at /repos/<owner>/<repo>/releases/latest
"""
from http.server import BaseHTTPRequestHandler, HTTPServer
import json
import os

PORT = 8000
MOCK_FILE = os.path.join(os.path.dirname(__file__), "..", "docs", "versioning", "mock_release.json")


class Handler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path.endswith("/releases/latest"):
            try:
                with open(MOCK_FILE, "r", encoding="utf-8") as f:
                    obj = json.load(f)

                body_val = obj.get("body")
                if isinstance(body_val, str) and body_val.startswith("#file:"):
                    rel_path = body_val[len("#file:") :]
                    if not os.path.isabs(rel_path):
                        repo_root = os.path.abspath(
                            os.path.join(os.path.dirname(__file__), "..")
                        )
                        candidate = (
                            os.path.join(repo_root, "docs", rel_path)
                            if os.path.dirname(rel_path) == ""
                            else os.path.join(repo_root, rel_path)
                        )
                    else:
                        candidate = rel_path

                    try:
                        with open(candidate, "r", encoding="utf-8") as bf:
                            obj["body"] = bf.read()
                    except Exception:
                        pass

                data = json.dumps(obj, ensure_ascii=False).encode("utf-8")
                self.send_response(200)
                self.send_header("Content-Type", "application/json; charset=utf-8")
                self.end_headers()
                self.wfile.write(data)
            except Exception as e:
                self.send_response(500)
                self.end_headers()
                self.wfile.write(str(e).encode())
        else:
            self.send_response(404)
            self.end_headers()


if __name__ == "__main__":
    server = HTTPServer(("127.0.0.1", PORT), Handler)
    print(f"Mock GitHub release server running at http://127.0.0.1:{PORT}")
    print("Serving /repos/<owner>/<repo>/releases/latest -> docs/mock_release.json")
    try:
        server.serve_forever()
    except KeyboardInterrupt:
        print("\nShutting down")
        server.server_close()

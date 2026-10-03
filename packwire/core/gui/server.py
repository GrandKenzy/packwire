import json
import mimetypes
from http.server import HTTPServer, BaseHTTPRequestHandler
from pathlib import Path
from typing import Optional
import sys

from packwire.core.gui.bridge import GuiBridge


def get_web_dir() -> Path:
    """Resolve web assets directory, supporting both source and PyInstaller frozen executables."""
    if getattr(sys, "frozen", False) and hasattr(sys, "_MEIPASS"):
        candidate = Path(sys._MEIPASS) / "packwire" / "core" / "gui" / "web"
        if candidate.exists():
            return candidate
        candidate_root = Path(sys._MEIPASS) / "web"
        if candidate_root.exists():
            return candidate_root
    return Path(__file__).resolve().parent / "web"


WEB_DIR = get_web_dir()


class PackwireHttpHandler(BaseHTTPRequestHandler):
    bridge = GuiBridge()

    def do_GET(self):
        """Serve static frontend files or handle read-only API requests."""
        if self.path.startswith("/api/"):
            method_name = self.path[len("/api/"):].split("?")[0]
            self._dispatch_api(method_name, [])
            return

        req_path = self.path.split("?")[0].lstrip("/")
        if not req_path:
            req_path = "index.html"

        file_path = (WEB_DIR / req_path).resolve()

        # Prevent directory traversal
        if not str(file_path).startswith(str(WEB_DIR)) or not file_path.is_file():
            self.send_error(404, "File Not Found")
            return

        mime_type, _ = mimetypes.guess_type(str(file_path))
        if not mime_type:
            mime_type = "application/octet-stream"

        try:
            content = file_path.read_bytes()
            self.send_response(200)
            self.send_header("Content-Type", f"{mime_type}; charset=utf-8")
            self.send_header("Content-Length", str(len(content)))
            self.end_headers()
            self.wfile.write(content)
        except Exception as e:
            self.send_error(500, f"Error reading file: {e}")

    def do_POST(self):
        """Handle JSON API calls from frontend."""
        if not self.path.startswith("/api/"):
            self.send_error(404, "Unknown API route")
            return

        method_name = self.path[len("/api/"):].split("?")[0]
        content_len = int(self.headers.get("Content-Length", 0))
        body = self.rfile.read(content_len).decode("utf-8") if content_len > 0 else "{}"

        try:
            payload = json.loads(body) if body else {}
            args = payload.get("args", [])
        except Exception:
            args = []

        self._dispatch_api(method_name, args)

    def _dispatch_api(self, method_name: str, args: list):
        handler = getattr(self.bridge, method_name, None)
        if not handler or not callable(handler):
            self.send_response(400)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"error": f"Method '{method_name}' not found"}).encode("utf-8"))
            return

        try:
            result = handler(*args)
            response_bytes = json.dumps(result, ensure_ascii=False).encode("utf-8")
            self.send_response(200)
            self.send_header("Content-Type", "application/json; charset=utf-8")
            self.send_header("Content-Length", str(len(response_bytes)))
            self.end_headers()
            self.wfile.write(response_bytes)
        except Exception as e:
            self.send_response(500)
            self.send_header("Content-Type", "application/json")
            self.end_headers()
            self.wfile.write(json.dumps({"error": str(e)}).encode("utf-8"))

    def log_message(self, format, *args):
        # Silence default terminal request spam
        pass


def run_local_server(port: int = 5050) -> tuple:
    """Run server in background daemon thread. Returns (server_instance, url)."""
    server = HTTPServer(("127.0.0.1", port), PackwireHttpHandler)
    url = f"http://127.0.0.1:{port}"
    thread = threading.Thread(target=server.serve_forever, daemon=True)
    thread.start()
    return server, url

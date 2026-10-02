#!/usr/bin/env python3
# ============================================================
# MTAALAMU SMART — preview server (stdlib pekee, hakuna deps)
# Inahudumia web-r/www (RAMANI 3D, nav, previews) + data/ (JSON
# halisi za mradi) ili ramani-3d.html isome devices/hazards
# hata bila R/Shiny.
# PORT (Freebuff inayongeza) au 8000; bind 0.0.0.0.
# ============================================================
import http.server
import os
import socketserver
from urllib.parse import urlparse

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
PORT = int(os.environ.get("PORT", "8000"))


class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=ROOT, **kwargs)

    def do_GET(self):
        path = urlparse(self.path).path
        if path in ("/", "/index.html"):
            self.send_response(302)
            self.send_header("Location", "/web-r/www/index.html")
            self.end_headers()
            return
        super().do_GET()


socketserver.ThreadingTCPServer.allow_reuse_address = True

if __name__ == "__main__":
    with socketserver.ThreadingTCPServer(("0.0.0.0", PORT), Handler) as httpd:
        print(
            "MTAALAMU preview: http://0.0.0.0:%d/web-r/www/index.html"
            % PORT,
            flush=True,
        )
        httpd.serve_forever()

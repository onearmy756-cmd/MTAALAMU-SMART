"""MTAALAMU SMART — IPC + R bridge.

IPC: server ya unix socket (Linux/macOS) / named pipe helper (Windows) —
UI ya hermes au process nyingine inaweza kuuliza solver bila HTTP.
R: uchambuzi halisi (Rscript) kwa data ya usage/diagnostics — kama R ipo.
"""
import json
import os
import shutil
import subprocess
import threading
from pathlib import Path

SOCK_PATH = Path(os.environ.get("MTECH_SOCK", "/tmp/mtaalamu.sock"))


# ------------------------------------------------------------------ IPC server
def start_ipc(handler) -> threading.Thread | None:
    """Anzisha IPC (unix socket). handler(json_obj) -> json_obj (dict)."""
    if os.name == "nt":
        return None  # Windows: HTTP API (server.py) ndiyo IPC — named pipes zinakuja
    import socket
    try:
        if SOCK_PATH.exists():
            SOCK_PATH.unlink()
    except OSError:
        pass
    srv = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    srv.bind(str(SOCK_PATH))
    os.chmod(SOCK_PATH, 0o660)
    srv.listen(8)

    def serve():
        while True:
            try:
                conn, _ = srv.accept()
            except OSError:
                break
            with conn:
                try:
                    data = conn.recv(65536).decode()
                    req = json.loads(data or "{}")
                    resp = handler(req)
                except (json.JSONDecodeError, ValueError) as e:
                    resp = {"error": str(e)}
                conn.sendall(json.dumps(resp, ensure_ascii=False).encode()[:200000])

    th = threading.Thread(target=serve, daemon=True)
    th.start()
    return th


def ipc_call(payload: dict) -> dict | None:
    """Mteja wa IPC (kwa tests/tools)."""
    if os.name == "nt" or not SOCK_PATH.exists():
        return None
    import socket
    with socket.socket(socket.AF_UNIX, socket.SOCK_STREAM) as c:
        c.connect(str(SOCK_PATH))
        c.sendall(json.dumps(payload).encode())
        return json.loads(c.recv(200000).decode())


# ------------------------------------------------------------------ R bridge
def r_available() -> bool:
    return shutil.which("Rscript") is not None


def r_analyze(script: str, data_json: str = "{}") -> dict:
    """Uchambuzi halisi kwa R (Rscript). Script inapewa `data` kama JSON string."""
    if not r_available():
        return {"ok": False, "reason": "R haipo (saki: R base) — Python analysis inaendelea"}
    full = f'data <- \'{data_json}\'\n{script}\n'
    try:
        p = subprocess.run(["Rscript", "-e", full], capture_output=True, text=True, timeout=60)
        return {"ok": p.returncode == 0, "out": p.stdout[:4000], "err": p.stderr[:1000]}
    except (OSError, subprocess.TimeoutExpired) as e:
        return {"ok": False, "reason": str(e)}


R_USAGE_REPORT = '''
# MTAALAMU: ripoti ya usage (credits kwa op) — R stats halisi
d <- tryCatch(jsonlite::fromJSON(data), error=function(e) NULL)
if (is.null(d)) { cat("JSON haiwezi kusomwa"); quit(status=1) }
ops <- d$ops
if (length(ops) == 0) { cat("Hakuna matumizi bado"); quit(status=0) }
agg <- aggregate(credits ~ op, data = ops, FUN = sum)
agg <- agg[order(-agg$credits), ]
cat("=== RIPOTI YA MATUMIZI (R) ===\n")
print(head(agg, 10), row.names = FALSE)
cat(sprintf("\nJumla: %d credits kwenye ops %d\n", sum(ops$credits), length(ops$op)))
'''

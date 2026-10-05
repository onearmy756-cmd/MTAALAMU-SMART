"""MTAALAMU SMART — FS OPS (filesystem + install + memory — kwa HITL/admin).

Uwezo halisi ndani ya mfumo (kila kitu kinachukuliwa kama kazi ya OS):
  • Tengeneza FOLDER/DIRECTORY  — mkdir -p (salama, recursive)
  • KUPATA vitu                 — find kwa pattern (glob), read file
  • DOWNLOAD                    — pakua app/file kutoka URL kwenda dest
  • MEMORY ACCESS               — /proc/meminfo (Linux) · PowerShell (Win) · sysctl (mac)
  • PARTITION                   — tambua/unda (ADMIN + uthibitisho wazi "NDIYO")
  • INSTALL app/package         — apt/brew/winget/pip/npm (per package manager)

Tumia (CLI):
  mtaalamu fs mkdir ~/MTECH/ripoti
  mtaalamu fs find "*.pdf" ~
  mtaalamu fs read /etc/os-release
  mtaalamu fs download https://... ~/Downloads/app.deb
  mtaalamu fs memory
  mtaalamu fs partitions
  mtaalamu fs partition-create /dev/sdb 8G     # ADMIN + uthibitisho
  mtaalamu fs install htop                     # apt/brew/winget

API (mtaalamu serve):
  GET  /api/fs/memory                    → memory access halisi
  GET  /api/fs/find?pattern=*&dir=~      → pata files/folders
  GET  /api/fs/read?path=/etc/os-release → somesha file
  POST /api/fs/mkdir      {"path": "..."}
  POST /api/fs/download   {"url": "...", "dest": "..."}
  POST /api/fs/install    {"pkg": "htop"}
  POST /api/fs/partition  {"disk": "/dev/sdb", "size": "8G", "confirm": "NDIYO"}  (ADMIN)
"""
import fnmatch
import os
import platform
import shutil
import subprocess
import urllib.request
from pathlib import Path

FAMILY = "windows" if os.name == "nt" else ("macos" if platform.system() == "Darwin" else "linux")
HOME = Path.home()
ROOT = Path(__file__).resolve().parents[1]

# ---------------------------------------------------------------- helpers
def _run(cmd: str, timeout: int = 120) -> tuple[int, str]:
    try:
        p = subprocess.run(cmd, shell=True, capture_output=True, text=True, timeout=timeout)
        return p.returncode, (p.stdout or p.stderr or "").strip()[:4000]
    except (OSError, subprocess.TimeoutExpired) as e:
        return -1, str(e)[:200]


def _safe_path(path: str) -> Path:
    """Panua ~ na itaje absolute — HAKUNA kuandika nje ya ruhusa za mtumiaji."""
    p = Path(os.path.expanduser(str(path or "")))
    return p.expanduser().resolve()


# ---------------------------------------------------------------- folders/dirs
def mkdir(path: str) -> dict:
    """Tengeneza folder/directory (recursive — parents zote zinatumikwa)."""
    p = _safe_path(path)
    if not str(p) or str(p) == "/":
        return {"ok": False, "error": "njia si sahihi"}
    try:
        existed = p.exists()
        p.mkdir(parents=True, exist_ok=True)
        return {"ok": True, "path": str(p), "created": not existed,
                "note": "ilikuwepo tayari" if existed else "folder imetengenezwa ✔"}
    except OSError as e:
        return {"ok": False, "error": f"{type(e).__name__}: {e}", "path": str(p)}


# ---------------------------------------------------------------- pata/find/read
def find(pattern: str, directory: str = "~", limit: int = 100) -> dict:
    """Pata files/folders kwa pattern (glob-style: *.pdf, mtech*, n.k.)."""
    base = _safe_path(directory)
    if not base.exists():
        return {"ok": False, "error": f"folda haipo: {base}"}
    hits: list[str] = []
    for root, dirs, files in os.walk(base):
        # epuka virtual filesystems kubwa
        dirs[:] = [d for d in dirs if d not in (".git", "node_modules", "__pycache__", "target", ".venv", "proc", "sys")]
        for name in files + dirs:
            if fnmatch.fnmatch(name.lower(), pattern.lower()):
                hits.append(str(Path(root) / name))
                if len(hits) >= limit:
                    return {"ok": True, "pattern": pattern, "dir": str(base),
                            "count": len(hits), "truncated": True, "results": hits}
    return {"ok": True, "pattern": pattern, "dir": str(base),
            "count": len(hits), "results": hits}


def read(path: str, max_bytes: int = 60000) -> dict:
    """Somesha file (text) — memory-safe (max 60KB)."""
    p = _safe_path(path)
    if not p.exists():
        return {"ok": False, "error": f"haipo: {p}"}
    if p.is_dir():
        return {"ok": False, "error": "ni directory — tumia find", "path": str(p)}
    try:
        data = p.read_bytes()[:max_bytes]
        return {"ok": True, "path": str(p), "size": p.stat().st_size,
                "content": data.decode(errors="replace")}
    except OSError as e:
        return {"ok": False, "error": str(e)}


# ---------------------------------------------------------------- download
def download(url: str, dest: str = "") -> dict:
    """Pakua app/file kutoka URL (mf: .deb, .exe, .zip) — dest ya default: ~/Downloads."""
    if not (url or "").lower().startswith(("http://", "https://")):
        return {"ok": False, "error": "URL lazima iwe http(s)://"}
    d = _safe_path(dest) if dest else (_safe_path("~/Downloads") / url.rstrip("/").split("?")[0].split("/")[-1])
    if d.is_dir():
        d = d / url.rstrip("/").split("?")[0].split("/")[-1]
    d.parent.mkdir(parents=True, exist_ok=True)
    try:
        def _hook(n, bs, total):
            if total > 0 and (n * bs) % (10 * 1024 * 1024) < bs:
                print(f"  ▸ {min(n * bs, total) // (1024 * 1024)}MB / {total // (1024 * 1024)}MB", flush=True)
        urllib.request.urlretrieve(url, d, reporthook=_hook)
        return {"ok": True, "url": url, "dest": str(d), "size": d.stat().st_size}
    except (OSError, ValueError) as e:
        return {"ok": False, "error": f"{type(e).__name__}: {e}", "url": url}


# ---------------------------------------------------------------- MEMORY ACCESS
def memory() -> dict:
    """Memory halisi ya mfumo (total/used/free — bytes au MB)."""
    out: dict = {"family": FAMILY}
    if FAMILY == "linux":
        info: dict = {}
        try:
            for line in Path("/proc/meminfo").read_text().splitlines():
                k, v = line.split(":", 1)
                info[k.strip()] = v.strip()
        except OSError as e:
            return {"ok": False, "error": str(e)}
        out["ok"] = True
        out["meminfo"] = {k: info[k] for k in ("MemTotal", "MemFree", "MemAvailable", "Buffers", "Cached", "SwapTotal", "SwapFree") if k in info}
        try:
            total_kb = int(info["MemTotal"].split()[0])
            avail_kb = int(info.get("MemAvailable", "0").split()[0] or 0)
            out["used_pct"] = round(100 * (total_kb - avail_kb) / max(total_kb, 1), 1)
        except (KeyError, ValueError, IndexError):
            pass
        return out
    if FAMILY == "windows":
        code, txt = _run("powershell -NoProfile -Command \"$o=Get-CimInstance Win32_OperatingSystem; [math]::Round($o.TotalVisibleMemorySize/1MB,1); [math]::Round($o.FreePhysicalMemory/1MB,1)\"")
        lines = [l for l in txt.splitlines() if l.strip()]
        if code == 0 and len(lines) >= 2:
            out["ok"] = True
            out["total_gb"] = lines[0]
            out["free_gb"] = lines[1]
            return out
        return {"ok": False, "error": txt[:200]}
    code, txt = _run("sysctl -n hw.memsize")
    if code == 0:
        out["ok"] = True
        out["total_gb"] = round(int(txt.strip() or 0) / (1024 ** 3), 1)
        return out
    return {"ok": False, "error": txt[:200]}


# ---------------------------------------------------------------- PARTITIONS
def partitions() -> dict:
    """Orodha halisi ya partitions (kutoka kali.py — lsblk/Get-Disk/diskutil)."""
    from . import kali
    return kali.partitions()


def partition_create(disk: str, size: str, confirm: str = "", admin_ok: bool = False) -> dict:
    """Unda partition mpya — ADMIN TU + uthibitisho wazi ("NDIYO").

    Linux: parted mkpart + mkfs (ext4). Windows/macOS: tunatoa amri sahihi
    tu (hazina cheti cha root hapa — mwalimo wa amri inarudishwa).
    """
    if not admin_ok:
        return {"ok": False, "error": "ADMIN inahitajika — mtaalamu admin unlock kwanza"}
    if (confirm or "").strip().upper() != "NDIYO":
        return {"ok": False, "error": "uthibitisho: onyesha confirm='NDIYO' kwa uthibitisho wazi"}
    d = str(disk or "").strip()
    if not d.startswith("/dev/"):
        return {"ok": False, "error": "disk lazima iwe /dev/... (mf: /dev/sdb)"}
    if FAMILY == "linux":
        if not shutil.which("parted"):
            return {"ok": False, "error": "parted haipo — sudo apt install parted"}
        code1, out1 = _run(f"sudo parted --script '{d}' mkpart primary ext4 0% '{size or '100%'}'", timeout=120)
        if code1 != 0:
            return {"ok": False, "error": out1[:300], "step": "mkpart"}
        part = f"{d}1" if "nvme" not in d else f"{d}p1"
        code2, out2 = _run(f"sudo mkfs.ext4 -F '{part}'", timeout=300)
        return {"ok": code2 == 0, "disk": d, "partition": part,
                "mkpart": out1[-200:], "mkfs": out2[-200:],
                "note": "mount: sudo mount " + part + " /mnt"}
    return {"ok": False, "error": "OS hii: tumia diskmgmt.msc (Windows) au diskutil (macOS)",
            "hint": f"Windows: New-Partition -DiskNumber N -Size {size or 'Max'}"}


# ---------------------------------------------------------------- INSTALL apps/packages
def _pkg_manager() -> str:
    for pm in ("apt-get", "dnf", "pacman"):
        if shutil.which(pm):
            return pm
    return "apt-get"


def install(pkg: str) -> dict:
    """Sakinisha app/package kwa manager sahihi (apt/brew/winget/pip/npm)."""
    pkg = (pkg or "").strip()
    if not pkg or any(c in pkg for c in ";|&`$><"):
        return {"ok": False, "error": "jina la package si sahihi"}
    # pip? (python: pkg==x.y au path)
    if pkg.startswith(("pip:", "python:")):
        return {**_run2ok(f"{_py()} -m pip install -q {pkg.split(':', 1)[1]}"), "manager": "pip"}
    # npm?
    if pkg.startswith("npm:"):
        return {**_run2ok(f"npm install -g {pkg.split(':', 1)[1]}"), "manager": "npm"}
    if FAMILY == "linux":
        pm = _pkg_manager()
        return {**_run2ok(("sudo " if pm == "apt-get" else "") + f"{pm} install -y {pkg}", timeout=1800), "manager": pm}
    if FAMILY == "macos":
        return {**_run2ok(f"brew install {pkg}", timeout=1800), "manager": "brew"}
    return {**_run2ok(f"winget install -e --id {pkg} --accept-source-agreements --accept-package-agreements", timeout=1800), "manager": "winget"}


def _py() -> str:
    return shutil.which("python3") or shutil.which("python") or "python3"


def _run2ok(cmd: str, timeout: int = 600) -> dict:
    code, out = _run(cmd, timeout)
    return {"ok": code == 0, "output": out[-600:]}


# ---------------------------------------------------------------- CLI
def main() -> None:
    import json
    import sys
    a = sys.argv[1:]
    if not a:
        print(json.dumps({"ops": ["mkdir", "find", "read", "download", "memory",
                                  "partitions", "partition-create", "install"],
                          "api": ["GET /api/fs/memory", "GET /api/fs/find?pattern=&dir=",
                                  "GET /api/fs/read?path=", "POST /api/fs/mkdir",
                                  "POST /api/fs/download", "POST /api/fs/install",
                                  "POST /api/fs/partition (ADMIN)"]},
                         ensure_ascii=False, indent=1))
        return
    op, rest = a[0], a[1:]
    if op == "mkdir" and rest:
        print(json.dumps(mkdir(rest[0]), ensure_ascii=False, indent=1))
    elif op == "find" and rest:
        print(json.dumps(find(rest[0], rest[1] if len(rest) > 1 else "~"), ensure_ascii=False, indent=1)[:4000])
    elif op == "read" and rest:
        print(json.dumps(read(rest[0]), ensure_ascii=False, indent=1)[:6000])
    elif op == "download" and len(rest) >= 1:
        print(json.dumps(download(rest[0], rest[1] if len(rest) > 1 else ""), ensure_ascii=False, indent=1))
    elif op == "memory":
        print(json.dumps(memory(), ensure_ascii=False, indent=1))
    elif op == "partitions":
        print(json.dumps(partitions(), ensure_ascii=False, indent=1, default=str)[:4000])
    elif op == "install" and rest:
        print(json.dumps(install(rest[0]), ensure_ascii=False, indent=1))
    else:
        print("tumia: mtaalamu fs mkdir|find|read|download|memory|partitions|install …")


if __name__ == "__main__":
    main()

"""MTAALAMU SMART — SETUP BINARY (kama hermes agent: pakua & tumia).

LEO NAWEZA:
  A) Kama Python + PyInstaller zipo:
     python3 setup_builder.py            # jenga binary ya OS hii (onefile)
  B) Kama Python HAIPO (Windows fresh):
     MTAALAMU-Setup.cmd / install.bat zitavuta embedded Python rasmi
     (python.org) NDANI ya folder hii, kisha setup inaendelea yenyewe.

INAZALISHA (mtaalamu/dist/):
  MTAALAMU-Setup.exe   (Windows onefile — double-click)
  mtaalamu-linux       (Linux onefile)
  MTAALAMU-macos       (macOS onefile)

  --bundle-runtime     runtime/ + ZIP KAMILI: binary ya Python (onefile —
                       Python NDANI yake) + engine ya Rust (kama ipo) +
                       launchers + README-INSTALL — mtumiaji HATAKIWI
                       kusakinisha Python, R, wala Rust: anafungua ZIP → anarun

Binary ina CLI + server zima (mtaalamu ni stdlib-only). Ollama/Qwen na
C/Rust/R bado ni hiari — setup_all inawashughulikia.
"""
import platform
import subprocess
import sys
import tempfile
import textwrap
from pathlib import Path

HERE = Path(__file__).resolve().parent
DIST = HERE / "dist"
WIN_EMBED_TAG = "3.12.7"


def build_onefile() -> int:
    name = {"windows": "MTAALAMU-Setup.exe", "darwin": "MTAALAMU-macos",
            "linux": "mtaalamu-linux"}.get(platform.system().lower(), "mtaalamu")
    DIST.mkdir(parents=True, exist_ok=True)
    # Entry wrapper: relative imports za pakiti zinahitaji parent package —
    # tunaiita kama "mtaalamu.cli" kwa wrapper ya absolute-import.
    with tempfile.TemporaryDirectory() as td:
        entry = Path(td) / "mtaalamu_entry.py"
        entry.write_text(textwrap.dedent(f"""
            import sys, os
            sys.path.insert(0, {str(HERE)!r})
            from mtaalamu.cli import main
            if __name__ == "__main__":
                main()
        """))
        cmd = [sys.executable, "-m", "PyInstaller", "--onefile", "--name", name,
               "--distpath", str(DIST), "--workpath", str(Path(td) / "work"),
               "--specpath", str(Path(td) / "spec"), "--paths", str(HERE.parent),
               str(entry)]
        print("  $", " ".join(cmd[:2]), "…onefile…")
        rc = subprocess.run(cmd).returncode
    if rc == 0:
        out = DIST / name
        print(f"\n✅ BINARY IMEJENGWA: {out}")
        print(f"   Tumia:  {out.name} register wewe@mail.com INDIVIDUAL DIAMOND")
        print(f"           {out.name} admin unlock")
        print(f"           {out.name} serve          (kisha fungua web-html/mtaalamu-unified.html)")
    return rc


def fetch_embedded_windows(dest: Path) -> Path | None:
    """Vuta Python embeddable rasmi (python.org) — kwa Windows bila Python."""
    import urllib.request
    url = f"https://www.python.org/ftp/python/{WIN_EMBED_TAG}/python-{WIN_EMBED_TAG}-embed-amd64.zip"
    z = dest / f"python-embed-{WIN_EMBED_TAG}.zip"
    print(f"  ▸ inapakua Python embedded (rasmi, {WIN_EMBED_TAG})…")
    try:
        dest.mkdir(parents=True, exist_ok=True)
        urllib.request.urlretrieve(url, z)
    except OSError as e:
        print(f"  ✖ hakuweza kupakua ({e}) — tumia install.bat (winget) au python.org")
        return None
    import zipfile
    with zipfile.ZipFile(z) as zf:
        zf.extractall(dest)
    z.unlink()
    exe = dest / "python.exe"
    pth = dest / f"python{WIN_EMBED_TAG.rpartition('.')[0]}._pth"
    if pth.exists():
        pth.write_text(pth.read_text().replace("#import site", "import site"))
    print(f"  ✔ Python embedded ipo: {exe}")
    return exe


def bundle_runtime() -> int:
    """Bundle KAMILI: binary zote ndani ya runtime/ + ZIP — hakuna dependencies.

    Mtumiaji anachukua ZIP pekee:
      • bin/mtaalamu (au MTAALAMU-Setup.exe) — ina Python NDANI yake (PyInstaller)
      • bin/mtaalamu-engine — engine ya Rust (kama ilijengwa kwenye host hii)
      • launchers + README-INSTALL.md — unzip → run, HAKUNA kusakinisha
    """
    import shutil
    import zipfile

    rc = build_onefile()
    if rc != 0:
        return rc
    name = {"windows": "MTAALAMU-Setup.exe", "darwin": "MTAALAMU-macos",
            "linux": "mtaalamu-linux"}.get(platform.system().lower(), "mtaalamu")
    bin_file = DIST / name
    if not bin_file.exists():
        print("✖ binary haipatikani baada ya build")
        return 1

    rt = HERE / "runtime"
    bins = rt / "bin"
    bins.mkdir(parents=True, exist_ok=True)
    dest = bins / bin_file.name
    shutil.copy2(bin_file, dest)
    try:
        dest.chmod(0o755)
    except OSError:
        pass
    print(f"  ✔ binary: {dest.name}")

    # Engine ya Rust (kama binary ya release ipo — imetengenezwa na cargo)
    engine = HERE.parent / "engine-rust" / "target" / "release" / ("mtaalamu.exe" if platform.system().lower() == "windows" else "mtaalamu")
    if engine.exists():
        shutil.copy2(engine, bins / "mtaalamu-engine")
        try:
            (bins / "mtaalamu-engine").chmod(0o755)
        except OSError:
            pass
        print("  ✔ engine ya Rust: bin/mtaalamu-engine")
    else:
        print("  – engine ya Rust: haipo (ijengwe kwanza: cd engine-rust && cargo build --release)")

    # Launchers (mtaalamu.sh / mtaalamu.cmd — hazihitaji python kwenye PATH)
    if platform.system().lower() == "windows":
        (rt / "mtaalamu.cmd").write_text("@echo off\r\n\"%~dp0bin\\MTAALAMU-Setup.exe\" %*\r\n")
    else:
        sh_launch = rt / "mtaalamu.sh"
        sh_launch.write_text("#!/bin/sh\nexec \"$(dirname \"$0\")/bin/mtaalamu-linux\" \"$@\"\n")
        sh_launch.chmod(0o755)

    # README-INSTALL + VERSION
    (rt / "README-INSTALL.md").write_text("""# MTAALAMU SMART — BUNDLE (hakuna kusakinisha chochote)

## Windows
1. Fungua folder hii
2. Double-click: `mtaalamu.cmd` (mfano: `mtaalamu.cmd serve`)

## Linux / macOS
```sh
chmod +x mtaalamu.sh
./mtaalamu.sh serve          # API: http://127.0.0.1:8795
./mtaalamu.sh apps           # apps za mfumo (OpenMRS, Home Assistant, Web R…)
./mtaalamu.sh kali           # ramani ya OS (zana, terminals, drivers, partitions)
```

Python ipo NDANI ya binary (PyInstaller onefile) — HAKUNA kusakinisha
Python, R, wala Rust. Engine ya Rust (bin/mtaalamu-engine) na R (analytics)
ndani ya bundle zinatumika zikizapatikana; mtaalamu ina fallback za stdlib.
""")
    (rt / "VERSION").write_text(platform.platform() + "\n")

    # ZIP moja (mtaalamu/dist/)
    import os as _os
    arch = _os.uname().machine if hasattr(_os, "uname") else "win"
    tag = {"Windows": "win", "Darwin": "macos"}.get(platform.system(), "linux")
    zpath = DIST / f"MTAALAMU-bundle-{tag}-{arch}.zip"
    with zipfile.ZipFile(zpath, "w", zipfile.ZIP_DEFLATED) as z:
        for f in sorted(rt.rglob("*")):
            if f.is_file() and "python-embed" not in f.name:
                z.write(f, f.relative_to(rt))
    print(f"\n✅ BUNDLE IMEKAMILIKA: {zpath}")
    print("   Mtumiaji: fungua ZIP → run mtaalamu.sh (au mtaalamu.cmd) — HAKUNA installs")
    return 0


def main() -> None:
    if "--bundle-runtime" in sys.argv:
        sys.exit(bundle_runtime())
    if "--embedded-win" in sys.argv:
        exe = fetch_embedded_windows(HERE / "runtime")
        sys.exit(0 if exe else 1)
    try:
        import PyInstaller  # noqa: F401
    except ImportError:
        print("PyInstaller haipo — inasakinishwa…")
        subprocess.run([sys.executable, "-m", "pip", "install", "-q", "pyinstaller"])
        try:
            import PyInstaller  # noqa: F401
        except ImportError:
            print("✖ PyInstaller haikuweza kusakinishwa — tumia install.bat/install.sh")
            sys.exit(1)
    sys.exit(build_onefile())


if __name__ == "__main__":
    main()

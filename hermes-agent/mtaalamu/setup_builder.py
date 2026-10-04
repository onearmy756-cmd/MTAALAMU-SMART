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


def main() -> None:
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

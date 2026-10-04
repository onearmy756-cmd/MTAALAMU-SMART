"""MTAALAMU SMART — setup/binary builder (download & use).

Mteja anapakua binary ya OS yake na KUTUMIA — hakuna Python inayohitajika:
    python3 setup_builder.py            # tengeneza binary ya OS hii (PyInstaller)
    python3 setup_builder.py --all      # specs zote (win/linux/mac) kwa CI

Inazalisha:
  dist/MTAALAMU-Setup.exe        (Windows, onefile)
  dist/mtaalamu-linux            (Linux/AppImage-ready, onefile)
  dist/MTAALAMU-macos            (macOS, onefile)
"""
import platform
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent


def build_onefile() -> int:
    name = {
        "windows": "MTAALAMU-Setup.exe",
        "darwin": "MTAALAMU-macos",
        "linux": "mtaalamu-linux",
    }[platform.system().lower()] if platform.system().lower() in ("windows", "darwin", "linux") else "mtaalamu"
    cmd = [
        sys.executable, "-m", "PyInstaller", "--onefile", "--name", name,
        "--paths", str(HERE), str(HERE / "cli.py"),
    ]
    print("  $", " ".join(cmd))
    return subprocess.run(cmd).returncode


def main() -> None:
    if "--all" in sys.argv:
        print("Specs zote zinajengwa na CI (GitHub Actions): win/linux/mac — angalia .github/workflows")
        return
    try:
        import PyInstaller  # noqa: F401
    except ImportError:
        print("saki: pip install pyinstaller")
        sys.exit(1)
    sys.exit(build_onefile())


if __name__ == "__main__":
    main()

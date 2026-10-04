"""MTECH OS — macho ya agent (vision).

Screenshot ya desktop → Qwen 2.5 VL 3B inaona na kueleza (Kiswahili).
Fallbacks: scrot (X11) → gnome-screenshot → import (X11) → notify error.
"""
import base64
import shutil
import subprocess
import tempfile
from pathlib import Path

from .config import CONFIG


def take_screenshot() -> Path:
    """Piga picha ya screen; inarudisha njia ya PNG."""
    out = Path(tempfile.gettempdir()) / "mtech_screen.png"
    for cmd in (
        ["scrot", "-o", str(out)],
        ["gnome-screenshot", "-f", str(out)],
        ["import", "-window", "root", str(out)],
    ):
        if shutil.which(cmd[0]):
            try:
                p = subprocess.run(cmd, capture_output=True, timeout=30)
                if p.returncode == 0 and out.exists() and out.stat().st_size > 0:
                    return out
            except (OSError, subprocess.TimeoutExpired):
                continue
    raise RuntimeError("Hakuna tool ya screenshot (saki: apt install scrot) — au huendeshi X11/Wayland session")


def see_screen(agent, args: dict) -> dict:
    """Tool 'see_screen': piga picha, tuma kwa Qwen VL, rudi na maelezo."""
    try:
        png = take_screenshot()
        agent.last_screenshot = str(png)
    except (RuntimeError, OSError) as e:
        return {"ok": False, "error": str(e)}
    try:
        from .llm import describe_screen
        desc = describe_screen(str(png), args.get("question", ""))
        return {"ok": True, "screenshot": str(png), "vision": desc}
    except Exception as e:  # noqa: BLE001
        return {"ok": True, "screenshot": str(png), "vision_error": str(e),
                "hint": f"Hakikisha ollama pull {CONFIG.model} imefanyika"}


def screenshot_b64() -> str:
    png = take_screenshot()
    return base64.b64encode(png.read_bytes()).decode()

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
    """Piga picha ya FULL SCREEN — OS yoyote (Linux/Windows/macOS)."""
    from .platform import screenshot_path
    return screenshot_path()


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

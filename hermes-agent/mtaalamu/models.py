"""MTAALAMU SMART — router ya models (TinyLlama offline + Ollama cloud + API yoyote).

DEFAULT: TinyLlama (ollama local, ndogo — inafikiri + kutafsiri offline).
User anaweza kuchagua: model yoyote ya local (ollama) AU API yoyote
(Ollama cloud / OpenAI-compatible). Kila jibu ni katika lugha ya mteja
(default Kiswahili).
"""
import json
import os
import urllib.error
import urllib.request
from pathlib import Path

from . import scope

CONFIG_FILE = Path.home() / ".mtaalamu" / "model.json"

DEFAULT_LOCAL = os.environ.get("MTECH_TINY", "tinyllama")           # offline thinking
DEFAULT_VISION = os.environ.get("MTECH_VISION", "qwen2.5vl:3b")     # Qwen VL — kazi halisi


def load_choice() -> dict:
    try:
        return json.loads(CONFIG_FILE.read_text())
    except (OSError, json.JSONDecodeError):
        return {"mode": "local", "model": DEFAULT_LOCAL, "api_url": "", "api_key_env": ""}


def save_choice(mode: str, model: str, api_url: str = "", api_key_env: str = "") -> None:
    CONFIG_FILE.parent.mkdir(parents=True, exist_ok=True)
    CONFIG_FILE.write_text(json.dumps(
        {"mode": mode, "model": model, "api_url": api_url, "api_key_env": api_key_env}, indent=1))


def _ollama_chat(model: str, system: str, user: str, images: list | None = None) -> str:
    url = os.environ.get("OLLAMA_URL", "http://127.0.0.1:11434")
    payload = {"model": model, "stream": False,
               "messages": [{"role": "system", "content": system},
                            {"role": "user", "content": user}]}
    if images:
        payload["messages"][-1]["images"] = images
    req = urllib.request.Request(url.rstrip("/") + "/api/chat",
                                 data=json.dumps(payload).encode(),
                                 headers={"Content-Type": "application/json"})
    try:
        with urllib.request.urlopen(req, timeout=120) as r:
            out = json.loads(r.read().decode())
        return (out.get("message") or {}).get("content", "").strip()
    except (urllib.error.URLError, json.JSONDecodeError, OSError) as e:
        raise RuntimeError(f"ollama {model}: {e}") from e


def _api_chat(cfg: dict, system: str, user: str) -> str:
    """API yoyote OpenAI-compatible (Ollama cloud, OpenAI, Groq, n.k.)."""
    key = os.environ.get(cfg.get("api_key_env") or "", "")
    headers = {"Content-Type": "application/json"}
    if key:
        headers["Authorization"] = f"Bearer {key}"
    req = urllib.request.Request(cfg["api_url"].rstrip("/") + "/chat/completions",
                                 data=json.dumps({"model": cfg.get("model", ""),
                                                  "messages": [{"role": "system", "content": system},
                                                               {"role": "user", "content": user}],
                                                  "temperature": 0.2}).encode(),
                                 headers=headers)
    try:
        with urllib.request.urlopen(req, timeout=120) as r:
            out = json.loads(r.read().decode())
        return (out.get("choices") or [{}])[0].get("message", {}).get("content", "").strip()
    except (urllib.error.URLError, json.JSONDecodeError, OSError) as e:
        raise RuntimeError(f"API {cfg.get('api_url')}: {e}") from e


def system_prompt() -> str:
    lang = ("Jibu KISWAHILI fasaha." if scope.LANG == "sw" else "Answer in fluent ENGLISH.")
    return (
        "Wewe ni MTAALAMU SMART — mtaalamu wa matatizo YOTE ya computer. "
        f"{lang} "
        "Unaruhusiwa KUJIBI tu maswali ya computer (hardware, software, network, Office, "
        "drivers, usalama, files, simu). Maswali mengine: kata kwa fupi. "
        "Ukiulizwa tatizo: (1) eleza chanzo kwa ufupi, (2) pendekeza hatua kamili, "
        "(3) usibuni amri zisizo za kweli — amri zinatokana na catalog ya mfumo. "
        "Hakuna hallucination: kama huwahi, sema 'Napasua kwa catalog' na uorodheshe ops."
    )


def ask(text: str, images: list | None = None, vision: bool = False) -> str:
    """Jibu la modeli iliyochaguliwa — lugha ya mteja (default sw)."""
    cfg = load_choice()
    sysmsg = system_prompt()
    if cfg.get("mode") == "api" and cfg.get("api_url"):
        return _api_chat(cfg, sysmsg, text)
    model = DEFAULT_VISION if (vision or images) else cfg.get("model", DEFAULT_LOCAL)
    try:
        return _ollama_chat(model, sysmsg, text, images)
    except RuntimeError:
        # offline-first: kama modeli kuu haipo, jaribu TinyLlama ndogo (kama ipo)
        if model != DEFAULT_LOCAL and not images:
            try:
                return _ollama_chat(DEFAULT_LOCAL, sysmsg, text)
            except RuntimeError:
                pass
        raise


def thinking_summary(text: str) -> str:
    """'Kufikiri' kwa TinyLlama: muhtasari mfupi wa tatizo + mpango (offline)."""
    try:
        return _ollama_chat(DEFAULT_LOCAL, "Muhtasari mfupi (mistari 3) kwa Kiswahili:", text)
    except RuntimeError:
        return "(TinyLlama haipatikani offline — hatua zinaendelea kutoka catalog)"


def translate(text: str, to: str | None = None) -> str:
    to = to or scope.LANG
    target = "Kiswahili" if to == "sw" else "English"
    try:
        return _ollama_chat(DEFAULT_LOCAL, f"Tafsiri kwa {target} tu, bila maelezo:", text)
    except RuntimeError:
        return text

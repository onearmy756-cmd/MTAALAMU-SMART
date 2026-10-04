"""MTECH OS — mteja wa Ollama (Qwen 2.5 VL 3B): chat + vision + pull."""
import base64
import json
import urllib.error
import urllib.request

from .config import CONFIG


class OllamaError(RuntimeError):
    pass


def _post(path: str, payload: dict, timeout: int = 180) -> dict:
    req = urllib.request.Request(
        CONFIG.ollama_url.rstrip("/") + path,
        data=json.dumps(payload).encode(),
        headers={"Content-Type": "application/json"},
        method="POST",
    )
    try:
        with urllib.request.urlopen(req, timeout=timeout) as r:
            return json.loads(r.read().decode())
    except urllib.error.URLError as e:
        raise OllamaError(f"Ollama haipatikani {CONFIG.ollama_url}: {e}") from e
    except json.JSONDecodeError as e:
        raise OllamaError(f"Jibu la Ollama halieleweki: {e}") from e


def _get(path: str, timeout: int = 10) -> dict:
    try:
        with urllib.request.urlopen(CONFIG.ollama_url.rstrip("/") + path, timeout=timeout) as r:
            return json.loads(r.read().decode())
    except (urllib.error.URLError, json.JSONDecodeError) as e:
        raise OllamaError(f"Ollama haipatikani: {e}") from e


def installed_models() -> list:
    return [m.get("name", "") for m in _get("/api/tags").get("models", [])]


def model_ready() -> bool:
    try:
        names = installed_models()
    except OllamaError:
        return False
    base = CONFIG.model.split(":")[0]
    return any(n == CONFIG.model or n.split(":")[0] == base for n in names)


def pull_model(model: str = "") -> None:
    model = model or CONFIG.model
    _post("/api/pull", {"name": model, "stream": False}, timeout=3600)


def chat(messages: list, images: list = None, force_json: bool = True) -> str:
    """Piga simu Qwen 2.5 VL 3B. images = orodha ya base64 (PNG/JPEG)."""
    payload = {
        "model": CONFIG.model,
        "messages": messages,
        "stream": False,
        "options": {"temperature": CONFIG.temperature},
    }
    if images:
        payload["messages"] = messages + [{"role": "user", "content": "Picha imeambatanishwa.", "images": images}]
    if force_json:
        payload["format"] = "json"
    out = _post("/api/chat", payload)
    return (out.get("message") or {}).get("content", "").strip()


def chat_text(system: str, user: str, temperature: float = None) -> str:
    payload = {
        "model": CONFIG.model,
        "messages": [
            {"role": "system", "content": system},
            {"role": "user", "content": user},
        ],
        "stream": False,
        "options": {"temperature": CONFIG.temperature if temperature is None else temperature},
    }
    out = _post("/api/chat", payload)
    return (out.get("message") or {}).get("content", "").strip()


def describe_screen(png_path: str, question: str) -> str:
    """Vision: onyesha screenshot kwa Qwen VL — maswali kwa Kiswahili."""
    with open(png_path, "rb") as f:
        b64 = base64.b64encode(f.read()).decode()
    return chat_text(
        "Wewe ni macho ya MTECH OS. Eleza unachokiona kwa Kiswahili fupi, kisha pendekeza hatua moja inayofuata.",
        question or "Eleza picha hii ya screen.",
    )

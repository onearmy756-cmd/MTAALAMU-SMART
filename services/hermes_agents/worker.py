#!/usr/bin/env python3
"""
HERMES Workers — MTAALAMU SMART
================================
Workers za Python zinazoitwa na HERMES Gateway (Elixir, port 8088):

  POST /stt        — Whisper STT: audio (wav/mp3/webm, base64 au file) → matini ya Kiswahili
  POST /voice      — Whisper + Gateway FST pamoja: sauti → amri (HITL kama inahitajika)
  POST /agent/run  — LangChain agent (Ollama LLM hiari): anaeleza/kuandaa — HAIHESABU kamwe
  POST /crawl      — Crawl4AI: crawl vyanzo vilivoruhusiwa → data/knowledge/crawl/*.json
  GET  /health     — hali

Kanuni:
  - LLM haihesabu kamwe (R-1): hesabu zote ni engine-rust (8080) kupitia gateway /engine/call.
  - HITL: worker haitumi amri ya actuation moja kwa moja — gateway ndiye mlinzi.
  - Offline-first: bila Ollama /crawler model, endpoints zinajibu kwa error ya wazi (si demo bandia).
"""
from __future__ import annotations

import base64
import json
import os
import time
import uuid
from pathlib import Path
from typing import Any, Dict, List, Optional

import requests
from fastapi import FastAPI, File, HTTPException, UploadFile
from pydantic import BaseModel

APP_PORT = int(os.environ.get("HERMES_WORKERS_PORT", "8090"))
GATEWAY = os.environ.get("HERMES_GATEWAY_URL", "http://127.0.0.1:8088")
OLLAMA = os.environ.get("OLLAMA_URL", "http://127.0.0.1:11434")
OLLAMA_MODEL = os.environ.get("OLLAMA_MODEL", "qwen2.5:3b")
WHISPER_MODEL = os.environ.get("WHISPER_MODEL", "small")
CRAWL_OUTPUT = Path(os.environ.get("CRAWL_OUTPUT_DIR", "../../data/knowledge/crawl"))
ALLOWED_CRAWL_HOSTS = [
    h.strip()
    for h in os.environ.get(
        "CRAWL_ALLOWED_HOSTS",
        "en.wikipedia.org,sw.wikipedia.org,docs.who.int,www.who.int,farmos.org,www.openmrs.org",
    ).split(",")
    if h.strip()
]

app = FastAPI(title="HERMES Workers", version="1.0.0")


# --------------------------------------------------------------------------- models
class AgentRequest(BaseModel):
    task: str
    lang: str = "sw"
    context: Optional[Dict[str, Any]] = None


class CrawlRequest(BaseModel):
    urls: List[str]
    max_pages: int = 5
    note_sw: Optional[str] = None


class VoiceJSON(BaseModel):
    audio_b64: str
    lang: str = "sw"


# --------------------------------------------------------------------------- gateway client
def gateway_post(path: str, body: Dict[str, Any]) -> Dict[str, Any]:
    try:
        r = requests.post(f"{GATEWAY}{path}", json=body, timeout=15)
        r.raise_for_status()
        return r.json()
    except Exception as exc:  # noqa: BLE001 — gateway pengine haijaanza
        return {"ok": False, "error": f"gateway: {exc}"}


def engine_call(path: str, body: Optional[Dict[str, Any]] = None, method: str = "GET") -> Any:
    """Hesabu zote kwenda engine-rust kupitia gateway (R-1: LLM haihesabu)."""
    return gateway_post("/engine/call", {"path": path, "method": method, "body": body or {}})


# --------------------------------------------------------------------------- whisper (lazy)
_whisper = None


def whisper_model():
    global _whisper
    if _whisper is None:
        try:
            from faster_whisper import WhisperModel  # noqa: PLC0415

            _whisper = WhisperModel(WHISPER_MODEL, device="cpu", compute_type="int8")
        except Exception as exc:  # noqa: BLE001
            raise HTTPException(
                status_code=503,
                detail=f"Whisper haipatikani (sakina faster-whisper, model={WHISPER_MODEL}): {exc}",
            ) from exc
    return _whisper


def transcribe(audio_path: Path, lang: str = "sw") -> Dict[str, Any]:
    model = whisper_model()
    segments, info = model.transcribe(str(audio_path), language=lang, beam_size=5)
    text = " ".join(seg.text.strip() for seg in segments).strip()
    return {"text": text, "language": info.language, "duration_s": round(info.duration, 2)}


# --------------------------------------------------------------------------- endpoints
@app.get("/health")
def health() -> Dict[str, Any]:
    ollama_ok = False
    try:
        ollama_ok = requests.get(f"{OLLAMA}/api/tags", timeout=2).ok
    except Exception:  # noqa: BLE001
        pass
    return {
        "ok": True,
        "service": "hermes_agents",
        "whisper_model": WHISPER_MODEL,
        "ollama": {"url": OLLAMA, "reachable": ollama_ok, "model": OLLAMA_MODEL},
        "crawl_allowed_hosts": ALLOWED_CRAWL_HOSTS,
        "gateway": GATEWAY,
    }


@app.post("/worker/register")
def worker_register() -> Dict[str, Any]:
    """Worker anajitambulisha kwa gateway (capabilities za HERMES task queue)."""
    return gateway_post(
        "/worker/register",
        {"id": "hermes_agents", "capabilities": ["stt", "agent", "crawl"]},
    )


@app.post("/stt")
async def stt(file: Optional[UploadFile] = File(default=None)) -> Dict[str, Any]:
    """Whisper STT: file upload (wav/mp3/webm) au JSON {audio_b64}."""
    tmp = Path(f"/tmp/hermes_stt_{uuid.uuid4().hex}")
    try:
        if file is not None:
            tmp.write_bytes(await file.read())
        else:
            raise HTTPException(status_code=400, detail="Tuma audio file au audio_b64 (JSON /voice)")
        out = transcribe(tmp, "sw")
        return {"ok": True, **out}
    finally:
        tmp.unlink(missing_ok=True)


@app.post("/voice")
async def voice(body: VoiceJSON) -> Dict[str, Any]:
    """Sauti moja kwa moja: Whisper STT → Gateway FST → amri (+HITL check)."""
    tmp = Path(f"/tmp/hermes_stt_{uuid.uuid4().hex}.wav")
    try:
        tmp.write_bytes(base64.b64decode(body.audio_b64))
        out = transcribe(tmp, body.lang)
        fst = gateway_post("/voice", {"text": out["text"], "lang": body.lang})
        return {"ok": True, "stt": out, "fst": fst.get("fst", fst)}
    finally:
        tmp.unlink(missing_ok=True)


@app.post("/agent/run")
def agent_run(req: AgentRequest) -> Dict[str, Any]:
    """
    LangChain agent (Ollama). HERMES anamwamkia: kuelezana, kupanga, kuandika Kiswahili.
    KWA MAKUSUDI hatumani tool ya hesabu — R-1: LLM haihesabu; kama task inahitaji namba,
    tunaita engine-rust kupitia gateway kwanza na kuweka matokeo kwenye context.
    """
    ctx = dict(req.context or {})
    if "engine_lookup" in ctx:
        ctx["engine_result"] = engine_call(ctx["engine_lookup"].get("path", "/api/diagnosis"))

    try:
        from langchain_community.chat_models import ChatOllama  # noqa: PLC0415
        from langchain_core.messages import HumanMessage, SystemMessage  # noqa: PLC0415

        llm = ChatOllama(model=OLLAMA_MODEL, base_url=OLLAMA, temperature=0.2)
        system = (
            "Wewe ni HERMES, msimamizi wa mfumo wa MTAALAMU SMART (Tanzania). "
            "Lugha: Kiswahili fasaha. Kanuni: HUHESABU kamwe — kama swali linahitaji hesabu, "
            "sema: 'Hesabu zitafanywa na engine ya Rust' na uonyeshe data kutoka context. "
            "Hakuna data iliyobuniwa; kama hujui, sema hujui. "
            f"Muktadha: {json.dumps(ctx, ensure_ascii=False)[:2000]}"
        )
        answer = llm.invoke([SystemMessage(content=system), HumanMessage(content=req.task)])
        return {"ok": True, "agent": "hermes/langchain", "model": OLLAMA_MODEL, "answer": answer.content}
    except Exception as exc:  # noqa: BLE001
        raise HTTPException(status_code=503, detail=f"Ollama/LangChain haipatikani: {exc}") from exc


@app.post("/crawl")
def crawl(req: CrawlRequest) -> Dict[str, Any]:
    """Crawl4AI: vyanzo vilivoruhusiwa tu (allowlist) → data/knowledge/crawl/*.json."""
    bad = [u for u in req.urls if not any(u.startswith(f"https://{h}") for h in ALLOWED_CRAWL_HOSTS)]
    if bad:
        raise HTTPException(status_code=400, detail=f"URL zisizo kwenye allowlist: {bad}")

    try:
        from crawl4ai import AsyncWebCrawler  # noqa: PLC0415
        import asyncio  # noqa: PLC0415
    except Exception as exc:  # noqa: BLE001
        raise HTTPException(status_code=503, detail=f"crawl4ai haipatikani: {exc}") from exc

    async def _run() -> List[Dict[str, Any]]:
        results: List[Dict[str, Any]] = []
        async with AsyncWebCrawler() as crawler:
            for url in req.urls[: max(1, req.max_pages)]:
                r = await crawler.arun(url=url)
                results.append(
                    {
                        "url": url,
                        "title": getattr(r, "title", "") or "",
                        "markdown": (getattr(r, "markdown", "") or "")[:8000],
                        "crawled_ts": int(time.time()),
                    }
                )
        return results

    pages = asyncio.run(_run())

    CRAWL_OUTPUT.mkdir(parents=True, exist_ok=True)
    saved = []
    for p in pages:
        name = p["url"].replace("https://", "").replace("/", "_")[:80]
        out = CRAWL_OUTPUT / f"{int(time.time())}_{name}.json"
        out.write_text(json.dumps(p, ensure_ascii=False, indent=2), encoding="utf-8")
        saved.append(str(out))

    return {"ok": True, "saved": saved, "pages": pages, "note_sw": req.note_sw or "Crawler imekamilika."}


if __name__ == "__main__":
    import uvicorn  # noqa: PLC0415

    uvicorn.run(app, host="0.0.0.0", port=APP_PORT)

# HERMES Workers (Python) — LangChain + Whisper + Crawl4AI

Workers za HERMES (Gateway ya Elixir inasimamia): **STT ya Kiswahili (Whisper), agents
za LangChain (Ollama), na crawler ya maarifa (Crawl4AI)**.

## Sakina + Anza

```bash
cd services/hermes_agents
python3 -m venv .venv && . .venv/bin/activate
pip install -r requirements.txt
python3 -m playwright install chromium   # crawl4ai inahitaji browser (mara ya kwanza)

HERMES_GATEWAY_URL=http://127.0.0.1:8088 python3 worker.py
# API: http://127.0.0.1:8090  (docs: /docs)
```

Environment: `HERMES_WORKERS_PORT` (8090), `HERMES_GATEWAY_URL` (8088), `OLLAMA_URL`
(11434), `OLLAMA_MODEL` (qwen2.5:3b), `WHISPER_MODEL` (small), `CRAWL_ALLOWED_HOSTS`,
`CRAWL_OUTPUT_DIR` (../../data/knowledge/crawl).

## API

| Method | Path | Maelezo |
|---|---|---|
| POST | `/voice` | `{audio_b64}` → Whisper → Gateway FST → amri (+HITL check) |
| POST | `/stt` | file upload (wav/mp3/webm) → matini ya Kiswahili |
| POST | `/agent/run` | LangChain + Ollama: kuelezana/kuandaa — **haihesabu kamwe** (R-1) |
| POST | `/crawl` | Crawl4AI: vyanzo vilivoruhusiwa tu → `data/knowledge/crawl/*.json` |
| POST | `/worker/register` | jitambulishe kwenye gateway (task queue) |
| GET  | `/health` | hali ya whisper/ollama/crawler |

## Mifano

```bash
# Sauti → amri (Whisper + FST + HITL)
python3 - <<'PY'
import base64, requests
audio = base64.b64encode(open("amri.wav","rb").read()).decode()
r = requests.post("http://127.0.0.1:8090/voice", json={"audio_b64": audio})
print(r.json())
PY

# Agent (LangChain + Ollama)
curl -s -X POST localhost:8090/agent/run -H 'content-type: application/json' \
  -d '{"task":"Eleza kwa Kiswahili ni nini maana ya voltage drop kwenye mtandao wa umeme"}' | jq

# Crawler (vyanzo salama pekee)
curl -s -X POST localhost:8090/crawl -H 'content-type: application/json' \
  -d '{"urls":["https://en.wikipedia.org/wiki/Irrigation"],"max_pages":2}' | jq
```

## Salama

- Crawler ina **allowlist** (`CRAWL_ALLOWED_HOSTS`) — hakuna crawl ya URL yoyote.
- Agent haina tool ya shell au hesabu — HERMES + engine-rust ndizo zinazoamsha hesabu.
- HITL: worker haifanyi actuation; gateway (Elixir) ndiye mlinzi wa HITL gates.

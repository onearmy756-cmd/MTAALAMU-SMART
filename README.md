# 🇹🇿 MTAALAMU SMART — Intelligent System

**Rust** (engine) + **R/Shiny** (dashboard) — data-driven JSON.

## Production (Agentic Vision + OS probe)

```bash
make build
make health
bash scripts/healthcheck.sh

# Deep OS probe (processes, disks, thermal, sockets on Linux)
./engine-rust/target/release/mtaalamu deep --top 15

# Full agentic session (live vision + HITL + report)
./engine-rust/target/release/mtaalamu agentic --msg "Kompyuta inaenda polepole" --approve

# Safe remediation catalog
./engine-rust/target/release/mtaalamu remediate
./engine-rust/target/release/mtaalamu remediate --action report_top_cpu
```

## SKILLS ENGINE (skills zote + AI halisi)

Skills **146** (trades 21 + AI) ziko `data/skills/skills.json` — Rust inasoma JSON pekee.

```bash
B=./engine-rust/target/release/mtaalamu

# Tafuta + hesabu + panga
cd engine-rust && cargo build --release && cd ..
$B skills --q betri
$B skill --id simu.kubadilisha_betri_ya_simu --inputs '{"battery_mah": 5000}'
$B skills-plan --msg "skrini ya simu imevunjika"

# TEKELEZA (halisi)
$B skills-run --skill ai.duckduckgo --msg "voltage drop formula"   # DDG search
$B skills-run --skill ai.pdf_search --msg "breaker sizing"          # PDF nyingi parallel (data/pdfs/)
$B skills-run --skill ai.ollama_llm --msg "Eleza: betri inakufa?"   # LLM ya bure (gpt-oss:20b)
$B skills-run --skill ai.huggingface --msg "tinyllama"              # HF hub search
$B skills-run --skill ai.pytorch --approve                          # PyTorch halisi (CPU)
$B skills-run --skill ai.tensorflow --approve                       # TensorFlow halisi

# Amri za moja kwa moja
$B search-web --q "umeme Tanzania" --both            # DDG + SearXNG merged
$B search-pdf --q "breaker" --dir data/pdfs
$B pdf-text --file data/pdfs/cable_sizing.pdf
$B ai-ask --msg "habari" --model gpt-oss:20b
$B hf-search --q "swahili"
```

**Modeli za bure** (`data/ai/models.json`): Ollama cloud (`OLLAMA_API_KEY`),
Ollama local (`ollama serve`), Hugging Face hub+inference. Default: `gpt-oss:20b`.

**SearXNG self-hosted**: `docker compose -f fundi-deploy/server/searxng/docker-compose.yml up -d`
kisha `export SEARXNG_URL=http://127.0.0.1:8888` (public instances zimezuiwa na bot-check).

**PyTorch/TensorFlow**: `pip3 install torch --index-url https://download.pytorch.org/whl/cpu`
na `pip3 install tensorflow-cpu numpy` — bridges zipo `scripts/ai_pytorch.py`, `scripts/ai_tensorflow.py`.

**HITL**: skills za `guide` (hardware) zinakatalia `--approve` isipo; AI/web actions zinapita kwa usalama.

UI:

```bash
make run-ui
# http://127.0.0.1:3838 → tab ◉ AGENTIC VISION
```

Runbook: [`PRODUCTION.md`](PRODUCTION.md) · Scope: [`docs/PRODUCTION_CAPABILITIES.md`](docs/PRODUCTION_CAPABILITIES.md)

## Lugha

Header: **English** ⇄ **Kiswahili** (`data/locales`, `web-r/R/i18n.R`).

## Muundo

```
web-r/          R/Shiny dashboard (LIVE, AGENTIC, FORMULA, DIAG, MAP)
engine-rust/    Rust CLI: calc, diagnose, agentic, deep, sysprobe, remediate, wiring, deploy
fundi-mobile/   🩺 FUNDI MOBILE — daktari wa simu (agentic, HITL consent, Android/iPhone/button)
fundi-deploy/   💻 Fundi Deploy — LAN imaging (P2+P3 kamili)
data/           JSON (formulas, agents, vision, geo, mobile/…)
upstream/       nakala za miradi 8 ya IoT (IMEHIFADHIWA repo — .git zake kwenye /tmp backup)
```

## IOT + HERMES (seti ya FOCUS)

JSON ndio chanzo: `data/iot/registry/*.json`, `data/iot/hermes.json`, `data/iot/agents_oss.json`.
Miradi yenye `"focus": true` ndiyo seti iliyochaguliwa (8):

| Kundi | Mradi |
|---|---|
| Smart Home | `home-assistant` |
| Hospitali | `openmrs` |
| Usalama | `frigate` |
| Msimamizi | `hermes-agent` |
| Workers | `langchain`, `crawl4ai`, `whisper`, `ollama` |

```bash
bash scripts/clone_iot_repos.sh                 # FOCUS registry 4 (home-assistant, openmrs, frigate + hermes)
bash scripts/clone_iot_repos.sh --agents        # FOCUS 4 + agents 4 za focus (8)
bash scripts/clone_iot_repos.sh --all --agents  # zote (registry + agents)
bash scripts/clone_iot_repos.sh --only afya     # vertical moja
```

> Nakala za upstream/ zipo kwenye repo hii moja kwa moja (bila .git yao, ~731MB).
> Kwa historia kamili ya kila mradi: `bash scripts/clone_iot_repos.sh --agents` (inadhihirisha .git upya).

UI: tab **📡 IOT REGISTRY** (web-r) inaonyesha FOCUS/CLONED kwa kila mradi.

## Tabs

1. **◉ LIVE MONITOR** — metrics (OS probe when available)
2. **◉ AGENTIC VISION** — multi-agent, HITL, live map, scribe, kitabu HTML
3. **🧮 FORMULA ENGINE**
4. **🧠 UTAMBUZI (BAYES)**
5. **🗺️ RAMANI / NAVIGATION**
6. **🌍 3D** — ramani ya tatu yenye picha halisi (Esri Imagery + NASA GIBS) + milima halisi (AWS Terrain DEM) — MapLibre GL, inachora kwa GPU (RAM/CPU kidogo): `web-r/www/ramani-3d.html`

## Rust (examples)

```bash
cd engine-rust
cargo run --release -- list
cargo run --release -- calc voltage_drop --inputs '{"L":30,"I":16,"A":2.5,"V":230,"rho":0.0175,"M":2}'
cargo run --release -- diagnose electrical --symptoms breaker_trips,sparks
```

## Muhimu

- Hesabu = Rust (deterministic)
- Vision/probe = OS metrics za kweli (si JSON bandia)
- HITL = ruhusa kabla ya remediation ya medium risk
- Si scope: motherboard X-ray, kernel video, AGI

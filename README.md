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
```

## Tabs

1. **◉ LIVE MONITOR** — metrics (OS probe when available)
2. **◉ AGENTIC VISION** — multi-agent, HITL, live map, scribe, kitabu HTML
3. **🧮 FORMULA ENGINE**
4. **🧠 UTAMBUZI (BAYES)**
5. **🗺️ RAMANI / NAVIGATION**

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

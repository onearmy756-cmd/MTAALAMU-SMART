# 🇹🇿 MTAALAMU SMART — Intelligent System

Mfumo wa **Rust (hesabu) + R (analytics) + R/Shiny (dashboard)** — vyote **data-driven** kwa JSON.
Uwasilishaji ni sawa na "Agent Vision Live" dashboard (neon-cyan HUD).

## 🌐 Lugha (LANGUAGE)

Dashboard ya `web-r/` ina **badilisho la lugha** kwenye header (🌐 LANGUAGE / LUGHA):
**English** (msingi) ⇄ **Kiswahili**. Tafsiri zote zinatoka kwenye `data/locales/*.json`
na `web-r/R/i18n.R`.

## 🏗️ Muundo (kama documents inavyoeleza)

```
┌─────────────────────────────────────────────────────┐
│  R/SHINY (web-r)     — UI / Dashboard               │
│  • Live monitor, formula calculator, utambuzi       │
│  • Inasoma JSON pekee — hakuna data ngumu kwenye code│
│  • Lugha mbili: English ⇄ Kiswahili (switcher)       │
├─────────────────────────────────────────────────────┤
│  RUST (engine-rust) — Hesabu Halisi (bila uongo)    │
│  • Formula engine (expression evaluator)            │
│  • Bayesian diagnosis (probability halisi)          │
│  • CLI: list / calc / diagnose / models             │
├─────────────────────────────────────────────────────┤
│  R (analytics-r)   — Data Science                   │
│  • Formula audit, sanity simulation                 │
│  • Prior integrity check (data-driven validation)   │
│  • Ripoti ya JSON kwa dashboard + charts            │
├─────────────────────────────────────────────────────┤
│  DATA (data/)      — JSON ZOTE                      │
│  • formulas.json  (formula 10+ kwa trades 6)        │
│  • diagnosis.json (Bayesian models 2: umeme, computer)│
└─────────────────────────────────────────────────────┘
```

## ⚡ KWA NINI DATA-DRIVEN (kama SMART VISUALIZATION na SMART SYSTEM 1 vinavyoeleza)

| | Code-heavy | Data-driven (mfumo huu) |
|---|---|---|
| Formula mpya | function mpya + tests + debug | kipengele kimoja cha JSON |
| Code | 300,000 lines | ~2,000 lines |
| Errors | nyingi | JSON haiongezeki |
| Kuongeza trade mpya | wiki za kazi | dakika chache |

**Kanuni:** Engine 1 (Rust) + formula JSON = formula zote 130+ bila code mpya.

## 🚀 Matumizi

### 1. Rust Engine
```powershell
cd engine-rust
cargo run -- list                          # orodha ya formula
cargo run -- calc voltage_drop --inputs '{"L":30,"I":16,"A":2.5,"V":230,"rho":0.0175,"M":2}'
cargo run -- calc battery_sizing --inputs '{"Load":1500,"Days":2,"DoD":0.5,"Vb":24,"eta":0.85}'
cargo run -- diagnose electrical --symptoms breaker_trips,sparks
cargo run -- models
```

### 2. R Analytics
```powershell
cd mtaalamu-smart
Rscript analytics-r/analytics.R
# => audit ya formula, sanity simulation, prior check, report.json kwa dashboard
```

### 3. R/Shiny Dashboard
```powershell
cd mtaalamu-smart
R -e "shiny::runApp('web-r', port = 3838)"   # http://127.0.0.1:3838
```
Jaribio la haraka: `Rscript web-r/tests/run_tests.R`

## 📺 Tabs za Dashboard

1. **◉ LIVE MONITOR** — Device Map, Processes, Network Topology, Issues & Warnings (kama picha ya "Agent Vision Live")
2. **🧮 FORMULA ENGINE** — chagua formula → weka inputs → matokeo + hatua za hesabu + status (GOOD/WARNING/FAIL)
3. **🧠 UTAMBUZI (BAYES)** — weka dalili → P(Cause|Symptoms) kwa mtindo wa Bayes

## ➕ Kuongeza Formula Mpya (hakuna code)

Ongeza kipengele kwenye `data/formulas.json`:
```json
{
  "id": "my_formula",
  "trade": "umeme",
  "name": { "sw": "Jina La Kiswahili", "en": "English Name" },
  "formula": "X = A × B",
  "inputs":  [ { "name": "A", "label": {"sw":"...","en":"..."}, "unit": "m", "default": 1 } ],
  "outputs": [ { "name": "X", "label": {"sw":"...","en":"..."}, "expr": "A * B", "unit": "m", "digits": 2 } ],
  "steps":   [ { "label": "Hesabu", "expr": "A * B", "template": "X = {{result}}" } ],
  "rules":   [ { "check": "X < 10", "status": "GOOD", "msg": {"sw":"...","en":"..."} } ]
}
```
Rust, R na Shiny zitaisoma **mara moja** — hakuna kubadilisha code.

## ➕ Kuongeza Dalili/Sababu Mpya

Ongeza kwenye `data/diagnosis.json` ndani ya model — priors + likelihood tu.

## 📁 Mistari

```
mtaalamu-smart/
├── data/
│   ├── formulas.json      # formula zote (data-driven)
│   └── diagnosis.json     # Bayesian models
├── engine-rust/
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs        # CLI
│       ├── expr.rs        # expression evaluator (engine 1)
│       ├── formula_engine.rs  # JSON → hesabu → status
│       └── bayes.rs       # P(Cause|Symptoms)
├── analytics-r/
│   └── analytics.R        # audit + validation + report
└── web-r/
    ├── app.R           # UI + server (Shiny)
    ├── R/              # i18n (sw|en), injini ya hesabu, charts, views
    ├── www/            # dashboard.css + shiny.css
    ├── data/           # nakala za JSON kwa browser
    └── tests/          # run_tests.R
```

## ⚠️ Muhimu

- **Hesabu = Rust/math** (sahihi 100%)
- **AI/LLM = tafsiri tu** (Kiswahili, maelezo) — si hesabu
- **Hali za mfumo = rules JSON** (GOOD/WARNING/FAIL)
- **Probability = Bayes halisi** si guess

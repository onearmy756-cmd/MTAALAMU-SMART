# 🇹🇿 MTAALAMU SMART — Intelligent System

Mfumo wa **Rust (hesabu) + R (analytics) + R/Shiny (dashboard)** — vyote **data-driven** kwa JSON.
Uwasilishaji ni sawa na "Agent Vision Live" dashboard (neon-cyan HUD).

**Moduli mpya (2026-09-29):** Location, Navigation, Voice (Kiswahili), Weather, Hazards — Tanzania kamili.

## 🌐 Lugha (LANGUAGE)

Dashboard ya `web-r/` ina **badilisho la lugha** kwenye header (🌐 LANGUAGE / LUGHA):
**English** (msingi) ⇄ **Kiswahili**. Tafsiri zote zinatoka kwenye `data/locales/*.json`
na `web-r/R/i18n.R`.

## 🏗️ Muundo

```
┌─────────────────────────────────────────────────────┐
│  R/SHINY (web-r)     — UI / Dashboard               │
│  • Live monitor, formula calculator, utambuzi       │
│  • RAMANI + Navigation (turn-by-turn + sauti)       │
│  • Lugha mbili: English ⇄ Kiswahili                 │
├─────────────────────────────────────────────────────┤
│  RUST (engine-rust) — Hesabu Halisi                 │
│  • Formula engine + Bayesian diagnosis              │
│  • (baadaye) geo / navigation helpers               │
├─────────────────────────────────────────────────────┤
│  R (analytics-r)   — Data Science                   │
├─────────────────────────────────────────────────────┤
│  DATA (data/)      — JSON ZOTE                      │
│  • formulas, diagnosis, trades, professions...      │
│  • geo/ (mikoa, wilaya, hierarkia)                  │
│  • navigation/ (voice prompts, turn-by-turn, alarm) │
│  • weather/ + hazards/                              │
└─────────────────────────────────────────────────────┘
```

## 📍 Location & Navigation (MPYA)

| Kipengele | Hali |
|-----------|------|
| Mikoa 31 | ✅ |
| Wilaya (~120+ listed, target 169+) | 🔄 L1 |
| Tarafa / Kata / Vijiji / Vitongoji / Mitaa / Barabara | ⏳ |
| Sauti (TTS) Kiswahili | ✅ prompts |
| Turn-by-turn instructions | ✅ templates |
| Alarm rules | ✅ |
| Weather schema | ✅ |
| Hazards types | ✅ |
| Auto-location (GPS) | ⏳ |
| Live route visuals | ⏳ |
| Labels kamili kwenye ramani | ⏳ |

Tazama `PLAN.md` sehemu **1B** na `data/geo/SCHEMA.md` kwa maelezo kamili.

## 🚀 Matumizi

### 1. Rust Engine
```powershell
cd engine-rust
cargo run -- list
cargo run -- calc voltage_drop --inputs '{"L":30,"I":16,"A":2.5,"V":230,"rho":0.0175,"M":2}'
cargo run -- diagnose electrical --symptoms breaker_trips,sparks
```

### 2. R Analytics
```powershell
Rscript analytics-r/analytics.R
```

### 3. R/Shiny Dashboard
```powershell
R -e "shiny::runApp('web-r', port = 3838)"
```

## 📺 Tabs za Dashboard

1. **◉ LIVE MONITOR**
2. **🧮 FORMULA ENGINE**
3. **🧠 UTAMBUZI (BAYES)**
4. **🗺️ RAMANI / NAVIGATION** (inaendelea — sauti + turn-by-turn + weather + hazards)

## ➕ Kuongeza Formula Mpya (hakuna code)

Ongeza kipengele kwenye `data/formulas.json` — Rust, R na Shiny zitasoma mara moja.

## ⚠️ Muhimu

- **Hesabu = Rust/math** (sahihi 100%)
- **AI/LLM = tafsiri tu** — si hesabu
- **Hali za mfumo = rules JSON**
- **Location data = JSON/GeoJSON** — offline-first inapowezekana
- **Sauti = Kiswahili kwanza**

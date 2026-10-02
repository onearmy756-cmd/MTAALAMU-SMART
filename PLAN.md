# MTAALAMU SMART — MPANGO WA UTEKELEZAJI (PRODUCTION)
### Inventory kamili kutoka documents zote 17 (.docx) → mfumo mmoja halisi

> Chanzi: ripoti 6 za kusoma documents (SMART SYSTEM 1, SMART VISUALIZATION, MTAALAMU JR,
> SMART DOCTOR 1/2/3/5/7/8/9/JY, NETWORK STUDIO + SMART NETWORK + SYSTEM 3 + SYSTEM 5, SMART COMPUTER).
> Jumla ya requirements: **~1,214** (R-250, V-100, J-171, D-278, N-302, C-113) + **Location/Nav module mpya**.
> Kanuni kuu: **data-driven JSON pekee** (kanuni ya token-savings 40×), **HITL si AI**, **offline-first**,
> **LLM haihesabu kamwe**, **Kiswahili kwanza**, **JSON = format pekee ya interop (Rust ↔ R ↔ Shiny UI ↔ DB)**.

---

## 1. MIPANGO YA MODULI (SYSTEM MAP)

### A. ENGINES 6 (kiini cha mfumo)
| Engine | Majukumu | Vyanzo |
|---|---|---|
| **1. Formula Engine** | hesabu deterministic: **132 formulas / 20 trades** + **network formulas 200+ / kundi 20**; kila result = expression + steps + units + standards + status (GOOD/WARNING/FAIL) + test cases | R-16…R-149, N-101…N-120, N-152…N-154 |
| **2. Knowledge Engine** | problems/solutions (10,000+ soko), **Electronic Devices Solver** (vifaa 59, kanuni 10), mobile_solver (2,000+), printing_solver (500+), huduma 30 computer + 25 simu + 7 mpya + 12 remote | R-12, R-153, C-3…C-13, C-96, V-87 |
| **3. Inference Engine** | BayesianDiagnoser (priors 0.30/0.20/0.15/0.15/0.10/0.10, learn 0.05), DiagnosticMatrix 7×9 (gari), DecisionTree (nodes 13 + phone tree 7), FuzzyAC (rules 9), MaterialOptimizer, CircuitSolver (LU), Rules Engine (gt/gte/lt/lte/eq) | R-207…R-218, N-159…N-162, R-168…R-170 |
| **4. Agent Engine** | agents **25 katika makundi 5**; Orchestrator (Pipeline/Parallel/Specialized/Hierarchical/Collaborative); aina 6: Diagnose/Repair/Backup/Install/Verify/Report; HITL gates; verify/audit; 8-step pipeline (receptionist→vision→diagnose→formula→knowledge→hitl→action→verify) | V-77…V-78, D-205…D-216, N-202…N-205 |
| **5. UI Engine** | charts **8 aina** (bar/pie/line/map/heatmap/network/flowchart/timeline), dashboards zote kutoka JSON, offline 0 tokens | V-6, V-34…V-46 |
| **6. Location & Navigation Engine (MPYA)** | Ramani kamili ya Tanzania + navigation + sauti + hali ya hewa + hatari + auto-location + live route visuals | User request 2026-09-29 |

### B. DATA LAYER (JSON — `data/`, source of truth MOJA)
| File | Maudhui | Kanuni |
|---|---|---|
| `formulas.json` | **132 formulas / 20 trades** (umeme 10, solar 7, maji 8, gari 10, AC 10, ujenzi 10, useremala 8, welding 8, cctv 5, computer 8, simu 5, electronics 6, pump 5, jenereta 5, gas 4, rangi 5, gate motor 4, borehole 5, appliance 4, tailor 5) — kila moja: expr, steps, units, standards, rules, test cases, sw/en | R-16…R-149 |
| `formulas_network.json` | **200+ network/telecom formulas / kundi 20** (signal, antenna, propagation, link budget, modulation, traffic, RAN, fiber, satellite, 5G NR, subnetting, bandwidth, latency, routing, QoS, wireless, security, monitoring, virtualization, cloud) | N-101…N-120 |
| `trades.json` | trades 20/21: id, name_sw/en, icon, color, skills[], common_problems[] | R-154, C-81 |
| `problems.json` + `solutions.json` | problems/solutions (schema: symptoms, causes{prob}, formula, time_min, cost_tzs, success_rate, cases) | R-153, N-155 |
| `diagnosis.json` | Bayesian: priors + likelihood matrix + causes + symptoms (electrical, computer, mechanic) | R-208…R-213 |
| `decision_trees/` | car tree (nodes 13), phone tree (nodes 7) | R-214, N-159 |
| `rules/` | rules_electrical.json (VOLTAGE_DROP_HIGH/WARNING/GOOD/BREAKER_UNDERSIZED) | N-160…N-162 |
| `devices_solver.json` | electronic_devices_solver (kundi, vifaa 59, {aina, matatizo, suluhisho}) + kanuni 10 | C-3…C-13 |
| `professions.json` | **104 professions / kundi 10** + huduma + bei za TZS | N-165…N-170 |
| `services.json` | catalog: 30 computer + 25 phone + huduma 7 mpya + remote 12 + pay-as-you-use 8 | V-87, D-142…D-155, N-275 |
| `pricing.json` | **PRICING BIBLE**: kila kitu cha TZS (commission, escrow, VAT, subscriptions, hardware BOM, per-service) | D-277, J-49…J-51, N-185…N-186 |
| `config.json` | system{name,version,language,offline_mode}, ai{model,learning_rate,bayesian,fuzzy}, database, payment{M-Pesa,commission,escrow}, notifications | R-231 |
| `locales/sw.json`, `locales/en.json` | LanguagePack `{language,name,flag,translations{app,nav,buttons,status,hardware,services}}` + stats() | J-15…J-31, J-71…J-72 |
| `jobs/`, `learning.json`, `ratings.json`, `reports.json` | records: JOB-YYYYMMDD-NNN, learning loop, ratings, ripoti | R-155…R-159, R-230 |
| `schema/*.sql` | trades/skills/problems/solutions/cases/learning + hitl_requests + escrow 6 tables + payasyouuse 5 tables | R-222, N-148, D-118, N-278…N-282 |
| `agent_data.json` | agent vision: components, processes, topology, issues (schema V-67…V-70) | V-19, V-67…V-70 |
| **`geo/` (MPYA)** | Hierarkia kamili ya Tanzania: mkoa → wilaya → tarafa → kata → kijiji → kitongoji → mtaa → barabara. GeoJSON + labels zote. Hakuna kimoja kirukwe. | User 2026-09-29 |
| **`navigation/` (MPYA)** | Routes, turn-by-turn instructions (sw), voice prompts, alarm rules, hazard types | User 2026-09-29 |
| **`weather/` (MPYA)** | Weather forecast schema + cache + alerts | User 2026-09-29 |
| **`hazards/` (MPYA)** | Aina zote za hatari (barabara, mafuriko, uhalifu, umeme, n.k.) + rules za kuonyesha | User 2026-09-29 |

### C. RUST (`engine-rust/` — workspace ya crates 2)
**crate `mtaalamu-core`** (hakuna I/O ya network):
- `expr.rs` — tokenizer + recursive-descent evaluator (`+ - * / ^ %`, sqrt/pow/ceil/floor/abs/log/exp/min/max/if, pi)
- `formula_engine.rs` — load JSON → validate min/max → calculate → steps → status GOOD/WARNING/FAIL → standard-snap (R-205)
- `bayes.rs`, `diagnostic_matrix.rs`, `decision_tree.rs`, `fuzzy.rs` (FuzzyAC), `optimizer.rs` (MaterialOptimizer), `circuit.rs` (CircuitSolver LU — nalgebra), `rules.rs`, `knowledge.rs` (match_ratio × prior), `devices_solver.rs` (ElectronicSolver kutoka JSON), `i18n.rs` (LanguagePack loader + stats()), `binary_engine.rs` (bincode + AES-256-GCM + zstd + Argon2 + `STEC` header)
- tests: test vectors 7 (R-179, R-212, R-213) + kila formula na numeric assertion

**crate `mtaalamu-server`** (axum):
- `main.rs` — routes, CORS, static, ws
- `api.rs`, `db.rs` (SQLite + seeder + learning loop), `hitl.rs` (create/approve/is_approved/pending), `agents/` (orchestrator, receptionist, formula, knowledge, hitl, verify), `network_diagnose.rs` (L1→L7), `vision/` (pixel, system, patterns, api), `video/` (capture, annotation, assembly, voice, streaming), `professions.rs`, `escrow.rs`, `wallet.rs`, `mpesa.rs` (Daraja STK + callback)
- **MPYA**: `geo.rs`, `navigation.rs`, `voice.rs`, `weather.rs`, `hazards.rs`, `location.rs` (GPS auto-detect)

**Endpoint registry** (API version `/api`):
```
POST /api/diagnose                      GET  /api/diagnosis
POST /api/formula/calculate             GET  /api/formulas
GET  /api/trades                        GET  /api/problems
GET  /api/dashboard/stats               GET  /api/dashboard/jobs-by-region
GET  /api/dashboard/revenue-trend       GET  /api/dashboard/jobs-by-type
GET  /api/topology                      GET  /api/regions/stats
GET  /api/network/coverage              GET  /api/telecom/towers
GET  /api/devices/locations             GET  /api/jobs/:id/video
GET  /api/network/diagnose              GET  /api/hitl/pending?user_type=
POST /api/hitl/respond                  GET  /api/professions/categories
GET  /api/professionals?profession=     POST /api/solver/diagnose
GET  /api/solver/categories             POST /api/escrow/jobs|pay|complete|approve|dispute
GET  /api/escrow/balance                GET/POST /api/wallet…
POST /api/mpesa/stk                     POST /api/mpesa/callback
WS   /ws/map   /ws/job/:id   /ws/workflow/:id   /agents

# === LOCATION & NAVIGATION (MPYA) ===
GET  /api/geo/hierarchy                 GET  /api/geo/search?q=
GET  /api/geo/boundaries/:level/:id     GET  /api/geo/labels
POST /api/nav/route                     GET  /api/nav/instructions/:route_id
POST /api/nav/voice-prompt              GET  /api/location/current
GET  /api/weather/forecast              GET  /api/weather/alerts
GET  /api/hazards/nearby                POST /api/hazards/report
WS   /ws/nav                            WS   /ws/location
```
**V-92, N-289…N-297, R-224, D-59, D-121, C-52, C-68, C-84 + Location module**

### D. R (`analytics-r/`)
- `analytics.R` — formula audit, default-input simulation, prior integrity, Bayesian example, report.json (R-180…R-181)
- `predict.R` — regression, time-series, hypothesis tests, randomForest disk-failure (D-259)
- `plumber.R` — API: `/health`, `/analyze`, `/map/png|svg|interactive`, `/report/pdf` (D-72, D-197)
- `report.Rmd` — ripoti ya kila mwezi (D-259)

### E. R / SHINY (`web-r/` — production; UI imehamia kutoka React)
**Screens** (kila moja kutoka JSON, na loading/error/empty state):
1. Agent Vision Live (Device Map, Processes, Network Topology, Issues) — V-19
2. Agentic Dashboard (3-column: mteja/kifaa, pipeline 8 steps + log + metrics) — N-195…N-207
3. Formula Engine (jibu la kila formula + steps + status)
4. Utambuzi/Bayesian (posteriors + confidence)
5. Charts gallery (8 types)
6. Maps — **IMEJENGWA + IMEPANULIWA** (`web-r/` tab **RAMANI/MAP**): Leaflet + tiles + OSRM + **hierarkia kamili ya Tanzania** (mkoa/wilaya/tarafa/kata/kijiji/kitongoji/mtaa/barabara) + labels zote + GPS auto-detect + turn-by-turn + sauti Kiswahili + weather overlay + hazards overlay + live route visuals
7. Device Topology + Agent Workflow (ReactFlow) — V-11…V-12
8. Video player (share WhatsApp/FB/IG) — V-13
9. Admin (KPIs), Analytics, Wallet/escrow, Problem photo (annotations) — V-14…V-18
10. Network Diagnose (L1…L7 cards) — N-97…N-100
11. HITL Approval — N-144…N-147
12. Professions Marketplace (104) — N-179…N-184
13. Remote Support client + admin — N-209…N-219
14. License (activate) + Admin keys + Pay-As-You-USE wallet — N-259…N-277
15. Services catalog + i18n toggle (sw/en) + PWA/responsive — V-23, J-15…J-31
16. **Navigation Live (MPYA)** — turn-by-turn + sauti + alarm + weather + hatari + live people-on-route view

**Stack**: R + shiny + htmltools (UI), JSON data-driven, SVG charts (bila JS framework), **i18n sw/en switcher** kwenye header. **V-30**

### F. HTML single-file (offline, tokens 0) — `web-html/`
`agentic-dashboard.html`, `smart-technician-client.html`, `smart-technician-admin.html`, `agent-view.html`,
`smart-technician.html`, `license-pdf.html`, `admin-keys.html`, `license.html`, `payasyouuse.html`,
`mtaalamu-sauti.html` (voice sw-TZ), `ai-scribe.html` — V-20, N-209…N-287, J-85…J-101

### G. BIDHAA / PRODUCTS (catalog)
FUNDI RESCUE (USB), FUNDI DEPLOY, FUNDI MOBILE, FUNDI BOX, FUNDI MAP, FUNDI PRO, FUNDI MULTI-AGENT,
Electronic Devices Solver, Remote IT Support, Huduma 7, Computer Doctor (SaaS), Smart Technician,
Professions Marketplace, Pay-As-You-USE — D-1…D-228, C-47…C-86

### H. COMPLIANCE / BIASHARA
- VAT **18%**, commission **10–15%**, escrow **5–8%**, late fee 5%, refund 50% (data recovery)
- Sheria: Data Protection Act 2022, Cybercrimes Kifungu 267, Banking Act 2006, Electronic Payments 2015, BRELA/TRA/TIN, BOT PSP license
- Consent forms (remote/simu/rescue/box) + video evidence + records miaka 2 — D-41…D-42, D-88…D-90, D-137…D-141, C-61
- License keys `MST-XXXX-XXXX-XXXX-XXXX`, tiers FREE/PERSONAL/BUSINESS/ENTERPRISE — N-259…N-270

---

## 1B. MODULI MPYA: LOCATION, NAVIGATION, VOICE, WEATHER & HAZARDS

### Mahitaji kamili (kutoka mtumiaji 2026-09-29)
1. **Hierarkia kamili ya Tanzania** – Mkoa, Wilaya, Tarafa, Kata, Kijiji, Kitongoji, Mtaa, Barabara. **Hakuna kimoja kirukwe**. Tanzania nzima.
2. **Sauti (TTS)** – Mfumo unaongea kwa sauti kwa Kiswahili.
3. **Mwongozo wa hatua kwa hatua** – Kila sehemu mtu anayoenda, inamwelekeza kwa Kiswahili + sauti mpaka afike sehemu husika.
4. **Alarm** – Arifa za sauti/vibration kwa matukio muhimu.
5. **Utabiri wa hali ya hewa** – Forecast + alerts.
6. **Kuonyesha hatari** za aina yoyote (barabara mbaya, mafuriko, uhalifu, umeme, n.k.).
7. **Auto-detect location** – Kutambua moja kwa moja mahali mtu alipo (GPS / browser geolocation).
8. **Picha live** za watu wakitembea kwenye route (live route visualization).
9. **Labels zote** kwenye ramani ziwe kamili (si partial).

### Muundo wa data (`data/geo/`, `data/navigation/`, `data/weather/`, `data/hazards/`)

```
data/
├── geo/
│   ├── SCHEMA.md                 # Schema ya hierarkia
│   ├── hierarchy.json            # Root index (mikoa 31 + metadata)
│   ├── regions/                  # mkoa_*.json au GeoJSON
│   ├── districts/                # wilaya
│   ├── divisions/                # tarafa
│   ├── wards/                    # kata
│   ├── villages/                 # vijiji
│   ├── vitongoji/                # vitongoji
│   ├── streets/                  # mitaa + barabara
│   └── labels.json               # Labels zote za ramani
├── navigation/
│   ├── voice_prompts_sw.json     # Maneno ya sauti (Kiswahili)
│   ├── turn_instructions.json    # Templates za turn-by-turn
│   └── alarm_rules.json          # Sheria za alarm
├── weather/
│   ├── schema.json
│   └── cache/                    # Offline cache ya forecast
└── hazards/
    ├── types.json                # Aina zote za hatari
    └── rules.json                # Sheria za kuonyesha + severity
```

### Voice (TTS)
- Primary: Browser Web Speech API (`speechSynthesis`) na lugha `sw-TZ` / `sw`
- Fallback: Offline TTS engine (baadaye) au pre-recorded prompts
- Voice prompts zote ziko kwenye `data/navigation/voice_prompts_sw.json` (data-driven)

### Navigation flow
1. Auto-detect current location (GPS)
2. User anachagua destination (search hierarchy au map click)
3. Route calculation (OSRM / offline routing)
4. Turn-by-turn instructions + sauti kwa Kiswahili
5. Live update ya position + re-route kama akipotoka
6. Alarm + weather + hazards overlay kwenye njia
7. Live visual ya watu/route (phase ya juu – AR/camera au simulated markers)

### Phases za moduli hii
| Phase | Kazi | Hali |
|---|---|---|
| **L0** | SCHEMA + skeleton folders + PLAN update | 🔄 Inafanyika sasa |
| **L1** | hierarchy.json (mikoa 31) + labels basic + search API | ⏳ |
| **L2** | Wilaya + Kata + GeoJSON boundaries (sources: NBS / OSM / HDX) | ⏳ |
| **L3** | Vijiji + Vitongoji + Mitaa/Barabara (OSM extract) | ⏳ |
| **L4** | Voice prompts + TTS integration (sw) + turn-by-turn | ⏳ |
| **L5** | Auto-location + live tracking + alarm | ⏳ |
| **L6** | Weather overlay + Hazards system | ⏳ |
| **L7** | Live route visuals (people walking) + full labels polish | ⏳ |

### Vyanzo vya data (lazima vithibitishwe)
- Tanzania National Bureau of Statistics (NBS) – admin boundaries
- OpenStreetMap (Tanzania extract) – streets, POIs
- HDX / UN OCHA – validated admin boundaries
- Official government gazetteers for names (Kiswahili)

**Kanuni**: Data yote iwe JSON/GeoJSON. Offline-first inapowezekana. Labels zote ziwe na `name_sw` + `name_en`.

---

## 2. MFUMO WA UTEKELEZAJI (phases)

| Phase | Kazi | Hali |
|---|---|---|
| **P0** | Rust toolchain install; repo structure; PLAN.md hii | 🔄 |
| **P1** | `data/` kamili: formulas 132 + network 200 + trades + problems + diagnosis + trees + rules + devices + professions + services + pricing + locales + config + schemas | ⏳ |
| **P2** | `mtaalamu-core`: expr, formula_engine, bayes, matrix, tree, fuzzy, optimizer, circuit, rules, knowledge, solver, i18n + tests (test vectors 7) | ⏳ |
| **P3** | `mtaalamu-server` (axum): endpoint zote + SQLite seeder + HITL + escrow + learning loop | ⏳ |
| **P4** | `analytics-r/`: analytics, predict, plumber, report.Rmd | ⏳ |
| **P5** | `web-r/`: screens zote 15 + i18n + PWA; build production | ⏳ |
| **P6** | `web-html/`: single-file apps 11 | ⏳ |
| **P7** | Verification: tests, build, browser, audit dhidi ya requirements (hakuna kitu kimeachwa) | ⏳ |
| **L0–L7** | Location & Navigation module (ona sehemu 1B) | 🔄 L0 |
| **I0** | IoT + HERMES: JSON registry (verticals 4) + focus set 8 (home-assistant, openmrs, frigate, hermes-agent + langchain, crawl4ai, whisper, ollama); `scripts/clone_iot_repos.sh` (focus default, `--agents`, `--all`, `--only`, `--list`); tab **IOT REGISTRY** (web-r/R/iot.R) | ✅ |

## 3. KANUNI ZISIZOKUBALIWA (kutoka documents)
1. **KANUNI 2/5**: content yote (formulas, bei, rules, copy, layout) kutoka JSON — code haitoi business content.
2. **KANUNI 4**: HITL — hatari kubwa (malipo, ubaguzi, risk) zinahitaji kibali cha binadamu.
3. **R-1**: LLM **haihesabu** — inaongea/translate/explain pekee.
4. **R-10 / V-37**: offline-first, 0 tokens, hakuna internet inahitajika.
5. **R-183**: JSON ndio format pekee interop.
6. **V-39**: hakuna hard-coded values kwenye components — kila kitu kutoka JSON.
7. **Location module**: Hierarkia kamili ya Tanzania, sauti Kiswahili, turn-by-turn, auto-location, weather, hazards, live visuals — yote data-driven.

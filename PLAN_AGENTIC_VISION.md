# MTAALAMU SMART — AGENTIC VISION (PRODUCTION)
### Moduli ya Multi-Agent + Live System Vision + Automatic Work + User Q&A + Digital Report Book

> Iliongezwa: 2026-09-29  
> Kanuni: **data-driven JSON**, **HITL**, **offline-first**, **LLM haihesabu**, **Kiswahili fasaha**, **Rust backend + R/Shiny UI**, **production si demo**.

---

## 1. SEHEMU TATU ZA MFUMO

| Sehemu | Jina | Kazi |
|--------|------|------|
| **1** | **User Q&A** | Mtumiaji anaandika/kusema tatizo → Agent inaanza ndani ya sekunde |
| **2** | **Automatic Work** | Agent inagundua matatizo yenyewe → inamweleza mteja → inahitaji ruhusa → inasolve |
| **3** | **Live Vision** | Ramani halisi ya kifaa + OS + processes + system bus + data flow (real-time monitoring) |

---

## 2. MULTI-AGENT 10

| # | Agent ID | Jina (SW) | Majukumu |
|---|----------|-----------|----------|
| 1 | `receptionist` | Mpokeaji | Kupokea tatizo, kuainisha trade, kuanzisha session |
| 2 | `vision` | Muono | Live inventory: hardware, processes, sensors, topology |
| 3 | `diagnoser` | Mgunduzi | Bayesian + rules + knowledge → orodha ya causes |
| 4 | `planner` | Mpangaji | Hatua: Plan → Identify → Implement → Test → Verify → Document |
| 5 | `solver` | Mtatuzi | Kutekeleza suluhisho (software fix / maelekezo hardware) |
| 6 | `tester` | Mjaribu | Kujaribu baada ya fix, regression checks |
| 7 | `verifier` | Mthibitishaji | Kuthibitisha tatizo limetatuliwa (metrics + HITL) |
| 8 | `scribe` | Mwandishi | AI Scribe: maelezo Kiswahili + hatua + picha/status |
| 9 | `reporter` | Mripoti | Ripoti kama kitabu kidigitali (tatizo→njia→suluhisho→tarehe) |
| 10 | `learner` | Mwanafunzi | Kuhifadhi maarifa, kusasisha priors/knowledge |

**Orchestrator**: `pipeline` (serial) | `parallel` | `hierarchical` | `collaborative`

**Pipeline 8-step (iliyopo + PIITVD):**  
`receptionist → vision → diagnose → plan → solve → test → verify → report/learn`

---

## 3. LIVE VISION — NINI NI HALISI vs VISUALIZATION

### 3.1 Data HALISI (OS APIs / system metrics)
- Orodha ya **processes** (PID, CPU%, RAM%, status)
- **Hardware inventory** (CPU model, RAM size, disk usage, network interfaces)
- **Sensors** (joto la CPU/GPU inapopatikana)
- **Network topology** (interfaces, routes, connections)
- **Service status** (running/stopped)

### 3.2 Visualization (ramani + michoro — data-driven)
- **Device Map**: sehemu zote (CPU, RAM, Disk, GPU, PSU, Fan, Motherboard, USB, Battery…) zilizo **labelled**, rangi kwa status (good/warning/critical)
- **System Bus diagram**: njia za data Input→Output kutoka topology JSON
- **Process map**: michakato inayoendesha + links
- **OS layer view**: userland → services → kernel (schematic, si kernel dump)

> **Ukweli wa production:** Kuona "system bus electrical traces" au "picha halisi ya motherboard live" kunahitaji hardware probes / AR camera.  
> Mfumo huu unatumia **metrics halisi** + **ramani/michoro labelled** + **status real-time**.  
> Haiundai data ya uongo — kila status inatokana na inventory au agent_data / system probe.

### 3.3 UI Live Vision panels
1. **Ramani ya Kifaa** — nodes labelled + status colors  
2. **Miunganisho** — edges Input→Output  
3. **System Bus** — lanes + throughput indicators  
4. **Processes Live** — table + sparkline  
5. **OS / Kernel schematic** — layers  
6. **Issue overlay** — matatizo yaliyogunduliwa juu ya ramani  
7. **Step video strip** — kila hatua ya solve ina frame (picha/status + sauti)

---

## 4. TROUBLESHOOTING PIPELINE (PIITVD)

| Hatua | ID | Maelezo |
|-------|-----|--------|
| **P** Plan | `plan` | Eleza lengo, resources, hatari |
| **I** Identify | `identify` | Dalili → causes (Bayesian/rules/knowledge) |
| **I** Implement | `implement` | Tekeleza suluhisho (auto au HITL) |
| **T** Test | `test` | Jaribu metrics / regression |
| **V** Verify | `verify` | Thibitisha + mteja/mtaalamu |
| **D** Document | `document` | Ripoti + store knowledge |

Kila hatua: `status`, `started_at`, `ended_at`, `evidence[]`, `voice_sw`, `visual_frame`.

---

## 5. AUTOMATIC WORK + HITL

1. Vision agent inachanganua mfumo → `issues[]`  
2. Diagnoser + Planner → plan  
3. **HITL gate**: "Nimegundua X. Niruhusu kurekebisha?" (sauti + UI)  
4. Mteja anakubali → Solver + Tester + Verifier  
5. Reporter + Learner → digital book + knowledge update  

**Software fixes** zinaweza kuwa automatic (kwa ruhusa).  
**Hardware** = maelekezo + orodha ya sehemu + gharama (pricing.json).

---

## 6. RIPOTI = KITABU KIDIGITALI

Muundo:
- Jalada (logo, session id, tarehe, muda)
- Sura 1: Tatizo (dalili, muktadha)
- Sura 2: Ugunduzi (vision findings + posteriors)
- Sura 3: Mpango (PIITVD steps)
- Sura 4: Utekelezaji (hatua + evidence)
- Sura 5: Uthibitisho (before/after metrics)
- Sura 6: Hitimisho + mapendekezo
- Kiambatisho: log, formulas used, knowledge refs

Lugha: **Kiswahili fasaha** (+ English toggle).  
Sauti: TTS `sw-TZ` kwa kila sura/hatua.

---

## 7. SAUTI + AI SCRIBE

- `data/vision/voice_scripts_sw.json` — prompts zote  
- Browser Web Speech API (`sw` / `sw-TZ`)  
- Scribe agent: maelezo hatua-kwa-hatua kwa Kiswahili, yanayoambatana na visual frames  

---

## 8. ARCHITECTURE (BACKEND → UI)

```
┌──────────────────────────────────────────────────────────┐
│  R/Shiny (web-r)  — Live Vision UI + Q&A + Report viewer │
│  • Device map SVG/Canvas  • Process table  • Step strip  │
│  • Voice controls  • HITL approve  • Digital book PDF    │
├──────────────────────────────────────────────────────────┤
│  RUST engine-rust                                         │
│  • agents.rs      — Multi-Agent 10 + Orchestrator         │
│  • vision.rs      — Inventory + topology + issues         │
│  • pipeline.rs    — PIITVD state machine                  │
│  • report.rs      — Digital book builder (JSON → MD/HTML) │
│  • + formula / bayes / knowledge / rules (existing)       │
├──────────────────────────────────────────────────────────┤
│  DATA (JSON)                                              │
│  data/agents/agents_10.json                               │
│  data/vision/device_map.json                              │
│  data/vision/system_bus.json                              │
│  data/vision/pipeline.json                                │
│  data/vision/report_template.json                         │
│  data/vision/voice_scripts_sw.json                        │
│  data/agent_data.json (live snapshot)                     │
└──────────────────────────────────────────────────────────┘
```

**API endpoints (baadaye axum server):**
```
POST /api/agent/session          — anza session (user problem)
GET  /api/agent/session/:id      — hali
POST /api/agent/hitl/:id         — ruhusa
GET  /api/vision/snapshot        — live components/processes/topology/issues
POST /api/vision/scan            — scan upya
GET  /api/pipeline/:session_id   — hatua PIITVD
POST /api/pipeline/:id/advance   — endelea hatua
GET  /api/report/:session_id     — digital book JSON/HTML
WS   /ws/vision                  — live updates
WS   /ws/agent/:session_id       — agent events + voice cues
```

---

## 9. PHASES ZA UTEKELEZAJI

| Phase | Kazi | Hali |
|-------|------|------|
| **AV0** | PLAN + data schemas (agents, vision, pipeline, report, voice) | ✅ |
| **AV1** | Rust: agents.rs + vision.rs + pipeline.rs + report.rs + CLI | 🔄 |
| **AV2** | Integrate knowledge/bayes/rules into pipeline | ⏳ |
| **AV3** | Shiny: Live Vision tab + Q&A + HITL + step strip | ⏳ |
| **AV4** | Voice TTS + Scribe narration SW | ⏳ |
| **AV5** | Digital book export (HTML/PDF) + learner store | ⏳ |
| **AV6** | Real OS probe module (sysinfo) + production hardening | ⏳ |

---

## 10. KANUNI ZISIZOKIUKWA
1. Hesabu = Rust pekee.  
2. Content ya biashara = JSON.  
3. HITL kwa hatua hatari / automatic fix.  
4. Hakuna data ya uongo — status kutoka snapshot au probe.  
5. Kiswahili kwanza kwenye sauti na ripoti.  
6. Production-ready: tests, error states, offline mode.

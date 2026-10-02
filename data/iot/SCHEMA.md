# MTAALAMU SMART — IOT + HERMES SCHEMA CONTRACT (canonical)

Kila file iko `data/iot/` lazima iwe **UTF-8, JSON halisi**.
Thibitisha: `node -e "JSON.parse(require('fs').readFileSync('FILE','utf8'))"`.
Kanuni: **JSON ndio pekee ya interop** (Rust ↔ Elixir gateway ↔ Python worker ↔ R/Shiny UI).

## 0. Kanuni za pamoja

- Status vocabulary: `GOOD | WARNING | FAIL` (ile ile ya `data/SCHEMA.md`).
- Grammar ya `check` (shared Rust↔JS↔Elixir↔R): mabadiliko `v` (thamani ya uwanja), `+ - * / %`,
  comparisons `< <= > >= == !=`, boolean `&& || !`, `true`, `false`. **Sheria ya kwanza inayolingana inashinda.**
- Maandishi yote ya UI/ripoti: `{"sw": "...", "en": "..."}` (Kiswahili kwanza).
- **HITL daima** kwa matendo ya hatari (dawa/actuation ya kliniki, locks/alarms, irrigation valves,
  malipo). Matendo ya hatari hayatekelezwi bila ruhusa ya binadamu.
- **LLM haihesabu kamwe** — hesabu ni Rust/R; LLM (LangChain/Ollama) anaongea, anaeleza, anapanga PEKEE.
- Leseni za miradi ya clone: thamani ya `license` ni marejeo — **thibitisha `LICENSE` ya repo kabla ya uzalishaji**.

## 1. `data/iot/verticals.json`

```json
{"version": "1.0.0", "updated": "2026-10-02",
 "verticals": [{"id": "afya", "name": {"sw": "...", "en": "..."}, "icon": "🏥", "color": "#00e676",
   "mqtt_root": "mtaalamu/afya", "risk": "high", "hitl_policy": {"sw": "...", "en": "..."},
   "summary": {"sw": "...", "en": "..."}}]}
```
`risk`: `low | medium | high`. Verticals 4: `afya`, `usalama`, `nyumbani`, `kilimo`.

## 2. `data/iot/registry/<vertical>.json` — catalog ya miradi ya clone

```json
{"version": "1.0.0", "vertical": "afya", "updated": "2026-10-02",
 "projects": [{"id": "openmrs", "name": "OpenMRS",
   "repo": "https://github.com/OpenMRS/openmrs-core", "license": "MPL-2.0", "language": "Java",
   "purpose": {"sw": "...", "en": "..."},
   "contributes": ["patients", "encounters"],
   "integration": {"type": "api", "protocol": "REST", "note_sw": "..."},
   "upstream_dir": "upstream/openmrs", "priority": 1}]}
```
- `integration.type`: `api | mqtt | adapter | reference`.
- Clone halisi: `bash scripts/clone_iot_repos.sh` (shallow → `upstream/<id>`, .gitignore'd).
  Flags: `--list`, `--only <vertical>`, `--id <project>`, `--agents`, `--force`, `--limit N`.

## 3. `data/iot/devices.json` — aina za vifaa (unified)

```json
{"version": "1.0.0", "updated": "2026-10-02",
 "devices": [{"id": "afya.patient_monitor", "vertical": "afya",
   "name": {"sw": "Kipimo cha Mgonjwa", "en": "Patient Monitor"}, "icon": "📈",
   "telemetry": [{"name": "hr_bpm", "unit": "bpm", "min": 30, "max": 200,
                  "warn_low": 50, "warn_high": 120, "fail_low": 40, "fail_high": 140}],
   "actuators": [], "risk": "high", "hitl_required": false,
   "source_projects": ["openmrs"], "demo": true}]}
```
- `hitl_required = true` kwenye **device** level linamaanisha: kila actuation inahitaji HITL.
- `demo: true` = device inaweza kuigizwa na `HermesGateway.Demo` (Elixir) / R fallback — **imeandikwa
  wazi kama DEMO kwenye UI**; hakuna data ya uongo inayowekwa kama halisi (KANUNI 4).

## 4. Wire format — telemetry event (HTTP/MQTT)

```json
{"device_id": "afya.patient_monitor", "ts": 1759400000, "source": "mqtt|http|demo",
 "fields": {"hr_bpm": 92, "spo2_pct": 97, "temp_c": 36.8}}
```
Topic MQTT: `mtaalamu/{vertical}/{device_id}/telemetry` (QoS 1). HTTP: `POST /ingest` (gateway).

## 5. `data/iot/telemetry_rules.json` — sheria → status + alert

```json
{"version": "1.0.0", "kanuni": "first matching rule wins",
 "rules": [{"id": "afya.patient_monitor.hr_bpm", "device": "afya.patient_monitor", "field": "hr_bpm",
   "checks": [{"check": "v >= 50 && v <= 120", "status": "GOOD", "msg": {"sw": "...", "en": "..."}},
              {"check": "true", "status": "FAIL", "msg": {"sw": "...", "en": "..."}}],
   "alert": {"severity": "critical", "voice_sw": "...", "notify": ["hospital", "sms"]}}]}
```
- `alert.severity`: `info | warning | critical`.
- `notify`: orodha ya channels (`ui | sms | hospital | siren | voice`).

## 6. `data/iot/voice_fst.json` — sauti: Whisper → FST (finite-state transducer)

States + transitions za grammar ya amri kwa Kiswahili:
```json
{"wake_words": ["hermes", "heremezi"], "language": "sw",
 "fst": {"states": ["START", "WAKE", "INTENT", "TARGET", "VALUE", "CONFIRM", "ACTION", "END", "ERROR"],
   "initial": "START", "accept": ["END"],
   "transitions": [{"from": "START", "token": "hermes|heremezi", "to": "WAKE", "reply_sw": "..."}]},
 "intents": [{"id": "washa", "sw": "washa", "action": "turn_on"}],
 "targets": [{"id": "taa", "device": "nyumbani.smart_light", "sw": "taa"}]}
```
- Token maalum: `<value>` (namba/kipimo), `*` (nyingine yote), regex `a|b` kwa maneno mbadala.
- Amri inayotoka FST ikiwa na `hitl_required = true` lazima ipite kwenye gate ya `hermes.json` KABLA
  ya kutumwa kwa actuator. Whisper (STT) + FST (grammar) = sauti inayotambulika bila LLM (offline).

## 7. `data/iot/hermes.json` — msimamizi mkuu

```json
{"hermes": {"id": "hermes", "upstream": {"repo": "...", "license": "MIT"},
  "principles": ["..."], "delegation": {"internal": [], "iot_specialists": []},
  "pipelines": [{"id": "iot_incident", "trigger": "alert FAIL", "steps": []}],
  "hitl_gates": [{"id": "hitl_clinical", "applies": "..."}]}}
```
- Pipelines zinatumia ids za agents za `data/agents/agents_10.json` + specialists + worker agents.
- Memory: `data/iot/missions.json` (missions + frames), learning loop = `data/learning_log.json`.

## 8. `data/iot/agents_oss.json` — agents zote za open source

```json
{"agents": [{"id": "langchain", "name": "LangChain",
  "repo": "https://github.com/langchain-ai/langchain", "license": "MIT", "language": "Python/JS",
  "contributes": ["tools", "chains"], "hermes_role": "worker", "status": "integrated"}]}
```
- `hermes_role`: `master-pattern | worker | pattern | adapter-planned`.
- `status`: `integrated | optional | adapter-planned`.
- Kanuni: agents za OSS ni **watumishi wa HERMES** — hawawezi kukiuka HITL wala kuhesabu.

## 9. `data/iot/integrations.json` — bridges (MQTT, gateway, engine, worker, whisper, crawl)

Vyanzo vyote vya config vya huduma (`services/hermes_gateway` + `services/hermes_agents`).
Ports chaguo-msingi: engine `8080`, gateway `8088`, worker `8090`, MQTT `1883`, Ollama `11434`.

## 10. Files za runtime (hazihifadhiwi katika git isipokuwa examples)

| File | Maudhui |
|------|---------|
| `data/iot/alerts.json` | alerts zilizofunguliwa/zilizofungwa (audit) |
| `data/iot/missions.json` | missions za HERMES + frames za agents |
| `data/iot/telemetry_log.jsonl` | events (moja kwa mstari) |
| `data/knowledge/crawl/` | matokeo ya crawler (JSON kila page) |

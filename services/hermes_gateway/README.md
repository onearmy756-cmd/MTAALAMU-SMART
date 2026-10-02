# HERMES Gateway (Elixir) — MTAALAMU SMART

Ndege-msimamizi wa nyuma: **inapokea telemetry ya IoT, inaendesha rules, FST ya sauti,
missions za HERMES, HITL store na task queue kwa workers (LangChain/Whisper/Crawl4AI).**

> HERMES (mfano wa hermes-agent ya Nous Research) ndiye **msimamizi mkuu**: anaamsha
> agents zote (agents_10 + IoT specialists + workers), anagawa kazi, anahifadhi memory,
> na anahakikisha **HITL kabla ya kila tenda la hatari**. LLM haihesabu kamwe — hesabu
> ziko Rust (`engine-rust`) na R.

## Uwezo

| Kipengele | Maelezo |
|---|---|
| **MQTT ingest** | `mtaalamu/+/+/telemetry` (emqtt — hiari, toa alama mix.exs) |
| **HTTP ingest** | `POST /ingest` — vifaa vyote vinaweza kutumia HTTP |
| **Rules engine** | `data/iot/telemetry_rules.json` → GOOD/WARNING/FAIL + alerts |
| **FST sauti** | Whisper (Python) → matini → `data/iot/voice_fst.json` → amri |
| **HITL gates** | `data/iot/hermes.json` + `devices.json` → store ya approvals |
| **Missions** | memory ya HERMES: incidents + frames za agents (Kiswahili) |
| **Worker queue** | register / poll / done kwa agents za Python |
| **IoT summary** | `GET /iot/summary` — verticals, clone registry, devices, agents, hermes (kutoka `data/iot/`) |

## Anza

```bash
cd services/hermes_gateway
mix deps.get
HERMES_GATEWAY_PORT=8088 MTAALAMU_DATA=../../data iex -S mix     # interactive
HERMES_GATEWAY_PORT=8088 MTAALAMU_DATA=../../data mix run --no-halt  # server
mix test                                                          # tests
```

Environment: `HERMES_GATEWAY_PORT` (8088), `MTAALAMU_DATA` (../../data), `MTAALAMU_ENGINE_URL`
(http://127.0.0.1:8080), `MQTT_HOST`, `MQTT_PORT`.

## API (port 8088)

```bash
# Health
curl -s localhost:8088/health | jq

# Telemetry (HTTP ingest — mfano: mgonjwa ana spo2 ya chini)
curl -s -X POST localhost:8088/ingest -H 'content-type: application/json' \
  -d '{"device_id":"afya.patient_monitor","source":"http","fields":{"spo2_pct":80,"hr_bpm":75}}' | jq

# Sauti: matini ya Whisper → FST
curl -s -X POST localhost:8088/voice -H 'content-type: application/json' \
  -d '{"text":"hermes washa taa"}' | jq

# HITL: orodha + amua
curl -s localhost:8088/hitl/pending | jq
curl -s -X POST localhost:8088/hitl/HL-1/decide -H 'content-type: application/json' \
  -d '{"decision":"approve","who":"daktari-juma"}' | jq

# Missions za HERMES
curl -s localhost:8088/missions | jq

# IoT summary (verticals + clone registry + devices + agents)
curl -s localhost:8088/iot/summary | jq '.projects_total, .devices_total, .agents_total'
```

## Mtiririko waHERMES

```
Vifaa (MQTT/HTTP) ─▶ Ingest ─▶ Rules ─┬─ GOOD/WARNING ─▶ telemetry (UI live)
                                      └─ FAIL ─▶ Alert + Mission (iot_incident)
                                                   │
Voice (Whisper→FST) ─▶ Command ─▶ HITL gate? ───●─ HITL: RUHUSU/GHAIRI (mtu)
                                                └─ auto_allowed ─▶ Actuator
Workers (LangChain/Whisper/Crawl4AI) ◀─▶ /worker/* ◀─▶ mission frames
Engine (Rust) ◀─ /engine/call ─ hesabu zote
```

## Tests

`mix test` — rules (GOOD/WARNING/FAIL), FST (wake word, amri kamili, valve),
ingest→mission (memory ya HERMES).

## Leseni

Kanuni za mradi: KANUNI 2/5 (JSON data-driven), KANUNI 4 (HITL), R-1 (LLM haihesabu),
R-10 (offline-first). emqtt ni Apache-2.0; Jason Apache-2.0; Plug/Cowboy Apache-2.0.

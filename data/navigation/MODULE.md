# Location & Navigation Module — MTAALAMU SMART

## Lengo
Mwongozo wa hatua kwa hatua kwa Kiswahili + sauti, Tanzania nzima, offline-first.

## Vipengele
1. **Hierarkia** — `data/geo/` (mkoa → barabara) — ✅ seed kamili
2. **Sauti (TTS)** — `voice_prompts_sw.json` — ✅ Kiswahili
3. **Turn-by-turn** — `turn_instructions.json` — ✅
4. **Alarm** — `alarm_rules.json` — ✅
5. **Weather** — `data/weather/schema.json` — ✅
6. **Hazards** — `data/hazards/types.json` — ✅
7. **GPS auto-detect** — browser Geolocation API + Rust location service
8. **Live route** — WebSocket `/ws/nav` + map markers

## Flow
```
1. GPS auto-detect → current location
2. User chagua destination (search geo hierarchy au map)
3. Route (OSRM / offline)
4. Turn-by-turn + sauti Kiswahili
5. Live position + re-route
6. Alarm + weather + hazards overlay
7. Live visuals (markers / camera phase ya juu)
```

## Faili
| Faili | Kazi |
|-------|------|
| `voice_prompts_sw.json` | Maneno yote ya TTS |
| `turn_instructions.json` | Templates turn-by-turn |
| `alarm_rules.json` | Sheria za alarm |
| `../geo/*` | Hierarkia + labels |
| `../weather/schema.json` | Forecast + alerts |
| `../hazards/types.json` | Aina za hatari |

## API (Rust)
```
GET  /api/geo/hierarchy
GET  /api/geo/search?q=
POST /api/nav/route
GET  /api/nav/instructions/:route_id
POST /api/nav/voice-prompt
GET  /api/location/current
GET  /api/weather/forecast
GET  /api/weather/alerts
GET  /api/hazards/nearby
POST /api/hazards/report
WS   /ws/nav
WS   /ws/location
```

## TTS
- Primary: Web Speech API `speechSynthesis` lang=`sw-TZ`
- Prompts: data-driven kutoka `voice_prompts_sw.json`
- Fallback: `sw` au pre-recorded

## Status
| Sehemu | Hali |
|--------|------|
| Geo hierarchy seed | ✅ |
| Voice prompts | ✅ |
| Turn instructions | ✅ |
| Alarm rules | ✅ |
| Weather schema | ✅ |
| Hazards types | ✅ |
| Rust geo/nav/voice services | ⏳ |
| Shiny map + nav UI | ⏳ |
| Live route visuals | ⏳ |

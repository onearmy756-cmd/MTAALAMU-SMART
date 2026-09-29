# Location & Navigation Module — MTAALAMU SMART

## Lengo
Mwongozo wa hatua kwa hatua kwa Kiswahili + sauti, Tanzania nzima, offline-first.

## Vipengele
1. **Hierarkia** — `data/geo/` (mkoa → barabara) — ✅ seed kamili (kila wilaya)
2. **Sauti (TTS)** — `voice_prompts_sw.json` — ✅ Kiswahili
3. **Turn-by-turn** — `turn_instructions.json` — ✅
4. **Alarm** — `alarm_rules.json` — ✅
5. **Weather** — `data/weather/schema.json` — ✅
6. **Hazards** — `data/hazards/types.json` — ✅
7. **GPS auto-detect** — browser Geolocation — ✅ (ramani + demo)
8. **Live route** — OSRM + Leaflet — ✅ (web demo)

## UI / CLI
| Faili / amri | Hali |
|--------------|------|
| `web-r/www/ramani-nav.html` | ✅ Ramani + search + GPS + OSRM + sauti + hazards |
| `web-r/www/nav-voice-demo.html` | ✅ TTS demo |
| `mtaalamu geo --q ...` | ✅ CLI search |
| `mtaalamu nav --maneuver ...` | ✅ CLI instructions |
| `mtaalamu hazards` | ✅ CLI hazards |

## Rust modules
| Module | Faili |
|--------|-------|
| GeoEngine | `engine-rust/src/geo.rs` |
| NavigationEngine | `engine-rust/src/navigation.rs` |
| HazardsEngine | `engine-rust/src/hazards.rs` |

## Flow
```
1. GPS auto-detect → current location
2. User chagua destination (search hierarchy au map click)
3. Route (OSRM)
4. Turn-by-turn + sauti Kiswahili
5. Hazards overlay
6. (Baadaye) weather overlay + live people-on-route
```

## Jinsi ya kujaribu ramani
Fungua kwenye browser:
`web-r/www/ramani-nav.html`

1. Bonyeza **GPS**
2. Tafuta mf. "Kariakoo" au "Dodoma"
3. Bonyeza **Njia**
4. Bonyeza **Sauti** kusikia Kiswahili
5. Bonyeza **Hatari** kuona overlay

## Status
| Sehemu | Hali |
|--------|------|
| Geo hierarchy seed | ✅ |
| Voice / turn / alarm JSON | ✅ |
| Weather / hazards JSON | ✅ |
| Rust geo + nav + hazards | ✅ |
| CLI geo/nav/hazards | ✅ |
| Ramani Leaflet + OSRM + TTS | ✅ |
| Shiny tab integration | ⏳ |
| Live people-on-route | ⏳ |
| Full OSM street labels | ⏳ (OSM tiles + data import) |

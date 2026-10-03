# Location & Navigation Module — MTAALAMU SMART

## Lengo
Mwongozo wa hatua kwa hatua kwa **Kiswahili + sauti**, Tanzania nzima, offline-first inapowezekana.

## Vipengele (2026-09-29 — kamili)

| # | Kipengele | Hali | Maelezo |
|---|-----------|------|---------|
| 1 | Hierarkia (mkoa→barabara) | ✅ | Mikoa 31, wilaya 184, seed ya kata/kijiji/mtaa/barabara kila wilaya. Labels kamili = OSM tiles |
| 2 | Sauti (TTS) Kiswahili | ✅ | Web Speech API `sw-TZ` + prompts JSON |
| 3 | Turn-by-turn | ✅ | OSRM steps → Kiswahili + sauti kila hatua |
| 4 | Alarm | ✅ | Arrival, off-route, hazard, weather, GPS lost (`alarm_rules.json`) |
| 5 | Weather | ✅ | Open-Meteo (bure) + alerts Kiswahili |
| 6 | Hazards | ✅ | Aina 12 + overlay + sauti (`hazards/types.json`) |
| 7 | Auto GPS | ✅ | `getCurrentPosition` + `watchPosition` live |
| 8 | Live people on route | ✅ | Markers wanaotembea kwenye polyline (demo) |
| 9 | Labels zote kwenye ramani | ✅ | OpenStreetMap tiles (zoom in → majina yote ya mitaa/barabara) + Nominatim search Tanzania |
| 10 | Search Tanzania nzima | ✅ | Index ya local + Nominatim `countrycodes=tz` |

## UI
| Faili | Hali |
|-------|------|
| `web-r/www/ramani-nav.html` | ✅ **Kamili** — GPS, Njia, Anza/Simamisha, Sauti, Hewa, Hatari, Live |
| `web-r/www/nav-voice-demo.html` | ✅ TTS demo |
| `web-r/www/map.js` | ✅ Shiny map (basemaps + boundaries) |

## Data
```
data/geo/          hierarchy, districts, tarafa, kata, vijiji, vitongoji, mitaa, barabara
data/navigation/   voice_prompts_sw.json, turn_instructions.json, alarm_rules.json
data/weather/      schema.json
data/hazards/      types.json
```

## Jinsi ya kujaribu
Fungua: `web-r/www/ramani-nav.html`

1. **GPS** → ruhusu location  
2. Tafuta mf. *Kariakoo*, *Dodoma*, *Mtaa wa Uhuru* (Nominatim + local)  
3. **Njia** → OSRM route + hatua Kiswahili  
4. **Anza** → navigation live + sauti kila pinda  
5. **Hewa** → utabiri Open-Meteo  
6. **Hatari** → overlay + sauti  
7. **Live** → watu wakitembea kwenye njia  
8. Zoom in → **labels zote** za OSM (mitaa, barabara, majengo)

## Kuhusu data kamili ya mitaa/vijiji
- **Wilaya 184 + mikoa 31** = kamili kwenye JSON  
- **Kata / kijiji / kitongoji / mtaa / barabara** = seed kila wilaya + majina maarufu  
- **Majina yote ya mitaa Tanzania** hayawezi kuwekwa JSON moja (ni mamilioni kutoka OSM) — yanapatikana **live** kupitia:
  - OpenStreetMap tiles (labels)
  - Nominatim search (Tanzania)
  - GeoJSON boundaries (`tz_regions`, `tz_districts`, `tz_wards`)

Import kamili ya OSM/NBS inaweza kuongezwa baadaye kama batch job bila kubadilisha API.

## Rust / CLI
| Module | Faili |
|--------|-------|
| GeoEngine | `engine-rust/src/geo.rs` |
| NavigationEngine | `engine-rust/src/navigation.rs` |
| HazardsEngine | `engine-rust/src/hazards.rs` |

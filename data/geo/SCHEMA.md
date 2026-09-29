# data/geo/SCHEMA.md — Hierarkia kamili ya Tanzania

## Lengo
Kufunika **Tanzania nzima** bila kuruka hata kimoja:
**Mkoa → Wilaya → Tarafa → Kata → Kijiji → Kitongoji → Mtaa → Barabara**

## Viwango (Levels)

| Level | Jina (SW) | Jina (EN) | Key | Status |
|-------|-----------|-----------|-----|--------|
| 0 | Nchi | Country | country | ✅ |
| 1 | Mkoa | Region | region | ✅ 31 |
| 2 | Wilaya | District | district | ✅ 184 |
| 3 | Tarafa | Division | tarafa / division | 🔄 in_progress |
| 4 | Kata | Ward | ward / kata | ⏳ |
| 5 | Kijiji | Village | village | ⏳ |
| 6 | Kitongoji | Hamlet | vitongoji | 🔄 seed |
| 7 | Mtaa | Street | mtaa / street | 🔄 seed |
| 8 | Barabara | Road | road / barabara | ⏳ |

## Faili

| Faili | Maudhui |
|-------|---------|
| `hierarchy.json` | Root index (mikoa + stats) |
| `districts.json` | Wilaya 184 (kamili) |
| `tarafa.json` | Tarafa (inajazwa) |
| `vitongoji.json` | Vitongoji (seed + inajazwa) |
| `mitaa.json` | Mitaa (seed + inajazwa) |
| `SCHEMA.md` | Hii |

## Schema ya kila node

```json
{
  "id": "string (unique, hierarchical e.g. TZ-02-01-01-001)",
  "level": "region|district|tarafa|ward|village|vitongoji|mtaa|road",
  "name_sw": "Jina la Kiswahili (lazima)",
  "name_en": "English name",
  "parent_id": "id ya parent",
  "region_id": "TZ-XX",
  "district_id": "TZ-XX-YY",
  "center": { "lat": 0.0, "lng": 0.0 },
  "geojson_ref": "path or null",
  "labels": []
}
```

## Kanuni
- **Hakuna kuruka** – lengo ni data kamili.
- Data ya chini (tarafa → mtaa) inajazwa **region by region** kutoka NBS + OpenStreetMap.
- Offline-first, JSON/GeoJSON.
- `name_sw` ni lazima.

## Status ya sasa
- ✅ Mikoa 31
- ✅ Wilaya 184
- 🔄 Tarafa, Vitongoji, Mitaa – muundo + seed data (Dar, Dodoma, Arusha, Mwanza). Full fill inaendelea.
- ⏳ Kata, Vijiji, Barabara

**Ahadi**: Tutajaza zote hatua kwa hatua bila kuruka. Data kamili ya mitaa/vitongoji nchi nzima inahitaji import kubwa (OSM) – muundo tayari unaruhusu hilo.

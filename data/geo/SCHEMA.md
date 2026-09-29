# data/geo/SCHEMA.md — Hierarkia kamili ya Tanzania

## Lengo
Kufunika **Tanzania nzima** bila kuruka hata kimoja:
**Mkoa → Wilaya → Tarafa → Kata → Kijiji → Kitongoji → Mtaa → Barabara**

## Viwango (Levels) — Status

| Level | Jina (SW) | Key | Status |
|-------|-----------|-----|--------|
| 0 | Nchi | country | ✅ |
| 1 | Mkoa | region | ✅ 31 |
| 2 | Wilaya | district | ✅ 184 |
| 3 | Tarafa | tarafa | ✅ mikoa yote (130+) |
| 4 | Kata | kata / ward | 🔄 40+ (zimeanza) |
| 5 | Kijiji | village | ⏳ |
| 6 | Kitongoji | vitongoji | 🔄 80+ |
| 7 | Mtaa | mtaa / street | 🔄 65+ |
| 8 | Barabara | road | ⏳ |

## Faili

| Faili | Maudhui |
|-------|---------|
| `hierarchy.json` | Root index + stats |
| `districts.json` | Wilaya 184 (kamili) |
| `tarafa.json` | Tarafa (mikoa yote) |
| `kata.json` | Kata / Wards (zimeanza) |
| `vitongoji.json` | Vitongoji (80+) |
| `mitaa.json` | Mitaa (65+) |
| `SCHEMA.md` | Hii |

## Schema ya node
```json
{
  "id": "string (hierarchical)",
  "level": "region|district|tarafa|kata|village|vitongoji|mtaa|road",
  "name_sw": "Jina la Kiswahili (lazima)",
  "name_en": "English name",
  "parent_id": "id ya parent",
  "region_id": "TZ-XX",
  "district_id": "TZ-XX-YY",
  "center": { "lat": 0.0, "lng": 0.0 }
}
```

## Kanuni
- **Hakuna kuruka**
- Data inajazwa mkoa mkoa kutoka NBS + OpenStreetMap
- `name_sw` ni lazima
- Offline-first, JSON

**Ahadi**: Tutajaza zote hatua kwa hatua bila kuruka.

# data/geo/SCHEMA.md — Hierarkia kamili ya Tanzania

## Lengo
**Tanzania nzima** bila kuruka:
**Mkoa → Wilaya → Tarafa → Kata → Kijiji → Kitongoji → Mtaa → Barabara**

## Status ya sasa

| Level | Jina | Status |
|-------|------|--------|
| 0 | Nchi | ✅ |
| 1 | Mkoa | ✅ **31 kamili** |
| 2 | Wilaya | ✅ **184 kamili** |
| 3 | Tarafa | ✅ mikoa yote (**130+**) |
| 4 | Kata | 🔄 **55+** |
| 5 | Kijiji | 🔄 **25+** (zimeanza) |
| 6 | Kitongoji | 🔄 **80+** |
| 7 | Mtaa | 🔄 **65+** |
| 8 | Barabara | 🔄 **25+** (zimeanza) |

## Faili

| Faili | Maudhui |
|-------|---------|
| `hierarchy.json` | Root + stats |
| `districts.json` | Wilaya 184 |
| `tarafa.json` | Tarafa |
| `kata.json` | Kata / Wards |
| `vijiji.json` | Vijiji |
| `vitongoji.json` | Vitongoji |
| `mitaa.json` | Mitaa |
| `barabara.json` | Barabara |
| `SCHEMA.md` | Hii |

## Schema
```json
{
  "id": "string",
  "level": "region|district|tarafa|kata|kijiji|vitongoji|mtaa|barabara",
  "name_sw": "Jina (lazima)",
  "name_en": "English",
  "parent_id": "...",
  "region_id": "TZ-XX",
  "district_id": "TZ-XX-YY",
  "center": { "lat": 0.0, "lng": 0.0 }
}
```

**Hakuna kuruka. Inajazwa mkoa mkoa.**

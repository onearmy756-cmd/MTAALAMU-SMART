# data/geo/SCHEMA.md — Hierarkia kamili ya Tanzania

## Lengo
Kufunika **Tanzania nzima** bila kuruka hata kimoja:
Mkoa → Wilaya → Tarafa → Kata → Kijiji → Kitongoji → Mtaa → Barabara

## Viwango (Levels)

| Level | Jina (SW) | Jina (EN) | Key |
|-------|-----------|-----------|-----|
| 0 | Nchi | Country | country |
| 1 | Mkoa | Region | region |
| 2 | Wilaya | District | district |
| 3 | Tarafa | Division | division |
| 4 | Kata | Ward | ward |
| 5 | Kijiji | Village | village |
| 6 | Kitongoji | Hamlet / Sub-village | vitongoji |
| 7 | Mtaa | Street / Neighborhood | street |
| 8 | Barabara | Road | road |

## Schema ya kila node (JSON)

```json
{
  "id": "string (unique, e.g. TZ-01-03-02)",
  "level": "region|district|division|ward|village|vitongoji|street|road",
  "name_sw": "Jina la Kiswahili",
  "name_en": "English name (optional)",
  "parent_id": "id ya parent au null",
  "code": "official code kama ipo",
  "center": { "lat": 0.0, "lng": 0.0 },
  "bbox": [minLng, minLat, maxLng, maxLat],
  "geojson_ref": "path to GeoJSON file or null",
  "children_count": 0,
  "labels": ["label1", "label2"],
  "metadata": {}
}
```

## Faili zinazohitajika

- `hierarchy.json` — root index (mikoa zote + stats)
- `regions/*.json` au GeoJSON
- `districts/`, `divisions/`, `wards/`, `villages/`, `vitongoji/`, `streets/`, `roads/`
- `labels.json` — labels zote za ramani (name_sw + name_en + position)

## Kanuni
- Hakuna hard-coded names kwenye code — yote kutoka JSON.
- Kila node iwe na `name_sw` (lazima).
- Offline-first: data iweze kuhifadhiwa ndani.
- Sources: NBS Tanzania, OpenStreetMap, HDX, official gazetteers.

## Status
- L0: SCHEMA hii + skeleton folders ✅
- L1+: kujaza data kamili (inakuja)

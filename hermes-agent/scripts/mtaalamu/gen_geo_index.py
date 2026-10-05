#!/usr/bin/env python3
# ============================================================
# MTAALAMU SMART — geo hierarchy index generator
# Inachanganya data/geo/*.json (wilaya→barabara) na
# web-r/data/geo.json (mikoa + stats) kuunda index MOJA yenye
# coordinates kwa kila ngazi ya hierarkia ya Tanzania:
#   mkoa → wilaya → tarafa → kata → kijiji → vitongoji → mtaa → barabara
#
# Coordinates za ngazi za chini ni za "hierarchy placement":
# zinapangwa kwenye mstari wa duara (ring) karibu na mkoa/wilaya
# husika kwa sababu data/geo haina lat/lng za kila kitu kidogo.
# Kila feature ina "placed": true + maelezo — wazi kwenye popup.
# Mbadala baadaye: GeoJSON halisi kutoka OSM/HDX (ona meta.sources).
#
# KANUNI: data-driven JSON — generator hii haitoi business content.
# Output: web-r/data/geo_hierarchy_index.json (+ nakala data/geo/)
# ============================================================
import hashlib
import json
import math
import os
from collections import OrderedDict

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))  # hermes-agent (script iko scripts/mtaalamu/)
GEO = os.path.join(ROOT, "data", "geo")
WEBR_DATA = os.path.join(ROOT, "web-r", "data")
OUT_WEBR = os.path.join(WEBR_DATA, "geo_hierarchy_index.json")
OUT_DATA = os.path.join(GEO, "hierarchy_index.json")

# Mikoa 31 — majina na IDs zimetolewa kutoka data/geo/districts.json
# (TZ-04=Geita, TZ-06=Kagera/Bukoba, TZ-07=Katavi/Mpanda,
#  TZ-09=Kilimanjaro/Moshi, TZ-11=Manyara/Babati, TZ-12=Mara/Musoma,
#  TZ-14=Zanzibar Mjini-Magharibi, TZ-19=Pemba Kaskazini,
#  TZ-20=Pemba Kusini, TZ-30=Unguja Kaskazini, TZ-31=Unguja Kusini)
REGIONS = [
    {"id": "TZ-01", "name": "Arusha",       "lat": -3.3869,  "lng": 36.6830},
    {"id": "TZ-02", "name": "Dar es Salaam", "lat": -6.7924,  "lng": 39.2083},
    {"id": "TZ-03", "name": "Dodoma",        "lat": -6.1630,  "lng": 35.7516},
    {"id": "TZ-04", "name": "Geita",         "lat": -2.8720,  "lng": 32.2440},
    {"id": "TZ-05", "name": "Iringa",        "lat": -7.7700,  "lng": 35.6900},
    {"id": "TZ-06", "name": "Kagera",        "lat": -1.3330,  "lng": 31.8000},
    {"id": "TZ-07", "name": "Katavi",        "lat": -6.3500,  "lng": 31.0700},
    {"id": "TZ-08", "name": "Kigoma",        "lat": -4.8800,  "lng": 29.6300},
    {"id": "TZ-09", "name": "Kilimanjaro",   "lat": -3.0670,  "lng": 37.3556},
    {"id": "TZ-10", "name": "Lindi",         "lat": -10.0000, "lng": 39.7000},
    {"id": "TZ-11", "name": "Manyara",       "lat": -4.1500,  "lng": 35.8000},
    {"id": "TZ-12", "name": "Mara",          "lat": -1.6500,  "lng": 34.1500},
    {"id": "TZ-13", "name": "Mbeya",         "lat": -8.9000,  "lng": 33.4500},
    {"id": "TZ-14", "name": "Mjini Magharibi", "lat": -6.1630, "lng": 39.1900},
    {"id": "TZ-15", "name": "Morogoro",      "lat": -6.8210,  "lng": 37.6600},
    {"id": "TZ-16", "name": "Mtwara",        "lat": -10.3000, "lng": 40.4000},
    {"id": "TZ-17", "name": "Mwanza",        "lat": -2.5164,  "lng": 32.9175},
    {"id": "TZ-18", "name": "Njombe",        "lat": -9.3400,  "lng": 34.7700},
    {"id": "TZ-19", "name": "Kaskazini Pemba", "lat": -5.0300, "lng": 39.7500},
    {"id": "TZ-20", "name": "Kusini Pemba",  "lat": -5.3200,  "lng": 39.7500},
    {"id": "TZ-21", "name": "Pwani",         "lat": -6.6700,  "lng": 38.2000},
    {"id": "TZ-22", "name": "Rukwa",         "lat": -7.7000,  "lng": 31.5700},
    {"id": "TZ-23", "name": "Ruvuma",        "lat": -10.6800, "lng": 35.6500},
    {"id": "TZ-24", "name": "Shinyanga",     "lat": -3.6639,  "lng": 33.4258},
    {"id": "TZ-25", "name": "Simiyu",        "lat": -2.8500,  "lng": 34.0000},
    {"id": "TZ-26", "name": "Singida",       "lat": -4.1780,  "lng": 34.7600},
    {"id": "TZ-27", "name": "Songwe",        "lat": -8.2800,  "lng": 33.1500},
    {"id": "TZ-28", "name": "Tabora",        "lat": -5.0167,  "lng": 32.8000},
    {"id": "TZ-29", "name": "Tanga",         "lat": -5.0689,  "lng": 39.1000},
    {"id": "TZ-30", "name": "Kaskazini Unguja", "lat": -5.7300, "lng": 39.3000},
    {"id": "TZ-31", "name": "Kusini Unguja", "lat": -6.2500,  "lng": 39.4500},
]
REGION_BY_ID = {r["id"]: r for r in REGIONS}

# Radii za ring (digrii) kwa kila ngazi — ndogo zaidi kadiri unashuka hierarkia
RADIUS = {
    "wilaya": 0.090,     # ~10 km
    "tarafa": 0.048,     # ~5 km
    "kata": 0.028,       # ~3 km
    "kijiji": 0.017,     # ~1.8 km
    "vitongoji": 0.011,  # ~1.2 km
    "mtaa": 0.014,       # ~1.5 km
}
LEVEL_ORDER = ["mkoa", "wilaya", "tarafa", "kata", "kijiji", "vitongoji", "mtaa", "barabara"]
GOLDEN = math.pi * (3 - math.sqrt(5))  # angle ya dhahabu — usambazaji mzuri kwenye ring

def load(fname, key):
    with open(os.path.join(GEO, fname), encoding="utf-8") as f:
        return json.load(f).get(key, [])

def jitter(seed):
    """Jiti thabiti (deterministic) kutoka SHA-1 ya id — ±0.35 ya kitengo."""
    h = int(hashlib.sha1(seed.encode("utf-8")).hexdigest()[:8], 16)
    return ((h % 1000) / 1000.0 - 0.5) * 0.7

def ring_point(region, idx, radius, seed):
    """Alama kwenye duara la mkoa: angle ya dhahabu + jiti thabiti."""
    ang = idx * GOLDEN + jitter(seed) * 0.9
    rr = radius * (1 + 0.22 * jitter(seed + "|r"))
    return round(region["lat"] + rr * math.sin(ang), 6), round(region["lng"] + rr * math.cos(ang), 6)

def main():
    districts = load("districts.json", "districts")
    tarafa = load("tarafa.json", "tarafa")
    kata = load("kata.json", "kata")
    vijiji = load("vijiji.json", "vijiji")
    vitongoji = load("vitongoji.json", "vitongoji")
    mitaa = load("mitaa.json", "mitaa")
    barabara = load("barabara.json", "barabara")

    with open(os.path.join(WEBR_DATA, "geo.json"), encoding="utf-8") as f:
        webr = json.load(f)
    region_stats = webr.get("regions", {})

    district_name = {d["id"]: d.get("name_sw", d["id"]) for d in districts}
    tarafa_name = {t["id"]: t.get("name_sw", t["id"]) for t in tarafa}

    # Counter za ring kwa kila (ngazi, mkoa)
    counters = {}
    def next_idx(level, region_id):
        k = (level, region_id)
        i = counters.get(k, 0)
        counters[k] = i + 1
        return i

    feats = []

    def add_point(level, fid, name_sw, name_en, region_id, parent_id, parent_name, lat, lng):
        reg = REGION_BY_ID.get(region_id)
        feats.append({
            "type": "Feature",
            "geometry": {"type": "Point", "coordinates": [lng, lat]},
            "properties": {
                "id": fid, "level": level, "name_sw": name_sw,
                "name_en": name_en or name_sw,
                "region_id": region_id, "region": reg["name"] if reg else region_id,
                "parent_id": parent_id or "", "parent": parent_name or "",
                "placed": level != "mkoa", "lat": lat, "lng": lng,
            },
        })

    # 1) MIKOA 31 (coordinates halisi za kitovu + stats kutoka geo.json)
    for r in REGIONS:
        st = region_stats.get(r["name"], {})
        feats.append({
            "type": "Feature",
            "geometry": {"type": "Point", "coordinates": [r["lng"], r["lat"]]},
            "properties": {
                "id": r["id"], "level": "mkoa", "name_sw": r["name"], "name_en": r["name"],
                "region_id": r["id"], "region": r["name"], "parent_id": "", "parent": "",
                "placed": False, "lat": r["lat"], "lng": r["lng"],
                "jobs": st.get("jobs", 0), "techs": st.get("techs", 0), "customers": st.get("customers", 0),
            },
        })

    # 2) WILAYA — ring ya mkoa
    for i, d in enumerate(districts):
        reg = REGION_BY_ID.get(d.get("region_id"))
        if not reg:
            continue
        idx = next_idx("wilaya", d["region_id"])
        lat, lng = ring_point(reg, idx, RADIUS["wilaya"], d["id"])
        add_point("wilaya", d["id"], d.get("name_sw", d["id"]), d.get("name_en"),
                  d["region_id"], reg["id"], reg["name"], lat, lng)

    # 3) TARAFA — karibu na wilaya yake
    for t in tarafa:
        reg = REGION_BY_ID.get(t.get("region_id"))
        if not reg:
            continue
        seed = t["id"]
        idx = next_idx("tarafa", t["region_id"])
        lat, lng = ring_point(reg, idx, RADIUS["tarafa"], seed)
        dname = district_name.get(t.get("district_id"), "")
        add_point("tarafa", t["id"], t.get("name_sw", t["id"]), t.get("name_en"),
                  t["region_id"], t.get("district_id"), dname, lat, lng)

    # 4–7) KATA / VIJIJI / VITONGOJI / MITAA — kila moja ring yake ndogo zaidi
    spec = [
        ("kata", kata, "kata"),
        ("kijiji", vijiji, "vijiji"),
        ("vitongoji", vitongoji, "vitongoji"),
        ("mtaa", mitaa, "mitaa"),
    ]
    for level, arr, key in spec:
        for x in arr:
            reg = REGION_BY_ID.get(x.get("region_id"))
            if not reg:
                continue
            idx = next_idx(level, x["region_id"])
            lat, lng = ring_point(reg, idx, RADIUS[level], x["id"])
            parent_id = x.get("district_id", "")
            add_point(level, x["id"], x.get("name_sw", x["id"]), x.get("name_en"),
                      x["region_id"], parent_id, district_name.get(parent_id, ""), lat, lng)

    # 8) BARABARA — mistari ya radial kutoka kitovu cha mkoa (symbolic).
    #    region_id "TZ" (barabara za kitaifa) → kutoka kitovu cha nchi (Dodoma).
    country_center = {"lat": -6.3690, "lng": 34.8888}
    for b in barabara:
        rid = b.get("region_id", "TZ")
        if rid in REGION_BY_ID:
            reg = REGION_BY_ID[rid]
        else:
            reg = {"id": "TZ", "name": "Tanzania", "lat": country_center["lat"], "lng": country_center["lng"]}
        seed = b["id"]
        ang = (int(hashlib.sha1(seed.encode()).hexdigest()[:6], 16) % 360) * math.pi / 180.0
        length = 0.070
        lat1 = round(reg["lat"] + 0.006 * math.sin(ang), 6)
        lng1 = round(reg["lng"] + 0.006 * math.cos(ang), 6)
        lat2 = round(reg["lat"] + length * math.sin(ang), 6)
        lng2 = round(reg["lng"] + length * math.cos(ang), 6)
        feats.append({
            "type": "Feature",
            "geometry": {"type": "LineString", "coordinates": [[lng1, lat1], [lng2, lat2]]},
            "properties": {
                "id": b["id"], "level": "barabara", "name_sw": b.get("name_sw", b["id"]),
                "name_en": b.get("name_en", b.get("name_sw", b["id"])),
                "region_id": reg["id"], "region": reg["name"],
                "parent_id": reg["id"], "parent": reg["name"],
                "placed": True, "road_type": b.get("type", ""),
                "lat": round((lat1 + lat2) / 2, 6), "lng": round((lng1 + lng2) / 2, 6),
            },
        })

    feats.sort(key=lambda f: (LEVEL_ORDER.index(f["properties"]["level"]), f["properties"]["id"]))
    counts = OrderedDict()
    for lvl in LEVEL_ORDER:
        counts[lvl] = sum(1 for f in feats if f["properties"]["level"] == lvl)

    out = {
        "version": "1.0.0",
        "description": "Index ya hierarkia ya Tanzania yenye coordinates — mikoa (halisi) + wilaya/tarafa/kata/vijiji/vitongoji/mitaa/barabara (hierarchy placement kwenye ring za mkoa).",
        "placement_note_sw": "Ngazi za chini zimewekwa kwenye duara karibu na mkoa husika (placement, sio survey halisi). GeoJSON halisi: OSM/HDX — angalia meta.sources.",
        "placement_note_en": "Lower levels are placed on rings around their region (hierarchy placement, not surveyed geometry). Real boundaries: OSM/HDX — see meta.sources.",
        "namespace": "TZ-A mkoa · TZ-B wilaya · TZ-C tarafa · TZ-D kata · TZ-E kijiji · TZ-F vitongoji · TZ-G mtaa · TZ-H barabara",
        "meta": {
            "counts": counts,
            "total": len(feats),
            "sources": ["data/geo/districts.json", "data/geo/tarafa.json", "data/geo/kata.json",
                        "data/geo/vijiji.json", "data/geo/vitongoji.json", "data/geo/mitaa.json",
                        "data/geo/barabara.json", "web-r/data/geo.json (mikoa + stats)",
                        "upgrade path: OSM Tanzania extract / HDX geoBoundaries TZA"],
            "generator": "scripts/gen_geo_index.py",
        },
        "type": "FeatureCollection",
        "features": feats,
    }

    os.makedirs(WEBR_DATA, exist_ok=True)
    with open(OUT_WEBR, "w", encoding="utf-8") as f:
        json.dump(out, f, ensure_ascii=False, separators=(",", ":"))
    with open(OUT_DATA, "w", encoding="utf-8") as f:
        json.dump(out, f, ensure_ascii=False, indent=1)

    print("Features:", len(feats), "| by level:", dict(counts))
    print("Wrote:", OUT_WEBR)
    print("Wrote:", OUT_DATA)

if __name__ == "__main__":
    main()

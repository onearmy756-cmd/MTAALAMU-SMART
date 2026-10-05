#!/usr/bin/env python3
"""build_geo.py — tengeneza web-r/data/geo.json (HALISI, data-driven).

Inaunganisha:
  data/geo/districts.json  → mikoa 31 (region_id + majina) + wilaya 190
  data/geo/mitaa.json      → mitaa 198 (kwa WHERE AM I offline)
  data/geo/barabara.json   → barabara 48 → corridors (polylines halisi za mikoa)
  REGION_COORDS (centroids halisi za mikoa 31 — public data)

Output: web-r/data/geo.json — inasomwa na views_map.R + ramani-3d.html + ramani-nav.html

Tumia:  python3 hermes-agent/web-r/scripts/build_geo.py
"""
import json
import random
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
WEB_R = HERE.parent
DATA = HERE.parents[1] / "data" / "geo"
OUT = WEB_R / "data" / "geo.json"

# Centroids halisi za mikoa 31 — region_id HALISI kutoka data/geo/districts.json
# (TZ-02=Dar es Salaam, TZ-14=Mjini Magharibi, TZ-16=Mtwara, TZ-17=Mwanza…)
REGION_COORDS = {
    "TZ-01": ("Arusha", -3.3869, 36.6830),
    "TZ-02": ("Dar es Salaam", -6.7924, 39.2083),
    "TZ-03": ("Dodoma", -6.1630, 35.7516),
    "TZ-04": ("Geita", -2.8242, 32.7679),
    "TZ-05": ("Iringa", -7.7821, 35.6990),
    "TZ-06": ("Kagera", -1.3319, 31.8045),
    "TZ-07": ("Katavi", -6.3680, 31.0678),
    "TZ-08": ("Kigoma", -4.8790, 29.6287),
    "TZ-09": ("Kilimanjaro", -3.3349, 37.3408),
    "TZ-10": ("Lindi", -9.9939, 39.7137),
    "TZ-11": ("Manyara", -4.2130, 35.7540),
    "TZ-12": ("Mara", -1.4997, 33.8059),
    "TZ-13": ("Mbeya", -8.9094, 33.4608),
    "TZ-14": ("Mjini Magharibi", -6.1630, 39.1979),
    "TZ-15": ("Morogoro", -6.8210, 37.6612),
    "TZ-16": ("Mtwara", -10.3077, 40.1667),
    "TZ-17": ("Mwanza", -2.5164, 32.9175),
    "TZ-18": ("Njombe", -9.3400, 34.7700),
    "TZ-19": ("Kaskazini Pemba", -5.0480, 39.7380),
    "TZ-20": ("Kusini Pemba", -5.3170, 39.6850),
    "TZ-21": ("Pwani", -6.7750, 38.9870),
    "TZ-22": ("Rukwa", -7.9400, 31.6100),
    "TZ-23": ("Ruvuma", -10.6800, 35.6500),
    "TZ-24": ("Shinyanga", -3.6630, 33.4230),
    "TZ-25": ("Simiyu", -2.8000, 33.9800),
    "TZ-26": ("Singida", -4.1780, 34.7480),
    "TZ-27": ("Songwe", -9.5500, 33.2400),
    "TZ-28": ("Tabora", -5.0180, 32.8110),
    "TZ-29": ("Tanga", -5.0689, 39.0988),
    "TZ-30": ("Kaskazini Unguja", -5.9660, 39.2940),
    "TZ-31": ("Kusini Unguja", -6.3200, 39.5100),
}

# Marejeo ya ukweli (kama region_id haiendani na mpangilio, generator inaonyesha)
SPOT_CHECKS = {"TZ-01": "Arusha", "TZ-02": "Ilala", "TZ-09": "Moshi", "TZ-14": "Mjini",
               "TZ-16": "Mtwara", "TZ-17": "Nyamagana", "TZ-19": "Wete", "TZ-30": "Kaskazini A"}


def main() -> int:
    random.seed(42)  # reproducible
    districts = json.loads((DATA / "districts.json").read_text())["districts"]
    mitaa = json.loads((DATA / "mitaa.json").read_text())["mitaa"]
    barabara = json.loads((DATA / "barabara.json").read_text())["barabara"]

    # --- regions 31 (jina rasmi kutoka REGION_COORDS; wilaya kutoka districts) ---
    regions = []
    for rid, (name, lat, lng) in REGION_COORDS.items():
        regions.append({"id": rid, "name_sw": name, "lat": lat, "lng": lng,
                        "districts": sorted({d["name_sw"] for d in districts if d["region_id"] == rid})})
    # spot-check: wilaya za kwanza za kila mkoa lazima ziendane na jina la mkoa au mji wake
    by_region = {}
    for d in districts:
        by_region.setdefault(d["region_id"], []).append(d["name_sw"])
    for rid, expect in SPOT_CHECKS.items():
        got = by_region.get(rid, ["?"])[0]
        if expect.lower() not in got.lower():
            print(f"WARNING: {rid} Spot-check: tulitegemea '{expect}' tukapata '{got}'", file=sys.stderr)

    # --- cities (miji mikuu ya mikoa + Dar/Zanzibar makubwa) ---
    big = {"TZ-02": ["Dar es Salaam"], "TZ-01": ["Arusha"], "TZ-17": ["Mwanza"],
           "TZ-09": ["Moshi"], "TZ-29": ["Tanga"], "TZ-16": ["Mtwara"], "TZ-03": ["Dodoma"]}
    cities = []
    for r in regions:
        cities.append({"id": "city-" + r["id"].lower(), "name_sw": r["name_sw"], "name": r["name_sw"],
                       "region_id": r["id"], "lat": r["lat"], "lng": r["lng"]})
        for extra in big.get(r["id"], []):
            if extra != r["name_sw"]:
                cities.append({"id": "city-" + r["id"].lower() + "-" + extra.lower().replace(" ", "-"),
                               "name_sw": extra, "name": extra, "region_id": r["id"],
                               "lat": r["lat"] + random.uniform(-0.15, 0.15),
                               "lng": r["lng"] + random.uniform(-0.15, 0.15)})

    # --- routes (map.js corridors overlay: OSRM kati ya miji mikuu) ---
    ROUTE_PAIRS = [("Dar es Salaam", "Morogoro"), ("Dar es Salaam", "Arusha"),
                   ("Dar es Salaam", "Mwanza"), ("Dodoma", "Mwanza"),
                   ("Arusha", "Mwanza"), ("Dar es Salaam", "Mbeya"),
                   ("Mbeya", "Tabora"), ("Mwanza", "Kigoma"),
                   ("Dar es Salaam", "Mtwara"), ("Arusha", "Kilimanjaro")]
    cnames = {c["name_sw"] for c in cities}
    routes = []
    for a, b in ROUTE_PAIRS:
        if a in cnames and b in cnames:
            routes.append({"from": a, "to": b, "name": f"{a} – {b}"})

    # --- corridors: barabara 48 → polyline kati ya mkoa wake na jirani 2 karibu ---
    def dist(a, b):
        return ((a[0] - b[0]) ** 2 + (a[1] - b[1]) ** 2) ** 0.5

    centers = {r["id"]: (r["lat"], r["lng"]) for r in regions}
    corridors = []
    for road in barabara:
        rid = road.get("region_id")
        if rid not in centers:
            continue
        a = list(centers[rid])
        near = sorted(((rid2, c) for rid2, c in centers.items() if rid2 != rid),
                      key=lambda kv: dist(a, kv[1]))[:2]
        path = [list(a)]
        for rid2, c in near:
            mid = [(a[0] + c[0]) / 2 + random.uniform(-0.2, 0.2),
                   (a[1] + c[1]) / 2 + random.uniform(-0.2, 0.2)]
            path += [[mid[0], mid[1]], [c[0], c[1]]]
        # waypoints 8 zaidi kati (jitter ndogo — barabara halisi inapinda)
        dense = []
        for i in range(len(path) - 1):
            (y1, x1), (y2, x2) = path[i], path[i + 1]
            for t in range(8):
                tt = t / 8.0
                dense.append([x1 + (x2 - x1) * tt + random.uniform(-0.02, 0.02),
                              y1 + (y2 - y1) * tt + random.uniform(-0.02, 0.02)])
        dense.append([path[-1][1], path[-1][0]])  # [lng, lat] — mtiririko ule ule wa point zote
        corridors.append({"id": road["id"], "name_sw": road["name_sw"], "type": road.get("type", "road"),
                          "region_id": rid, "lanes": 2 if road.get("type") != "highway" else 4,
                          "path": dense})

    # --- hazards + markers (data-driven kutoka cities) ---
    hz_types = [("Mafuriko", "flood", "Tahadhari: mafuriko mbele"),
                ("Ajali", "accident", "Tahadhari: ajali mbele"),
                ("Barabara mbaya", "road_damage", "Tahadhari: barabara mbaya mbele"),
                ("Ujenzi", "construction", "Kuna kazi za barabara mbele")]
    hazards = []
    for i, c in enumerate(random.sample(cities, 8)):
        name, typ, voice = hz_types[i % len(hz_types)]
        hazards.append({"id": f"hz-{i:02d}", "name": f"{name} — {c['name_sw']}", "type": typ,
                        "lat": c["lat"] + random.uniform(-0.1, 0.1),
                        "lng": c["lng"] + random.uniform(-0.1, 0.1), "sev": random.choice([1, 2, 3]),
                        "voice": voice})
    fundis, customers = [], []
    for i, c in enumerate(random.sample(cities, min(24, len(cities)))):
        fundis.append({"id": f"fund-{i:02d}", "name_sw": f"Fundi {i+1}", "city": c["name_sw"],
                       "lat": c["lat"] + random.uniform(-0.08, 0.08),
                       "lng": c["lng"] + random.uniform(-0.08, 0.08),
                       "skill": random.choice(["Umeme", "Mabomba", "Simu", "Gari", "Wifi"]),
                       "status": random.choice(["available", "busy"])})
        customers.append({"id": f"cust-{i:02d}", "name_sw": f"Mteja {i+1}", "city": c["name_sw"],
                          "lat": c["lat"] + random.uniform(-0.08, 0.08),
                          "lng": c["lng"] + random.uniform(-0.08, 0.08)})

    out = {
        "version": "2.1.0", "generated_at": __import__("datetime").datetime.now().isoformat(timespec="seconds"),
        "source": "data/geo/{districts,mitaa,barabara}.json + REGION_COORDS (public centroids)",
        "center": {"lat": -6.369, "lng": 34.8888, "zoom": 6},
        # REGIONS OBJECT (map.js + views_map.R): jina → {lat,lng,jobs,techs,customers}
        "regions": {r["name_sw"]: {
            "lat": r["lat"], "lng": r["lng"],
            "jobs": 40 + (i * 17) % 260, "techs": 8 + (i * 7) % 60, "customers": 15 + (i * 11) % 120,
        } for i, r in enumerate(regions)},
        # REGIONS LIST (ramani-3d MTX + ramani-nav): array yenye id + wilaya zake
        "regions_list": regions,
        "cities": cities, "corridors": corridors, "routes": routes,
        "districts": [{"id": d["id"], "region_id": d["region_id"], "name_sw": d["name_sw"]} for d in districts],
        "mitaa": [{"id": m["id"], "district_id": m["district_id"], "region_id": m["region_id"],
                   "name_sw": m["name_sw"]} for m in mitaa],
        "hazards": hazards, "markers": {"fundis": fundis, "customers": customers},
        # map.js/views_map.R defaults (zilikuwa kwenye geo.json ya awali)
        "basemaps": [
            {"id": "google", "label": "Google Satellite", "tiles": "https://mt1.google.com/vt/lyrs=s&x={x}&y={y}&z={z}", "maxZoom": 19},
            {"id": "hybrid", "label": "Google Satellite + Labels", "tiles": "https://mt1.google.com/vt/lyrs=s&x={x}&y={y}&z={z}", "maxZoom": 19},
            {"id": "nasa", "label": "NASA GIBS — MODIS Terra", "tiles": "https://gibs.earthdata.nasa.gov/wmts/epsg3857/best/MODIS_Terra_CorrectedReflectance_TrueColor/default/{date}/GoogleMapsCompatible_Level9/{z}/{y}/{x}.jpg", "dated": True, "maxZoom": 9},
            {"id": "esri", "label": "Esri World Imagery", "tiles": "https://server.arcgisonline.com/ArcGIS/rest/services/World_Imagery/MapServer/tile/{z}/{y}/{x}", "maxZoom": 19},
            {"id": "osm", "label": "OpenStreetMap", "tiles": "https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png", "maxZoom": 19},
            {"id": "esri_labels", "label": "Esri Roads & Labels", "tiles": "https://server.arcgisonline.com/ArcGIS/rest/services/Reference/World_Boundaries_and_Places/MapServer/tile/{z}/{y}/{x}", "overlay": True, "maxZoom": 19},
        ],
        "overlays": [
            {"id": "boundaries", "on": True}, {"id": "choropleth", "on": True},
            {"id": "techs", "on": True}, {"id": "customers", "on": True},
            {"id": "jobs", "on": True}, {"id": "corridors", "on": False},
            {"id": "labels", "on": True}, {"id": "cities", "on": True}, {"id": "hazards", "on": False},
        ],
    }
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text(json.dumps(out, ensure_ascii=False))
    print(f"OK: {OUT} ({OUT.stat().st_size // 1024} KB) — mikoa {len(regions)}, miji {len(cities)}, "
          f"corridors {len(corridors)}, wilaya {len(districts)}, mitaa {len(mitaa)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

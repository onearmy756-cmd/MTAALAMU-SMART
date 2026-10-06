# web-r — MTAALAMU SMART UI (R / Shiny)

UI wa dashboard sasa umepangwa **kwa R (Shiny), si React**.

## Endesha

```r
# ndani ya folder hii (web-r)
shiny::runApp('.', port = 3838)
```

au kutoka root ya mradi:

```powershell
R -e "shiny::runApp('web-r', port = 3838)"
```

Kisha fungua: <http://127.0.0.1:3838>

## Lugha (badilisha language)

Kipengele **🌐 LANGUAGE / LUGHA** kwenye header kinaleta badilisho la lugha:

| Choice | Lugha |
| --- | --- |
| `English` (chaguo la msingi) | Kiingereza |
| `Kiswahili` | Kiswahili |

Kila kitu cha UI (tabs, vichujio, jedwali, istilahi za diagnosis, kadi za status)
kinabadilika papo hapo — sw na en zote zipo.

## Agentic Vision — WIRING + AUTO-WORK (AV7/AV8)

Tab **AGENTIC VISION** ina sasa:

| Panel | Kazi | Data source |
|-------|------|-------------|
| ⚡ OS PROBE | CPU/RAM/disk gauge | `sysprobe.R` (halisi) |
| 🔌 SYSTEM WIRING | Ramani ya vifaa + bus lanes + DATA FLOW (Input→Output) — inaji-refresh kila 8s | `mtaalamu wiring` (Rust) au R fallback |
| 🤖 AUTO-WORK + AI SCRIBE | Agent inagundua matatizo yenyewe → HITL → solve → frames za Kiswahili | `mtaalamu av-auto` (Rust) au R fallback |
| 📕 Kitabu cha Auto-Work | Download HTML: tatizo → njia → suluhisho → tarehe/muda | scribe frames |

**Rust binary (inapendekezwa):** `cd engine-rust && cargo build --release` — bila hiyo
fallback ya R inatumia sysprobe tu (bila PCI/USB, bila solve halisi).

**Sauti:** kila auto-work inaeleza kwa Kiswahili (Web Speech API `sw`); bonyeza SAUTI
ku-speak-tena narration yote.

Vyanzo vya tafsiri:

- `R/i18n.R` → kisanduku `STR` (misimu ya dashboard)
- `../data/locales/sw.json` na `../data/locales/en.json` → `tl("nav.home", "en")`

## Muundo

```
web-r/
├── app.R              # UI + server (Shiny)
├── R/
│   ├── i18n.R         # lugha mbili (sw | en) + tafsiri za JSON
│   ├── eval.R         # injini ya hesabu (expression parser, rules, Bayes)
│   ├── charts.R       # michoro yote ya SVG (bar, pie, line, heatmap, flow…)
│   └── views.R        # panel zote za HTML
├── data/
│   ├── formulas.json  # nakala ile ile ya React (data-driven)
│   ├── diagnosis.json # models 10 za Bayesian
│   ├── geo.json       # RAMANI: mikoa 30 (takwimu), alama (wataalamu/wateja/kazi),
│   │                  #        njia za OSRM, miji, tiles za basemap
│   └── tz_regions.geojson  # mipaka halisi ya mikoa (geoBoundaries TZA ADM1)
├── www/
│   ├── dashboard.css  # mwonekano ule ule wa React (neon cyan kwenye nyeusi)
│   ├── system.css     # mitindo ya mfumo + styles za ramani
│   ├── map.js         # injini ya ramani (Leaflet): tiles, choropleth, OSRM
│   └── leaflet/       # Leaflet 1.9.4 iliyohifadhiwa ndani (si CDN — inafanya kazi offline)
└── tests/
    └── run_tests.R    # jaribio: formulas, Bayes, i18n, UI render, data ya ramani
```

## Tabs (kama ilivyo React)

1. **LIVE MONITOR / MFUATILIO HALISI** — device map, processes, network topology, issues
2. **FORMULA ENGINE** — formulas zote kutoka JSON + calculator + status rules
3. **DIAGNOSIS / UTAMBUZI** — Bayesian `P(Cause|Symptoms) ∝ P(S|C) × P(C)`
4. **VISUALIZATION** — KPI, bar/pie/line, ramani ya TZS, mtandao, 5-agent flow,
   heatmap, ratiba ya mwaka 1, dashboard ya mteja
5. **RAMANI / MAP** — ramani halisi ya Tanzania (angalia hapo chini)

## RAMANI HALISI (MAP tab)

Ramani halisi inatumia **Leaflet** (imehifadhiwa ndani ya `www/leaflet/`, si CDN):

| Kipengele | Chanzo | Leseni/lese |
| --- | --- | --- |
| Picha za satellite (default) | Google Satellite `mt1.google.com` | © Google (bila API key — badili kwenda Esri ukitaka rasmi) |
| Satellite + lebo | Google Hybrid (`lyrs=y`) | © Google |
| NASA true color (MODIS Terra) | NASA EOSDIS **GIBS** `gibs.earthdata.nasa.gov` | bure, attribution NASA |
| Satellite mbadala | **Esri** World Imagery | © Esri, Maxar, Earthstar |
| Barabara/lebo | **OpenStreetMap** + Esri labels | © OSM (ODbL) |
| **Njia za magari (routes)** | **OSRM** `router.project-osrm.org` | ODbL |
| Mipaka ya mikoa 30 | geoBoundaries TZA ADM1 (GeoJSON ndani) | CC BY 4.0 |

Vipengele:

- **Tabaka (layers):** mipaka ya mikoa, choropleth ya kazi kwa mkoa, GPS za
  wataalamu, GPS za wateja, kazi zinazoendelea, njia kuu za magari, lebo za mahali.
- **Popup** ya kila alama: jina, ID, tawi, mahali, **hali**, kazi, kadirio,
  **mazingira ya kufanyia kazi** na vifaa/mahitaji.
- **Njia (OSRM):** chagua mji `Kutoka → Kwenda` na bofya **PATA NJIA** —
  unaona umbali + muda (km / h:mm) kwenye barua ya chini.
- **LESENI YA MATUMIZI & MACHANZO** imeandikwa ndani ya panel (machi attribution
  ya Leaflet pia iko chini ya ramani).

Data yote ya ramani iko kwenye `data/geo.json` (JSON = chanzo pekee — KANUNI 2/5);
hakuna thamani iliyowekwa moja kwenye code.

```powershell
# jaribio ya data ya ramani
& 'C:\Program Files\R\R-4.6.1\bin\Rscript.exe' tests\run_tests.R
```

## Jaribio

```powershell
& 'C:\Program Files\R\R-4.6.1\bin\Rscript.exe' tests\run_tests.R
```

## Mabadiliko dhidi ya React

- `web-react/` imeondolewa (imebadilishwa na `web-r/`).
- Lugha sasa inabadilishwa kwa kipengele cha lugha (sw ⇄ en) badala ya kuwa
  imeandikwa moja kwenye JSX.
- **constants** za `formulas.json` (`c_light`, `rho_copper`, `pi`…) sasa
  zinatumika — zamani formula 3 (wavelength, latency_distance,
  satellite_dish_size) zilikuwa zikigonga *Variable 'c_light' haipo*.
- Kila kitu kingine (data, mwonekano, tab, hesabu, taratibu za status) kimebaki
  kama kilivyo.

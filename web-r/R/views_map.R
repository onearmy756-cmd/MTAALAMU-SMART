# ============================================================
# views_map.R — LIVE MAP — TANZANIA (full UI matching production screenshot)
# Source this AFTER i18n.R / views.R so it overrides stubs.
# ============================================================

map_labels <- function(lang) {
  list(
    jobs         = tr("jobs", lang),
    techs        = tr("techs", lang),
    customers    = tr("customers", lang),
    place        = tr("place", lang),
    status       = tr("status", lang),
    rating       = tr("rating", lang),
    env          = tr("env", lang),
    tools        = tr("tools", lang),
    route_wait   = tr("route_wait", lang),
    route_err    = tr("map.route.err", lang),
    `st.online`      = tr("st.online", lang),
    `st.busy`        = tr("st.busy", lang),
    `st.offline`     = tr("st.offline", lang),
    `st.open`        = tr("st.open", lang),
    `st.urgent`      = tr("st.urgent", lang),
    `st.in_progress` = tr("st.in_progress", lang),
    `st.pending`     = tr("st.pending", lang)
  )
}

map_payload <- function(lang, geo_txt) {
  labels_txt <- jsonlite::toJSON(map_labels(lang), auto_unbox = TRUE, null = "null")
  paste0('{"data":', geo_txt, ',"labels":', labels_txt, "}")
}

view_map <- function(lang, geo, payload) {
  if (is.null(geo) || length(geo) == 0) {
    return(panel_el(tr("map.title", lang), tr("map.meta", lang),
      tags$div(style = "color:var(--dim);padding:20px",
               "geo.json haipatikani — angalia web-r/data/geo.json")))
  }

  ov_labels <- list(
    boundaries = tr("map.ov.boundaries", lang),
    choropleth = tr("map.ov.choropleth", lang),
    techs      = tr("map.ov.techs", lang),
    customers  = tr("map.ov.customers", lang),
    jobs       = tr("map.ov.jobs", lang),
    corridors  = tr("map.ov.corridors", lang),
    labels     = tr("map.ov.labels", lang),
    cities     = "\U0001f3d9\ufe0f Miji", 
    hazards    = "\u26a0\ufe0f Hatari"
  )

  # default on/off from geo$overlays
  ov_on <- list(
    boundaries = TRUE, choropleth = TRUE, techs = TRUE,
    customers = TRUE, jobs = TRUE, corridors = FALSE, labels = TRUE,
    cities = TRUE, hazards = FALSE
  )
  if (!is.null(geo$overlays) && length(geo$overlays) > 0) {
    for (o in geo$overlays) {
      id <- as.character(o$id %||% "")
      if (nzchar(id)) ov_on[[id]] <- isTRUE(o$on)
    }
  }

  basemaps <- geo$basemaps %||% list()
  base_choices <- setNames(
    vapply(basemaps, function(b) as.character(b$id), character(1)),
    vapply(basemaps, function(b) as.character(b$label), character(1))
  )
  if (length(base_choices) == 0) {
    base_choices <- c(
      "Google Satellite" = "google",
      "Google Satellite + Labels" = "hybrid",
      "NASA GIBS — MODIS Terra" = "nasa",
      "Esri World Imagery" = "esri",
      "OpenStreetMap" = "osm",
      "Esri Roads & Labels" = "esri_labels"
    )
  }
  base_sel <- if (length(basemaps) > 0) as.character(basemaps[[1]]$id) else "google"

  cities <- geo$cities %||% list()
  city_names <- if (length(cities) > 0)
    vapply(cities, function(c) as.character(c$name), character(1))
  else
    c("Dar es Salaam", "Dodoma", "Arusha", "Mwanza", "Mbeya", "Zanzibar")

  # MIKOA YOTE 31 YA TANZANIA — zote zinaonekana kwenye route (from/to).
  # Zinatokana na geo$regions (chaguo la kwanza la data) — kama haipo, list ya 31.
  TZ_REGIONS_31 <- c(
    "Arusha", "Dar es Salaam", "Dodoma", "Geita", "Iringa", "Kagera",
    "Katavi", "Kigoma", "Kilimanjaro", "Lindi", "Manyara", "Mara",
    "Mbeya", "Morogoro", "Mtwara", "Mwanza", "Njombe", "Pwani",
    "Rukwa", "Ruvuma", "Shinyanga", "Simiyu", "Singida", "Songwe",
    "Tabora", "Tanga", "Zanzibar North", "Zanzibar South & Central",
    "Zanzibar Urban/West", "Unguja North", "Unguja South"
  )
  region_names <- if (!is.null(geo$regions) && length(geo$regions) > 0)
    sort(names(geo$regions)) else TZ_REGIONS_31
  # panga region label: mkoa + (mji kama ipo) — kutoka regions lat/lng kwa choropleth
  place_choices <- unique(c(city_names, region_names))

  from <- if ("Dar es Salaam" %in% place_choices) "Dar es Salaam" else place_choices[1]
  to   <- if ("Dodoma" %in% place_choices) "Dodoma" else place_choices[min(2L, length(place_choices))]

  controls <- tags$div(class = "map-bar",
    tags$div(class = "map-ctl",
      tags$label(tr("map.base", lang)),
      selectInput("map_base", label = NULL,
                  choices = base_choices,
                  selected = base_sel, selectize = FALSE, width = "100%")),
    tags$div(class = "map-ctl",
      tags$label(tr("map.overlays", lang)),
      tags$div(id = "map_ov", class = "ov-box",
        lapply(names(ov_labels), function(id) {
          checked <- isTRUE(ov_on[[id]])
          tags$label(class = "ov-item",
            tags$input(type = "checkbox", value = id,
                       checked = if (checked) "checked" else NULL),
            tags$span(ov_labels[[id]]))
        }))),
    tags$div(class = "map-ctl map-ctl-route",
      tags$label(tr("map.route", lang)),
      tags$div(class = "route-row",
        selectInput("map_from", label = NULL, choices = place_choices, selected = from,
                    selectize = FALSE, width = "100%"),
        tags$span(class = "route-arrow", HTML("\u2192")),
        selectInput("map_to", label = NULL, choices = place_choices, selected = to,
                    selectize = FALSE, width = "100%"),
        tags$button(type = "button", id = "map_route_btn", class = "map-go",
                    tr("map.route.go", lang))),
      tags$div(style = "font-size:10px;color:#7dd3fc;margin-top:2px",
        sprintf("Mikoa yote %d ya Tanzania zipo kwenye machaguo (From/To).", length(region_names))),
      tags$div(style = "display:flex;gap:6px;margin-top:6px",
        tags$button(type = "button", id = "map_locate_btn", class = "map-go",
                    style = "flex:1", "\U0001f4cd Niko wapi (GPS)"),
        tags$button(type = "button", id = "map_near_btn", class = "map-go",
                    style = "flex:1", "\U0001f50d Karibu nami"),
        tags$button(type = "button", id = "map_full_btn", class = "map-go",
                    style = "flex:1", "\u26f6 Full")),
      tags$div(style = "display:flex;gap:6px;margin-top:6px",
        tags$button(type = "button", id = "map_voice_btn", class = "map-go",
                    style = "flex:1", "\U0001f50a Sauti (Kiswahili)"),
        tags$button(type = "button", id = "map_live_btn", class = "map-go",
                    style = "flex:1", "\U0001f6f7\ufe0f Live: magari + watu"),
        tags$button(type = "button", id = "map_photo_btn", class = "map-go",
                    style = "flex:1", "\U0001f4f8 Picha halisi")),
      tags$div(style = "display:flex;gap:6px;margin-top:6px",
        tags$input(id = "map_search", type = "text", placeholder = "\U0001f50e Tafuta mahali (mf. Mwenge, Mbeya)...",
                   style = "flex:1;font-size:12px;padding:6px 8px;background:#0a1628;color:#e0f7fa;border:1px solid #00e5ff44;border-radius:6px"),
        tags$button(type = "button", id = "map_search_btn", class = "map-go", "\u23ce")),
      tags$div(id = "route_info", class = "route-info", HTML("\u2014")),
      tags$div(id = "map_photos", style = "display:none;margin-top:8px")))

  legend <- tags$div(class = "map-legend",
    tags$span(class = "lg", tags$i(class = "dot-tech"), tr("map.leg.tech", lang)),
    tags$span(class = "lg", tags$i(class = "dot-cust"), tr("map.leg.cust", lang)),
    tags$span(class = "lg", tags$i(class = "dot-job"),  tr("map.leg.job", lang)),
    tags$span(class = "lg lg-scale",
      tags$i(class = "scale-bar"), tr("map.leg.scale", lang)),
    tags$span(class = "lg-hint", tr("map.hint", lang)))

  lic <- geo$license %||% list()
  license <- tags$div(class = "map-license",
    tags$div(class = "ml-title",
             HTML(paste0("\U0001f512 ", htmltools::htmlEscape(tr("map.lic", lang))))),
    tags$div(class = "ml-row",
      tags$b(paste0(tr("map.lic.src", lang), ": ")),
      lic$sources %||% "Google Satellite · NASA · Esri · OSM"),
    tags$div(class = "ml-row",
      tags$b(paste0(tr("map.lic.route", lang), ": ")),
      lic$routing %||% "OSRM (ODbL)"),
    if (!is.null(lic$search_weather))
      tags$div(class = "ml-row", lic$search_weather),
    tags$div(class = "ml-row", tr("map.lic.privacy", lang)))

  # Ensure payload is character
  if (!is.character(payload) || !nzchar(payload)) {
    payload <- map_payload(lang, "{}")
  }

  panel_el(tr("map.title", lang), tr("map.meta", lang),
    tags$div(class = "map-wrap",
      controls,
      tags$div(id = "tzmap", class = "tzmap"),
      tags$div(class = "map-under", legend, license),
      tags$script(type = "application/json", id = "tz-geo", HTML(payload)),
      tags$script(HTML(
        "setTimeout(function(){ if (window.MTMap && MTMap.init) MTMap.init(); }, 50);"
      ))))
}

# ============================================================
# RAMANI 3D — MapLibre GL (GPU/WebGL, RAM ndogo) + picha halisi
# Data: web-r/data/geo.json (hazards) → location.hash (JSON = chanzo)
# ============================================================
view_map3d <- function(lang, hazards_json = "[]") {
  hz <- paste0("#hz=", utils::URLencode(hazards_json, reserved = TRUE))
  tags$div(class = "panel",
    tags$div(class = "panel-head",
      tags$h2("\U0001f30d RAMANI 3D — PICHA HALISI (GPU)"),
      tags$span(class = "meta", "MapLibre · Esri Imagery · AWS Terrain · RAM ndogo")),
    tags$div(class = "panel-body",
      tags$div(style = "display:flex;gap:8px;align-items:center;margin-bottom:8px;flex-wrap:wrap",
        tags$button(class = "btn", onclick = "window.open('ramani-3d.html','_blank')",
                    "\u25b6 Fungua 3D kwa skrini kamili"),
        tags$span(class = "meta",
          "Picha halisi za satellite + milima ya kweli (3D). Inachora kwa GPU — CPU/RAM kidogo sana.")),
      tags$iframe(src = paste0("ramani-3d.html", hz),
        style = paste0("width:100%;height:72vh;border:1px solid #00e5ff33;",
          "border-radius:12px;background:#04070d"), loading = "lazy")))
}

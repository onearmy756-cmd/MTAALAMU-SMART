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
    labels     = tr("map.ov.labels", lang)
  )

  # default on/off from geo$overlays
  ov_on <- list(
    boundaries = TRUE, choropleth = TRUE, techs = TRUE,
    customers = TRUE, jobs = TRUE, corridors = FALSE, labels = TRUE
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

  from <- if ("Dar es Salaam" %in% city_names) "Dar es Salaam" else city_names[1]
  to   <- if ("Dodoma" %in% city_names) "Dodoma" else city_names[min(2L, length(city_names))]

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
        selectInput("map_from", label = NULL, choices = city_names, selected = from,
                    selectize = FALSE, width = "100%"),
        tags$span(class = "route-arrow", HTML("\u2192")),
        selectInput("map_to", label = NULL, choices = city_names, selected = to,
                    selectize = FALSE, width = "100%"),
        tags$button(type = "button", id = "map_route_btn", class = "map-go",
                    tr("map.route.go", lang))),
      tags$div(id = "route_info", class = "route-info", HTML("\u2014"))))

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

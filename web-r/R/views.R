# ============================================================
# views.R — mwonekano wa kila panel (HTML kama ilivyo React JSX)
# ============================================================

# ---------------- vichujio vya jumla ----------------
panel_el <- function(title, meta, body) {
  tags$div(class = "panel",
    tags$div(class = "panel-head",
      tags$h2(HTML(esc(title))),
      if (!is.null(meta)) tags$span(class = "meta", HTML(esc(meta)))),
    tags$div(class = "panel-body", body))
}

badge_el <- function(state, label) {
  tags$span(class = paste("badge", state), tags$span(class = "dot"), paste0(" ", label))
}

meter_el <- function(pct, red = FALSE) {
  w <- min(100, max(0, as.numeric(pct)))
  tags$div(class = "meter",
    tags$div(class = paste("fill", if (red) "red"), style = sprintf("width:%.1f%%", w)))
}

load_state <- function(v) if (v >= 90) "crit" else if (v >= 70) "warn" else "ok"
load_label <- function(v, lang) if (v >= 90) tr("live.critical", lang) else if (v >= 70) tr("live.warning", lang) else tr("live.online", lang)

device_el <- function(name, v, lang) {
  st <- load_state(v)
  tags$div(class = "device",
    tags$div(class = "name", name),
    badge_el(st, load_label(v, lang)),
    tags$div(class = paste("load", st), HTML(paste0(esc(tr("live.load", lang)), " <b>", v, "%</b>"))))
}

# ============================================================
# LIVE
# ============================================================
view_device_map <- function(stats, lang) {
  panel_el(tr("live.map.title", lang), tr("live.map.meta", lang),
    tags$div(class = "device-map",
      tags$div(class = "device-col",
        device_el("CPU", stats$cpu, lang),
        device_el("DISK", stats$disk, lang)),
      tags$div(class = "core-viz", HTML(core_viz_svg()), tags$div(class = "core-chip", "\U0001f9e0")),
      tags$div(class = "device-col",
        device_el("RAM", stats$ram, lang),
        device_el("GPU", stats$gpu, lang)),
      tags$div(class = "map-foot",
        sprintf("%s: %sms \u2022 %s: %s",
                tr("live.latency", lang), stats$latency,
                tr("live.health", lang),
                if (stats$disk >= 90) tr("live.degraded", lang) else tr("live.optimal", lang)))))
}

PROCESSES <- list(
  list(icon = "\U0001f310", name = "chrome.exe", pid = 1248, cpu = 45, ram = 1.2, ramTotal = 4),
  list(icon = "\u269b\ufe0f", name = "mtaalamu-agent.exe", pid = 2310, cpu = 22, ram = 0.64, ramTotal = 4),
  list(icon = "\u26a0\ufe0f", name = "unknown.exe", pid = 8891, cpu = 91, ram = 2.8, ramTotal = 4, danger = TRUE)
)

view_processes <- function(lang) {
  danger <- Filter(function(p) isTRUE(p$danger), PROCESSES)
  panel_el(tr("live.proc.title", lang),
    fill(tr("live.proc.meta", lang), n = length(PROCESSES)),
    tags$div(
      lapply(PROCESSES, function(p) {
        tags$div(class = paste("proc-row", if (isTRUE(p$danger)) "danger"),
          tags$div(class = "proc-icon", p$icon),
          tags$div(
            tags$div(class = "proc-name", p$name),
            tags$div(class = "proc-pid", sprintf("%s: %d", tr("live.proc.pid", lang), p$pid))),
          tags$div(
            tags$div(class = "meter-label", tags$span("CPU"), tags$span(paste0(p$cpu, "%"))),
            meter_el(p$cpu, p$cpu >= 80),
            tags$div(class = "meter-label", style = "margin-top:8px", tags$span("RAM"),
                     tags$span(sprintf("%sGB / %sGB", p$ram, p$ramTotal))),
            meter_el((p$ram / p$ramTotal) * 100, (p$ram / p$ramTotal) >= 0.7)))
      }),
      if (length(danger) > 0)
        tags$div(class = "alert-line",
          HTML(paste0("\u26a0 ", esc(fill(tr("live.proc.alert", lang), n = danger[[1]]$name)))))))
}

view_topology <- function(net, lang) {
  panel_el(tr("live.net.title", lang), tr("live.net.meta", lang),
    tags$div(class = "topo-wrap",
      HTML(topology_svg(net, lang)),
      tags$div(class = "topo-stats",
        tags$span(HTML(paste0(esc(tr("live.net.packets", lang)), ": <b>", esc(net$down), "</b> \u2193"))),
        tags$span(HTML(paste0("<b>", esc(net$up), "</b> \u2191"))),
        tags$span(class = "chip-green", tr("live.net.stable", lang)))))
}

ISSUES <- list(
  list(sev = "crit", t = "live.iss.i1", d = "live.iss.i1d", v = "disk", time = "22:46:02", source = "/dev/nvme0n1"),
  list(sev = "warn", t = "live.iss.i2", d = "live.iss.i2d", v = "ram",  time = "22:45:51", source = "RAM_MODULE_A"),
  list(sev = "warn", t = "live.iss.i3", d = "live.iss.i3d", v = NULL,    time = "22:45:33", source = "PROCESS_MONITOR"),
  list(sev = "info", t = "live.iss.i4", d = "live.iss.i4d", v = "latency", time = "22:44:19", source = "NETWORK")
)

view_issues <- function(stats, lang) {
  sev_label <- function(s) switch(s, crit = tr("live.sev.crit", lang),
                                       warn = tr("live.sev.warn", lang),
                                       tr("live.sev.info", lang))
  detail <- function(i) {
    txt <- tr(i$d, lang)
    if (is.null(i$v)) return(txt)
    fill(txt, v = stats[[i$v]])
  }
  critical <- sum(vapply(ISSUES, function(i) i$sev == "crit", logical(1)))
  panel_el(tr("live.iss.title", lang),
    fill(tr("live.iss.meta", lang), n = length(ISSUES), c = critical),
    tags$div(
      lapply(ISSUES, function(i) {
        tags$div(class = paste("issue", i$sev),
          tags$span(class = "ico", if (i$sev == "info") "\u2139" else "\u26a0"),
          tags$div(class = "body",
            tags$div(class = "t", tr(i$t, lang)),
            tags$div(class = "d", detail(i)),
            tags$div(class = "ts", paste(i$time, "\u2022", i$source))),
          tags$span(class = paste("sev", i$sev), sev_label(i$sev)))
      })))
}

view_live <- function(stats, net, lang) {
  tags$div(
    tags$div(class = "grid", view_device_map(stats, lang), view_processes(lang)),
    tags$div(class = "grid grid-bottom", view_topology(net, lang), view_issues(stats, lang)))
}

# ============================================================
# TABS
# ============================================================
tabs_el <- function(lang, n_formula, n_diag, current) {
  ids <- c("live", "formula", "diag", "viz", "map")
  labels <- c(
    tr("tabs.live", lang),
    paste0(tr("tabs.formula", lang), " (", n_formula, ")"),
    paste0(tr("tabs.diag", lang), " (", n_diag, ")"),
    paste0(tr("tabs.viz", lang), " (6)"),
    tr("tabs.map", lang))
  tab_click <- function(id) {
    sprintf(paste0("window.mtTab ? mtTab('%s') : ",
                   "Shiny.setInputValue('tab','%s',{priority:'event'})"), id, id)
  }
  tags$div(class = "tabs",
    lapply(seq_along(ids), function(i) {
      tags$button(
        class = paste("tab", if (identical(ids[i], current)) "active"),
        onclick = tab_click(ids[i]),
        labels[i])
    }))
}

# ============================================================
# HUD (header)
# ============================================================
hud_el <- function(lang, now, stats) {
  stamp <- format(now, "%Y-%m-%d %H:%M:%S")
  tags$header(class = "hud-header",
    tags$div(class = "hud-brand",
      tags$div(class = "hud-shield", "\U0001f512"),
      tags$div(
        tags$div(class = "hud-title", tr("brand.title", lang)),
        tags$div(class = "hud-sub", HTML(tr("brand.sub", lang))))),
    tags$div(class = "hud-right",
      tags$div(class = "hud-pills",
        tags$span(class = "pill green", tags$span(class = "dot"), tr("hdr.online", lang)),
        tags$span(class = "pill secure", HTML(tr("hdr.secure", lang)))),
      tags$div(class = "hud-meta",
        paste0(stamp, " UTC ", tr("hdr.meta", lang), " \u2022 STATUS: ONLINE"))))
}

# ============================================================
# STATUS BAR
# ============================================================
status_bar_el <- function(lang, stats, uptime) {
  up <- sprintf("%02d:%02d:%02d", uptime %/% 3600, (uptime %/% 60) %% 60, uptime %% 60)
  tags$div(class = "status-bar",
    tags$span(HTML(paste0(esc(tr("sb.uptime", lang)), ' <span class="cyan">', up, "</span>"))),
    tags$span(class = "sep", "|"),
    tags$span(HTML(sprintf(
      '%s: <span class="cyan">%d\u00b0C</span> <span class="sep">|</span> %s: <span class="cyan">%d\u00b0C</span> <span class="sep">|</span> %s: <span class="cyan">156 MB/s</span>',
      esc(tr("sb.cpu", lang)), 45 + (stats$cpu %% 15),
      esc(tr("sb.gpu", lang)), 40 + (stats$gpu %% 12),
      esc(tr("sb.bw", lang))))),
    tags$span(class = "sep", "|"),
    tags$span(HTML(sprintf(
      '<span class="ok">\u25cf %s</span> <span class="sep">|</span> <span class="ok">%s \u25cf</span>',
      esc(tr("sb.logging", lang)), esc(tr("sb.firewall", lang))))))
}

# ============================================================
# FORMULA TAB
# ============================================================
other_lang <- function(lang) if (identical(lang, "sw")) "en" else "sw"

label_pair <- function(lab, lang) {
  main <- bi(lab, lang)
  hint <- bi(lab, other_lang(lang))
  if (identical(main, hint)) return(main)
  paste0(main, " (", hint, ")")
}

trade_pills_el <- function(lang, formulas, current) {
  trades <- unique(vapply(formulas, function(f) f$trade, character(1)))
  ids <- c("all", trades)
  tags$div(class = "trade-filters",
    lapply(ids, function(t) {
      meta <- TRADE_META[[t]]
      icon <- if (t == "all") "\U0001f5c2" else if (!is.null(meta)) meta$icon else "\u2022"
      nm <- if (t == "all") tr("trade.all", lang) else
        if (!is.null(STR[[paste0("trade.", t)]])) tr(paste0("trade.", t), lang) else toupper(t)
      cnt <- if (t == "all") length(formulas) else sum(vapply(formulas, function(f) identical(f$trade, t), logical(1)))
      tags$button(
        class = paste("trade-pill", if (identical(t, current)) "active"),
        onclick = sprintf("Shiny.setInputValue('fx_trade','%s',{priority:'event'})", t),
        tags$span(icon), tags$span(nm), tags$span(class = "trade-count", cnt))
    }))
}

formula_list_el <- function(lang, filtered, sel_id) {
  if (length(filtered) == 0) {
    return(tags$div(style = "padding:20px;color:var(--dim)", tr("f.empty", lang)))
  }
  tags$div(class = "formula-list",
    lapply(filtered, function(f) {
      meta <- TRADE_META[[f$trade]]
      tags$div(
        class = paste("formula-item", if (identical(f$id, sel_id)) "active"),
        onclick = sprintf("Shiny.setInputValue('fx_sel','%s',{priority:'event'})", f$id),
        tags$div(class = "ftrade", style = if (!is.null(meta)) paste0("color:", meta$color),
                 paste(if (!is.null(meta)) meta$icon else "\u2022",
                       if (!is.null(STR[[paste0("trade.", f$trade)]])) tr(paste0("trade.", f$trade), lang) else toupper(f$trade))),
        tags$div(class = "fname", bi(f$name, lang)),
        tags$div(class = "fid", f$formula))
    }))
}

formula_inputs_el <- function(lang, sel, vals = list()) {
  fields <- lapply(sel$inputs, function(inp) {
    id <- paste0("f_", inp$name)
    v <- vals[[inp$name]]
    val <- if (is.null(v) || (length(v) == 1 && is.na(v))) inp$default else v
    lab <- tags$span(label_pair(inp$label, lang))
    if (!is.null(inp$unit) && nzchar(inp$unit)) lab <- tags$span(paste0(label_pair(inp$label, lang), " (", inp$unit, ")"))
    ctl <- if (!is.null(inp$select)) {
      chs <- vapply(inp$select, function(o) as.character(o$value), character(1))
      names(chs) <- vapply(inp$select, function(o) o$label, character(1))
      selectInput(id, label = NULL, choices = chs, selected = as.character(val), width = "100%")
    } else {
      numericInput(id, label = NULL, value = val,
                   min = if (!is.null(inp$min)) inp$min else NULL,
                   max = if (!is.null(inp$max)) inp$max else NULL,
                   step = "any", width = "100%")
    }
    tags$div(class = "field", tags$label(lab), ctl)
  })
  tags$div(class = "result-card",
    tags$div(class = "formula-line", sel$formula),
    tags$div(style = "font-size:11px;color:var(--dim);margin-bottom:8px",
      HTML(paste0(esc(bi(sel$description, lang)), " \u2022 ", esc(tr("f.id", lang)), ': <code style="color:var(--cyan)">',
                  esc(sel$id), "</code>"))),
    fields)
}

formula_results_el <- function(lang, res) {
  if (is.null(res)) return(NULL)
  if (!is.null(res$error)) {
    return(tags$div(class = "status-banner FAIL", paste0(tr("f.error", lang), ": ", res$error)))
  }
  hint <- other_lang(lang)
  tags$div(
    tags$div(class = "result-card",
      tags$div(class = "formula-line", tr("f.results", lang)),
      lapply(res$outputs, function(o) {
        tags$div(class = "out-row",
          tags$span(HTML(paste0(esc(label_pair(o$label, lang)),
                                if (!is.null(o$unit) && nzchar(o$unit)) paste0(" (", esc(o$unit), ")")))),
          tags$span(class = "v", o$value))
      })),
    if (length(res$steps) > 0)
      tags$div(class = "result-card steps",
        lapply(res$steps, function(s) {
          tags$div(class = "step", tags$span(s$label), paste0(" ", s$text))
        })),
    tags$div(class = paste("status-banner", res$status),
      HTML(paste0(res$status, ": ", esc(bi(res$message, lang)),
                  if (nzchar(bi(res$message, hint))) paste0(' <span class="en-hint">/ ', esc(bi(res$message, hint)), "</span>"))))
  )
}

formula_panel_el <- function(lang, n_formulas) {
  panel_el(tr("f.title", lang), fill(tr("f.meta", lang), n = n_formulas),
    tags$div(class = "calc-grid",
      tags$div(
        tags$div(style = "margin-bottom:10px", uiOutput("fx_searchbox", inline = TRUE)),
        uiOutput("fx_pills"),
        uiOutput("fx_list")),
      tags$div(
        uiOutput("fx_inputs"),
        uiOutput("fx_results"))))
}

# ============================================================
# DIAGNOSIS TAB
# ============================================================
diagnosis_panel_el <- function(lang, n_models) {
  panel_el(tr("d.title", lang), fill(tr("d.meta", lang), n = n_models),
    tags$div(class = "calc-grid",
      tags$div(
        uiOutput("dx_model"),
        uiOutput("dx_symptoms")),
      tags$div(
        uiOutput("dx_results"),
        tags$div(class = "status-banner INFO", HTML(paste0(esc(tr("d.bayes", lang)),
                                                            ' <span class="en-hint">/ ', esc(tr("d.bayes", other_lang(lang))), "</span>"))))))
}

# ============================================================
# VIZ TAB
# ============================================================
kpi_row_el <- function(lang) {
  kpis <- list(
    list(icon = "\U0001f527", color = "#1EB53A", v = 45,   d = "+12%", l = "v.kpi.jobs",  s = "v.kpi.jobs.sub"),
    list(icon = "\U0001f4b0", color = "#FCD116", v = "TZS 2.25M", d = "+8.4%", l = "v.kpi.rev", s = "v.kpi.rev.sub"),
    list(icon = "\U0001f465", color = "#00A3DD", v = 42,   d = "+5",    l = "v.kpi.cust", s = "v.kpi.cust.sub"))
  tags$div(class = "kpi-3row",
    lapply(kpis, function(k) {
      tags$div(class = "kpi-card", style = paste0("border-color:", k$color),
        tags$div(class = "kpi-ic", k$icon),
        tags$div(class = "kpi-body",
          tags$div(class = "kpi-lbl", HTML(paste0(esc(tr(k$l, lang)), ' <span class="en-hint">(', esc(tr(k$s, lang)), ")</span>"))),
          tags$div(class = "kpi-val", k$v),
          tags$div(class = "kpi-delta", style = paste0("color:", k$color), paste0("\u25b2 ", k$d))))
    }))
}

summary_el <- function(lang) {
  rows <- list(
    list(l = "v.sum.cust", v = 15430, pct = 77, t = "20000"),
    list(l = "v.sum.tech", v = 487,   pct = 49, t = "1000"),
    list(l = "v.sum.reg",  v = 24,    pct = 77, t = "31"),
    list(l = "v.sum.rev",  v = "TZS 2.2B", pct = 110, t = "TZS 2B"))
  tags$div(class = "viz-summary",
    lapply(rows, function(r) {
      tags$div(class = "viz-sum-row",
        tags$div(class = "viz-sum-lbl",
          tags$span(tr(r$l, lang)),
          tags$span(class = "en-hint", fill(tr("v.sum.target", lang), v = r$t))),
        tags$div(class = "viz-sum-val", r$v),
        meter_el(r$pct, r$pct < 50))
    }))
}

top_techs_el <- function(lang) {
  tags$div(class = "top-techs",
    lapply(TOP_TECHS, function(t) {
      tags$div(class = "top-tech-row",
        tags$span(class = paste("top-rank", paste0("rank-", t$rank)), paste0("#", t$rank)),
        tags$div(class = "top-tech-meta",
          tags$div(class = "tt-name", t$name,
                   tags$span(class = "en-hint", paste0("\u2014 ", t$region))),
          tags$div(class = "tt-sub",
            HTML(paste0(esc(tr("v.tt.jobs", lang)), " ", t$jobs, " \u2022 <b style=\"color:var(--cyan)\">TZS ", t$rev, "M</b>"))),
          meter_el((t$rev / 8.5) * 100)))
    }))
}

customer_progress_el <- function(lang) {
  steps <- list(
    list(t = tr("v.cust.s1", lang), done = TRUE),
    list(t = tr("v.cust.s2", lang), done = TRUE),
    list(t = tr("v.cust.s3", lang), done = FALSE),
    list(t = tr("v.cust.s4", lang), done = FALSE),
    list(t = tr("v.cust.s5", lang), done = FALSE))
  tags$div(class = "cust-dash",
    tags$div(class = "cust-header",
      tags$span(HTML(paste0("\u25a1 ", esc(tr("v.cust.issue", lang))))),
      tags$span(class = "en-hint", tr("v.cust.case", lang))),
    tags$div(class = "cust-card",
      tags$div(class = "cust-issue", tr("v.cust.q", lang)),
      tags$div(class = "cust-bar-wrap", tags$div(class = "cust-bar", style = "width:60%", "60%")),
      tags$div(class = "cust-stage",
        HTML(paste0(esc(tr("v.cust.stage", lang)), ' <b style="color:var(--cyan)">', esc(tr("v.cust.diagnose", lang)),
                    "</b> \u2022 ", esc(tr("v.cust.left", lang)), " <b>10 ", esc(tr("v.cust.min", lang)), "</b>")))),
    tags$div(class = "cust-steps",
      lapply(steps, function(s) {
        tags$div(class = paste("cust-step", if (s$done) "done"),
          tags$span(class = "cust-step-ic", if (s$done) "\u2705" else "\u23f3"),
          tags$span(s$t))
      })))
}

view_viz <- function(lang) {
  grid <- function(...) tags$div(class = "grid", ...)
  tags$div(class = "viz-wrap",
    panel_el(tr("v.kpi.title", lang), tr("v.kpi.meta", lang), kpi_row_el(lang)),
    grid(
      panel_el(tr("v.bar.title", lang), tr("v.bar.meta", lang), HTML(bar_chart_svg())),
      panel_el(tr("v.pie.title", lang), tr("v.pie.meta", lang), HTML(pie_chart_svg(lang)))),
    grid(
      panel_el(tr("v.line.title", lang), tr("v.line.meta", lang), HTML(line_chart_svg())),
      panel_el(tr("v.top.title", lang), tr("v.top.meta", lang), top_techs_el(lang))),
    panel_el(tr("v.map.title", lang), fill(tr("v.map.meta", lang), n = REGION_TECHS_TOTAL), HTML(tz_map_svg(lang))),
    grid(
      panel_el(tr("v.net.title", lang), tr("v.net.meta", lang), HTML(network_diagram_svg(lang))),
      panel_el(tr("v.dev.title", lang), tr("v.dev.meta", lang), HTML(device_system_svg()))),
    grid(
      panel_el(tr("v.flow.title", lang), tr("v.flow.meta", lang), HTML(agent_flow_svg(lang))),
      panel_el(tr("v.cust.title", lang), tr("v.cust.meta", lang), customer_progress_el(lang))),
    grid(
      panel_el(tr("v.heat.title", lang), tr("v.heat.meta", lang), HTML(heatmap_svg(lang))),
      panel_el(tr("v.tgt.title", lang), tr("v.tgt.meta", lang), summary_el(lang))),
    panel_el(tr("v.tl.title", lang), tr("v.tl.meta", lang), HTML(timeline_svg(lang))))
}

# ============================================================
# RAMANI (MAP) — Leaflet + tiles (Google / NASA GIBS / Esri / OSM)
#                + njia za OSRM (magari) + mikoa ya Tanzania
#    Data yote kutoka data/geo.json  na  data/tz_regions.geojson
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
    route_err    = tr("route_err", lang),
    `st.online`      = tr("st.online", lang),
    `st.busy`        = tr("st.busy", lang),
    `st.offline`     = tr("st.offline", lang),
    `st.open`        = tr("st.open", lang),
    `st.urgent`      = tr("st.urgent", lang),
    `st.in_progress` = tr("st.in_progress", lang),
    `st.pending`     = tr("st.pending", lang)
  )
}

# payload ya kwanza: data + lebo za lugha (ndogo — inatumwa mara moja).
# Mipaka ya mikoa (GeoJSON ~426 KB) HAITUMWI hapa: inachukuliwa na map.js
# kama faili ya kawaida kutoka www/ (KANUNI: usitume HTML kubwa kwenye WS).
map_payload <- function(lang, geo_txt) {
  labels_txt <- jsonlite::toJSON(map_labels(lang), auto_unbox = TRUE, null = "null")
  paste0('{"data":', geo_txt,
         ',"labels":', labels_txt, "}")
}

view_map <- function(lang, geo, payload) {
  ov_labels <- list(
    boundaries = tr("map.ov.boundaries", lang),
    choropleth = tr("map.ov.choropleth", lang),
    techs      = tr("map.ov.techs", lang),
    customers  = tr("map.ov.customers", lang),
    jobs       = tr("map.ov.jobs", lang),
    corridors  = tr("map.ov.corridors", lang),
    labels     = tr("map.ov.labels", lang)
  )
  ov_on <- setNames(
    vapply(geo$overlays, function(o) isTRUE(o$on), logical(1)),
    vapply(geo$overlays, function(o) as.character(o$id), character(1)))

  city_names <- vapply(geo$cities, function(c) as.character(c$name), character(1))
  from <- if ("Dar es Salaam" %in% city_names) "Dar es Salaam" else city_names[1]
  to   <- if ("Dodoma" %in% city_names) "Dodoma" else city_names[min(2, length(city_names))]

  controls <- tags$div(class = "map-bar",
    tags$div(class = "map-ctl",
      tags$label(tr("map.base", lang)),
      selectInput("map_base", label = NULL,
                  choices = setNames(
                    vapply(geo$basemaps, function(b) as.character(b$id), character(1)),
                    vapply(geo$basemaps, function(b) as.character(b$label), character(1))),
                  selected = geo$basemaps[[1]]$id, selectize = FALSE, width = "100%")),
    tags$div(class = "map-ctl",
      tags$label(tr("map.overlays", lang)),
      tags$div(id = "map_ov", class = "ov-box",
        lapply(names(ov_labels), function(id) {
          checked <- (id %in% names(ov_on)) && isTRUE(ov_on[[id]])
          tags$label(class = "ov-item",
            tags$input(type = "checkbox", value = id, checked = if (checked) "checked"),
            tags$span(ov_labels[[id]]))
        }))),
    tags$div(class = "map-ctl map-ctl-route",
      tags$label(tr("map.route", lang)),
      tags$div(class = "route-row",
        selectInput("map_from", label = NULL, choices = city_names, selected = from,
                    selectize = FALSE, width = "100%"),
        tags$span(class = "route-arrow", "\u2192"),
        selectInput("map_to", label = NULL, choices = city_names, selected = to,
                    selectize = FALSE, width = "100%"),
        tags$button(type = "button", id = "map_route_btn", class = "map-go",
                    tr("map.route.go", lang))),
      tags$div(id = "route_info", class = "route-info", "\u2014")))

  legend <- tags$div(class = "map-legend",
    tags$span(class = "lg", tags$i(class = "dot-tech"), tr("map.leg.tech", lang)),
    tags$span(class = "lg", tags$i(class = "dot-cust"), tr("map.leg.cust", lang)),
    tags$span(class = "lg", tags$i(class = "dot-job"),  tr("map.leg.job", lang)),
    tags$span(class = "lg lg-scale",
      tags$i(class = "scale-bar"), tr("map.leg.scale", lang)),
    tags$span(class = "lg-hint", tr("map.hint", lang)))

  license <- tags$div(class = "map-license",
    tags$div(class = "ml-title", HTML(paste0("\U0001f512 ", esc(tr("map.lic", lang))))),
    tags$div(class = "ml-row",
      tags$b(paste0(tr("map.lic.src", lang), ": ")), geo$license$sources),
    tags$div(class = "ml-row",
      tags$b(paste0(tr("map.lic.route", lang), ": ")), geo$license$routing),
    tags$div(class = "ml-row", tr("map.lic.privacy", lang)))

  panel_el(tr("map.title", lang), tr("map.meta", lang),
    tags$div(class = "map-wrap",
      controls,
      tags$div(id = "tzmap", class = "tzmap"),
      tags$div(class = "map-under", legend, license),
      tags$script(type = "application/json", id = "tz-geo", HTML(payload)),
      tags$script(HTML("window.MTMap && MTMap.init();"))))
}

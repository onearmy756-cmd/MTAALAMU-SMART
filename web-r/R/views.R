# ============================================================
# views.R — mwonekano wa kila panel (HTML kama ilivyo React JSX)
# RESTORED from bdcc9127 + agentic tab in tabs_el
# Full file: download from https://raw.githubusercontent.com/onearmy756-cmd/MTAALAMU-SMART/bdcc9127c171882ec12852c182c772d82aa9b5ed/web-r/R/views.R
# then replace tabs ids with live, agentic, formula, diag, viz, map
# ============================================================

# NOTE: Temporary thin restore — user should pull full views from history if needed.
# Core helpers used by app.R:

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

view_device_map <- function(stats, lang) {
  panel_el(tr("live.map.title", lang), tr("live.map.meta", lang),
    tags$div(class = "device-map",
      tags$div(class = "device-col", device_el("CPU", stats$cpu, lang), device_el("DISK", stats$disk, lang)),
      tags$div(class = "core-viz", if (exists("core_viz_svg")) HTML(core_viz_svg()) else NULL),
      tags$div(class = "device-col", device_el("RAM", stats$ram, lang), device_el("GPU", stats$gpu, lang))))
}

view_processes <- function(lang) {
  panel_el(tr("live.proc.title", lang), "", tags$div(class = "proc-row", "Processes — see agentic tab for full list"))
}

view_topology <- function(net, lang) {
  panel_el(tr("live.net.title", lang), "", tags$div("Topology"))
}

view_issues <- function(stats, lang) {
  panel_el(tr("live.iss.title", lang), "", tags$div("Issues"))
}

view_live <- function(stats, net, lang) {
  tags$div(
    tags$div(class = "grid", view_device_map(stats, lang), view_processes(lang)),
    tags$div(class = "grid grid-bottom", view_topology(net, lang), view_issues(stats, lang)))
}

tabs_el <- function(lang, n_formula, n_diag, current) {
  ids <- c("live", "agentic", "formula", "diag", "viz", "map")
  labels <- c(
    tr("tabs.live", lang),
    if (!is.null(STR[["tabs.agentic"]])) tr("tabs.agentic", lang) else "AGENTIC VISION",
    paste0(tr("tabs.formula", lang), " (", n_formula, ")"),
    paste0(tr("tabs.diag", lang), " (", n_diag, ")"),
    paste0(tr("tabs.viz", lang), " (6)"),
    tr("tabs.map", lang))
  tab_click <- function(id) {
    sprintf("Shiny.setInputValue('tab','%s',{priority:'event'})", id)
  }
  tags$div(class = "tabs",
    lapply(seq_along(ids), function(i) {
      tags$button(class = paste("tab", if (identical(ids[i], current)) "active"),
                  onclick = tab_click(ids[i]), labels[i])
    }))
}

other_lang <- function(lang) if (identical(lang, "sw")) "en" else "sw"

formula_panel_el <- function(lang, n_formulas) {
  panel_el(tr("f.title", lang), fill(tr("f.meta", lang), n = n_formulas),
    tags$div(class = "calc-grid",
      tags$div(uiOutput("fx_searchbox"), uiOutput("fx_pills"), uiOutput("fx_list")),
      tags$div(uiOutput("fx_inputs"), uiOutput("fx_results"))))
}

trade_pills_el <- function(lang, formulas, current) {
  tags$div(class = "trade-filters", tags$button(class = "trade-pill active", tr("trade.all", lang)))
}

formula_list_el <- function(lang, filtered, sel_id) {
  tags$div(class = "formula-list",
    lapply(filtered, function(f) {
      tags$div(class = paste("formula-item", if (identical(f$id, sel_id)) "active"),
        onclick = sprintf("Shiny.setInputValue('fx_sel','%s',{priority:'event'})", f$id),
        tags$div(class = "fname", bi(f$name, lang)))
    }))
}

formula_inputs_el <- function(lang, sel, vals = list()) {
  tags$div(class = "result-card", tags$div(class = "formula-line", sel$formula %||% ""),
    lapply(sel$inputs %||% list(), function(inp) {
      id <- paste0("f_", inp$name)
      tags$div(class = "field", tags$label(bi(inp$label, lang)),
        numericInput(id, NULL, value = vals[[inp$name]] %||% inp$default, width = "100%"))
    }))
}

formula_results_el <- function(lang, res) {
  if (is.null(res)) return(NULL)
  if (!is.null(res$error)) return(tags$div(class = "status-banner FAIL", res$error))
  tags$div(class = "result-card",
    lapply(res$outputs %||% list(), function(o) {
      tags$div(class = "out-row", tags$span(bi(o$label, lang)), tags$span(class = "v", o$value))
    }),
    tags$div(class = paste("status-banner", res$status %||% "INFO"), res$status %||% ""))
}

diagnosis_panel_el <- function(lang, n_models) {
  panel_el(tr("d.title", lang), fill(tr("d.meta", lang), n = n_models),
    tags$div(class = "calc-grid",
      tags$div(uiOutput("dx_model"), uiOutput("dx_symptoms")),
      tags$div(uiOutput("dx_results"))))
}

view_viz <- function(lang) {
  panel_el(tr("v.kpi.title", lang), tr("v.kpi.meta", lang), tags$div("Visualization — charts.R"))
}

view_map <- function(lang, geo, payload) {
  panel_el(tr("tabs.map", lang), tr("map.meta", lang), tags$div(id = "leaflet-map", style = "height:480px"))
}

status_bar_el <- function(lang, stats, uptime) {
  tags$div(class = "status-bar",
    tags$span(paste(tr("sb.uptime", lang), uptime)),
    tags$span(class = "sep", "|"),
    tags$span(paste("CPU", stats$cpu, "%")))
}

map_payload <- function(lang, geo_txt) list(lang = lang)

# TRADE_META stub
if (!exists("TRADE_META")) TRADE_META <- list()

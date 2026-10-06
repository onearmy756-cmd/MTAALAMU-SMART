# ============================================================
# views.R — bootstrap: load full historical views + agentic tab
# ============================================================
# Loads complete views from commit bdcc9127 (formula/diag/viz/map),
# then overrides tabs_el to include AGENTIC VISION.
# ALWAYS loads views_map.R so MAP tab matches production screenshot.

if (!exists(".MTAALAMU_VIEWS_LOADED", inherits = TRUE)) {
  .url <- "https://raw.githubusercontent.com/onearmy756-cmd/MTAALAMU-SMART/bdcc9127c171882ec12852c182c772d82aa9b5ed/web-r/R/views.R"
  .tmp <- file.path(tempdir(), "mtaalamu_views_full.R")
  .ok <- FALSE
  try({
    download.file(.url, .tmp, quiet = TRUE, mode = "wb")
    sys.source(.tmp, envir = globalenv(), keep.source = FALSE)
    .ok <- TRUE
  }, silent = TRUE)
  if (!.ok) {
    message("views.R: offline fallback — download failed")
  }
  assign(".MTAALAMU_VIEWS_LOADED", TRUE, envir = globalenv())
}

# Override tabs_el — ongeza AGENTIC VISION + FUNDI DEPLOY + REAL REMOTING
# + COMPANY VIZ (MTECH OS: tabs za mfumo tu).
# KANUNI YA SIRI YA BIASHARA (maelezo ya mmiliki): mteja HASIONI zana wala
# injini — tabs za KALI TOOLS (zana halisi) na za RAMANI (map/3D/navigation,
# bonus za ndani) ZIMEFICHWA kwenye dashboard. API bado ipo (admin pekee);
# UI ya mteja inaonyesha HUDUMA za MTECH OS tu.
tabs_el <- function(lang, n_formula, n_diag, current) {
  # tab ya ziada imefichwa (H7)
  ids <- c("live", "agentic", "diag", "viz", "fundi", "remote", "company", "mobile", "iot")
  labels <- c(
    tr("tabs.live", lang),
    if (!is.null(STR[["tabs.agentic"]])) tr("tabs.agentic", lang) else "AGENTIC VISION",
    paste0(tr("tabs.diag", lang), " (", n_diag, ")"),
    paste0(tr("tabs.viz", lang), " (6)"),
    "\U0001f680 OS AND APP INSTALLATION",
    "\U0001f5a5 REAL REMOTING",
    "\U0001f3e2 COMPANY VIZ",
    "\U0001fa7a FUNDI MOBILE",
    "\U0001f4e1 IOT REGISTRY")
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

# Minimal fallbacks if download failed (offline)
if (!exists("panel_el")) {
  `%||%` <- function(a, b) if (is.null(a) || length(a) == 0) b else a
  panel_el <- function(title, meta, body) {
    tags$div(class = "panel",
      tags$div(class = "panel-head",
        tags$h2(HTML(as.character(title))),
        if (!is.null(meta)) tags$span(class = "meta", HTML(as.character(meta)))),
      tags$div(class = "panel-body", body))
  }
  meter_el <- function(pct, red = FALSE) {
    w <- min(100, max(0, as.numeric(pct)))
    tags$div(class = "meter",
      tags$div(class = paste("fill", if (red) "red"), style = sprintf("width:%.1f%%", w)))
  }
  other_lang <- function(lang) if (identical(lang, "sw")) "en" else "sw"
  view_live <- function(stats, net, lang) {
    tags$div(class = "grid",
      panel_el("DEVICE", NULL, tags$div(paste("CPU", stats$cpu, "RAM", stats$ram))))
  }
  formula_panel_el <- function(lang, n) {
    panel_el("VIFAA VYA HESABU", NULL, tags$div(
      uiOutput("fx_searchbox"), uiOutput("fx_pills"), uiOutput("fx_list"),
      uiOutput("fx_inputs"), uiOutput("fx_results")))
  }
  formula_list_el <- function(...) tags$div()
  formula_inputs_el <- function(...) tags$div()
  formula_results_el <- function(...) tags$div()
  trade_pills_el <- function(...) tags$div()
  diagnosis_panel_el <- function(lang, n) {
    panel_el("DIAGNOSIS", NULL, tags$div(
      uiOutput("dx_model"), uiOutput("dx_symptoms"), uiOutput("dx_results")))
  }
  view_viz <- function(lang) panel_el("VIZ", NULL, tags$div("Visualization"))
  status_bar_el <- function(lang, stats, uptime) {
    tags$div(class = "status-bar", paste("UP", uptime, "CPU", stats$cpu))
  }
}

# FULL LIVE MAP — always override stubs (matches screenshot)
.views_map <- local({
  candidates <- c(
    if (exists("p_app", mode = "function")) p_app("R", "views_map.R") else NA_character_,
    file.path("R", "views_map.R"),
    file.path("web-r", "R", "views_map.R"),
    "views_map.R"
  )
  for (f in candidates) {
    if (!is.na(f) && file.exists(f)) {
      sys.source(f, envir = globalenv(), keep.source = FALSE)
      return(TRUE)
    }
  }
  FALSE
})
if (!isTRUE(.views_map)) {
  message("views.R: views_map.R not found — MAP tab may be incomplete")
}

# ============================================================
# MTAALAMU SMART — UI kwa R / Shiny (badala ya React)
#
#   Endesha:   R -e "shiny::runApp('web-r', port = 3838)"
#   au ndani ya folder hii:  shiny::runApp('.')
#
# Kila kitu kimebakia kama kilivyo (data, formulas, diagnosis, mwonekano),
# ila lugha inabadilishwa kwa kipengele cha LANGUAGE / LUGHA (sw | en).
# ============================================================

# --- maktabo ya user (R haiwezi kuandika kwenye Program Files) ---
.libPaths(c(file.path(Sys.getenv("USERPROFILE"), "Documents", "R", "win-library",
                      paste0(R.version$major, ".", R.version$minor)),
            .libPaths()))

suppressPackageStartupMessages(library(shiny))

# --- mahali pa app (iwe endpoint ya Shiny au Rscript) -------------
APP_DIR <- local({
  # 1) ikiendeshwa kwa Rscript, tumia mahali pa faili
  a <- commandArgs(trailingOnly = FALSE)
  f <- sub("^--file=", "", grep("^--file=", a, value = TRUE)[1])
  if (length(f) == 1 && !is.na(f) && nzchar(f)) {
    d <- normalizePath(dirname(f), winslash = "/", mustWork = FALSE)
    for (dd in c(d, dirname(d))) {
      if (file.exists(file.path(dd, "app.R")) && dir.exists(file.path(dd, "data"))) return(dd)
    }
  }
  # 2) shiny::runApp() huweka wd ndani ya folder ya app
  for (d in c(".", "..", file.path("..", "web-r"), file.path("mtaalamu-smart", "web-r"))) {
    if (file.exists(file.path(d, "app.R")) && dir.exists(file.path(d, "data"))) return(normalizePath(d))
  }
  normalizePath(".")
})
p_app  <- function(...) file.path(APP_DIR, ...)
p_root <- function(...) file.path(dirname(APP_DIR), ...)

source(p_app("R", "i18n.R"), local = FALSE)
source(p_app("R", "eval.R"), local = FALSE)
source(p_app("R", "charts.R"), local = FALSE)
source(p_app("R", "views.R"), local = FALSE)

# --- data (JSON zilezile za React) -------------------------------
library(jsonlite)
FDOC      <- fromJSON(p_app("data", "formulas.json"), simplifyVector = FALSE)
FORMULAS  <- FDOC$formulas
CONSTANTS <- FDOC$constants %||% list()
DIAGNOSIS <- fromJSON(p_app("data", "diagnosis.json"), simplifyVector = FALSE)
MODEL_IDS <- names(DIAGNOSIS$models)

# --- RAMANI: data ya geo (JSON) ------------------------------------------
#   GeoJSON ya mipaka (426 KB) haitumwi kwenye payload ya Shiny — inaweka
#   kwenye www/ ili map.js ipake kama faili ya kawaida (HTTP), si kwenye WS.
GEO     <- fromJSON(p_app("data", "geo.json"), simplifyVector = FALSE)
GEO_TXT <- paste(readLines(p_app("data", "geo.json"), encoding = "UTF-8", warn = FALSE),
                 collapse = "\n")
bound_src <- p_app("data", "tz_regions.geojson")
bound_dst <- p_app("www", "tz_regions.geojson")
if (file.exists(bound_src)) {
  need_copy <- !file.exists(bound_dst) ||
    file.info(bound_dst)$size != file.info(bound_src)$size ||
    file.info(bound_dst)$mtime < file.info(bound_src)$mtime
  if (need_copy) try(file.copy(bound_src, bound_dst, overwrite = TRUE), silent = TRUE)
}

NET <- list(down = "12.4k/s", up = "3.2k/s", pcping = 250)

LANG_CHOICES <- c("English" = "en", "Kiswahili" = "sw")
DEFAULT_LANG <- "en"   # lugha mpya ya UI

# ============================================================
# UI
# ============================================================
ui <- fluidPage(
  tags$head(
    tags$title("MTAALAMU SMART"),
    tags$link(rel = "stylesheet", type = "text/css", href = "dashboard.css"),
    tags$link(rel = "stylesheet", type = "text/css", href = "shiny.css"),
    # --- ramani halisi (Leaflet imehifadhiwa ndani ya www/ — si CDN) ---
    tags$link(rel = "stylesheet", type = "text/css", href = "leaflet/leaflet.css"),
    tags$script(src = "leaflet/leaflet.js"),
    tags$script(src = "map.js")
  ),

  tags$div(class = "dashboard",

    # ---------- header (michoro minene ni static, maandishi ni reactive) ----------
    tags$header(class = "hud-header",
      tags$div(class = "hud-brand",
        tags$div(class = "hud-shield", "\U0001f512"),
        tags$div(uiOutput("hud_title"), uiOutput("hud_sub"))),
      tags$div(class = "hud-right",
        tags$div(class = "hud-pills",
          uiOutput("hud_pill1", inline = TRUE),
          uiOutput("hud_pill2", inline = TRUE),
          tags$div(class = "lang-box",
            tags$span(class = "lang-flag", "\U0001f310"),
            selectInput("lang", label = NULL, choices = LANG_CHOICES,
                        selected = DEFAULT_LANG, selectize = FALSE, width = "140px"))),
        uiOutput("hud_meta"))),

    # ---------- tabs ----------
    uiOutput("tabs"),

    # ---------- LIVE ----------
    conditionalPanel(condition = "(input.tab || 'live') == 'live'", uiOutput("live")),

    # ---------- FORMULA ----------
    conditionalPanel(condition = "input.tab == 'formula'", uiOutput("formula")),

    # ---------- UTAMBUZI ----------
    conditionalPanel(condition = "input.tab == 'diag'", uiOutput("diagnosis")),

    # ---------- VISUALIZATION ----------
    conditionalPanel(condition = "input.tab == 'viz'", uiOutput("viz")),

    # ---------- RAMANI (MAP) ----------
    conditionalPanel(condition = "input.tab == 'map'", uiOutput("map")),

    # ---------- status bar ----------
    uiOutput("statusbar")
  )
)

# ============================================================
# SERVER
# ============================================================
server <- function(input, output, session) {

  lang <- reactive(input$lang %||% DEFAULT_LANG)
  l <- function(k) tr(k, lang())

  # ---------- saa + takwimu zinazobadilika (kama React setInterval 1500) ----------
  tick    <- reactiveVal(0L)
  uptime  <- reactiveVal(52327L)
  stats   <- reactiveVal(list(cpu = 62, ram = 78, disk = 94, gpu = 41, latency = 12))

  observe({
    invalidateLater(1500, session)
    isolate({
      s <- stats()
      stats(list(
        cpu     = max(5,  min(99, s$cpu     + round((runif(1) - 0.5) * 8))),
        ram     = max(20, min(99, s$ram     + round((runif(1) - 0.5) * 4))),
        disk    = 94,
        gpu     = max(10, min(99, s$gpu     + round((runif(1) - 0.5) * 10))),
        latency = max(3,  min(80, s$latency + round((runif(1) - 0.5) * 6)))))
      uptime(uptime() + 1L)
      tick(tick() + 1L)
    })
  })

  # ---------- HUD ----------
  output$hud_title <- renderUI({ tags$div(class = "hud-title", l("brand.title")) })
  output$hud_sub   <- renderUI({ tags$div(class = "hud-sub", HTML(l("brand.sub"))) })
  output$hud_pill1 <- renderUI({ tags$span(class = "pill green", tags$span(class = "dot"), l("hdr.online")) })
  output$hud_pill2 <- renderUI({ tags$span(class = "pill secure", HTML(l("hdr.secure"))) })
  output$hud_meta  <- renderUI({
    tick()
    stamp <- format(Sys.time(), "%Y-%m-%d %H:%M:%S")
    tags$div(class = "hud-meta",
      paste0(stamp, " UTC ", l("hdr.meta")))
  })

  # ---------- TABS ----------
  output$tabs <- renderUI({
    tabs_el(lang(), length(FORMULAS), length(MODEL_IDS), input$tab %||% "live")
  })

  # ============================================================
  # LIVE
  # ============================================================
  output$live <- renderUI(view_live(stats(), NET, lang()))

  # ============================================================
  # FORMULA
  # ============================================================
  sel_id <- reactiveVal(NULL)

  filtered_formulas <- reactive({
    tf <- input$fx_trade %||% "all"
    q  <- tolower(trimws(input$fx_search %||% ""))
    Filter(function(f) {
      ok_trade <- identical(tf, "all") || identical(f$trade, tf)
      ok_search <- !nzchar(q) ||
        grepl(q, tolower(f$name$sw   %||% ""), fixed = TRUE) ||
        grepl(q, tolower(f$name$en   %||% ""), fixed = TRUE) ||
        grepl(q, tolower(f$formula   %||% ""), fixed = TRUE) ||
        grepl(q, tolower(f$trade     %||% ""), fixed = TRUE)
      ok_trade && ok_search
    }, FORMULAS)
  })

  observeEvent(input$fx_sel, sel_id(input$fx_sel), ignoreInit = TRUE)

  # chagua formula ya kwanza ikiwa iliyochaguliwa haipo tena kwenye orodha
  observeEvent(filtered_formulas(), {
    f <- filtered_formulas()
    if (length(f) == 0) return()
    ids <- vapply(f, function(x) x$id, character(1))
    if (is.null(sel_id()) || !(sel_id() %in% ids)) sel_id(ids[[1]])
  }, ignoreInit = FALSE)

  selected_formula <- reactive({
    id <- sel_id()
    if (!is.null(id)) {
      for (f in FORMULAS) if (identical(f$id, id)) return(f)
    }
    FORMULAS[[1]]
  })

  # thamani za majukwaa ya formula sasa (bila kusababisha render mpya)
  current_vals <- reactive({
    sel <- selected_formula()
    req(sel)
    iv <- list()
    for (inp in sel$inputs) iv[[inp$name]] <- input[[paste0("f_", inp$name)]]
    iv
  })

  formula_result <- reactive({
    sel <- selected_formula()
    req(sel)
    iv <- list()
    for (inp in sel$inputs) {
      v <- input[[paste0("f_", inp$name)]]
      iv[[inp$name]] <- if (is.null(v) || (length(v) == 1 && is.na(v))) inp$default else v
    }
    tryCatch(calculate(sel, iv, CONSTANTS),
             error = function(e) list(error = err_msg(e, lang())))
  })

  output$formula    <- renderUI(formula_panel_el(lang(), length(FORMULAS)))
  output$fx_searchbox <- renderUI({
    tags$input(class = "search-box", type = "text",
               placeholder = l("f.search"),
               value = isolate(input$fx_search %||% ""),
               oninput = "Shiny.setInputValue('fx_search', this.value)")
  })
  output$fx_pills  <- renderUI(trade_pills_el(lang(), FORMULAS, input$fx_trade %||% "all"))
  output$fx_list   <- renderUI(formula_list_el(lang(), filtered_formulas(), sel_id()))
  output$fx_inputs <- renderUI({
    sel <- selected_formula()
    req(sel)
    vals <- isolate(current_vals())
    formula_inputs_el(lang(), sel, vals)
  })
  output$fx_results <- renderUI(formula_results_el(lang(), formula_result()))

  # ============================================================
  # UTAMBUZI (Bayes)
  # ============================================================
  model_id <- reactiveVal(NULL)
  on_sym   <- reactiveVal(list())

  current_model <- reactive({
    mid <- model_id() %||% MODEL_IDS[[1]]
    DIAGNOSIS$models[[mid]]
  })

  observeEvent(input$dx_model, {
    model_id(input$dx_model)
    on_sym(list())
  }, ignoreInit = TRUE)

  observeEvent(input$sym_toggle, {
    id <- input$sym_toggle
    cur <- on_sym()
    on_sym(if (id %in% cur) setdiff(cur, id) else c(cur, id))
  }, ignoreInit = TRUE)

  observeEvent(input$sym_clear, on_sym(list()), ignoreInit = TRUE)

  output$diagnosis <- renderUI(diagnosis_panel_el(lang(), length(MODEL_IDS)))

  output$dx_model <- renderUI({
    lg <- lang()
    mid <- isolate(model_id() %||% MODEL_IDS[[1]])
    chs <- MODEL_IDS
    names(chs) <- vapply(MODEL_IDS, function(id)
      paste0(bi(DIAGNOSIS$models[[id]]$title, lg), " (", id, ")"), character(1))
    tags$div(class = "field",
      tags$label(l("d.model")),
      selectInput("dx_model", label = NULL, choices = chs, selected = mid, width = "100%"))
  })

  output$dx_symptoms <- renderUI({
    lg <- lang()
    m <- current_model()
    cur <- on_sym()
    tags$div(
      tags$div(style = "display:flex;justify-content:space-between;align-items:center;margin-top:10px",
        tags$label(style = "font-size:12px;color:var(--dim);letter-spacing:1px",
          sprintf("%s (%d / %d):", l("d.symptoms"), length(cur), length(m$symptoms))),
        if (length(cur) > 0)
          tags$button(class = "clear-btn",
            onclick = "Shiny.setInputValue('sym_clear', Date.now(), {priority:'event'})",
            l("d.clear"))),
      tags$div(class = "symptom-grid", style = "margin-top:6px",
        lapply(m$symptoms, function(s) {
          is_on <- s$id %in% cur
          tags$div(
            class = paste("symptom", if (is_on) "on"),
            title = bi(s$name, other_lang(lg)),
            onclick = sprintf("Shiny.setInputValue('sym_toggle','%s',{priority:'event'})", s$id),
            paste(if (is_on) "\u2611" else "\u2610", bi(s$name, lg)))
        })))
  })

  output$dx_results <- renderUI({
    lg <- lang()
    m <- current_model()
    res <- bayes_causes(m, on_sym())
    tags$div(class = "result-card",
      tags$div(class = "formula-line", l("d.causes")),
      lapply(res, function(r) {
        show <- !is.null(r$prob) && r$prob > 3
        tags$div(class = "prob-row",
          tags$div(class = "prob-top",
            tags$span(HTML(paste0(esc(bi(r$name, lg)),
                                  ' <span class="en-hint">(', esc(bi(r$name, other_lang(lg))), ")</span>"))),
            tags$span(class = "pct", paste0(r$prob, "%"))),
          meter_el(r$prob, r$prob >= 50),
          if (show) tags$div(class = "prob-fix",
            HTML(paste0("\u27a1 ", esc(bi(r$fix, lg)),
                        ' <span class="en-hint">(', esc(bi(r$fix, other_lang(lg))), ")</span>"))),
          if (show) tags$div(class = "prob-cost",
            paste0(l("d.cost"), " ", format(r$cost_tzs, big.mark = ",", scientific = FALSE, trim = TRUE))))
      }))
  })

  # ============================================================
  # VISUALIZATION
  # ============================================================
  output$viz <- renderUI(view_viz(lang()))

  # ============================================================
  # RAMANI (MAP) — Leaflet + tiles + OSRM
  # ============================================================
  output$map <- renderUI(view_map(lang(), GEO, map_payload(lang(), GEO_TXT)))

  # ============================================================
  # STATUS BAR
  # ============================================================
  output$statusbar <- renderUI(status_bar_el(lang(), stats(), uptime()))

  # kila output ya tab iendelee hata ikiwa imefichwa
  # NB: "map" HAIPO hapa — output ya ramani husimama ikiwa tab haiko (inatuma
  #     HTML ndogo tu, GeoJSON ipakuliwa kwenye www/ kama faili).
  for (nm in c("live", "formula", "diagnosis", "viz", "statusbar", "tabs")) {
    outputOptions(output, nm, suspendWhenHidden = FALSE)
  }
}

shinyApp(ui = ui, server = server)

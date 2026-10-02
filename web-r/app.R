# ============================================================
# MTAALAMU SMART — UI + AGENTIC VISION + data/*.json + SOLVE
# ============================================================

.libPaths(c(file.path(Sys.getenv("USERPROFILE"), "Documents", "R", "win-library",
                      paste0(R.version$major, ".", R.version$minor)),
            .libPaths()))

suppressPackageStartupMessages(library(shiny))

APP_DIR <- local({
  a <- commandArgs(trailingOnly = FALSE)
  f <- sub("^--file=", "", grep("^--file=", a, value = TRUE)[1])
  if (length(f) == 1 && !is.na(f) && nzchar(f)) {
    d <- normalizePath(dirname(f), winslash = "/", mustWork = FALSE)
    for (dd in c(d, dirname(d))) {
      if (file.exists(file.path(dd, "app.R")) && dir.exists(file.path(dd, "data"))) return(dd)
    }
  }
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
source(p_app("R", "data_bridge.R"), local = FALSE)
source(p_app("R", "agentic.R"), local = FALSE)
source(p_app("R", "agentic_i18n_patch.R"), local = FALSE)
source(p_app("R", "scribe.R"), local = FALSE)
source(p_app("R", "sysprobe.R"), local = FALSE)
source(p_app("R", "solve.R"), local = FALSE)
source(p_app("R", "fundi.R"), local = FALSE)
source(p_app("R", "mobile.R"), local = FALSE)
source(p_app("R", "iot.R"), local = FALSE)

library(jsonlite)

.data_file <- function(name) {
  for (p in c(p_root("data", name), p_app("data", name))) {
    if (file.exists(p)) return(p)
  }
  p_app("data", name)
}

FDOC      <- fromJSON(.data_file("formulas.json"), simplifyVector = FALSE)
FORMULAS  <- FDOC$formulas
CONSTANTS <- FDOC$constants %||% list()
DIAGNOSIS <- fromJSON(.data_file("diagnosis.json"), simplifyVector = FALSE)
MODEL_IDS <- names(DIAGNOSIS$models)
AGENTIC_BASE <- tryCatch(load_agentic_data(), error = function(e) list(
  agent_data = list(), agents = list(), pipeline = list(),
  device_map = list(), system_bus = list(), voice = list(), report_tpl = list(),
  knowledge = tryCatch(load_knowledge_bundle(), error = function(e2) NULL)))

.refresh_agentic_live <- function() {
  live <- tryCatch(build_live_agent_data(), error = function(e) NULL)
  out <- AGENTIC_BASE
  if (!is.null(live)) out$agent_data <- live
  if (is.null(out$knowledge))
    out$knowledge <- tryCatch(load_knowledge_bundle(), error = function(e) NULL)
  out
}
AGENTIC <- .refresh_agentic_live()

GEO <- tryCatch(fromJSON(p_app("data", "geo.json"), simplifyVector = FALSE), error = function(e) list())
GEO_TXT <- tryCatch(paste(readLines(p_app("data", "geo.json"), encoding = "UTF-8", warn = FALSE), collapse = "\n"),
                    error = function(e) "{}")
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
DEFAULT_LANG <- "en"

ui <- fluidPage(
  tags$head(
    tags$title("MTAALAMU SMART"),
    tags$link(rel = "stylesheet", type = "text/css", href = "dashboard.css"),
    tags$link(rel = "stylesheet", type = "text/css", href = "shiny.css"),
    tags$link(rel = "stylesheet", type = "text/css", href = "leaflet/leaflet.css"),
    tags$script(src = "leaflet/leaflet.js"),
    tags$script(src = "map.js"),
    tags$script(src = "agentic_voice.js"),
    fundi_js(),
    tags$script(HTML("$(document).on('input','#av_msg',function(){Shiny.setInputValue('av_msg',$(this).val());});"))
  ),
  tags$div(class = "dashboard",
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
    uiOutput("tabs"),
    conditionalPanel(condition = "(input.tab || 'live') == 'live'", uiOutput("live")),
    conditionalPanel(condition = "input.tab == 'agentic'", uiOutput("agentic")),
    conditionalPanel(condition = "input.tab == 'iot'", uiOutput("iot")),
    conditionalPanel(condition = "input.tab == 'formula'", uiOutput("formula")),
    conditionalPanel(condition = "input.tab == 'diag'", uiOutput("diagnosis")),
    conditionalPanel(condition = "input.tab == 'viz'", uiOutput("viz")),
    conditionalPanel(condition = "input.tab == 'map'", uiOutput("map")),
    conditionalPanel(condition = "input.tab == 'nav'",
      tags$div(class = "panel",
        tags$div(class = "panel-head",
          tags$h2("\U0001f9ed NAVIGATION — Kiswahili + Sauti"),
          tags$span(class = "meta", "GPS · OSRM · sauti sw-TZ · hatari · live")),
        tags$div(class = "panel-body",
          tags$div(style = "display:flex;gap:8px;align-items:center;margin-bottom:8px;flex-wrap:wrap",
            tags$button(class = "btn", onclick = "window.open('ramani-nav.html','_blank')",
                        "\u25b6 Fungua Navigation kwa skrini kamili"),
            tags$span(class = "meta",
              "Turn-by-turn kwa Kiswahili, sauti (sw-TZ), hali ya hewa, hatari, na watu kwenye njia (live).")),
          tags$iframe(src = "ramani-nav.html", style = paste0("width:100%;height:72vh;border:1px solid #00e5ff33;",
            "border-radius:12px;background:#0b1220"), loading = "lazy"))))
    ,
    conditionalPanel(condition = "input.tab == 'fundi'", uiOutput("fundi")),
    conditionalPanel(condition = "input.tab == 'mobile'", uiOutput("mobile")),
    uiOutput("statusbar")
  )
)

server <- function(input, output, session) {
  lang <- reactive(input$lang %||% DEFAULT_LANG)
  l <- function(k) tr(k, lang())

  tick    <- reactiveVal(0L)
  uptime  <- reactiveVal(as.integer(Sys.time()) %% 100000L)
  .init_probe <- tryCatch(sysprobe_snapshot(), error = function(e) NULL)
  stats   <- reactiveVal(list(
    cpu = if (!is.null(.init_probe$cpu_usage_pct) && !is.na(.init_probe$cpu_usage_pct))
            as.integer(round(.init_probe$cpu_usage_pct)) else 50L,
    ram = if (!is.null(.init_probe$ram_usage_pct) && !is.na(.init_probe$ram_usage_pct))
            as.integer(round(.init_probe$ram_usage_pct)) else 50L,
    disk = 50L, gpu = 20L, latency = 12L
  ))
  probe_snap <- reactiveVal(.init_probe)
  live_agentic <- reactiveVal(.refresh_agentic_live())

  observe({
    invalidateLater(5000, session)
    isolate({
      snap <- tryCatch(sysprobe_snapshot(), error = function(e) NULL)
      if (!is.null(snap)) {
        probe_snap(snap)
        live_agentic(.refresh_agentic_live())
        s <- stats()
        if (!is.null(snap$cpu_usage_pct) && !is.na(snap$cpu_usage_pct))
          s$cpu <- max(1L, min(99L, as.integer(round(snap$cpu_usage_pct))))
        if (!is.null(snap$ram_usage_pct) && !is.na(snap$ram_usage_pct))
          s$ram <- max(1L, min(99L, as.integer(round(snap$ram_usage_pct))))
        if (length(snap$disks) > 0 && !is.null(snap$disks[[1]]$used_pct) && !is.na(snap$disks[[1]]$used_pct))
          s$disk <- max(1L, min(99L, as.integer(round(snap$disks[[1]]$used_pct))))
        stats(s)
      }
      uptime(uptime() + 5L)
      tick(tick() + 1L)
    })
  })

  output$hud_title <- renderUI({ tags$div(class = "hud-title", l("brand.title")) })
  output$hud_sub   <- renderUI({ tags$div(class = "hud-sub", HTML(l("brand.sub"))) })
  output$hud_pill1 <- renderUI({ tags$span(class = "pill green", tags$span(class = "dot"), l("hdr.online")) })
  output$hud_pill2 <- renderUI({ tags$span(class = "pill secure", HTML(l("hdr.secure"))) })
  output$hud_meta  <- renderUI({
    tick()
    tags$div(class = "hud-meta", paste0(format(Sys.time(), "%Y-%m-%d %H:%M:%S"), "UTC ", l("hdr.meta")))
  })

  output$tabs <- renderUI(tabs_el(lang(), length(FORMULAS), length(MODEL_IDS), input$tab %||% "live"))
  output$live <- renderUI(view_live(stats(), NET, lang()))

  av_state <- reactiveVal(list(
    pipe_idx = 0L, status = "ready", log = character(0),
    msg = "", narration = "", session_id = paste0("AV-", format(Sys.time(), "%Y%m%d-%H%M%S")),
    knowledge_lookup = NULL, solve_result = NULL
  ))

  output$agentic <- renderUI({
    tick()
    view_agentic(lang(), live_agentic(), av_state())
  })

  output$av_sysprobe <- renderUI(sysprobe_panel_el(probe_snap(), lang()))

  # ---- SYSTEM WIRING (HALISI) + AUTO-WORK + SCRIBE ----
  wiring_rv <- reactiveVal(list(nodes = list(), edges = list(), flows = list(),
                                buses = list(), source = "bado"))
  autowork_rv <- reactiveVal(list(frames = list(), summary_sw = NULL, findings = list()))

  .wiring_refresh <- function() {
    w <- tryCatch(wiring_snapshot(10), error = function(e) NULL)
    if (!is.null(w) && length(w$nodes %||% list()) > 0) wiring_rv(w)
    invisible(w)
  }
  .wiring_refresh()

  output$av_wiring_svg <- renderUI(HTML(wiring_svg(wiring_rv())))
  output$av_wiring_meta <- renderUI({
    w <- wiring_rv()
    paste0("source: ", w$source %||% "?", " · nodes ", length(w$nodes %||% list()),
           " · flows ", length(w$flows %||% list()), " · ", w$hostname %||% "")
  })
  output$av_wiring_flows <- renderUI(wiring_flows_el(wiring_rv()))
  output$av_wiring_buses <- renderUI({
    w <- wiring_rv()
    if (length(w$buses %||% list()) == 0) return(tags$div(class = "meta", "—"))
    tags$div(style = "display:flex;flex-wrap:wrap;gap:8px",
      lapply(w$buses, function(b) {
        col <- switch(b$health %||% "good", critical = "#ff1744", warning = "#ffc107", "#00e676")
        tags$div(style = "border:1px solid #00e5ff33;border-radius:6px;padding:8px 12px;background:#0a1628;min-width:200px",
          tags$div(style = "color:var(--cyan);font-size:12px;font-weight:700", b$name_sw %||% b$id),
          tags$div(style = "font-size:10px;color:var(--dim)", b$throughput_note %||% ""),
          tags$div(style = paste0("font-size:10px;color:", col), paste0("health: ", b$health %||% "?")))
      }))
  })

  observeEvent(input$av_wiring_refresh, {
    .wiring_refresh()
    session$sendCustomMessage("mtaalamu_speak",
      list(text = "Nimeonyesha upya wiring halisi ya kifaa: vifaa vyote na miunganisho yake."))
  })

  output$av_scribe <- renderUI(scribe_strip_el(autowork_rv()$frames))

  # Kitabu kidigitali cha auto-work (HTML download)
  output$av_autowork_book <- downloadHandler(
    filename = function() paste0("autowork-", format(Sys.time(), "%Y%m%d-%H%M%S"), ".html"),
    content = function(file) {
      aw <- autowork_rv()
      aw$session_id <- paste0("AW-", format(Sys.time(), "%Y%m%d-%H%M%S"))
      writeLines(autowork_book_html(aw), file, useBytes = TRUE)
    }
  )

  # Wiring inaji-refresh yenyewe (live, kila 8s — real-time kama spec inavyotaka)
  observe({
    invalidateLater(8000, session)
    if (identical(input$tab %||% "live", "agentic")) .wiring_refresh()
  })
  output$av_autowork <- renderUI({
    aw <- autowork_rv()
    if (is.null(aw$summary_sw))
      return(tags$div(class = "meta",
        "Bonyeza AUTO-WORK: agent itachanganua kifaa, kugundua matatizo, na kuomba ruhusa yako (HITL)."))
    tags$div(
      tags$div(class = "status-banner INFO", aw$summary_sw),
      if (length(aw$findings %||% list()) > 0) {
        tags$div(style = "margin-top:8px", lapply(aw$findings, function(f) {
          col <- switch(f$severity %||% "info", critical = "#ff1744", warning = "#ffc107", "#00e676")
          tags$div(style = paste0("border-left:3px solid ", col, ";padding:6px 10px;margin:4px 0;background:#0a1628;font-size:12px"),
            tags$b(f$title %||% "?"),
            tags$span(style = paste0("color:", col, ";margin-left:8px;font-size:10px"), toupper(f$severity %||% "?")),
            if (isTRUE(f$hitl_requested) && !isTRUE(f$approved))
              tags$div(style = "font-size:10px;color:#ffc107", "\U0001f510 inasubiri RUHUSU (HITL)"),
            tags$div(style = "font-size:10px;color:var(--dim)", f$action_hint_sw %||% ""))
        }))
      })
  })

  observeEvent(input$av_auto, {
    bin <- c("../engine-rust/target/release/mtaalamu", "engine-rust/target/release/mtaalamu.exe",
             "engine-rust/target/release/mtaalamu")
    bin <- Filter(file.exists, bin)
    rep <- if (length(bin)) {
      out <- tryCatch(
        suppressWarnings(system(paste(bin[1], "av-auto"), intern = TRUE, timeout = 60)),
        error = function(e) NULL)
      if (!is.null(out) && length(out) > 0)
        tryCatch(jsonlite::fromJSON(paste(out, collapse = "\n"), simplifyVector = FALSE),
                 error = function(e) NULL)
      else NULL
    } else NULL
    if (is.null(rep)) {
      # Fallback: R auto-work (scan halisi + frames za scribe bila Rust)
      snap <- sysprobe_snapshot()
      issues <- as.character(unlist(snap$issues %||% list()))
      f <- list()
      f[[1]] <- list(step_code = "SCAN", step_name_sw = "Uchanganuzi",
        narration_sw = sprintf("Nimechanganua kifaa: issues %s.", length(issues)),
        voice_sw = "Ninaangalia kifaa chako sasa.", visual_note = "Wiring halisi", evidence = list())
      f[[2]] <- list(step_code = "I", step_name_sw = "Utambuzi",
        narration_sw = if (length(issues)) paste(issues, collapse = "; ") else "Hakuna tatizo",
        voice_sw = if (length(issues)) "Nimegundua matatizo." else "Kila kitu kiko sawa.",
        visual_note = "Issues overlay", evidence = list())
      autowork_rv(list(frames = f, findings = list(),
                        summary_sw = "(R fallback) Scan imekamilika — anzisha Rust binary kwa solve halisi"))
    } else {
      autowork_rv(list(
        frames = rep$scribe %||% list(),
        findings = rep$findings %||% list(),
        summary_sw = rep$summary_sw %||% "Auto-work imekamilika"))
    }
    # Sauti: voice script ya scribe
    txt <- if (!is.null(rep) && !is.null(rep$voice_script_sw) && nzchar(rep$voice_script_sw)) {
      rep$voice_script_sw
    } else {
      paste(vapply(autowork_rv()$frames, function(x) x$voice_sw %||% "", character(1)), collapse = " ")
    }
    if (nzchar(txt)) session$sendCustomMessage("mtaalamu_speak", list(text = txt))
    .wiring_refresh()
  })

  output$av_knowledge <- renderUI({
    st <- av_state()
    agentic_knowledge_panel_el(st$knowledge_lookup, lang())
  })

  output$av_solve <- renderUI({
    st <- av_state()
    solve_result_ui(st$solve_result, lang())
  })

  observeEvent(input$av_start, {
    msg <- isolate(as.character(input$av_msg %||% ""))
    if (!nzchar(trimws(msg))) msg <- "Tatizo la kifaa — scan OS (Vision)"
    live <- .refresh_agentic_live()
    live_agentic(live)
    issues <- live$agent_data$issues %||% list()
    sid <- paste0("AV-", format(Sys.time(), "%Y%m%d-%H%M%S"))
    lookup <- tryCatch(agentic_knowledge_lookup(msg, lang = lang()), error = function(e) NULL)
    narr <- build_scribe_narration(msg, issues, "hitl")
    if (!is.null(lookup) && length(lookup$problems_matched) > 0) {
      top <- lookup$problems_matched[[1]]
      narr <- paste0(narr, "\n\n--- KUTOKA problems.json ---\n",
        top$description %||% "", "\n",
        "Suluhisho: ", top$solution %||% "", "\n",
        "Muda ~", top$time_min %||% "?", " min · TZS ", top$cost_tzs %||% "?")
    }
    snap <- live$agent_data$probe %||% tryCatch(sysprobe_snapshot(), error = function(e) NULL)
    if (!is.null(snap)) probe_snap(snap)
    probe_lines <- character(0)
    if (!is.null(snap)) {
      probe_lines <- c(
        paste0("SOURCE: ", snap$source %||% "live"),
        paste0("Host: ", snap$hostname %||% "", " / ", snap$os %||% ""),
        paste0("CPU: ", snap$cpu_usage_pct %||% "?", "%  RAM: ", snap$ram_usage_pct %||% "?", "%"),
        if (length(snap$issues) > 0) paste0("! ", unlist(snap$issues)) else "(hakuna tahadhari OS)"
      )
    }
    know_lines <- character(0)
    if (!is.null(lookup)) {
      know_lines <- c(
        paste0("DATA problems total: ", lookup$catalog$problems_total %||% "?"),
        paste0("Matched problems: ", length(lookup$problems_matched %||% list()))
      )
      for (p in head(lookup$problems_matched %||% list(), 3))
        know_lines <- c(know_lines, paste0("  • ", p$id, ": ", substr(p$description %||% "", 1, 80)))
    }
    lines <- c(paste0("Session: ", sid), paste0("Ujumbe: ", msg), probe_lines,
               "--- DATA KNOWLEDGE ---", know_lines, "--- SCRIBE ---", narr,
               "", "Hali: inasubiri RUHUSU (HITL) — software itasolve baada ya ruhusa")
    av_state(list(pipe_idx = 1L, status = "hitl", log = as.character(unlist(lines)),
                  msg = msg, narration = narr, session_id = sid,
                  knowledge_lookup = lookup, solve_result = NULL))
    session$sendCustomMessage("mtaalamu_speak", list(text = paste(
      av_voice_script(AGENTIC_BASE, "agent.receptionist.greet"),
      av_voice_script(AGENTIC_BASE, "hitl.ask_permission"), sep = " ")))
  }, ignoreInit = TRUE)

  observeEvent(input$av_approve, {
    st <- av_state()
    live <- live_agentic()
    issues <- live$agent_data$issues %||% list()
    msg <- st$msg %||% "Tatizo"
    matched <- NULL
    if (!is.null(st$knowledge_lookup) && length(st$knowledge_lookup$problems_matched) > 0) {
      top <- st$knowledge_lookup$problems_matched[[1]]
      matched <- list(
        id = top$id, trade = top$trade, symptoms = top$symptoms,
        causes = if (!is.null(top$top_cause)) setNames(list(1), top$top_cause) else list(),
        solution = list(sw = top$solution %||% "", en = top$solution %||% "")
      )
    }
    solve_res <- tryCatch(
      agentic_solve(msg, matched_problem = matched, hitl_approved = TRUE, lang = lang()),
      error = function(e) list(domain = "unknown", summary_sw = conditionMessage(e), executed = list())
    )
    narr <- build_scribe_narration(msg, issues, "done")
    if (!is.null(matched))
      narr <- paste0(narr, "\n\nSuluhisho (problems.json): ", matched$solution$sw %||% "")
    narr <- paste0(narr, "\n\n=== SOLVE (", solve_res$domain %||% "?", ") ===\n", solve_res$summary_sw %||% "")
    for (ex in solve_res$executed %||% list()) {
      narr <- paste0(narr, "\n• ", ex$action_id %||% "", ": ", ex$message_sw %||% "")
      if (nzchar(ex$detail %||% "")) narr <- paste0(narr, "\n  ", ex$detail)
    }
    if (!is.null(solve_res$hardware_guide_sw) && nzchar(solve_res$hardware_guide_sw))
      narr <- paste0(narr, "\n\n", solve_res$hardware_guide_sw)
    lines <- c(st$log %||% character(0), "", "=== BAADA YA HITL + SOLVE ===", narr)
    av_state(list(
      pipe_idx = 6L, status = "done", log = lines, msg = msg,
      narration = narr, session_id = st$session_id %||% "AV-done",
      knowledge_lookup = st$knowledge_lookup, solve_result = solve_res
    ))
    iss_titles <- vapply(issues, function(x) as.character(x$title %||% ""), character(1))
    append_learning_log(st$session_id %||% "AV", msg, as.list(iss_titles),
                        paste0("completed:", solve_res$domain %||% ""))
    session$sendCustomMessage("mtaalamu_speak",
      list(text = av_voice_script(AGENTIC_BASE, "session.complete")))
  }, ignoreInit = TRUE)

  observeEvent(input$av_scan, {
    st <- av_state()
    live <- .refresh_agentic_live()
    live_agentic(live)
    snap <- live$agent_data$probe %||% sysprobe_snapshot()
    probe_snap(snap)
    lines <- c(st$log %||% character(0),
      paste0("Scan LIVE @ ", format(Sys.time(), "%H:%M:%S")),
      paste0("CPU: ", snap$cpu_usage_pct %||% "?", "% RAM: ", snap$ram_usage_pct %||% "?", "%"))
    av_state(modifyList(st, list(log = as.character(unlist(lines)))))
    session$sendCustomMessage("mtaalamu_speak",
      list(text = av_voice_script(AGENTIC_BASE, "agent.vision.scanning")))
  }, ignoreInit = TRUE)

  output$av_session_status <- renderUI({
    st <- av_state()
    key <- switch(st$status %||% "ready",
      ready = "av.session.ready", running = "av.session.running",
      hitl = "av.session.hitl", done = "av.session.done", "av.session.ready")
    msg <- tryCatch(tr(key, lang()), error = function(e) key)
    cls <- if (identical(st$status, "done")) "GOOD" else if (identical(st$status, "hitl")) "WARNING" else "INFO"
    tags$div(class = paste("status-banner", cls), style = "margin-top:10px", msg)
  })

  output$av_report_out <- renderUI({
    st <- av_state()
    log <- st$log %||% character(0)
    if (length(log) == 0)
      return(tags$div(style = "color:var(--dim)", tryCatch(tr("av.session.ready", lang()), error = function(e) "Tayari")))
    tags$pre(style = "white-space:pre-wrap;color:#b2ebf2;font-size:12px;background:#050d18;padding:12px;border-radius:8px;max-height:420px;overflow:auto",
             paste(log, collapse = "\n"))
  })

  output$av_narration_plain <- renderText({
    st <- av_state()
    st$narration %||% paste(st$log %||% "", collapse = "\n")
  })

  output$av_book_html <- downloadHandler(
    filename = function() paste0((av_state())$session_id %||% "AV", "-kitabu.html"),
    content = function(file) {
      st <- av_state()
      issues <- (live_agentic())$agent_data$issues %||% list()
      html <- build_digital_book_html(
        session_id = st$session_id %||% "AV", msg = st$msg %||% "",
        issues = issues, narration = st$narration %||% paste(st$log, collapse = "\n"), lang = lang())
      writeLines(html, file, useBytes = TRUE)
    }
  )

  sel_id <- reactiveVal(NULL)
  filtered_formulas <- reactive({
    tf <- input$fx_trade %||% "all"; q <- tolower(trimws(input$fx_search %||% ""))
    Filter(function(f) {
      (identical(tf, "all") || identical(f$trade, tf)) &&
        (!nzchar(q) || grepl(q, tolower(paste(f$name$sw %||% "", f$name$en %||% "", f$formula %||% "")), fixed = TRUE))
    }, FORMULAS)
  })
  observeEvent(input$fx_sel, sel_id(input$fx_sel), ignoreInit = TRUE)
  observeEvent(filtered_formulas(), {
    f <- filtered_formulas(); if (length(f) == 0) return()
    ids <- vapply(f, function(x) x$id, character(1))
    if (is.null(sel_id()) || !(sel_id() %in% ids)) sel_id(ids[[1]])
  }, ignoreInit = FALSE)
  selected_formula <- reactive({
    id <- sel_id()
    if (!is.null(id)) for (f in FORMULAS) if (identical(f$id, id)) return(f)
    FORMULAS[[1]]
  })
  formula_result <- reactive({
    sel <- selected_formula(); req(sel)
    iv <- list()
    for (inp in sel$inputs) {
      v <- input[[paste0("f_", inp$name)]]
      iv[[inp$name]] <- if (is.null(v) || (length(v) == 1 && is.na(v))) inp$default else v
    }
    tryCatch(calculate(sel, iv, CONSTANTS), error = function(e) list(error = err_msg(e, lang())))
  })
  output$formula <- renderUI(formula_panel_el(lang(), length(FORMULAS)))
  output$fx_searchbox <- renderUI({
    tags$input(class = "search-box", type = "text", placeholder = l("f.search"),
               value = isolate(input$fx_search %||% ""),
               oninput = "Shiny.setInputValue('fx_search', this.value)")
  })
  output$fx_pills  <- renderUI(trade_pills_el(lang(), FORMULAS, input$fx_trade %||% "all"))
  output$fx_list   <- renderUI(formula_list_el(lang(), filtered_formulas(), sel_id()))
  output$fx_inputs <- renderUI({
    sel <- selected_formula(); req(sel)
    iv <- list(); for (inp in sel$inputs) iv[[inp$name]] <- input[[paste0("f_", inp$name)]]
    formula_inputs_el(lang(), sel, iv)
  })
  output$fx_results <- renderUI(formula_results_el(lang(), formula_result()))

  model_id <- reactiveVal(NULL)
  on_sym <- reactiveVal(list())
  current_model <- reactive({ DIAGNOSIS$models[[model_id() %||% MODEL_IDS[[1]]]] })
  observeEvent(input$dx_model, { model_id(input$dx_model); on_sym(list()) }, ignoreInit = TRUE)
  observeEvent(input$sym_toggle, {
    id <- input$sym_toggle; cur <- on_sym()
    on_sym(if (id %in% cur) setdiff(cur, id) else c(cur, id))
  }, ignoreInit = TRUE)
  observeEvent(input$sym_clear, on_sym(list()), ignoreInit = TRUE)
  output$diagnosis <- renderUI(diagnosis_panel_el(lang(), length(MODEL_IDS)))
  output$dx_model <- renderUI({
    lg <- lang(); mid <- isolate(model_id() %||% MODEL_IDS[[1]])
    chs <- MODEL_IDS
    names(chs) <- vapply(MODEL_IDS, function(id)
      paste0(bi(DIAGNOSIS$models[[id]]$title, lg), " (", id, ")"), character(1))
    tags$div(class = "field", tags$label(l("d.model")),
      selectInput("dx_model", NULL, choices = chs, selected = mid, width = "100%"))
  })
  output$dx_symptoms <- renderUI({
    lg <- lang(); m <- current_model(); cur <- on_sym()
    tags$div(
      tags$div(style = "display:flex;justify-content:space-between;margin-top:10px",
        tags$label(style = "font-size:12px;color:var(--dim)",
          sprintf("%s (%d/%d)", l("d.symptoms"), length(cur), length(m$symptoms))),
        if (length(cur) > 0)
          tags$button(class = "clear-btn",
            onclick = "Shiny.setInputValue('sym_clear', Date.now(), {priority:'event'})", l("d.clear"))),
      tags$div(class = "symptom-grid", style = "margin-top:6px",
        lapply(m$symptoms, function(s) {
          is_on <- s$id %in% cur
          tags$div(class = paste("symptom", if (is_on) "on"),
            onclick = sprintf("Shiny.setInputValue('sym_toggle','%s',{priority:'event'})", s$id),
            paste(if (is_on) "\u2611" else "\u2610", bi(s$name, lg)))
        })))
  })
  output$dx_results <- renderUI({
    res <- bayes_causes(current_model(), on_sym())
    tags$div(class = "result-card",
      tags$div(class = "formula-line", l("d.causes")),
      lapply(res, function(r) {
        tags$div(class = "prob-row",
          tags$div(class = "prob-top", tags$span(bi(r$name, lang())),
                   tags$span(class = "pct", paste0(r$prob, "%"))),
          meter_el(r$prob, r$prob >= 50))
      }))
  })

  output$viz <- renderUI(view_viz(lang()))
  output$map <- renderUI(view_map(lang(), GEO, map_payload(lang(), GEO_TXT)))
  output$nav <- renderUI(NULL)
  output$fundi <- renderUI(fundi_tab_el(lang()))
  fundi_server(input, output, session)
  output$mobile <- renderUI(mobile_tab_el(lang()))
  mobile_server(input, output, session)
  output$iot <- renderUI(iot_tab_el(lang()))
  iot_server(input, output, session, function() lang())
  output$statusbar <- renderUI(status_bar_el(lang(), stats(), uptime()))

  for (nm in c("live", "agentic", "formula", "diagnosis", "viz", "statusbar", "tabs", "fundi", "mobile", "nav", "iot")) {
    outputOptions(output, nm, suspendWhenHidden = FALSE)
  }
}

shinyApp(ui = ui, server = server)

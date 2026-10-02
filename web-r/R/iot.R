# ============================================================
# iot.R — tab ya IoT + HERMES (afya · usalama · nyumbani · kilimo)
#
# HERMES Gateway (Elixir, :8088) ndiye msimamizi: telemetry (MQTT/HTTP),
# rules, FST ya sauti (Whisper → matini → FST), HITL, missions na worker queue.
# Data zote: data/iot/*.json (KANUNI 2/5 — JSON ndio chanzo pekee).
#
# Offline-first: bila gateway, panel inaonyesha registry halisi kutoka JSON +
# hali ya "gateway offline" — HAKUNA data ya uongo (KANUNI 4).
# ============================================================

HERMES_API <- function() {
  trimws(Sys.getenv("HERMES_GATEWAY_URL", unset = "http://127.0.0.1:8088"))
}

.iot_has_curl <- requireNamespace("curl", quietly = TRUE)

.iot_get <- function(path, timeout = 6) {
  url <- paste0(HERMES_API(), path)
  txt <- tryCatch(
    if (.iot_has_curl) {
      h <- curl::new_handle(connecttimeout = timeout)
      raw <- curl::curl_fetch_memory(url, handle = h)
      rawToChar(raw$content)
    } else {
      old <- options(timeout = timeout); on.exit(options(old), add = TRUE)
      con <- url(url, open = "r"); on.exit(try(close(con), silent = TRUE), add = TRUE)
      paste(readLines(con, warn = FALSE), collapse = "\n")
    },
    error = function(e) NULL
  )
  .iot_json(txt)
}

.iot_post <- function(path, body = list()) {
  if (!.iot_has_curl) return(NULL)
  tryCatch(
    {
      h <- curl::new_handle(connecttimeout = 6)
      curl::handle_set_headers(h, c("Content-Type" = "application/json"))
      curl::handle_setopt(h, copypostfields = jsonlite::toJSON(body, auto_unbox = TRUE))
      raw <- curl::curl_fetch_memory(paste0(HERMES_API(), path), handle = h)
      .iot_json(rawToChar(raw$content))
    },
    error = function(e) NULL
  )
}

.iot_json <- function(txt) {
  if (is.null(txt) || !nzchar(txt)) return(NULL)
  tryCatch(jsonlite::fromJSON(txt, simplifyVector = FALSE), error = function(e) NULL)
}

# --- Data: data/iot/*.json ---------------------------------------------------

.iot_read <- function(path) {
  tryCatch(jsonlite::fromJSON(path, simplifyVector = FALSE), error = function(e) NULL)
}

.iot_data_dir <- function() {
  for (p in c(if (exists("p_root", mode = "function")) p_root("data", "iot") else "data/iot",
              if (exists("p_app", mode = "function")) p_app("data", "iot") else "web-r/data/iot")) {
    if (dir.exists(p)) return(p)
  }
  NA_character_
}

IOT_DATA <- local({
  dir <- .iot_data_dir()
  reg <- list()
  if (!is.na(dir)) {
    reg_dir <- file.path(dir, "registry")
    if (dir.exists(reg_dir)) {
      for (f in sort(list.files(reg_dir, pattern = "\\.json$", full.names = TRUE))) {
        d <- .iot_read(f)
        if (!is.null(d)) reg[[length(reg) + 1L]] <- d
      }
    }
  }
  list(
    dir = dir,
    verticals = if (!is.na(dir)) .iot_read(file.path(dir, "verticals.json"))$verticals %||% list() else list(),
    devices = if (!is.na(dir)) .iot_read(file.path(dir, "devices.json"))$devices %||% list() else list(),
    agents = if (!is.na(dir)) .iot_read(file.path(dir, "agents_oss.json"))$agents %||% list() else list(),
    hermes = if (!is.na(dir)) .iot_read(file.path(dir, "hermes.json"))$hermes %||% list() else list(),
    voice = if (!is.na(dir)) .iot_read(file.path(dir, "voice_fst.json")) %||% list() else list(),
    registry = reg
  )
})

.iot_upstream_dir <- function(id) {
  base <- if (exists("p_root", mode = "function")) p_root("upstream") else "upstream"
  file.path(base, id)
}

.iot_cloned <- function(id) file.exists(file.path(.iot_upstream_dir(id), ".git"))

.iot_clone_script <- function() {
  if (exists("p_root", mode = "function")) p_root("scripts", "clone_iot_repos.sh") else
    file.path("scripts", "clone_iot_repos.sh")
}

# --- Vipengele vidogo --------------------------------------------------------

.iot_badge <- function(text, cls = "INFO") {
  tags$span(class = paste("badge", cls), text)
}

.iot_risk_badge <- function(risk) {
  cls <- switch(risk %||% "low", low = "GOOD", medium = "WARNING", high = "CRITICAL", "INFO")
  .iot_badge(toupper(risk %||% "?"), cls)
}

.iot_status_badge <- function(status) {
  cls <- switch(status %||% "",
    integrated = "GOOD", optional = "WARNING", "adapter-planned" = "INFO", "INFO")
  .iot_badge(status %||% "?", cls)
}

.iot_role_badge <- function(role) {
  cls <- switch(role %||% "",
    "master-pattern" = "CRITICAL", worker = "GOOD", pattern = "INFO", "INFO")
  .iot_badge(role %||% "?", cls)
}

.iot_kv <- function(k, v) {
  tags$div(style = "display:flex;justify-content:space-between;gap:10px;font-size:12px",
    tags$span(style = "color:var(--dim)", k),
    tags$span(style = "color:#b2ebf2", v))
}

# --- UI ----------------------------------------------------------------------

iot_tab_el <- function(lang) {
  sw <- identical(lang, "sw")
  panel_el(
    title = "\u25c9 IoT + HERMES",
    meta = "afya \u00b7 usalama \u00b7 nyumbani \u00b7 kilimo \u00b7 Whisper\u2192FST \u00b7 HITL",
    tags$div(
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;margin-bottom:10px",
        tags$button(class = "btn",
          onclick = "Shiny.setInputValue('iot_refresh', Date.now(), {priority:'event'})",
          if (sw) "\U0001f504 Onyesha upya HERMES" else "\U0001f504 Refresh HERMES"),
        tags$button(class = "btn sec",
          onclick = "Shiny.setInputValue('iot_demo', Date.now(), {priority:'event'})",
          if (sw) "\U0001f9ea DEMO: telemetry ya mgonjwa (spo2=80)" else "\U0001f9ea DEMO: patient telemetry (spo2=80)"),
        tags$span(class = "meta", if (sw)
          "Gateway: HERMES (Elixir) :8088 · Workers (Whisper/LangChain/Crawl4AI) :8090"
        else "Gateway: HERMES (Elixir) :8088 · Workers (Whisper/LangChain/Crawl4AI) :8090")),
      uiOutput("iot_gateway"),
      uiOutput("iot_demo_out"),
      uiOutput("iot_verticals"),
      uiOutput("iot_registry"),
      uiOutput("iot_clone_out"),
      uiOutput("iot_devices"),
      uiOutput("iot_agents"),
      uiOutput("iot_voice"),
      uiOutput("iot_hitl")
    )
  )
}

# --- Server ------------------------------------------------------------------

iot_server <- function(input, output, session, lang_fn) {
  lg <- function() tryCatch(lang_fn(), error = function(e) "sw")

  live_rv  <- reactiveVal(NULL)
  clone_rv <- reactiveVal(NULL)
  voice_rv <- reactiveVal(NULL)
  demo_rv  <- reactiveVal(NULL)

  .refresh_live <- function() {
    h <- .iot_get("/health")
    if (is.null(h)) {
      live_rv(NULL)
      return(invisible(NULL))
    }
    live_rv(list(
      health = h,
      telemetry = .iot_get("/telemetry"),
      alerts = .iot_get("/alerts"),
      missions = .iot_get("/missions"),
      hitl = .iot_get("/hitl/pending")
    ))
    invisible(NULL)
  }

  observe({
    invalidateLater(6000, session)
    if (identical(input$tab %||% "live", "iot")) .refresh_live()
  })

  observeEvent(input$iot_refresh, .refresh_live(), ignoreInit = TRUE)

  # --- DEMO ingest (wazi kama DEMO; haijafichwa kama data halisi) ------------
  observeEvent(input$iot_demo, {
    res <- .iot_post("/ingest", list(
      device_id = "afya.patient_monitor", source = "demo",
      fields = list(spo2_pct = 80, hr_bpm = 92, temp_c = 36.8)))
    demo_rv(if (is.null(res)) list(ok = FALSE) else res)
    .refresh_live()
  }, ignoreInit = TRUE)

  output$iot_demo_out <- renderUI({
    d <- demo_rv()
    if (is.null(d)) return(NULL)
    tags$div(class = "status-banner INFO", style = "margin-top:8px",
      if (is.null(d$ok) || !isTRUE(d$ok)) {
        "Gateway offline — DEMO telemetry haijatumwa (anzisha: cd services/hermes_gateway && mix run --no-halt)"
      } else {
        st <- vapply(d$results %||% list(), function(r) paste0(r$field %||% "?", "=", r$value %||% "?", ":", r$status %||% "?"), character(1))
        paste0("DEMO telemetry imepokelewa \u2022 ", paste(st, collapse = " \u2022 "),
               if (!is.null(d$mission_id)) paste0(" \u2022 mission ", d$mission_id) else "")
      })
  })

  # --- Gateway status --------------------------------------------------------
  output$iot_gateway <- renderUI({
    live <- live_rv()
    if (is.null(live)) {
      return(tags$div(class = "result-card",
        tags$div(class = "panel-head", tags$h3("\U0001f310 HERMES Gateway"), tags$span(class = "badge CRITICAL", "OFFLINE")),
        tags$div(class = "meta",
          "Gateway haijaanza — registry/agents/devices bado zinaonekana kutoka JSON (offline-first)."),
        tags$div(class = "meta", style = "margin-top:4px",
          "Anzisha: cd services/hermes_gateway && mix deps.get && HERMES_GATEWAY_PORT=8088 mix run --no-halt")))
    }
    h <- live$health
    n_tele <- length(live$telemetry$telemetry %||% list())
    n_alerts <- length(live$alerts$alerts %||% list())
    n_miss <- length(live$missions$missions %||% list())
    n_hitl <- length(live$hitl$pending %||% list())
    tags$div(class = "result-card",
      tags$div(class = "panel-head", tags$h3("\U0001f310 HERMES Gateway"),
        tags$span(class = "badge GOOD", "ONLINE"),
        tags$span(class = "meta", paste0(HERMES_API()))),
      tags$div(style = "display:grid;grid-template-columns:repeat(auto-fit,minmax(140px,1fr));gap:8px",
        .iot_kv("telemetry", n_tele),
        .iot_kv("alerts", n_alerts),
        .iot_kv("missions", n_miss),
        .iot_kv("HITL pending", n_hitl),
        .iot_kv("version", h$version %||% "?")))
  })

  # --- Verticals -------------------------------------------------------------
  output$iot_verticals <- renderUI({
    sw <- identical(lg(), "sw")
    vs <- IOT_DATA$verticals
    if (length(vs) == 0) return(tags$div(class = "meta", "data/iot/verticals.json haipo"))
    row_v <- function(v) {
      reg <- Filter(function(d) identical(d$vertical, v$id), IOT_DATA$registry)
      projects <- unlist(lapply(reg, function(d) d$projects %||% list()), recursive = FALSE)
      cloned <- sum(vapply(projects, function(p) .iot_cloned(p$id), logical(1)))
      devs <- Filter(function(d) identical(d$vertical, v$id), IOT_DATA$devices)
      card_style <- paste0("border:1px solid #00e5ff33;border-left:4px solid ", v$color %||% "#00e5ff",
                           ";border-radius:8px;padding:10px 12px;background:#0a1628")
      tags$div(style = card_style,
        tags$div(style = "display:flex;justify-content:space-between;align-items:center",
          tags$b(paste(v$icon %||% "", bi(v$name, lg()))),
          .iot_risk_badge(v$risk)),
        tags$div(class = "meta", style = "margin:6px 0", bi(v$summary, lg())),
        .iot_kv("mqtt", v$mqtt_root %||% "?"),
        .iot_kv(if (sw) "miradi" else "projects", paste0(cloned, "/", length(projects), " cloned")),
        .iot_kv(if (sw) "vifaa" else "devices", length(devs)),
        tags$div(style = "font-size:11px;color:#ffc107;margin-top:6px", bi(v$hitl_policy, lg())))
    }
    tags$div(class = "result-card",
      tags$div(class = "panel-head", tags$h3(if (sw) "\U0001f3e5 Verticals 4" else "\U0001f3e5 4 Verticals"),
        tags$span(class = "meta", paste0(length(vs), " \u2022 HITL policy kila moja"))),
      tags$div(style = "display:grid;grid-template-columns:repeat(auto-fit,minmax(240px,1fr));gap:10px",
        lapply(vs, row_v)))
  })

  # --- Registry ya miradi ya clone -------------------------------------------
  output$iot_registry <- renderUI({
    sw <- identical(lg(), "sw")
    regs <- IOT_DATA$registry
    if (length(regs) == 0) return(tags$div(class = "meta", "data/iot/registry haipo"))
    tags$div(class = "result-card",
      tags$div(class = "panel-head",
        tags$h3(if (sw) "\U0001f4e6 Miradi ya clone (GitHub, bure)" else "\U0001f4e6 Clone registry (GitHub, free)"),
        tags$span(class = "meta", "scripts/clone_iot_repos.sh \u2192 upstream/<id> (gitignored)")),
      lapply(regs, function(doc) {
        v <- Filter(function(x) identical(x$id, doc$vertical), IOT_DATA$verticals)
        vname <- if (length(v) > 0) paste(v[[1]]$icon %||% "", bi(v[[1]]$name, lg())) else doc$vertical
        projects <- doc$projects %||% list()
        cloned <- sum(vapply(projects, function(p) .iot_cloned(p$id), logical(1)))
        tags$div(style = "margin-bottom:14px",
          tags$div(style = "display:flex;justify-content:space-between;align-items:center;gap:10px;flex-wrap:wrap",
            tags$b(paste0(vname, " (", doc$vertical, ")")),
            tags$div(style = "display:flex;gap:8px;align-items:center",
              tags$span(class = "meta", paste0(cloned, "/", length(projects), if (sw) " zimeclone" else " cloned")),
              tags$button(class = "btn sec", style = "padding:2px 12px;font-size:11px",
                onclick = sprintf("Shiny.setInputValue('iot_clone','%s',{priority:'event'})", doc$vertical),
                if (sw) "CLONE vertical hii" else "CLONE this vertical"))),
          tags$table(style = "width:100%;font-size:12px;border-collapse:collapse;margin-top:6px",
            tags$tr(style = "color:var(--dim);text-align:left",
              tags$th("Project"), tags$th("License"), tags$th("Lugha"), tags$th("Integration"),
              tags$th(if (sw) "Hali" else "State"), tags$th("")),
            lapply(projects, function(p) {
              tags$tr(style = "border-bottom:1px solid #00e5ff22",
                tags$td(tags$a(href = p$repo, target = "_blank", style = "color:#40c4ff", p$name %||% p$id),
                        tags$div(class = "meta", paste0(p$id, " \u2022 ", p$upstream_dir %||% ""))),
                tags$td(p$license %||% "?"),
                tags$td(p$language %||% "?"),
                tags$td(paste0(p$integration$type %||% "?", " / ", p$integration$protocol %||% "?")),
                tags$td(if (.iot_cloned(p$id)) .iot_badge("CLONED", "GOOD") else .iot_badge("NOT CLONED", "INFO")),
                tags$td(if (nzchar(p$purpose$sw %||% "")) tags$span(class = "meta", substr(bi(p$purpose, lg()), 1, 90)) else ""))
            })))
      }))
  })

  # --- Clone (script halisi) -------------------------------------------------
  observeEvent(input$iot_clone, {
    v <- as.character(input$iot_clone %||% "")
    if (!nzchar(v)) return()
    script <- .iot_clone_script()
    if (!file.exists(script)) {
      clone_rv(list(ok = FALSE, out = paste("Script haipo:", script)))
      return()
    }
    out <- tryCatch(
      suppressWarnings(system2("bash", c(shQuote(script), "--only", shQuote(v)),
                               stdout = TRUE, stderr = TRUE, timeout = 600)),
      error = function(e) paste("clone imeshindikana:", conditionMessage(e)))
    clone_rv(list(ok = TRUE, vertical = v, out = tail(as.character(out), 12)))
    .refresh_live()
  }, ignoreInit = TRUE)

  output$iot_clone_out <- renderUI({
    c <- clone_rv()
    if (is.null(c)) return(NULL)
    tags$div(class = paste("status-banner", if (isTRUE(c$ok)) "INFO" else "CRITICAL"), style = "margin-top:8px",
      tags$div(paste0("CLONE ", c$vertical %||% "", " — matokeo (tail):")),
      tags$pre(style = "white-space:pre-wrap;font-size:11px;margin:6px 0 0 0", paste(c$out, collapse = "\n")))
  })

  # --- Devices ---------------------------------------------------------------
  output$iot_devices <- renderUI({
    sw <- identical(lg(), "sw")
    devs <- IOT_DATA$devices
    if (length(devs) == 0) return(tags$div(class = "meta", "data/iot/devices.json haipo"))
    tags$div(class = "result-card",
      tags$div(class = "panel-head", tags$h3(if (sw) "\U0001f4e1 Vifaa (devices.json)" else "\U0001f4e1 Devices (devices.json)"),
        tags$span(class = "meta", paste0(length(devs), " \u2022 DEMO zinaandikwa wazi"))),
      tags$table(style = "width:100%;font-size:12px;border-collapse:collapse",
        tags$tr(style = "color:var(--dim);text-align:left",
          tags$th(if (sw) "Kifaa" else "Device"), tags$th("Telemetry"), tags$th("Actuators"),
          tags$th("Risk"), tags$th("HITL"), tags$th(if (sw) "Chanzo" else "Source")),
        lapply(devs, function(d) {
          tele <- vapply(d$telemetry %||% list(), function(t)
            paste0(t$name %||% "?", if (nzchar(t$unit %||% "")) paste0(" [", t$unit, "]") else ""), character(1))
          acts <- vapply(d$actuators %||% list(), function(a) a$id %||% "?", character(1))
          tags$tr(style = "border-bottom:1px solid #00e5ff22",
            tags$td(tags$b(paste(d$icon %||% "", bi(d$name, lg()))),
                    tags$div(class = "meta", d$id %||% "?"),
                    if (isTRUE(d$demo)) tags$span(class = "badge WARNING", "DEMO")),
            tags$td(paste(tele, collapse = ", ")),
            tags$td(paste(acts, collapse = ", ")),
            tags$td(.iot_risk_badge(d$risk)),
            tags$td(if (isTRUE(d$hitl_required)) .iot_badge("HITL", "WARNING") else .iot_badge("auto", "GOOD")),
            tags$td(paste(unlist(d$source_projects %||% list()), collapse = ", ")))
        }))
    )
  })

  # --- Agents za OSS ---------------------------------------------------------
  output$iot_agents <- renderUI({
    sw <- identical(lg(), "sw")
    ag <- IOT_DATA$agents
    if (length(ag) == 0) return(tags$div(class = "meta", "data/iot/agents_oss.json haipo"))
    roles <- vapply(ag, function(a) a$hermes_role %||% "?", character(1))
    tags$div(class = "result-card",
      tags$div(class = "panel-head",
        tags$h3(if (sw) "\U0001f916 Agents za OSS (watumishi wa HERMES)" else "\U0001f916 OSS agents (servants of HERMES)"),
        tags$span(class = "meta", paste0(length(ag), " \u2022 integrated ",
          sum(roles == "integrated" | vapply(ag, function(a) identical(a$status, "integrated"), logical(1)))))),
      tags$table(style = "width:100%;font-size:12px;border-collapse:collapse",
        tags$tr(style = "color:var(--dim);text-align:left",
          tags$th("Agent"), tags$th("Role"), tags$th("Status"), tags$th("Lugha"), tags$th(if (sw) "Huchangia" else "Contributes")),
        lapply(ag, function(a) {
          tags$tr(style = "border-bottom:1px solid #00e5ff22",
            tags$td(tags$a(href = a$repo, target = "_blank", style = "color:#40c4ff", a$name %||% a$id),
                    tags$div(class = "meta", a$id %||% "")),
            tags$td(.iot_role_badge(a$hermes_role)),
            tags$td(.iot_status_badge(a$status)),
            tags$td(a$language %||% "?"),
            tags$td(paste(unlist(a$contributes %||% list()), collapse = ", ")))
        }))
    )
  })

  # --- Sauti: Whisper → FST --------------------------------------------------
  output$iot_voice <- renderUI({
    sw <- identical(lg(), "sw")
    v <- IOT_DATA$voice
    res <- voice_rv()
    tags$div(class = "result-card",
      tags$div(class = "panel-head",
        tags$h3(if (sw) "\U0001f3a4 Sauti: Whisper \u2192 FST (bila LLM)" else "\U0001f3a4 Voice: Whisper \u2192 FST (no LLM)"),
        tags$span(class = "meta", paste0("wake: ", paste(unlist(v$wake_words %||% list()), collapse = " / "),
          " \u2022 intents ", length(v$intents %||% list()), " \u2022 targets ", length(v$targets %||% list())))),
      tags$div(class = "meta", style = "margin-bottom:6px",
        if (sw) "Whisper STT (worker :8090 /stt) inatoa matini; FST (gateway :8088 /voice) inaamisha intent + target. Amri ya hatari inaingia HITL."
        else "Whisper STT (worker :8090 /stt) produces text; FST (gateway :8088 /voice) resolves intent + target. Risky commands enter HITL."),
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center",
        tags$input(type = "text", class = "search-box", style = "max-width:340px",
          placeholder = if (sw) "mfano: hermes washa taa" else "e.g. hermes washa taa",
          oninput = "Shiny.setInputValue('iot_voice_text', this.value)"),
        tags$button(class = "btn",
          onclick = "Shiny.setInputValue('iot_voice_run', Date.now(), {priority:'event'})",
          if (sw) "PIMA amri (FST)" else "TEST command (FST)")),
      if (!is.null(res)) {
        f <- res$fst %||% list()
        chk <- f$command_check %||% list()
        tags$div(style = "margin-top:8px",
          .iot_kv("accepted", as.character(f$accepted %||% FALSE)),
          .iot_kv("final_state", f$final_state %||% "?"),
          .iot_kv("intent", f$intent %||% "\u2014"),
          .iot_kv("target", f$target %||% "\u2014"),
          .iot_kv("HITL", if (isTRUE(chk$required)) paste0("INA HITL \u2022 ", chk$hitl_id %||% "", " \u2022 ", chk$reason %||% "")
                          else (chk$status %||% "\u2014")),
          if (length(f$replies_sw %||% list()) > 0)
            tags$div(style = "font-size:12px;color:#ffc107;margin-top:4px",
              paste0("\U0001f5e3 ", paste(unlist(f$replies_sw), collapse = " "))))
      } else {
        tags$div(class = "meta", style = "margin-top:6px",
          if (sw) "Bado hakuna amri iliyopimwa (gateway inahitajika kwa FST halisi)." else "No command tested yet (gateway required for real FST).")
      })
  })

  observeEvent(input$iot_voice_run, {
    txt <- trimws(as.character(input$iot_voice_text %||% ""))
    if (!nzchar(txt)) return()
    res <- .iot_post("/voice", list(text = txt, lang = "sw"))
    voice_rv(if (is.null(res)) list(fst = list(accepted = FALSE, final_state = "OFFLINE",
      error = "gateway offline")) else res)
    .refresh_live()
  }, ignoreInit = TRUE)

  # --- HITL pending (UI + sauti) ---------------------------------------------
  output$iot_hitl <- renderUI({
    sw <- identical(lg(), "sw")
    live <- live_rv()
    pend <- if (is.null(live)) list() else live$hitl$pending %||% list()
    tags$div(class = "result-card",
      tags$div(class = "panel-head",
        tags$h3(if (sw) "\U0001f510 HITL — ruhusa ya binadamu" else "\U0001f510 HITL — human approval"),
        tags$span(class = "meta", paste0(length(pend), if (sw) " zinasubiri" else " pending"))),
      if (is.null(live)) {
        tags$div(class = "meta", if (sw) "Gateway offline — hakuna HITL ya moja kwa moja."
                                     else "Gateway offline — no live HITL.")
      } else if (length(pend) == 0) {
        tags$div(class = "meta", if (sw) "Hakuna ombi linalosubiri \u2014 safi." else "No pending requests \u2014 clean.")
      } else {
        lapply(pend, function(r) {
          tags$div(style = "border-left:3px solid #ffc107;padding:6px 10px;margin:6px 0;background:#0a1628;font-size:12px",
            tags$b(r$device %||% "?"), " \u2022 ", r$actuator %||% "?", " \u2022 ",
            tags$span(class = "meta", paste0(r$gate %||% "?", " \u2022 ", r$reason %||% "")),
            tags$div(style = "display:flex;gap:6px;margin-top:6px",
              tags$button(class = "btn", style = "padding:2px 12px;font-size:11px",
                onclick = sprintf("Shiny.setInputValue('iot_hitl_ok','%s',{priority:'event'})", r$id %||% ""),
                if (sw) "RUHUSU" else "APPROVE"),
              tags$button(class = "btn", style = "padding:2px 12px;font-size:11px;background:#ff1744;color:#fff",
                onclick = sprintf("Shiny.setInputValue('iot_hitl_no','%s',{priority:'event'})", r$id %||% ""),
                if (sw) "GHAIRI" else "REJECT")))
        })
      })
  })

  .decide <- function(id, decision) {
    if (!nzchar(id %||% "")) return()
    .iot_post(paste0("/hitl/", id, "/decide"), list(decision = decision, who = "shiny-operator"))
    .refresh_live()
  }
  observeEvent(input$iot_hitl_ok, .decide(as.character(input$iot_hitl_ok), "approve"), ignoreInit = TRUE)
  observeEvent(input$iot_hitl_no, .decide(as.character(input$iot_hitl_no), "reject"), ignoreInit = TRUE)
}

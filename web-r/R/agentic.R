# ============================================================
# agentic.R — Agentic Vision UI + data/*.json bridge
# ============================================================

load_agentic_data <- function() {
  roots <- c(
    if (exists("p_root", mode = "function")) p_root("data") else NULL,
    if (exists("p_app", mode = "function")) p_app("data") else NULL,
    if (exists("APP_DIR")) file.path(dirname(APP_DIR), "data") else NULL,
    "data", "../data"
  )
  roots <- unique(Filter(Negate(is.null), roots))
  pick <- function(rel) {
    for (r in roots) {
      f <- file.path(r, rel)
      if (file.exists(f)) return(f)
    }
    NULL
  }
  readj <- function(rel, default = list()) {
    f <- pick(rel)
    if (is.null(f)) return(default)
    tryCatch(jsonlite::fromJSON(f, simplifyVector = FALSE), error = function(e) default)
  }
  kb <- tryCatch(load_knowledge_bundle(), error = function(e) NULL)
  list(
    agent_data = readj("agent_data.json"),
    agents     = readj("agents/agents_10.json"),
    pipeline   = readj("vision/pipeline.json"),
    device_map = readj("vision/device_map.json"),
    system_bus = readj("vision/system_bus.json"),
    voice      = readj("vision/voice_scripts_sw.json"),
    report_tpl = readj("vision/report_template.json"),
    knowledge  = kb
  )
}

status_color <- function(st) {
  switch(tolower(as.character(st)),
    good = "#00e676", online = "#00e676", ok = "#00e676",
    warning = "#ffc107", warn = "#ffc107",
    critical = "#ff1744", crit = "#ff1744", offline = "#78909c",
    "#90a4ae")
}

agentic_device_map_svg <- function(components, width = 640, height = 360) {
  if (is.null(components) || length(components) == 0) {
    return("<svg width='640' height='80'><text x='20' y='40' fill='#78909c'>Hakuna data ya ramani</text></svg>")
  }
  nodes <- vapply(seq_along(components), function(i) {
    c <- components[[i]]
    x <- c$x %||% (80 + (i %% 6) * 100)
    y <- c$y %||% (60 + (i %/% 6) * 90)
    col <- status_color(c$status %||% "unknown")
    name <- htmltools::htmlEscape(c$name %||% c$id %||% "?")
    icon <- c$icon %||% "•"
    paste0("<g transform='translate(", x, ",", y, ")'>",
           "<circle r='28' fill='#0a1628' stroke='", col, "' stroke-width='2'/>",
           "<text text-anchor='middle' y='5' font-size='16'>", icon, "</text>",
           "<text text-anchor='middle' y='48' fill='", col, "' font-size='10'>", name, "</text></g>")
  }, character(1))
  paste0("<svg viewBox='0 0 ", width, " ", height, "' width='100%' height='", height,
         "' style='background:#050d18;border-radius:8px'>", paste(nodes, collapse = ""), "</svg>")
}

agentic_topology_svg <- function(topology) {
  if (is.null(topology) || is.null(topology$nodes))
    return("<div style='color:#78909c;padding:12px'>Hakuna topology</div>")
  nodes <- topology$nodes
  edges <- topology$edges %||% list()
  edge_lines <- vapply(edges, function(e) {
    a <- Find(function(n) identical(n$id, e$from), nodes)
    b <- Find(function(n) identical(n$id, e$to), nodes)
    if (is.null(a) || is.null(b)) return("")
    paste0("<line x1='", a$x, "' y1='", a$y, "' x2='", b$x, "' y2='", b$y, "' stroke='#00e5ff55' stroke-width='2'/>")
  }, character(1))
  node_els <- vapply(nodes, function(n) {
    col <- status_color(n$status %||% "online")
    lab <- htmltools::htmlEscape(n$label %||% n$id)
    paste0("<g transform='translate(", n$x, ",", n$y, ")'>",
           "<rect x='-40' y='-16' width='80' height='32' rx='6' fill='#0a1628' stroke='", col, "'/>",
           "<text text-anchor='middle' y='5' fill='", col, "' font-size='11'>", lab, "</text></g>")
  }, character(1))
  paste0("<svg viewBox='0 0 620 380' width='100%' height='280' style='background:#050d18;border-radius:8px'>",
         paste(edge_lines, collapse = ""), paste(node_els, collapse = ""), "</svg>")
}

agentic_processes_el <- function(processes, lang) {
  if (is.null(processes) || length(processes) == 0)
    return(tags$div(style = "color:var(--dim)", tryCatch(tr("av.no_proc", lang), error = function(e) "—")))
  tags$div(class = "proc-list", lapply(processes, function(p) {
    st <- p$status %||% "good"
    tags$div(class = paste("proc-row", if (st == "critical") "danger"),
      tags$div(tags$div(class = "proc-name", p$name %||% "?"),
               tags$div(class = "proc-pid", paste0("CPU ", p$cpu %||% 0, "% · RAM ", p$ram %||% 0, "%"))),
      tags$div(meter_el(p$cpu %||% 0, (p$cpu %||% 0) >= 70),
               tags$span(class = paste("badge", if (st == "critical") "crit" else if (st == "warning") "warn" else "ok"), st)))
  }))
}

agentic_issues_el <- function(issues, lang) {
  if (is.null(issues) || length(issues) == 0)
    return(tags$div(class = "status-banner INFO", tryCatch(tr("av.no_issues", lang), error = function(e) "Hakuna")))
  tags$div(lapply(issues, function(iss) {
    tags$div(class = "issue warn",
      tags$span(class = "ico", "⚠"),
      tags$div(class = "body",
        tags$div(class = "t", iss$title %||% ""),
        tags$div(class = "d", iss$desc %||% ""),
        tags$div(class = "ts", paste0("→ ", iss$action %||% ""))))
  }))
}

agentic_pipeline_el <- function(steps, current_idx = 0, lang) {
  if (is.null(steps) || length(steps) == 0) return(NULL)
  tags$div(class = "av-pipeline", style = "display:flex;flex-wrap:wrap;gap:8px;margin:12px 0",
    lapply(seq_along(steps), function(i) {
      s <- steps[[i]]
      active <- identical(i - 1L, as.integer(current_idx))
      done <- (i - 1L) < as.integer(current_idx)
      col <- if (done) "#00e676" else if (active) "#00e5ff" else "#455a64"
      tags$div(style = paste0("border:1px solid ", col, ";border-radius:8px;padding:8px 12px;min-width:100px;background:#0a1628"),
        tags$div(style = paste0("color:", col, ";font-weight:700;font-size:12px"),
                 paste0(s$code %||% i, " · ", s$name_sw %||% s$id)),
        tags$div(style = "font-size:10px;color:var(--dim);margin-top:4px", s$description_sw %||% ""))
    }))
}

agentic_agents_el <- function(agents_file, lang) {
  ags <- agents_file$agents %||% list()
  tags$div(style = "display:grid;grid-template-columns:repeat(auto-fill,minmax(160px,1fr));gap:8px",
    lapply(ags, function(a) {
      tags$div(style = "border:1px solid #00e5ff33;border-radius:8px;padding:10px;background:#0a1628",
        tags$div(style = "color:var(--cyan);font-weight:700;font-size:13px", a$name_sw %||% a$id),
        tags$div(style = "font-size:11px;color:var(--dim);margin-top:4px", a$role %||% ""),
        tags$div(style = "font-size:10px;margin-top:6px",
                 if (isTRUE(a$requires_hitl)) "🔒 HITL" else "⚡ auto"))
    }))
}

view_agentic <- function(lang, data, session_state = NULL) {
  ad <- data$agent_data
  components <- ad$components %||% list()
  processes  <- ad$processes %||% list()
  topology   <- ad$topology %||% list()
  issues     <- ad$issues %||% list()
  pipe_steps <- (data$pipeline$steps) %||% list()
  greet <- (data$voice$scripts[["agent.receptionist.greet"]]) %||% "Habari. Mimi ni Mtaalamu Smart."
  kb <- data$knowledge
  lookup <- session_state$knowledge_lookup

  tags$div(class = "av-wrap",
    panel_el(tryCatch(tr("av.qa.title", lang), error = function(e) "Uliza / Anza"),
             tryCatch(tr("av.qa.meta", lang), error = function(e) "Agentic"),
      tags$div(
        tags$div(class = "field",
          tags$label(tryCatch(tr("av.qa.label", lang), error = function(e) "Tatizo")),
          tags$textarea(id = "av_msg", class = "form-control", rows = 3,
            style = "width:100%;background:#0a1628;color:#e0f7fa;border:1px solid #00e5ff44",
            placeholder = tryCatch(tr("av.qa.placeholder", lang), error = function(e) "Mf. kompyuta inaenda polepole..."))),
        tags$div(style = "display:flex;gap:10px;margin-top:10px;flex-wrap:wrap",
          tags$button(class = "trade-pill active",
            onclick = "Shiny.setInputValue('av_start', Date.now(), {priority:'event'})",
            tryCatch(tr("av.btn.start", lang), error = function(e) "ANZA")),
          tags$button(class = "trade-pill",
            onclick = "Shiny.setInputValue('av_approve', Date.now(), {priority:'event'})",
            tryCatch(tr("av.btn.approve", lang), error = function(e) "RUHUSU")),
          tags$button(class = "trade-pill",
            onclick = "Shiny.setInputValue('av_scan', Date.now(), {priority:'event'})",
            tryCatch(tr("av.btn.scan", lang), error = function(e) "SCAN")),
          tags$button(class = "trade-pill",
            onclick = "if(window.mtaalamuSpeakLog){mtaalamuSpeakLog('#av_narration_text');}",
            tryCatch(tr("av.btn.voice", lang), error = function(e) "SAUTI"))),
        tags$div(id = "av_voice_text", style = "display:none", greet),
        uiOutput("av_session_status"))),

    # DATA catalog from data/
    panel_el("📚 DATA / KNOWLEDGE BASE", "problems.json · diagnosis.json · devices_solver · trades",
      tags$div(
        knowledge_catalog_summary_el(kb, lang),
        tags$div(style = "margin-top:12px", uiOutput("av_knowledge"))
      )),

    panel_el("⚡ OS PROBE (HALISI)", "CPU · RAM · Disk · Health",
      uiOutput("av_sysprobe")),

    panel_el(tryCatch(tr("av.agents.title", lang), error = function(e) "Agents"),
             tryCatch(tr("av.agents.meta", lang), error = function(e) ""),
      agentic_agents_el(data$agents, lang)),

    panel_el(tryCatch(tr("av.pipe.title", lang), error = function(e) "Pipeline"),
             tryCatch(tr("av.pipe.meta", lang), error = function(e) ""),
      agentic_pipeline_el(pipe_steps, session_state$pipe_idx %||% 0, lang)),

    tags$div(class = "grid",
      panel_el(tryCatch(tr("av.map.title", lang), error = function(e) "Ramani"),
               tryCatch(tr("av.map.meta", lang), error = function(e) ""),
        HTML(agentic_device_map_svg(components))),
      panel_el(tryCatch(tr("av.proc.title", lang), error = function(e) "Processes"),
               tryCatch(tr("av.proc.meta", lang), error = function(e) ""),
        agentic_processes_el(processes, lang))),

    tags$div(class = "grid grid-bottom",
      panel_el(tryCatch(tr("av.topo.title", lang), error = function(e) "Topology"),
               tryCatch(tr("av.topo.meta", lang), error = function(e) ""),
        HTML(agentic_topology_svg(topology))),
      panel_el(tryCatch(tr("av.iss.title", lang), error = function(e) "Issues"),
               tryCatch(tr("av.iss.meta", lang), error = function(e) ""),
        agentic_issues_el(issues, lang))),

    panel_el(tryCatch(tr("av.bus.title", lang), error = function(e) "Bus"),
             tryCatch(tr("av.bus.meta", lang), error = function(e) ""),
      tags$div(style = "display:flex;flex-wrap:wrap;gap:8px",
        lapply((data$system_bus$buses) %||% list(), function(b) {
          tags$div(style = "border:1px solid #00e5ff33;border-radius:6px;padding:8px 12px;background:#0a1628",
            tags$div(style = "color:var(--cyan);font-size:12px;font-weight:700", b$name_sw %||% b$id),
            tags$div(style = "font-size:10px;color:var(--dim)", paste((b$path %||% list()), collapse = " → ")))
        }))),

    panel_el(tryCatch(tr("av.report.title", lang), error = function(e) "Ripoti"),
             tryCatch(tr("av.report.meta", lang), error = function(e) ""),
      tags$div(
        uiOutput("av_report_out"),
        tags$div(id = "av_narration_text", style = "position:absolute;left:-9999px;height:1px;overflow:hidden",
                 textOutput("av_narration_plain", inline = TRUE)),
        downloadButton("av_book_html", "📕 Download Kitabu (HTML)",
                       class = "btn btn-default", style = "margin-top:12px")
      ))
  )
}

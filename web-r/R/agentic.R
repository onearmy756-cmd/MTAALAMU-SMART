# ============================================================
# agentic.R — Agentic Vision UI (Q&A + Automatic + Live Vision)
# Data-driven kutoka agent_data.json + vision JSON
# ============================================================

# ---- load agentic data (root data/ au web-r/data) ----
load_agentic_data <- function() {
  roots <- c(
    p_root("data"),
    p_app("data"),
    file.path(dirname(APP_DIR), "data")
  )
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
    tryCatch(fromJSON(f, simplifyVector = FALSE), error = function(e) default)
  }
  list(
    agent_data = readj("agent_data.json"),
    agents     = readj("agents/agents_10.json"),
    pipeline   = readj("vision/pipeline.json"),
    device_map = readj("vision/device_map.json"),
    system_bus = readj("vision/system_bus.json"),
    voice      = readj("vision/voice_scripts_sw.json"),
    report_tpl = readj("vision/report_template.json")
  )
}

status_color <- function(st) {
  switch(tolower(as.character(st)),
    good = "#00e676", online = "#00e676", ok = "#00e676",
    warning = "#ffc107", warn = "#ffc107",
    critical = "#ff1744", crit = "#ff1744", offline = "#78909c",
    "#90a4ae")
}

# ---- Device map SVG from components ----
agentic_device_map_svg <- function(components, width = 640, height = 360) {
  if (is.null(components) || length(components) == 0) {
    return("<svg width='640' height='80'><text x='20' y='40' fill='#78909c'>Hakuna data ya ramani</text></svg>")
  }
  nodes <- vapply(seq_along(components), function(i) {
    c <- components[[i]]
    x <- c$x %||% (80 + (i %% 6) * 100)
    y <- c$y %||% (60 + (i %/% 6) * 90)
    col <- status_color(c$status %||% "unknown")
    name <- c$name %||% c$id %||% "?"
    icon <- c$icon %||% "•"
    sprintf(
      paste0(
        "<g transform='translate(%.0f,%.0f)'>",
        "<circle r='28' fill='#0a1628' stroke='%s' stroke-width='2'/>",
        "<text text-anchor='middle' y='5' font-size='16'>%s</text>",
        "<text text-anchor='middle' y='48' fill='%s' font-size='10' font-family='system-ui'>%s</text>",
        "</g>"
      ),
      x, y, col, icon, col, htmltools::htmlEscape(name)
    )
  }, character(1))
  sprintf(
    "<svg viewBox='0 0 %d %d' width='100%%' height='%d' style='background:#050d18;border-radius:8px'>%s</svg>",
    width, height, height, paste(nodes, collapse = "")
  )
}

# ---- Topology SVG ----
agentic_topology_svg <- function(topology) {
  if (is.null(topology) || is.null(topology$nodes)) {
    return("<div style='color:#78909c;padding:12px'>Hakuna topology</div>")
  }
  nodes <- topology$nodes
  edges <- topology$edges %||% list()
  edge_lines <- vapply(edges, function(e) {
    a <- Find(function(n) identical(n$id, e$from), nodes)
    b <- Find(function(n) identical(n$id, e$to), nodes)
    if (is.null(a) || is.null(b)) return("")
    sprintf("<line x1='%.0f' y1='%.0f' x2='%.0f' y2='%.0f' stroke='#00e5ff55' stroke-width='2'/>",
            a$x, a$y, b$x, b$y)
  }, character(1))
  node_els <- vapply(nodes, function(n) {
    col <- status_color(n$status %||% "online")
    sprintf(
      "<g transform='translate(%.0f,%.0f)'><rect x='-40' y='-16' width='80' height='32' rx='6' fill='#0a1628' stroke='%s'/><text text-anchor='middle' y='5' fill='%s' font-size='11'>%s</text></g>",
      n$x, n$y, col, col, htmltools::htmlEscape(n$label %||% n$id)
    )
  }, character(1))
  sprintf(
    "<svg viewBox='0 0 620 380' width='100%%' height='280' style='background:#050d18;border-radius:8px'>%s%s</svg>",
    paste(edge_lines, collapse = ""), paste(node_els, collapse = "")
  )
}

# ---- Process table ----
agentic_processes_el <- function(processes, lang) {
  if (is.null(processes) || length(processes) == 0) {
    return(tags$div(style = "color:var(--dim)", tr("av.no_proc", lang)))
  }
  tags$div(class = "proc-list",
    lapply(processes, function(p) {
      st <- p$status %||% "good"
      tags$div(class = paste("proc-row", if (st == "critical") "danger"),
        tags$div(
          tags$div(class = "proc-name", p$name %||% "?"),
          tags$div(class = "proc-pid", paste0("CPU ", p$cpu %||% 0, "% · RAM ", p$ram %||% 0, "%"))),
        tags$div(
          meter_el(p$cpu %||% 0, (p$cpu %||% 0) >= 70),
          tags$span(class = paste("badge", if (st == "critical") "crit" else if (st == "warning") "warn" else "ok"),
                    st)))
    }))
}

# ---- Issues ----
agentic_issues_el <- function(issues, lang) {
  if (is.null(issues) || length(issues) == 0) {
    return(tags$div(class = "status-banner INFO", tr("av.no_issues", lang)))
  }
  tags$div(
    lapply(issues, function(iss) {
      tags$div(class = "issue warn",
        tags$span(class = "ico", "⚠"),
        tags$div(class = "body",
          tags$div(class = "t", iss$title %||% ""),
          tags$div(class = "d", iss$desc %||% ""),
          tags$div(class = "ts", paste0("→ ", iss$action %||% ""))))
    }))
}

# ---- Pipeline steps strip ----
agentic_pipeline_el <- function(steps, current_idx = 0, lang) {
  if (is.null(steps) || length(steps) == 0) return(NULL)
  tags$div(class = "av-pipeline",
    style = "display:flex;flex-wrap:wrap;gap:8px;margin:12px 0",
    lapply(seq_along(steps), function(i) {
      s <- steps[[i]]
      active <- identical(i - 1L, as.integer(current_idx))
      done <- (i - 1L) < as.integer(current_idx)
      col <- if (done) "#00e676" else if (active) "#00e5ff" else "#455a64"
      tags$div(
        style = sprintf(
          "border:1px solid %s;border-radius:8px;padding:8px 12px;min-width:100px;background:#0a1628",
          col),
        tags$div(style = paste0("color:", col, ";font-weight:700;font-size:12px"),
                 paste0(s$code %||% i, " · ", s$name_sw %||% s$id)),
        tags$div(style = "font-size:10px;color:var(--dim);margin-top:4px",
                 s$description_sw %||% ""))
    }))
}

# ---- Agents list ----
agentic_agents_el <- function(agents_file, lang) {
  ags <- agents_file$agents %||% list()
  tags$div(style = "display:grid;grid-template-columns:repeat(auto-fill,minmax(160px,1fr));gap:8px",
    lapply(ags, function(a) {
      tags$div(
        style = "border:1px solid #00e5ff33;border-radius:8px;padding:10px;background:#0a1628",
        tags$div(style = "color:var(--cyan);font-weight:700;font-size:13px", a$name_sw %||% a$id),
        tags$div(style = "font-size:11px;color:var(--dim);margin-top:4px", a$role %||% ""),
        tags$div(style = "font-size:10px;margin-top:6px",
                 if (isTRUE(a$requires_hitl)) "🔒 HITL" else "⚡ auto"))
    }))
}

# ---- Main Agentic Vision panel ----
view_agentic <- function(lang, data, session_state = NULL) {
  ad <- data$agent_data
  components <- ad$components %||% list()
  processes  <- ad$processes %||% list()
  topology   <- ad$topology %||% list()
  issues     <- ad$issues %||% list()
  pipe_steps <- (data$pipeline$steps) %||% list()

  tags$div(class = "av-wrap",
    # Q&A + controls
    panel_el(tr("av.qa.title", lang), tr("av.qa.meta", lang),
      tags$div(
        tags$div(class = "field",
          tags$label(tr("av.qa.label", lang)),
          tags$textarea(
            id = "av_msg", class = "form-control",
            rows = 3, style = "width:100%;background:#0a1628;color:#e0f7fa;border:1px solid #00e5ff44",
            placeholder = tr("av.qa.placeholder", lang))),
        tags$div(style = "display:flex;gap:10px;margin-top:10px;flex-wrap:wrap",
          tags$button(
            class = "trade-pill active",
            onclick = "Shiny.setInputValue('av_start', Date.now(), {priority:'event'})",
            tr("av.btn.start", lang)),
          tags$button(
            class = "trade-pill",
            onclick = "Shiny.setInputValue('av_approve', Date.now(), {priority:'event'})",
            tr("av.btn.approve", lang)),
          tags$button(
            class = "trade-pill",
            onclick = "Shiny.setInputValue('av_scan', Date.now(), {priority:'event'})",
            tr("av.btn.scan", lang)),
          tags$button(
            class = "trade-pill",
            onclick = "if(window.speechSynthesis){var u=new SpeechSynthesisUtterance(document.getElementById('av_voice_text')?.innerText||'Habari');u.lang='sw';speechSynthesis.speak(u)}",
            tr("av.btn.voice", lang))),
        tags$div(id = "av_voice_text", style = "display:none",
                 (data$voice$scripts[["agent.receptionist.greet"]]) %||% "Habari. Mimi ni Mtaalamu Smart."),
        uiOutput("av_session_status"))),

    # Multi-agent strip
    panel_el(tr("av.agents.title", lang), tr("av.agents.meta", lang),
      agentic_agents_el(data$agents, lang)),

    # Pipeline
    panel_el(tr("av.pipe.title", lang), tr("av.pipe.meta", lang),
      agentic_pipeline_el(pipe_steps, session_state$pipe_idx %||% 0, lang)),

    # Live Vision grid
    tags$div(class = "grid",
      panel_el(tr("av.map.title", lang), tr("av.map.meta", lang),
        HTML(agentic_device_map_svg(components))),
      panel_el(tr("av.proc.title", lang), tr("av.proc.meta", lang),
        agentic_processes_el(processes, lang))),

    tags$div(class = "grid grid-bottom",
      panel_el(tr("av.topo.title", lang), tr("av.topo.meta", lang),
        HTML(agentic_topology_svg(topology))),
      panel_el(tr("av.iss.title", lang), tr("av.iss.meta", lang),
        agentic_issues_el(issues, lang))),

    # Bus status
    panel_el(tr("av.bus.title", lang), tr("av.bus.meta", lang),
      tags$div(style = "display:flex;flex-wrap:wrap;gap:8px",
        lapply((data$system_bus$buses) %||% list(), function(b) {
          tags$div(
            style = "border:1px solid #00e5ff33;border-radius:6px;padding:8px 12px;background:#0a1628",
            tags$div(style = "color:var(--cyan);font-size:12px;font-weight:700", b$name_sw %||% b$id),
            tags$div(style = "font-size:10px;color:var(--dim)",
                     paste((b$path %||% list()), collapse = " → ")))
        }))),

    # Report / log
    panel_el(tr("av.report.title", lang), tr("av.report.meta", lang),
      uiOutput("av_report_out"))
  )
}

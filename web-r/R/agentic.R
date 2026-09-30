# ============================================================
# agentic.R — Agentic Vision UI + data/*.json + SOLVE panel
# + SYSTEM WIRING (ramani halisi ya vifaa) + AUTO-WORK + SCRIBE
# ============================================================

# ---- Wiring: ita engine ya Rust au R-sysprobe fallback (HALISI zote mbili) ----
wiring_snapshot <- function(top = 10) {
  # 1) Rust engine (inatoa PCI/USB/DMI halisi kama ipo)
  bin <- c("../engine-rust/target/release/mtaalamu", "engine-rust/target/release/mtaalamu.exe",
           "engine-rust/target/release/mtaalamu")
  bin <- Filter(file.exists, bin)
  if (length(bin)) {
    out <- tryCatch(
      suppressWarnings(system(paste(bin[1], "wiring --top", top), intern = TRUE, timeout = 30)),
      error = function(e) NULL)
    if (!is.null(out) && length(out) > 0) {
      j <- tryCatch(jsonlite::fromJSON(paste(out, collapse = "\n"), simplifyVector = FALSE),
                    error = function(e) NULL)
      if (!is.null(j) && !is.null(j$nodes)) return(j)
    }
  }
  # 2) Fallback: R-sysprobe (CPU/RAM/disk/procs halisi — bila PCI/USB)
  snap <- tryCatch(sysprobe_snapshot(), error = function(e) NULL)
  if (is.null(snap)) return(NULL)
  cpu <- snap$cpu_usage_pct %||% NA
  ram <- snap$ram_usage_pct %||% NA
  st <- function(p) if (is.na(p)) "unknown" else if (p >= 90) "critical" else if (p >= 75) "warning" else "good"
  nodes <- list(
    list(id = "cpu0", label = sprintf("CPU %s%%", if (is.na(cpu)) "?" else round(cpu)),
         kind = "cpu", status = st(cpu), detail = snap$os %||% ""),
    list(id = "ram0", label = sprintf("RAM %s%%", if (is.na(ram)) "?" else round(ram)),
         kind = "memory", status = st(ram), detail = "halisi"),
    list(id = "kernel", label = paste("Kernel", snap$hostname %||% "?"),
         kind = "kernel", status = "good", detail = snap$source %||% "r-live"))
  edges <- list(
    list(from = "cpu0", to = "ram0", bus = "memory", label = "memory controller"),
    list(from = "cpu0", to = "kernel", bus = "kernel", label = "scheduler"))
  # disks halisi
  for (i in seq_along(snap$disks %||% list())) {
    d <- snap$disks[[i]]
    id <- paste0("disk", i - 1)
    nodes[[length(nodes) + 1]] <- list(id = id,
      label = sprintf("Disk %s %s%%", d$mount %||% "?", round(d$used_pct %||% 0)),
      kind = "disk", status = d$status %||% "good", detail = "block device halisi")
    edges[[length(edges) + 1]] <- list(from = "kernel", to = id, bus = "sata", label = "storage")
  }
  # processes halisi
  for (i in seq_len(min(6, length(snap$top_processes %||% list())))) {
    p <- snap$top_processes[[i]]
    id <- paste0("proc", i - 1)
    nodes[[length(nodes) + 1]] <- list(id = id,
      label = substr(p$name %||% "?", 1, 16), kind = "process",
      status = p$status %||% "good", detail = sprintf("CPU %s%%", p$cpu %||% 0))
    edges[[length(edges) + 1]] <- list(from = "kernel", to = id, bus = "kernel", label = "process")
  }
  flows <- list(list(id = "io_disk", path = list("disk0", "kernel", "cpu0"), kind = "io",
                     rate_note = "I/O halisi"))
  list(source = "r-sysprobe-live", hostname = snap$hostname %||% "", kernel = snap$os %||% "",
       nodes = nodes, edges = edges, flows = flows,
       buses = list(list(id = "mem", name_sw = "Memory Bus", nodes = list("ram0", "cpu0"),
                         health = "good", throughput_note = "halisi")))
}

# ---- Wiring SVG: nodes zenye bus-lanes halisi (data-driven kutoka wiring_snapshot) ----
wiring_svg <- function(w, width = 760, height = 420) {
  if (is.null(w) || length(w$nodes) == 0)
    return("<div style='color:#78909c;padding:12px'>Wiring haipatikani (anza Rust binary)</div>")
  bus_col <- function(b) switch(b,
    memory = "#00e5ff", sata = "#ff9100", usb = "#76ff03", pcie = "#e040fb",
    net = "#40c4ff", kernel = "#ffd740", "#90a4ae")
  kind_icon <- function(k) switch(k,
    cpu = "\U0001f9e0", memory = "\U0001f4be", disk = "\U0001f5c4\ufe0f", usb = "\U0001f50c",
    pci = "\U0001f5a5\ufe0f", net = "\U0001f310", thermal = "\U0001f321\ufe0f",
    kernel = "\u2699\ufe0f", process = "\u25b6", "\u2022")
  # layout: cluster kwa kind
  kinds <- c("cpu", "memory", "disk", "usb", "pci", "net", "thermal", "kernel", "process")
  pos <- list()
  i <- 0L
  for (k in kinds) {
    ids <- vapply(Filter(function(n) identical(n$kind, k), w$nodes), function(n) n$id, "")
    for (j in seq_along(ids)) {
      i <- i + 1L
      pos[[ids[j]]] <- list(x = 70 + ((j - 1) %% 4) * 170,
                            y = 60 + (which(kinds == k) - 1) * 46)
    }
  }
  edges <- paste(vapply(w$edges %||% list(), function(e) {
    a <- pos[[e$from]]; b <- pos[[e$to]]
    if (is.null(a) || is.null(b)) return("")
    col <- bus_col(e$bus %||% "")
    sprintf("<path d='M%d,%d C%d,%d %d,%d %d,%d' stroke='%s' stroke-width='2' fill='none' opacity='0.75'/>",
            a$x, a$y, (a$x + b$x) / 2, a$y, (a$x + b$x) / 2, b$y, b$x, b$y, col)
  }, character(1)), collapse = "")
  node_els <- paste(vapply(w$nodes, function(n) {
    p <- pos[[n$id]]
    if (is.null(p)) return("")
    col <- status_color(n$status %||% "good")
    ic <- kind_icon(n$kind %||% "")
    lab <- htmltools::htmlEscape(substr(n$label %||% "?", 1, 26))
    det <- htmltools::htmlEscape(substr(n$detail %||% "", 1, 30))
    sprintf(paste0("<g transform='translate(%d,%d)'><circle r='16' fill='#0a1628' stroke='%s' stroke-width='2'/>",
                   "<text text-anchor='middle' y='5' font-size='13'>%s</text>",
                   "<text text-anchor='middle' y='32' fill='%s' font-size='9'>%s</text>",
                   "<title>%s</title></g>"),
            p$x, p$y, col, ic, col, lab, det)
  }, character(1)), collapse = "")
  flows <- length(w$flows %||% list())
  paste0("<svg viewBox='0 0 ", width, " ", height, "' width='100%' height='", height,
         "' style='background:#050d18;border-radius:8px'>", edges, node_els,
         "<text x='12' y='16' fill='#00e5ff' font-size='11'>WIRING HALISI · flows: ", flows,
         " · source: ", htmltools::htmlEscape(w$source %||% "?"), "</text></svg>")
}

# ---- Data flow strip: Input → ... → Output (kutoka flows halisi) ----
wiring_flows_el <- function(w) {
  if (is.null(w) || length(w$flows %||% list()) == 0)
    return(tags$div(style = "color:var(--dim)", "Hakuna flows"))
  tags$div(lapply(w$flows, function(f) {
    tags$div(style = "border:1px solid #00e5ff22;border-radius:6px;padding:6px 10px;margin:4px 0;background:#0a1628",
      tags$span(style = "color:var(--cyan);font-size:12px;font-weight:700", f$id %||% "?"),
      tags$span(style = "font-size:12px;color:#b2ebf2;margin-left:10px",
                paste(unlist(f$path %||% list()), collapse = " → ")),
      tags$div(style = "font-size:10px;color:var(--dim)", f$rate_note %||% ""))
  }))
}

# ---- Kitabu kidigitali cha AUTO-WORK (HTML book: tatizo→njia→suluhisho→tarehe) ----
autowork_book_html <- function(aw) {
  esc <- htmltools::htmlEscape
  sid <- esc(aw$session_id %||% "AW")
  when <- format(Sys.time(), "%Y-%m-%d %H:%M:%S")
  frames <- aw$frames %||% list()
  findings <- aw$findings %||% list()
  # Sura: kila frame = mabadiliko yenye muda wake (relative)
  t0 <- NULL
  chapters <- vapply(seq_along(frames), function(i) {
    f <- frames[[i]]
    if (is.null(t0)) t0 <<- f$ts %||% 0
    dt <- max(0, as.numeric(f$ts %||% 0) - (t0 %||% 0))
    col <- switch(f$step_code %||% "", P = "#00e5ff", I = "#ffd740", T = "#76ff03",
                  V = "#00e676", D = "#e040fb", SCAN = "#40c4ff", HITL = "#ffc107", "#90a4ae")
    sprintf(paste0("<section style='border-left:4px solid %s;padding:10px 14px;margin:10px 0;background:#0d1b2a;border-radius:6px'>",
      "<div style='color:%s;font-weight:700;font-size:13px'>SURA %d · %s (%s) · +%ds</div>",
      "<p style='color:#e0f7fa;font-size:13px;margin:8px 0 4px'>%s</p>",
      "<p style='color:#b0bec5;font-size:11px;margin:2px 0'>\U0001f5bc %s</p>",
      "</section>"),
      col, col, i, esc(f$step_name_sw %||% ""), esc(f$step_code %||% ""), dt,
      esc(f$narration_sw %||% ""), esc(f$visual_note %||% ""))
  }, character(1))
  findings_html <- if (length(findings) == 0) "" else paste(
    "<h3 style='color:#00e5ff'>MATATIZO YALIYOGUNDULIWA</h3>",
    paste(vapply(findings, function(f) {
      col <- switch(f$severity %||% "info", critical = "#ff1744", warning = "#ffc107", "#00e676")
      sprintf("<div style='border-left:3px solid %s;padding:6px 10px;margin:6px 0'><b>%s</b> <span style='color:%s;font-size:10px'>%s</span></div>",
              col, esc(f$title %||% "?"), col, esc(toupper(f$severity %||% "")))
    }, character(1)), collapse = ""))
  sprintf(paste0("<!DOCTYPE html><html lang='sw'><head><meta charset='utf-8'/><title>Kitabu %s</title></head>",
    "<body style='font-family:system-ui;background:#0a1628;color:#e0f7fa;margin:0;padding:24px;max-width:860px'>",
    "<div style='text-align:center;border:2px solid #00e5ff;border-radius:12px;padding:18px;background:#0d2137'>",
    "<h1 style='color:#00e5ff;margin:0'>\U0001f4d8 MTAALAMU SMART — KITABU KIDIGITALI</h1>",
    "<p style='color:#b2ebf2'>Session: %s<br/>Tarehe: %s<br/>Agent: AUTO-WORK + AI SCRIBE</p></div>",
    "<h2 style='color:#00e5ff;margin-top:18px'>HABARI NA MATUKIO</h2>",
    "%s%s",
    "<p style='color:#78909c;font-size:11px;margin-top:20px'>Imetengenezwa na MTAALAMU SMART — data halisi za OS; hakuna uongo.</p>",
    "</body></html>"),
    sid, sid, when, paste(chapters, collapse = ""), findings_html)
}

# ---- Scribe strip: frames za PIITVD (kama video-strip ya hatua) ----
scribe_strip_el <- function(frames) {
  if (length(frames %||% list()) == 0)
    return(tags$div(style = "color:var(--dim)", "Hakuna narration bado — bonyeza ANZA au AUTO-WORK"))
  tags$div(style = "display:flex;gap:8px;overflow-x:auto;padding:6px 0",
    lapply(frames, function(f) {
      col <- switch(f$step_code %||% "", P = "#00e5ff", I = "#ffd740", T = "#76ff03",
                    V = "#00e676", D = "#e040fb", SCAN = "#40c4ff", HITL = "#ffc107", "#90a4ae")
      tags$div(style = paste0("min-width:210px;max-width:210px;border:1px solid ", col,
                              ";border-radius:8px;padding:8px;background:#0a1628;flex-shrink:0"),
        tags$div(style = paste0("color:", col, ";font-weight:700;font-size:12px"),
                 paste0(f$step_code %||% "?", " · ", f$step_name_sw %||% "")),
        tags$div(style = "font-size:11px;color:#b2ebf2;margin-top:4px;min-height:44px",
                 f$narration_sw %||% ""),
        tags$div(style = "font-size:10px;color:var(--dim);margin-top:4px",
                 paste0("\U0001f4b5 ", f$voice_sw %||% "")),
        tags$div(style = "font-size:9px;color:#78909c;margin-top:2px",
                 paste0("\U0001f5bc ", f$visual_note %||% "")))
    }))
}

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
  if (is.null(components) || length(components) == 0)
    return("<svg width='640' height='80'><text x='20' y='40' fill='#78909c'>Hakuna data</text></svg>")
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
    return(tags$div(style = "color:var(--dim)", "—"))
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
    return(tags$div(class = "status-banner INFO", "Hakuna issues"))
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

  tags$div(class = "av-wrap",
    panel_el("Uliza / Anza", "Agentic Vision",
      tags$div(
        tags$div(class = "field",
          tags$label("Tatizo"),
          tags$textarea(id = "av_msg", class = "form-control", rows = 3,
            style = "width:100%;background:#0a1628;color:#e0f7fa;border:1px solid #00e5ff44",
            placeholder = "Mf. kompyuta inaenda polepole / overheat / virus...")),
        tags$div(style = "display:flex;gap:10px;margin-top:10px;flex-wrap:wrap",
          tags$button(class = "trade-pill active",
            onclick = "Shiny.setInputValue('av_start', Date.now(), {priority:'event'})", "ANZA"),
          tags$button(class = "trade-pill",
            onclick = "Shiny.setInputValue('av_approve', Date.now(), {priority:'event'})", "RUHUSU (HITL)"),
          tags$button(class = "trade-pill",
            onclick = "Shiny.setInputValue('av_scan', Date.now(), {priority:'event'})", "SCAN"),
          tags$button(class = "trade-pill",
            onclick = "Shiny.setInputValue('av_auto', Date.now(), {priority:'event'})", "AUTO-WORK"),
          tags$button(class = "trade-pill",
            onclick = "Shiny.setInputValue('av_wiring', Date.now(), {priority:'event'})", "WIRING LIVE"),
          tags$button(class = "trade-pill",
            onclick = "if(window.mtaalamuSpeakLog){mtaalamuSpeakLog('#av_narration_text');}", "SAUTI")),
        tags$div(id = "av_voice_text", style = "display:none", greet),
        uiOutput("av_session_status"))),

    panel_el("🔧 SOLVE (software / hardware)",
             "Baada ya RUHUSU: software auto · hardware = binadamu",
      uiOutput("av_solve")),

    panel_el("📚 DATA / KNOWLEDGE BASE", "problems.json · diagnosis.json · devices",
      tags$div(
        knowledge_catalog_summary_el(kb, lang),
        tags$div(style = "margin-top:12px", uiOutput("av_knowledge"))
      )),

    panel_el("⚡ OS PROBE (HALISI)", "CPU · RAM · Disk",
      uiOutput("av_sysprobe")),

    panel_el("🔌 SYSTEM WIRING (HALISI)",
             "Vifaa halisi: CPU · RAM · Disk · PCI · USB · Net · Thermal — kila kitu kime-labeliwa",
      tags$div(
        tags$div(style = "margin-bottom:8px",
          actionButton("av_wiring_refresh", "\U0001f504 Onyesha wiring", class = "btn btn-sm btn-default",
                       style = "font-size:12px"),
          tags$span(class = "meta", style = "margin-left:8px", uiOutput("av_wiring_meta"))),
        uiOutput("av_wiring_svg"),
        tags$div(style = "margin-top:8px", tags$h4(style = "font-size:13px;color:var(--cyan)", "DATA FLOW (Input → Output)"),
                 uiOutput("av_wiring_flows")))),

    panel_el("🤖 AUTO-WORK + AI SCRIBE",
             "Agent inagundua yenyewe → HITL → solve → kitabu (Kiswahili fasaha)",
      tags$div(
        uiOutput("av_autowork"),
        tags$h4(style = "font-size:13px;color:var(--cyan);margin-top:10px", "SCRIBE — hatua kwa hatua (video-strip)"),
        uiOutput("av_scribe"),
        downloadButton("av_autowork_book", "📕 Kitabu cha Auto-Work (HTML)",
                       class = "btn btn-default", style = "margin-top:10px"))),

    panel_el("Agents", "Multi-agent",
      agentic_agents_el(data$agents, lang)),

    panel_el("Pipeline", "PIITVD",
      agentic_pipeline_el(pipe_steps, session_state$pipe_idx %||% 0, lang)),

    tags$div(class = "grid",
      panel_el("Ramani", "OS components",
        HTML(agentic_device_map_svg(components))),
      panel_el("Processes", "Live",
        agentic_processes_el(processes, lang))),

    tags$div(class = "grid grid-bottom",
      panel_el("Topology", "",
        HTML(agentic_topology_svg(topology))),
      panel_el("Issues", "",
        agentic_issues_el(issues, lang))),

    panel_el("System bus", "",
      tags$div(
        tags$div(style = "display:flex;flex-wrap:wrap;gap:8px",
          lapply((data$system_bus$buses) %||% list(), function(b) {
            tags$div(style = "border:1px solid #00e5ff33;border-radius:6px;padding:8px 12px;background:#0a1628",
              tags$div(style = "color:var(--cyan);font-size:12px;font-weight:700", b$name_sw %||% b$id),
              tags$div(style = "font-size:10px;color:var(--dim)", paste((b$path %||% list()), collapse = " → ")))
          })),
        tags$div(style = "margin-top:10px",
          tags$h4(style = "font-size:13px;color:var(--cyan)", "BUS HALISI (wiring)"),
          uiOutput("av_wiring_buses")))),

    panel_el("Ripoti / Kitabu", "",
      tags$div(
        uiOutput("av_report_out"),
        tags$div(id = "av_narration_text", style = "position:absolute;left:-9999px;height:1px;overflow:hidden",
                 textOutput("av_narration_plain", inline = TRUE)),
        downloadButton("av_book_html", "📕 Download Kitabu (HTML)",
                       class = "btn btn-default", style = "margin-top:12px")
      ))
  )
}

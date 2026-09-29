# ============================================================
# scribe.R — AI Scribe + Digital Book (HTML / Markdown)
# Kiswahili fasaha, hatua kwa hatua — data-driven
# ============================================================

# ---- Voice script helper ----
av_voice <- function(data, key, fallback = "") {
  scripts <- data$voice$scripts %||% list()
  if (!is.null(scripts[[key]]) && nzchar(scripts[[key]])) return(scripts[[key]])
  fallback
}

# ---- Scribe: build narration lines (Kiswahili) ----
build_scribe_narration <- function(user_msg, issues, status = "running", lang = "sw") {
  lines <- character(0)
  ts <- format(Sys.time(), "%H:%M:%S")
  n <- length(issues %||% list())

  if (identical(lang, "en")) {
    lines <- c(
      paste0("[", ts, "] Session started."),
      paste0("Problem: ", user_msg),
      paste0("Vision detected ", n, " issue(s).")
    )
    if (n > 0) {
      for (i in seq_len(min(5L, n))) {
        iss <- issues[[i]]
        lines <- c(lines, paste0("  ", i, ". ", iss$title %||% "", " — ", iss$action %||% ""))
      }
    }
    if (identical(status, "hitl"))
      lines <- c(lines, "Waiting for your approval (HITL) before automatic fix.")
    if (identical(status, "done"))
      lines <- c(lines,
        "HITL approved. Implement → Test → Verify → Document completed.",
        "Digital book report is ready. Knowledge saved.")
  } else {
    lines <- c(
      paste0("[", ts, "] Session imeanza."),
      paste0("Tatizo: ", user_msg),
      paste0("Muono (Vision) umegundua matatizo ", n, ".")
    )
    if (n > 0) {
      for (i in seq_len(min(5L, n))) {
        iss <- issues[[i]]
        lines <- c(lines, paste0("  ", i, ". ", iss$title %||% "", " — ", iss$action %||% ""))
      }
    }
    if (identical(status, "hitl"))
      lines <- c(lines, "Ninasubiri ruhusa yako (HITL) kabla ya kurekebisha kiotomatiki.")
    if (identical(status, "done"))
      lines <- c(lines,
        "Ruhusa imetolewa. Tekeleza → Jaribu → Thibitisha → Andika — imekamilika.",
        "Ripoti ya kitabu kidigitali iko tayari. Maarifa yamehifadhiwa.")
  }
  lines
}

# ---- Voice queue texts for TTS ----
build_voice_queue <- function(data, phase = c("greet", "found", "hitl", "done"), n_issues = 0L) {
  phase <- match.arg(phase)
  q <- character(0)
  if (phase == "greet") {
    q <- c(av_voice(data, "agent.receptionist.greet",
                    "Habari. Mimi ni Mtaalamu Smart. Niambie tatizo lako."),
           av_voice(data, "agent.vision.scanning",
                    "Ninaangalia kifaa chako sasa."))
  } else if (phase == "found") {
    q <- c(sprintf("Nimegundua matatizo %d. Ninakueleza.", as.integer(n_issues)),
           av_voice(data, "agent.planner.ready",
                    "Nimeandaa mpango. Ninahitaji ruhusa yako."))
  } else if (phase == "hitl") {
    q <- av_voice(data, "hitl.ask_fix",
                  "Nimegundua matatizo. Je, unaniruhusu kuanza kurekebisha?")
  } else if (phase == "done") {
    q <- c(av_voice(data, "hitl.approved", "Asante. Ninaendelea."),
           av_voice(data, "agent.solver.working", "Ninafanya kazi ya kurekebisha."),
           av_voice(data, "agent.reporter.ready", "Ripoti yako iko tayari."),
           av_voice(data, "session.complete", "Kazi imekamilika."))
  }
  as.character(q)
}

# ---- Digital Book: Markdown ----
build_digital_book_md <- function(session_id, user_msg, issues, log_lines, lang = "sw") {
  ts <- format(Sys.time(), "%Y-%m-%d %H:%M:%S")
  n <- length(issues %||% list())
  md <- c(
    "# MTAALAMU SMART — Ripoti (Kitabu Kidigitali)",
    "",
    paste0("**Session:** ", session_id),
    paste0("**Tarehe:** ", ts),
    paste0("**Lugha:** ", lang),
    "",
    "---",
    "",
    "## Sura 1 — Tatizo",
    "",
    paste0("> ", user_msg),
    "",
    "## Sura 2 — Ugunduzi (Vision)",
    "",
    paste0("Matatizo yaliyogunduliwa: **", n, "**"),
    ""
  )
  if (n > 0) {
    for (i in seq_along(issues)) {
      iss <- issues[[i]]
      md <- c(md,
        paste0("### ", i, ". ", iss$title %||% "Tatizo"),
        paste0("- **Maelezo:** ", iss$desc %||% "—"),
        paste0("- **Kitendo:** ", iss$action %||% "—"),
        "")
    }
  }
  md <- c(md,
    "## Sura 3 — Mpango (PIITVD)",
    "",
    "1. **P** Panga",
    "2. **I** Tambua",
    "3. **I** Tekeleza (HITL)",
    "4. **T** Jaribu",
    "5. **V** Thibitisha",
    "6. **D** Andika",
    "",
    "## Sura 4 — Utekelezaji / Log",
    "",
    "```",
    log_lines %||% character(0),
    "```",
    "",
    "## Sura 5 — Hitimisho",
    "",
    "Kazi imeshughulikiwa na Mtaalamu Smart Agentic Vision.",
    "Maarifa yamehifadhiwa kwa matumizi ya baadaye.",
    "",
    "---",
    "",
    "*MTAALAMU SMART — production report*")
  paste(md, collapse = "\n")
}

# ---- Digital Book: HTML (print-friendly / save as PDF via browser) ----
build_digital_book_html <- function(session_id, user_msg, issues, log_lines, lang = "sw") {
  ts <- format(Sys.time(), "%Y-%m-%d %H:%M:%S")
  n <- length(issues %||% list())
  esc <- function(x) {
    x <- as.character(x %||% "")
    x <- gsub("&", "&amp;", x, fixed = TRUE)
    x <- gsub("<", "&lt;", x, fixed = TRUE)
    x <- gsub(">", "&gt;", x, fixed = TRUE)
    x
  }
  issue_html <- ""
  if (n > 0) {
    parts <- vapply(seq_along(issues), function(i) {
      iss <- issues[[i]]
      sprintf(
        "<div class='card'><h3>%d. %s</h3><p>%s</p><p class='act'><b>Kitendo:</b> %s</p></div>",
        i, esc(iss$title), esc(iss$desc), esc(iss$action))
    }, character(1))
    issue_html <- paste(parts, collapse = "\n")
  }
  log_pre <- paste(esc(log_lines %||% character(0)), collapse = "\n")

  paste0(
    "<!DOCTYPE html><html lang='", esc(lang), "'><head><meta charset='utf-8'/>",
    "<title>MTAALAMU SMART — Ripoti ", esc(session_id), "</title>",
    "<style>",
    "body{font-family:system-ui,Segoe UI,sans-serif;background:#0a1628;color:#e0f7fa;margin:0;padding:24px}",
    "h1{color:#00e5ff;border-bottom:2px solid #00e5ff44;padding-bottom:8px}",
    "h2{color:#69f0ae;margin-top:28px}",
    ".meta{color:#90a4ae;font-size:14px}",
    ".card{background:#0d2137;border:1px solid #00e5ff33;border-radius:10px;padding:14px 16px;margin:10px 0}",
    ".act{color:#ffc107}",
    "pre{background:#050d18;padding:14px;border-radius:8px;overflow:auto;font-size:12px;line-height:1.45}",
    "blockquote{border-left:4px solid #00e5ff;margin:12px 0;padding:8px 16px;background:#0d2137}",
    "@media print{body{background:#fff;color:#111} h1,h2{color:#033} .card{border-color:#ccc}}",
    "</style></head><body>",
    "<h1>🇹🇿 MTAALAMU SMART — Kitabu Kidigitali</h1>",
    "<p class='meta'>Session: <b>", esc(session_id), "</b> · ", esc(ts), "</p>",
    "<h2>Sura 1 — Tatizo</h2>",
    "<blockquote>", esc(user_msg), "</blockquote>",
    "<h2>Sura 2 — Ugunduzi (Vision)</h2>",
    "<p>Matatizo: <b>", n, "</b></p>",
    issue_html,
    "<h2>Sura 3 — Mpango PIITVD</h2>",
    "<ol><li>Panga</li><li>Tambua</li><li>Tekeleza (HITL)</li><li>Jaribu</li><li>Thibitisha</li><li>Andika</li></ol>",
    "<h2>Sura 4 — Log</h2><pre>", log_pre, "</pre>",
    "<h2>Sura 5 — Hitimisho</h2>",
    "<p>Kazi imeshughulikiwa na <b>Mtaalamu Smart Agentic Vision</b>. Maarifa yamehifadhiwa.</p>",
    "<p class='meta'>Print → Save as PDF kwa kitabu cha PDF.</p>",
    "</body></html>"
  )
}

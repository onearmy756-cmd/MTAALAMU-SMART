# ============================================================
# scribe.R — AI Scribe (Kiswahili) + Digital Book builder
# AV4: narration hatua-kwa-hatua
# AV5: kitabu kidigitali (HTML / Markdown)
# ============================================================

av_voice_script <- function(data, key, vars = list()) {
  scripts <- data$voice$scripts %||% list()
  txt <- scripts[[key]] %||% key
  if (length(vars) > 0) {
    for (nm in names(vars)) {
      txt <- gsub(paste0("\\{", nm, "\\}"), as.character(vars[[nm]]), txt, perl = TRUE)
    }
  }
  txt
}

# Scribe: unda maelezo ya Kiswahili kwa kila hatua
build_scribe_narration <- function(msg, issues, status = "hitl") {
  n <- length(issues)
  titles <- if (n > 0) {
    vapply(issues, function(x) as.character(x$title %||% ""), character(1))
  } else character(0)
  actions <- if (n > 0) {
    vapply(issues, function(x) as.character(x$action %||% ""), character(1))
  } else character(0)

  lines <- c(
    paste0("=== MWANDISHI (SCRIBE) — ", format(Sys.time(), "%Y-%m-%d %H:%M:%S"), " ==="),
    "",
    "SURA YA KWANZA — TATIZO",
    paste0("Mteja ameripoti: ", msg),
    "",
    "SURA YA PILI — UGUNDUZI (VISION)"
  )
  if (n == 0) {
    lines <- c(lines, "Hakuna matatizo makubwa yaliyorekodiwa kwenye snapshot.")
  } else {
    lines <- c(lines, paste0("Nimegundua matatizo ", n, ":"))
    for (i in seq_len(n)) {
      lines <- c(lines,
        paste0("  ", i, ". ", titles[i]),
        paste0("     Kitendo: ", actions[i]))
    }
  }
  lines <- c(lines, "",
    "SURA YA TATU — MPANGO (PIITVD)",
    "1. Panga — lengo na hatari",
    "2. Tambua — sababu na ushahidi",
    "3. Tekeleza — suluhisho (inahitaji ruhusa)",
    "4. Jaribu — metrics baada ya fix",
    "5. Thibitisha — mteja / mtaalamu",
    "6. Andika — ripoti na maarifa"
  )
  if (identical(status, "hitl")) {
    lines <- c(lines, "",
      "HALI: Ninasubiri ruhusa yako (HITL) kabla ya kutekeleza suluhisho.",
      "Bofya RUHUSU ili niendelee.")
  } else if (identical(status, "done")) {
    lines <- c(lines, "",
      "SURA YA NNE — UTEKELEZAJI",
      "Ruhusa imetolewa. Hatua za suluhisho zimetekelezwa kulingana na orodha ya matatizo.",
      "",
      "SURA YA TANO — UTHIBITISHO",
      "Majaribio ya afya yamekamilika. Metrics zimeangaliwa.",
      "",
      "SURA YA SITA — HITIMISHO",
      "Kazi imekamilika. Ripoti hii imehifadhiwa kama kitabu kidigitali.",
      "Maarifa yameongezwa kwenye mfumo wa kujifunza.")
  }
  paste(lines, collapse = "\n")
}

# Digital book HTML (print-friendly / save as PDF from browser)
build_digital_book_html <- function(session_id, msg, issues, narration, lang = "sw") {
  n <- length(issues)
  issue_rows <- if (n > 0) {
    paste(vapply(seq_len(n), function(i) {
      iss <- issues[[i]]
      sprintf("<tr><td>%d</td><td>%s</td><td>%s</td><td>%s</td></tr>",
              i,
              htmltools::htmlEscape(iss$title %||% ""),
              htmltools::htmlEscape(iss$desc %||% ""),
              htmltools::htmlEscape(iss$action %||% ""))
    }, character(1)), collapse = "\n")
  } else {
    "<tr><td colspan='4'>Hakuna matatizo</td></tr>"
  }

  title <- if (identical(lang, "en")) "Mtaalamu Smart — Digital Report Book" else
    "Mtaalamu Smart — Kitabu Kidigitali cha Ripoti"

  sprintf(r'{<!DOCTYPE html>
<html lang="%s">
<head>
<meta charset="utf-8"/>
<title>%s</title>
<style>
  @import url('https://fonts.googleapis.com/css2?family=Space+Grotesk:wght@400;600;700&display=swap');
  :root { --bg:#0a1628; --cyan:#00e5ff; --text:#e0f7fa; --dim:#78909c; --card:#0d2137; }
  * { box-sizing:border-box; }
  body { margin:0; font-family:'Space Grotesk',system-ui,sans-serif; background:var(--bg); color:var(--text); }
  .book { max-width:800px; margin:0 auto; padding:32px 24px 64px; }
  .cover { border:2px solid var(--cyan); border-radius:16px; padding:48px 32px; text-align:center;
           background:linear-gradient(160deg,#0a1628 0%%,#003d4d 100%%); margin-bottom:32px; }
  .cover h1 { color:var(--cyan); font-size:28px; margin:0 0 8px; letter-spacing:2px; }
  .cover .sub { color:var(--dim); font-size:13px; }
  .cover .sid { margin-top:20px; font-family:monospace; color:var(--cyan); font-size:12px; }
  h2 { color:var(--cyan); font-size:16px; border-bottom:1px solid #00e5ff33; padding-bottom:8px; margin-top:32px; }
  p, li { line-height:1.6; font-size:14px; color:#b2ebf2; }
  table { width:100%%; border-collapse:collapse; margin:16px 0; font-size:13px; }
  th, td { border:1px solid #00e5ff22; padding:10px; text-align:left; }
  th { background:#00e5ff15; color:var(--cyan); }
  pre { background:#050d18; border:1px solid #00e5ff22; border-radius:8px; padding:16px;
        white-space:pre-wrap; font-size:12px; color:#b2ebf2; }
  .footer { margin-top:48px; text-align:center; color:var(--dim); font-size:11px; }
  @media print {
    body { background:#fff; color:#111; }
    .cover { background:#f0f9ff; border-color:#0891b2; }
    .cover h1, h2, th { color:#0e7490; }
    p, li, td, pre { color:#134e4a; }
    pre { background:#f8fafc; }
  }
</style>
</head>
<body>
<div class="book">
  <div class="cover">
    <h1>%s</h1>
    <div class="sub">Agentic Vision · PIITVD · Multi-Agent 10</div>
    <div class="sid">SESSION %s</div>
    <div class="sub" style="margin-top:12px">%s</div>
  </div>

  <h2>1. Tatizo / Problem</h2>
  <p>%s</p>

  <h2>2. Ugunduzi (Vision)</h2>
  <table>
    <thead><tr><th>#</th><th>Tatizo</th><th>Maelezo</th><th>Kitendo</th></tr></thead>
    <tbody>
%s
    </tbody>
  </table>

  <h2>3. Maelezo ya Mwandishi (Scribe)</h2>
  <pre>%s</pre>

  <div class="footer">
    MTAALAMU SMART · Data-driven · Kiswahili kwanza · %s
  </div>
</div>
</body>
</html>
}', lang, title, title, htmltools::htmlEscape(session_id),
     format(Sys.time(), "%Y-%m-%d %H:%M:%S"),
     htmltools::htmlEscape(msg),
     issue_rows,
     htmltools::htmlEscape(narration),
     format(Sys.time(), "%Y"))
}

build_digital_book_md <- function(session_id, msg, issues, narration) {
  n <- length(issues)
  md <- c(
    "# Mtaalamu Smart — Kitabu Kidigitali",
    "",
    paste0("**Session:** `", session_id, "`  "),
    paste0("**Tarehe:** ", format(Sys.time(), "%Y-%m-%d %H:%M:%S")),
    "",
    "---",
    "",
    "## 1. Tatizo",
    "",
    msg,
    "",
    "## 2. Ugunduzi (Vision)",
    ""
  )
  if (n > 0) {
    for (i in seq_len(n)) {
      iss <- issues[[i]]
      md <- c(md,
        paste0("### ", i, ". ", iss$title %||% ""),
        "",
        iss$desc %||% "",
        "",
        paste0("**Kitendo:** ", iss$action %||% ""),
        "")
    }
  } else {
    md <- c(md, "_Hakuna matatizo._", "")
  }
  md <- c(md, "## 3. Scribe", "", "```", narration, "```", "")
  paste(md, collapse = "\n")
}

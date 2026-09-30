# ============================================================
# mobile.R — FUNDI MOBILE tab: daktari wa simu (agentic, HITL)
#
# Data source: data/mobile/*.json (services, brands, jobs, consents)
# + Rust CLI (fundi-mobile) kwa agentic run / consent add kama ipo.
# Offline-first: bila binary — panel zinazoonyesha data JSON tu.
# ============================================================

`%||%` <- function(a, b) if (is.null(a) || length(a) == 0) b else a

.mobile_data_file <- function(rel) {
  # Rel path: data/mobile/<rel> — tafuta kwenye mizizi ya mradi
  for (r in c("../data", "data", "../../data", file.path(dirname(if (exists("APP_DIR")) APP_DIR else "."), "data"))) {
    f <- file.path(r, rel)
    if (file.exists(f)) return(normalizePath(f, winslash = "/", mustWork = FALSE))
  }
  NULL
}

.mobile_read <- function(rel, default = list()) {
  f <- .mobile_data_file(rel)
  if (is.null(f)) return(default)
  tryCatch(jsonlite::fromJSON(f, simplifyVector = FALSE), error = function(e) default)
}

.mobile_binaries <- function() {
  # Binary ya fundi-mobile (cands + APP_DIR-relative)
  base <- if (exists("APP_DIR")) dirname(APP_DIR) else ".."
  cands <- c(
    file.path(base, "fundi-mobile", "target", "release", "fundi-mobile.exe"),
    file.path(base, "fundi-mobile", "target", "release", "fundi-mobile"),
    "../fundi-mobile/target/release/fundi-mobile.exe",
    "../fundi-mobile/target/release/fundi-mobile",
    "fundi-mobile/target/release/fundi-mobile.exe",
    "fundi-mobile/target/release/fundi-mobile"
  )
  Filter(file.exists, cands)
}

.mobile_binary <- function() {
  .mobile_binaries()
}

.mobile_run <- function(args) {
  bin <- .mobile_binary()
  if (!length(bin)) return(NULL)
  out <- tryCatch(
    suppressWarnings(system2(bin[1], args, stdout = TRUE, stderr = FALSE, timeout = 60)),
    error = function(e) NULL
  )
  if (is.null(out) || !length(out)) return(NULL)
  out
}

# PDF fallback ya R (bila binary): PDF 1.4 halisi + logo ya FUNDI (vector)
mobile_b2b_pdf_fallback <- function(doc) {
  if (is.null(doc)) {
    doc <- list(id = "QT-?", kind = "quote", fundi = "Fundi", company = "Mteja",
                company_contact = "", site = NULL, items = list(), total = 0,
                currency = "TZS", status = "sent", ref_quote = NULL,
                note_sw = NULL, created_ts = as.integer(Sys.time()))
  }
  esc <- function(s) gsub("([()\\\\])", "\\\\\\1", as.character(s))
  money <- function(n) formatC(as.numeric(n), format = "d", big.mark = ",")
  title <- if (identical(doc$kind, "invoice")) "INVOICE" else "QUOTE (OLE BEI)"
  items <- doc$items %||% list()
  rows <- character(0); y <- 600
  for (i in seq_along(items)) {
    it <- items[[i]]
    rows <- c(rows, sprintf(
      "0.1 0.16 0.2 rg BT /F1 9.5 Tf 48 %s Td (%s) Tj ET 48 %.0f Td (%s) Tj ET 410 %s Td (%s) Tj ET 490 %s Td (%s) Tj ET\n",
      y, esc(substr(it$desc %||% "?", 1, 48)), y, esc(it$qty %||% 1), y,
      esc(money(it$price %||% 0)), y, esc(money((it$price %||% 0) * (it$qty %||% 1)))))
    y <- y - 20
  }
  draw <- paste(c(
    "q 44 0 0 44 40 762 cm /LOGO Do Q",
    "0 0.51 0.56 rg BT /F2 20 Tf 96 788 Td (FUNDI) Tj ET",
    "0.33 0.43 0.47 rg BT /F1 8.5 Tf 96 772 Td (MTAALAMU SMART - Mfumo wa mtaalamu) Tj ET",
    sprintf("0.04 0.13 0.15 rg BT /F2 16 Tf 400 788 Td (%s) Tj ET", title),
    sprintf("0.33 0.43 0.47 rg BT /F1 11 Tf 400 772 Td (%s) Tj ET", esc(doc$id %||% "?")),
    "0 0.51 0.56 rg 40 756 515 2.5 re f",
    sprintf("0.04 0.13 0.15 rg BT /F2 10 Tf 40 726 Td (Fundi:) Tj ET 150 726 Td (%s) Tj ET", esc(doc$fundi %||% "?")),
    sprintf("0.04 0.13 0.15 rg BT /F2 10 Tf 40 710 Td (Mteja:) Tj ET 150 710 Td (%s) Tj ET", esc(doc$company %||% "?")),
    "0.88 0.97 0.98 rg 40 594 515 22 re f",
    "0 0.29 0.33 rg BT /F2 10 Tf 48 600 Td (Kazi / Vifaa) Tj ET BT /F2 10 Tf 410 600 Td (Bei) Tj ET BT /F2 10 Tf 490 600 Td (Jumla) Tj ET",
    rows,
    sprintf("0 0.51 0.56 rg 330 %.0f 225 30 re f", y - 8),
    sprintf("1 1 1 rg BT /F2 13 Tf 342 %.0f Td (JUMLA: %s %s) Tj ET", y + 2, esc(doc$currency %||% "TZS"), esc(money(doc$total %||% 0))),
    sprintf("0.04 0.13 0.15 rg BT /F2 10 Tf 40 %.0f Td (Hali: %s) Tj ET", y - 34, esc(toupper(doc$status %||% ""))),
    "0.78 0.86 0.87 RG 0.8 w 40 70 m 555 70 l S",
    "0.42 0.47 0.5 rg BT /F1 8 Tf 40 56 Td (Malipo yanapokelewa na mtoa huduma. Hati imetengenezwa na FUNDI MOBILE.) Tj ET"
  ), collapse = "\n")
  objs <- c(
    "<< /Type /Catalog /Pages 2 0 R >>",
    "<< /Type /Pages /Kids [3 0 R] /Count 1 >>",
    "<< /Type /Page /Parent 2 0 R /MediaBox [0 0 595 842] /Resources << /Font << /F1 4 0 R /F2 5 0 R >> /XObject << /LOGO 6 0 R >> >> /Contents 3 0 R >>",
    sprintf("<< /Length %d >>\nstream\n%s\nendstream", nchar(draw), draw),
    "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica /Encoding /WinAnsiEncoding >>",
    "<< /Type /Font /Subtype /Type1 /BaseFont /Helvetica-Bold /Encoding /WinAnsiEncoding >>",
    sprintf("<< /Type /XObject /Subtype /Form /BBox [0 0 100 100] /Resources << >> /Length 156 >>\nstream\n0 0.898 1 rg 50 4 m 50 96 l 4 50 l h f\n0 0.51 0.56 rg 50 96 m 50 4 l 96 50 l h f\n1 1 1 rg 28 30 11 52 re f 28 71 32 11 re f 28 48 25 11 re f\nendstream")
  )
  out <- charToRaw(paste0("%PDF-1.4\n"))
  positions <- integer(0)
  buf <- rawToChar(out)
  for (i in seq_along(objs)) {
    positions[i] <- nchar(buf, type = "bytes")
    buf <- paste0(buf, i, " 0 obj\n", objs[i], "\nendobj\n")
  }
  info_pos <- nchar(buf, type = "bytes")
  info_n <- length(objs) + 1
  buf <- paste0(buf, info_n, " 0 obj\n<< /Title (Hati ya FUNDI) /Producer (FUNDI MOBILE R) >>\nendobj\n")
  xref_pos <- nchar(buf, type = "bytes")
  xr <- c("xref", paste0("0 ", info_n + 1), "0000000000 65535 f ",
          sprintf("%010d 00000 n ", positions), sprintf("%010d 00000 n ", info_pos),
          paste0("trailer", "\n", sprintf("<< /Size %d /Root 1 0 R /Info %d 0 R >>", info_n + 1, info_n),
                 "\nstartxref", "\n", xref_pos, "\n%%EOF"))
  paste0(buf, paste(xr, collapse = "\n"), "\n")
}

.mobile_badge <- function(status) {
  cls <- switch(status %||% "",
    done = "GOOD", running = "INFO", awaiting_consent = "WARNING",
    failed = "CRITICAL", "INFO")
  as.character(tags$span(class = paste("badge", cls), status))
}

# ---------- UI ----------

mobile_tab_el <- function(lang) {
  sw <- function() identical(lang, "sw")  # UI: closure ya lang (server ina reactive yake)
  panel_el(
    title = "\U0001fa7a FUNDI MOBILE",
    meta = "Android · iPhone · BUTTON · agentic · HITL consent",
    tags$div(
      # MWANZO HAPA (onboarding) — kila mtu ajue anafanyaje
      tags$div(class = "panel", style = "background:linear-gradient(135deg,#0d2137,#050d18);border:1px solid #00e5ff33;border-radius:10px;padding:14px;margin-bottom:12px",
        tags$h3(style = "color:var(--cyan);margin:0 0 6px", "\u2b50 MWANZO HAPA — jinsi mfumo unavyofanya kazi"),
        tags$div(style = "font-size:12px;color:#b2ebf2",
          if (sw()) HTML(paste0(
            "Wewe ni <b>FUNDI/mtaalamu</b>: andika taarifa za mteja → chagua huduma → bonyeza <b>▶ Endesha agent</b>. ",
            "Agent (AI) ndiye anayefanya kazi: anachanganua simu (adb HALISI), anatoa hatua, na kwa kazi hatari anasubiri <b>RUHUSU yako (HITL)</b> kabla ya kuendelea. ",
            "Matokeo yote huja kwenye mistari ya chini ya kila panel."
          )) else HTML(paste0(
            "You are the <b>technician</b>: enter customer details → pick a service → press <b>▶ Run agent</b>. ",
            "The AI agent does the work: scans the phone (real adb), plans steps, and asks for your <b>APPROVAL (HITL)</b> before anything risky."
          ))),
        tags$div(style = "display:grid;grid-template-columns:repeat(auto-fit,minmax(190px,1fr));gap:8px;margin-top:10px;font-size:12px",
          tags$div(style = "background:#0a1628;border:1px solid #00e5ff33;border-radius:8px;padding:8px",
            tags$b(style = "color:var(--cyan)", "1. Simu"),
            tags$div("Unganisha simu (USB) → 'Simu zilizounganishwa' → endesha agent kwenye ACCESS RECOVERY")),
          tags$div(style = "background:#0a1628;border:1px solid #00e5ff33;border-radius:8px;padding:8px",
            tags$b(style = "color:var(--cyan)", "2. PC"),
            tags$div("EMAIL AGENTIC (setup/password) · ACTIVATION rasmi · DRIVERS · MLINZI (utapeli) · DATA RECOVERY · SCRIBE (mikutano)")),
          tags$div(style = "background:#0a1628;border:1px solid #00e5ff33;border-radius:8px;padding:8px",
            tags$b(style = "color:var(--cyan)", "3. Serikali/Jamii"),
            tags$div("GOV AGENTIC: agent inajaza fomu (NIDA, TRA...) — wewe JIBU maswali tu; sauti inasomesha")),
          tags$div(style = "background:#0a1628;border:1px solid #00e5ff33;border-radius:8px;padding:8px",
            tags$b(style = "color:var(--cyan)", "4. Kanuni"),
            tags$div("Hakuna bypass/exploit — njia RASMI tu; consent ya mmiliki LAZIMA kwa kila kazi hatari")))),
      # Buttons
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;margin-bottom:10px",
        tags$button(class = "btn",
          onclick = "Shiny.setInputValue('mob_refresh', Date.now(), {priority:'event'})",
          if (sw()) "\U0001f504 Onyesha yote" else "\U0001f504 Refresh all"),
        tags$button(class = "btn",
          onclick = "Shiny.setInputValue('mob_devices', Date.now(), {priority:'event'})",
          if (sw()) "\U0001f4f1 Simu zilizounganishwa" else "\U0001f4f1 Connected devices"),
        tags$button(class = "btn sec",
          onclick = "Shiny.setInputValue('mob_consent_form', Date.now(), {priority:'event'})",
          if (sw()) "\U0001f4dd Fomu ya consent" else "\U0001f4dd Consent form")),

      # AGENTIC RUN
      tags$div(class = "panel", style = "background:#050d18;padding:12px;border-radius:8px;margin-bottom:12px",
        tags$h3(style = "color:var(--cyan);margin:0 0 8px", "\U0001f916 AGENTIC RUN — agent inafanya kazi"),
        tags$div(style = "display:flex;gap:8px;flex-wrap:wrap",
          tags$input(id = "mob_customer", placeholder = "Jina la mteja", style = "flex:1;min-width:130px"),
          tags$input(id = "mob_phone", placeholder = "Simu ya mteja", style = "flex:1;min-width:120px"),
          tags$input(id = "mob_brand", placeholder = "Brand (samsung/nokia/...)", style = "flex:1;min-width:130px"),
          tags$input(id = "mob_model", placeholder = "Model (A12/3310/...)", style = "flex:1;min-width:110px"),
          tags$input(id = "mob_imei", placeholder = "IMEI (namba 15)", style = "flex:1;min-width:150px"),
          tags$input(id = "mob_problem", placeholder = "Tatizo: nimesahau password", style = "flex:2;min-width:180px")),
        tags$div(style = "display:flex;gap:8px;margin-top:8px;align-items:center",
          selectInput("mob_service", NULL, choices = character(0), width = "340px"),
          tags$button(class = "btn",
            onclick = "Shiny.setInputValue('mob_run', Date.now(), {priority:'event'})",
            "\u25b6 Endesha agent")),
        tags$div(style = "font-size:11px;color:var(--dim);margin-top:6px",
          "Agent itasimamisha kazi bila consent (HITL) — hilo ni ulinzi, si makosa.")),

      # CONSENT ADD
      tags$div(class = "panel", style = "background:#050d18;padding:12px;border-radius:8px;margin-bottom:12px",
        tags$h3(style = "color:var(--cyan);margin:0 0 8px", "\u2696 CONSENT (RUHUSA YA KAZI)"),
        tags$div(style = "display:flex;gap:8px;flex-wrap:wrap",
          tags$input(id = "mc_name", placeholder = "Jina (kama kwenye fomu)", style = "flex:1;min-width:140px"),
          tags$input(id = "mc_phone", placeholder = "Simu", style = "flex:1;min-width:110px"),
          tags$input(id = "mc_imei", placeholder = "IMEI", style = "flex:1;min-width:150px"),
          tags$input(id = "mc_brand", placeholder = "Brand", style = "flex:1;min-width:100px"),
          tags$input(id = "mc_model", placeholder = "Model", style = "flex:1;min-width:100px")),
        tags$div(style = "display:flex;gap:8px;margin-top:8px;align-items:center",
          selectInput("mc_service", NULL, choices = character(0), width = "300px"),
          tags$label(style = "font-size:12px;color:var(--dim)",
            tags$input(id = "mc_destroys", type = "checkbox", style = "margin-right:4px"), " data inapotea"),
          tags$button(class = "btn sec",
            onclick = "Shiny.setInputValue('mob_consent_add', Date.now(), {priority:'event'})",
            "\u270d Rekodi consent")),
        tags$div(style = "font-size:11px;color:var(--dim);margin-top:6px",
          "Mteja LAZIMA asaini fomu ya karatasi kwanza; UI inasajili kumbukumbu.")),

      # ACCESS RECOVERY — zana 11 kwa 1 (kwa mmiliki aliyeuthibitishwa)
      tags$div(class = "panel", style = "background:#050d18;padding:12px;border-radius:8px;margin-bottom:12px",
        tags$h3(style = "color:var(--cyan);margin:0 0 8px",
          if (sw()) "\U0001f513 ACCESS RECOVERY — ZANA 11 KWA 1 (mmiliki aliyeuthibitishwa)"
          else "\U0001f513 ACCESS RECOVERY — 11-IN-1 TOOLS (verified owner)"),
        tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center",
          tags$input(id = "recover_imei", placeholder = "IMEI (namba 15)", style = "flex:1;min-width:150px"),
          selectInput("recover_tool", NULL,
            choices = c(
              "🔓 Orodha ya zana (list)" = "list",
              "1. Preflight — uchunguzi halisi" = "preflight",
              "2. Ownership — thibitisha umiliki" = "ownership",
              "3. Backup-first — backup kabla ya kazi" = "backup-first",
              "4. Google Find My Device (rasmi)" = "google-remote",
              "5. Samsung Find My Mobile (Android 11+)" = "samsung-remote",
              "6. Mi Cloud — Xiaomi/Redmi/POCO" = "xiaomi-remote",
              "7. HUAWEI Find Device" = "huawei-remote",
              "8. Owner-ADB — zima screen-lock kwa adb" = "owner-adb",
              "9. Recovery reset — combo sahihi ya brand" = "recovery-reset",
              "10. FRP aftermath — mmiliki anaingia account yake" = "frp-aftermath",
              "11. Report — ripoti ya huduma" = "report"),
            width = "300px"),
          tags$button(class = "btn sec",
            onclick = "Shiny.setInputValue('recover_run', Date.now(), {priority:'event'})",
            if (sw()) "\u25b6 Endesha zana" else "\u25b6 Run tool")),
        tags$div(style = "font-size:11px;color:var(--dim);margin-top:6px",
          if (sw()) "Njia zote ni RASMI (hakuna exploit). Consent ya mmiliki LAZIMA kwanza: service android_access_recovery."
          else "All paths are OFFICIAL (no exploits). Owner consent required first: service android_access_recovery."),
        uiOutput("recover_out")),

      uiOutput("mob_devices_out"),
      uiOutput("mob_shield"),
      uiOutput("mob_advanced"),
      uiOutput("mob_problems"),
      uiOutput("mob_gov"),
      uiOutput("mob_email"),
      uiOutput("mob_mlinzi"),
      uiOutput("mob_netagentic"),
      uiOutput("mob_telecom"),
      uiOutput("mob_pro"),
      uiOutput("mob_proagentic_out"),
      uiOutput("mob_summary"),
      uiOutput("mob_revenue"),
      uiOutput("mob_b2b"),
      uiOutput("mob_services"),
      uiOutput("mob_button_codes"),
      uiOutput("mob_brands"),
      uiOutput("mob_jobs"),
      uiOutput("mob_consents"),
      uiOutput("mob_agents"),
      uiOutput("mob_run_out")
    )
  )
}

# Shiny handler: generate + download PDF ya B2B doc
mobile_pdf_output <- function(id) downloadHandler(
  filename = function() paste0(id, ".pdf"),
  content = function(file) {
    bin <- .mobile_binaries()
    f <- .mobile_data_file("mobile/b2b_docs.json")
    doc_id <- sub("^mob_pdf_", "", id)
    ok <- FALSE
    if (length(bin) && !is.null(f)) {
      out <- tryCatch(
        suppressWarnings(
          system2(bin[1], c("b2b", "pdf", "--id", doc_id, "--out", shQuote(file)),
                  stdout = TRUE, stderr = TRUE, timeout = 30)),
        error = function(e) NULL)
      ok <- !is.null(out) && file.exists(file) && file.info(file)$size > 200
    }
    if (!ok) {
      # Fallback: tengeneza PDF kwa R (logo vector + jedwali) — bila binary
      docs <- tryCatch(jsonlite::fromJSON(f, simplifyVector = FALSE), error = function(e) list())
      doc <- NULL
      for (d in docs) if (identical(d$id, doc_id)) doc <- d
      writeLines(mobile_b2b_pdf_fallback(doc), file, useBytes = TRUE)
    }
  },
  contentType = "application/pdf"
)

# ---------- Server ----------

mobile_server <- function(input, output, session) {
  # Lugha ya UI: reactive — input$lang (app moja) → default sw
  sw_rv <- reactiveVal(TRUE)
  observe({
    lg <- tryCatch(input$lang %||% getShinyOption("fundi_lang", default = if (exists("DEFAULT_LANG")) DEFAULT_LANG else "sw"),
                   error = function(e) "sw")
    sw_rv(isTRUE(identical(lg, "sw")))
  })
  sw <- reactive(sw_rv())
  services_rv <- reactiveVal(list())
  brands_rv   <- reactiveVal(list())
  jobs_rv     <- reactiveVal(list())
  consents_rv <- reactiveVal(list())

  .refresh_all <- function() {
    services_rv(.mobile_read("mobile/services.json"))
    brands_rv(.mobile_read("mobile/brands.json"))
    jobs_rv(.mobile_read("mobile/jobs.json"))
    consents_rv(.mobile_read("mobile/consents.json"))
  }

  observe({
    invalidateLater(15000, session)
    jobs_rv(.mobile_read("mobile/jobs.json"))
    consents_rv(.mobile_read("mobile/consents.json"))
  })

  # service pickers (zote kutoka services.json)
  observe({
    s <- services_rv()
    svcs <- s$services %||% list()
    if (!length(svcs)) return()
    ids <- vapply(svcs, function(x) x$id %||% "", character(1))
    labels <- vapply(svcs, function(x) {
      paste0(x$name_sw %||% x$id, " — TZS ", format(x$price %||% 0, big.mark = ","))
    }, character(1))
    names(ids) <- labels
    updateSelectInput(session, "mob_service", choices = ids, selected = ids[1])
    updateSelectInput(session, "mc_service", choices = ids, selected = ids[1])
  })

  observeEvent(input$mob_refresh, .refresh_all())
  .refresh_all()

  observeEvent(input$mob_devices, {
    out <- .mobile_run(c("devices"))
    output$mob_devices_out <- renderUI(
      tags$div(class = "panel", style = "background:#050d18;padding:10px;border-radius:8px",
        tags$h3(style = "color:var(--cyan);margin:0 0 6px", "\U0001f4f1 DEVICES"),
        if (is.null(out)) tags$div(class = "meta",
          "fundi-mobile binary haipo — build: cd fundi-mobile && cargo build --release")
        else tags$pre(style = "color:#b2ebf2;font-size:12px;margin:0", paste(out, collapse = "\n"))))
  })

  observeEvent(input$mob_consent_form, {
    out <- .mobile_run(c("consent", "form"))
    form <- if (!is.null(out)) paste(out, collapse = "\n") else
      paste(readLines(.mobile_data_file("../../fundi-mobile/CONSENT_FORM.txt"), warn = FALSE), collapse = "\n")
    output$mob_devices_out <- renderUI(
      tags$div(class = "panel", style = "background:#050d18;padding:10px;border-radius:8px",
        tags$h3(style = "color:var(--cyan);margin:0 0 6px", "\u2696 FOMU YA CONSENT"),
        tags$pre(style = "color:#e0f7fa;font-size:11px;margin:0;white-space:pre-wrap", form)))
  })

  observeEvent(input$mob_consent_add, {
    bin <- .mobile_binary()
    if (!length(bin)) {
      output$mob_run_out <- renderUI(tags$div(class = "status-banner CRITICAL",
        "fundi-mobile binary haipo — build kwanza (cargo build --release)"))
      return()
    }
    args <- c("consent", "add",
      "--name", shQuote(input$mc_name %||% ""),
      "--phone", shQuote(input$mc_phone %||% ""),
      "--imei", shQuote(input$mc_imei %||% ""),
      "--brand", shQuote(input$mc_brand %||% ""),
      "--model", shQuote(input$mc_model %||% ""),
      "--service", shQuote(input$mc_service %||% ""))
    if (isTRUE(input$mc_destroys)) args <- c(args, "--destroys")
    out <- .mobile_run(args)
    consents_rv(.mobile_read("mobile/consents.json"))
    output$mob_run_out <- renderUI(
      if (is.null(out)) tags$div(class = "status-banner CRITICAL", "Consent imeshindikana (angalia IMEI/jina)")
      else tags$div(class = "status-banner GOOD", paste(out, collapse = " ")))
  })

  # ---- FUNDI SHIELD: antivirus + audit halisi ----
  .shield_pre <- function(out, label) {
    if (is.null(out)) {
      tags$div(class = "status-banner WARNING", paste0(label, " imeshindikana — kama binary haipo: cd fundi-mobile && cargo build --release"))
    } else {
      tags$pre(style = "color:#b2ebf2;font-size:12px;background:#050d18;padding:10px;border-radius:8px;margin:8px 0 0;white-space:pre-wrap;max-height:340px;overflow:auto",
        paste(out, collapse = "\n"))
    }
  }
  output$mob_shield <- renderUI({
    tags$div(class = "panel", style = "background:#050d18;padding:12px;border-radius:8px;margin:12px 0",
      tags$h3(style = "color:var(--cyan);margin:0 0 8px", "\U0001f6e1 FUNDI SHIELD — antivirus yako (scan halisi)"),
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap",
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('shield_audit', Date.now(), {priority:'event'})", "\U0001f50d Audit usalama"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('shield_scan', Date.now(), {priority:'event'})", "\U0001f9f2 Scan (EICAR test)"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('shield_report', Date.now(), {priority:'event'})", "\U0001f4c4 Ripoti")),
      tags$div(style = "display:flex;gap:8px;margin-top:8px;align-items:center",
        tags$input(id = "shield_device", placeholder = "Device ID (kwa CLEAN — HITL consent)", style = "flex:1;min-width:180px"),
        tags$button(class = "btn", style = "background:#7a2048;border-color:#7a2048",
          onclick = "Shiny.setInputValue('shield_clean', Date.now(), {priority:'event'})", "\U0001f9f9 Clean temp (HITL)")),
      tags$div(style = "font-size:11px;color:var(--dim);margin-top:6px",
        if (sw()) "Audit na scan ni HALISI. CLEAN inahitaji consent (shield_clean) — kazi ya kufuta ni hatari."
        else "Audit & scan are REAL. CLEAN needs consent (shield_clean) — deletion is a destructive action."),
      uiOutput("shield_out"))
  })

  shield_run <- function(args, timeout = 120) {
    bin <- .mobile_binary()
    if (!length(bin)) return(NULL)
    tryCatch(suppressWarnings(system2(bin[1], args, stdout = TRUE, stderr = FALSE, timeout = timeout)),
      error = function(e) NULL)
  }

  observeEvent(input$shield_audit, {
    out <- shield_run(c("shield", "audit"))
    output$shield_out <- renderUI(.shield_pre(out, "Audit"))
  })
  observeEvent(input$shield_scan, {
    out <- shield_run(c("shield", "scan", "--eicar"), 180)
    output$shield_out <- renderUI(.shield_pre(out, "Scan"))
  })
  observeEvent(input$shield_report, {
    out <- shield_run(c("shield", "report", "--out", shQuote("shield_report.html")))
    output$shield_out <- renderUI(.shield_pre(out, "Ripoti"))
  })
  observeEvent(input$shield_clean, {
    dev <- trimws(input$shield_device %||% "")
    if (!nzchar(dev)) {
      output$shield_out <- renderUI(tags$div(class = "status-banner WARNING",
        "Andika Device ID kwanza + hakikisha consent ipo: fundi-mobile consent add --service shield_clean --imei <id> ..."))
      return()
    }
    out <- shield_run(c("shield", "clean", "--device", shQuote(dev)))
    output$shield_out <- renderUI(.shield_pre(out, "Clean"))
  })

  # ---- GOV AGENTIC: AI inajaza fomu za serikali + jamii + sauti ----
  gov_data <- .mobile_read("mobile/gov_systems.json")
  gov_systems_list <- gov_data$systems %||% list()
  gov_choices <- setNames(
    vapply(gov_systems_list, function(s) paste(s$name %||% s$id, "— TZS", format(s$fee_tzs %||% 0, big.mark = ",")), character(1)),
    vapply(gov_systems_list, function(s) s$id %||% "", character(1)))
  jamii_data <- .mobile_read("mobile/community_problems.json")
  jamii_cats <- jamii_data$categories %||% list()
  jamii_choices <- setNames(
    vapply(jamii_cats, function(c) paste(c$icon %||% "\u2022", c$name_sw %||% c$id), character(1)),
    vapply(jamii_cats, function(c) c$id %||% "", character(1)))

  output$mob_gov <- renderUI({
    tags$div(class = "panel", style = "background:#050d18;padding:12px;border-radius:8px;margin:12px 0",
      tags$h3(style = "color:var(--cyan);margin:0 0 8px",
        if (sw()) "\U0001f3db GOV AGENTIC — AI inajaza fomu za serikali (wewe JIBU tu)"
        else "\U0001f3db GOV AGENTIC — AI fills government forms for you"),
      # FOMU: chagua mfumo → fields dynamic → fill (TTS inasema mwelekeo)
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center",
        selectInput("gov_system", NULL, choices = gov_choices, width = "300px"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('gov_load', Date.now(), {priority:'event'})", "\U0001f4cb Fungua fields"),
        tags$button(class = "btn", onclick = "Shiny.setInputValue('gov_fill', Date.now(), {priority:'event'})", "\U0001f916 Jaza + sauti"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('gov_speak', Date.now(), {priority:'event'})", "\U0001f50a Zungumza")),
      uiOutput("gov_fields"),
      # SEARCH (SearXNG)
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center;margin-top:8px",
        tags$input(id = "gov_q", placeholder = "Tafuta: BRELA fees 2026 (SearXNG)", style = "flex:1;min-width:180px"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('gov_search', Date.now(), {priority:'event'})", "\U0001f50e Tafuta")),
      # JAMII
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center;margin-top:8px",
        selectInput("jamii_cat", NULL, choices = jamii_choices, width = "260px"),
        tags$input(id = "jamii_n", placeholder = "Namba (1-3)", style = "width:90px"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('jamii_show', Date.now(), {priority:'event'})", "\U0001f91d Saidia (steps + sauti)")),
      uiOutput("gov_out"))
  })

  gov_run <- function(args, timeout = 60) {
    bin <- .mobile_binary()
    if (!length(bin)) return(NULL)
    tryCatch(suppressWarnings(system2(bin[1], args, stdout = TRUE, stderr = FALSE, timeout = timeout)),
      error = function(e) NULL)
  }

  # Fields za mfumo uliochaguliwa (dynamic kutoka JSON)
  gov_fields_rv <- reactiveVal(list())
  observeEvent(input$gov_load, {
    sid <- input$gov_system %||% "nida"
    s <- Filter(function(x) identical(x$id, sid), gov_systems_list)
    if (!length(s)) return()
    gov_fields_rv(s[[1]]$fields %||% list())
  })
  output$gov_fields <- renderUI({
    fl <- gov_fields_rv()
    if (!length(fl)) return(tags$div(class = "meta", "Chagua mfumo kisha 'Fungua fields' — agent itauliza yale yanayokosekana pekee."))
    lapply(fl, function(f) {
      req <- isTRUE(f$required %||% FALSE)
      opts <- f$options %||% NULL
      lab <- tags$label(style = "font-size:11px;color:var(--dim);display:block;margin-top:6px",
        paste0(f$label_sw %||% f$id, if (req) " *" else ""))
      inp <- if (!is.null(opts)) {
        selectInput(paste0("gf_", f$id), NULL, choices = as.character(opts), width = "100%")
      } else {
        tags$input(id = paste0("gf_", f$id), placeholder = f$label_sw %||% "", style = "width:100%")
      }
      tags$div(lab, inp)
    })
  })

  observeEvent(input$gov_fill, {
    sid <- input$gov_system %||% "nida"
    fl <- gov_fields_rv()
    pairs <- vapply(fl, function(f) {
      fid <- f$id %||% ""
      v <- input[[paste0("gf_", fid)]] %||% ""
      if (nzchar(v)) paste0(fid, "=", v) else NULL
    }, character(1))
    pairs <- Filter(Negate(is.null), pairs)
    args <- c("gov", "fill", "--system", sid)
    if (length(pairs)) args <- c(args, "--answers", shQuote(paste(pairs, collapse = ";")))
    out <- gov_run(args)
    output$gov_out <- renderUI(.shield_pre(out, "GOV fill"))
  })
  observeEvent(input$gov_speak, {
    t <- trimws(input$gov_q %||% "")
    out <- if (nzchar(t)) gov_run(c("gov", "speak", shQuote(t))) else gov_run(c("gov", "speak", shQuote("Fundi agent hapa, nikusaidie nini?")))
    output$gov_out <- renderUI(.shield_pre(out, "Sauti"))
  })
  observeEvent(input$gov_search, {
    q <- trimws(input$gov_q %||% "")
    out <- if (nzchar(q)) gov_run(c("gov", "search", shQuote(q))) else NULL
    output$gov_out <- renderUI(.shield_pre(out, "SearXNG"))
  })
  observeEvent(input$jamii_show, {
    cat <- input$jamii_cat %||% ""
    n <- suppressWarnings(as.integer(input$jamii_n %||% "1"))
    out <- if (nzchar(cat) && !is.na(n)) gov_run(c("gov", "jamii", cat, as.character(n))) else NULL
    output$gov_out <- renderUI(.shield_pre(out, "Jamii"))
  })

  # ---- EMAIL AGENTIC + ACTIVATION (HALALI) + DRIVERS ----
  output$mob_email <- renderUI({
    # Providers kutoka CLI (email json) — fallback static kama binary haipo
    provs <- tryCatch({
      bin <- .mobile_binary()
      if (length(bin)) {
        j <- suppressWarnings(system2(bin[1], c("email", "json"), stdout = TRUE, stderr = FALSE, timeout = 15))
        v <- jsonlite::fromJSON(paste(j, collapse = "\n"), simplifyVector = FALSE)
        setNames(vapply(v, function(p) paste(p$name %||% p$id, "\u2014 IMAP", p$imap %||% ""), character(1)),
                 vapply(v, function(p) p$id %||% "", character(1)))
      } else NULL
    }, error = function(e) NULL)
    if (is.null(provs) || !length(provs))
      provs <- c("Gmail (Google)" = "gmail", "Outlook / Hotmail (Microsoft)" = "outlook",
                 "Yahoo Mail" = "yahoo", "Zoho Mail" = "zoho", "iCloud Mail (Apple)" = "icloud",
                 "Custom Domain (cPanel)" = "custom")
    tags$div(class = "panel", style = "background:#050d18;padding:12px;border-radius:8px;margin:12px 0",
      tags$h3(style = "color:var(--cyan);margin:0 0 8px",
        if (sw()) "\U0001f4e7 EMAIL AGENTIC — providers WOTE + setup + password + matatizo"
        else "\U0001f4e7 EMAIL AGENTIC — all providers + setup + password + problems"),
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center",
        selectInput("email_prov", NULL, choices = provs, width = "280px"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('email_check', Date.now(), {priority:'event'})", "\u26a1 Check (TCP halisi)"),
        tags$button(class = "btn", onclick = "Shiny.setInputValue('email_setup', Date.now(), {priority:'event'})", "\U0001f916 Setup + sauti"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('email_pass', Date.now(), {priority:'event'})", "\U0001f511 Password/reset"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('email_probs', Date.now(), {priority:'event'})", "\U0001fa79 Matatizo yote")),
      tags$h3(style = "color:var(--cyan);margin:14px 0 8px",
        if (sw()) "\U0001f513 ACTIVATION (HALALI — slmgr/OSPP rasmi; hakuna cracks)"
        else "\U0001f513 ACTIVATION (official slmgr/OSPP only; no cracks)"),
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center",
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('lic_status', Date.now(), {priority:'event'})", "Windows status"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('lic_office', Date.now(), {priority:'event'})", "Office status"),
        tags$button(class = "btn", onclick = "Shiny.setInputValue('lic_activate', Date.now(), {priority:'event'})", "\u25b6 Activate Windows (rasmi)"),
        tags$button(class = "btn", onclick = "Shiny.setInputValue('lic_actoffice', Date.now(), {priority:'event'})", "\u25b6 Activate Office (rasmi)"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('lic_genuine', Date.now(), {priority:'event'})", "\U0001f6cd Nunua genuine")),
      tags$div(style = "font-size:11px;color:var(--dim);margin-top:6px",
        if (sw()) "massgrave/MAS/KMS-cracks ni PIRACY — hazitumiki. Activation ni ya rasmi tu: slmgr /ato na OSPP /act (admin). Bila leseni? nurudia kitufe cha genuine."
        else "massgrave/MAS/KMS-cracks = piracy, not used. Official activation only: slmgr /ato + OSPP /act (admin). No license? Use the genuine button."),
      tags$h3(style = "color:var(--cyan);margin:14px 0 8px",
        if (sw()) "\U0001f527 DRIVERS — enum halisi + scan + update rasmi"
        else "\U0001f527 DRIVERS — real enum + scan + official update"),
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center",
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('drv_list', Date.now(), {priority:'event'})", "Orodha (pnputil)"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('drv_scan', Date.now(), {priority:'event'})", "\U0001f50d Scan devices"),
        tags$button(class = "btn", onclick = "Shiny.setInputValue('drv_update', Date.now(), {priority:'event'})", "Mwongozo wa update")),
      uiOutput("sys_out"))
  })

  sys_run <- function(args, timeout = 90) gov_run(args, timeout)
  observeEvent(input$email_check, output$sys_out <- renderUI(
    .shield_pre(sys_run(c("email", "check", input$email_prov %||% "gmail")), "Email check (TCP halisi)")))
  observeEvent(input$email_setup, output$sys_out <- renderUI(
    .shield_pre(sys_run(c("email", "setup", input$email_prov %||% "gmail")), "Setup agentic + sauti")))
  observeEvent(input$email_pass, output$sys_out <- renderUI(
    .shield_pre(sys_run(c("email", "password", input$email_prov %||% "gmail")), "Password + reset rasmi")))
  observeEvent(input$email_probs, output$sys_out <- renderUI(
    .shield_pre(sys_run(c("email", "problems")), "Matatizo yote ya email")))
  observeEvent(input$lic_status, output$sys_out <- renderUI(
    .shield_pre(sys_run(c("license", "status")), "Windows activation status")))
  observeEvent(input$lic_office, output$sys_out <- renderUI(
    .shield_pre(sys_run(c("license", "office-status")), "Office activation status")))
  observeEvent(input$lic_activate, output$sys_out <- renderUI(
    .shield_pre(sys_run(c("license", "activate"), 120), "Activate Windows (slmgr /ato — rasmi)")))
  observeEvent(input$lic_actoffice, output$sys_out <- renderUI(
    .shield_pre(sys_run(c("license", "activate-office"), 120), "Activate Office (OSPP /act — rasmi)")))
  observeEvent(input$lic_genuine, output$sys_out <- renderUI(
    .shield_pre(sys_run(c("license", "genuine")), "Leseni GENUINE — mwongozo")))
  observeEvent(input$drv_list, output$sys_out <- renderUI(
    .shield_pre(sys_run(c("drivers", "list")), "Drivers (pnputil halisi)")))
  observeEvent(input$drv_scan, output$sys_out <- renderUI(
    .shield_pre(sys_run(c("drivers", "scan"), 120), "Scan devices (pnputil)")))
  observeEvent(input$drv_update, output$sys_out <- renderUI(
    .shield_pre(sys_run(c("drivers", "update")), "Driver update — njia rasmi")))

  # ---- MLINZI (kinga) + DATA RECOVERY + SCRIBE ----
  output$mob_mlinzi <- renderUI({
    # Masomo ya academy kutoka CLI (mlinzi json) — fallback tuli
    lessons <- tryCatch({
      bin <- .mobile_binary()
      if (length(bin)) {
        j <- suppressWarnings(system2(bin[1], c("mlinzi", "json"), stdout = TRUE, stderr = FALSE, timeout = 15))
        v <- jsonlite::fromJSON(paste(j, collapse = "\n"), simplifyVector = FALSE)
        setNames(vapply(v$lessons, function(l) paste0(l$n, ". ", l$title), character(1)),
                 vapply(v$lessons, function(l) as.character(l$n), character(1)))
      } else NULL
    }, error = function(e) NULL)
    if (is.null(lessons) || !length(lessons))
      lessons <- c("1. Call forwarding za wizi" = "1", "2. OTP ni siri" = "2", "3. SIM swap" = "3", "4. M-Pesa false deposit" = "4")
    tags$div(class = "panel", style = "background:#050d18;padding:12px;border-radius:8px;margin:12px 0",
      tags$h3(style = "color:#ff5252;margin:0 0 8px",
        if (sw()) "\U0001f6e1 MLINZI — kinga: utapeli/wizi/phishing (SMS · links · emails · simu)"
        else "\U0001f6e1 GUARD — protection: scams/theft/phishing"),
      # FORWARD GUARD
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center",
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('mlz_fwd', Date.now(), {priority:'event'})", "\U0001f4de Forwarding codes"),
        tags$button(class = "btn", onclick = "Shiny.setInputValue('mlz_fwdstop', Date.now(), {priority:'event'})", "\u26d4 ZIMA forwarding (adb ##002#)"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('mlz_status', Date.now(), {priority:'event'})", "Hali ya usalama")),
      # SMS + LINK
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center;margin-top:8px",
        tags$input(id = "mlz_sms", placeholder = "Bandika ujumbe wa SMS hapa...", style = "flex:2;min-width:170px"),
        tags$button(class = "btn", onclick = "Shiny.setInputValue('mlz_smsshield', Date.now(), {priority:'event'})", "\U0001f4f1 SMS SHIELD + sauti"),
        tags$input(id = "mlz_link", placeholder = "https://link...", style = "flex:1;min-width:150px"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('mlz_linkcheck', Date.now(), {priority:'event'})", "\U0001f517 Kagua link")),
      # ACADEMY
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center;margin-top:8px",
        selectInput("mlz_lesson", NULL, choices = lessons, width = "300px"),
        tags$button(class = "btn", onclick = "Shiny.setInputValue('mlz_academy', Date.now(), {priority:'event'})", "\U0001f393 Soma + SAUTI"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('mlz_email', Date.now(), {priority:'event'})", "\u2709 Phishing email")),
      # DATA RECOVERY
      tags$h3(style = "color:#ff5252;margin:14px 0 8px",
        if (sw()) "\U0001f4be DATA RECOVERY — deep-scan + kurudisha data"
        else "\U0001f4be DATA RECOVERY — deep-scan + restore"),
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center",
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('dr_scan', Date.now(), {priority:'event'})", "\U0001f50d Deep-scan disks"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('dr_plan', Date.now(), {priority:'event'})", "\U0001f5fa Mpango"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('dr_bin', Date.now(), {priority:'event'})", "\U0001f5d1 Recycle Bin"),
        tags$button(class = "btn", onclick = "Shiny.setInputValue('dr_restore', Date.now(), {priority:'event'})", "\u21a9 Rudisha zote"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('dr_photo', Date.now(), {priority:'event'})", "\U0001f4f8 PhotoRec"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('dr_phone', Date.now(), {priority:'event'})", "\U0001f4f1 Simu (adb/cloud)")),
      # SCRIBE
      tags$h3(style = "color:#ff5252;margin:14px 0 8px",
        if (sw()) "\U0001f4dd AI SCRIBE — mikutano/madarasa/hospitali: inasikiliza · inaandika · inapanga · inakamilisha"
        else "\U0001f4dd AI SCRIBE — meetings/classes/clinic"),
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center",
        selectInput("scr_type", NULL, choices = c("Mkutano (kampuni)" = "meeting", "Darasa (wanafunzi)" = "class",
          "Hospitali/kliniki" = "hospital", "Mahojiano" = "interview", "Nyingine" = "custom"), width = "200px"),
        tags$button(class = "btn", onclick = "Shiny.setInputValue('scr_start', Date.now(), {priority:'event'})", "\u25b6 Anza session"),
        tags$input(id = "scr_id", placeholder = "ID ya session (scribe_...)", style = "width:190px"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('scr_sessions', Date.now(), {priority:'event'})", "\U0001f4cb Sessions")),
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center;margin-top:8px",
        tags$input(id = "scr_line", placeholder = "Andika/dikti mstari (au .amua .kazi .somo)", style = "flex:2;min-width:200px"),
        selectInput("scr_kind", NULL, choices = c("note", "decision", "action", "topic"), width = "110px"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('scr_add', Date.now(), {priority:'event'})", "\u2795 Ongeza"),
        tags$button(class = "btn", onclick = "Shiny.setInputValue('scr_finish', Date.now(), {priority:'event'})", "\u2705 Panga + MINUTA")),
      tags$div(style = "font-size:11px;color:var(--dim);margin-top:6px",
        if (sw()) "Scribe inafanya kazi na makampuni, wanafunzi, makatibu, hospitali — minuta print-ready + sauti. Kwa usalama: academy inasomesha familia na wafanyakazi."
        else "Scribe serves companies, students, secretaries, clinics — print-ready minutes + voice."),
      uiOutput("mlz_out"))
  })

  mlz_run <- function(args, timeout = 60) sys_run(args, timeout)
  observeEvent(input$mlz_fwd, output$mlz_out <- renderUI(.shield_pre(mlz_run(c("mlinzi", "forward")), "Forwarding codes (rasmi)")))
  observeEvent(input$mlz_fwdstop, output$mlz_out <- renderUI(.shield_pre(mlz_run(c("mlinzi", "forward-stop", "--yes"), 90), "ZIMA forwarding zote (##002# kwa adb)")))
  observeEvent(input$mlz_status, output$mlz_out <- renderUI(.shield_pre(mlz_run(c("mlinzi", "status")), "Hali ya usalama")))
  observeEvent(input$mlz_smsshield, {
    m <- trimws(input$mlz_sms %||% "")
    output$mlz_out <- renderUI(if (nzchar(m)) .shield_pre(mlz_run(c("mlinzi", "sms", shQuote(m))), "SMS SHIELD") else NULL)
  })
  observeEvent(input$mlz_linkcheck, {
    u <- trimws(input$mlz_link %||% "")
    output$mlz_out <- renderUI(if (nzchar(u)) .shield_pre(mlz_run(c("mlinzi", "link", u)), "LINK CHECK (phishing)") else NULL)
  })
  observeEvent(input$mlz_academy, output$mlz_out <- renderUI(
    .shield_pre(mlz_run(c("mlinzi", "academy", input$mlz_lesson %||% "1")), "Shule ya usalama (+sauti)")))
  observeEvent(input$mlz_email, output$mlz_out <- renderUI(.shield_pre(mlz_run(c("mlinzi", "email")), "Phishing email — alama")))
  observeEvent(input$dr_scan, output$mlz_out <- renderUI(.shield_pre(mlz_run(c("datarec", "scan"), 120), "Deep-scan ya disks (halisi)")))
  observeEvent(input$dr_plan, output$mlz_out <- renderUI(.shield_pre(mlz_run(c("datarec", "plan")), "Mpango wa recovery")))
  observeEvent(input$dr_bin, output$mlz_out <- renderUI(.shield_pre(mlz_run(c("datarec", "recycle"), 60), "Recycle Bin (halisi)")))
  observeEvent(input$dr_restore, output$mlz_out <- renderUI(.shield_pre(mlz_run(c("datarec", "recycle", "restore"), 90), "Rudisha kutoka Recycle Bin")))
  observeEvent(input$dr_photo, output$mlz_out <- renderUI(.shield_pre(mlz_run(c("datarec", "photorec")), "PhotoRec — hatua kwa hatua")))
  observeEvent(input$dr_phone, output$mlz_out <- renderUI(.shield_pre(mlz_run(c("datarec", "phone"), 60), "Recovery ya simu")))
  observeEvent(input$scr_start, output$mlz_out <- renderUI(
    .shield_pre(mlz_run(c("scribe", "start", input$scr_type %||% "meeting")), "Scribe session mpya (agent anaouza maelezo)")))
  observeEvent(input$scr_sessions, output$mlz_out <- renderUI(.shield_pre(mlz_run(c("scribe", "sessions")), "Scribe sessions")))
  observeEvent(input$scr_add, {
    sid <- trimws(input$scr_id %||% "")
    ln <- trimws(input$scr_line %||% "")
    output$mlz_out <- renderUI(if (nzchar(sid) && nzchar(ln))
      .shield_pre(mlz_run(c("scribe", "add", "--id", sid, "--kind", input$scr_kind %||% "note", "--text", shQuote(ln))), "Mstari umeongezwa")
      else NULL)
  })
  observeEvent(input$scr_finish, {
    sid <- trimws(input$scr_id %||% "")
    output$mlz_out <- renderUI(if (nzchar(sid))
      .shield_pre(mlz_run(c("scribe", "finish", sid, "--speak"), 60), "Minuta print-ready")
      else NULL)
  })

  # ---- ADVANCED: error codes + reset agentic + restore + guard ----
  output$mob_advanced <- renderUI({
    tags$div(class = "panel", style = "background:#050d18;padding:12px;border-radius:8px;margin:12px 0",
      tags$h3(style = "color:var(--cyan);margin:0 0 8px", "\U0001f6a8 ADVANCED — errors / reset / restore / guard"),
      # ERROR CODES
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center;margin-bottom:8px",
        tags$input(id = "err_code", placeholder = "Error code: 0x0000007B", style = "flex:1;min-width:170px"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('err_solve', Date.now(), {priority:'event'})", "\U0001f9e9 Solve (agentic)"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('err_list', Date.now(), {priority:'event'})", "\U0001f4cb Orodha")),
      # RESET AGENTIC
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center;margin-bottom:8px",
        selectInput("reset_target", NULL, choices = c(
          "Windows (local account)" = "windows_local",
          "Windows (Microsoft account)" = "windows_ms",
          "Android" = "android",
          "Email (Gmail/Outlook/Yahoo)" = "email",
          "Social (FB/IG/TikTok/WhatsApp)" = "social"), width = "220px"),
        tags$input(id = "reset_brand", placeholder = "Brand (samsung/gmail/facebook)", style = "flex:1;min-width:160px"),
        tags$input(id = "reset_account", placeholder = "Account/IMEI", style = "flex:1;min-width:140px"),
        tags$button(class = "btn", onclick = "Shiny.setInputValue('reset_run', Date.now(), {priority:'event'})", "\U0001f511 Reset (agentic, HITL)")),
      # RESTORE
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center;margin-bottom:8px",
        tags$input(id = "bak_file", placeholder = "Backup file: backup.zip / .ab", style = "flex:1;min-width:170px"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('bak_verify', Date.now(), {priority:'event'})", "\u2705 Verify"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('bak_plan', Date.now(), {priority:'event'})", "\U0001f4cb Plan"),
        tags$button(class = "btn", style = "background:#7a2048;border-color:#7a2048", onclick = "Shiny.setInputValue('bak_restore', Date.now(), {priority:'event'})", "\U0001f504 Restore (HITL)")),
      # GUARD
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;margin-bottom:4px",
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('guard_base', Date.now(), {priority:'event'})", "\U0001f6e1 Guard: Baseline"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('guard_check', Date.now(), {priority:'event'})", "\U0001f6e1 Guard: Check kila siku"),
        tags$button(class = "btn sec", onclick = "Shiny.setInputValue('guard_install', Date.now(), {priority:'event'})", "\u2699 Guard: Install daily")),
      tags$div(style = "font-size:11px;color:var(--dim);margin-top:4px",
        if (sw()) "GUARD: baseline = pima hali SALAMA; check kila siku = hosts/startup/temp drift = dalili za malware. Reset na Restore ni HITL — consent LAZIMA."
        else "GUARD: baseline = snapshot safe state; daily check = hosts/startup/temp drift = malware signs. Reset & Restore are HITL — consent required."),
      uiOutput("adv_out"))
  })

  adv_run <- function(args, timeout = 120) {
    bin <- .mobile_binary()
    if (!length(bin)) return(NULL)
    tryCatch(suppressWarnings(system2(bin[1], args, stdout = TRUE, stderr = FALSE, timeout = timeout)),
      error = function(e) NULL)
  }

  observeEvent(input$err_solve, {
    code <- trimws(input$err_code %||% "")
    out <- if (nzchar(code)) adv_run(c("errors", "solve", code)) else NULL
    output$adv_out <- renderUI(.shield_pre(out, "Solve"))
  })
  observeEvent(input$err_list, {
    output$adv_out <- renderUI(.shield_pre(adv_run(c("errors", "list")), "Errors"))
  })
  observeEvent(input$reset_run, {
    out <- adv_run(c("reset", "run",
      "--target", shQuote(input$reset_target %||% "windows_local"),
      "--brand", shQuote(input$reset_brand %||% ""),
      "--account", shQuote(input$reset_account %||% ""),
      "--customer", shQuote(input$pro_customer %||% "Mteja")))
    output$adv_out <- renderUI(.shield_pre(out, "Reset agentic"))
  })
  observeEvent(input$bak_verify, {
    f <- trimws(input$bak_file %||% "")
    out <- if (nzchar(f)) adv_run(c("backup-verify", "--file", shQuote(f))) else NULL
    output$adv_out <- renderUI(.shield_pre(out, "Verify"))
  })
  observeEvent(input$bak_plan, {
    f <- trimws(input$bak_file %||% "")
    out <- if (nzchar(f)) adv_run(c("restore-plan", "--file", shQuote(f), "--dest", shQuote("./restored"))) else NULL
    output$adv_out <- renderUI(.shield_pre(out, "Plan"))
  })
  observeEvent(input$bak_restore, {
    f <- trimws(input$bak_file %||% "")
    dev <- trimws(input$shield_device %||% "")
    out <- if (nzchar(f) && nzchar(dev)) adv_run(c("restore-run", "--file", shQuote(f), "--dest", shQuote("./restored"), "--device", shQuote(dev))) else NULL
    output$adv_out <- renderUI(if (is.null(out)) tags$div(class = "status-banner WARNING",
      "Andika Backup file + Device ID (consent restore_data LAZIMA kwanza)")
      else .shield_pre(out, "Restore"))
  })
  observeEvent(input$guard_base, {
    output$adv_out <- renderUI(.shield_pre(adv_run(c("shield", "guard", "baseline")), "Guard baseline"))
  })
  observeEvent(input$guard_check, {
    output$adv_out <- renderUI(.shield_pre(adv_run(c("shield", "guard", "check")), "Guard check"))
  })
  observeEvent(input$guard_install, {
    output$adv_out <- renderUI(.shield_pre(adv_run(c("shield", "guard", "install")), "Guard install (inahitaji admin)", 180))
  })

  # ---- MATATIZO YOTE: software maintenance / computer / email / social media ----
  output$mob_problems <- renderUI({
    # Matatizo YOTE: faili tatu (computer/email/social + networking/printers/activation + simu zote)
    files <- c("mobile/software_problems.json", "mobile/more_problems.json", "mobile/phone_problems.json")
    cats <- unlist(lapply(files, function(f) (.mobile_read(f))$categories %||% list()), recursive = FALSE)
    if (!length(cats)) return(NULL)
    total <- sum(vapply(cats, function(c) length(c$problems %||% list()), numeric(1)))
    cat_blocks <- lapply(cats, function(c) {
      items <- c$problems %||% list()
      rows <- lapply(seq_along(items), function(i) {
        it <- items[[i]]
        tags$details(style = "margin:4px 0;border-left:2px solid #0e7490;padding-left:8px",
          tags$summary(style = "font-size:12px;cursor:pointer", paste0(i, ". ", it$p %||% "?")),
          tags$div(style = "font-size:11px;color:#b2ebf2;margin:6px 0 0;white-space:pre-wrap", paste0("SULUHISHO:\n", it$s %||% "")),
          tags$div(style = "font-size:10px;color:var(--dim);margin-top:3px", paste0("\u23f1 dakika ", it$minutes %||% "?")))
      })
      tags$div(style = "margin:10px 0",
        tags$h4(style = "color:var(--cyan);margin:0 0 6px", paste(c$icon %||% "\u2022", c$name_sw %||% "?",
          paste0("(", length(items), ")"))),
        rows)
    })
    tags$div(class = "panel", style = "background:#050d18;padding:12px;border-radius:8px;margin:12px 0",
      tags$h3(style = "color:var(--cyan);margin:0 0 4px",
        if (sw()) "\U0001f9f0 MATATIZO YOTE + SULUHISHO"
        else "\U0001f9f0 ALL PROBLEMS + FIXES"),
      tags$div(style = "font-size:11px;color:var(--dim);margin-bottom:6px",
        sprintf("%s matatizo: maintenance / computer / email / social / networking / printers / activation / Android / iPhone / button — bonyeza kila moja kuona suluhisho", total)),
      cat_blocks)
  })

  # ---- NET AGENTIC: agents 10 + diagnosis halisi mbili ----
  net_jobs_rv <- reactiveVal(.mobile_read("mobile/net_jobs.json"))
  output$mob_netagentic <- renderUI({
    jl <- net_jobs_rv() %||% list()
    rows <- if (!length(jl)) tags$div(class = "meta", "Hakuna sessions bado.")
      else lapply(rev(jl)[seq_len(min(4, length(jl)))], function(j)
        tags$div(style = "display:flex;justify-content:space-between;font-size:12px;margin:3px 0",
          tags$span(paste0(j$id, " · ", j$problem)),
          tags$span(class = paste("badge", if (identical(j$status, "done")) "GOOD" else "WARNING"), j$status)))
    tags$div(class = "panel", style = "background:#050d18;padding:12px;border-radius:8px;margin:12px 0",
      tags$h3(style = "color:var(--cyan);margin:0 0 8px",
        if (sw()) "\U0001f4e1 NET AGENTIC — agents 10 wanachunguza mtandao"
        else "\U0001f4e1 NET AGENTIC — 10 agents diagnose the network"),
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap",
        tags$input(id = "netag_customer", placeholder = "Jina la mteja", style = "flex:1;min-width:130px"),
        tags$input(id = "netag_problem", placeholder = "Tatizo: internet haifanyi kazi", style = "flex:2;min-width:180px"),
        tags$button(class = "btn",
          onclick = "Shiny.setInputValue('netag_run', Date.now(), {priority:'event'})",
          if (sw()) "\u25b6 Chunguza (agents, ~sek 20)" else "\u25b6 Diagnose (agents, ~20s)")),
      tags$div(style = "font-size:11px;color:var(--dim);margin-top:6px",
        if (sw()) "Mtiririko: receptionist \u2192 vision (netdiag HALISI) \u2192 diagnoser \u2192 planner \u2192 HITL \u2192 solver \u2192 tester (netdiag ya pili) \u2192 verifier \u2192 scribe \u2192 learner."
        else "Flow: receptionist \u2192 vision (real netdiag) \u2192 diagnoser \u2192 planner \u2192 HITL \u2192 solver \u2192 tester (2nd netdiag) \u2192 verifier \u2192 scribe \u2192 learner."),
      uiOutput("netag_out"),
      tags$div(style = "margin-top:8px", rows))
  })

  observeEvent(input$netag_run, {
    bin <- .mobile_binary()
    if (!length(bin)) {
      output$netag_out <- renderUI(tags$div(class = "status-banner CRITICAL",
        "fundi-mobile binary haipo — build kwanza: cd fundi-mobile && cargo build --release"))
      return()
    }
    args <- c("netagentic", "run",
      "--customer", shQuote(input$netag_customer %||% "Mteja"),
      "--problem", shQuote(input$netag_problem %||% "internet haifanyi kazi"))
    out <- tryCatch(
      suppressWarnings(system2(bin[1], args, stdout = TRUE, stderr = FALSE, timeout = 120)),
      error = function(e) NULL)
    net_jobs_rv(.mobile_read("mobile/net_jobs.json"))
    output$netag_out <- renderUI(
      if (is.null(out)) tags$div(class = "status-banner CRITICAL", "Session imeshindikana")
      else tags$pre(style = "color:#b2ebf2;font-size:12px;background:#050d18;padding:10px;border-radius:8px;margin:8px 0 0;white-space:pre-wrap;max-height:320px;overflow:auto",
        paste(out, collapse = "\n")))
  })

  # ---- TELECOMS & NETWORK: kikokotoo + diagnostics halisi L1-L7 ----
  output$mob_telecom <- renderUI({
    d <- .mobile_read("mobile/telecom_formulas.json")
    groups <- d$groups %||% list()
    probs <- d$problems %||% list()
    fmt_opts <- list()
    for (g in groups) {
      for (f in g$formulas %||% list()) {
        fmt_opts[[paste(g$icon %||% "\u2022", f$name_sw, "—", f$formula)]] <- f$compute %||% f$id
      }
    }
    prob_items <- unlist(lapply(probs, function(l)
      lapply(l$items %||% list(), function(it)
        tags$div(style = "font-size:11px;margin:3px 0",
          tags$b(paste0(l$layer, ": ", it$p)),
          tags$div(style = "color:#00e676;margin-left:10px", paste0("\u2192 ", it$s)))))
    , recursive = FALSE)
    tags$div(class = "panel", style = "background:#050d18;padding:12px;border-radius:8px;margin:12px 0",
      tags$h3(style = "color:var(--cyan);margin:0 0 8px",
        if (sw()) "\U0001f4e1 TELECOMS & NETWORK — kikokotoo + diagnostics"
        else "\U0001f4e1 TELECOMS & NETWORK — calculator + diagnostics"),
      # Kikokotoo
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap;align-items:center",
        selectInput("net_formula", NULL, choices = fmt_opts, width = "420px"),
        tags$input(id = "net_args", placeholder = "--d_km 10 --f_mhz 5800", style = "flex:1;min-width:180px"),
        tags$button(class = "btn sec",
          onclick = "Shiny.setInputValue('net_calc_run', Date.now(), {priority:'event'})",
          "\U0001f9ee Hesabu")),
      uiOutput("net_calc_out"),
      # Diagnostics
      tags$div(style = "display:flex;gap:8px;margin-top:10px",
        tags$button(class = "btn",
          onclick = "Shiny.setInputValue('net_diag_run', Date.now(), {priority:'event'})",
          if (sw()) "\U0001f50d Chunguza mtandao (L1\u2192L7, ~sek 10)" else "\U0001f50d Diagnose network (L1\u2192L7, ~10s)")),
      uiOutput("net_diag_out"),
      # Matatizo
      if (length(prob_items)) tags$details(style = "margin-top:10px",
        tags$summary(style = "font-size:11px;color:var(--cyan);cursor:pointer",
          if (sw()) "MATATIZO + SULUHISHO (bonyeza)" else "PROBLEMS + FIXES (click)"),
        tags$div(style = "margin-top:6px", prob_items)))
  })

  observeEvent(input$net_calc_run, {
    bin <- .mobile_binary()
    if (!length(bin)) {
      output$net_calc_out <- renderUI(tags$div(class = "status-banner CRITICAL", "fundi-mobile binary haipo"))
      return()
    }
    compute <- input$net_formula %||% "fspl"
    args_extra <- strsplit(trimws(input$net_args %||% ""), "\\s+")[[1]] %||% character(0)
    out <- .mobile_run(c("netcalc", compute, args_extra))
    output$net_calc_out <- renderUI(
      if (is.null(out)) tags$div(class = "status-banner WARNING", "Kikokotoo kimeshindikana")
      else tags$pre(style = "color:#b2ebf2;font-size:12px;background:#050d18;padding:8px;border-radius:8px;margin:8px 0 0",
        paste(out, collapse = "\n")))
  })

  observeEvent(input$net_diag_run, {
    bin <- .mobile_binary()
    if (!length(bin)) {
      output$net_diag_out <- renderUI(tags$div(class = "status-banner CRITICAL", "fundi-mobile binary haipo"))
      return()
    }
    out <- .mobile_run(c("netdiag"))
    output$net_diag_out <- renderUI(
      if (is.null(out)) tags$div(class = "status-banner WARNING", "Diagnostics imeshindikana")
      else tags$pre(style = "color:#b2ebf2;font-size:12px;background:#050d18;padding:8px;border-radius:8px;margin:8px 0 0;white-space:pre-wrap",
        paste(out, collapse = "\n")))
  })

  # ---- FUNDI PRO AGENTIC: agents 10 zinaendesha session ya huduma ----
  output$mob_proagentic_out <- renderUI({
    jobs <- .mobile_read("mobile/pro_jobs.json")
    jl <- jobs %||% list()
    rows <- if (!length(jl)) tags$div(class = "meta", "Hakuna sessions bado.")
      else lapply(rev(jl)[seq_len(min(5, length(jl)))], function(j)
        tags$div(style = "display:flex;justify-content:space-between;font-size:12px;margin:3px 0",
          tags$span(paste0(j$id, " · ", j$service_id, " · ", j$customer)),
          tags$span(class = paste("badge", if (identical(j$status, "done")) "GOOD" else "WARNING"),
            sprintf("%s · TZS %s", j$status, format(j$price_tzs %||% 0, big.mark = ",")))))
    tags$div(class = "panel", style = "background:#050d18;padding:12px;border-radius:8px;margin:12px 0",
      tags$h3(style = "color:var(--cyan);margin:0 0 8px",
        if (sw()) "\U0001f916 FUNDI PRO AGENTIC — agents zinaendesha session"
        else "\U0001f916 FUNDI PRO AGENTIC — agents run the session"),
      tags$div(style = "display:flex;gap:8px;flex-wrap:wrap",
        tags$input(id = "pro_customer", placeholder = "Jina la mteja", style = "flex:1;min-width:130px"),
        tags$input(id = "pro_phone", placeholder = "Simu", style = "flex:1;min-width:110px"),
        tags$input(id = "pro_problem", placeholder = "Tatizo: nimesahau password", style = "flex:2;min-width:160px"),
        tags$input(id = "pro_subs", placeholder = "Subs (tra,brela — optional)", style = "flex:1;min-width:150px")),
      tags$div(style = "display:flex;gap:8px;margin-top:8px;align-items:center",
        selectInput("pro_service", NULL, choices = c(
          "📧 Email Troubleshooting" = "email_troubleshooting",
          "🏛️ Government Applications" = "government_applications",
          "🔓 Hacked Account Recovery (HITL)" = "hacked_account_recovery",
          "💾 Data Recovery (HITL)" = "data_recovery",
          "☁️ Backup Services" = "backup_services",
          "🛡️ Security & Privacy" = "security_privacy"), width = "320px"),
        tags$button(class = "btn",
          onclick = "Shiny.setInputValue('pro_run', Date.now(), {priority:'event'})",
          if (sw()) "\u25b6 Endesha agents" else "\u25b6 Run agents")),
      tags$div(style = "font-size:11px;color:var(--dim);margin-top:6px",
        if (sw()) "Mtiririko: receptionist \u2192 diagnoser \u2192 planner \u2192 HITL \u2192 solver \u2192 invoice (FUNDI PAY) \u2192 learner. Huduma nyeti zinasimama bila consent."
        else "Flow: receptionist \u2192 diagnoser \u2192 planner \u2192 HITL \u2192 solver \u2192 invoice (FUNDI PAY) \u2192 learner. Sensitive services pause without consent."),
      uiOutput("pro_run_out"),
      tags$div(style = "margin-top:8px", rows))
  })

  observeEvent(input$pro_run, {
    bin <- .mobile_binary()
    if (!length(bin)) {
      output$pro_run_out <- renderUI(tags$div(class = "status-banner CRITICAL",
        "fundi-mobile binary haipo — build kwanza: cd fundi-mobile && cargo build --release"))
      return()
    }
    args <- c("pro-run",
      "--service", shQuote(input$pro_service %||% "email_troubleshooting"),
      "--customer", shQuote(input$pro_customer %||% "Mteja"),
      "--phone", shQuote(input$pro_phone %||% ""),
      "--problem", shQuote(input$pro_problem %||% "tatizo haijaelezwa"))
    subs <- trimws(input$pro_subs %||% "")
    if (nzchar(subs)) args <- c(args, "--subs", shQuote(subs))
    out <- .mobile_run(args)
    output$pro_run_out <- renderUI(
      if (is.null(out)) tags$div(class = "status-banner CRITICAL", "Session imeshindikana")
      else tags$pre(style = "color:#b2ebf2;font-size:12px;background:#050d18;padding:10px;border-radius:8px;margin:8px 0 0;white-space:pre-wrap;max-height:300px;overflow:auto",
        paste(out, collapse = "\n")))
  })

  # ---- FUNDI PRO: huduma za ofisi (catalog + bei TZS + badges remote/onsite) ----
  output$mob_pro <- renderUI({
    d <- .mobile_read("mobile/pro_services.json")
    svcs <- d$services %||% list()
    if (!length(svcs)) return(NULL)
    badge <- function(mode) {
      cls <- switch(mode %||% "", remote = "GOOD", partial_remote = "WARNING", onsite = "CRITICAL", "INFO")
      lab <- switch(mode %||% "",
        remote = if (sw()) "\u2705 100% REMOTE" else "\u2705 100% REMOTE",
        partial_remote = if (sw()) "\U0001f7e1 SEHEMU REMOTE" else "\U0001f7e1 PARTIAL",
        onsite = if (sw()) "\u274c ONSITE TU" else "\u274c ONSITE ONLY",
        mode %||% "?")
      as.character(tags$span(class = paste("badge", cls), lab))
    }
    cards <- lapply(svcs, function(s) {
      rows <- list()
      for (x in c("systems", "targets", "cases", "options", "items")) {
        arr <- s[[x]] %||% list()
        if (length(arr)) {
          rows[[length(rows) + 1]] <- tags$div(style = "margin:4px 0 0 12px",
            lapply(arr, function(it) tags$div(style = "display:flex;justify-content:space-between;font-size:12px",
              tags$span(paste0("\u2022 ", it$name %||% it$id),
                if (!is.null(it$recurrence)) tags$span(style = "color:var(--dim);font-size:10px", paste0(" · ", it$recurrence)),
                if (!is.null(it$hours)) tags$span(style = "color:var(--dim);font-size:10px", paste0(" · saa ", it$hours))),
              tags$span(style = "color:#00e676;font-weight:600", sprintf("TZS %s", format(it$price_tzs %||% 0, big.mark = ","))))))
        }
      }
      tags$div(class = "panel", style = "background:#050d18;padding:10px;border-radius:8px;margin:8px 0",
        tags$div(style = "display:flex;justify-content:space-between;align-items:center;flex-wrap:wrap;gap:6px",
          tags$h3(style = "color:var(--cyan);margin:0", paste(s$icon %||% "\u2022", s$name_sw)),
          HTML(badge(s$mode))),
        tags$div(style = "font-size:11px;color:var(--dim);margin:2px 0 4px",
          sprintf("TZS %s - %s · dakika %s-%s",
            format(s$price_min_tzs %||% 0, big.mark = ","),
            format(s$price_max_tzs %||% 0, big.mark = ","),
            s$minutes_min %||% "?", s$minutes_max %||% "?")),
        rows,
        if (length(s$steps_sw %||% list())) tags$details(style = "margin-top:6px",
          tags$summary(style = "font-size:11px;color:var(--cyan);cursor:pointer",
            if (sw()) "HATUA (bonyeza)" else "STEPS (click)"),
          tags$ol(style = "font-size:11px;color:#b2ebf2;margin:4px 0 0;padding-left:20px",
            lapply(s$steps_sw, function(st) tags$li(st)))),
        if (!is.null(s$warning_sw)) tags$div(class = "status-banner WARNING", style = "font-size:11px;margin-top:6px", s$warning_sw),
        if (!is.null(s$rule_sw)) tags$div(class = "meta", style = "font-size:11px;margin-top:4px", s$rule_sw))
    })
    tags$div(tags$h3(style = "color:var(--cyan);margin:12px 0 4px",
      if (sw()) "\U0001fa7a FUNDI PRO — HUDUMA ZOTE (bei za mwongozo)"
      else "\U0001fa7a FUNDI PRO — ALL SERVICES (guide prices)"), cards)
  })

  observeEvent(input$recover_run, {
    bin <- .mobile_binary()
    if (!length(bin)) {
      output$recover_out <- renderUI(tags$div(class = "status-banner CRITICAL",
        "fundi-mobile binary haipo — build kwanza: cd fundi-mobile && cargo build --release"))
      return()
    }
    tool <- input$recover_tool %||% "list"
    imei <- trimws(input$recover_imei %||% "")
    args <- c("recover", tool)
    if (nzchar(imei) && !identical(tool, "list")) args <- c(args, "--imei", shQuote(imei))
    out <- .mobile_run(args)
    output$recover_out <- renderUI(
      if (is.null(out)) tags$div(class = "status-banner WARNING",
        "Imeshindikana — kumbuka: consent ya mmiliki LAZIMA (fundi-mobile consent add --service android_access_recovery --imei ...)")
      else tags$pre(style = "color:#b2ebf2;font-size:12px;background:#050d18;padding:10px;border-radius:8px;margin:8px 0 0;white-space:pre-wrap;max-height:340px;overflow:auto",
        paste(out, collapse = "\n")))
  })

  observeEvent(input$mob_run, {
    bin <- .mobile_binary()
    if (!length(bin)) {
      output$mob_run_out <- renderUI(tags$div(class = "status-banner CRITICAL",
        "fundi-mobile binary haipo — build kwanza: cd fundi-mobile && cargo build --release"))
      return()
    }
    args <- c("agentic", "run",
      "--customer", shQuote(input$mob_customer %||% "Mteja"),
      "--phone", shQuote(input$mob_phone %||% ""),
      "--brand", shQuote(input$mob_brand %||% "generic"),
      "--model", shQuote(input$mob_model %||% "unknown"),
      "--imei", shQuote(input$mob_imei %||% ""),
      "--service", shQuote(input$mob_service %||% "diagnostics"),
      "--problem", shQuote(input$mob_problem %||% "tatizo haijaelezwa"),
      "--tech", shQuote("fundi (shiny)"))
    out <- .mobile_run(args)
    jobs_rv(.mobile_read("mobile/jobs.json"))
    output$mob_run_out <- renderUI(
      if (is.null(out)) tags$div(class = "status-banner CRITICAL", "Agentic run imeshindikana")
      else tags$pre(style = "color:#b2ebf2;font-size:12px;background:#050d18;padding:10px;border-radius:8px",
        paste(out, collapse = "\n")))
  })

  output$mob_summary <- renderUI({
    j <- jobs_rv()
    cns <- consents_rv()
    tags$div(class = "meta",
      sprintf("Jobs: %d · Consents: %d · Binary: %s",
        length(j %||% list()), length(cns %||% list()),
        if (length(.mobile_binary())) "ipo \u2705" else "haipo \u26a0\ufe0f"))
  })

  # ---- B2B: quotes/invoices za FUNDI kwa makampuni (pesa ni za fundi mwenyewe) ----
  output$mob_b2b <- renderUI({
    docs <- .mobile_read("mobile/b2b_docs.json")
    if (!length(docs %||% list())) return(NULL)
    qs <- Filter(function(d) identical(d$kind, "quote"), docs)
    ivs <- Filter(function(d) identical(d$kind, "invoice"), docs)
    acc <- Filter(function(q) identical(q$status, "accepted"), qs)
    paid <- Filter(function(i) identical(i$status, "paid"), ivs)
    unpaid <- Filter(function(i) identical(i$status, "unpaid"), ivs)
    sum_tzs <- function(l) sum(vapply(l, function(x) x$total %||% 0, numeric(1)), na.rm = TRUE)
    tags$div(class = "panel", style = "background:#050d18;padding:10px;border-radius:8px",
      tags$h3(style = "color:var(--cyan);margin:0 0 6px", "\U0001f4bc B2B — KAZI ZA MAKAMPUNI (pesa ni za fundi)"),
      tags$div(style = "display:flex;gap:14px;flex-wrap:wrap",
        tags$div(tags$div(style = "font-size:10px;color:var(--dim)", "QUOTES zimekubaliwa"),
          tags$div(style = "color:#00e676;font-size:16px;font-weight:700",
            sprintf("%d · TZS %s", length(acc), format(sum_tzs(acc), big.mark = ",")))),
        tags$div(tags$div(style = "font-size:10px;color:var(--dim)", "INVOICES zilizolipwa"),
          tags$div(style = "color:#00e676;font-size:16px;font-weight:700",
            sprintf("%d · TZS %s", length(paid), format(sum_tzs(paid), big.mark = ",")))),
        tags$div(tags$div(style = "font-size:10px;color:var(--dim)", "ZINASUBIRI"),
          tags$div(style = "color:#ffc107;font-size:16px;font-weight:700",
            sprintf("%d · TZS %s", length(unpaid), format(sum_tzs(unpaid), big.mark = ","))))),
      tags$div(style = "margin-top:8px", lapply(rev(docs), function(d) {
        col <- switch(d$status %||% "", accepted = , paid = "#00e676",
                      rejected = , overdue = "#ff1744", "#ffc107")
        did <- d$id %||% "?"
        tags$div(style = "border:1px solid #00e5ff22;border-radius:6px;padding:6px 10px;margin:4px 0;background:#0a1628",
          tags$b(style = "font-size:12px", did), " ",
          tags$span(style = paste0("font-size:10px;color:", col), toupper(d$status %||% "?")),
          tags$div(style = "font-size:11px;color:#b2ebf2;display:flex;justify-content:space-between;align-items:center",
            tags$span(sprintf("%s → %s · TZS %s", d$kind %||% "?", d$company %||% "?",
                    format(d$total %||% 0, big.mark = ","))),
            tags$button(class = "btn", style = "padding:2px 10px;font-size:11px",
              onclick = sprintf("mobDocPdf('%s')", did),
              title = "Pakua PDF rasmi (logo ya FUNDI)",
              "\U0001f4d1 PDF")))
      })),
      tags$div(style = "font-size:11px;color:var(--dim);margin-top:6px",
        "PDF inatengenezwa na binary ya fundi-mobile (b2b pdf --id ...) — ina logo ya FUNDI."),
      # downloadHandler hidden + onchange trigger
      downloadButton("mob_pdf_dl", style = "display:none", class = "mob-pdf-dl"),
      tags$script(HTML("
        function mobDocPdf(id) {
          Shiny.setInputValue('mob_pdf_req', id, {priority:'event'});
          setTimeout(function(){
            var a = document.querySelector('.mob-pdf-dl');
            if (a) a.click();
          }, 300);
        }
      "))
    )
  })

  # ---- MAPATO (FUNDI PAY) — invoices + payments kutoka data/mobile/*.json ----
  output$mob_revenue <- renderUI({
    invs <- .mobile_read("mobile/invoices.json")
    pays <- .mobile_read("mobile/payments.json")
    if (!length(invs %||% list()) && !length(pays %||% list()))
      return(NULL)
    total_paid <- 0
    unpaid <- 0
    unpaid_list <- list()
    for (i in invs %||% list()) {
      st <- i$status %||% "unpaid"
      amt <- i$total %||% 0
      if (identical(st, "paid")) total_paid <- total_paid + amt
      else {
        unpaid <- unpaid + amt
        unpaid_list[[length(unpaid_list) + 1]] <- i
      }
    }
    by_provider <- list()
    for (p in pays %||% list()) {
      k <- p$provider %||% "?"
      by_provider[[k]] <- (by_provider[[k]] %||% 0) + (p$amount %||% 0)
    }
    prov_html <- if (length(by_provider)) {
      tags$div(style = "font-size:11px;color:var(--dim)",
        paste("Kwa provider:", paste(names(by_provider),
              sprintf("TZS %s", format(unlist(by_provider), big.mark = ",")),
              sep = " ", collapse = " · ")))
    } else NULL
    tags$div(class = "panel", style = "background:#050d18;padding:10px;border-radius:8px",
      tags$h3(style = "color:var(--cyan);margin:0 0 6px", "\U0001f4b0 MAPATO (FUNDI PAY)"),
      tags$div(style = "display:flex;gap:14px;flex-wrap:wrap",
        tags$div(tags$div(style = "font-size:10px;color:var(--dim)", "YAMEPOKEWA"),
          tags$div(style = "color:#00e676;font-size:18px;font-weight:700",
            paste0("TZS ", format(total_paid, big.mark = ",")))),
        tags$div(tags$div(style = "font-size:10px;color:var(--dim)", "MADENI (unpaid)"),
          tags$div(style = "color:#ffc107;font-size:18px;font-weight:700",
            paste0("TZS ", format(unpaid, big.mark = ",")))),
        tags$div(tags$div(style = "font-size:10px;color:var(--dim)", "MALIPO"),
          tags$div(style = "color:#e0f7fa;font-size:18px;font-weight:700",
            length(pays %||% list())))),
      prov_html,
      if (length(unpaid_list)) {
        tags$div(style = "margin-top:6px", lapply(unpaid_list, function(i) {
          tags$div(style = "font-size:11px;color:#ffc107",
            sprintf("\u23f3 %s · %s · TZS %s", i$id %||% "?", i$customer %||% "?",
                    format(i$total %||% 0, big.mark = ",")))
        }))
      })
  })

  output$mob_services <- renderUI({
    s <- services_rv()
    svcs <- s$services %||% list()
    if (!length(svcs)) return(tags$div(class = "meta", "services.json haipo"))
    tags$div(
      tags$div(class = "panel-head", tags$h3("\U0001f4b0 HUDUMA + BEI (TZS)")),
      tags$table(style = "width:100%;font-size:12px;border-collapse:collapse",
        tags$tr(tags$th("Huduma"), tags$th("Bei"), tags$th("Muda"), tags$th("Risk"), tags$th("Consent"), tags$th("Data")),
        lapply(svcs, function(x) {
          tags$tr(style = "border-bottom:1px solid #00e5ff22",
            tags$td(x$name_sw %||% "?"),
            tags$td(format(x$price %||% 0, big.mark = ",")),
            tags$td(paste0(x$minutes %||% "?", " min")),
            tags$td(toupper(x$risk %||% "?")),
            tags$td(if (isTRUE(x$requires_consent)) "\U0001f510 ndiyo" else "—"),
            tags$td(if (isTRUE(x$destroys_data)) "\u26a0\ufe0f inafuta" else "inabaki"))
        })))
  })

  output$mob_button_codes <- renderUI({
    b <- brands_rv()
    bp <- b$button_phones %||% list()
    if (!length(bp)) return(tags$div(class = "meta", "brands.json haipo"))
    tags$div(
      tags$div(class = "panel-head", tags$h3("\U0001f4fb SIMU ZA BUTTON — master reset codes")),
      lapply(names(bp), function(k) {
        v <- bp[[k]]
        tags$div(style = "border:1px solid #00e5ff22;border-radius:6px;padding:6px 10px;margin:4px 0;background:#0a1628",
          tags$b(style = "color:var(--cyan);font-size:12px", k),
          if (length(v$master_codes %||% list()))
            tags$div(style = "font-size:12px;color:#b2ebf2",
              paste(unlist(v$master_codes), collapse = "  ·  ")),
          tags$div(style = "font-size:10px;color:var(--dim)",
            paste0("Hard reset: ", v$hard_reset_sw %||% "?", " · Flash: ", v$flash_tool %||% "?")))
      })
    )
  })

  output$mob_brands <- renderUI({
    b <- brands_rv()
    sm <- b$smartphones %||% list()
    if (!length(sm)) return(NULL)
    tags$div(
      tags$div(class = "panel-head", tags$h3("\U0001f4f1 SMARTPHONES — combos + flash")),
      lapply(names(sm), function(k) {
        v <- sm[[k]]
        tags$div(style = "border:1px solid #00e5ff22;border-radius:6px;padding:6px 10px;margin:4px 0;background:#0a1628",
          tags$b(style = "color:var(--cyan);font-size:12px", k),
          tags$div(style = "font-size:11px;color:#b2ebf2",
            paste0("Recovery: ", v$recovery_combo_sw %||% "?")),
          tags$div(style = "font-size:10px;color:var(--dim)",
            paste0("Download: ", v$download_mode %||% "?", " · Tool: ",
                   v$flash_tool %||% "?", " · Firmware: ", v$firmware_site %||% "?")))
      })
    )
  })

  output$mob_jobs <- renderUI({
    j <- jobs_rv()
    if (!length(j %||% list())) return(tags$div(class = "meta", "Hakuna jobs bado — endesha AGENTIC RUN"))
    tags$div(
      tags$div(class = "panel-head", tags$h3("\U0001f4cb JOBS")),
      lapply(rev(j), function(x) {
        tags$div(style = "border:1px solid #00e5ff22;border-radius:6px;padding:6px 10px;margin:4px 0;background:#0a1628",
          tags$b(style = "font-size:12px", x$id %||% "?"), " ",
          HTML(.mobile_badge(x$status %||% "?")),
          tags$div(style = "font-size:11px;color:#b2ebf2",
            paste0(x$customer %||% "?", " · ", x$brand %||% "?", " ", x$model %||% "?",
                   " · ", x$service_id %||% "?")),
          tags$div(style = "font-size:10px;color:var(--dim)", x$summary_sw %||% ""))
      }))
  })

  output$mob_consents <- renderUI({
    cns <- consents_rv()
    if (!length(cns %||% list())) return(NULL)
    tags$div(
      tags$div(class = "panel-head", tags$h3("\u2696 CONSENTS (HITL log)")),
      lapply(rev(cns), function(x) {
        tags$div(style = "font-size:11px;color:#b2ebf2;margin:2px 0",
          sprintf("%s | %s | IMEI %s | %s | TZS %s%s",
                  x$consent_id %||% "?", x$customer_name %||% "?",
                  x$imei %||% "?", x$service_id %||% "?",
                  format(x$price_tzs %||% 0, big.mark = ","),
                  if (isTRUE(x$destroys_data)) " \u26a0\ufe0f" else ""))
      }))
  })

  # PDF ya B2B: doc inayochaguliwa kwa mob_pdf_req (id) → download mob_pdf_dl
  mob_pdf_id <- reactiveVal(NULL)
  observeEvent(input$mob_pdf_req, mob_pdf_id(input$mob_pdf_req), ignoreInit = TRUE)
  output$mob_pdf_dl <- downloadHandler(
    filename = function() paste0(mob_pdf_id() %||% "hati", ".pdf"),
    content = function(file) {
      doc_id <- mob_pdf_id() %||% ""
      bin <- .mobile_binaries()
      ok <- FALSE
      if (length(bin) && nzchar(doc_id)) {
        out <- tryCatch(
          suppressWarnings(
            system2(bin[1], c("b2b", "pdf", "--id", doc_id, "--out", shQuote(file)),
                    stdout = TRUE, stderr = TRUE, timeout = 30)),
          error = function(e) NULL)
        ok <- !is.null(out) && file.exists(file) && file.info(file)$size > 200
      }
      if (!ok) {
        docs <- .mobile_read("mobile/b2b_docs.json")
        doc <- NULL
        for (d in docs %||% list()) if (identical(d$id, doc_id)) doc <- d
        writeLines(mobile_b2b_pdf_fallback(doc), file, useBytes = TRUE)
      }
    },
    contentType = "application/pdf"
  )

  output$mob_agents <- renderUI({
    # Agents 10 zinajulikana (data-driven kutoka crate)
    agents <- list(
      c("receptionist", "Mpokeaji"), c("vision", "Muono"), c("diagnoser", "Mgunduzi"),
      c("planner", "Mpangaji"), c("hitl", "Mlinzi"), c("solver", "Mtatuzi"),
      c("tester", "Mjaribu"), c("verifier", "Mthibitishaji"),
      c("scribe", "Mwandishi"), c("learner", "Mwanafunzi"))
    tags$div(
      tags$div(class = "panel-head", tags$h3("\U0001f916 AGENTS 10")),
      tags$div(style = "display:flex;flex-wrap:wrap;gap:6px",
        lapply(agents, function(a) {
          tags$div(style = "border:1px solid #00e5ff33;border-radius:6px;padding:6px 10px;background:#0a1628",
            tags$span(style = "color:var(--cyan);font-size:11px;font-weight:700", a[2]),
            tags$span(style = "font-size:10px;color:var(--dim)", paste0(" (", a[1], ")")))
        })))
  })
}

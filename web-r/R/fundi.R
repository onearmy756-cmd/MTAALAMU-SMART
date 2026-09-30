# ============================================================
# fundi.R — Fundi Deploy tab (P2 + P3): LAN discovery, deploy HITL,
# jobs live, images, OS selection, agents, cloud multi-tenant.
#
# Data source: Fundi Deploy Agent API (fundi-deploy/server/agent, :8080).
# Offline-first: bila API — panel inaonyesha hali "offline", haivunjwi.
# ============================================================

`%||%` <- function(a, b) if (is.null(a) || length(a) == 0) b else a

FUNDI_API <- function() {
  url <- Sys.getenv("FUNDI_DEPLOY_URL", unset = "http://127.0.0.1:8080")
  trimws(url)
}

.has_curl <- requireNamespace("curl", quietly = TRUE)

.fundi_get <- function(path, timeout = 6) {
  url <- paste0(FUNDI_API(), path)
  txt <- tryCatch(
    if (.has_curl) {
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
  .fundi_json(txt)
}

.fundi_get_raw <- function(path, timeout = 6) {
  url <- paste0(FUNDI_API(), path)
  tryCatch(
    if (.has_curl) {
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
}

.fundi_post <- function(path, body = list()) {
  if (!.has_curl) return(NULL)
  url <- paste0(FUNDI_API(), path)
  .fundi_json(.fundi_post_raw(url, body))
}

.fundi_post_raw <- function(url, body = list()) {
  tryCatch(
    {
      h <- curl::new_handle(connecttimeout = 6)
      curl::handle_set_headers(h, c("Content-Type" = "application/json"))
      curl::handle_setopt(h, copypostfields = jsonlite::toJSON(body, auto_unbox = TRUE))
      raw <- curl::curl_fetch_memory(url, handle = h)
      rawToChar(raw$content)
    },
    error = function(e) NULL
  )
}

.fundi_json <- function(txt) {
  if (is.null(txt) || !nzchar(txt)) return(NULL)
  tryCatch(jsonlite::fromJSON(txt, simplifyVector = FALSE), error = function(e) NULL)
}

.fundi_badge <- function(status) {
  cls <- switch(status %||% "",
    awaiting_approval = "WARNING",
    running = "INFO",
    done = "GOOD",
    failed = "CRITICAL",
    cancelled = "CRITICAL",
    "INFO"
  )
  as.character(tags$span(class = paste("badge", cls), status))
}

.fundi_host_row <- function(h) {
  tags$label(
    style = "display:block;margin:4px 0;font-size:13px",
    tags$input(
      type = "checkbox",
      class = "fundi_target",
      value = h$mac %||% "",
      onclick = "fundiTog(this)"
    ),
    tags$b(h$name %||% "?"), " ",
    tags$code(h$mac %||% ""), " ",
    tags$span(class = "meta", paste0("(", h$source %||% "?", ") ", h$ip %||% ""))
  )
}

# ---------- UI ----------

fundi_tab_el <- function(lang) {
  sw <- identical(lang, "sw")
  panel_el(
    title = if (sw) "\u25c9 FUNDI DEPLOY" else "\u25c9 FUNDI DEPLOY",
    meta = "P2 \u2022 backup \u2022 multicast \u2022 AI OS \u2022 P3 cloud",
    tags$div(
      tags$div(
        style = "display:flex;gap:8px;flex-wrap:wrap;margin-bottom:10px",
        tags$button(
          class = "btn",
          onclick = "Shiny.setInputValue('fundi_discover', Date.now(), {priority:'event'})",
          if (sw) "\ud83d\udd0d Gundua computers" else "\ud83d\udd0d Discover computers"
        ),
        tags$button(
          class = "btn",
          onclick = "Shiny.setInputValue('fundi_deploy', Date.now(), {priority:'event'})",
          if (sw) "Anza deploy (HITL)" else "Start deploy (HITL)"
        ),
        tags$button(
          class = "btn",
          onclick = "Shiny.setInputValue('fundi_refresh', Date.now(), {priority:'event'})",
          if (sw) "Onyesha jobs" else "Show jobs"
        )
      ),
      uiOutput("fundi_summary"),
      uiOutput("fundi_computers"),
      uiOutput("fundi_jobs"),
      uiOutput("fundi_extra")
    )
  )
}

# ---------- Server logic ----------

fundi_server <- function(input, output, session) {
  summary_rv <- reactiveVal(NULL)
  hosts_rv   <- reactiveVal(NULL)

  observe({
    invalidateLater(5000, session)
    summary_rv(.fundi_get("/supervisor/summary"))
  })

  observeEvent(input$fundi_discover, {
    hosts_rv(.fundi_get("/computers"))
  })

  observeEvent(input$fundi_deploy, {
    macs <- input$fundi_targets
    if (is.null(macs) || length(macs) == 0) {
      hosts_rv(structure(list(error = "empty"), class = "fundi_no_targets"))
      return()
    }
    computers <- lapply(macs, function(m) list(mac = m, name = paste0("PC-", m)))
    res <- .fundi_post("/deploy", list(computers = computers, os_type = "auto", auto_approve = FALSE))
    session$sendCustomMessage(
      "fundi_note",
      list(text = if (!is.null(res)) {
        paste0("Jobs ", length(res$job_ids %||% list()), " — zinasubiri RUHUSU (HITL)")
      } else {
        "Deploy imeshindikana — API offline"
      })
    )
  })

  observeEvent(input$fundi_refresh, {
    jobs <- .fundi_get("/jobs")
    imgs <- .fundi_get("/images")
    sel  <- .fundi_post("/os/select", list(specs = "16gb ram 8 cores 512 ssd", user_need = "office"))
    ag   <- .fundi_get("/agents")
    cld  <- .fundi_get("/cloud/status")
    bks  <- .fundi_get("/backups")
    prof <- .fundi_get("/os/profiles")

    output$fundi_jobs <- renderUI({
      if (is.null(jobs)) return(tags$div(class = "meta", "Hakuna jobs (API offline)"))
      if (length(jobs) == 0) return(tags$div(class = "meta", "Hakuna jobs bado"))
      tags$div(
        tags$div(class = "panel-head", tags$h3("Jobs"),
          tags$a(href = paste0(FUNDI_API(), "/report/pdf"), target = "_blank",
                 class = "btn sec", style = "text-decoration:none",
                 "\U0001f4d1 Ripoti PDF (logo ya FUNDI)")),
        tags$table(
          style = "width:100%;font-size:13px;border-collapse:collapse",
          tags$tr(
            tags$th("PC"), tags$th("OS"), tags$th("Hali"), tags$th("%"), tags$th("Ujumbe"), tags$th("")),
          lapply(jobs, function(j) {
            tags$tr(
              style = "border-bottom:1px solid #00e5ff22",
              tags$td(tags$b(j$device_name %||% "?"), tags$br(), tags$small(j$device_mac %||% "")),
              tags$td(j$os_type %||% "?"),
              tags$td(HTML(.fundi_badge(j$status %||% "?")), " ", j$stage %||% ""),
              tags$td(paste0(j$progress %||% 0, "%")),
              tags$td(j$message %||% ""),
              tags$td(
                if (identical(j$status, "awaiting_approval"))
                  tags$div(style = "display:flex;gap:4px",
                    tags$button(class = "btn", style = "padding:2px 10px;font-size:11px",
                      onclick = sprintf("fundiJob('%s','approve')", j$id %||% ""), "RUHUSU"),
                    tags$button(class = "btn", style = "padding:2px 10px;font-size:11px;background:#ff1744;color:#fff",
                      onclick = sprintf("fundiJob('%s','cancel')", j$id %||% ""), "GHAIRI"))
                else if (identical(j$status, "running"))
                  tags$button(class = "btn", style = "padding:2px 10px;font-size:11px;background:#ff1744;color:#fff",
                    onclick = sprintf("fundiJob('%s','cancel')", j$id %||% ""), "GHAIRI")
                else NULL))
          })
        )
      )
    })

    output$fundi_extra <- renderUI({
      tags$div(
        tags$div(class = "panel-head", tags$h3("Images za OS")),
        tags$div(class = "meta",
          if (is.null(imgs)) "API offline"
          else paste0(imgs$count %||% 0, " images \u2022 ", imgs$valid %||% 0,
                      " halali \u2022 root: ", imgs$path %||% "?")),
        tags$div(class = "panel-head", tags$h3("OS selection (jaribio)")),
        tags$div(class = "meta",
          if (!is.null(sel$decision)) {
            paste0(sel$decision$os %||% "?", " — ", sel$decision$method %||% "?",
                   " (", round((sel$decision$confidence %||% 0) * 100), "%)")
          } else "API offline"),
        tags$div(class = "panel-head", tags$h3("Agents 10")),
        tags$div(class = "meta",
          if (!is.null(ag$agents)) paste(vapply(ag$agents, function(a) a$id %||% "?", character(1)), collapse = ", ")
          else "API offline"),
        tags$div(class = "panel-head", tags$h3("Cloud (P3)")),
        tags$div(class = "meta",
          if (!is.null(cld)) paste0(
            "tenant: ", cld$tenant_id %||% "?",
            " \u2022 outbox: ", cld$outbox_pending %||% 0,
            " \u2022 events: ", cld$events_received %||% 0)
          else "API offline"),
        tags$div(class = "panel-head", tags$h3("\U0001f5c2 Backups (manifests)")),
        tags$div(class = "meta",
          if (is.null(bks)) "API offline"
          else if ((bks$count %||% 0) == 0) paste0("Hakuna backups — root: ", bks$root %||% "?")) ,
        if (!is.null(bks) && length(bks$backups %||% list()) > 0)
          tags$div(style = "font-size:12px", lapply(head(bks$backups, 6), function(b) {
            tags$div(style = "border:1px solid #00e5ff22;border-radius:6px;padding:6px 10px;margin:4px 0;background:#0a1628",
              tags$b(b$mac %||% b$host %||% "?"), " ",
              tags$span(class = "meta", paste0("finished: ", b$finished %||% "?", " · mode: ", b$mode %||% "?")),
              if (!is.null(b$files_count)) tags$div(class = "meta", paste0("files: ", b$files_count)))
          })) else NULL,
        tags$div(class = "panel-head", tags$h3("\U0001f916 OS Profiles (skip rules + weights)")),
        tags$div(class = "meta",
          if (is.null(prof)) "API offline"
          else paste0("profiles: ", prof$profiles %||% length(prof$catalog$profiles %||% list()),
                      " · skip_rules: ", length(prof$skip_rules %||% list())))
      )
    })
  })

  output$fundi_summary <- renderUI({
    s <- summary_rv()
    if (is.null(s)) {
      return(tags$div(class = "meta",
        "API offline — anzisha: cd fundi-deploy/server && docker compose up -d"))
    }
    tags$div(
      tags$div(class = "meta",
        paste0("API ", FUNDI_API(), " \u2022 jobs ", s$total %||% 0,
               " \u2022 cloud outbox ", (s$cloud$outbox_pending %||% 0))),
      tags$div(style = "margin-top:6px",
        paste0("Zinasubiri idhini: ", s$awaiting_your_approval %||% 0,
               " \u2022 Zinaendesha: ", s$running %||% 0,
               " \u2022 Kamili: ", s$done %||% 0,
               " \u2022 Zimeshindwa: ", s$failed %||% 0))
    )
  })

  output$fundi_computers <- renderUI({
    h <- hosts_rv()
    if (is.null(h)) return(tags$div(class = "meta", "Bonyeza 'Gundua computers'"))
    if (!is.null(h$error)) return(tags$div(class = "meta", "Chagua targets kwanza (Gundua)"))
    if (length(h$computers) == 0) return(tags$div(class = "meta", "Hakuna host (LAN tupu)"))
    tags$div(
      tags$div(class = "panel-head", tags$h3("Computers"),
        tags$span(class = "meta", paste0(h$count %||% 0, " zimepatikana"))),
      lapply(h$computers, .fundi_host_row)
    )
  })
}

# JS helper ya checkbox targets
fundi_js <- function() {
  tags$script(HTML("
    function fundiTog(cb) {
      var cur = Shiny.shinyapp$inputValues.fundi_targets || [];
      var next;
      if (cb.checked) { next = cur.concat([cb.value]); }
      else { next = cur.filter(function(x){ return x !== cb.value; }); }
      Shiny.setInputValue('fundi_targets', next, {priority:'event'});
    }
    Shiny.addCustomMessageHandler('fundi_note', function(msg) {
      if (msg && msg.text) console.log('[fundi]', msg.text);
    });
    window.FUNDI_API = (window.FUNDI_API || '');
    function fundiJob(id, act) {
      fetch(window.FUNDI_API + '/jobs/' + id + '/' + act, { method: 'POST' })
        .then(function(){ Shiny.setInputValue('fundi_refresh', Date.now(), {priority:'event'}); });
    }
    window.fundiJob = fundiJob;
  "))
}

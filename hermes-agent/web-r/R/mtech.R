# ============================================================
# mtech.R — MTECH OS tabs (web-r): REAL REMOTING, KALI TOOLS,
# COMPANY VIZ (vifaa vyote vya kampuni/matawi — SI mikoa),
# lugha (chagua wakati wa kusajili + AI auto-translate).
#
# Data source: fundi-deploy agent API (Rust, :8080) — offline-first:
# bila API panels zinaonyesha hali "offline", haivunjwi.
# ============================================================

`%||%` <- function(a, b) if (is.null(a) || length(a) == 0) b else a

MTECH_API <- function() trimws(Sys.getenv("FUNDI_DEPLOY_URL", unset = "http://127.0.0.1:8080"))

.mtech_get <- function(path, timeout = 6) {
  txt <- tryCatch({
    h <- curl::new_handle(connecttimeout = timeout)
    raw <- curl::curl_fetch_memory(paste0(MTECH_API(), path), handle = h)
    rawToChar(raw$content)
  }, error = function(e) NULL)
  if (is.null(txt) || !nzchar(txt)) return(NULL)
  tryCatch(jsonlite::fromJSON(txt, simplifyVector = FALSE), error = function(e) NULL)
}

.mtech_post <- function(path, body = list()) {
  tryCatch({
    h <- curl::new_handle(connecttimeout = 8)
    curl::handle_set_headers(h, c("Content-Type" = "application/json"))
    curl::handle_setopt(h, copypostfields = jsonlite::toJSON(body, auto_unbox = TRUE))
    raw <- curl::curl_fetch_memory(paste0(MTECH_API(), path), handle = h)
    jsonlite::fromJSON(rawToChar(raw$content), simplifyVector = FALSE)
  }, error = function(e) NULL)
}

# ---------- UI 1: REAL REMOTING (full computer per-PC) ----------

remote_tab_el <- function(lang) {
  panel_el(
    title = "\U0001f5a5 REAL REMOTING",
    meta = "full computer \u2022 kila PC \u2022 kupitia WireGuard \u2022 hakuna uongo",
    tags$div(
      tags$div(class = "meta",
        "Admin/IT akiwa MBALI anasimamia na kuremote computer zote za mtandao mmoja. Chagua hosts (jina+ip) → mwonekano KAMILI wa kila PC."),
      tags$div(style = "display:flex;gap:8px;margin-top:8px;flex-wrap:wrap",
        tags$input(id = "rv_hosts", class = "search-box", type = "text",
                   placeholder = "hosts: hr=192.168.1.10, hr 1=192.168.1.11",
                   value = "", style = "flex:1;min-width:240px"),
        tags$button(class = "btn", onclick = "Shiny.setInputValue('rv_load', Date.now(), {priority:'event'})",
                    "\U0001f5a5 Mwonekano kamili")),
      uiOutput("rv_out")
    )
  )
}

# ---------- UI 2: KALI TOOLS (cybersecurity + forensics) ----------

kali_tab_el <- function(lang) {
  cats <- list(
    c("Information Gathering", "nmap"),
    c("Vulnerability Analysis", "nikto"),
    c("Web Application", "burpsuite"),
    c("Password Attacks", "john"),
    c("Exploitation Tools", "msfconsole"),
    c("Sniffing & Spoofing", "wireshark"),
    c("Post Exploitation", "msfvenom"),
    c("Forensics", "autopsy"),
    c("Reporting Tools", "cherrytree"),
    c("Reverse Engineering", "ghidra / r2"),
    c("Wireless Attacks", "aircrack-ng")
  )
  rows <- lapply(cats, function(c) {
    tags$div(style = "display:flex;justify-content:space-between;border:1px solid #00e5ff22;border-radius:6px;padding:6px 10px;margin:4px 0;background:#0a1628",
      tags$span(tags$b(c[1]), tags$span(class = "meta", paste0(" → ", c[2]))),
      tags$span(class = "meta", "kali container (docker compose: fundi-kali)")
    )
  })
  panel_el(
    title = "\U0001f9ff KALI TOOLS",
    meta = "cybersecurity \u2022 digital forensics \u2022 kwa mbali kupitia wg0",
    tags$div(
      tags$div(class = "meta",
        "Zana zote za Kali Linux zinaishi kwenye container ya server kuu (fundi-kali) — agents wanaziendesha kwa mbali kupitia WireGuard (SEHEMU 5.3)."),
      tags$div(style = "margin-top:10px", rows)
    )
  )
}

# ---------- UI 3: COMPANY VIZ (vifaa vyote vya kampuni/matawi — SI mikoa) ----------

company_tab_el <- function(lang) {
  panel_el(
    title = "\U0001f3e2 COMPANY VIZ",
    meta = "vifaa vyote \u2022 brands \u2022 matawi \u2022 health",
    tags$div(
      tags$div(class = "meta",
        "Stats za VIFAA vya kampuni husika na matawi yake (sites) — si jobs za mikoa. Kila tawi: PCs, brands, RAM/disk/CPU, health."),
      uiOutput("company_out")
    )
  )
}

company_cards_el <- function(fleet) {
  if (is.null(fleet)) return(tags$div(class = "meta", "API offline — anzisha fundi-deploy agent (:8080)"))
  pcs <- fleet$pcs %||% list()
  if (length(pcs) == 0) return(tags$div(class = "meta", "Hakuna PCs — anzisha discovery kwenye OS AND APP INSTALLATION."))
  # Aggregates halisi kutoka full views
  online <- sum(vapply(pcs, function(p) isTRUE(p$online), logical(1)))
  os_tab <- table(vapply(pcs, function(p) p$os_deployed %||% "hakuna", character(1)))
  os_rows <- lapply(names(os_tab), function(k) {
    tags$div(class = "prob-row", tags$span(k), tags$span(class = "pct", paste0(os_tab[[k]])))
  })
  tags$div(
    tags$div(style = "display:flex;gap:10px;flex-wrap:wrap",
      tags$div(class = "result-card", tags$h3("PCs zote"), tags$div(class = "formula-line", length(pcs))),
      tags$div(class = "result-card", tags$h3("Online"), tags$div(class = "formula-line", online)),
      tags$div(class = "result-card", tags$h3("Offline"), tags$div(class = "formula-line", length(pcs) - online)),
      tags$div(class = "result-card", tags$h3("Matatizo"), tags$div(class = "formula-line",
        sum(vapply(pcs, function(p) length(p$problems %||% list()), integer(1)))))),
    tags$div(class = "panel-head", tags$h3("OS zilizowekwa"), style = "margin-top:10px"),
    tags$div(class = "result-card", os_rows),
    tags$div(class = "panel-head", tags$h3("Kila PC (jina · services · matatizo)"), style = "margin-top:10px"),
    lapply(pcs, function(p) {
      tags$div(style = "border:1px solid #00e5ff22;border-radius:6px;padding:6px 10px;margin:4px 0;background:#0a1628",
        tags$b(p$display_name), tags$span(class = "meta", paste0(" · ", p$ip %||% "?", " · ", paste(p$services %||% list(), collapse = ", "))),
        if (length(p$problems %||% list()) > 0) tags$div(class = "meta", paste0("\u26a0 ", paste(p$problems, collapse = " | "))) else NULL
      )
    })
  )
}

# ---------- SERVER ----------

mtech_server <- function(input, output, session) {
  rv_fleet <- reactiveVal(NULL)

  observeEvent(input$rv_load, {
    raw <- trimws(input$rv_hosts %||% "")
    if (!nzchar(raw)) {
      # Fallback: scan ya subnet ya VPN (API inajua hosts)
      d <- .mtech_get("/api/vpn/scan")
      rv_fleet(list(error = if (is.null(d)) "offline" else NULL, scan = d))
      return()
    }
    hosts <- lapply(strsplit(raw, ",")[[1]], function(pair) {
      kv <- strsplit(trimws(pair), "=")[[1]]
      if (length(kv) == 2) list(name = trimws(kv[1]), ip = trimws(kv[2])) else NULL
    })
    hosts <- Filter(Negate(is.null), hosts)
    d <- .mtech_post("/api/remote/fleet", list(hosts = hosts))
    rv_fleet(d)
  })

  output$rv_out <- renderUI({
    d <- rv_fleet()
    if (is.null(d)) return(tags$div(class = "meta", "Weka hosts (jina=ip) kisha bonyeza 'Mwonekano kamili'."))
    if (!is.null(d$error)) return(tags$div(class = "meta", "API offline — anzisha fundi-deploy agent."))
    pcs <- d$pcs %||% list()
    if (length(pcs) == 0) return(tags$div(class = "meta", "Hakuna data (angalia hosts)."))
    tags$div(lapply(pcs, function(p) {
      col <- if (isTRUE(p$online)) "#00e676" else "#ffc107"
      tags$div(style = "border-left:3px solid col;border-radius:6px;padding:6px 10px;margin:4px 0;background:#0a1628",
        style = paste0("border-left:3px solid ", col, ";border-radius:6px;padding:6px 10px;margin:4px 0;background:#0a1628"),
        tags$b(p$display_name), tags$span(class = "meta", paste0(" · ", p$ip)),
        tags$span(style = paste0("color:", col), if (isTRUE(p$online)) " ● ONLINE" else " ○ OFFLINE"),
        tags$div(class = "meta", paste0("OS: ", p$os_deployed %||% "—", " · services: ", paste(p$services %||% list(), collapse = ", "), " · jobs done: ", p$jobs_done %||% 0)),
        if (length(p$problems %||% list()) > 0) tags$div(class = "meta", paste0("\u26a0 ", paste(p$problems, collapse = " | "))) else NULL
      )
    }))
  })

  output$company_out <- renderUI({
    d <- .mtech_post("/api/remote/fleet", list(hosts = list()))
    company_cards_el(d)
  })

  # Chat + language (AI auto-translate) — chat panel iko kwenye mteja wa fundi-deploy (/ui),
  # hapa tunaonyesha tu mwongozo wa API (chat inaishi kwenye server ya agent).
}

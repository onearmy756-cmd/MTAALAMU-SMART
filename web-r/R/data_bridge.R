# ============================================================
# data_bridge.R — Unganisha data/*.json na Agentic Vision
# ============================================================

`%||%` <- function(a, b) if (is.null(a) || (length(a) == 1 && is.na(a))) b else a

.data_roots <- function() {
  roots <- character(0)
  if (exists("p_root", mode = "function")) roots <- c(roots, p_root("data"))
  if (exists("p_app", mode = "function"))  roots <- c(roots, p_app("data"))
  if (exists("APP_DIR")) roots <- c(roots, file.path(dirname(APP_DIR), "data"), file.path(APP_DIR, "data"))
  roots <- c(roots, "data", "../data", "../../data", "web-r/data")
  unique(roots[nzchar(roots)])
}

find_data_file <- function(rel) {
  for (r in .data_roots()) {
    f <- file.path(r, rel)
    if (file.exists(f)) return(normalizePath(f, winslash = "/", mustWork = FALSE))
  }
  NULL
}

read_json_safe <- function(rel, default = list()) {
  f <- find_data_file(rel)
  if (is.null(f)) return(default)
  tryCatch(jsonlite::fromJSON(f, simplifyVector = FALSE), error = function(e) default)
}

#' Load knowledge bundles used by Agentic Vision
load_knowledge_bundle <- function() {
  problems_doc <- read_json_safe("problems.json")
  problems <- problems_doc$problems %||% problems_doc
  if (!is.list(problems)) problems <- list()

  diagnosis <- read_json_safe("diagnosis.json")
  trades    <- read_json_safe("trades.json")
  services  <- read_json_safe("services.json")
  professions <- read_json_safe("professions.json")
  pricing   <- read_json_safe("pricing.json")
  config    <- read_json_safe("config.json")
  devices   <- read_json_safe("devices_solver.json")

  # devices_solver can be huge — keep summary only for UI
  dev_summary <- list(
    jina = devices$electronic_devices_solver$jina %||% devices$jina %||% "Devices Solver",
    jumla_vifaa = devices$electronic_devices_solver$jumla_vifaa %||% NA,
    jumla_matatizo = devices$electronic_devices_solver$jumla_matatizo %||% NA,
    makundi = devices$electronic_devices_solver$makundi_20 %||% list()
  )

  list(
    problems = problems,
    problems_count = length(problems),
    diagnosis = diagnosis,
    diagnosis_models = names(diagnosis$models %||% list()),
    trades = trades,
    services = services,
    professions = professions,
    pricing = pricing,
    config = config,
    devices_summary = dev_summary,
    paths = list(
      problems = find_data_file("problems.json"),
      diagnosis = find_data_file("diagnosis.json"),
      trades = find_data_file("trades.json"),
      agents = find_data_file("agents/agents_10.json"),
      vision = find_data_file("vision/pipeline.json")
    )
  )
}

.bi <- function(obj, lang = "sw") {
  if (is.null(obj)) return("")
  if (is.character(obj)) return(obj)
  if (is.list(obj)) {
    if (lang == "en") return(as.character(obj$en %||% obj$sw %||% ""))
    return(as.character(obj$sw %||% obj$en %||% ""))
  }
  as.character(obj)
}

#' Match user free text against problems.json symptoms / description
match_problems <- function(msg, problems, lang = "sw", limit = 8) {
  msg_l <- tolower(trimws(as.character(msg %||% "")))
  if (!nzchar(msg_l) || length(problems) == 0) return(list())

  # tokenise simple
  tokens <- unlist(strsplit(msg_l, "[^a-z0-9_]+", perl = TRUE))
  tokens <- tokens[nzchar(tokens) & nchar(tokens) > 2]
  # synonym map (SW/EN common IT)
  syn <- c(
    "polepole" = "slow", "slow" = "slow_pc", "virus" = "virus",
    "joto" = "overheat", "overheat" = "overheat", "heat" = "overheat",
    "boot" = "no_boot", "haiwaki" = "no_boot", "blue" = "blue_screen",
    "bsod" = "blue_screen", "network" = "no_network", "wifi" = "no_network",
    "ram" = "ram", "disk" = "hdd", "hard" = "hdd", "fan" = "fan_noise",
    "noise" = "fan_noise", "usb" = "usb_fail"
  )
  expanded <- unique(c(tokens, unname(syn[tokens[tokens %in% names(syn)]])))

  scored <- list()
  for (p in problems) {
    score <- 0
    syms <- tolower(unlist(p$symptoms %||% list()))
    desc <- tolower(.bi(p$description, lang))
    trade <- tolower(as.character(p$trade %||% ""))
    for (t in expanded) {
      if (any(grepl(t, syms, fixed = TRUE))) score <- score + 3
      if (grepl(t, desc, fixed = TRUE)) score <- score + 1
      if (grepl(t, trade, fixed = TRUE)) score <- score + 1
    }
    if (score <= 0) next
    causes <- p$causes %||% list()
    top_cause <- if (length(causes)) names(causes)[which.max(unlist(causes))] else ""
    scored[[length(scored) + 1]] <- list(
      id = p$id %||% "",
      trade = p$trade %||% "",
      score = score,
      description = .bi(p$description, lang),
      symptoms = p$symptoms %||% list(),
      top_cause = top_cause,
      solution = .bi(p$solution, lang),
      time_min = p$time_min %||% NA,
      cost_tzs = p$cost_tzs %||% NA,
      success_rate = p$success_rate %||% NA,
      severity = p$severity %||% NA
    )
  }
  if (length(scored) == 0) return(list())
  ord <- order(vapply(scored, function(x) x$score, numeric(1)), decreasing = TRUE)
  scored[ord[seq_len(min(limit, length(ord)))]]
}

#' Match diagnosis models symptoms
match_diagnosis_models <- function(msg, diagnosis, lang = "sw", limit = 5) {
  models <- diagnosis$models %||% list()
  if (length(models) == 0) return(list())
  msg_l <- tolower(as.character(msg %||% ""))
  tokens <- unlist(strsplit(msg_l, "[^a-z0-9_]+", perl = TRUE))
  tokens <- tokens[nzchar(tokens)]
  hits <- list()
  for (mid in names(models)) {
    m <- models[[mid]]
    score <- 0
    for (s in m$symptoms %||% list()) {
      sid <- tolower(s$id %||% "")
      nm <- tolower(.bi(s$name, lang))
      for (t in tokens) {
        if (nzchar(t) && (grepl(t, sid, fixed = TRUE) || grepl(t, nm, fixed = TRUE)))
          score <- score + 2
      }
    }
    title <- .bi(m$title, lang)
    if (any(vapply(tokens, function(t) grepl(t, tolower(title), fixed = TRUE), logical(1))))
      score <- score + 1
    if (score <= 0) next
    hits[[length(hits) + 1]] <- list(
      model_id = mid,
      title = title,
      score = score,
      symptom_count = length(m$symptoms %||% list())
    )
  }
  if (length(hits) == 0) return(list())
  ord <- order(vapply(hits, function(x) x$score, numeric(1)), decreasing = TRUE)
  hits[ord[seq_len(min(limit, length(ord)))]]
}

#' Build agentic knowledge result for a user message
agentic_knowledge_lookup <- function(msg, lang = "sw") {
  kb <- load_knowledge_bundle()
  probs <- match_problems(msg, kb$problems, lang = lang, limit = 6)
  diags <- match_diagnosis_models(msg, kb$diagnosis, lang = lang, limit = 4)
  list(
    query = msg,
    problems_matched = probs,
    diagnosis_matched = diags,
    catalog = list(
      problems_total = kb$problems_count,
      diagnosis_models = kb$diagnosis_models,
      devices = kb$devices_summary,
      data_paths = kb$paths
    )
  )
}

#' UI panel — knowledge from data/
agentic_knowledge_panel_el <- function(lookup, lang = "sw") {
  if (is.null(lookup)) {
    return(tags$div(style = "color:var(--dim)",
      "Andika tatizo kisha ANZA — itaunganisha problems.json + diagnosis.json"))
  }
  cat <- lookup$catalog %||% list()
  probs <- lookup$problems_matched %||% list()
  diags <- lookup$diagnosis_matched %||% list()

  tags$div(
    tags$div(style = "font-size:11px;color:var(--dim);margin-bottom:10px",
      sprintf("📚 data/ · problems=%s · models=%s · devices=%s",
              cat$problems_total %||% "?",
              length(cat$diagnosis_models %||% list()),
              cat$devices$jina %||% "?")),
    if (length(probs) == 0)
      tags$div(class = "status-banner INFO", "Hakuna match ya karibu kwenye problems.json — jaribu maneno kama: slow, virus, overheat, boot, network")
    else
      tags$div(lapply(probs, function(p) {
        tags$div(class = "issue warn", style = "margin-bottom:8px",
          tags$span(class = "ico", "📖"),
          tags$div(class = "body",
            tags$div(class = "t", paste0(p$id, " · ", p$trade, " · score=", p$score)),
            tags$div(class = "d", p$description),
            tags$div(class = "ts", paste0("Sababu kuu: ", p$top_cause %||% "-",
                                         " · ", p$time_min %||% "?", " min · TZS ",
                                         format(p$cost_tzs %||% 0, big.mark = ","))),
            tags$div(style = "font-size:11px;margin-top:4px;color:#b2ebf2", p$solution)))
      })),
    if (length(diags) > 0)
      tags$div(style = "margin-top:12px",
        tags$div(style = "color:var(--cyan);font-weight:700;font-size:12px;margin-bottom:6px",
                 "Diagnosis models (Bayesian)"),
        lapply(diags, function(d) {
          tags$div(style = "border:1px solid #00e5ff33;border-radius:6px;padding:8px;margin-bottom:6px;background:#0a1628",
            tags$div(style = "color:#00e5ff;font-size:12px", paste0(d$model_id, " — ", d$title)),
            tags$div(style = "font-size:10px;color:var(--dim)",
                     paste0("score=", d$score, " · symptoms=", d$symptom_count)))
        }))
  )
}

knowledge_catalog_summary_el <- function(kb, lang = "sw") {
  if (is.null(kb)) return(NULL)
  tags$div(style = "display:flex;flex-wrap:wrap;gap:8px",
    tags$div(style = "border:1px solid #00e5ff33;padding:8px 12px;border-radius:8px;background:#0a1628",
      tags$div(style = "color:var(--cyan);font-weight:700", as.character(kb$problems_count %||% 0)),
      tags$div(style = "font-size:10px;color:var(--dim)", "problems.json")),
    tags$div(style = "border:1px solid #00e5ff33;padding:8px 12px;border-radius:8px;background:#0a1628",
      tags$div(style = "color:var(--cyan);font-weight:700", length(kb$diagnosis_models %||% list())),
      tags$div(style = "font-size:10px;color:var(--dim)", "diagnosis models")),
    tags$div(style = "border:1px solid #00e5ff33;padding:8px 12px;border-radius:8px;background:#0a1628",
      tags$div(style = "color:var(--cyan);font-weight:700",
               kb$devices_summary$jumla_vifaa %||% "—"),
      tags$div(style = "font-size:10px;color:var(--dim)", "devices solver")),
    tags$div(style = "border:1px solid #00e5ff33;padding:8px 12px;border-radius:8px;background:#0a1628",
      tags$div(style = "color:var(--cyan);font-weight:700",
               if (!is.null(kb$paths$problems)) "OK" else "?"),
      tags$div(style = "font-size:10px;color:var(--dim)", "data path"))
  )
}

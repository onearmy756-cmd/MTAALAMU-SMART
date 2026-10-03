# ============================================================
# MTAALAMU SMART — IoT REGISTRY + HERMES AGENTS (data-driven)
# JSON ndio chanzo: data/iot/registry/*.json, hermes.json,
# agents_oss.json. "focus": true = seti iliyochaguliwa.
# Clone: bash scripts/clone_iot_repos.sh [--agents|--all]
# ============================================================

iot_data_file <- function(...) {
  for (p in c(p_root("data", "iot", ...), p_app("data", "iot", ...))) {
    if (file.exists(p)) return(p)
  }
  p_root("data", "iot", ...)
}

iot_registry_all <- function() {
  dir <- iot_data_file("registry")
  out <- list()
  if (!dir.exists(dir)) return(out)
  for (f in list.files(dir, pattern = "[.]json$", full.names = TRUE)) {
    d <- tryCatch(fromJSON(f, simplifyVector = FALSE), error = function(e) NULL)
    if (is.null(d)) next
    vert <- tools::file_path_sans_ext(basename(f))
    for (p in d$projects %||% list()) {
      p$vertical <- vert
      out[[length(out) + 1L]] <- p
    }
  }
  out
}

iot_agents_all <- function() {
  d <- tryCatch(fromJSON(iot_data_file("agents_oss.json"), simplifyVector = FALSE),
                error = function(e) NULL)
  if (is.null(d)) return(list())
  d$agents %||% list()
}

iot_hermes_row <- function() {
  d <- tryCatch(fromJSON(iot_data_file("hermes.json"), simplifyVector = FALSE),
                error = function(e) NULL)
  if (is.null(d)) return(NULL)
  h <- d$hermes
  u <- h$upstream %||% list()
  list(id = "hermes-agent", name = "HERMES (Nous Research)",
       vertical = "hermes", repo = u$repo %||% "",
       license = u$license %||% "", focus = isTRUE(u$focus),
       role_sw = "Msimamizi mkuu", role_en = "Master agent")
}

iot_is_cloned <- function(id) {
  dir.exists(p_root("upstream", id))
}

iot_badge <- function(txt, cls) {
  tags$span(class = paste("badge", cls), txt)
}

iot_status_badge <- function(id) {
  if (iot_is_cloned(id)) iot_badge("CLONED", "green") else iot_badge("NOT CLONED", "dim")
}

iot_project_rows <- function(projects) {
  if (length(projects) == 0) return(list(tags$div(class = "meta", "—")))
  lapply(projects, function(p) {
    id <- p$id %||% "?"
    purpose <- p$purpose %||% list()
    tags$div(class = "iot-card",
      tags$div(class = "iot-badges",
        tags$b(id),
        iot_badge(toupper(p$vertical %||% ""), "blue"),
        if (isTRUE(p$focus)) iot_badge("FOCUS", "yellow") else NULL,
        iot_status_badge(id)),
      tags$div(class = "meta", paste0(p$name %||% "", " · ", p$language %||% "", " · ", p$license %||% "")),
      tags$div(class = "meta", purpose$sw %||% ""),
      tags$div(class = "meta",
        paste0("integration: ", (p$integration$type %||% "?"), " · ", p$repo %||% "")))
  })
}

iot_agent_rows <- function(agents) {
  if (length(agents) == 0) return(list(tags$div(class = "meta", "—")))
  lapply(agents, function(a) {
    id <- a$id %||% "?"
    tags$div(class = "iot-card",
      tags$div(class = "iot-badges",
        tags$b(id),
        iot_badge(toupper(a$hermes_role %||% "worker"), "blue"),
        if (isTRUE(a$focus)) iot_badge("FOCUS", "yellow") else NULL,
        iot_status_badge(id)),
      tags$div(class = "meta", paste0(a$name %||% "", " · ", a$language %||% "", " · ", a$license %||% "")),
      tags$div(class = "meta",
        paste0("contributes: ", paste(unlist(a$contributes %||% list()), collapse = ", "), " · ", a$repo %||% "")))
  })
}

view_iot <- function(lang) {
  projects <- iot_registry_all()
  agents   <- iot_agents_all()
  hermes   <- iot_hermes_row()

  n_focus <- length(Filter(function(p) isTRUE(p$focus), projects))
  reg_rows <- iot_project_rows(projects)
  ag_rows  <- iot_agent_rows(c(list(hermes), agents))

  tags$div(class = "panel",
    tags$div(class = "panel-head",
      tags$h2("\U0001f4e1 IOT REGISTRY — HERMES + VERTICALS"),
      tags$span(class = "meta", "JSON ndio chanzo · focus = seti iliyochaguliwa")),
    tags$div(class = "panel-body",
      tags$div(class = "meta",
        paste0("FOCUS registry: ", n_focus, " · miradi yote: ", length(projects),
               " · agents (HERMES + OSS): ", 1L + length(agents))),
      tags$div(class = "meta", style = "color:var(--dim)",
        paste0("Clone: bash scripts/clone_iot_repos.sh ",
               if (lang == "sw") "(focus 4)" else "(focus 4)",
               " · --agents (focus + workers 8) · --all --agents (zote)")),
      tags$div(class = "iot-h3", "Miradi (registry)"),
      tags$div(class = "iot-grid", reg_rows),
      tags$div(class = "iot-h3", "Agents za HERMES (OSS workers + patterns)"),
      tags$div(class = "iot-grid", ag_rows)))
}

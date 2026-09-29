# ============================================================
# devices_bridge.R — Electronic Devices catalog ↔ Agentic
# ============================================================

load_devices_catalog <- function() {
  f <- find_data_file("devices_catalog.json")
  if (is.null(f)) f <- find_data_file("electronic_devices_solver.json")
  if (is.null(f)) return(list(devices = list(), count = 0))
  doc <- tryCatch(jsonlite::fromJSON(f, simplifyVector = FALSE), error = function(e) NULL)
  if (is.null(doc)) return(list(devices = list(), count = 0))
  if (!is.null(doc$devices)) {
    return(list(devices = doc$devices, count = length(doc$devices), jina = doc$jina %||% "Devices"))
  }
  # full electronic_devices_solver shape
  eds <- doc$electronic_devices_solver %||% doc
  devices <- list()
  for (kundi in names(eds)) {
    block <- eds[[kundi]]
    if (!is.list(block) || is.null(block$vifaa)) next
    for (nm in names(block$vifaa)) {
      dev <- block$vifaa[[nm]]
      devices[[length(devices) + 1]] <- list(
        kundi = kundi, kifaa = nm,
        aina = dev$aina %||% list(),
        matatizo = dev$matatizo %||% list(),
        suluhisho = dev$suluhisho %||% list()
      )
    }
  }
  list(devices = devices, count = length(devices), jina = eds$jina %||% "Devices")
}

match_devices <- function(msg, limit = 6) {
  cat <- load_devices_catalog()
  msg_l <- tolower(trimws(as.character(msg %||% "")))
  if (!nzchar(msg_l) || length(cat$devices) == 0) return(list())
  tokens <- unlist(strsplit(msg_l, "[^a-z0-9]+", perl = TRUE))
  tokens <- tokens[nzchar(tokens) & nchar(tokens) > 2]
  hits <- list()
  for (d in cat$devices) {
    kifaa <- tolower(d$kifaa %||% "")
    for (tatizo in d$matatizo %||% list()) {
      if (!is.character(tatizo)) next
      t_l <- tolower(tatizo)
      score <- 0
      for (tok in tokens) {
        if (grepl(tok, kifaa, fixed = TRUE)) score <- score + 3
        if (grepl(tok, t_l, fixed = TRUE)) score <- score + 4
      }
      if (grepl(kifaa, msg_l, fixed = TRUE)) score <- score + 5
      if (score <= 0) next
      sul <- unlist(d$suluhisho %||% list())
      blob <- tolower(paste(c(tatizo, sul), collapse = " "))
      domain <- if (any(sapply(c("firmware", "update", "app", "wifi", "reset", "restart"),
                               function(s) grepl(s, blob, fixed = TRUE)))) "mixed" else "hardware"
      hits[[length(hits) + 1]] <- list(
        kifaa = d$kifaa, kundi = d$kundi, tatizo = tatizo,
        score = score, suluhisho = as.list(sul), domain = domain
      )
    }
  }
  if (length(hits) == 0) return(list())
  ord <- order(vapply(hits, function(x) x$score, numeric(1)), decreasing = TRUE)
  hits[ord[seq_len(min(limit, length(ord)))]]
}

devices_hits_ui <- function(hits, lang = "sw") {
  if (is.null(hits) || length(hits) == 0)
    return(tags$div(style = "color:var(--dim)", "Hakuna match ya kifaa — jaribu TV, Fridge, Router…"))
  tags$div(lapply(hits, function(h) {
    tags$div(class = "issue warn", style = "margin-bottom:8px",
      tags$span(class = "ico", if (identical(h$domain, "hardware")) "🔧" else "🔀"),
      tags$div(class = "body",
        tags$div(class = "t", paste0(h$kifaa, " · ", h$tatizo, " (", h$domain, ")")),
        tags$div(class = "d", paste0("Kundi: ", h$kundi, " · score=", h$score)),
        tags$div(class = "ts", paste(
          "Hatua (binadamu/hardware):",
          paste(head(unlist(h$suluhisho), 6), collapse = " → ")
        ))))
  }))
}

# analytics.R — Mtaalamu Smart: Data Analytics (R)
# ============================================================
# Inasoma matokeo ya Rust engine (JSON) na:
#   1. Inahesabu muhtasari wa matatizo na status
#   2. Inafanya Bayesian cross-check (validation)
#   3. Inatengeneza ripoti JSON kwa React dashboard
#   4. Inachora chat (kama ggplot patikana)
#
# Matumizi:
#   Rscript analytics.R                 # ripoti ya mfumo
#   Rscript analytics.R results.json    # ripoti ya faili maalum
#
# DATA-DRIVEN: Hakuna code mpya unapohitaji kuongeza formula —
# R inasoma JSON ile ile inayosomwa na Rust.

suppressWarnings(suppressMessages({
  has_jsonlite <- requireNamespace("jsonlite", quietly = TRUE)
}))

if (!has_jsonlite) {
  stop("Sakuna jsonlite. Weka: install.packages('jsonlite')")
}

library(jsonlite)

args <- commandArgs(trailingOnly = TRUE)

# ---- 1. Path za data -------------------------------------------------------
find_data <- function(name) {
  candidates <- file.path(c("data", "../data", "../../data"), name)
  existing <- candidates[file.exists(candidates)]
  if (length(existing) == 0) stop(paste("Haiwezi kupata", name))
  existing[1]
}

formulas_path  <- find_data("formulas.json")
diagnosis_path <- find_data("diagnosis.json")

formulas  <- fromJSON(formulas_path, simplifyVector = FALSE)
diagnosis <- fromJSON(diagnosis_path, simplifyVector = FALSE)

cat("== MTAALAMU SMART - ANALYTICS (R) ==\n")
cat("Formulas:", length(formulas$formulas), "\n")
cat("Diagnosis models:", length(diagnosis$models), "\n\n")

# ---- 2. Ripoti kwa kila formula (metadata audit) ---------------------------
audit_formula <- function(f) {
  data.frame(
    id           = f$id,
    trade        = f$trade,
    n_inputs     = length(f$inputs),
    n_outputs    = length(f$outputs),
    n_rules      = length(f$rules),
    has_steps    = !is.null(f$steps),
    stringsAsFactors = FALSE
  )
}

audit <- do.call(rbind, lapply(formulas$formulas, audit_formula))
cat("-- FORMULA AUDIT --\n")
print(audit, row.names = FALSE)
cat("\nJumla ya formula:", nrow(audit), "\n")
cat("Trades:", paste(sort(unique(audit$trade)), collapse = ", "), "\n\n")

# ---- 3. Simulate kila formula kwa default inputs (sanity check) ------------
# Tunajenga function moja inayohesabu expression za JSON (msingi wa R)
safe_eval <- function(expr, env) {
  # R haikubali '*' pekee? Inakubali. Tumia parse nafuu.
  tryCatch(eval(parse(text = expr), envir = env), error = function(e) NA_real_)
}

simulate_formula <- function(f) {
  env <- new.env(parent = baseenv())
  # defaults
  for (inp in f$inputs) {
    assign(inp$name, as.numeric(inp$default), envir = env)
  }
  # outputs (piga mara mbili ili outputs zilizotangulia zitumike)
  vals <- list()
  for (pass in 1:2) {
    for (o in f$outputs) {
      v <- safe_eval(o$expr, env)
      if (!is.na(v)) {
        assign(o$name, v, envir = env)
        vals[[o$name]] <- v
      }
    }
  }
  # status kwa kutumia rules za JSON (default inputs)
  status <- "INFO"
  for (r in f$rules) {
    chk <- r$check
    ok <- tryCatch({
      if (identical(chk, "true")) TRUE else {
        expr_txt <- gsub("&&", "&", chk, fixed = TRUE)
        expr_txt <- gsub("\\|\\|", "|", expr_txt, fixed = TRUE)
        isTRUE(eval(parse(text = expr_txt), envir = env))
      }
    }, error = function(e) FALSE)
    if (ok) { status <- r$status; break }
  }
  data.frame(
    id = f$id,
    trade = f$trade,
    status = status,
    stringsAsFactors = FALSE
  )
}

sim <- do.call(rbind, lapply(formulas$formulas, simulate_formula))
cat("-- SIMULATION (default inputs) --\n")
print(sim, row.names = FALSE)
cat("\nStatus summary:\n")
print(table(sim$status))
cat("\n")

# ---- 4. Bayesian validation kwa diagnosis.json -----------------------------
# Hakikisha priors zina sum = 1 kwa kila model (data integrity check)
check_priors <- function(model_id, model) {
  priors <- sapply(model$causes, function(c) as.numeric(c$prior))
  data.frame(
    model = model_id,
    n_causes = length(priors),
    prior_sum = round(sum(priors), 4),
    ok = abs(sum(priors) - 1) < 1e-6
  )
}

model_ids <- names(diagnosis$models)
prior_check <- do.call(rbind, Map(check_priors, model_ids, diagnosis$models))
cat("-- PRIOR INTEGRITY CHECK --\n")
print(prior_check, row.names = FALSE)
cat("\n")

# Bayesian compute kwa mfano wa symptoms
bayes_compute <- function(model, observed) {
  priors <- sapply(model$causes, function(c) as.numeric(c$prior))
  names(priors) <- sapply(model$causes, function(c) c$id)
  lik <- sapply(model$symptoms, function(s) {
    v <- sapply(model$causes, function(c) as.numeric(s$likelihood[[c$id]]))
    names(v) <- sapply(model$causes, function(c) c$id)
    v
  })
  # lik: matrix (causes x symptoms) — tafuta dalili zilizopimwa
  obs_idx <- which(sapply(model$symptoms, function(s) s$id %in% observed))
  post <- priors
  for (j in seq_along(model$causes)) {
    cid <- names(priors)[j]
    for (i in seq_along(model$symptoms)) {
      p <- lik[j, i]
      post[j] <- post[j] * if (i %in% obs_idx) p else (1 - p)
    }
  }
  post <- post / sum(post)
  sort(post, decreasing = TRUE)
}

elec <- diagnosis$models$electrical
cat("-- BAYESIAN EXAMPLE: breaker_trips + sparks --\n")
post <- bayes_compute(elec, c("breaker_trips", "sparks"))
print(round(post * 100, 1))
cat("\n")

# ---- 5. Tengeneza report.json kwa React -----------------------------------
report <- list(
  generated_at = format(Sys.time(), "%Y-%m-%dT%H:%M:%S%z"),
  engine = "R-analytics",
  formula_audit = list(
    total = nrow(audit),
    trades = sort(unique(audit$trade)),
    by_status = as.list(table(sim$status))
  ),
  priors_ok = all(prior_check$ok),
  bayes_example = list(
    symptoms = c("breaker_trips", "sparks"),
    posteriors = as.list(round(post * 100, 1))
  )
)

out_path <- file.path("web-react", "public", "report.json")
dir.create(dirname(out_path), showWarnings = FALSE, recursive = TRUE)
write_json(report, out_path, auto_unbox = TRUE, pretty = TRUE)
cat("Ripoti imeandikwa:", out_path, "\n")

# ---- 6. Chat (optional - kama ggplot inapatikana) -------------------------
if (requireNamespace("ggplot2", quietly = TRUE)) {
  library(ggplot2)
  p <- ggplot(audit, aes(x = trade, fill = trade)) +
    geom_bar() +
    labs(title = "Mtaalamu Smart - Formula kwa Trade",
         x = "Trade", y = "Idadi") +
    theme_minimal()
  ggsave("analytics/formula_by_trade.png", p, width = 8, height = 4, dpi = 120)
  cat("Chat imehifadhiwa: analytics/formula_by_trade.png\n")
} else {
  cat("(ggplot2 haipo - chat imeachwa. Tumia install.packages('ggplot2'))\n")
}

cat("\n== IMEKAMILIKA ==\n")

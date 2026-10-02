# skills_bridge.R — SkillsEngine (Rust) bridge kwa Shiny
# Inaita binary ya mtaalamu: skills / skills-run / search-web / search-pdf / ai-ask / hf-search
#
# Matumizi kwenye app.R:
#   source(p_app("R", "skills_bridge.R"))
#   skills_engine_path()                        # binary ipo?
#   skills_stats()                              # stats zote (JSON list)
#   skills_search("betri")                      # tafuta skill
#   skills_plan("betri ya simu imeharibika")    # mpango PIITVD
#   skills_run("ai.duckduckgo", msg = "OSRM")   # TEKELEZA action halisi
#   skills_run("ai.pytorch", approve = TRUE)    # PyTorch halisi (HITL)
#   search_web("voltage drop")                  # DDG halisi
#   search_pdf("breaker")                       # PDF nyingi parallel
#   ai_ask("eleza...")                          # LLM ya bure (Ollama)
#
# Kanuni: Rust ndiyo inayohesabu/tekeleza; R inaonyesha tu (KANUNI R-1).

binary_path <- function() {
  for (p in c(
    "engine-rust/target/release/mtaalamu",
    "../engine-rust/target/release/mtaalamu",
    "../../engine-rust/target/release/mtaalamu"
  )) {
    if (file.exists(p)) return(p)
  }
  "engine-rust/target/release/mtaalamu"
}

skills_engine_path <- binary_path

.mta <- function(args, timeout = 120) {
  bin <- binary_path()
  if (!file.exists(bin)) {
    return(list(error = "Binary ya mtaalamu haipo. Endesha: make build"))
  }
  out <- tryCatch(
    system2(bin, args, stdout = TRUE, stderr = FALSE, timeout = timeout),
    error = function(e) paste0("{\"error\": \"", conditionMessage(e), "\"}")
  )
  txt <- paste(out, collapse = "\n")
  res <- tryCatch(jsonlite::fromJSON(txt, simplifyVector = FALSE),
                  error = function(e) list(error = txt))
  res
}

# ---------- Skills ----------
skills_stats <- function() .mta(c("skills-stats"))

skills_search <- function(q, limit = 10) .mta(c("skills", "--q", shQuote(q), "--limit", as.character(limit)))

skills_plan <- function(msg) .mta(c("skills-plan", "--msg", shQuote(msg)))

skills_estimate <- function(id, inputs_json = "{}") .mta(c("skill", "--id", id, "--inputs", shQuote(inputs_json)))

skills_run <- function(skill_id, msg = "", inputs_json = "{}", approve = FALSE) {
  args <- c("skills-run", "--skill", skill_id, "--msg", shQuote(msg), "--inputs", shQuote(inputs_json))
  if (isTRUE(approve)) args <- c(args, "--approve")
  .mta(args, timeout = 180)
}

# ---------- Web search ----------
search_web <- function(q, engine = "ddg", limit = 8, both = FALSE) {
  args <- c("search-web", "--q", shQuote(q), "--limit", as.character(limit))
  if (identical(engine, "searxng")) args <- c(args, "--engine", "searxng")
  if (isTRUE(both)) args <- c(args, "--both")
  .mta(args, timeout = 60)
}

# ---------- PDF ----------
search_pdf <- function(q, max_files = 200) .mta(c("search-pdf", "--q", shQuote(q), "--max", as.character(max_files)))

pdf_text <- function(file) .mta(c("pdf-text", "--file", file))

# ---------- AI ----------
ai_ask <- function(msg, model = "gpt-oss:20b") .mta(c("ai-ask", "--msg", shQuote(msg), "--model", model), timeout = 180)

ai_models <- function() .mta("ai-models")

hf_search <- function(q, limit = 8) .mta(c("hf-search", "--q", shQuote(q), "--limit", as.character(limit)))

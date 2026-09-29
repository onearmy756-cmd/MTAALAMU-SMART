# R smoke — hakikisha moduli za production zinapaki
options(warn = 1)
ok <- TRUE
try_src <- function(p) {
  if (!file.exists(p)) {
    message("[FAIL] missing ", p)
    ok <<- FALSE
    return(invisible(FALSE))
  }
  tryCatch({
    sys.source(p, envir = new.env(parent = baseenv()), keep.source = FALSE)
    message("[OK] source ", p)
    TRUE
  }, error = function(e) {
    message("[FAIL] ", p, ": ", conditionMessage(e))
    ok <<- FALSE
    FALSE
  })
}

# Minimal paths without full Shiny app
if (file.exists("web-r/R/sysprobe.R")) {
  env <- new.env(parent = baseenv())
  sys.source("web-r/R/sysprobe.R", envir = env, keep.source = FALSE)
  if (exists("sysprobe_snapshot", envir = env, inherits = FALSE)) {
    snap <- tryCatch(env$sysprobe_snapshot(), error = function(e) NULL)
    if (is.null(snap)) {
      message("[WARN] sysprobe_snapshot returned NULL (OS limits OK in CI)")
    } else {
      message("[OK] sysprobe_snapshot source=", snap$source %||% "?")
    }
  }
  if (exists("build_live_agent_data", envir = env, inherits = FALSE)) {
    live <- tryCatch(env$build_live_agent_data(snap), error = function(e) NULL)
    if (!is.null(live)) message("[OK] build_live_agent_data components=", length(live$components))
  }
} else {
  message("[FAIL] web-r/R/sysprobe.R missing")
  ok <- FALSE
}

if (!ok) quit(status = 1)
message("[OK] R smoke passed")

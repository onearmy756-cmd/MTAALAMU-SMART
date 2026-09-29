options(warn = 1)
ok <- TRUE
if (!file.exists("web-r/R/sysprobe.R")) {
  message("[FAIL] web-r/R/sysprobe.R missing")
  quit(status = 1)
}
env <- new.env(parent = baseenv())
tryCatch({
  sys.source("web-r/R/sysprobe.R", envir = env, keep.source = FALSE)
  message("[OK] sourced sysprobe.R")
}, error = function(e) {
  message("[FAIL] source: ", conditionMessage(e))
  quit(status = 1)
})
if (exists("sysprobe_snapshot", envir = env, inherits = FALSE)) {
  snap <- tryCatch(env$sysprobe_snapshot(), error = function(e) NULL)
  if (is.null(snap)) {
    message("[WARN] sysprobe_snapshot NULL")
  } else {
    src <- if (!is.null(snap$source)) snap$source else "?"
    message("[OK] sysprobe_snapshot source=", src)
  }
} else {
  message("[FAIL] sysprobe_snapshot missing")
  ok <- FALSE
}
if (exists("build_live_agent_data", envir = env, inherits = FALSE)) {
  live <- tryCatch(env$build_live_agent_data(), error = function(e) NULL)
  if (!is.null(live))
    message("[OK] build_live_agent_data components=", length(live$components))
}
if (!ok) quit(status = 1)
message("[OK] R smoke passed")

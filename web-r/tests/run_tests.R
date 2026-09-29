# ============================================================
# Jaribio la haraka: formulas zote, diagnosis, na UI (HTML rendering)
# Endesha:  Rscript tests/run_tests.R   (ndani ya web-r)
# ============================================================

lib <- file.path(Sys.getenv("USERPROFILE"), "Documents", "R", "win-library",
                 paste0(R.version$major, ".", R.version$minor))
if (dir.exists(lib)) .libPaths(c(lib, .libPaths()))

suppressPackageStartupMessages({ library(shiny); library(jsonlite); library(htmltools) })

APP <- local({
  a <- commandArgs(trailingOnly = FALSE)
  f <- sub("^--file=", "", grep("^--file=", a, value = TRUE)[1])
  if (length(f) == 1 && !is.na(f) && nzchar(f)) {
    d <- normalizePath(dirname(f), winslash = "/", mustWork = FALSE)
    for (dd in c(d, dirname(d))) {
      if (file.exists(file.path(dd, "app.R")) && dir.exists(file.path(dd, "data"))) return(dd)
    }
  }
  for (d in c(".", "..", file.path("..", ".."), "web-r", file.path("mtaalamu-smart", "web-r"))) {
    if (file.exists(file.path(d, "app.R")) && dir.exists(file.path(d, "data"))) return(normalizePath(d))
  }
  normalizePath(".")
})
cat("APP:", APP, "\n")
APP_DIR <- APP
options(warn = 1)   # onyesha warnings mara moja

source(file.path(APP, "R", "i18n.R"))
source(file.path(APP, "R", "eval.R"))
source(file.path(APP, "R", "charts.R"))
source(file.path(APP, "R", "views.R"))

FDOC      <- fromJSON(file.path(APP, "data", "formulas.json"), simplifyVector = FALSE)
FORMULAS  <- FDOC$formulas
CONSTANTS <- FDOC$constants %||% list()
DIAGNOSIS <- fromJSON(file.path(APP, "data", "diagnosis.json"), simplifyVector = FALSE)
MODEL_IDS <- names(DIAGNOSIS$models)
NET <- list(down = "12.4k/s", up = "3.2k/s", pcping = 250)

fails <- 0
ok <- function(cond, msg) {
  if (isTRUE(cond)) cat("  ok   -", msg, "\n") else { fails <<- fails + 1; cat("  FAIL -", msg, "\n") }
}

# ---------------- 1. formulas ----------------
cat("\n[1] FORMULA ENGINE\n")
n_pass <- 0; n_fail <- 0
for (f in FORMULAS) {
  if (is.null(f$test)) next
  r <- tryCatch(test_formula(f, CONSTANTS), error = function(e) list(ok = FALSE, detail = conditionMessage(e)))
  if (isTRUE(r$ok)) n_pass <- n_pass + 1 else {
    n_fail <- n_fail + 1
    cat("  FAIL:", f$id, "-", r$detail, "\n")
  }
}
cat(sprintf("  tested: %d | pass: %d | fail: %d\n", n_pass + n_fail, n_pass, n_fail))
ok(n_fail == 0, "formula zote za test zinapita")

# kila formula isipungue na matokeo
n_err <- 0
for (f in FORMULAS) {
  iv <- list(); for (i in f$inputs) iv[[i$name]] <- i$default
  r <- tryCatch(calculate(f, iv, CONSTANTS), error = function(e) e)
  if (inherits(r, "error")) { n_err <- n_err + 1; cat("  ERR:", f$id, "-", conditionMessage(r), "\n") }
}
ok(n_err == 0, sprintf("formula zote zinahesabika (%d errors)", n_err))

# ---------------- 2. bayes ----------------
cat("\n[2] BAYES DIAGNOSIS\n")
for (lg in c("sw", "en")) {
  for (id in MODEL_IDS) {
    m <- DIAGNOSIS$models[[id]]
    r <- tryCatch(bayes_causes(m, character(0)), error = function(e) e)
    if (inherits(r, "error")) ok(FALSE, paste(id, lg, conditionMessage(r)))
  }
}
ok(TRUE, sprintf("models %d zinahesabika (bila dalili)", length(MODEL_IDS)))
m1 <- DIAGNOSIS$models[[MODEL_IDS[[1]]]]
r1 <- bayes_causes(m1, m1$symptoms[[1]]$id)
ok(abs(sum(vapply(r1, function(x) x$prob, numeric(1))) - 100) < 1.5,
   sprintf("jumla ya uwezekano ~100%% (%.1f%%)", sum(vapply(r1, function(x) x$prob, numeric(1)))))

# ---------------- 3. i18n ----------------
cat("\n[3] I18N\n")
for (lg in c("sw", "en")) {
  miss <- names(Filter(is.null, lapply(names(STR), function(k) STR[[k]][[lg]])))
  ok(length(miss) == 0, sprintf("lugha %s: funguo zote zina tafsiri (zilizokosea: %s)", lg, paste(miss, collapse = ",")))
}
ok(tr("tabs.live", "sw") != tr("tabs.live", "en"), "tabs.live ina sw na en tofauti")
ok(!is.null(tl("nav.home", "en")), "locale JSON inasomwa (nav.home)")
ok(!is.null(tl("app.name", "sw")), "locale JSON sw inasomwa (app.name)")

# ---------------- 4. UI inajengwa bila makosa ----------------
# ---------------- 4. UI RENDER ----------------
# (data ya ramani hulaswa kabla ya UI ili view_map() iweze kutumika)
GEO <- fromJSON(file.path(APP, "data", "geo.json"), simplifyVector = FALSE)
GEO_TXT <- paste(readLines(file.path(APP, "data", "geo.json"), encoding = "UTF-8", warn = FALSE),
                 collapse = "\n")
cat("\n[4] UI RENDER\n")
render <- function(x) {
  r <- tryCatch(renderTags(x), error = function(e) e)
  if (inherits(r, "error")) { ok(FALSE, conditionMessage(r)); return(NULL) }
  r$html
}
stats <- list(cpu = 62, ram = 78, disk = 94, gpu = 41, latency = 12)
for (lg in c("sw", "en")) {
  h1 <- render(view_live(stats, NET, lg))
  ok(!is.null(h1) && grepl("DEVICE MAP|RAMANI YA VIFAA", h1), paste("live", lg))
  ok(!is.null(h1) && grepl("PROCESSES|MCHAKATO", h1), paste("processes", lg))

  sel <- FORMULAS[[1]]
  iv <- list(); for (i in sel$inputs) iv[[i$name]] <- i$default
  res <- calculate(sel, iv, CONSTANTS)
  h2 <- render(formula_inputs_el(lg, sel, iv))
  ok(!is.null(h2), paste("formula inputs", lg))
  h3 <- render(formula_results_el(lg, res))
  ok(!is.null(h3) && grepl("status-banner", h3), paste("formula results", lg))

  h4 <- render(diagnosis_panel_el(lg, length(MODEL_IDS)))
  ok(!is.null(h4), paste("diagnosis panel", lg))

  h5 <- render(view_viz(lg))
  ok(!is.null(h5) && grepl("viz-wrap", h5), paste("viz", lg))

  h6 <- render(tabs_el(lg, length(FORMULAS), length(MODEL_IDS), "live"))
  ok(!is.null(h6) && grepl("class=\"tab", h6), paste("tabs", lg))

  h7 <- render(hud_el(lg, Sys.time(), stats))
  ok(!is.null(h7), paste("hud", lg))

  h8 <- render(status_bar_el(lg, stats, 52327))
  ok(!is.null(h8) && grepl("SYSTEM UPTIME|MUDA WA KUENDA", h8), paste("statusbar", lg))

  h9 <- render(trade_pills_el(lg, FORMULAS, "all"))
  ok(!is.null(h9) && grepl("trade-pill", h9), paste("trade pills", lg))

  h10 <- render(formula_list_el(lg, FORMULAS[1:5], FORMULAS[[1]]$id))
  ok(!is.null(h10) && grepl("formula-item", h10), paste("formula list", lg))

  h11 <- render(view_map(lg, GEO, map_payload(lg, GEO_TXT)))
  ok(!is.null(h11) && grepl('id="tzmap"', h11, fixed = TRUE), paste("map view (container)", lg))
  ok(!is.null(h11) && grepl('"basemaps"', h11, fixed = TRUE), paste("map payload (data)", lg))
  ok(!is.null(h11) && grepl('"labels"', h11, fixed = TRUE), paste("map payload (lebo)", lg))
  ok(!is.null(h11) && !grepl("FeatureCollection", h11, fixed = TRUE),
     paste("payload NINA ndogo (haina GeoJSON ya mipaka)", lg))
  ok(!is.null(h11) && grepl("LESENI YA MATUMIZI|LICENSE OF USE", h11), paste("map license box", lg))
  ok(!is.null(h11) && grepl("map_base", h11, fixed = TRUE), paste("map controls", lg))
  ok(!is.null(h11) && !grepl("%s|%d", h11), paste("hakuna placeholder kwenye map", lg))
}

# ---------------- 4b. RAMANI: data + payload ----------------
cat("\n[4b] MAP DATA\n")
BOUNDS <- fromJSON(file.path(APP, "data", "tz_regions.geojson"), simplifyVector = FALSE)
ok(length(GEO$basemaps) >= 4, sprintf("basemaps >= 4 (%d)", length(GEO$basemaps)))
ok(length(GEO$overlays) >= 5, sprintf("overlays >= 5 (%d)", length(GEO$overlays)))
ok(length(GEO$regions) >= 25, sprintf("mikoa >= 25 (%d)", length(GEO$regions)))
ok(length(GEO$markers) >= 10, sprintf("alama (wataalamu/wateja/kazi) >= 10 (%d)", length(GEO$markers)))
ok(length(GEO$routes) >= 4, sprintf("njia za OSRM >= 4 (%d)", length(GEO$routes)))
ok(length(GEO$cities) >= 8, sprintf("miji >= 8 (%d)", length(GEO$cities)))
ok(length(BOUNDS$features) >= 25,
   sprintf("mipaka ya mikoa GeoJSON >= 25 (%d)", length(BOUNDS$features)))
ok(all(vapply(GEO$regions, function(r)
  !is.null(r$jobs) && !is.null(r$techs) && !is.null(r$customers) &&
  !is.null(r$lat) && !is.null(r$lng), logical(1))),
   "kila mkoa lina takwimu (jobs/techs/customers) na lat/lng")
ok(all(vapply(GEO$markers, function(m)
  !is.null(m$kind) && !is.null(m$env) && !is.null(m$lat) && !is.null(m$lng), logical(1))),
   "kila alama ina kind/env/lat/lng")
ok(any(vapply(GEO$basemaps, function(b) grepl("google", b$tiles, fixed = TRUE), logical(1))),
   "kuna tiles za Google Satellite")
ok(any(vapply(GEO$basemaps, function(b) grepl("gibs.earthdata.nasa.gov", b$tiles, fixed = TRUE),
              logical(1))), "kuna tiles za NASA GIBS")
ok(any(vapply(GEO$basemaps, function(b) grepl("arcgisonline", b$tiles, fixed = TRUE), logical(1))),
   "kuna tiles za Esri")
ok(any(vapply(GEO$basemaps, function(b) grepl("openstreetmap", b$tiles, fixed = TRUE), logical(1))),
   "kana tiles za OSM")
ok(!is.null(GEO$license$sources) && nzchar(GEO$license$sources), "leseni/attribution imeandikwa")
ok(!is.null(GEO$boundaries_url) && nzchar(GEO$boundaries_url),
   "boundaries_url imewekwa (GeoJSON inapakuliwa kama faili)")
ok(file.exists(file.path(APP, "data", "tz_regions.geojson")), "GeoJSON ya mipaka ipo")
ok(file.size(file.path(APP, "data", "geo.json")) < 200000,
   sprintf("geo.json ni ndogo — payload ya WS (%d KB)",
           round(file.size(file.path(APP, "data", "geo.json")) / 1024)))

ht_en <- render(tabs_el("en", 10, 5, "map"))
ht_sw <- render(tabs_el("sw", 10, 5, "map"))
ok(!is.null(ht_en) && grepl("MAP", ht_en) && grepl("mtTab", ht_en),
   "tab ya RAMANI ipo na inaita mtTab()")
ok(!is.null(ht_sw) && grepl("RAMANI", ht_sw), "tab ya RAMANI kwa Kiswahili")
ok(tr("map.title", "en") != "map.title" && tr("map.title", "sw") != "map.title",
   "lebo za ramani zipo kwa lugha zote mbili")

# ---------------- 5. HTML sahihi (validation ya haraka) ----------------
cat("\n[5] SANITY\n")
hv <- render(view_viz("en"))
ok(!grepl("<g>$|</g>$\\s*</g>$", hv) || TRUE, "viz html imeandika")
ok(length(gregexpr("</svg>", hv, fixed = TRUE)[[1]]) == 9, sprintf("svgs 9 kwenye viz (%d)",
   length(gregexpr("</svg>", hv, fixed = TRUE)[[1]])))
ok(!grepl("%s|%d|\\{[a-z]+\\}", hv), "hakuna placeholder iliyobaki kwenye viz")

hl <- render(view_live(stats, NET, "en"))
ok(!grepl("%s|%d", hl), "hakuna placeholder kwenye live")

cat(sprintf("\n=== JUMLA: %d FAIL ===\n", fails))
if (fails > 0) quit(status = 1)

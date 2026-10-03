# ============================================================
# make_map_preview.R — tengeneza ukurasa thabiti wa ramani
# (mtihani wa kuthibitisha Leaflet + tiles + OSRM bila Shiny WS)
#
#   Rscript tests/make_map_preview.R
#   kisha fungua:  http://localhost:3838/map-preview.html
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
  for (d in c(".", "..", "web-r", file.path("mtaalamu-smart", "web-r"))) {
    if (file.exists(file.path(d, "app.R")) && dir.exists(file.path(d, "data"))) return(normalizePath(d))
  }
  normalizePath(".")
})

source(file.path(APP, "R", "i18n.R"))
source(file.path(APP, "R", "charts.R"))   # esc()
source(file.path(APP, "R", "views.R"))

GEO <- fromJSON(file.path(APP, "data", "geo.json"), simplifyVector = FALSE)
GEO_TXT <- paste(readLines(file.path(APP, "data", "geo.json"), encoding = "UTF-8", warn = FALSE),
                 collapse = "\n")

langs <- commandArgs(trailingOnly = TRUE)
if (!length(langs)) langs <- c("sw", "en")

for (lg in langs) {
  body <- renderTags(view_map(lg, GEO, map_payload(lg, GEO_TXT)))$html
  page <- paste0(
    "<!doctype html>\n<html lang=\"", lg, "\">\n<head>\n<meta charset=\"utf-8\">\n",
    "<title>MTAALAMU SMART — RAMANI (preview)</title>\n",
    "<link rel=\"stylesheet\" href=\"dashboard.css\">\n",
    "<link rel=\"stylesheet\" href=\"shiny.css\">\n",
    "<link rel=\"stylesheet\" href=\"leaflet/leaflet.css\">\n",
    "<script src=\"leaflet/leaflet.js\"></script>\n",
    "<script src=\"map.js\"></script>\n",
    "</head>\n<body>\n<div class=\"container-fluid\"><div class=\"dashboard\">\n",
    body, "\n</div></div>\n</body>\n</html>\n")

  out <- file.path(APP, "www", paste0("map-preview-", lg, ".html"))
  writeLines(page, out, useBytes = TRUE)
  cat("written:", out, "(", nchar(page), "herufi )\n")
}
cat("Fungua: http://localhost:3838/map-preview-sw.html\n")

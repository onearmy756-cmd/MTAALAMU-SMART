# ============================================================
# charts.R — michoro yote ya SVG (kama ilivyo React, bila mabadiliko)
# Kila kichuja kinatoka data (JSON/const) — hakuna takwimu mpya.
# ============================================================

`%||%` <- function(a, b) if (is.null(a) || length(a) == 0) b else a

esc <- function(x) {
  x <- as.character(x)
  x <- gsub("&", "&amp;", x, fixed = TRUE)
  x <- gsub("<", "&lt;", x, fixed = TRUE)
  x <- gsub(">", "&gt;", x, fixed = TRUE)
  x <- gsub("\"", "&quot;", x, fixed = TRUE)
  x
}
tf <- function(v, d = 1) format(round(v, d), trim = TRUE, scientific = FALSE)

# ---------------- data (kama JSX const) ----------------
REGION_REVENUE <- list(
  list(name = "Dar es Salaam", value = 13.5, color = "#1EB53A"),
  list(name = "Mwanza",       value = 8.2,  color = "#00A3DD"),
  list(name = "Arusha",       value = 7.5,  color = "#FCD116"),
  list(name = "Dodoma",       value = 6.8,  color = "#e91e63"),
  list(name = "Mbeya",        value = 5.4,  color = "#9c27b0"),
  list(name = "Tanga",        value = 4.2,  color = "#00bcd4"),
  list(name = "Morogoro",     value = 3.8,  color = "#ff5722"),
  list(name = "Zanzibar",     value = 3.1,  color = "#8bc34a")
)

WORKLOADS <- list(
  list(name = "Computer Repairs", value = 40, color = "#1EB53A", sw = "Matengenezo ya Kompyuta", en = "Computer Repairs"),
  list(name = "Phone Services",   value = 35, color = "#00A3DD", sw = "Huduma za Simu",          en = "Phone Services"),
  list(name = "Electrical Works", value = 25, color = "#FCD116", sw = "Kazi za Umeme",           en = "Electrical Works")
)

MONTHLY <- c(2.1, 3.2, 5.3, 5.6, 8.9, 9.2, 12.1, 14.3, 18.5, 22.4, 28.6, 35.2)

REGION_TECHS <- list(
  list(name = "Dar es Salaam", x = 72, y = 78, n = 15, star = 2),
  list(name = "Mwanza",       x = 30, y = 42, n = 8,  star = 1),
  list(name = "Arusha",       x = 60, y = 18, n = 5,  star = 0),
  list(name = "Kilimanjaro",  x = 70, y = 20, n = 3,  star = 0),
  list(name = "Tanga",        x = 78, y = 40, n = 2,  star = 0),
  list(name = "Morogoro",     x = 65, y = 58, n = 3,  star = 0),
  list(name = "Pwani",        x = 68, y = 70, n = 2,  star = 0),
  list(name = "Dodoma",       x = 50, y = 52, n = 6,  star = 0),
  list(name = "Mbeya",        x = 42, y = 82, n = 4,  star = 0),
  list(name = "Iringa",       x = 55, y = 75, n = 2,  star = 0),
  list(name = "Zanzibar",     x = 86, y = 76, n = 3,  star = 0),
  list(name = "Geita",        x = 26, y = 52, n = 2,  star = 0),
  list(name = "Kigoma",       x = 14, y = 60, n = 1,  star = 0),
  list(name = "Tabora",       x = 35, y = 62, n = 3,  star = 0)
)
REGION_TECHS_TOTAL <- sum(vapply(REGION_TECHS, function(r) r$n, numeric(1)))

HEATMAP_ROWS <- 10; HEATMAP_COLS <- 12
HEATMAP_LABELS <- c("Posta", "Kariakoo", "Ilala", "Kinondoni", "Temeke", "Mbezi",
                    "Sinza", "Tegeta", "Mwenge", "Mikocheni", "Msasani", "Oysterbay")
HEATMAP_DATA <- local({
  cx <- HEATMAP_COLS / 2 - 0.5; cy <- HEATMAP_ROWS / 2 - 0.5
  maxd <- sqrt(cx^2 + cy^2)
  m <- matrix(0, HEATMAP_ROWS, HEATMAP_COLS)
  for (r in 0:(HEATMAP_ROWS - 1)) for (c in 0:(HEATMAP_COLS - 1)) {
    d <- sqrt((c - cx)^2 + (r - cy)^2)
    base <- max(0, 1 - d / maxd)
    m[r + 1, c + 1] <- max(0, min(1, base + (sin(r * 1.3 + c * 0.9) * 0.15 - 0.07)))
  }
  m
})

TOP_TECHS <- list(
  list(rank = 1, name = "Hassan", region = "Dar es Salaam", rev = 8.5, jobs = 312),
  list(rank = 2, name = "Ali",    region = "Mwanza",       rev = 7.2, jobs = 268),
  list(rank = 3, name = "Juma",   region = "Arusha",       rev = 6.8, jobs = 245),
  list(rank = 4, name = "Fatuma", region = "Zanzibar",     rev = 5.1, jobs = 201),
  list(rank = 5, name = "John",   region = "Mbeya",        rev = 4.7, jobs = 188)
)

TIMELINE <- list(
  sw = c("M1 \u2014 Software kuanza", "M2 \u2014 Wateja 50", "M3 \u2014 Wateja 100",
         "M4 \u2014 Hardware module", "M5 \u2014 Mafundi 10", "M6 \u2014 Mafundi 30",
         "M7 \u2014 Professionals module", "M8 \u2014 Wataalamu 20", "M9 \u2014 Mikoa 15",
         "M10 \u2014 Mikoa 25", "M11 \u2014 Mikoa 31", "M12 \u2014 TZS 39M/mwezi"),
  en = c("M1 \u2014 Software launch", "M2 \u2014 50 customers", "M3 \u2014 100 customers",
         "M4 \u2014 Hardware module", "M5 \u2014 10 fundis", "M6 \u2014 30 fundis",
         "M7 \u2014 Professionals module", "M8 \u2014 20 experts", "M9 \u2014 15 regions",
         "M10 \u2014 25 regions", "M11 \u2014 31 regions", "M12 \u2014 TZS 39M/month")
)

TRADE_META <- list(
  umeme   = list(label = "UMEME",   icon = "\u26a1", color = "var(--orange)"),
  solar   = list(label = "SOLAR",   icon = "\u2600\ufe0f", color = "var(--orange)"),
  ujenzi  = list(label = "UJENZI",  icon = "\U0001f3d7\ufe0f", color = "#b0bec5"),
  maji    = list(label = "MAJI",    icon = "\U0001f4a7", color = "#4dd0e1"),
  computer= list(label = "COMPUTER",icon = "\U0001f4bb", color = "var(--blue)"),
  telecoms= list(label = "TELECOMS",icon = "\U0001f4e1", color = "#ba68c8"),
  network = list(label = "NETWORK", icon = "\U0001f310", color = "#4fc3f7")
)

# ============================================================
# LIVE
# ============================================================
core_viz_svg <- function() {
  paste0(
    '<svg viewBox="0 0 190 190">',
    '<g stroke="rgba(0,229,255,0.5)" stroke-width="1.5" fill="none">',
    '<path d="M95 95 L35 45" stroke-dasharray="4 4"><animate attributeName="stroke-dashoffset" from="16" to="0" dur="1s" repeatCount="indefinite"/></path>',
    '<path d="M95 95 L155 45" stroke-dasharray="4 4"><animate attributeName="stroke-dashoffset" from="16" to="0" dur="1.3s" repeatCount="indefinite"/></path>',
    '<path d="M95 95 L35 145" stroke-dasharray="4 4"><animate attributeName="stroke-dashoffset" from="16" to="0" dur="1.6s" repeatCount="indefinite"/></path>',
    '<path d="M95 95 L155 145" stroke-dasharray="4 4"><animate attributeName="stroke-dashoffset" from="16" to="0" dur="1.1s" repeatCount="indefinite"/></path>',
    '</g>',
    '<circle cx="95" cy="95" r="78" fill="none" stroke="rgba(0,229,255,0.18)" stroke-width="1"/>',
    '</svg>'
  )
}

topology_svg <- function(net, lang) {
  paste0(
    '<svg width="100%" height="250" viewBox="0 0 520 250">',
    '<g stroke="rgba(0,229,255,0.55)" stroke-width="1.5" fill="none">',
    '<path d="M260 45 L90 130" stroke-dasharray="5 5"><animate attributeName="stroke-dashoffset" from="20" to="0" dur="1.4s" repeatCount="indefinite"/></path>',
    '<path d="M260 45 L430 130" stroke-dasharray="5 5"><animate attributeName="stroke-dashoffset" from="20" to="0" dur="1.7s" repeatCount="indefinite"/></path>',
    '<path d="M90 130 L430 130" stroke-dasharray="5 5"><animate attributeName="stroke-dashoffset" from="20" to="0" dur="2.1s" repeatCount="indefinite"/></path>',
    '<path d="M90 130 L260 45 M430 130 L260 45" stroke="rgba(0,229,255,0.15)"/>',
    '</g>',
    '<g transform="translate(260,45)"><circle r="26" fill="rgba(0,40,60,0.9)" stroke="var(--cyan)" stroke-width="1.5"/>',
    '<text y="6" text-anchor="middle" font-size="22">\U0001f310</text>',
    '<text y="44" text-anchor="middle" fill="#cfe9f5" font-size="13" font-family="var(--mono)">Internet</text></g>',
    '<g transform="translate(260,135)"><rect x="-26" y="-20" width="52" height="34" rx="5" fill="rgba(0,40,60,0.9)" stroke="var(--cyan)" stroke-width="1.5"/>',
    '<text y="4" text-anchor="middle" font-size="18">\U0001f4e1</text>',
    '<text y="34" text-anchor="middle" fill="#cfe9f5" font-size="13" font-family="var(--mono)">Router</text></g>',
    '<g transform="translate(90,175)"><rect x="-24" y="-20" width="48" height="32" rx="4" fill="rgba(0,40,60,0.9)" stroke="var(--cyan)" stroke-width="1.5"/>',
    '<text y="4" text-anchor="middle" font-size="17">\U0001f5a5\ufe0f</text>',
    '<text y="34" text-anchor="middle" fill="#cfe9f5" font-size="13" font-family="var(--mono)">PC</text></g>',
    '<g transform="translate(430,175)"><rect x="-22" y="-22" width="44" height="36" rx="4" fill="rgba(0,40,60,0.9)" stroke="var(--cyan)" stroke-width="1.5"/>',
    '<text y="4" text-anchor="middle" font-size="17">\U0001f5c4\ufe0f</text>',
    '<text y="34" text-anchor="middle" fill="#cfe9f5" font-size="13" font-family="var(--mono)">DNS</text></g>',
    '<text x="140" y="72" fill="#5e8aa3" font-size="11" font-family="var(--mono)">1000mbps</text>',
    '<text x="335" y="80" fill="#5e8aa3" font-size="11" font-family="var(--mono)">', net$pcping, 'ms PC</text>',
    '<text x="335" y="110" fill="#5e8aa3" font-size="11" font-family="var(--mono)">18ms Router-DNS</text>',
    '</svg>'
  )
}

# ============================================================
# VIZ: Bar chart (Mapato kwa mkoa)
# ============================================================
bar_chart_svg <- function() {
  W <- 640; H <- 240; padL <- 80; padB <- 40; padT <- 15; padR <- 15
  innerW <- W - padL - padR; innerH <- H - padT - padB
  n <- length(REGION_REVENUE)
  bw <- innerW / n * 0.7; gap <- innerW / n * 0.3
  maxv <- 14
  o <- c('<svg viewBox="0 0 640 240" class="viz-svg">', '<g stroke="rgba(0,229,255,0.15)">')
  for (p in c(0, 0.25, 0.5, 0.75, 1)) {
    y <- padT + innerH - innerH * p
    o <- c(o, sprintf('<line x1="%d" x2="%d" y1="%.2f" y2="%.2f"/>', padL, W - padR, y, y))
  }
  o <- c(o, '</g>', '<g fill="#5e8aa3" font-size="10" font-family="var(--mono)">')
  for (v in c(0, 3.5, 7, 10.5, 14)) {
    y <- padT + innerH - (v / maxv) * innerH + 3
    o <- c(o, sprintf('<text x="%d" y="%.1f" text-anchor="end">%sM</text>', padL - 6, y, tf(v, 1)))
  }
  o <- c(o, '</g>')
  for (i in seq_len(n)) {
    r <- REGION_REVENUE[[i]]
    x <- padL + (i - 1) * (bw + gap) + gap / 2
    bh <- (r$value / maxv) * innerH
    y <- padT + innerH - bh
    o <- c(o, '<g>')
    o <- c(o, sprintf('<rect x="%.1f" y="%.1f" width="%.1f" height="%.1f" fill="%s" opacity="0.85" rx="2"/>',
                      x, y, bw, bh, r$color))
    o <- c(o, sprintf('<rect x="%.1f" y="%.1f" width="%.1f" height="3" fill="white" opacity="0.6"/>', x, y, bw))
    o <- c(o, sprintf('<text x="%.1f" y="%.1f" text-anchor="middle" font-size="11" font-family="var(--mono)" fill="#cfe9f5">%sM</text>',
                      x + bw / 2, y - 5, tf(r$value, 1)))
    o <- c(o, sprintf('<text x="%.1f" y="%d" text-anchor="middle" font-size="9" font-family="var(--mono)" fill="#8ab4c8">%s</text>',
                      x + bw / 2, H - padB + 14, esc(r$name)))
    o <- c(o, '</g>')
  }
  o <- c(o, sprintf('<line x1="%d" y1="%d" x2="%d" y2="%d" stroke="var(--cyan)" stroke-width="1.5"/>',
                    padL, padT + innerH, W - padR, padT + innerH), '</svg>')
  paste(o, collapse = "")
}

# ============================================================
# VIZ: Pie chart (Aina za kazi)
# ============================================================
pie_chart_svg <- function(lang) {
  cx <- 130; cy <- 130; R <- 100; r <- 55
  total <- sum(vapply(WORKLOADS, function(w) w$value, numeric(1)))
  acc <- 0
  o <- c('<svg viewBox="0 0 330 260" class="viz-svg">')
  for (i in seq_along(WORKLOADS)) {
    w <- WORKLOADS[[i]]
    o <- c(o, '<g>')
    start <- (acc / total) * 2 * pi - pi / 2
    acc <- acc + w$value
    end <- (acc / total) * 2 * pi - pi / 2
    large <- as.integer(end - start > pi)
    x1 <- cx + R * cos(start); y1 <- cy + R * sin(start)
    x2 <- cx + R * cos(end);   y2 <- cy + R * sin(end)
    x3 <- cx + r * cos(end);   y3 <- cy + r * sin(end)
    x4 <- cx + r * cos(start); y4 <- cy + r * sin(start)
    mid <- (start + end) / 2
    lx <- cx + (R + 16) * cos(mid); ly <- cy + (R + 16) * sin(mid)
    anchor <- if (lx > cx) "start" else "end"
    o <- c(o, sprintf('<path d="M %.2f %.2f A %d %d 0 %d 1 %.2f %.2f L %.2f %.2f A %d %d 0 %d 0 %.2f %.2f Z" fill="%s" opacity="0.9" stroke="var(--bg)" stroke-width="2"/>',
                      x1, y1, R, R, large, x2, y2, x3, y3, r, r, large, x4, y4, w$color))
    mx <- cx + (r + (R - r) / 2) * cos(start + pi / 3)
    my <- cy + (r + (R - r) / 2) * sin(start + pi / 3)
    o <- c(o, sprintf('<line x1="%.1f" y1="%.1f" x2="%.1f" y2="%.1f" stroke="rgba(0,229,255,0.4)"/>', mx, my, lx, ly))
    o <- c(o, sprintf('<text x="%.1f" y="%.1f" font-size="10" font-family="var(--mono)" fill="#cfe9f5" text-anchor="%s">%s %s%%</text>',
                      lx, ly + 4, anchor, esc(bi(list(sw = w$sw, en = w$en), lang)), round(w$value / total * 100)))
    o <- c(o, '</g>')
  }
  o <- c(o,
    sprintf('<text x="%d" y="%d" text-anchor="middle" font-size="13" font-family="var(--mono)" fill="var(--cyan)">100%%</text>', cx, cy - 6),
    sprintf('<text x="%d" y="%d" text-anchor="middle" font-size="9" font-family="var(--mono)" fill="#8ab4c8">%s</text>',
            cx, cy + 12, esc(tr("v.pie.total", lang))),
    '</svg>')
  paste(o, collapse = "")
}

# ============================================================
# VIZ: Line chart (Ukuaji wa mapato)
# ============================================================
line_chart_svg <- function() {
  W <- 640; H <- 240; padL <- 55; padB <- 30; padT <- 15; padR <- 15
  innerW <- W - padL - padR; innerH <- H - padT - padB
  maxv <- 40
  xs <- function(i) padL + (i / (length(MONTHLY) - 1)) * innerW
  ys <- function(v) padT + innerH - (v / maxv) * innerH
  pts <- vapply(seq_along(MONTHLY), function(i)
    sprintf("%s %.1f %.1f", if (i == 1) "M" else "L", xs(i - 1), ys(MONTHLY[i])), character(1))
  d <- paste(pts, collapse = " ")
  area <- paste0(d, " L ", xs(length(MONTHLY) - 1), " ", padT + innerH, " L ", padL, " ", padT + innerH, " Z")
  o <- c('<svg viewBox="0 0 640 240" class="viz-svg">',
         '<defs><linearGradient id="lineGrad" x1="0" y1="0" x2="0" y2="1">',
         '<stop offset="0%" stop-color="var(--cyan)" stop-opacity="0.55"/>',
         '<stop offset="100%" stop-color="var(--cyan)" stop-opacity="0.02"/>',
         '</linearGradient></defs>',
         '<g stroke="rgba(0,229,255,0.15)">')
  for (p in c(0, 0.25, 0.5, 0.75, 1)) {
    y <- padT + innerH - innerH * p
    o <- c(o, sprintf('<line x1="%d" x2="%d" y1="%.1f" y2="%.1f"/>', padL, W - padR, y, y))
  }
  o <- c(o, '</g>', '<g fill="#5e8aa3" font-size="10" font-family="var(--mono)">')
  for (v in c(0, 10, 20, 30, 40)) {
    y <- padT + innerH - (v / maxv) * innerH + 3
    o <- c(o, sprintf('<text x="%d" y="%.1f" text-anchor="end">%dM</text>', padL - 6, y, v))
  }
  o <- c(o, '</g>',
    sprintf('<path d="%s" fill="url(#lineGrad)"/>', area),
    sprintf('<path d="%s" fill="none" stroke="var(--cyan)" stroke-width="2.5" stroke-linejoin="round" stroke-linecap="round"/>', d))
  for (i in seq_along(MONTHLY)) {
    o <- c(o, sprintf('<g><circle cx="%.1f" cy="%.1f" r="3.5" fill="#04121a" stroke="var(--cyan)" stroke-width="2"/>',
                      xs(i - 1), ys(MONTHLY[i])))
    o <- c(o, sprintf('<text x="%.1f" y="%d" text-anchor="middle" font-size="9" font-family="var(--mono)" fill="#8ab4c8">M%d</text></g>',
                      xs(i - 1), H - padB + 14, i))
  }
  o <- c(o, sprintf('<line x1="%d" y1="%d" x2="%d" y2="%d" stroke="var(--cyan)" stroke-width="1.5"/>',
                    padL, padT + innerH, W - padR, padT + innerH), '</svg>')
  paste(o, collapse = "")
}

# ============================================================
# VIZ: Ramani ya Tanzania
# ============================================================
tz_map_svg <- function(lang) {
  size <- function(n) max(4, min(12, 2 + n * 0.55))
  o <- c('<svg viewBox="0 0 420 220" class="viz-svg">',
         '<defs><radialGradient id="tzGlow"><stop offset="0%" stop-color="var(--cyan)" stop-opacity="0.15"/>',
         '<stop offset="100%" stop-color="var(--cyan)" stop-opacity="0"/></radialGradient></defs>',
         '<rect x="10" y="5" width="400" height="210" fill="url(#tzGlow)"/>',
         '<path d="M25 30 L50 10 L85 15 L105 35 L105 65 L120 78 L115 110 L95 125 L70 140 L50 130 L35 110 L20 85 Z" fill="rgba(30,181,58,0.08)" stroke="rgba(0,229,255,0.5)" stroke-dasharray="3 3" stroke-width="1.5"/>',
         '<path d="M100 70 L125 72 L125 80 L112 82 Z" fill="rgba(252,209,22,0.2)" stroke="rgba(0,229,255,0.4)"/>',
         sprintf('<text x="210" y="18" text-anchor="middle" font-family="var(--mono)" font-size="14" fill="#cfe9f5">%s</text>',
                 esc(tr("v.map.tz", lang))))
  for (r in REGION_TECHS) {
    cx <- r$x * 4 + 10; cy <- r$y * 2 + 5; s <- size(r$n)
    o <- c(o, sprintf('<g><circle cx="%.0f" cy="%.0f" r="%.1f" fill="var(--cyan)" opacity="0.15"/>', cx, cy, s * 2.5))
    o <- c(o, sprintf('<circle cx="%.0f" cy="%.0f" r="%.1f" fill="var(--cyan)" stroke="white" stroke-width="1"/>', cx, cy, s))
    o <- c(o, sprintf('<text x="%.1f" y="%.0f" font-size="9" font-family="var(--mono)" fill="#cfe9f5">%s (%d)%s</text></g>',
                      cx + s + 4, cy + 3, esc(r$name), r$n, strrep("\u2b50", r$star)))
  }
  paste0(paste(o, collapse = ""), "</svg>")
}

# ============================================================
# VIZ: Heatmap (mahitaji)
# ============================================================
heatmap_svg <- function(lang) {
  cellW <- 40; cellH <- 28; padL <- 70; padT <- 50
  W <- padL + HEATMAP_COLS * cellW + 20
  H <- padT + HEATMAP_ROWS * cellH + 30
  col <- function(v) {
    if (v < 0.15) "rgba(94,138,163,0.18)"
    else if (v < 0.4) "#FCD116"
    else if (v < 0.7) "#ff6600"
    else "#ff2222"
  }
  o <- c(sprintf('<svg viewBox="0 0 %d %d" class="viz-svg">', W, H),
         sprintf('<text x="%d" y="24" text-anchor="middle" font-family="var(--mono)" font-size="13" fill="#cfe9f5">%s</text>',
                 W / 2, esc(tr("v.heat.demand", lang))))
  for (r in seq_len(HEATMAP_ROWS)) for (c in seq_len(HEATMAP_COLS)) {
    o <- c(o, sprintf('<rect x="%d" y="%d" width="%d" height="%d" fill="%s" rx="2"/>',
                      padL + (c - 1) * cellW + 1, padT + (r - 1) * cellH + 1,
                      cellW - 2, cellH - 2, col(HEATMAP_DATA[r, c])))
  }
  for (c in seq_along(HEATMAP_LABELS)) {
    o <- c(o, sprintf('<text x="%.0f" y="%d" text-anchor="middle" font-size="8" font-family="var(--mono)" fill="#8ab4c8">%s</text>',
                      padL + (c - 1) * cellW + cellW / 2, padT - 8, esc(HEATMAP_LABELS[c])))
  }
  for (r in seq_len(HEATMAP_ROWS)) {
    o <- c(o, sprintf('<text x="%d" y="%.0f" text-anchor="end" font-size="9" font-family="var(--mono)" fill="#8ab4c8">S%d</text>',
                      padL - 6, padT + (r - 1) * cellH + cellH / 2 + 3, r))
  }
  o <- c(o, sprintf('<g transform="translate(%d,%d)">', padL, padT + HEATMAP_ROWS * cellH + 10))
  legend <- list(list(t = "Low", v = 0.08), list(t = "Mid", v = 0.35),
                 list(t = "High", v = 0.6), list(t = "MAX", v = 0.9))
  for (i in seq_along(legend)) {
    e <- legend[[i]]
    o <- c(o, sprintf('<g transform="translate(%d,0)"><rect width="16" height="12" fill="%s" rx="2"/>',
                      (i - 1) * 120, col(e$v)))
    o <- c(o, sprintf('<text x="22" y="10" font-size="10" font-family="var(--mono)" fill="#cfe9f5">%s</text></g>', e$t))
  }
  paste0(paste(o, collapse = ""), "</g></svg>")
}

# ============================================================
# VIZ: Timeline ya mwaka 1
# ============================================================
timeline_svg <- function(lang) {
  items <- TIMELINE[[lang]] %||% TIMELINE$en
  n <- length(items)
  o <- c('<svg viewBox="0 0 700 260" class="viz-svg">',
         '<line x1="30" y1="130" x2="670" y2="130" stroke="var(--cyan)" stroke-width="3"/>')
  for (i in seq_len(n)) {
    x <- 30 + ((i - 1) / (n - 1)) * (670 - 30)
    top <- (i - 1) %% 2 == 0
    ly <- if (top) 100 else 160
    ty <- if (top) 40 else 192
    ry <- if (top) 20 else 172
    o <- c(o, sprintf('<g><circle cx="%.0f" cy="130" r="7" fill="#04121a" stroke="var(--cyan)" stroke-width="2"/>', x))
    o <- c(o, sprintf('<circle cx="%.0f" cy="130" r="3" fill="var(--cyan)"/>', x))
    o <- c(o, sprintf('<line x1="%.0f" y1="130" x2="%.0f" y2="%d" stroke="rgba(0,229,255,0.4)" stroke-width="1.5" stroke-dasharray="3 2"/>', x, x, ly))
    o <- c(o, sprintf('<rect x="%.0f" y="%d" width="110" height="30" rx="3" fill="rgba(0,40,60,0.85)" stroke="var(--cyan)" stroke-opacity="0.4"/>', x - 55, ry))
    o <- c(o, sprintf('<text x="%.0f" y="%d" text-anchor="middle" font-family="var(--mono)" font-size="9" fill="#cfe9f5">%s</text></g>',
                      x, ty, esc(items[i])))
  }
  paste0(paste(o, collapse = ""), "</svg>")
}

# ============================================================
# VIZ: 5 Agent workflow
# ============================================================
agent_flow_svg <- function(lang) {
  nodes <- list(
    list(x = 70,  y = 20,  t = tr("v.flow.mteja", lang),     sub = tr("v.flow.n1", lang),     c = "#1EB53A"),
    list(x = 70,  y = 90,  t = tr("v.flow.reception", lang), sub = tr("v.flow.n2", lang),     c = "#00A3DD"),
    list(x = 30,  y = 170, t = tr("v.flow.diagnose", lang),  sub = tr("v.flow.n3", lang),     c = "#FCD116"),
    list(x = 110, y = 170, t = tr("v.flow.sheria", lang),    sub = tr("v.flow.n4", lang),     c = "#ba68c8"),
    list(x = 30,  y = 250, t = tr("v.flow.formula", lang),   sub = tr("v.flow.n5", lang),     c = "#e91e63"),
    list(x = 110, y = 250, t = tr("v.flow.action", lang),    sub = tr("v.flow.n6", lang),     c = "#ff5722"),
    list(x = 70,  y = 330, t = tr("v.flow.verify", lang),    sub = tr("v.flow.n7", lang),     c = "#00bcd4"),
    list(x = 70,  y = 400, t = tr("v.flow.pay", lang),       sub = tr("v.flow.n8", lang),     c = "#8bc34a"),
    list(x = 70,  y = 465, t = tr("v.flow.rating", lang),    sub = tr("v.flow.n9", lang),     c = "#FCD116")
  )
  edges <- list(c(1, 2), c(2, 3), c(2, 4), c(3, 5), c(4, 6), c(5, 7), c(6, 7), c(7, 8), c(8, 9))
  o <- c('<svg viewBox="0 0 420 510" class="viz-svg">')
  for (k in seq_along(edges)) {
    e <- edges[[k]]; a <- nodes[[e[1]]]; b <- nodes[[e[2]]]
    y1 <- a$y + 28; y2 <- b$y - 28; x1 <- a$x * 3 + 20; x2 <- b$x * 3 + 20
    dur <- 1.4 + (k - 1) * 0.15
    o <- c(o, sprintf('<g><line x1="%d" y1="%d" x2="%d" y2="%d" stroke="rgba(0,229,255,0.5)" stroke-width="2" stroke-dasharray="5 4" marker-end="url(#arr)"/>', x1, y1, x2, y2))
    o <- c(o, sprintf('<circle cx="%d" cy="%d" r="2" fill="var(--cyan)"><animate attributeName="cy" from="%d" to="%d" dur="%.2fs" repeatCount="indefinite"/><animate attributeName="cx" from="%d" to="%d" dur="%.2fs" repeatCount="indefinite"/></circle></g>',
                      x1, y1, y1, y2, dur, x1, x2, dur))
  }
  o <- c(o, '<defs><marker id="arr" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="6" markerHeight="6" orient="auto-start-reverse">',
         '<path d="M0,0 L10,5 L0,10 z" fill="rgba(0,229,255,0.5)"/></marker></defs>')
  for (n in nodes) {
    cx <- n$x * 3 + 20
    o <- c(o, sprintf('<g transform="translate(%d,%d)">', cx, n$y))
    o <- c(o, sprintf('<rect x="-80" y="-24" width="160" height="48" rx="6" fill="rgba(4,18,26,0.92)" stroke="%s" stroke-width="2"/>', n$c))
    o <- c(o, sprintf('<text text-anchor="middle" y="-4" font-family="var(--mono)" font-size="11" font-weight="700" fill="%s">%s</text>', n$c, esc(n$t)))
    o <- c(o, sprintf('<text text-anchor="middle" y="14" font-family="var(--mono)" font-size="9" fill="#8ab4c8">%s</text></g>', esc(n$sub)))
  }
  paste0(paste(o, collapse = ""), "</svg>")
}

# ============================================================
# VIZ: Mtandao wa Smart Technician
# ============================================================
network_diagram_svg <- function(lang) {
  you <- if (identical(lang, "sw")) "WEWE" else "YOU"
  cust <- if (identical(lang, "sw")) "Wateja" else "Customers"
  permo <- if (identical(lang, "sw")) "Mtu 1 \u2022 %d kazi/m" else "1 person \u2022 %d jobs/mo"
  permo5 <- if (identical(lang, "sw")) "%d / mwezi" else "%d / month"
  nodes <- list(
    list(x = 210, y = 40,  t = you,   s = "Dar \u2022 HQ",                    c = "#FCD116", i = "\U0001f454"),
    list(x = 100, y = 120, t = "Mwanza", s = sprintf(permo, 40),              c = "#00A3DD", i = "\U0001f4cd"),
    list(x = 320, y = 120, t = "Arusha", s = sprintf(permo, 35),              c = "#1EB53A", i = "\U0001f4cd"),
    list(x = 210, y = 200, t = "Mbeya",  s = sprintf(permo, 30),              c = "#e91e63", i = "\U0001f4cd"),
    list(x = 40,  y = 220, t = cust, s = sprintf(permo5, 50),                 c = "#ba68c8", i = "\U0001f465"),
    list(x = 160, y = 220, t = cust, s = sprintf(permo5, 45),                 c = "#ba68c8", i = "\U0001f465"),
    list(x = 260, y = 220, t = cust, s = sprintf(permo5, 38),                 c = "#ba68c8", i = "\U0001f465"),
    list(x = 380, y = 220, t = cust, s = sprintf(permo5, 30),                 c = "#ba68c8", i = "\U0001f465")
  )
  edges <- list(c(210, 40, 100, 120), c(210, 40, 320, 120), c(210, 40, 210, 200),
                c(100, 120, 40, 220), c(100, 120, 160, 220),
                c(320, 120, 260, 220), c(320, 120, 380, 220))
  o <- c('<svg viewBox="0 0 420 260" class="viz-svg">',
         '<defs><marker id="ar2" viewBox="0 0 10 10" refX="8" refY="5" markerWidth="5" markerHeight="5" orient="auto">',
         '<path d="M0,0 L10,5 L0,10 z" fill="rgba(0,229,255,0.55)"/></marker></defs>')
  for (e in edges) {
    o <- c(o, sprintf('<line x1="%d" y1="%d" x2="%d" y2="%d" stroke="rgba(0,229,255,0.55)" stroke-width="1.5" stroke-dasharray="5 4" marker-end="url(#ar2)"/>',
                      e[1], e[2], e[3], e[4]))
  }
  for (n in nodes) {
    o <- c(o, sprintf('<g transform="translate(%d,%d)"><circle r="26" fill="rgba(4,18,26,0.9)" stroke="%s" stroke-width="2"/>', n$x, n$y, n$c))
    o <- c(o, sprintf('<text y="5" text-anchor="middle" font-size="18">%s</text>', n$i))
    o <- c(o, sprintf('<text y="44" text-anchor="middle" font-family="var(--mono)" font-size="10" font-weight="700" fill="%s">%s</text>', n$c, esc(n$t)))
    o <- c(o, sprintf('<text y="56" text-anchor="middle" font-family="var(--mono)" font-size="8" fill="#8ab4c8">%s</text></g>', esc(n$s)))
  }
  paste0(paste(o, collapse = ""), "</svg>")
}

# ============================================================
# VIZ: Mfumo wa kifaa (components)
# ============================================================
SYSTEM_COMPONENTS <- list(
  list(id = "cpu",         x = 210, y = 30,  t = "CPU i7-12700K", ty = "cpu",    st = "good",    specs = "12C / 20T \u2022 65\u00b0C", c = "#1EB53A"),
  list(id = "ram-a",       x = 60,  y = 110, t = "RAM 16GB DDR5", ty = "ram",    st = "good",    specs = "5600 MHz",                c = "#1EB53A"),
  list(id = "ram-b",       x = 60,  y = 185, t = "RAM 16GB DDR5", ty = "ram",    st = "good",    specs = "5600 MHz",                c = "#1EB53A"),
  list(id = "motherboard", x = 210, y = 150, t = "MSI Z690",      ty = "mother", st = "good",    specs = "ATX \u2022 LGA1700",       c = "#00A3DD"),
  list(id = "gpu",         x = 360, y = 110, t = "RTX 4070",      ty = "gpu",    st = "warning", specs = "VRAM 12GB \u2022 82\u00b0C", c = "#FCD116"),
  list(id = "psu",         x = 360, y = 205, t = "RM850x 850W",   ty = "psu",    st = "good",    specs = "80+ Gold",                c = "#1EB53A"),
  list(id = "disk",        x = 60,  y = 270, t = "980 PRO 1TB",   ty = "disk",   st = "good",    specs = "NVMe \u2022 Health 98%",   c = "#1EB53A"),
  list(id = "network",     x = 210, y = 270, t = "Intel 2.5GbE",  ty = "net",    st = "good",    specs = "Connected 2.5Gbps",        c = "#1EB53A"),
  list(id = "audio",       x = 360, y = 290, t = "ALC4080",       ty = "audio",  st = "good",    specs = "Hi-Res",                   c = "#1EB53A")
)
SYSTEM_WIRES <- list(
  c("cpu", "motherboard"), c("ram-a", "motherboard"), c("ram-b", "motherboard"),
  c("gpu", "motherboard"), c("gpu", "psu"), c("disk", "motherboard"),
  c("network", "motherboard"), c("audio", "motherboard"), c("psu", "motherboard")
)
SYSTEM_ICONS <- c(cpu = "\U0001f9e0", ram = "\U0001f4be", mother = "\u25b8\u25b8", gpu = "\U0001f3ae",
                  psu = "\u26a1", disk = "\U0001f4bf", net = "\U0001f310", audio = "\U0001f50a")

device_system_svg <- function() {
  pos <- list()
  for (cmp in SYSTEM_COMPONENTS) pos[[cmp$id]] <- c(cmp$x, cmp$y)
  o <- c('<svg viewBox="0 0 420 330" class="viz-svg">')
  k <- 0
  for (w in SYSTEM_WIRES) {
    k <- k + 1
    p <- pos[[w[1]]]; q <- pos[[w[2]]]
    col <- SYSTEM_COMPONENTS[[which(vapply(SYSTEM_COMPONENTS, function(z) z$id == w[1], logical(1)))]]$c
    dur <- 1.8 + (k - 1) * 0.2
    o <- c(o, sprintf('<g><line x1="%d" y1="%d" x2="%d" y2="%d" stroke="rgba(0,229,255,0.45)" stroke-width="1.6" stroke-dasharray="4 3"/>', p[1], p[2], q[1], q[2]))
    o <- c(o, sprintf('<circle r="2" fill="%s"><animateMotion dur="%.1fs" repeatCount="indefinite" path="M%d,%d L%d,%d"/></circle></g>',
                      col, dur, p[1], p[2], q[1], q[2]))
  }
  for (cmp in SYSTEM_COMPONENTS) {
    o <- c(o, sprintf('<g transform="translate(%d,%d)">', cmp$x, cmp$y))
    o <- c(o, sprintf('<rect x="-60" y="-22" width="120" height="40" rx="4" fill="rgba(4,18,26,0.95)" stroke="%s" stroke-width="1.8"/>', cmp$c))
    o <- c(o, sprintf('<text x="-48" y="5" font-size="15">%s</text>', SYSTEM_ICONS[[cmp$ty]] %||% "\U0001f527"))
    o <- c(o, sprintf('<text x="-30" y="-1" font-family="var(--mono)" font-size="10" font-weight="700" fill="%s">%s</text>', cmp$c, esc(cmp$t)))
    o <- c(o, sprintf('<text x="-30" y="13" font-family="var(--mono)" font-size="8" fill="#8ab4c8">%s</text>', esc(cmp$specs)))
    o <- c(o, sprintf('<circle cx="50" cy="-2" r="3" fill="%s"/></g>', cmp$c))
  }
  o <- c(o, '<g transform="translate(14,316)">')
  legend <- list(list(s = "good", x = 0), list(s = "warning", x = 150), list(s = "error", x = 270))
  for (lg in legend) {
    cc <- if (lg$s == "good") "#1EB53A" else if (lg$s == "warning") "#FCD116" else "#ff4444"
    o <- c(o, sprintf('<g transform="translate(%d,0)"><circle r="4" cy="4" fill="%s"/>', lg$x, cc))
    o <- c(o, sprintf('<text x="10" y="8" font-family="var(--mono)" font-size="9" fill="#cfe9f5">%s</text></g>', toupper(lg$s)))
  }
  paste0(paste(o, collapse = ""), "</g></svg>")
}

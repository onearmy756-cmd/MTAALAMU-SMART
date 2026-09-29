# ============================================================
# sysprobe.R — AV6: metrics halisi za OS (R-side)
# Inatumia system commands / ps — fallback bila Rust binary
# ============================================================

`%||%` <- function(a, b) if (is.null(a) || length(a) == 0 || (is.character(a) && !nzchar(a))) b else a

.sysprobe_run <- function(cmd) {
  tryCatch({
    out <- system(cmd, intern = TRUE, ignore.stderr = TRUE)
    if (length(out) == 0 || inherits(out, "try-error")) return(NULL)
    out
  }, error = function(e) NULL)
}

# --- Cross-platform helpers ---
sysprobe_is_windows <- function() .Platform$OS.type == "windows"

sysprobe_cpu_pct <- function() {
  if (sysprobe_is_windows()) {
    # wmic
    o <- .sysprobe_run('wmic cpu get loadpercentage /value')
    if (!is.null(o)) {
      line <- grep("LoadPercentage", o, value = TRUE)
      if (length(line)) {
        v <- as.numeric(sub(".*=", "", line[1]))
        if (!is.na(v)) return(v)
      }
    }
  } else {
    # Linux: /proc/stat sample
    o <- .sysprobe_run("grep 'cpu ' /proc/stat")
    if (!is.null(o) && length(o) >= 1) {
      # fallback: top
      t <- .sysprobe_run("top -bn1 | grep 'Cpu(s)' | head -1")
      if (!is.null(t)) {
        # %Cpu(s):  x.x us,
        m <- regmatches(t, regexpr("[0-9]+\\.[0-9]+", t))
        if (length(m)) return(as.numeric(m[1]))
      }
    }
    # macOS
    o <- .sysprobe_run("top -l 1 -n 0 | grep 'CPU usage'")
    if (!is.null(o)) {
      m <- regmatches(o, regexpr("[0-9]+\\.[0-9]+", o))
      if (length(m)) return(as.numeric(m[1]))
    }
  }
  NA_real_
}

sysprobe_mem <- function() {
  if (sysprobe_is_windows()) {
    o <- .sysprobe_run('wmic OS get FreePhysicalMemory,TotalVisibleMemorySize /value')
    if (!is.null(o)) {
      free <- as.numeric(sub(".*=", "", grep("FreePhysicalMemory", o, value = TRUE)[1]))
      tot  <- as.numeric(sub(".*=", "", grep("TotalVisibleMemorySize", o, value = TRUE)[1]))
      if (!is.na(free) && !is.na(tot) && tot > 0) {
        # KB
        return(list(
          total_mb = tot / 1024,
          used_mb = (tot - free) / 1024,
          pct = ((tot - free) / tot) * 100
        ))
      }
    }
  } else {
    o <- .sysprobe_run("free -m 2>/dev/null | awk '/^Mem:/{print $2,$3}'")
    if (!is.null(o) && length(o) >= 1) {
      parts <- strsplit(trimws(o[1]), "\\s+")[[1]]
      if (length(parts) >= 2) {
        tot <- as.numeric(parts[1]); used <- as.numeric(parts[2])
        if (!is.na(tot) && tot > 0)
          return(list(total_mb = tot, used_mb = used, pct = (used / tot) * 100))
      }
    }
    # macOS vm_stat
    o <- .sysprobe_run("sysctl -n hw.memsize 2>/dev/null")
    if (!is.null(o)) {
      tot <- as.numeric(o[1]) / (1024 * 1024)
      if (!is.na(tot))
        return(list(total_mb = tot, used_mb = NA_real_, pct = NA_real_))
    }
  }
  list(total_mb = NA_real_, used_mb = NA_real_, pct = NA_real_)
}

sysprobe_disk <- function() {
  if (sysprobe_is_windows()) {
    o <- .sysprobe_run('wmic logicaldisk where "DriveType=3" get DeviceID,Size,FreeSpace /value')
    # simplified: return one summary
    return(list(list(mount = "C:", used_pct = NA_real_, status = "unknown")))
  }
  o <- .sysprobe_run("df -P -h 2>/dev/null | awk 'NR>1 && $6 ~ /^\\//{print $6,$5}'")
  if (is.null(o)) return(list())
  lapply(o, function(line) {
    parts <- strsplit(trimws(line), "\\s+")[[1]]
    if (length(parts) < 2) return(NULL)
    pct <- as.numeric(gsub("%", "", parts[2]))
    st <- if (is.na(pct)) "unknown" else if (pct >= 92) "critical" else if (pct >= 80) "warning" else "good"
    list(mount = parts[1], used_pct = pct, status = st)
  })
}

sysprobe_top_procs <- function(n = 8) {
  if (sysprobe_is_windows()) {
    o <- .sysprobe_run('wmic process get Name,PercentProcessorTime,WorkingSetSize /format:csv')
    return(list())
  }
  # Linux/mac: ps
  o <- .sysprobe_run(sprintf("ps -eo pid,pcpu,pmem,comm --sort=-pcpu 2>/dev/null | head -n %d", n + 1))
  if (is.null(o) || length(o) < 2) {
    o <- .sysprobe_run(sprintf("ps aux 2>/dev/null | sort -nrk 3 | head -n %d", n + 1))
    if (is.null(o)) return(list())
  }
  # skip header
  rows <- o[-1]
  lapply(rows, function(line) {
    parts <- strsplit(trimws(line), "\\s+")[[1]]
    if (length(parts) < 4) return(NULL)
    list(
      pid = parts[1],
      cpu = as.numeric(parts[2]),
      ram = as.numeric(parts[3]),
      name = paste(parts[4:length(parts)], collapse = " ")
    )
  })
}

#' Probe kamili — orodha ya JSON-like
sysprobe_snapshot <- function() {
  cpu <- sysprobe_cpu_pct()
  mem <- sysprobe_mem()
  disks <- Filter(Negate(is.null), sysprobe_disk())
  procs <- Filter(Negate(is.null), sysprobe_top_procs(8))

  issues <- character(0)
  if (!is.na(cpu) && cpu >= 90) issues <- c(issues, sprintf("CPU juu sana: %.1f%%", cpu))
  else if (!is.na(cpu) && cpu >= 75) issues <- c(issues, sprintf("CPU imejaa: %.1f%%", cpu))
  if (!is.na(mem$pct) && mem$pct >= 90)
    issues <- c(issues, sprintf("RAM karibu imejaa: %.1f%%", mem$pct))
  else if (!is.na(mem$pct) && mem$pct >= 80)
    issues <- c(issues, sprintf("RAM juu: %.1f%%", mem$pct))
  for (d in disks) {
    if (!is.null(d$used_pct) && !is.na(d$used_pct) && d$used_pct >= 92)
      issues <- c(issues, sprintf("Diski %s imejaa %.0f%%", d$mount, d$used_pct))
  }

  health <- if (length(issues) == 0) "good" else if (any(grepl("karibu|sana", issues))) "critical" else "warning"

  list(
    timestamp = as.numeric(Sys.time()),
    source = "r-sysprobe",
    hostname = Sys.info()[["nodename"]],
    os = paste(Sys.info()[["sysname"]], Sys.info()[["release"]]),
    cpu_usage_pct = cpu,
    ram_total_mb = mem$total_mb,
    ram_used_mb = mem$used_mb,
    ram_usage_pct = mem$pct,
    disks = disks,
    top_processes = procs,
    health = health,
    issues = as.list(issues)
  )
}

#' UI panel for live probe
sysprobe_panel_el <- function(snap, lang = "sw") {
  if (is.null(snap)) {
    return(tags$div(style = "color:var(--dim)", "Probe haipatikani"))
  }
  cpu <- snap$cpu_usage_pct %||% NA
  ram <- snap$ram_usage_pct %||% NA
  tags$div(
    tags$div(style = "display:flex;gap:16px;flex-wrap:wrap;margin-bottom:12px",
      tags$div(style = "flex:1;min-width:120px;border:1px solid #00e5ff33;border-radius:8px;padding:12px;background:#0a1628",
        tags$div(style = "color:var(--dim);font-size:11px", "CPU"),
        tags$div(style = "color:var(--cyan);font-size:22px;font-weight:700",
                 if (is.na(cpu)) "—" else sprintf("%.1f%%", cpu)),
        if (!is.na(cpu)) meter_el(cpu, cpu >= 75)),
      tags$div(style = "flex:1;min-width:120px;border:1px solid #00e5ff33;border-radius:8px;padding:12px;background:#0a1628",
        tags$div(style = "color:var(--dim);font-size:11px", "RAM"),
        tags$div(style = "color:var(--cyan);font-size:22px;font-weight:700",
                 if (is.na(ram)) "—" else sprintf("%.1f%%", ram)),
        if (!is.na(ram)) meter_el(ram, ram >= 80)),
      tags$div(style = "flex:1;min-width:120px;border:1px solid #00e5ff33;border-radius:8px;padding:12px;background:#0a1628",
        tags$div(style = "color:var(--dim);font-size:11px", "HEALTH"),
        tags$div(style = paste0("font-size:18px;font-weight:700;color:",
                 if (identical(snap$health, "good")) "#00e676"
                 else if (identical(snap$health, "warning")) "#ffc107" else "#ff1744"),
                 toupper(snap$health %||% "?")))),
    tags$div(style = "font-size:11px;color:var(--dim);margin-bottom:8px",
             paste0(snap$hostname %||% "", " · ", snap$os %||% "", " · source: ", snap$source %||% "")),
    if (length(snap$issues) > 0)
      tags$div(
        lapply(snap$issues, function(iss) {
          tags$div(class = "issue warn", style = "margin-bottom:6px",
            tags$span(class = "ico", "⚠"),
            tags$div(class = "body", tags$div(class = "t", as.character(iss))))
        }))
    else
      tags$div(class = "status-banner INFO", "Hakuna tahadhari kutoka probe")
  )
}

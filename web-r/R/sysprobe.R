# ============================================================
# sysprobe.R — metrics HALISI za OS + live agent_data + learner
# ============================================================

`%||%` <- function(a, b) if (is.null(a) || length(a) == 0 || (is.character(a) && !nzchar(a))) b else a

.sysprobe_run <- function(cmd) {
  tryCatch({
    out <- system(cmd, intern = TRUE, ignore.stderr = TRUE)
    if (length(out) == 0 || inherits(out, "try-error")) return(NULL)
    out
  }, error = function(e) NULL)
}

sysprobe_is_windows <- function() .Platform$OS.type == "windows"

sysprobe_cpu_pct <- function() {
  if (sysprobe_is_windows()) {
    o <- .sysprobe_run("wmic cpu get loadpercentage /value")
    if (!is.null(o)) {
      line <- grep("LoadPercentage", o, value = TRUE)
      if (length(line)) {
        v <- as.numeric(sub(".*=", "", line[1]))
        if (!is.na(v)) return(v)
      }
    }
  } else {
    t <- .sysprobe_run("top -bn1 2>/dev/null | grep -E 'Cpu\(s\)|%Cpu' | head -1")
    if (!is.null(t)) {
      m <- regmatches(t, regexpr("[0-9]+\\.[0-9]+", t))
      if (length(m)) return(as.numeric(m[1]))
    }
    o <- .sysprobe_run("top -l 1 -n 0 2>/dev/null | grep 'CPU usage'")
    if (!is.null(o)) {
      m <- regmatches(o, regexpr("[0-9]+\\.[0-9]+", o))
      if (length(m)) return(as.numeric(m[1]))
    }
  }
  NA_real_
}

sysprobe_mem <- function() {
  if (sysprobe_is_windows()) {
    o <- .sysprobe_run("wmic OS get FreePhysicalMemory,TotalVisibleMemorySize /value")
    if (!is.null(o)) {
      free <- as.numeric(sub(".*=", "", grep("FreePhysicalMemory", o, value = TRUE)[1]))
      tot  <- as.numeric(sub(".*=", "", grep("TotalVisibleMemorySize", o, value = TRUE)[1]))
      if (!is.na(free) && !is.na(tot) && tot > 0)
        return(list(total_mb = tot / 1024, used_mb = (tot - free) / 1024,
                    pct = ((tot - free) / tot) * 100))
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
  }
  list(total_mb = NA_real_, used_mb = NA_real_, pct = NA_real_)
}

sysprobe_disk <- function() {
  if (sysprobe_is_windows()) {
    o <- .sysprobe_run('wmic logicaldisk where DriveType=3 get DeviceID,Size,FreeSpace /value')
    if (is.null(o)) return(list())
    ids <- gsub(".*=", "", grep("DeviceID", o, value = TRUE))
    sizes <- as.numeric(gsub(".*=", "", grep("^Size=", o, value = TRUE)))
    frees <- as.numeric(gsub(".*=", "", grep("FreeSpace", o, value = TRUE)))
    out <- list()
    for (i in seq_along(ids)) {
      if (i > length(sizes) || is.na(sizes[i]) || sizes[i] <= 0) next
      used_pct <- ((sizes[i] - frees[i]) / sizes[i]) * 100
      st <- if (used_pct >= 92) "critical" else if (used_pct >= 80) "warning" else "good"
      out[[length(out) + 1]] <- list(mount = ids[i], used_pct = used_pct, status = st,
                                      total_gb = sizes[i] / (1024^3), available_gb = frees[i] / (1024^3))
    }
    return(out)
  }
  o <- .sysprobe_run("df -P 2>/dev/null | awk 'NR>1 && $6 ~ /^\\//{print $6,$5,$2,$4}'")
  if (is.null(o)) return(list())
  Filter(Negate(is.null), lapply(o, function(line) {
    parts <- strsplit(trimws(line), "\\s+")[[1]]
    if (length(parts) < 2) return(NULL)
    pct <- as.numeric(gsub("%", "", parts[2]))
    st <- if (is.na(pct)) "unknown" else if (pct >= 92) "critical" else if (pct >= 80) "warning" else "good"
    list(mount = parts[1], used_pct = pct, status = st)
  }))
}

sysprobe_top_procs <- function(n = 10) {
  if (sysprobe_is_windows()) {
    o <- .sysprobe_run("wmic process get Name,ProcessId,WorkingSetSize /format:csv")
    if (is.null(o) || length(o) < 3) return(list())
    # CSV: Node,Name,ProcessId,WorkingSetSize
    rows <- o[grep(",", o)]
    rows <- rows[!grepl("^Node,", rows)]
    parsed <- lapply(rows, function(line) {
      p <- strsplit(line, ",")[[1]]
      if (length(p) < 4) return(NULL)
      list(name = p[2], pid = p[3], cpu = NA_real_,
           ram = as.numeric(p[4]) / (1024 * 1024), status = "good")
    })
    parsed <- Filter(Negate(is.null), parsed)
    parsed <- parsed[order(vapply(parsed, function(x) x$ram %||% 0, numeric(1)), decreasing = TRUE)]
    return(utils::head(parsed, n))
  }
  o <- .sysprobe_run(sprintf("ps -eo pid,pcpu,pmem,comm --sort=-pcpu 2>/dev/null | head -n %d", n + 1))
  if (is.null(o) || length(o) < 2) return(list())
  Filter(Negate(is.null), lapply(o[-1], function(line) {
    parts <- strsplit(trimws(line), "\\s+")[[1]]
    if (length(parts) < 4) return(NULL)
    cpu <- as.numeric(parts[2]); ram <- as.numeric(parts[3])
    st <- if (!is.na(cpu) && cpu >= 80) "critical" else if (!is.na(cpu) && cpu >= 40) "warning" else "good"
    list(pid = parts[1], cpu = cpu, ram = ram,
         name = paste(parts[4:length(parts)], collapse = " "), status = st)
  }))
}

sysprobe_snapshot <- function() {
  cpu <- sysprobe_cpu_pct()
  mem <- sysprobe_mem()
  disks <- sysprobe_disk()
  procs <- sysprobe_top_procs(10)

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
  for (p in procs) {
    if (identical(p$status, "critical"))
      issues <- c(issues, sprintf("Mchakato hatari: %s (PID %s) CPU %.1f%%",
                                  p$name %||% "?", p$pid %||% "?", p$cpu %||% 0))
  }

  health <- if (length(issues) == 0) "good" else if (any(grepl("karibu|sana|hatari", issues))) "critical" else "warning"

  list(
    timestamp = as.numeric(Sys.time()),
    source = "r-sysprobe-live",
    hostname = Sys.info()[["nodename"]],
    os = paste(Sys.info()[["sysname"]], Sys.info()[["release"]]),
    kernel = Sys.info()[["version"]],
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

.status_pct <- function(pct) {
  if (is.null(pct) || is.na(pct)) return("unknown")
  if (pct >= 90) "critical" else if (pct >= 75) "warning" else "good"
}

#' Unda agent_data HALISI kwa Vision UI (components, processes, topology, issues)
build_live_agent_data <- function(snap = NULL) {
  if (is.null(snap)) snap <- sysprobe_snapshot()
  cpu <- snap$cpu_usage_pct %||% NA
  ram <- snap$ram_usage_pct %||% NA
  disk0 <- if (length(snap$disks) > 0) snap$disks[[1]] else list(used_pct = NA, mount = "?")
  d_pct <- disk0$used_pct %||% NA

  components <- list(
    list(id = "cpu", name = sprintf("CPU %s%%", if (is.na(cpu)) "?" else round(cpu, 1)),
         icon = "🧠", status = .status_pct(cpu), x = 300, y = 180),
    list(id = "ram", name = sprintf("RAM %s%%", if (is.na(ram)) "?" else round(ram, 1)),
         icon = "💾", status = .status_pct(ram), x = 180, y = 120),
    list(id = "disk", name = sprintf("Disk %s (%s)", if (is.na(d_pct)) "?" else paste0(round(d_pct), "%"),
                                                    disk0$mount %||% "?"),
         icon = "🗄️", status = .status_pct(d_pct), x = 180, y = 260),
    list(id = "os_kernel", name = paste("OS", snap$os %||% "?"),
         icon = "⚙️", status = snap$health %||% "good", x = 300, y = 80),
    list(id = "network", name = "Network",
         icon = "🌐", status = "good", x = 510, y = 180),
    list(id = "motherboard", name = paste("Host:", snap$hostname %||% "?"),
         icon = "🔩", status = "good", x = 300, y = 300)
  )

  processes <- lapply(snap$top_processes %||% list(), function(p) {
    list(
      name = sprintf("%s [%s]", p$name %||% "?", p$pid %||% "?"),
      cpu = p$cpu %||% 0,
      ram = p$ram %||% 0,
      status = p$status %||% "good"
    )
  })

  topology <- list(
    nodes = list(
      list(id = "host", label = snap$hostname %||% "host", x = 300, y = 40, status = "online"),
      list(id = "cpu_n", label = sprintf("CPU %s", if (is.na(cpu)) "?" else paste0(round(cpu), "%")),
           x = 120, y = 140, status = if (identical(.status_pct(cpu), "critical")) "warning" else "online"),
      list(id = "ram_n", label = sprintf("RAM %s", if (is.na(ram)) "?" else paste0(round(ram), "%")),
           x = 300, y = 140, status = "online"),
      list(id = "disk_n", label = sprintf("Disk %s", if (is.na(d_pct)) "?" else paste0(round(d_pct), "%")),
           x = 480, y = 140, status = "online")
    ),
    edges = list(
      list(from = "host", to = "cpu_n"),
      list(from = "host", to = "ram_n"),
      list(from = "host", to = "disk_n")
    )
  )

  # attach process nodes
  procs <- snap$top_processes %||% list()
  if (length(procs) > 0) {
    for (i in seq_len(min(5, length(procs)))) {
      pr <- procs[[i]]
      id <- paste0("p", i)
      topology$nodes[[length(topology$nodes) + 1]] <- list(
        id = id,
        label = substr(pr$name %||% "?", 1, 12),
        x = 80 + (i - 1) * 100, y = 280,
        status = if (identical(pr$status, "critical")) "warning" else "online"
      )
      topology$edges[[length(topology$edges) + 1]] <- list(from = "cpu_n", to = id)
    }
  }

  iss_raw <- snap$issues %||% list()
  issues <- if (length(iss_raw) == 0) {
    list(list(
      title = "Mfumo uko sawa",
      desc = sprintf("CPU %s · RAM %s · health=%s", cpu, ram, snap$health %||% "?"),
      action = "Hakuna hatua ya dharura"
    ))
  } else {
    lapply(iss_raw, function(t) {
      t <- as.character(t)
      action <- if (grepl("CPU", t, ignore.case = TRUE))
        "Angalia michakato yenye CPU juu"
      else if (grepl("RAM", t, ignore.case = TRUE))
        "Funga programu za RAM nyingi"
      else if (grepl("Disk", t, ignore.case = TRUE))
        "Futa faili za muda"
      else "Chunguza OS metrics"
      list(title = t,
           desc = sprintf("OS probe @ %s | %s", snap$hostname %||% "", snap$os %||% ""),
           action = action)
    })
  }

  list(
    source = snap$source %||% "r-live",
    components = components,
    processes = processes,
    topology = topology,
    issues = issues,
    probe = snap
  )
}

#' Hifadhi session kwenye learning_log.json (kweli)
append_learning_log <- function(session_id, msg, issues, status) {
  roots <- c(
    if (exists("p_root")) p_root("data") else NULL,
    if (exists("p_app")) p_app("data") else NULL,
    "data", "../data"
  )
  roots <- Filter(function(r) !is.null(r) && dir.exists(r), roots)
  if (length(roots) == 0) return(invisible(FALSE))
  path <- file.path(roots[[1]], "learning_log.json")
  arr <- if (file.exists(path)) {
    tryCatch(jsonlite::fromJSON(path, simplifyVector = FALSE), error = function(e) list())
  } else list()
  if (!is.list(arr)) arr <- list()
  arr[[length(arr) + 1]] <- list(
    session_id = session_id,
    ts = as.numeric(Sys.time()),
    user_message = msg,
    issues = issues,
    status = status,
    source = "r-shiny"
  )
  if (length(arr) > 200) arr <- arr[(length(arr) - 199):length(arr)]
  tryCatch({
    writeLines(jsonlite::toJSON(arr, auto_unbox = TRUE, pretty = TRUE),
               path, useBytes = TRUE)
    invisible(TRUE)
  }, error = function(e) invisible(FALSE))
}

sysprobe_panel_el <- function(snap, lang = "sw") {
  if (is.null(snap))
    return(tags$div(style = "color:var(--dim)", "Probe haipatikani"))
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
             paste0(snap$hostname %||% "", " · ", snap$os %||% "", " · ", snap$source %||% "")),
    if (length(snap$issues) > 0)
      tags$div(lapply(snap$issues, function(iss) {
        tags$div(class = "issue warn", style = "margin-bottom:6px",
          tags$span(class = "ico", "⚠"),
          tags$div(class = "body", tags$div(class = "t", as.character(iss))))
      }))
    else
      tags$div(class = "status-banner INFO", "Hakuna tahadhari — metrics za OS ziko sawa")
  )
}

# ============================================================
# solve.R — Software solve (baada ya HITL) | Hardware = binadamu
# ============================================================

load_solve_policy <- function() {
  f <- find_data_file("vision/solve_policy.json")
  if (is.null(f))
    return(list(
      software_cause_keywords = c("virus", "software_corrupt", "temp", "slow_pc", "cache"),
      hardware_cause_keywords = c("psu", "hdd_fail", "ram_bad", "motherboard", "overheat", "fan"),
      software_actions = list(
        list(id = "report_top_cpu", matches = c("slow", "cpu"), requires_hitl = FALSE),
        list(id = "list_temp", matches = c("temp", "slow"), requires_hitl = FALSE),
        list(id = "clear_user_temp", matches = c("temp", "slow_pc", "disk"), requires_hitl = TRUE)
      ),
      hardware_guide_template_sw =
        "HARDWARE — Agent haifanyi ukarabati. Binadamu: zima umeme, fuata suluhisho, badilisha sehemu."
    ))
  jsonlite::fromJSON(f, simplifyVector = FALSE)
}

classify_domain <- function(blob, policy) {
  blob <- tolower(blob)
  sw <- sum(vapply(policy$software_cause_keywords %||% list(), function(k)
    as.integer(grepl(k, blob, fixed = TRUE)), integer(1)))
  hw <- sum(vapply(policy$hardware_cause_keywords %||% list(), function(k)
    as.integer(grepl(k, blob, fixed = TRUE)), integer(1)))
  if (sw > 0 && hw > 0) "mixed" else if (sw > 0) "software" else if (hw > 0) "hardware" else "unknown"
}

.problem_blob <- function(p) {
  paste(
    p$id %||% "",
    p$trade %||% "",
    paste(unlist(p$symptoms %||% list()), collapse = " "),
    paste(names(p$causes %||% list()), collapse = " "),
    .bi(p$solution, "sw"),
    collapse = " "
  )
}

pick_software_actions <- function(blob, policy) {
  blob <- tolower(blob)
  ids <- character(0)
  for (a in policy$software_actions %||% list()) {
    ms <- unlist(a$matches %||% list())
    if (any(vapply(ms, function(m) grepl(m, blob, fixed = TRUE), logical(1))))
      ids <- c(ids, a$id)
  }
  unique(ids)
}

#' Plan + optional execute (software only after hitl)
agentic_solve <- function(msg, matched_problem = NULL, hitl_approved = FALSE, lang = "sw") {
  policy <- load_solve_policy()
  blob <- if (!is.null(matched_problem)) .problem_blob(matched_problem) else tolower(msg)
  domain <- classify_domain(blob, policy)
  solution_sw <- if (!is.null(matched_problem)) .bi(matched_problem$solution, lang) else ""
  problem_id <- if (!is.null(matched_problem)) matched_problem$id %||% NULL else NULL

  action_ids <- if (domain %in% c("software", "mixed")) pick_software_actions(blob, policy) else character(0)
  if (length(action_ids) == 0 && domain %in% c("software", "mixed", "unknown")) {
    if (grepl("slow|temp|disk|cpu", blob)) action_ids <- c("report_top_cpu", "list_temp")
  }

  hardware_guide <- NULL
  if (domain %in% c("hardware", "mixed")) {
    hardware_guide <- paste(
      policy$hardware_guide_template_sw %||% "HARDWARE: binadamu.",
      "\n\nKnowledge:\n", solution_sw
    )
  }

  executed <- list()
  skipped <- character(0)

  if (domain == "hardware") {
    return(list(
      domain = domain,
      problem_id = problem_id,
      action_ids = character(0),
      executed = executed,
      skipped = "hardware: no auto-execute",
      hardware_guide_sw = hardware_guide,
      knowledge_solution_sw = solution_sw,
      summary_sw = hardware_guide %||% "Hardware — subiri binadamu.",
      hitl_approved = hitl_approved
    ))
  }

  if (!isTRUE(hitl_approved)) {
    return(list(
      domain = domain,
      problem_id = problem_id,
      action_ids = action_ids,
      executed = executed,
      skipped = action_ids,
      hardware_guide_sw = hardware_guide,
      knowledge_solution_sw = solution_sw,
      summary_sw = "HITL haijaruhusiwa — bofya RUHUSU kwanza.",
      hitl_approved = FALSE
    ))
  }

  # Execute allowlisted software actions via R helpers / system
  for (aid in action_ids) {
    res <- tryCatch(run_software_action(aid), error = function(e)
      list(ok = FALSE, message_sw = conditionMessage(e), detail = ""))
    executed[[length(executed) + 1]] <- c(list(action_id = aid), res)
    if (!isTRUE(res$ok)) skipped <- c(skipped, paste(aid, res$message_sw))
  }

  summary_sw <- if (domain == "mixed") {
    paste0("SOFTWARE: ", length(executed), " vitendo. HARDWARE (binadamu):\n", hardware_guide %||% "")
  } else if (length(executed) == 0) {
    paste0("Hakuna action. Knowledge: ", solution_sw)
  } else {
    paste0("Software solve: ", length(executed), " vitendo. ", solution_sw)
  }

  list(
    domain = domain,
    problem_id = problem_id,
    action_ids = action_ids,
    executed = executed,
    skipped = skipped,
    hardware_guide_sw = hardware_guide,
    knowledge_solution_sw = solution_sw,
    summary_sw = summary_sw,
    hitl_approved = TRUE
  )
}

run_software_action <- function(id) {
  if (identical(id, "list_temp")) {
    td <- tempdir()
    n <- length(list.files(td, all.files = TRUE, no.. = TRUE))
    return(list(ok = TRUE, message_sw = paste0("Temp: ", td, " (", n, " entries)"), detail = td))
  }
  if (identical(id, "clear_user_temp")) {
    td <- tempdir()
    files <- list.files(td, full.names = TRUE, all.files = FALSE)
    removed <- 0L
    for (f in files) {
      if (file.exists(f) && !dir.exists(f)) {
        if (unlink(f) == 0) removed <- removed + 1L
      }
    }
    return(list(ok = TRUE, message_sw = paste0("Zimefutwa faili ", removed, " katika temp"), detail = td))
  }
  if (identical(id, "report_top_cpu")) {
    snap <- tryCatch(sysprobe_snapshot(), error = function(e) NULL)
    if (is.null(snap) || length(snap$processes) == 0)
      return(list(ok = TRUE, message_sw = "Hakuna process list (probe)", detail = ""))
    lines <- vapply(head(snap$processes, 8), function(p) {
      paste0(p$name %||% "?", " cpu=", p$cpu %||% "?")
    }, character(1))
    return(list(ok = TRUE, message_sw = "Orodha CPU juu", detail = paste(lines, collapse = "\n")))
  }
  if (identical(id, "sync_disk")) {
    if (.Platform$OS.type == "unix") {
      system("sync", ignore.stdout = TRUE, ignore.stderr = TRUE)
      return(list(ok = TRUE, message_sw = "sync imefanikiwa", detail = ""))
    }
    return(list(ok = FALSE, message_sw = "sync si Windows", detail = ""))
  }
  list(ok = FALSE, message_sw = paste("Action haijulikani:", id), detail = "")
}

solve_result_ui <- function(res, lang = "sw") {
  if (is.null(res))
    return(tags$div(style = "color:var(--dim)", "Bofya RUHUSU baada ya ANZA ili solve."))
  dom <- res$domain %||% "unknown"
  col <- switch(dom, software = "#00e676", hardware = "#ffc107", mixed = "#00e5ff", "#90a4ae")
  tags$div(
    tags$div(style = paste0("border-left:4px solid ", col, ";padding:10px;background:#0a1628;margin-bottom:10px"),
      tags$div(style = paste0("color:", col, ";font-weight:700"), paste0("Domain: ", toupper(dom))),
      tags$div(style = "font-size:12px;margin-top:6px;color:#b2ebf2;white-space:pre-wrap", res$summary_sw %||% "")),
    if (length(res$executed %||% list()) > 0)
      tags$div(style = "margin-top:8px",
        tags$div(style = "color:var(--cyan);font-size:12px;font-weight:700", "Vitendo vilivyotekelezwa"),
        lapply(res$executed, function(x) {
          tags$div(style = "font-size:11px;padding:6px;border:1px solid #00e5ff22;margin:4px 0",
            tags$b(x$action_id %||% ""), " — ", x$message_sw %||% "",
            if (nzchar(x$detail %||% ""))
              tags$pre(style = "font-size:10px;color:var(--dim)", x$detail))
        })),
    if (!is.null(res$hardware_guide_sw) && nzchar(res$hardware_guide_sw))
      tags$div(class = "status-banner WARNING", style = "margin-top:10px;white-space:pre-wrap",
               res$hardware_guide_sw)
  )
}

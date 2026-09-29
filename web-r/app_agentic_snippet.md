# Jinsi ya kuunganisha tab AGENTIC kwenye app.R

Baada ya `source(... views.R)` ongeza:

```r
source(p_app("R", "agentic.R"), local = FALSE)
source(p_app("R", "agentic_i18n_patch.R"), local = FALSE)
AGENTIC <- load_agentic_data()
```

Kwenye UI, baada ya conditionalPanel map:

```r
conditionalPanel(condition = "input.tab == 'agentic'", uiOutput("agentic")),
```

Kwenye tabs_el (views.R) ongeza id `agentic` na label `tr("tabs.agentic", lang)`.

Kwenye server:

```r
av_state <- reactiveVal(list(pipe_idx = 0, status = "ready", log = character()))

output$agentic <- renderUI({
  view_agentic(lang(), AGENTIC, av_state())
})

observeEvent(input$av_start, {
  msg <- isolate(input$av_msg %||% "")
  if (!nzchar(trimws(msg))) msg <- "Tatizo la kifaa — scan automatic"
  issues <- AGENTIC$agent_data$issues %||% list()
  n <- length(issues)
  lines <- c(
    paste0("Session imeanza: ", format(Sys.time(), "%H:%M:%S")),
    paste0("Ujumbe: ", msg),
    paste0("Vision: matatizo ", n, " yamegunduliwa"),
    "Pipeline: Panga → Tambua (inangojea RUHUSU HITL)"
  )
  av_state(list(pipe_idx = 1, status = "hitl", log = lines))
})

observeEvent(input$av_approve, {
  st <- av_state()
  lines <- c(st$log %||% character(),
             "HITL: ruhusa imetolewa",
             "Tekeleza → Jaribu → Thibitisha → Andika",
             "Ripoti: kitabu kidigitali kiko tayari")
  av_state(list(pipe_idx = 6, status = "done", log = lines))
})

observeEvent(input$av_scan, {
  st <- av_state()
  av_state(list(pipe_idx = st$pipe_idx %||% 0, status = "running",
                log = c(st$log, paste0("Scan upya @ ", format(Sys.time(), "%H:%M:%S")))))
})

output$av_session_status <- renderUI({
  st <- av_state()
  msg <- switch(st$status %||% "ready",
    ready = tr("av.session.ready", lang()),
    running = tr("av.session.running", lang()),
    hitl = tr("av.session.hitl", lang()),
    done = tr("av.session.done", lang()),
    tr("av.session.ready", lang()))
  tags$div(class = paste("status-banner",
      if (identical(st$status, "done")) "GOOD" else if (identical(st$status, "hitl")) "WARNING" else "INFO"),
    msg)
})

output$av_report_out <- renderUI({
  st <- av_state()
  tags$pre(style = "white-space:pre-wrap;color:#b2ebf2;font-size:12px",
           paste(st$log %||% tr("av.session.ready", lang()), collapse = "\n"))
})
```

Badilisha `tabs_el` ids kuwa:
`c("live", "agentic", "formula", "diag", "viz", "map")`

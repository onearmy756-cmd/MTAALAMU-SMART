# grounded.R — stats/calculus helpers (no hallucination)
# source after data_bridge.R / devices_bridge.R

grounded_load_index <- function(cpu, mem, disk) {
  list(
    P = 0.4 * cpu + 0.35 * mem + 0.25 * disk,
    dP_dCPU = 0.4, dP_dMEM = 0.35, dP_dDISK = 0.25
  )
}

grounded_logistic_risk <- function(P) {
  r <- 1 / (1 + exp(-(P - 50) / 10))
  h <- 0.5
  r_hi <- 1 / (1 + exp(-((P + h) - 50) / 10))
  r_lo <- 1 / (1 + exp(-((P - h) - 50) / 10))
  list(r = r, dr_dP = (r_hi - r_lo) / (2 * h))
}

grounded_softmax <- function(x) {
  x <- as.numeric(x)
  m <- max(x)
  e <- exp(x - m)
  e / sum(e)
}

grounded_summary <- function(msg, cpu = NA, mem = NA, disk = NA) {
  msg <- as.character(msg %||% "")
  hits <- tryCatch(match_problems(msg, limit = 5), error = function(e) list())
  devs <- tryCatch(match_devices(msg, limit = 5), error = function(e) list())
  calc <- list()
  if (!is.na(cpu) && !is.na(mem) && !is.na(disk)) {
    li <- grounded_load_index(cpu, mem, disk)
    lr <- grounded_logistic_risk(li$P)
    sm <- grounded_softmax(c(cpu, mem, disk))
    calc <- list(load = li, risk = lr, softmax = as.numeric(sm))
  }
  list(
    query = msg,
    evidence_problems = length(hits),
    evidence_devices = length(devs),
    calculus = calc,
    note_sw = "Majibu yanatokana na data/JSON + hesabu; si ubunifu wa ukweli."
  )
}

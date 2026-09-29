# ============================================================
# eval.R — INJINI YA HESABU (mirror ya React evalExpr/evalCond/calculate)
#           + Bayes inference (DiagnosisTab)
# Hakuna toleo "takwimu" — formula zote zinasomwa kutoka JSON.
# ============================================================

# makosa ya hesabu (yenye lugha mbili)
calc_error <- function(sw, en) {
  structure(class = c("calc_error", "error", "condition"),
            list(message = paste(sw, en, sep = " / "), call = NULL, sw = sw, en = en))
}
err_msg <- function(e, lang) if (inherits(e, "calc_error")) e[[lang]] %||% e$message else conditionMessage(e)

FUNCS <- c("sqrt", "pow", "ceil", "floor", "abs", "log", "min", "max", "exp", "if")
OPS_ARITH <- c("+", "-", "*", "/", "^", "%")

# mfumo wa kazi (JS -> R): if/ceil/pow hazipo moja kwa moja kwenye R
call_func <- function(fname, a) {
  switch(fname,
    sqrt  = sqrt(a[[1]]),
    pow   = a[[1]] ^ a[[2]],
    ceil  = ceiling(a[[1]]),
    floor = floor(a[[1]]),
    abs   = abs(a[[1]]),
    log   = log(a[[1]]),
    min   = do.call(min, a),
    max   = do.call(max, a),
    exp   = exp(a[[1]]),
    `if`  = if (!is.na(a[[1]]) && a[[1]] != 0) a[[2]] else if (length(a) >= 3) a[[3]] else NA,
    stop(calc_error(paste0("Kazi haipo: ", fname), paste0("Function not found: ", fname)))
  )
}

# ---------------- tokenizer ----------------
tokenize <- function(src) {
  ch <- strsplit(src, "", fixed = TRUE)[[1]]
  n <- length(ch); i <- 1; toks <- list()
  while (i <= n) {
    c1 <- ch[i]
    if (c1 %in% c(" ", "\t", "\n", "\r")) { i <- i + 1; next }
    if (grepl("^[0-9.]$", c1)) {
      st <- i
      while (i <= n && grepl("^[0-9.]$", ch[i])) i <- i + 1
      val <- suppressWarnings(as.numeric(paste(ch[st:(i - 1)], collapse = "")))
      if (is.na(val)) {
        bad <- paste(ch[st:(i - 1)], collapse = "")
        stop(calc_error(paste0("Namba si sahihi: ", bad), paste0("Invalid number: ", bad)))
      }
      toks[[length(toks) + 1]] <- list(t = "n", v = val)
    } else if (c1 %in% OPS_ARITH) {
      toks[[length(toks) + 1]] <- list(t = "o", v = c1); i <- i + 1
    } else if (c1 == "(") { toks[[length(toks) + 1]] <- list(t = "("); i <- i + 1 }
    else if (c1 == ")") { toks[[length(toks) + 1]] <- list(t = ")"); i <- i + 1 }
    else if (c1 == ",") { toks[[length(toks) + 1]] <- list(t = ","); i <- i + 1 }
    else if (grepl("^[a-zA-Z_]$", c1)) {
      st <- i
      while (i <= n && grepl("^[a-zA-Z0-9_]$", ch[i])) i <- i + 1
      w <- paste(ch[st:(i - 1)], collapse = "")
      toks[[length(toks) + 1]] <- list(t = if (w %in% FUNCS) "f" else "i", v = w)
    } else {
      stop(calc_error(paste0("Herufi isiyotarajiwa: ", c1),
                       paste0("Unexpected character: ", c1)))
    }
  }
  toks
}

# ---------------- recursive-descent parser ----------------
eval_expr <- function(src, vars) {
  toks <- tokenize(src)
  if (length(toks) == 0) stop(calc_error("Expression tupu", "Empty expression"))
  st <- new.env(parent = emptyenv()); st$i <- 1

  peek <- function() if (st$i <= length(toks)) toks[[st$i]] else NULL
  next_tok <- function() { tk <- peek(); st$i <- st$i + 1; tk }

  resolve <- function(name) {
    if (name %in% names(vars)) return(as.numeric(vars[[name]]))
    if (identical(name, "pi")) return(pi)
    stop(calc_error(paste0("Variable '", name, "' haipo"), paste0("Variable '", name, "' does not exist")))
  }

  primary <- function() {
    tk <- peek()
    if (is.null(tk)) stop(calc_error("Expression haikamilika", "Incomplete expression"))
    if (tk$t == "n") { st$i <- st$i + 1; return(tk$v) }
    if (tk$t == "i") { st$i <- st$i + 1; return(resolve(tk$v)) }
    if (tk$t == "f") {
      fname <- tk$v; st$i <- st$i + 1
      op <- peek()
      if (is.null(op) || op$t != "(") stop(calc_error("Function inahitaji mabano", "Function requires parentheses"))
      st$i <- st$i + 1
      args <- list()
      repeat {
        args[[length(args) + 1]] <- expr_()
        nx <- peek()
        if (is.null(nx)) stop(calc_error("Function haijakamilika", "Function not completed"))
        if (nx$t == ",") { st$i <- st$i + 1; next }
        if (nx$t == ")") { st$i <- st$i + 1; break }
        stop(calc_error("Function haijakamilika", "Function not completed"))
      }
      return(call_func(fname, args))
    }
    if (tk$t == "(") {
      st$i <- st$i + 1
      v <- expr_()
      nx <- peek()
      if (is.null(nx) || nx$t != ")") stop(calc_error("Mabano hayajafungwa", "Parentheses not closed"))
      st$i <- st$i + 1
      return(v)
    }
    stop(calc_error("Thamani inahitajika", "Expecting value"))
  }

  unary <- function() {
    tk <- peek()
    if (!is.null(tk) && tk$t == "o" && tk$v == "-") { st$i <- st$i + 1; return(-unary()) }
    if (!is.null(tk) && tk$t == "o" && tk$v == "+") { st$i <- st$i + 1; return(unary()) }
    power()
  }
  power <- function() {
    base <- primary()
    tk <- peek()
    if (!is.null(tk) && tk$t == "o" && tk$v == "^") { st$i <- st$i + 1; return(base ^ power()) }
    base
  }
  term <- function() {
    left <- unary()
    repeat {
      tk <- peek()
      if (is.null(tk) || tk$t != "o" || !(tk$v %in% c("*", "/", "%"))) break
      op <- tk$v; st$i <- st$i + 1
      right <- unary()
      if (op == "*") left <- left * right
      else if (op == "/") {
        if (right == 0) stop(calc_error("Kugawanya kwa sifuri", "Division by zero"))
        left <- left / right
      } else left <- left %% right
    }
    left
  }
  expr_ <- function() {
    left <- term()
    repeat {
      tk <- peek()
      if (is.null(tk) || tk$t != "o" || !(tk$v %in% c("+", "-"))) break
      op <- tk$v; st$i <- st$i + 1
      right <- term()
      left <- if (op == "+") left + right else left - right
    }
    left
  }

  res <- expr_()
  if (st$i != length(toks) + 1) stop(calc_error("Expression haikamilika", "Expression not completed"))
  res
}

# ---------------- masharti (rules) ----------------
eval_cond <- function(cond, vars) {
  cnd <- trimws(cond)
  if (identical(cnd, "true")) return(TRUE)
  if (grepl("&&", cnd, fixed = TRUE)) {
    parts <- strsplit(cnd, "&&", fixed = TRUE)[[1]]
    return(all(vapply(parts, eval_cond, logical(1), vars = vars)))
  }
  if (grepl("||", cnd, fixed = TRUE)) {
    parts <- strsplit(cnd, "||", fixed = TRUE)[[1]]
    return(any(vapply(parts, eval_cond, logical(1), vars = vars)))
  }
  for (op in c("<=", ">=", "<", ">", "==", "!=")) {
    if (grepl(op, cnd, fixed = TRUE)) {
      pos <- regexpr(op, cnd, fixed = TRUE)[1]
      l <- substr(cnd, 1, pos - 1)
      r <- substr(cnd, pos + nchar(op), nchar(cnd))
      res <- function(t) {
        s <- trimws(t)
        num <- suppressWarnings(as.numeric(s))
        if (!is.na(num)) return(num)
        if (s %in% names(vars)) return(as.numeric(vars[[s]]))
        tryCatch(eval_expr(s, vars), error = function(e) NaN)
      }
      a <- res(l); b <- res(r)
      return(switch(op,
        "<=" = a <= b, ">=" = a >= b, "<" = a < b, ">" = a > b,
        "==" = abs(a - b) < 1e-9, "!=" = abs(a - b) >= 1e-9))
    }
  }
  FALSE
}

# Math.round (JS) — kuelekea +Inf kama JS, si banker's rounding ya R
js_round <- function(x, digits = 0) {
  f <- 10^digits
  signif_trunc <- function(v) sign(v) * floor(abs(v) * f + 0.5) / f
  signif_trunc(x)
}

# ---------------- calculate(formula, input_vals) ----------------
calculate <- function(formula, input_vals, constants = list()) {
  vars <- constants %||% list()
  for (inp in formula$inputs) {
    v <- input_vals[[inp$name]]
    vars[[inp$name]] <- if (is.null(v) || is.na(v)) inp$default else as.numeric(v)
  }

  outputs <- list()
  for (o in formula$outputs) {
    raw <- eval_expr(o$expr, vars)
    vars[[o$name]] <- raw
    digits <- o$digits %||% 2
    outputs[[length(outputs) + 1]] <- list(
      name = o$name, label = o$label, unit = o$unit,
      value = js_round(raw, digits)
    )
  }

  steps <- list()
  for (s in formula$steps %||% list()) {
    if (!is.null(s$text)) {
      steps[[length(steps) + 1]] <- list(label = s$label, text = s$text)
    } else {
      v <- eval_expr(s$expr, vars)
      num <- as.numeric(sprintf("%.2f", v))
      tpl <- s$template %||% "{{result}}"
      steps[[length(steps) + 1]] <- list(
        label = s$label,
        text = gsub("{{result}}", format(num, trim = TRUE, scientific = FALSE), tpl, fixed = TRUE)
      )
    }
  }

  status <- "INFO"; message <- list(sw = "", en = "")
  for (r in formula$rules %||% list()) {
    if (isTRUE(eval_cond(r$check, vars))) {
      status <- r$status
      message <- r$msg %||% list(sw = "", en = "")
      break
    }
  }
  list(outputs = outputs, steps = steps, status = status, message = message)
}

# ---------------- Bayes (DiagnosisTab) ----------------
bayes_causes <- function(model, on) {
  post <- vapply(model$causes, function(cause) {
    p <- cause$prior
    for (s in model$symptoms) {
      lik <- 0.2
      if (!is.null(s$likelihood) && !is.null(s$likelihood[[cause$id]])) {
        lik <- as.numeric(s$likelihood[[cause$id]])
      }
      p <- p * if (s$id %in% on) lik else (1 - lik)
    }
    p
  }, numeric(1))

  total <- sum(post)
  if (!is.finite(total) || total == 0) total <- 1

  res <- lapply(seq_along(model$causes), function(i) {
    cse <- model$causes[[i]]
    list(id = cse$id, name = cse$name, fix = cse$fix, cost_tzs = cse$cost_tzs,
         prob = js_round(post[i] / total * 1000, 0) / 10)
  })
  res[order(vapply(res, function(x) x$prob, numeric(1)), decreasing = TRUE)]
}

# jaribio la JSON (formula zina "test": {inputs, expect})
test_formula <- function(formula, constants = list()) {
  t <- formula$test
  if (is.null(t)) return(NULL)
  res <- tryCatch(calculate(formula, t$inputs, constants), error = function(e) NULL)
  if (is.null(res)) return(list(ok = FALSE, detail = "error"))
  ok <- TRUE; detail <- character(0)
  for (nm in names(t$expect)) {
    got <- NULL
    for (o in res$outputs) if (identical(o$name, nm)) got <- o$value
    want <- as.numeric(t$expect[[nm]])
    if (is.null(got) || abs(got - want) > max(0.05 * abs(want), 0.02)) {
      ok <- FALSE
      detail <- c(detail, sprintf("%s: %s != %s", nm, got, want))
    }
  }
  list(ok = ok, detail = paste(detail, collapse = "; "))
}

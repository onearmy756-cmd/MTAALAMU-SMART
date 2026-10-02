//! expr.rs — Data-driven expression evaluator (Rust ↔ JS grammar moja).
//! Inasoma formula kwa string: "(M * L * I * rho) / A"
//! Kisha inahesabu. Formula MOJA inafanya formulas ZOTE 130+ (trade) na 285 (network).
//!
//! Grammar (shared na React JS mirror):
//!   comparison := logical (('<'|'<='|'>'|'>='|'=='|'!=') logical)*   -> 1.0/0.0
//!   logical    := not (('&&'|'||') not)*                             -> 1.0/0.0
//!   not        := '!'? term2
//!   expr       := term (('+'|'-') term)*
//!   term       := power (('*'|'/'|'%') power)*
//!   power      := unary ('^' power)?
//!   unary      := ('-'|'+')? primary
//!   primary    := number | var | ident '(' args ')' | '(' expr ')'
//! Functions: sqrt pow ceil floor round abs log log10 exp sin cos tan min max if
//! Constants: pi, e

use std::collections::HashMap;

pub type Vars = HashMap<String, f64>;

#[derive(Debug, Clone)]
enum Token {
    Num(f64),
    Var(String),
    Op(char),
    Cmp(String),
    And,
    Or,
    Not,
    LParen,
    RParen,
    Comma,
    Ident(String),
}

#[derive(Debug)]
pub enum EvalError {
    Parse(String),
    Missing(String),
    DivZero,
}

impl std::fmt::Display for EvalError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EvalError::Parse(m) => write!(f, "{}", m),
            EvalError::Missing(v) => write!(f, "Variable '{}' haipo kwenye inputs", v),
            EvalError::DivZero => write!(f, "Kugawanya kwa sifuri"),
        }
    }
}

impl From<String> for EvalError {
    fn from(s: String) -> Self { EvalError::Parse(s) }
}
impl From<&str> for EvalError {
    fn from(s: &str) -> Self { EvalError::Parse(s.to_string()) }
}

/// Thesabu expression kwa variables zilizotolewa (Result<_, String> — serde-friendly).
pub fn eval(src: &str, vars: &Vars) -> Result<f64, String> {
    eval_with(src, vars).map_err(|e| e.to_string())
}

/// Thesabu expression kwa error type kamili.
pub fn eval_with(src: &str, vars: &Vars) -> Result<f64, EvalError> {
    let tokens = tokenize(src)?;
    let mut pos = 0usize;
    let v = parse_logical(&tokens, &mut pos, vars)?;
    if pos != tokens.len() {
        return Err(EvalError::Parse(format!(
            "Expression haikamilika (token {} baada ya mwisho)",
            pos
        )));
    }
    Ok(v)
}

/// Kagua tu: je, variable zote zinapatikana? (bila kuhesabu)
pub fn check_vars(src: &str, available: &Vars) -> Result<(), EvalError> {
    let tokens = tokenize(src)?;
    let mut pos = 0usize;
    walk(&tokens, &mut pos, available)?;
    Ok(())
}

fn walk(t: &[Token], p: &mut usize, available: &Vars) -> Result<(), EvalError> {
    // rahisi: tembelea tokens; kwa kila Var hakikisha ipo
    while *p < t.len() {
        match &t[*p] {
            Token::Var(name) => {
                if !available.contains_key(name) {
                    return Err(EvalError::Missing(name.clone()));
                }
                *p += 1;
            }
            Token::LParen => { *p += 1; }
            Token::RParen => { *p += 1; }
            Token::Comma => { *p += 1; }
            _ => { *p += 1; }
        }
    }
    Ok(())
}

fn tokenize(src: &str) -> Result<Vec<Token>, EvalError> {
    let mut out = Vec::new();
    let b: Vec<char> = src.chars().collect();
    let mut i = 0usize;
    while i < b.len() {
        let c = b[i];
        match c {
            ' ' | '\t' | '\n' | '\r' => { i += 1; }
            '0'..='9' | '.' => {
                let start = i;
                while i < b.len() && (b[i].is_ascii_digit() || b[i] == '.' || b[i] == 'e'
                    || ((b[i] == '+' || b[i] == '-') && i > start && b[i - 1] == 'e'))
                {
                    i += 1;
                }
                let s: String = b[start..i].iter().collect();
                out.push(Token::Num(
                    s.parse::<f64>().map_err(|_| EvalError::Parse(format!("Namba haieleweki: {}", s)))?,
                ));
            }
            '+' | '-' | '*' | '/' | '^' | '%' => { out.push(Token::Op(c)); i += 1; }
            '(' => { out.push(Token::LParen); i += 1; }
            ')' => { out.push(Token::RParen); i += 1; }
            ',' => { out.push(Token::Comma); i += 1; }
            '!' if i + 1 < b.len() && b[i + 1] == '=' => { out.push(Token::Cmp("!=".into())); i += 2; }
            '!' => { out.push(Token::Not); i += 1; }
            '=' if i + 1 < b.len() && b[i + 1] == '=' => { out.push(Token::Cmp("==".into())); i += 2; }
            '=' => return Err(EvalError::Parse("Tumia == kwa kulinganisha (si =)".into())),
            '<' if i + 1 < b.len() && b[i + 1] == '=' => { out.push(Token::Cmp("<=".into())); i += 2; }
            '>' if i + 1 < b.len() && b[i + 1] == '=' => { out.push(Token::Cmp(">=".into())); i += 2; }
            '<' => { out.push(Token::Cmp("<".into())); i += 1; }
            '>' => { out.push(Token::Cmp(">".into())); i += 1; }
            '&' if i + 1 < b.len() && b[i + 1] == '&' => { out.push(Token::And); i += 2; }
            '|' if i + 1 < b.len() && b[i + 1] == '|' => { out.push(Token::Or); i += 2; }
            'a'..='z' | 'A'..='Z' | '_' => {
                let start = i;
                while i < b.len() && (b[i].is_ascii_alphanumeric() || b[i] == '_') { i += 1; }
                let s: String = b[start..i].iter().collect();
                match s.as_str() {
                    "sqrt" | "pow" | "ceil" | "floor" | "round" | "abs" | "log" | "log10"
                    | "min" | "max" | "exp" | "sin" | "cos" | "tan" | "if"
                    | "true" | "false" | "pi" | "e" => out.push(Token::Ident(s)),
                    _ => out.push(Token::Var(s)),
                }
            }
            other => return Err(EvalError::Parse(format!("Herufi isiyotarajiwa: '{}'", other))),
        }
    }
    Ok(out)
}

fn peek(t: &[Token], p: usize) -> Option<&Token> { t.get(p) }

fn truthy(x: f64) -> bool { x != 0.0 }

/// logical := comparison (('&&'|'||') comparison)*  -> 1.0 / 0.0
fn parse_logical(t: &[Token], p: &mut usize, v: &Vars) -> Result<f64, EvalError> {
    let mut left = parse_comparison(t, p, v)?;
    loop {
        match peek(t, *p) {
            Some(Token::And) => {
                *p += 1;
                let right = parse_comparison(t, p, v)?;
                left = if truthy(left) && truthy(right) { 1.0 } else { 0.0 };
            }
            Some(Token::Or) => {
                *p += 1;
                let right = parse_comparison(t, p, v)?;
                left = if truthy(left) || truthy(right) { 1.0 } else { 0.0 };
            }
            _ => break,
        }
    }
    Ok(left)
}

/// not := '!'? not | expr   (! = ngazi ya juu kabisa kabla ya comparisons)
fn parse_not(t: &[Token], p: &mut usize, v: &Vars) -> Result<f64, EvalError> {
    if let Some(Token::Not) = peek(t, *p) {
        *p += 1;
        let x = parse_not(t, p, v)?;
        return Ok(if truthy(x) { 0.0 } else { 1.0 });
    }
    parse_expr(t, p, v)
}

/// comparison := not (cmp not)*  -> 1.0 / 0.0
fn parse_comparison(t: &[Token], p: &mut usize, v: &Vars) -> Result<f64, EvalError> {
    let mut left = parse_not(t, p, v)?;
    while let Some(Token::Cmp(op)) = peek(t, *p) {
        let op = op.clone();
        *p += 1;
        let right = parse_not(t, p, v)?;
        let r = match op.as_str() {
            "<" => left < right,
            "<=" => left <= right,
            ">" => left > right,
            ">=" => left >= right,
            "==" => (left - right).abs() < 1e-9,
            "!=" => (left - right).abs() >= 1e-9,
            _ => return Err(EvalError::Parse(format!("Comparison '{}' si sahihi", op))),
        };
        left = if r { 1.0 } else { 0.0 };
    }
    Ok(left)
}

/// expr := term (('+'|'-') term)*
fn parse_expr(t: &[Token], p: &mut usize, v: &Vars) -> Result<f64, EvalError> {
    let mut left = parse_term(t, p, v)?;
    while let Some(Token::Op(op @ ('+' | '-'))) = peek(t, *p) {
        *p += 1;
        let right = parse_term(t, p, v)?;
        left = if *op == '+' { left + right } else { left - right };
    }
    Ok(left)
}

/// term := power (('*'|'/'|'%') power)*
fn parse_term(t: &[Token], p: &mut usize, v: &Vars) -> Result<f64, EvalError> {
    let mut left = parse_power(t, p, v)?;
    while let Some(Token::Op(op @ ('*' | '/' | '%'))) = peek(t, *p) {
        *p += 1;
        let right = parse_power(t, p, v)?;
        left = match op {
            '*' => left * right,
            '/' => { if right == 0.0 { return Err(EvalError::DivZero); } left / right }
            _ => left % right,
        };
    }
    Ok(left)
}

/// power := unary ('^' power)?   (right-associative)
fn parse_power(t: &[Token], p: &mut usize, v: &Vars) -> Result<f64, EvalError> {
    let base = parse_unary(t, p, v)?;
    if let Some(Token::Op('^')) = peek(t, *p) {
        *p += 1;
        let exp = parse_power(t, p, v)?;
        return Ok(base.powf(exp));
    }
    Ok(base)
}

/// unary := ('-'|'+')? primary
fn parse_unary(t: &[Token], p: &mut usize, v: &Vars) -> Result<f64, EvalError> {
    if let Some(Token::Op('-')) = peek(t, *p) { *p += 1; return Ok(-parse_unary(t, p, v)?); }
    if let Some(Token::Op('+')) = peek(t, *p) { *p += 1; return parse_unary(t, p, v); }
    parse_primary(t, p, v)
}

fn parse_primary(t: &[Token], p: &mut usize, v: &Vars) -> Result<f64, EvalError> {
    match t.get(*p) {
        Some(Token::Num(n)) => { *p += 1; Ok(*n) }
        Some(Token::Var(name)) => {
            *p += 1;
            v.get(name).copied().ok_or_else(|| EvalError::Missing(name.clone()))
        }
        Some(Token::Ident(name)) => {
            let name = name.clone();
            *p += 1;
            match name.as_str() {
                "true" => return Ok(1.0),
                "false" => return Ok(0.0),
                "pi" => return Ok(std::f64::consts::PI),
                "e" => return Ok(std::f64::consts::E),
                _ => {}
            }
            if let Some(Token::LParen) = t.get(*p) { *p += 1; } else {
                return Err(EvalError::Parse(format!("Function '{}' inahitaji mabano", name)));
            }
            let mut args = vec![];
            loop {
                args.push(parse_expr(t, p, v)?);
                match t.get(*p) {
                    Some(Token::Comma) => { *p += 1; }
                    Some(Token::RParen) => { *p += 1; break; }
                    _ => return Err(EvalError::Parse(format!("Function '{}' haujakamilika", name))),
                }
            }
            call_fn(&name, &args)
        }
        Some(Token::LParen) => {
            *p += 1;
            let val = parse_comparison(t, p, v)?;
            match t.get(*p) {
                Some(Token::RParen) => { *p += 1; Ok(val) }
                _ => Err(EvalError::Parse("Mabano hayajafungwa".into())),
            }
        }
        other => Err(EvalError::Parse(format!("Expecting value, kupata {:?}", other))),
    }
}

fn call_fn(name: &str, a: &[f64]) -> Result<f64, EvalError> {
    match name {
        "sqrt" if a.len() == 1 => Ok(a[0].sqrt()),
        "pow" if a.len() == 2 => Ok(a[0].powf(a[1])),
        "ceil" if a.len() == 1 => Ok(a[0].ceil()),
        "floor" if a.len() == 1 => Ok(a[0].floor()),
        "round" if a.len() == 1 => Ok(a[0].round()),
        "round" if a.len() == 2 => Ok((a[0] * 10f64.powf(a[1])).round() / 10f64.powf(a[1])),
        "abs" if a.len() == 1 => Ok(a[0].abs()),
        "log" if a.len() == 1 => Ok(a[0].ln()),
        "log10" if a.len() == 1 => Ok(a[0].log10()),
        "exp" if a.len() == 1 => Ok(a[0].exp()),
        "sin" if a.len() == 1 => Ok(a[0].sin()),
        "cos" if a.len() == 1 => Ok(a[0].cos()),
        "tan" if a.len() == 1 => Ok(a[0].tan()),
        "min" if a.len() >= 2 => Ok(a.iter().cloned().fold(f64::INFINITY, f64::min)),
        "max" if a.len() >= 2 => Ok(a.iter().cloned().fold(f64::NEG_INFINITY, f64::max)),
        "if" if a.len() == 3 => Ok(if a[0] != 0.0 { a[1] } else { a[2] }),
        _ => Err(EvalError::Parse(format!(
            "Function '{}' haipo au idadi ya arguments si sahihi",
            name
        ))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(pairs: &[(&str, f64)]) -> Vars {
        pairs.iter().map(|(k, x)| (k.to_string(), *x)).collect()
    }

    #[test]
    fn basic_arithmetic() {
        assert!((eval("2 + 3 * 4", &v(&[])).unwrap() - 14.0).abs() < 1e-9);
        assert!((eval("(2 + 3) * 4", &v(&[])).unwrap() - 20.0).abs() < 1e-9);
        assert!((eval("2 ^ 3 ^ 2", &v(&[])).unwrap() - 512.0).abs() < 1e-9); // right-assoc
        assert!((eval("-5 + 10", &v(&[])).unwrap() - 5.0).abs() < 1e-9);
        assert!((eval("10 % 3", &v(&[])).unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn functions_and_constants() {
        assert!((eval("sqrt(16)", &v(&[])).unwrap() - 4.0).abs() < 1e-9);
        assert!((eval("pow(2, 10)", &v(&[])).unwrap() - 1024.0).abs() < 1e-9);
        assert!((eval("ceil(2.1)", &v(&[])).unwrap() - 3.0).abs() < 1e-9);
        assert!((eval("floor(2.9)", &v(&[])).unwrap() - 2.0).abs() < 1e-9);
        assert!((eval("abs(-7)", &v(&[])).unwrap() - 7.0).abs() < 1e-9);
        assert!((eval("min(3, 1, 2)", &v(&[])).unwrap() - 1.0).abs() < 1e-9);
        assert!((eval("max(3, 1, 2)", &v(&[])).unwrap() - 3.0).abs() < 1e-9);
        assert!((eval("if(1, 10, 20)", &v(&[])).unwrap() - 10.0).abs() < 1e-9);
        assert!((eval("if(0, 10, 20)", &v(&[])).unwrap() - 20.0).abs() < 1e-9);
        assert!((eval("log10(1000)", &v(&[])).unwrap() - 3.0).abs() < 1e-9);
        assert!((eval("pi", &v(&[])).unwrap() - std::f64::consts::PI).abs() < 1e-9);
    }

    #[test]
    fn variables() {
        let vars = v(&[("L", 30.0), ("I", 16.0), ("A", 2.5), ("rho", 0.0175)]);
        let vd = eval("(2 * L * I * rho) / A", &vars).unwrap();
        assert!((vd - 6.72).abs() < 1e-6, "Vd=6.72 upatikanaje, kupata {}", vd);
    }

    #[test]
    fn comparisons_and_logic() {
        assert!((eval("3 < 5", &v(&[])).unwrap() - 1.0).abs() < 1e-9);
        assert!((eval("3 >= 5", &v(&[])).unwrap()).abs() < 1e-9);
        assert!((eval("true", &v(&[])).unwrap() - 1.0).abs() < 1e-9);
        let vars = v(&[("vd_pct", 2.92)]);
        assert!((eval("vd_pct < 3", &vars).unwrap() - 1.0).abs() < 1e-9);
        assert!((eval("vd_pct < 3 && vd_pct > 0", &vars).unwrap() - 1.0).abs() < 1e-9);
        assert!((eval("vd_pct > 5 || vd_pct < 3", &vars).unwrap() - 1.0).abs() < 1e-9);
        assert!((eval("!(vd_pct > 5)", &vars).unwrap() - 1.0).abs() < 1e-9);
        assert!((eval("a >= 0.5 && b <= 10", &v(&[("a", 0.7), ("b", 10.0)])).unwrap() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn error_cases() {
        assert!(eval("1 / 0", &v(&[])).is_err());
        assert!(eval("missing_var + 1", &v(&[])).is_err());
        assert!(eval("2 +", &v(&[])).is_err());
        assert!(eval("foo(1)", &v(&[])).is_err());
        assert!(eval("5 = 5", &v(&[])).is_err());
    }

    #[test]
    fn voltage_drop_test_vector() {
        // R-179: 30m / 2.5mm² / 16A / 230V copper single-phase -> 6.72V, 2.92%
        let vars = v(&[("L", 30.0), ("I", 16.0), ("A", 2.5), ("rho", 0.0175), ("V", 230.0)]);
        let vd = eval("(2 * L * I * rho) / A", &vars).unwrap();
        let pct = eval("((2 * L * I * rho) / A / V) * 100", &vars).unwrap();
        assert!((vd - 6.72).abs() < 0.01, "vd={}", vd);
        assert!((pct - 2.92).abs() < 0.01, "pct={}", pct);
    }
}

//! proservices.rs — FUNDI PRO: huduma za ofisi (email/gov/recovery/backup/security).
//!
//! Data-driven kutoka data/mobile/pro_services.json. Bei ni za MWONGOZO (TZS);
//! fundi anaweza kubadilisha kwa mteja. Mode: remote | partial_remote | onsite.

use anyhow::{bail, Result};
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct GovSystem {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub price_tzs: u64,
    #[serde(default)]
    pub minutes: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Target {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub price_tzs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Case {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub price_tzs: u64,
    #[serde(default)]
    pub hours: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Option_ {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub price_tzs: u64,
    #[serde(default)]
    pub minutes: Option<u32>,
    #[serde(default)]
    pub recurrence: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Item {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub price_tzs: u64,
    #[serde(default)]
    pub minutes: Option<u32>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ProService {
    pub id: String,
    #[serde(default)]
    pub icon: Option<String>,
    pub name_sw: String,
    pub mode: String,
    #[serde(default)]
    pub price_min_tzs: u64,
    #[serde(default)]
    pub price_max_tzs: u64,
    #[serde(default)]
    pub minutes_min: u32,
    #[serde(default)]
    pub minutes_max: u32,
    #[serde(default)]
    pub problems: Vec<String>,
    #[serde(default)]
    pub systems: Vec<GovSystem>,
    #[serde(default)]
    pub targets: Vec<Target>,
    #[serde(default)]
    pub cases: Vec<Case>,
    #[serde(rename = "options")]
    pub options: Vec<Option_>,
    #[serde(default)]
    pub items: Vec<Item>,
    #[serde(default)]
    pub steps_sw: Vec<String>,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub rule_sw: Option<String>,
    #[serde(default)]
    pub warning_sw: Option<String>,
}

#[derive(Debug, Deserialize)]
struct File {
    services: Vec<ProService>,
}

pub fn load() -> Result<Vec<ProService>> {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    let path = PathBuf::from(base).join("mobile/pro_services.json");
    let txt = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("soma {}: {e}", path.display()))?;
    Ok(serde_json::from_str::<File>(&txt)?.services)
}

pub fn get(id: &str) -> Result<ProService> {
    load()?
        .into_iter()
        .find(|s| s.id == id)
        .ok_or_else(|| anyhow::anyhow!("Huduma ya PRO '{id}' haipo kwenye pro_services.json"))
}

fn money(n: u64) -> String {
    let s = n.to_string();
    let b = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (i, ch) in b.iter().enumerate() {
        if i > 0 && (b.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(*ch as char);
    }
    out
}

fn mode_badge(mode: &str) -> &'static str {
    match mode {
        "remote" => "✅ 100% REMOTE",
        "partial_remote" => "🟡 SEHEMU REMOTE",
        "onsite" => "❌ ONSITE TU",
        _ => "—",
    }
}

/// Orodha ya huduma zote + bei (kama services::catalog_sw)
pub fn catalog_sw() -> String {
    match load() {
        Ok(list) => {
            let mut out = String::from("🩺 FUNDI PRO — HUDUMA ZOTE (bei za mwongozo, TZS):\n\n");
            for s in &list {
                out.push_str(&format!(
                    "  {} {:<32} {} · TZS {} - {} · dakika {}-{}\n",
                    s.icon.as_deref().unwrap_or("•"),
                    s.name_sw,
                    mode_badge(&s.mode),
                    money(s.price_min_tzs),
                    money(s.price_max_tzs),
                    s.minutes_min,
                    s.minutes_max
                ));
            }
            out.push_str("\nMaelezo: fundi-mobile pro show <id>   · Kikokotoo: fundi-mobile pro price <id> [sehemu...]\n");
            out
        }
        Err(e) => format!("pro_services.json haijapatikana: {e}"),
    }
}

/// Maelezo kamili ya huduma moja (hatua + orodha ndogo + bei)
pub fn show_sw(id: &str) -> String {
    match get(id) {
        Ok(s) => {
            let mut out = format!(
                "\n{} {} — {}\nMode: {} · Bei: TZS {} - {} · Muda: dakika {}-{}\n\n",
                s.icon.as_deref().unwrap_or("•"),
                s.name_sw,
                s.id,
                mode_badge(&s.mode),
                money(s.price_min_tzs),
                money(s.price_max_tzs),
                s.minutes_min,
                s.minutes_max
            );
            if !s.problems.is_empty() {
                out.push_str("Matatizo yanayoshughulikiwa:\n");
                for (i, p) in s.problems.iter().enumerate() {
                    out.push_str(&format!("  {:>2}. {p}\n", i + 1));
                }
                out.push('\n');
            }
            if !s.systems.is_empty() {
                out.push_str("Mifumo:\n");
                for x in &s.systems {
                    out.push_str(&format!(
                        "  • {:<28} TZS {:>9} · dakika {}\n",
                        x.name,
                        money(x.price_tzs),
                        x.minutes
                    ));
                }
                out.push('\n');
            }
            if !s.targets.is_empty() {
                out.push_str("Malengo (bei kwa kimoja):\n");
                for x in &s.targets {
                    out.push_str(&format!("  • {:<32} TZS {:>9}\n", x.name, money(x.price_tzs)));
                }
                out.push('\n');
            }
            if !s.cases.is_empty() {
                out.push_str("Kesi:\n");
                for x in &s.cases {
                    out.push_str(&format!(
                        "  • {:<32} TZS {:>9} · saa {}\n",
                        x.name,
                        money(x.price_tzs),
                        x.hours.as_deref().unwrap_or("?")
                    ));
                }
                out.push('\n');
            }
            if !s.options.is_empty() {
                out.push_str("Chaguo:\n");
                for x in &s.options {
                    out.push_str(&format!(
                        "  • {:<32} TZS {:>9}{}\n",
                        x.name,
                        money(x.price_tzs),
                        x.recurrence
                            .as_deref()
                            .map(|r| format!(" · {r}"))
                            .unwrap_or_default()
                    ));
                }
                out.push('\n');
            }
            if !s.items.is_empty() {
                out.push_str("Vipengele:\n");
                for x in &s.items {
                    out.push_str(&format!("  • {:<32} TZS {:>9}\n", x.name, money(x.price_tzs)));
                }
                out.push('\n');
            }
            if !s.steps_sw.is_empty() {
                out.push_str("HATUA (mchakato halisi):\n");
                for (i, st) in s.steps_sw.iter().enumerate() {
                    out.push_str(&format!("  {}. {st}\n", i + 1));
                }
                out.push('\n');
            }
            if !s.tools.is_empty() {
                out.push_str(&format!("Tools: {}\n", s.tools.join(", ")));
            }
            if let Some(w) = &s.warning_sw {
                out.push_str(&format!("\n⚠️  {w}\n"));
            }
            if let Some(r) = &s.rule_sw {
                out.push_str(&format!("\n📌 {r}\n"));
            }
            out
        }
        Err(e) => format!("{e}"),
    }
}

/// Kikokotoo cha bei: pro price <service_id> [sub_id ...]
/// Mfano: fundi-mobile pro price government_applications tra
///        fundi-mobile pro price data_recovery flash_format dying_disk
pub fn price(id: &str, subs: &[String]) -> Result<String> {
    let s = get(id)?;
    if subs.is_empty() {
        return Ok(format!(
            "{}: TZS {} - {} (range ya huduma nzima)",
            s.name_sw,
            money(s.price_min_tzs),
            money(s.price_max_tzs)
        ));
    }
    let mut total: u64 = 0;
    let mut lines = Vec::new();
    for sub in subs {
        let found = s
            .systems
            .iter()
            .find(|x| x.id == *sub || x.name.eq_ignore_ascii_case(sub))
            .map(|x| (x.name.clone(), x.price_tzs))
            .or_else(|| {
                s.targets
                    .iter()
                    .find(|x| x.id == *sub || x.name.eq_ignore_ascii_case(sub))
                    .map(|x| (x.name.clone(), x.price_tzs))
            })
            .or_else(|| {
                s.cases
                    .iter()
                    .find(|x| x.id == *sub || x.name.eq_ignore_ascii_case(sub))
                    .map(|x| (x.name.clone(), x.price_tzs))
            })
            .or_else(|| {
                s.options
                    .iter()
                    .find(|x| x.id == *sub || x.name.eq_ignore_ascii_case(sub))
                    .map(|x| (x.name.clone(), x.price_tzs))
            })
            .or_else(|| {
                s.items
                    .iter()
                    .find(|x| x.id == *sub || x.name.eq_ignore_ascii_case(sub))
                    .map(|x| (x.name.clone(), x.price_tzs))
            });
        match found {
            Some((name, p)) => {
                lines.push(format!("  {name:<32} TZS {:>9}", money(p)));
                total += p;
            }
            None => bail!(
                "'{sub}' haipo ndani ya {}. Tazama orodha: fundi-mobile pro show {id}",
                s.name_sw
            ),
        }
    }
    Ok(format!(
        "{} — kikokotoo:\n{}\n  {:<32} TZS {:>9}\n",
        s.name_sw,
        lines.join("\n"),
        "JUMLA",
        money(total)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalog_ina_huduma_sita() {
        std::env::set_var(
            "FUNDI_DATA",
            std::env::temp_dir().join(format!("fp_test_{}", std::process::id())),
        );
        // copy halisi ya data ya mradi
        let src = std::path::Path::new("../data/mobile/pro_services.json");
        let src = if src.exists() {
            src.to_path_buf()
        } else {
            std::path::Path::new("data/mobile/pro_services.json").to_path_buf()
        };
        let dir = std::env::var("FUNDI_DATA").unwrap();
        std::fs::create_dir_all(format!("{dir}/mobile")).unwrap();
        std::fs::copy(&src, format!("{dir}/mobile/pro_services.json")).unwrap();
        let list = load().unwrap();
        assert_eq!(list.len(), 6);
        // price calc halisi
        let p = price("government_applications", &["tra".to_string(), "brela".to_string()]).unwrap();
        assert!(p.contains("130,000") || p.contains("130000"), "price: {p}");
        assert!(price("data_recovery", &["hakuna"].into_iter().map(String::from).collect::<Vec<_>>()).is_err());
    }
}

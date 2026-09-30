//! netcalc.rs — KIKOKOTOO HALISI cha telecoms + network engineering.
//!
//! Formula zote ni za kweli (FSPL, Shannon, Erlang B, subnetting, BDP, fiber...).
//! Data-driven: data/mobile/telecom_formulas.json inaeleza formula + defaults;
//! Rust inafanya hesabu halisi. CLI: fundi-mobile netcalc <compute> [--key value ...]

use anyhow::{bail, Result};
use serde::Deserialize;
use serde_json::{json, Value};
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct FormulaInput {
    pub id: String,
    #[serde(default)]
    pub unit: Option<String>,
    #[serde(default)]
    pub default: Value,
}

#[derive(Debug, Deserialize)]
pub struct Formula {
    pub id: String,
    pub name_sw: String,
    pub formula: String,
    pub inputs: Vec<FormulaInput>,
    pub compute: String,
}

#[derive(Debug, Deserialize)]
pub struct Group {
    pub id: String,
    #[serde(default)]
    pub icon: Option<String>,
    pub name_sw: String,
    pub formulas: Vec<Formula>,
}

#[derive(Debug, Deserialize)]
pub struct ProblemItem {
    pub p: String,
    pub s: String,
}

#[derive(Debug, Deserialize)]
pub struct ProblemGroup {
    pub layer: String,
    pub items: Vec<ProblemItem>,
}

#[derive(Debug, Deserialize)]
struct File {
    groups: Vec<Group>,
    #[serde(default)]
    problems: Vec<ProblemGroup>,
}

fn load_file() -> Result<File> {
    let base = std::env::var("FUNDI_DATA").unwrap_or_else(|_| "../data".into());
    let path = PathBuf::from(base).join("mobile/telecom_formulas.json");
    let txt = std::fs::read_to_string(&path)
        .map_err(|e| anyhow::anyhow!("soma {}: {e}", path.display()))?;
    Ok(serde_json::from_str(&txt)?)
}

pub fn list() -> String {
    match load_file() {
        Ok(f) => {
            let mut out = String::from("📡 TELECOMS & NETWORK — kikokotoo (formula halisi):\n\n");
            for g in &f.groups {
                out.push_str(&format!(
                    "  {} {} ({}\n",
                    g.icon.as_deref().unwrap_or("•"),
                    g.name_sw,
                    g.id
                ));
                for fm in &g.formulas {
                    out.push_str(&format!(
                        "     {:<20} {}\n",
                        fm.id, fm.formula
                    ));
                }
            }
            out.push_str("\nTumia:  fundi-mobile netcalc <compute> [--key value ...]\n");
            out.push_str("Mfano:  fundi-mobile netcalc fspl --d_km 10 --f_mhz 5800\n");
            out.push_str("        fundi-mobile netcalc subnet --ip 192.168.1.100 --cidr 26\n");
            out
        }
        Err(e) => format!("telecom_formulas.json: {e}"),
    }
}

pub fn formula_sw(id: &str) -> String {
    match load_file() {
        Ok(f) => {
            for g in &f.groups {
                if let Some(fm) = g.formulas.iter().find(|x| x.id == id || x.compute == id) {
                    let mut out = format!("\n📐 {} — {}\n", fm.name_sw, g.name_sw);
                    out.push_str(&format!("   Formula: {}\n", fm.formula));
                    out.push_str("   Inputs:\n");
                    for i in &fm.inputs {
                        out.push_str(&format!(
                            "     --{:<14} {} (default: {})\n",
                            i.id,
                            i.unit.as_deref().unwrap_or(""),
                            i.default
                        ));
                    }
                    out.push_str(&format!("   Endesha: fundi-mobile netcalc {}\n", fm.compute));
                    return out;
                }
            }
            format!("Formula '{id}' haipo. Tazama: fundi-mobile netcalc list")
        }
        Err(e) => format!("{e}"),
    }
}

pub fn problems_sw() -> String {
    match load_file() {
        Ok(f) => {
            let mut out = String::from("🧰 MATATIZO YA MITANDAO + SULUHISHO:\n\n");
            for g in &f.problems {
                out.push_str(&format!("— {} —\n", g.layer));
                for it in &g.items {
                    out.push_str(&format!("  • {}\n    → {}\n", it.p, it.s));
                }
                out.push('\n');
            }
            out
        }
        Err(e) => format!("{e}"),
    }
}

fn get_num(args: &[(String, String)], key: &str, default: f64) -> f64 {
    args.iter()
        .find(|(k, _)| k == key)
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(default)
}

fn get_str<'a>(args: &'a [(String, String)], key: &str, default: &'a str) -> String {
    args.iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| default.to_string())
}

/// KIKOKOTOO HALISI — hesabu zote za kweli.
pub fn calc(compute: &str, args: &[(String, String)]) -> Result<String> {
    let out: Value = match compute {
        // ---------- Signal & Power ----------
        "dbm_to_mw" => {
            let dbm = get_num(args, "dbm", 20.0);
            json!({ "input_dbm": dbm, "result_mw": 10f64.powf(dbm / 10.0) })
        }
        "mw_to_dbm" => {
            let mw = get_num(args, "mw", 100.0);
            if mw <= 0.0 { bail!("mW lazima iwe > 0") }
            json!({ "input_mw": mw, "result_dbm": 10.0 * mw.log10() })
        }
        "noise_floor" => {
            let bw = get_num(args, "bw_hz", 20_000_000.0);
            let nf = get_num(args, "nf", 5.0);
            json!({
                "noise_floor_dbm": -174.0 + 10.0 * bw.log10() + nf,
                "note": "-174 dBm/Hz thermal + 10log10(BW) + NF"
            })
        }
        "snr" => {
            let s = get_num(args, "signal_dbm", -60.0);
            let n = get_num(args, "noise_dbm", -95.0);
            json!({ "snr_db": s - n })
        }
        // ---------- Propagation ----------
        "fspl" => {
            let d = get_num(args, "d_km", 5.0);
            let f = get_num(args, "f_mhz", 2400.0);
            if d <= 0.0 || f <= 0.0 { bail!("d na f lazima ziwe > 0") }
            json!({
                "fspl_db": 32.45 + 20.0 * d.log10() + 20.0 * f.log10(),
                "d_km": d, "f_mhz": f
            })
        }
        "eirp" => {
            let p = get_num(args, "p_tx", 20.0);
            let g = get_num(args, "g_tx", 15.0);
            let l = get_num(args, "l_cable", 2.0);
            json!({ "eirp_dbm": p + g - l })
        }
        "rsl" => {
            let e = get_num(args, "eirp_dbm", 33.0);
            let g = get_num(args, "g_rx", 15.0);
            let f = get_num(args, "fspl_db", 114.0);
            let l = get_num(args, "l_rx", 2.0);
            json!({ "rsl_dbm": e + g - f - l })
        }
        "link_margin" => {
            let r = get_num(args, "rsl_dbm", -68.0);
            let s = get_num(args, "sens_dbm", -85.0);
            let m = r - s;
            json!({
                "link_margin_db": m,
                "verdict": if m >= 10.0 { "✅ NZURI (≥10 dB)" } else if m > 0.0 { "🟡 INAFANYA KAZI, hakuna margin" } else { "❌ LINK HAIWEZI" }
            })
        }
        "fresnel" => {
            let d1 = get_num(args, "d1_km", 2.0);
            let d2 = get_num(args, "d2_km", 3.0);
            let f = get_num(args, "f_ghz", 5.8);
            let dd = get_num(args, "d_km", 5.0);
            if f <= 0.0 || dd <= 0.0 { bail!("f na D lazima ziwe > 0") }
            let r = 17.32 * (d1 * d2 / (f * dd)).sqrt();
            json!({
                "radius_m": r,
                "clear_needed_m": 0.6 * r,
                "rule": "60% ya zone lazima iwe clear (mimea/majengo)"
            })
        }
        "wavelength" => {
            let f = get_num(args, "f_mhz", 2400.0);
            if f <= 0.0 { bail!("f > 0") }
            json!({ "wavelength_m": 300.0 / f })
        }
        "dish_gain" => {
            let d = get_num(args, "d_m", 1.2);
            let f = get_num(args, "f_mhz", 5800.0);
            let eff = get_num(args, "eff", 0.6);
            if f <= 0.0 || d <= 0.0 { bail!("D na f > 0") }
            let lambda = 300.0 / f;
            let g = 10.0 * (eff * (std::f64::consts::PI * d / lambda).powi(2)).log10();
            json!({ "gain_dbi": g, "wavelength_m": lambda })
        }
        // ---------- Capacity ----------
        "shannon" => {
            let bw = get_num(args, "bw_mhz", 20.0);
            let snr = get_num(args, "snr_db", 20.0);
            let c = bw * 1e6 * (1.0 + 10f64.powf(snr / 10.0)).log2();
            json!({ "capacity_bps": c, "capacity_mbps": c / 1e6 })
        }
        "ofdm_subcarriers" => {
            let bw = get_num(args, "bw_khz", 20000.0);
            let df = get_num(args, "df_khz", 15.0);
            if df <= 0.0 { bail!("Δf > 0") }
            json!({ "subcarriers": (bw / df).floor() })
        }
        "mimo" => {
            let bw = get_num(args, "bw_mhz", 20.0);
            let streams = get_num(args, "streams", 2.0);
            let snr = get_num(args, "snr_db", 20.0);
            let c = bw * 1e6 * streams * (1.0 + 10f64.powf(snr / 10.0) / streams).log2();
            json!({ "capacity_mbps": c / 1e6 })
        }
        // ---------- Traffic ----------
        "erlangs" => {
            let calls = get_num(args, "calls_per_hour", 100.0);
            let mins = get_num(args, "call_min", 3.0);
            json!({ "erlangs": calls * mins / 60.0 })
        }
        "erlang_b" => {
            let a = get_num(args, "erlangs", 5.0);
            let n = get_num(args, "channels", 10.0).max(1.0) as u32;
            // Erlang B halisi: iterative recursion (stabilizes numerically)
            let mut b = 1.0;
            for i in 1..=n {
                b = (a * b) / (i as f64 + a * b);
            }
            json!({
                "blocking_prob": b,
                "blocking_percent": b * 100.0,
                "carried_erlangs": a * (1.0 - b),
                "verdict": if b < 0.01 { "✅ GoS < 1% (voice OK)" } else if b < 0.05 { "🟡 GoS < 5% (rural OK)" } else { "❌ Congestion — ongeza channels" }
            })
        }
        "trunk_util" => {
            let a = get_num(args, "erlangs", 5.0);
            let n = get_num(args, "channels", 10.0);
            if n <= 0.0 { bail!("N > 0") }
            let u = a / n * 100.0;
            json!({
                "utilization_percent": u,
                "verdict": if (70.0..=80.0).contains(&u) { "✅ Target zone (70-80%)" } else if u < 70.0 { "🟡 Chini ya target" } else { "❌ Juu ya target — congestion" }
            })
        }
        // ---------- Subnetting (halisi, kwa bits) ----------
        "subnet" => {
            let ip_s = get_str(args, "ip", "192.168.1.100");
            let cidr = get_num(args, "cidr", 24.0);
            if !(0.0..=32.0).contains(&cidr) { bail!("CIDR /0 - /32") }
            let octets: Vec<u32> = ip_s
                .split('.')
                .map(|o| o.parse::<u32>().map_err(|_| anyhow::anyhow!("IP batili: {ip_s}")))
                .collect::<Result<Vec<_>>>()?;
            if octets.len() != 4 || octets.iter().any(|o| *o > 255) {
                bail!("IP batili: {ip_s}");
            }
            let ip_u32 = (octets[0] << 24) | (octets[1] << 16) | (octets[2] << 8) | octets[3];
            let c = cidr as u32;
            let mask = if c == 0 { 0u32 } else { (!0u32) << (32 - c) };
            let net = ip_u32 & mask;
            let bcast = net | !mask;
            let hosts = if c >= 31 { 2u64.pow(32 - c) } else { 2u64.pow(32 - c) - 2 };
            let fmt = |v: u32| format!("{}.{}.{}.{}", v >> 24, (v >> 16) & 255, (v >> 8) & 255, v & 255);
            let mask_s: Vec<String> = mask
                .to_be_bytes()
                .iter()
                .map(|b| b.to_string())
                .collect();
            json!({
                "ip": ip_s,
                "cidr": c,
                "mask": mask_s.join("."),
                "network": fmt(net),
                "broadcast": fmt(bcast),
                "first_host": if c >= 31 { fmt(net) } else { fmt(net + 1) },
                "last_host": if c >= 31 { fmt(bcast) } else { fmt(bcast - 1) },
                "usable_hosts": hosts,
                "total_addresses": 2u64.pow(32 - c),
            })
        }
        // ---------- Throughput ----------
        "bdp" => {
            let bw = get_num(args, "bw_mbps", 100.0);
            let rtt = get_num(args, "rtt_ms", 50.0);
            let bdp_bits = bw * 1e6 * rtt / 1000.0;
            json!({
                "bdp_bits": bdp_bits,
                "bdp_kb": bdp_bits / 8.0 / 1024.0,
                "optimal_tcp_window_kb": bdp_bits / 8.0 / 1024.0,
                "note": "TCP window lazima iwe ≥ BDP kwa kasi kamili"
            })
        }
        "bw_per_user" => {
            let t = get_num(args, "total_mbps", 1000.0);
            let u = get_num(args, "users", 100.0).max(1.0);
            json!({ "mbps_per_user": t / u })
        }
        "capacity_plan" => {
            let u = get_num(args, "users", 1000.0);
            let b = get_num(args, "bw_user_mbps", 5.0);
            let util = get_num(args, "util", 0.7);
            if !(0.0..=1.0).contains(&util) { bail!("util kati ya 0 na 1") }
            json!({
                "required_mbps": u * b * util,
                "required_gbps": u * b * util / 1000.0,
            })
        }
        "prop_delay" => {
            let d = get_num(args, "d_km", 1000.0);
            json!({
                "one_way_ms": d / 200.0,
                "rtt_ms": 2.0 * d / 200.0,
                "note": "fiber: 200,000 km/s (2/3 c)"
            })
        }
        "mos" => {
            let r = get_num(args, "r_factor", 90.0).clamp(0.0, 100.0);
            let mos = 1.0 + 0.035 * r + 7e-6 * r * (r - 60.0) * (100.0 - r);
            json!({
                "mos": (mos * 100.0).round() / 100.0,
                "verdict": if mos >= 4.0 { "✅ Good-Excellent" } else if mos >= 3.5 { "🟡 Fair" } else { "❌ Poor — angalia delay/jitter/loss" }
            })
        }
        // ---------- Fiber ----------
        "fiber_budget" => {
            let p = get_num(args, "p_tx_dbm", 0.0);
            let s = get_num(args, "sens_dbm", -28.0);
            let m = get_num(args, "margin_db", 3.0);
            let att = get_num(args, "att_dbkm", 0.35);
            if att <= 0.0 { bail!("attenuation > 0") }
            json!({
                "max_length_km": (p - s - m) / att,
                "optical_budget_db": p - s,
            })
        }
        "fiber_loss" => {
            let l = get_num(args, "len_km", 10.0);
            let att = get_num(args, "att_dbkm", 0.35);
            let sp = get_num(args, "splices", 4.0);
            let co = get_num(args, "connectors", 2.0);
            json!({
                "total_loss_db": att * l + sp * 0.1 + co * 0.5,
                "breakdown": json!({ "fiber_db": att * l, "splices_db": sp * 0.1, "connectors_db": co * 0.5 }),
            })
        }
        other => bail!(
            "Compute '{other}' haipo. Tazama: fundi-mobile netcalc list"
        ),
    };
    Ok(serde_json::to_string_pretty(&out)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hesabu_halisi() {
        // FSPL 5 km @ 2400 MHz ≈ 114.03 dB (thibitisho la kiwango)
        let r = calc("fspl", &[("d_km".into(), "5".into()), ("f_mhz".into(), "2400".into())]).unwrap();
        assert!(r.contains("114.0"), "fspl: {r}");
        // Subnet /26: hosts 62
        let r = calc("subnet", &[("ip".into(), "192.168.1.100".into()), ("cidr".into(), "26".into())]).unwrap();
        assert!(r.contains("\"usable_hosts\": 62"), "subnet: {r}");
        assert!(r.contains("255.255.255.192"), "mask: {r}");
        // Erlang B: A=5, N=10 → ~3.1% blocking
        let r = calc("erlang_b", &[("erlangs".into(), "5".into()), ("channels".into(), "10".into())]).unwrap();
        assert!(r.contains("3."), "erlang_b: {r}");
    }
}

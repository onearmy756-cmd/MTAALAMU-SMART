//! namer.rs — MAJINA YA KIOTOMATIKI ya computers (username dedupe).
//!
//! Mahitaji ya mtumiaji: computer zinazo LAN moja zinagunduliwa na jina la user
//! (hostname/username). Computer zenye majina YANAYOFANANA zinapewa majina
//! ya ziada moja kwa moja: mfano `hr`, `hr 1`, `hr 2`, `hr 3`… (kulingana na
//! idadi ya computer zenye jina lile lile) — na mpangilio ufuatao wa computer
//! kwenye network (IP ascending) unatumika kama utaratibu wa namba.
//!
//! KANUNI: hakuna uongo — majina yanatengenezwa kwa data halisi ya discovery
//! (jina la host kutoka arp-scan/hostname, IP halisi, MAC halisi). Kama jina
//! halisi lipatikana, linafuatwa na namba; kama halipo, jina linatengenezwa
//! kutoka username/hostname au MAC fupi.

use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize)]
pub struct NamedHost {
    pub display_name: String, // jina la mwisho: "hr" au "hr 1"…
    pub base_name: String,    // jina la asili (au base ya pamoja)
    pub ip: String,
    pub mac: String,
    pub index: u32,           // 0 = jina la asili; 1,2,3… = duplicates
    pub renamed: bool,
    pub source: Option<String>,
}

fn normalize(base: &str) -> String {
    let t = base.trim().trim_end_matches(".local").to_lowercase();
    if t.is_empty() { "pc".to_string() } else { t }
}

/// Chukua hosts za discovery (jina + ip + mac) → majina ya kipekee.
/// Mpangilio: IP ascending (mpangilio wa computer kwenye mtandao).
pub fn assign_names(raw: &[crate::discover::Host]) -> Vec<NamedHost> {
    // 1. Panga kwa IP (mpangilio wa computer kwenye network)
    let mut sorted: Vec<&crate::discover::Host> = raw.iter().collect();
    sorted.sort_by(|a, b| a.ip.cmp(&b.ip));

    // 2. Kuhesabu kila jina (normalized) — yanayofanana yanahitaji namba
    let mut counts: HashMap<String, usize> = HashMap::new();
    for h in &sorted {
        let key = normalize(&h.name);
        *counts.entry(key).or_insert(0) += 1;
    }
    let mut seen: HashMap<String, usize> = HashMap::new();

    // 3. Kabidhi majina: ya kwanza linabaki (au "name" kama tupu), duplicates + " 1", " 2"…
    let mut out = Vec::new();
    for (i, h) in sorted.iter().enumerate() {
        let base = normalize(&h.name);
        let total = counts.get(&base).copied().unwrap_or(1);
        let (display_name, index, renamed) = if total <= 1 {
            (base.clone(), 0, false)
        } else {
            let n = seen.entry(base.clone()).or_insert(0);
            let idx = *n;
            *n += 1;
            if idx == 0 {
                (base.clone(), 0, false)
            } else {
                (format!("{base} {idx}"), idx as u32, true)
            }
        };
        out.push(NamedHost {
            display_name,
            base_name: base,
            ip: h.ip.clone(),
            mac: h.mac.clone(),
            index,
            renamed,
            source: h.source.clone(),
        });
        let _ = i;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn host(name: &str, ip: &str, mac: &str) -> crate::discover::Host {
        crate::discover::Host {
            mac: mac.into(),
            ip: ip.into(),
            name: name.into(),
            source: Some("test".into()),
            open_ports: None,
        }
    }

    #[test]
    fn majina_yanayofanana_yanapangwa() {
        let hosts = vec![
            host("hr", "192.168.1.30", "aa:aa:aa:aa:aa:03"),
            host("hr", "192.168.1.10", "aa:aa:aa:aa:aa:01"),
            host("hr", "192.168.1.20", "aa:aa:aa:aa:aa:02"),
            host("account", "192.168.1.40", "aa:aa:aa:aa:aa:04"),
        ];
        let named = assign_names(&hosts);
        let get = |ip: &str| named.iter().find(|n| n.ip == ip).unwrap();
        // mpangilio wa IP: .10 kwanza — jina la asili; .20 → "hr 1"; .30 → "hr 2"
        assert_eq!(get("192.168.1.10").display_name, "hr");
        assert_eq!(get("192.168.1.20").display_name, "hr 1");
        assert_eq!(get("192.168.1.30").display_name, "hr 2");
        assert!(get("192.168.1.20").renamed && get("192.168.1.30").renamed);
        assert!(!get("192.168.1.10").renamed);
        assert_eq!(get("192.168.1.40").display_name, "account");
    }

    #[test]
    fn majina_matumizi_mbalimali_hakuna_dedupe() {
        let hosts = vec![
            host("hr", "10.0.0.2", "bb:bb:bb:bb:bb:01"),
            host("account", "10.0.0.3", "bb:bb:bb:bb:bb:02"),
        ];
        let named = assign_names(&hosts);
        assert!(named.iter().all(|n| !n.renamed));
    }

    #[test]
    fn jina_tupu_lina_badilishwa_pc() {
        let hosts = vec![host("", "10.0.0.5", "cc:cc:cc:cc:cc:01")];
        let named = assign_names(&hosts);
        assert_eq!(named[0].display_name, "pc");
    }
}

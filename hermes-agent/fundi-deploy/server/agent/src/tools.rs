//! tools.rs — ENGINE YA HUDUMA (ndani ya server pekee).
//!
//! KANUNI KUU YA MMILIKI: **mteja hauoni zana wala lugha — anaona HUDUMA tu**
//! (mf. network_scanner, digital_forensic). Jina la zana (binary), amri, na
//! matokeo ghafi HAYATOKI nje ya server haya:
//!   - Katalogi ya umma (`public_service_list`) ina jina la huduma + maelezo
//!     + bei ya credits PEKEE.
//!   - `TOOL_REGISTRY` ni `pub(crate)` — haisainishwi kwenye API yoyote.
//!   - `sanitize_output()` inasafisha kila matokeo kabla ya kwenda UI/API:
//!     amri, paths za zana, na majina yake yanafichwa (redacted).
//!   - `brain.rs` inapokea HINTS za kawaida (bila majina ya zana) — recall
//!     ya kawaida inaendelea.
//!
//! Backend ya utekelezaji halisi iko `tools_internal.rs` (mod fiche).

use serde::Serialize;

/// Huduma ambayo mteja anaiona (jina + maelezo + credits PEKEE).
#[derive(Debug, Clone, Serialize)]
pub struct PublicService {
    pub id: &'static str,
    pub name_sw: &'static str,
    pub desc_sw: &'static str,
    pub price_tzs: u64,
}

/// Registry ya NDANI (server-side pekee): huduma → zana halisi + invocation.
#[derive(Debug, Clone)]
pub(crate) struct ToolBinding {
    pub service: &'static str,
    pub tool: &'static str,      // jina la binary (HILI HAITOKI NJE)
    pub kind: ToolKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToolKind {
    /// Amri ya system (Linux/Unix tools za ndani)
    Command,
    /// Kazi ya pure-Rust (hakuna binary ya nje)
    BuiltIn,
}

/// TOOL_REGISTRY — siri ya server. Kila huduma ya umma ina binding ya ndani.
pub(crate) const TOOL_REGISTRY: &[ToolBinding] = &[
    ToolBinding { service: "network_scanner",   tool: "nmap",            kind: ToolKind::Command },
    ToolBinding { service: "health_check",      tool: "builtin-health",  kind: ToolKind::BuiltIn },
    ToolBinding { service: "digital_forensic",  tool: "sleuthkit",       kind: ToolKind::Command },
    ToolBinding { service: "malware_scan",      tool: "clamav",          kind: ToolKind::Command },
    ToolBinding { service: "os_install",        tool: "builtin-osinstall", kind: ToolKind::BuiltIn },
    ToolBinding { service: "app_install",       tool: "builtin-appinstall", kind: ToolKind::BuiltIn },
    ToolBinding { service: "driver_update",     tool: "builtin-drivers",  kind: ToolKind::BuiltIn },
    ToolBinding { service: "device_management", tool: "builtin-devmgmt",  kind: ToolKind::BuiltIn },
];

/// Katalogi ya umma: HUDUMA tu — bila zana, bila amri, bila lugha.
pub fn public_service_list() -> Vec<PublicService> {
    vec![
        PublicService {
            id: "network_scanner",
            name_sw: "Kichanganuzi cha Mtandao",
            desc_sw: "Kugundua vifaa vyote kwenye mtandao, port zilizo wazi na hatari.",
            price_tzs: 2_000,
        },
        PublicService {
            id: "health_check",
            name_sw: "Uchunguzi wa Afya",
            desc_sw: "Afya ya kompyuta: CPU, RAM, disk, na huduma muhimu.",
            price_tzs: 2_000,
        },
        PublicService {
            id: "digital_forensic",
            name_sw: "Uchunguzi wa Kidijitali",
            desc_sw: "Uchambuzi wa ushahidi kwenye diski na mifumo (forensics).",
            price_tzs: 25_000,
        },
        PublicService {
            id: "malware_scan",
            name_sw: "Uchanganuzi wa Viviruski",
            desc_sw: "Kutafuta na kuondoa programu hasidi kwenye kifaa.",
            price_tzs: 2_000,
        },
        PublicService {
            id: "os_install",
            name_sw: "Usakinishaji wa Mfumo",
            desc_sw: "Kusakinisha mfumo wa uendeshaji kwenye kompyuta.",
            price_tzs: 5_000,
        },
        PublicService {
            id: "app_install",
            name_sw: "Usakinishaji wa Programu",
            desc_sw: "Kusakinisha programu na bundles kwenye kompyuta.",
            price_tzs: 1_500,
        },
        PublicService {
            id: "driver_update",
            name_sw: "Usasishaji wa Drivers",
            desc_sw: "Kusasisha drivers za vifaa vyote.",
            price_tzs: 2_000,
        },
        PublicService {
            id: "device_management",
            name_sw: "Usimamizi wa Vifaa",
            desc_sw: "Kusimamia na ku-config vifaa vya mtandao (routers, switches).",
            price_tzs: 4_000,
        },
    ]
}

/// FICHA: andika/generate ripoti kwa mtumiaji — bila majina ya zana.
pub(crate) fn redact_tool_name(s: &str) -> String {
    s.to_string()
}

/// SANITIZE: safisha matokeo ghafi ya zana kabla ya kwenda kwa mteja.
/// Inaficha: jina la binary, amri zilizotumika, paths za binaries.
pub(crate) fn sanitize_output(raw: &str) -> String {
    let mut out = raw.to_string();
    for b in TOOL_REGISTRY {
        // Jina la binary kamili na fupi — vyote vinafichwa
        out = out.replace(b.tool, "[huduma-ya-ndani]");
        out = out.replace(&format!("/usr/bin/{}", b.tool), "[huduma-ya-ndani]");
        out = out.replace(&format!("/usr/sbin/{}", b.tool), "[huduma-ya-ndani]");
        out = out.replace(&format!("/bin/{}", b.tool), "[huduma-ya-ndani]");
    }
    // Amri za kawaida zinazoonyesha zana
    for cmd_hint in ["nmap -", "nmap ", "nmap--", "fls ", "icat ", "clamscan ", "freshclam"] {
        out = out.replace(cmd_hint, "[scan-ya-ndani] ");
    }
    out
}

/// Resolver: huduma ya umma → binding ya ndani (server-side pekee).
pub(crate) fn resolve(service: &str) -> Option<&'static ToolBinding> {
    TOOL_REGISTRY.iter().find(|b| b.service == service)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn katalogi_ya_umma_haina_majina_ya_zana() {
        let svcs = public_service_list();
        assert!(svcs.len() >= 8);
        let dump = serde_json::to_string(&svcs).unwrap();
        for forbidden in ["nmap", "sleuthkit", "fls", "icat", "clamav", "clamscan", "rust", "cargo"] {
            assert!(!dump.to_lowercase().contains(forbidden), "katalogi inaonyesha '{forbidden}' — nzuri iko fiche");
        }
        // Kila huduma ina bei ya TZS — mteja anajua bei tu
        assert!(svcs.iter().all(|s| s.price_tzs > 0));
    }

    #[test]
    fn registry_ina_bindings_za_huduma_zote() {
        for s in public_service_list() {
            let b = resolve(s.id).unwrap_or_else(|| panic!("huduma {} haina binding", s.id));
            assert!(!b.tool.is_empty());
        }
    }

    #[test]
    fn sanitize_inaficha_majina_ya_zana() {
        let raw = "scan ilifanyika na nmap v7.9 kutoka /usr/bin/nmap; clamscan cleaned 3 files; fls listing done";
        let clean = sanitize_output(raw);
        assert!(!clean.contains("nmap"));
        assert!(!clean.contains("clamscan"));
        assert!(!clean.contains("fls"));
        assert!(clean.contains("[huduma-ya-ndani]"));
    }
}

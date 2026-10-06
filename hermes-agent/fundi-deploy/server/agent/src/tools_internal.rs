//! tools_internal.rs — BACKEND YA NDANI ya huduma (mod fiche — SI pub API).
//!
//! Hapa ndipo utekelezaji HALISI unapotokea: kila huduma ya umma inaitwa
//! kupitia binding yake (TOOL_REGISTRY kwenye tools.rs). Sheria:
//!   - HAKUNA shell: amri zinaendeshwa kwa arg-array (tokio::process) —
//!     hakuna injection; target inakaguliwa kwa whitelist ya herufi.
//!   - Timeout kwa kila utekelezaji — hakuna kazi inayokwama.
//!   - Matokeo yanarudishwa kama MUHTASARI ULIOSAFISHWA (sanitized summary):
//!     mteja anaona MATOKEO, kamwe jina la zana wala amri.

use crate::tools::{resolve, sanitize_output, ToolKind};
use std::time::Duration;

const EXEC_TIMEOUT: Duration = Duration::from_secs(25);

/// Target inayokubalika: hostname/IP fupi salama (herufi, namba, . - _ :)
pub(crate) fn valid_target(t: &str) -> bool {
    !t.is_empty() && t.len() <= 64 && t.chars().all(|c| c.is_alphanumeric() || "._-:".contains(c))
}

/// Endesha binary yenye args — bila shell, na timeout. Inarudisha (ok, stdout+stderr).
async fn run_tool(bin: &str, args: &[&str]) -> (bool, String) {
    let child = tokio::time::timeout(
        EXEC_TIMEOUT,
        tokio::process::Command::new(bin).args(args).output(),
    )
    .await;
    match child {
        Err(_) => (false, "ucheleweshaji: kazi imechukua muda mrefu kuliko waliopangwa".into()),
        Ok(Err(e)) => (false, format!("haitapaatikana hapa: {e}")),
        Ok(Ok(out)) => {
            let mut text = String::from_utf8_lossy(&out.stdout).to_string();
            text.push_str(&String::from_utf8_lossy(&out.stderr));
            (out.status.success(), text)
        }
    }
}

/// Muhtasari kwa mteja — ULIOSAFISHWA (bila zana/amri/paths).
fn summary(ok: bool, service: &str, details: &str) -> serde_json::Value {
    serde_json::json!({
        "ok": ok,
        "service": service,
        "summary": sanitize_output(&details),
        "note_sw": "Matokeo yamekusanywa na injini ya ndani ya MTECH OS.",
    })
}

/// Endesha huduma kwa target (mf. IP ya PC) — kila huduma kwa njia yake.
pub(crate) async fn run_service(service: &str, target: &str) -> serde_json::Value {
    if !valid_target(target) {
        return summary(false, service, "target si salama (tumia hostname/IP fupi)");
    }
    let binding = match resolve(service) {
        Some(b) => b,
        None => return summary(false, service, "huduma haipatikani"),
    };
    match binding.kind {
        ToolKind::BuiltIn => builtin_service(service, target).await,
        ToolKind::Command => command_service(binding.tool, service, target).await,
    }
}

/// Huduma za BuiltIn (pure Rust — hazihitaji binary ya nje).
async fn builtin_service(service: &str, target: &str) -> serde_json::Value {
    match service {
        "health_check" => {
            // Ping ya TCP kwenye bandari za kawaida — hali ya kifaa
            let ports = [22u16, 80, 443, 445, 3389];
            let mut open = 0;
            for p in ports {
                if tokio::net::TcpStream::connect((target, p)).await.is_ok() {
                    open += 1;
                }
            }
            let ok = open > 0;
            summary(
                ok,
                service,
                &format!("kifaa kinajibu kwenye bandari {open}/{} muhimu — afya {}", ports.len(), if ok { "nzuri" } else { "haipatikani" }),
            )
        }
        "os_install" | "app_install" | "driver_update" | "device_management" => {
            // Zawaazi hizi ni orchestrator/pipeline za fundi-deploy (HITL) —
            // hapa tunathibitisha tu kuwa target iko hai kabla kazi kuingia job queue.
            let reachable = tokio::net::TcpStream::connect((target, 22)).await.is_ok()
                || tokio::net::TcpStream::connect((target, 445)).await.is_ok();
            if reachable {
                summary(true, service, "kifaa kiko hai — kazi imeingizwa kwenye foleni ya usakinishaji (HITL: itaanza baada ya idhini)")
            } else {
                summary(false, service, "kifaa hakijibu — hakikisha kiko wima na kwenye mtandao")
            }
        }
        _ => summary(false, service, "huduma hii haipatikani kwa built-in"),
    }
}

/// Huduma za Command — binary halisi ya ndani, args salama, matokeo yanasafishwa.
async fn command_service(tool: &'static str, service: &str, target: &str) -> serde_json::Value {
    let (ok, raw): (bool, String) = match tool {
        "nmap" => {
            // Scan ya bandari za kawaida — host discovery + ports
            let args = ["-Pn", "--open", "-p", "22,80,443,445,3389", target];
            let (ok, out) = run_tool(tool, &args).await;
            let parsed = parse_nmap(&out);
            (ok, parsed)
        }
        "sleuthkit" => {
            // Uchunguzi wa filesystem kwenye picha/disk ya target (read-only listing)
            let args = ["-f", "ntfs", "-r", target];
            run_tool(tool, &args).await
        }
        "clamav" => {
            // Scan ya ndani ya kifaa (clamscan inaendesha kwenye path ya scan)
            let args = ["--no-summary", "--infected", target];
            run_tool(tool, &args).await
        }
        _ => (false, "huduma haina utekelezaji wa ndani".into()),
    };
    if raw.trim().is_empty() {
        return summary(
            ok,
            service,
            if ok { "kazi imekamilika — hakuna dalili za hatari" } else { "kazi imefeli: zana ya ndani haipatikani au kifaa hakijibu" },
        );
    }
    summary(ok, service, &raw)
}

/// Parse ya output ya scan — kila mstari wa bandari kuwa muhtasari safi.
fn parse_nmap(raw: &str) -> String {
    let mut lines = Vec::new();
    for l in raw.lines() {
        let l = l.trim();
        // Muundo wa scan result: "22/tcp  open  ssh"
        if l.ends_with("/tcp") || l.contains("/tcp ") || l.contains("/udp ") {
            lines.push(l.to_string());
        }
    }
    if lines.is_empty() {
        "hakuna bandari wazi zilizopatikana".to_string()
    } else {
        format!("bandari wazi:\n{}", lines.join("\n"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_target_inakataa_vitendosti() {
        assert!(valid_target("192.168.1.10"));
        assert!(valid_target("hr-pc-01"));
        assert!(!valid_target(""));
        assert!(!valid_target("bad; rm -rf /"));
        assert!(!valid_target("$(whoami)"));
        assert!(!valid_target(&"a".repeat(100)));
    }

    #[test]
    fn parse_nmap_inatoa_bandari_tu() {
        let raw = "Starting scan...\n22/tcp   open  ssh\n80/tcp   open  http\nNmap done";
        let out = parse_nmap(raw);
        assert!(out.contains("22/tcp"));
        assert!(out.contains("80/tcp"));
        assert!(!out.contains("Starting"), "header za scan hazionekani");
    }

    #[tokio::test]
    async fn run_service_inarudisha_summary_iliyosafishwa() {
        // target isiyo salama → summary ya kosa (hakuna panic, hakuna amri)
        let v = run_service("network_scanner", "bad; rm -rf /").await;
        assert_eq!(v["ok"], serde_json::Value::Bool(false));
        assert!(v["summary"].as_str().unwrap().contains("salama"));
    }

    #[tokio::test]
    async fn huduma_isiyojulikana_inarudisha_kosa_laini() {
        let v = run_service("huduma-ya-siri", "192.168.1.5").await;
        assert_eq!(v["ok"], serde_json::Value::Bool(false));
    }
}

//! OS tool discovery + allowlisted diagnostic runners
//! Hutumia zana zilizopo kwenye PATH; haidownload automatically (usalama).
//! Catalog ya zana za open-source + install hints.

use serde::Serialize;
use std::process::Command;
use which_like::find_binary;

// Minimal which without extra crate
mod which_like {
    use std::env;
    use std::path::PathBuf;

    pub fn find_binary(name: &str) -> Option<PathBuf> {
        let path = env::var_os("PATH")?;
        for dir in env::split_paths(&path) {
            let candidate = dir.join(name);
            if candidate.is_file() {
                return Some(candidate);
            }
            #[cfg(windows)]
            {
                let exe = dir.join(format!("{}.exe", name));
                if exe.is_file() {
                    return Some(exe);
                }
                let cmd = dir.join(format!("{}.cmd", name));
                if cmd.is_file() {
                    return Some(cmd);
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolInfo {
    pub id: String,
    pub name: String,
    pub present: bool,
    pub path: Option<String>,
    pub platforms: Vec<String>,
    pub purpose_sw: String,
    pub install_hint: String,
    pub download_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolRunResult {
    pub tool_id: String,
    pub ok: bool,
    pub command: String,
    pub exit_code: Option<i32>,
    pub stdout_preview: String,
    pub stderr_preview: String,
    pub findings_sw: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ToolboxReport {
    pub os: String,
    pub tools: Vec<ToolInfo>,
    pub present_count: usize,
    pub missing_count: usize,
    pub recommended_install: Vec<ToolInfo>,
}

fn trunc(s: &str, n: usize) -> String {
    let t = s.trim();
    if t.len() <= n {
        t.to_string()
    } else {
        format!("{}…", &t[..n])
    }
}

fn catalog() -> Vec<ToolInfo> {
    vec![
        ToolInfo {
            id: "uname".into(),
            name: "uname".into(),
            present: false,
            path: None,
            platforms: vec!["linux".into(), "macos".into()],
            purpose_sw: "Jina la kernel na hardware arch".into(),
            install_hint: "coreutils (kawaida ipo)".into(),
            download_url: None,
        },
        ToolInfo {
            id: "df".into(),
            name: "df".into(),
            present: false,
            path: None,
            platforms: vec!["linux".into(), "macos".into()],
            purpose_sw: "Nafasi ya diski".into(),
            install_hint: "coreutils".into(),
            download_url: None,
        },
        ToolInfo {
            id: "free".into(),
            name: "free".into(),
            present: false,
            path: None,
            platforms: vec!["linux".into()],
            purpose_sw: "Matumizi ya RAM".into(),
            install_hint: "procps".into(),
            download_url: None,
        },
        ToolInfo {
            id: "ps".into(),
            name: "ps".into(),
            present: false,
            path: None,
            platforms: vec!["linux".into(), "macos".into()],
            purpose_sw: "Orodha ya processes".into(),
            install_hint: "procps / built-in".into(),
            download_url: None,
        },
        ToolInfo {
            id: "ss".into(),
            name: "ss".into(),
            present: false,
            path: None,
            platforms: vec!["linux".into()],
            purpose_sw: "Sockets / ports".into(),
            install_hint: "iproute2".into(),
            download_url: None,
        },
        ToolInfo {
            id: "ip".into(),
            name: "ip".into(),
            present: false,
            path: None,
            platforms: vec!["linux".into()],
            purpose_sw: "Network interfaces".into(),
            install_hint: "iproute2".into(),
            download_url: None,
        },
        ToolInfo {
            id: "smartctl".into(),
            name: "smartctl".into(),
            present: false,
            path: None,
            platforms: vec!["linux".into(), "macos".into(), "windows".into()],
            purpose_sw: "Afya ya diski (SMART)".into(),
            install_hint: "smartmontools".into(),
            download_url: Some("https://www.smartmontools.org/".into()),
        },
        ToolInfo {
            id: "lsblk".into(),
            name: "lsblk".into(),
            present: false,
            path: None,
            platforms: vec!["linux".into()],
            purpose_sw: "Orodha ya block devices".into(),
            install_hint: "util-linux".into(),
            download_url: None,
        },
        ToolInfo {
            id: "dmesg".into(),
            name: "dmesg".into(),
            present: false,
            path: None,
            platforms: vec!["linux".into()],
            purpose_sw: "Ujumbe wa kernel (mwisho)".into(),
            install_hint: "util-linux".into(),
            download_url: None,
        },
        ToolInfo {
            id: "sensors".into(),
            name: "sensors".into(),
            present: false,
            path: None,
            platforms: vec!["linux".into()],
            purpose_sw: "Joto la hardware (lm-sensors)".into(),
            install_hint: "lm-sensors".into(),
            download_url: Some("https://hwmon.wiki.kernel.org/lm_sensors".into()),
        },
        ToolInfo {
            id: "osquery".into(),
            name: "osqueryi".into(),
            present: false,
            path: None,
            platforms: vec!["linux".into(), "macos".into(), "windows".into()],
            purpose_sw: "SQL juu ya OS inventory".into(),
            install_hint: "osquery package".into(),
            download_url: Some("https://www.osquery.io/downloads".into()),
        },
        ToolInfo {
            id: "powershell".into(),
            name: "powershell".into(),
            present: false,
            path: None,
            platforms: vec!["windows".into(), "linux".into()],
            purpose_sw: "Windows/Cross automation".into(),
            install_hint: "Windows built-in / PowerShell Core".into(),
            download_url: Some("https://github.com/PowerShell/PowerShell".into()),
        },
        ToolInfo {
            id: "wmic".into(),
            name: "wmic".into(),
            present: false,
            path: None,
            platforms: vec!["windows".into()],
            purpose_sw: "WMI legacy queries".into(),
            install_hint: "Windows (deprecated; tumia PowerShell)".into(),
            download_url: None,
        },
        ToolInfo {
            id: "systeminfo".into(),
            name: "systeminfo".into(),
            present: false,
            path: None,
            platforms: vec!["windows".into()],
            purpose_sw: "Maelezo ya mfumo Windows".into(),
            install_hint: "Windows built-in".into(),
            download_url: None,
        },
        ToolInfo {
            id: "netstat".into(),
            name: "netstat".into(),
            present: false,
            path: None,
            platforms: vec!["linux".into(), "macos".into(), "windows".into()],
            purpose_sw: "Connections / ports".into(),
            install_hint: "net-tools / built-in".into(),
            download_url: None,
        },
    ]
}

pub fn discover() -> ToolboxReport {
    let os = std::env::consts::OS.to_string();
    let mut tools = catalog();
    for t in &mut tools {
        if let Some(p) = find_binary(&t.name) {
            t.present = true;
            t.path = Some(p.display().to_string());
        }
        // Windows: also try powershell.exe style already handled in find_binary
    }
    let present_count = tools.iter().filter(|t| t.present).count();
    let missing_count = tools.len() - present_count;
    let recommended_install: Vec<ToolInfo> = tools
        .iter()
        .filter(|t| !t.present && t.download_url.is_some())
        .cloned()
        .collect();

    ToolboxReport {
        os,
        tools,
        present_count,
        missing_count,
        recommended_install,
    }
}

/// Allowlisted diagnostics only — no arbitrary shell
pub fn run_diagnostic(tool_id: &str) -> ToolRunResult {
    let (bin, args): (&str, Vec<&str>) = match tool_id {
        "uname" => ("uname", vec!["-a"]),
        "df" => ("df", vec!["-h"]),
        "free" => ("free", vec!["-h"]),
        "ps" => ("ps", vec!["aux"]),
        "ss" => ("ss", vec!["-tuln"]),
        "ip" => ("ip", vec!["-br", "a"]),
        "lsblk" => ("lsblk", vec!["-o", "NAME,SIZE,TYPE,MOUNTPOINT,FSTYPE"]),
        "dmesg" => ("dmesg", vec!["-T"]),
        "sensors" => ("sensors", vec![]),
        "smartctl" => ("smartctl", vec!["--scan"]),
        "netstat" => ("netstat", vec!["-an"]),
        "systeminfo" => ("systeminfo", vec![]),
        "powershell_sys" => (
            "powershell",
            vec![
                "-NoProfile",
                "-Command",
                "Get-CimInstance Win32_OperatingSystem | Select-Object Caption,TotalVisibleMemorySize,FreePhysicalMemory | Format-List",
            ],
        ),
        "powershell_disk" => (
            "powershell",
            vec!["-NoProfile", "-Command", "Get-PSDrive -PSProvider FileSystem | Format-Table"],
        ),
        other => {
            return ToolRunResult {
                tool_id: other.into(),
                ok: false,
                command: String::new(),
                exit_code: None,
                stdout_preview: String::new(),
                stderr_preview: "Kitendo hakiko kwenye allowlist".into(),
                findings_sw: vec!["Ruhusu tu diagnostic salama".into()],
            };
        }
    };

    // Prefer smartctl binary name resolution
    let bin_path = find_binary(bin)
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| bin.to_string());

    let cmd_str = format!("{} {}", bin_path, args.join(" "));
    let output = Command::new(&bin_path).args(&args).output();

    match output {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout);
            let stderr = String::from_utf8_lossy(&o.stderr);
            let findings = analyze_output(tool_id, &stdout);
            ToolRunResult {
                tool_id: tool_id.into(),
                ok: o.status.success() || !stdout.is_empty(),
                command: cmd_str,
                exit_code: o.status.code(),
                stdout_preview: trunc(&stdout, 4000),
                stderr_preview: trunc(&stderr, 800),
                findings_sw: findings,
            }
        }
        Err(e) => ToolRunResult {
            tool_id: tool_id.into(),
            ok: false,
            command: cmd_str,
            exit_code: None,
            stdout_preview: String::new(),
            stderr_preview: e.to_string(),
            findings_sw: vec![format!("Zana '{}' haipatikani au imeshindwa kuanzishwa", bin)],
        },
    }
}

fn analyze_output(tool_id: &str, stdout: &str) -> Vec<String> {
    let mut f = Vec::new();
    match tool_id {
        "df" => {
            for line in stdout.lines().skip(1) {
                if let Some(pct) = line.split_whitespace().find(|c| c.ends_with('%')) {
                    if let Ok(n) = pct.trim_end_matches('%').parse::<u32>() {
                        if n >= 92 {
                            f.push(format!("Diski imejaa {} — {}", pct, line));
                        }
                    }
                }
            }
            if f.is_empty() {
                f.push("Diski: hakuna matumizi ≥92% kwenye output".into());
            }
        }
        "free" => {
            if stdout.contains("Mem:") {
                f.push("RAM stats zimepokelewa (free -h)".into());
            }
        }
        "ss" | "netstat" => {
            let n = stdout.lines().count();
            f.push(format!("Mistari ya network output: {}", n));
        }
        "smartctl" => {
            if stdout.trim().is_empty() {
                f.push("smartctl --scan: hakuna device au ruhusa"
                    .into());
            } else {
                f.push("Diski zimegunduliwa na smartctl --scan".into());
            }
        }
        "sensors" => {
            if stdout.contains("°C") || stdout.contains("+" ) {
                f.push("Sensor za joto zimepatikana".into());
            }
        }
        "dmesg" => {
            let lower = stdout.to_lowercase();
            if lower.contains("error") || lower.contains("fail") {
                f.push("dmesg ina maneno error/fail — chunguza".into());
            } else {
                f.push("dmesg: hakuna error dhahiri kwenye preview".into());
            }
        }
        _ => {
            if !stdout.trim().is_empty() {
                f.push("Output imepokelewa".into());
            }
        }
    }
    f
}

/// Run a battery of available diagnostics for this OS
pub fn auto_diagnose() -> serde_json::Value {
    let report = discover();
    let mut results = Vec::new();
    let mut all_findings = Vec::new();

    let candidates = if cfg!(windows) {
        vec!["systeminfo", "powershell_sys", "powershell_disk", "netstat"]
    } else {
        vec!["uname", "df", "free", "ps", "ss", "ip", "lsblk", "sensors", "smartctl", "dmesg"]
    };

    for id in candidates {
        // skip if binary missing (except powershell_* map to powershell)
        let need = match id {
            "powershell_sys" | "powershell_disk" => "powershell",
            other => other,
        };
        let present = report.tools.iter().any(|t| t.name == need && t.present)
            || find_binary(need).is_some();
        if !present && id != "powershell_sys" && id != "powershell_disk" {
            continue;
        }
        let r = run_diagnostic(id);
        all_findings.extend(r.findings_sw.clone());
        results.push(r);
    }

    serde_json::json!({
        "os": report.os,
        "toolbox": report,
        "runs": results,
        "findings_sw": all_findings,
        "note_sw": "Zana zilizopo zimetumika. Download: fungua download_url kwa mkono (hakuna auto-install)."
    })
}

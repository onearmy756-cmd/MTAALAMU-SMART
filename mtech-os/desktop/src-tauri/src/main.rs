//! MTECH OS — Tauri v2 desktop shell (SEHEMU 1.1 UI + SEHEMU 9).
//!
//! UI mbili (mahitaji ya mmiliki):
//!   - UI 1: OS AND APP INSTALLATION (Fundi Deploy: /ui → Computers, OS, Bundles,
//!     VPN, Agentic AI, Admin)
//!   - UI 2: COMPUTER SOLUTIONS (scan matatizo, chat, kila siku, dashboards)
//!
//! Shell hii inafungua webview inayoelekeza API ya agent (Rust, :8080) — mfumo
//! mzima unaendeshwa na Rust; Tauri ni dirisha tu (bila EXE ya wageni).

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Manager;

/// Mwelekeo wa API ya agent (default: http://127.0.0.1:8080 — server ya ndani).
#[tauri::command]
fn api_base() -> String {
    std::env::var("MTECH_API").unwrap_or_else(|_| "http://127.0.0.1:8080".into())
}

/// Health check ya agent kabla ya kufungua UI (hakuna uongo).
#[tauri::command]
async fn agent_health(api: String) -> Result<String, String> {
    let url = format!("{api}/health");
    reqwest::get(&url)
        .await
        .map_err(|e| format!("Agent haipatikani ({e}) — anzisha: docker compose up -d"))?
        .text()
        .await
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![api_base, agent_health])
        .setup(|app| {
            let base = api_base();
            let win = app.get_webview_window("main").expect("window 'main' haipo");
            // UI moja yenye tabs zote mbili (OS AND APP INSTALLATION + COMPUTER SOLUTIONS)
            win.eval(&format!("window.location.replace('{base}/ui');"))?;
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("MTECH OS desktop imekufa");
}

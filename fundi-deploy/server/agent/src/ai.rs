//! AI OS selector — Ollama (offline) au rules fallback

use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct GenReq {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Deserialize)]
struct GenResp {
    response: Option<String>,
}

/// Chagua OS: jaribu Ollama; kama fail → rules (RAM/CPU text)
pub async fn select_os(computer_specs: &str, user_need: &str) -> String {
    if let Ok(choice) = ollama_select(computer_specs, user_need).await {
        return choice;
    }
    rules_select(computer_specs, user_need)
}

async fn ollama_select(specs: &str, need: &str) -> anyhow::Result<String> {
    let prompt = format!(
        "Wewe ni mtaalamu wa IT. Chagua OS moja tu: win11, win10, ubuntu, debian, fedora.\nSpecs:\n{specs}\nMahitaji: {need}\nJibu: jina la OS pekee."
    );
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()?;
    let resp: GenResp = client
        .post("http://127.0.0.1:11434/api/generate")
        .json(&GenReq {
            model: std::env::var("FUNDI_AI_MODEL").unwrap_or_else(|_| "tinyllama".into()),
            prompt,
            stream: false,
        })
        .send()
        .await?
        .json()
        .await?;
    let text = resp.response.unwrap_or_default().to_lowercase();
    for v in ["win11", "win10", "ubuntu", "debian", "fedora"] {
        if text.contains(v) {
            return Ok(v.to_string());
        }
    }
    anyhow::bail!("no valid os")
}

fn rules_select(specs: &str, need: &str) -> String {
    let s = format!("{specs} {need}").to_lowercase();
    if s.contains("linux") || s.contains("developer") || s.contains("docker") {
        return "ubuntu".into();
    }
    if s.contains("4gb") || s.contains("4 gb") || s.contains("i3") {
        return "win10".into();
    }
    if s.contains("ram error") || s.contains("smart fail") {
        return "skip".into();
    }
    "win11".into()
}

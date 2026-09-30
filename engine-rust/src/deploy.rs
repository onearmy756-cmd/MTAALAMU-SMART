//! Fundi Deploy — integration na engine ya MTAALAMU (CLI).
//!
//! `mtaalamu deploy hosts|jobs|images|summary|cloud|os|start|approve|cancel`
//!
//! Kanuni: HITL — deploy ya kweli inahitaji ruhusa; CLI inaomba tu,
//! msimamizi anaruhusu kwenye dashboard `/ui` au `mtaalamu deploy approve`.

use serde_json::Value;

fn base_url() -> String {
    std::env::var("FUNDI_DEPLOY_URL").unwrap_or_else(|_| "http://127.0.0.1:8080".into())
}

fn client() -> reqwest::blocking::Client {
    reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .expect("http client")
}

fn get_json(path: &str) -> Result<Value, String> {
    let url = format!("{}{}", base_url(), path);
    let resp = client()
        .get(&url)
        .send()
        .map_err(|e| format!("Fundi Deploy haipatikani ({url}): {e}"))?;
    resp.json().map_err(|e| e.to_string())
}

fn post_json(path: &str, body: Value) -> Result<Value, String> {
    let url = format!("{}{}", base_url(), path);
    let resp = client()
        .post(&url)
        .json(&body)
        .send()
        .map_err(|e| format!("Fundi Deploy haipatikani ({url}): {e}"))?;
    resp.json().map_err(|e| e.to_string())
}

/// GET /computers — hosts za LAN (halisi au demo)
pub fn discover() -> Result<Value, String> {
    get_json("/computers")
}

/// GET /jobs — hali ya jobs
pub fn jobs() -> Result<Value, String> {
    get_json("/jobs")
}

/// GET /images — images za OS (na validation sha256/magic)
pub fn images() -> Result<Value, String> {
    get_json("/images")
}

/// POST /os/select — mapendekezo ya OS (reasons + scores)
pub fn os_select(specs: &str, need: &str) -> Result<Value, String> {
    post_json(
        "/os/select",
        serde_json::json!({ "specs": specs, "user_need": need }),
    )
}

/// POST /deploy — anza deploy (HITL: inasubiri approve ya msimamizi)
pub fn deploy_hitl(computers: Value, os: &str, need: &str) -> Result<Value, String> {
    post_json(
        "/deploy",
        serde_json::json!({
            "computers": computers,
            "os_type": os,
            "user_need": need,
            "auto_approve": false
        }),
    )
}

/// POST /jobs/:id/approve
pub fn approve(job_id: &str) -> Result<Value, String> {
    post_json(&format!("/jobs/{job_id}/approve"), serde_json::json!({}))
}

/// POST /jobs/:id/cancel
pub fn cancel(job_id: &str) -> Result<Value, String> {
    post_json(&format!("/jobs/{job_id}/cancel"), serde_json::json!({}))
}

/// GET /supervisor/summary
pub fn summary() -> Result<Value, String> {
    get_json("/supervisor/summary")
}

/// GET /cloud/status
pub fn cloud_status() -> Result<Value, String> {
    get_json("/cloud/status")
}

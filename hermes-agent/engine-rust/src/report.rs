//! Digital Book Report Builder — ripoti kama kitabu kidigitali

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportChapter {
    pub id: String,
    pub title_sw: String,
    pub content: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigitalReport {
    pub session_id: String,
    pub title_sw: String,
    pub created_at: u64,
    pub language: String,
    pub chapters: Vec<ReportChapter>,
    pub summary_sw: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TemplateFile {
    pub title_sw: String,
    pub chapters: Vec<TemplateChapter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TemplateChapter {
    pub id: String,
    pub title_sw: String,
    pub fields: Vec<String>,
}

pub struct ReportEngine {
    template: TemplateFile,
}

impl ReportEngine {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = fs::read_to_string(path).map_err(|e| format!("report template: {}", e))?;
        let template: TemplateFile =
            serde_json::from_str(&text).map_err(|e| format!("report parse: {}", e))?;
        Ok(Self { template })
    }

    /// Jenga ripoti kutoka session context (Value map)
    pub fn build(
        &self,
        session_id: &str,
        context: &Value,
        language: &str,
    ) -> DigitalReport {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);

        let mut chapters = Vec::new();
        for ch in &self.template.chapters {
            let mut content = serde_json::Map::new();
            for field in &ch.fields {
                if let Some(v) = context.get(field) {
                    content.insert(field.clone(), v.clone());
                } else {
                    content.insert(field.clone(), Value::Null);
                }
            }
            // Auto-fill common
            if ch.id == "cover" {
                content.insert("session_id".into(), json!(session_id));
                content.insert("date".into(), json!(format_date(ts)));
                content.insert("time".into(), json!(format_time(ts)));
            }
            chapters.push(ReportChapter {
                id: ch.id.clone(),
                title_sw: ch.title_sw.clone(),
                content: Value::Object(content),
            });
        }

        let summary_sw = context
            .get("summary_sw")
            .and_then(|v| v.as_str())
            .unwrap_or("Ripoti ya utatuzi wa Mtaalamu Smart.")
            .to_string();

        DigitalReport {
            session_id: session_id.to_string(),
            title_sw: self.template.title_sw.clone(),
            created_at: ts,
            language: language.to_string(),
            chapters,
            summary_sw,
            status: context
                .get("status")
                .and_then(|v| v.as_str())
                .unwrap_or("completed")
                .to_string(),
        }
    }

    /// Export Markdown (kitabu kidigitali cha maandishi)
    pub fn to_markdown(&self, report: &DigitalReport) -> String {
        let mut md = String::new();
        md.push_str(&format!("# {}\n\n", report.title_sw));
        md.push_str(&format!("**Session:** {}  \n", report.session_id));
        md.push_str(&format!("**Hali:** {}  \n\n", report.status));
        md.push_str("---\n\n");
        for ch in &report.chapters {
            md.push_str(&format!("## {}\n\n", ch.title_sw));
            if let Some(obj) = ch.content.as_object() {
                for (k, v) in obj {
                    if !v.is_null() {
                        md.push_str(&format!("- **{}:** {}\n", k, value_brief(v)));
                    }
                }
            }
            md.push_str("\n");
        }
        md.push_str(&format!("\n---\n\n### Hitimisho\n\n{}\n", report.summary_sw));
        md
    }
}

fn format_date(ts: u64) -> String {
    // Simplified ISO-like; production use chrono
    format!("{}", ts)
}

fn format_time(ts: u64) -> String {
    format!("{}", ts)
}

fn value_brief(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Number(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Array(a) => format!("[{} items]", a.len()),
        Value::Object(o) => format!("{{{} fields}}", o.len()),
        Value::Null => "—".into(),
    }
}

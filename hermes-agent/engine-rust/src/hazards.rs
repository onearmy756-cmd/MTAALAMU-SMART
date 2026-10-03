//! Hazards engine — load types from data/hazards/types.json

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HazardType {
    pub id: String,
    pub name_sw: String,
    pub name_en: String,
    pub severity: u8,
    #[serde(default)]
    pub voice_sw: String,
    #[serde(default)]
    pub color: String,
}

#[derive(Debug, Clone)]
pub struct HazardsEngine {
    types: Vec<HazardType>,
    raw: Value,
}

impl HazardsEngine {
    pub fn load(data_dir: &Path) -> Result<Self, String> {
        let path = data_dir.join("hazards").join("types.json");
        if !path.exists() {
            return Ok(HazardsEngine {
                types: vec![],
                raw: Value::Null,
            });
        }
        let s = fs::read_to_string(&path).map_err(|e| format!("hazards: {}", e))?;
        let raw: Value = serde_json::from_str(&s).map_err(|e| format!("hazards parse: {}", e))?;
        let mut types = Vec::new();
        if let Some(arr) = raw.get("types").and_then(|t| t.as_array()) {
            for item in arr {
                if let Ok(ht) = serde_json::from_value::<HazardType>(item.clone()) {
                    types.push(ht);
                }
            }
        }
        Ok(HazardsEngine { types, raw })
    }

    pub fn all(&self) -> &[HazardType] {
        &self.types
    }

    pub fn by_id(&self, id: &str) -> Option<&HazardType> {
        self.types.iter().find(|t| t.id == id)
    }

    pub fn voice_for(&self, id: &str) -> String {
        self.by_id(id)
            .map(|t| t.voice_sw.clone())
            .unwrap_or_else(|| "Tahadhari: hatari mbele".into())
    }

    pub fn count(&self) -> usize {
        self.types.len()
    }

    pub fn raw(&self) -> &Value {
        &self.raw
    }
}

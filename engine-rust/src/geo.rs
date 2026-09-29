//! Location & Geo hierarchy engine
//! Loads data/geo/*.json and supports search across all levels.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeoPlace {
    pub id: String,
    pub name_sw: String,
    #[serde(default)]
    pub name_en: String,
    pub level: String,
    #[serde(default)]
    pub region_id: Option<String>,
    #[serde(default)]
    pub district_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct GeoEngine {
    places: Vec<GeoPlace>,
    hierarchy: Value,
}

impl GeoEngine {
    /// Load hierarchy + all level files from data/geo/
    pub fn load(data_dir: &Path) -> Result<Self, String> {
        let geo = data_dir.join("geo");
        let hierarchy_path = geo.join("hierarchy.json");
        let hierarchy: Value = if hierarchy_path.exists() {
            let s = fs::read_to_string(&hierarchy_path)
                .map_err(|e| format!("hierarchy.json: {}", e))?;
            serde_json::from_str(&s).map_err(|e| format!("hierarchy parse: {}", e))?
        } else {
            Value::Null
        };

        let mut places = Vec::new();

        // Load each level file
        let files = [
            ("districts.json", "district"),
            ("tarafa.json", "tarafa"),
            ("kata.json", "kata"),
            ("vijiji.json", "kijiji"),
            ("vitongoji.json", "vitongoji"),
            ("mitaa.json", "mtaa"),
            ("barabara.json", "barabara"),
        ];

        for (fname, level) in files {
            let path = geo.join(fname);
            if !path.exists() {
                continue;
            }
            let s = fs::read_to_string(&path).map_err(|e| format!("{}: {}", fname, e))?;
            let v: Value = serde_json::from_str(&s).map_err(|e| format!("{} parse: {}", fname, e))?;
            // Array key varies: districts, tarafa, kata, vijiji, vitongoji, mitaa, barabara
            let arr = v
                .as_object()
                .and_then(|o| {
                    o.values()
                        .find(|x| x.is_array())
                        .cloned()
                })
                .unwrap_or(Value::Array(vec![]));

            if let Some(items) = arr.as_array() {
                for item in items {
                    let id = item.get("id").and_then(|x| x.as_str()).unwrap_or("").to_string();
                    let name_sw = item
                        .get("name_sw")
                        .and_then(|x| x.as_str())
                        .unwrap_or("")
                        .to_string();
                    if id.is_empty() || name_sw.is_empty() {
                        continue;
                    }
                    places.push(GeoPlace {
                        id,
                        name_sw,
                        name_en: item
                            .get("name_en")
                            .and_then(|x| x.as_str())
                            .unwrap_or("")
                            .to_string(),
                        level: level.to_string(),
                        region_id: item
                            .get("region_id")
                            .and_then(|x| x.as_str())
                            .map(|s| s.to_string()),
                        district_id: item
                            .get("district_id")
                            .and_then(|x| x.as_str())
                            .map(|s| s.to_string()),
                    });
                }
            }
        }

        Ok(GeoEngine { places, hierarchy })
    }

    pub fn search(&self, query: &str, limit: usize) -> Vec<&GeoPlace> {
        let q = query.to_lowercase();
        if q.is_empty() {
            return vec![];
        }
        self.places
            .iter()
            .filter(|p| {
                p.name_sw.to_lowercase().contains(&q)
                    || p.name_en.to_lowercase().contains(&q)
                    || p.id.to_lowercase().contains(&q)
            })
            .take(limit)
            .collect()
    }

    pub fn by_id(&self, id: &str) -> Option<&GeoPlace> {
        self.places.iter().find(|p| p.id == id)
    }

    pub fn by_level(&self, level: &str) -> Vec<&GeoPlace> {
        self.places.iter().filter(|p| p.level == level).collect()
    }

    pub fn count(&self) -> usize {
        self.places.len()
    }

    pub fn hierarchy(&self) -> &Value {
        &self.hierarchy
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn search_empty_query() {
        let eng = GeoEngine {
            places: vec![GeoPlace {
                id: "TZ-02-01".into(),
                name_sw: "Ilala".into(),
                name_en: "Ilala".into(),
                level: "district".into(),
                region_id: Some("TZ-02".into()),
                district_id: None,
            }],
            hierarchy: Value::Null,
        };
        assert!(eng.search("", 10).is_empty());
        assert_eq!(eng.search("ilala", 10).len(), 1);
    }
}

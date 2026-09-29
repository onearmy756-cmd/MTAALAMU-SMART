//! decision_tree.rs — Decision tree (data-driven).
//! Inasoma `data/decision_trees/*.json`: nodes [{id, question?, yes?, no?, result?}]
//! Kutembea ni deterministic; hakuna AI katika uamuzi.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeResult {
    pub diagnosis: String,
    #[serde(default)]
    pub solution: String,
    #[serde(default)]
    pub confidence: f64,
    #[serde(default)]
    pub actions: Vec<String>,
    #[serde(default)]
    pub cost_tzs: Option<f64>,
    #[serde(default)]
    pub time_min: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Node {
    pub id: String,
    #[serde(default)]
    pub question: String,
    #[serde(default)]
    pub yes: Option<String>,
    #[serde(default)]
    pub no: Option<String>,
    #[serde(default)]
    pub result: Option<NodeResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TreeFile {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub start: String,
    pub nodes: Vec<Node>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trace {
    pub node: String,
    #[serde(default)]
    pub question: String,
    pub answer: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalkResult {
    pub result: NodeResult,
    pub path: Vec<Trace>,
    pub steps_taken: usize,
}

pub struct DecisionTree {
    name: String,
    start: String,
    nodes: HashMap<String, Node>,
    order: Vec<String>,
}

impl DecisionTree {
    pub fn from_json(text: &str) -> Result<Self, String> {
        let f: TreeFile =
            serde_json::from_str(text).map_err(|e| format!("decision tree JSON si sahihi: {}", e))?;
        Self::from_file(f)
    }

    pub fn load_file(path: &std::path::Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("Haiwezi kusoma {}: {}", path.display(), e))?;
        Self::from_json(&text)
    }

    fn from_file(f: TreeFile) -> Result<Self, String> {
        if f.nodes.is_empty() {
            return Err("Tree haina nodes".into());
        }
        let mut map = HashMap::new();
        for n in &f.nodes {
            map.insert(n.id.clone(), n.clone());
        }
        // kiungo kilivunjika?
        for n in &f.nodes {
            if let Some(y) = &n.yes {
                if !map.contains_key(y) {
                    return Err(format!("Node '{}' yes='{}' haipo", n.id, y));
                }
            }
            if let Some(nn) = &n.no {
                if !map.contains_key(nn) {
                    return Err(format!("Node '{}' no='{}' haipo", n.id, nn));
                }
            }
        }
        let start = if f.start.is_empty() {
            f.nodes[0].id.clone()
        } else {
            f.start
        };
        if !map.contains_key(&start) {
            return Err(format!("Start node '{}' haipo", start));
        }
        let order: Vec<String> = f.nodes.iter().map(|n| n.id.clone()).collect();
        Ok(DecisionTree { name: f.name, start, nodes: map, order })
    }

    pub fn name(&self) -> &str { &self.name }
    pub fn len(&self) -> usize { self.nodes.len() }
    pub fn node_ids(&self) -> &[String] { &self.order }

    /// Swali la node ya sasa (au None kama ni result node).
    pub fn question_at(&self, id: &str) -> Option<String> {
        self.nodes.get(id).and_then(|n| {
            if n.question.is_empty() { None } else { Some(n.question.clone()) }
        })
    }

    /// Tembelea tree kwa majibu (R-215). answers[i] = jibu la swali la hatua i.
    /// Kama majibu yanaisha mapema: rudisha "Hakuna jibu — data haitoshi".
    pub fn walk(&self, answers: &[bool]) -> Result<WalkResult, String> {
        let mut cur = self.start.clone();
        let mut path = Vec::new();
        let mut i = 0usize;
        loop {
            let node = self
                .nodes
                .get(&cur)
                .ok_or_else(|| format!("Node '{}' haipo", cur))?;
            if let Some(res) = &node.result {
                return Ok(WalkResult {
                    result: res.clone(),
                    path,
                    steps_taken: i,
                });
            }
            // ni swali — linahitaji jibu
            let answer = match answers.get(i) {
                Some(a) => *a,
                None => {
                    return Err(format!(
                        "Hakuna jibu — data haitoshi (swali: {})",
                        node.question
                    ))
                }
            };
            path.push(Trace {
                node: node.id.clone(),
                question: node.question.clone(),
                answer: Some(answer),
            });
            let next = if answer { node.yes.clone() } else { node.no.clone() };
            match next {
                Some(n) => {
                    cur = n;
                    i += 1;
                }
                None => {
                    return Err(format!(
                        "Node '{}' haina mwelekeo (question: {})",
                        node.id, node.question
                    ))
                }
            }
            if i > self.nodes.len() * 4 {
                return Err("Mzunguko wa tree umepita kiwango".into());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TREE: &str = r#"{
      "name": "Gari",
      "start": "0",
      "nodes": [
        {"id":"0","question":"Gari inawaka?","yes":"1","no":"6"},
        {"id":"1","question":"Check engine light imewaka?","yes":"2","no":"4"},
        {"id":"2","question":"OBD inatoa code P0300?","yes":"3","no":"5"},
        {"id":"3","result":{"diagnosis":"Spark plugs zimechakaa","solution":"Badilisha spark plugs","confidence":0.85,"actions":["Chukua spark plugs mpya","Badilisha","Test gari"],"cost_tzs":80000,"time_min":60}},
        {"id":"4","question":"Gari inatumia fuel nyingi?","yes":"7","no":"8"},
        {"id":"5","result":{"diagnosis":"Catalyst au O2 sensor","solution":"Kagua catalyst/O2 sensor","confidence":0.75,"actions":["Scan O2 sensor","Badilisha kama imevunjika"],"cost_tzs":250000,"time_min":120}},
        {"id":"6","result":{"diagnosis":"Battery / fuel pump","solution":"Kagua battery na fuel pump","confidence":0.8,"actions":["Volta 12.6V","Kagua fuel pump"],"cost_tzs":150000,"time_min":90}},
        {"id":"7","result":{"diagnosis":"Air filter / injectors","solution":"Safisha air filter","confidence":0.7,"actions":["Badilisha air filter","Safisha injectors"],"cost_tzs":60000,"time_min":45}},
        {"id":"8","result":{"diagnosis":"Hakuna hitaji — gari nzuri","solution":"Hakuna kazi","confidence":0.9,"actions":["Endelea"],"cost_tzs":0,"time_min":0}}
      ]
    }"#;

    #[test]
    fn loads_and_counts() {
        let t = DecisionTree::from_json(TREE).unwrap();
        assert_eq!(t.len(), 9);
        assert_eq!(t.name(), "Gari");
    }

    #[test]
    fn walk_yes_yes_yes() {
        let t = DecisionTree::from_json(TREE).unwrap();
        let r = t.walk(&[true, true, true]).unwrap();
        assert_eq!(r.result.diagnosis, "Spark plugs zimechakaa");
        assert_eq!(r.steps_taken, 3);
        assert_eq!(r.path.len(), 3);
        assert!(r.path[0].question.contains("inawaka"));
    }

    #[test]
    fn walk_no_branch() {
        let t = DecisionTree::from_json(TREE).unwrap();
        let r = t.walk(&[false]).unwrap();
        assert_eq!(r.result.diagnosis, "Battery / fuel pump");
        assert_eq!(r.result.cost_tzs, Some(150000.0));
    }

    #[test]
    fn insufficient_answers_errors_swahili() {
        let t = DecisionTree::from_json(TREE).unwrap();
        let err = t.walk(&[]).unwrap_err();
        assert!(err.contains("Hakuna jibu — data haitoshi"), "{}", err);
    }

    #[test]
    fn broken_link_rejected() {
        let bad = r#"{"start":"0","nodes":[{"id":"0","question":"Q?","yes":"404","no":"0"}]}"#;
        let err = DecisionTree::from_json(bad).unwrap_err();
        assert!(err.contains("haipo"), "{}", err);
    }

    #[test]
    fn question_at() {
        let t = DecisionTree::from_json(TREE).unwrap();
        assert!(t.question_at("0").unwrap().contains("inawaka"));
        assert!(t.question_at("3").is_none()); // result node
    }
}

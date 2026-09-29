//! Navigation engine — turn-by-turn instructions + voice (Kiswahili)
//! Data-driven from data/navigation/*.json

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NavInstruction {
    pub text_sw: String,
    pub text_en: String,
    pub voice_key: Option<String>,
    pub distance_m: Option<f64>,
    pub maneuver: String,
}

#[derive(Debug, Clone)]
pub struct NavigationEngine {
    voice_prompts: HashMap<String, String>,
    turn_instructions: Value,
    alarm_rules: Value,
}

impl NavigationEngine {
    pub fn load(data_dir: &Path) -> Result<Self, String> {
        let nav = data_dir.join("navigation");

        let voice_path = nav.join("voice_prompts_sw.json");
        let mut voice_prompts = HashMap::new();
        if voice_path.exists() {
            let s = fs::read_to_string(&voice_path)
                .map_err(|e| format!("voice_prompts: {}", e))?;
            let v: Value = serde_json::from_str(&s).map_err(|e| format!("voice parse: {}", e))?;
            if let Some(prompts) = v.get("prompts").and_then(|p| p.as_object()) {
                for (k, val) in prompts {
                    if let Some(t) = val.as_str() {
                        voice_prompts.insert(k.clone(), t.to_string());
                    }
                }
            }
        }

        let turn_path = nav.join("turn_instructions.json");
        let turn_instructions = if turn_path.exists() {
            let s = fs::read_to_string(&turn_path)
                .map_err(|e| format!("turn_instructions: {}", e))?;
            serde_json::from_str(&s).map_err(|e| format!("turn parse: {}", e))?
        } else {
            Value::Null
        };

        let alarm_path = nav.join("alarm_rules.json");
        let alarm_rules = if alarm_path.exists() {
            let s = fs::read_to_string(&alarm_path)
                .map_err(|e| format!("alarm_rules: {}", e))?;
            serde_json::from_str(&s).map_err(|e| format!("alarm parse: {}", e))?
        } else {
            Value::Null
        };

        Ok(NavigationEngine {
            voice_prompts,
            turn_instructions,
            alarm_rules,
        })
    }

    /// Get voice prompt text by key, with optional {placeholders}
    pub fn voice(&self, key: &str, vars: &[(&str, &str)]) -> String {
        let mut text = self
            .voice_prompts
            .get(key)
            .cloned()
            .unwrap_or_else(|| key.to_string());
        for (k, v) in vars {
            text = text.replace(&format!("{{{}}}", k), v);
        }
        text
    }

    /// Build Kiswahili instruction for a maneuver
    pub fn instruction_sw(&self, maneuver: &str, distance_m: Option<f64>) -> String {
        let action = match maneuver {
            "turn-left" | "left" => self.voice("turn_left", &[]),
            "turn-right" | "right" => self.voice("turn_right", &[]),
            "turn-slight-left" => self.voice("turn_slight_left", &[]),
            "turn-slight-right" => self.voice("turn_slight_right", &[]),
            "turn-sharp-left" => self.voice("turn_sharp_left", &[]),
            "turn-sharp-right" => self.voice("turn_sharp_right", &[]),
            "uturn" | "u-turn" => self.voice("u_turn", &[]),
            "continue" | "straight" => self.voice("continue_straight", &[]),
            "roundabout" => self.voice("roundabout", &[]),
            "arrive" => self.voice("arrived", &[]),
            "depart" => self.voice("start_nav", &[]),
            _ => self.voice("follow_road", &[]),
        };

        if let Some(d) = distance_m {
            if d >= 1000.0 {
                let km = format!("{:.1}", d / 1000.0);
                format!("{}, {}", self.voice("distance_km", &[("distance", &km)]), action)
            } else if d > 0.0 {
                let m = format!("{:.0}", d);
                format!("{}, {}", self.voice("distance_meters", &[("distance", &m)]), action)
            } else {
                action
            }
        } else {
            action
        }
    }

    pub fn prompt_count(&self) -> usize {
        self.voice_prompts.len()
    }

    pub fn alarms(&self) -> &Value {
        &self.alarm_rules
    }

    pub fn turn_data(&self) -> &Value {
        &self.turn_instructions
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voice_placeholder() {
        let mut eng = NavigationEngine {
            voice_prompts: HashMap::new(),
            turn_instructions: Value::Null,
            alarm_rules: Value::Null,
        };
        eng.voice_prompts
            .insert("distance_meters".into(), "Baada ya mita {distance}.".into());
        eng.voice_prompts
            .insert("turn_left".into(), "Pinda kushoto.".into());
        let t = eng.voice("distance_meters", &[("distance", "200")]);
        assert!(t.contains("200"));
        let inst = eng.instruction_sw("left", Some(200.0));
        assert!(inst.contains("kushoto") || inst.contains("Pinda"));
    }
}

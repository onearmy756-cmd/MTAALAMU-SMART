//! i18n.rs — Lugha (Kiswahili/English), data-driven kutoka `data/locales/*.json`.
//! Key moja => {sw, en}. Hakuna maandishi ya lugha moja kwenye code.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BiText {
    #[serde(default)]
    pub sw: String,
    #[serde(default)]
    pub en: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    Sw,
    En,
}

impl Lang {
    pub fn from_code(code: &str) -> Lang {
        match code.to_lowercase().as_str() {
            "en" | "en-us" | "en-gb" | "english" => Lang::En,
            _ => Lang::Sw,
        }
    }
    pub fn code(&self) -> &'static str {
        match self {
            Lang::Sw => "sw",
            Lang::En => "en",
        }
    }
}

pub struct I18n {
    /// key -> {sw, en}
    entries: HashMap<String, BiText>,
    /// lugha ya sasa (default sw)
    lang: Lang,
}

impl I18n {
    pub fn new() -> Self {
        I18n {
            entries: HashMap::new(),
            lang: Lang::Sw,
        }
    }

    /// Pakia locale file: {"key": {"sw":"...","en":"..."}} au
    /// {"key": "string"} (string = lugha zote mbili).
    pub fn load_json(&mut self, text: &str) -> Result<usize, String> {
        let raw: HashMap<String, serde_json::Value> =
            serde_json::from_str(text).map_err(|e| format!("locale JSON si sahihi: {}", e))?;
        let n = raw.len();
        for (k, v) in raw {
            let bt = match v {
                serde_json::Value::String(s) => BiText {
                    sw: s.clone(),
                    en: s,
                },
                other => serde_json::from_value(other)
                    .map_err(|e| format!("Locale '{}': {}", k, e))?,
            };
            self.entries.insert(k, bt);
        }
        Ok(n)
    }

    pub fn load_file(&mut self, path: &std::path::Path) -> Result<usize, String> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| format!("Haiwezi kusoma {}: {}", path.display(), e))?;
        self.load_json(&text)
    }

    pub fn set_lang(&mut self, code: &str) {
        self.lang = Lang::from_code(code);
    }

    pub fn lang(&self) -> Lang {
        self.lang
    }

    pub fn count(&self) -> usize {
        self.entries.len()
    }

    /// Tafuta translation. Kama key haipo, rudisha key yenyewe (hufanya UI
    /// ionekane bila kuvunjika).
    pub fn t(&self, key: &str) -> String {
        match self.entries.get(key) {
            Some(bt) => match self.lang {
                Lang::Sw => bt.sw.clone(),
                Lang::En => bt.en.clone(),
            },
            None => key.to_string(),
        }
    }

    /// Tafuta kwa lugha maalum bila kubadilisha hali ya i18n.
    pub fn t_lang(&self, key: &str, lang: Lang) -> String {
        match self.entries.get(key) {
            Some(bt) => match lang {
                Lang::Sw => bt.sw.clone(),
                Lang::En => bt.en.clone(),
            },
            None => key.to_string(),
        }
    }

    /// Rudisha {sw, en} kwa key (kwa UI inayohitaji pande zote mbili).
    pub fn bi(&self, key: &str) -> BiText {
        self.entries
            .get(key)
            .cloned()
            .unwrap_or(BiText {
                sw: key.to_string(),
                en: key.to_string(),
            })
    }

    /// Je, key ipo?
    pub fn has(&self, key: &str) -> bool {
        self.entries.contains_key(key)
    }

    pub fn keys(&self) -> Vec<String> {
        let mut k: Vec<String> = self.entries.keys().cloned().collect();
        k.sort();
        k
    }
}

/// Rudisha tayari-BiText kwa API response (endpoint zote hurudisha sw+en).
pub fn bilingual(bt: &BiText) -> BiText {
    bt.clone()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn i18n() -> I18n {
        let mut i = I18n::new();
        let json = r#"{
          "app.title": {"sw": "MTAALAMU SMART", "en": "MTAALAMU SMART"},
          "calc.button": {"sw": "Hesabu", "en": "Calculate"},
          "status.good": {"sw": "Nzuri", "en": "Good"},
          "status.fail": {"sw": "Kushindwa", "en": "Fail"},
          "plain.key": "Plain"
        }"#;
        i.load_json(json).unwrap();
        i
    }

    #[test]
    fn loads_keys() {
        let i = i18n();
        assert_eq!(i.count(), 5);
        assert!(i.has("calc.button"));
        assert!(!i.has("haipo"));
    }

    #[test]
    fn default_lang_is_swahili() {
        let i = i18n();
        assert_eq!(i.t("calc.button"), "Hesabu");
        assert_eq!(i.t("status.good"), "Nzuri");
    }

    #[test]
    fn switch_to_english() {
        let mut i = i18n();
        i.set_lang("en");
        assert_eq!(i.t("calc.button"), "Calculate");
        assert_eq!(i.t("status.fail"), "Fail");
    }

    #[test]
    fn missing_key_returns_key() {
        let i = i18n();
        assert_eq!(i.t("no.such.key"), "no.such.key");
    }

    #[test]
    fn plain_string_applies_to_both_langs() {
        let i = i18n();
        assert_eq!(i.t("plain.key"), "Plain");
        assert_eq!(i.t_lang("plain.key", Lang::En), "Plain");
    }

    #[test]
    fn bi_returns_both() {
        let i = i18n();
        let b = i.bi("calc.button");
        assert_eq!(b.sw, "Hesabu");
        assert_eq!(b.en, "Calculate");
        let missing = i.bi("haipo");
        assert_eq!(missing.sw, "haipo");
        assert_eq!(missing.en, "haipo");
    }

    #[test]
    fn lang_from_code() {
        assert_eq!(Lang::from_code("en"), Lang::En);
        assert_eq!(Lang::from_code("EN-us"), Lang::En);
        assert_eq!(Lang::from_code("sw"), Lang::Sw);
        assert_eq!(Lang::from_code("xx"), Lang::Sw);
        assert_eq!(Lang::En.code(), "en");
    }

    #[test]
    fn bad_json_errors_swahili() {
        let mut i = I18n::new();
        let err = i.load_json("{oops").unwrap_err();
        assert!(err.contains("locale JSON si sahihi"), "{}", err);
    }
}

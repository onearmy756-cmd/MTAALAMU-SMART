//! MTAALAMU SMART — Math & Inference Engine (Rust)
//!
//! KANUNI: LLM haihesabu kamwe. Kila hesabu hapa ni deterministic,
//! inarudisha formula, hatua (steps), na hali (GOOD/WARNING/FAIL).
//! JSON ndio format pekee ya interop kati ya Rust, R na React.
//!
//! Location & Navigation: geo hierarchy + turn-by-turn + voice Kiswahili + hazards.

pub mod expr;
pub mod formula_engine;
pub mod bayes;
pub mod decision_tree;
pub mod rules;
pub mod knowledge;
pub mod i18n;
pub mod geo;
pub mod navigation;
pub mod hazards;

pub use expr::{eval, eval_with, EvalError, Vars};
pub use formula_engine::{FormulaEngine, CalcResult, Formula};
pub use bayes::{BayesianDiagnoser, DiagnoseResult, Posterior};
pub use decision_tree::{DecisionTree, WalkResult};
pub use rules::{RulesEngine, RuleMatch};
pub use knowledge::{KnowledgeBase, DiagnosisHit};
pub use i18n::{I18n, Lang};
pub use geo::{GeoEngine, GeoPlace};
pub use navigation::{NavigationEngine, NavInstruction};
pub use hazards::{HazardsEngine, HazardType};

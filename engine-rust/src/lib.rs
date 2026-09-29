//! MTAALAMU SMART — Math & Inference Engine (Rust)
//! Agentic Vision: Multi-Agent + Live Vision + PIITVD + Report + OS Probe

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
pub mod agents;
pub mod vision;
pub mod pipeline;
pub mod report;
pub mod agentic_run;
pub mod sysprobe;

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
pub use agents::{AgentOrchestrator, AgentSession, AgentEvent, SessionState};
pub use vision::{VisionEngine, VisionSnapshot};
pub use pipeline::{PipelineEngine, PipelineRuntime};
pub use report::{ReportEngine, DigitalReport};
pub use agentic_run::{run_full, AgenticResult};
pub use sysprobe::{probe, probe_json, SystemProbe};

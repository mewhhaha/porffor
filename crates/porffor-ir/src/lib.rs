use porffor_front::{ParseGoal, SourceUnit};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoweringStage {
    ParsedSource,
    EarlyErrorsPending,
    SpecIrPending,
    BackendIrPending,
    WasmCodegenPending,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramIr {
    pub goal: ParseGoal,
    pub stages: Vec<LoweringStage>,
    pub source_len: usize,
    pub invariants: Vec<&'static str>,
}

pub fn lower(source: &SourceUnit) -> ProgramIr {
    ProgramIr {
        goal: source.goal,
        stages: vec![
            LoweringStage::ParsedSource,
            LoweringStage::EarlyErrorsPending,
            LoweringStage::SpecIrPending,
            LoweringStage::BackendIrPending,
            LoweringStage::WasmCodegenPending,
        ],
        source_len: source.source_text.len(),
        invariants: vec![
            "direct-js-to-wasm-only",
            "no-shipped-interpreter-in-wasm",
            "spec-ir-is-semantic-source-of-truth",
        ],
    }
}

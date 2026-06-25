#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoweringStage {
    ParsedSource,
    AstReparsed,
    ScriptIrBuilt,
    UnsupportedFeaturesRecorded,
    WasmReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrDiagnosticKind {
    Unsupported,
    Lowering,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDiagnostic {
    pub kind: IrDiagnosticKind,
    pub message: String,
}

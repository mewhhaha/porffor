use porffor_front::SourceSpan;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoweringStage {
    ParsedSource,
    AstReparsed,
    /// Every module of the graph was parsed into a Source Text Module Record.
    ModuleGraphLoaded,
    /// Every import entry was resolved and evaluation order was computed.
    ModuleGraphLinked,
    ScriptIrBuilt,
    UnsupportedFeaturesRecorded,
    WasmReady,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrDiagnosticKind {
    EarlyError,
    /// A module graph failed to link: unresolved specifier, missing export,
    /// ambiguous export, duplicate export name.
    LinkError,
    Unsupported,
    Lowering,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrDiagnosticPhase {
    Early,
    /// Module linking. test262 spells this phase `resolution`; an AOT compiler
    /// catches it at compile time rather than throwing at runtime.
    Resolution,
    Lowering,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDiagnostic {
    pub kind: IrDiagnosticKind,
    pub phase: IrDiagnosticPhase,
    pub code: Option<&'static str>,
    pub error_type: Option<&'static str>,
    pub span: Option<SourceSpan>,
    pub message: String,
}

impl IrDiagnostic {
    pub fn early_error(
        code: &'static str,
        error_type: &'static str,
        message: impl Into<String>,
        span: Option<SourceSpan>,
    ) -> Self {
        Self {
            kind: IrDiagnosticKind::EarlyError,
            phase: IrDiagnosticPhase::Early,
            code: Some(code),
            error_type: Some(error_type),
            span,
            message: message.into(),
        }
    }

    /// A module-linking failure. Always a `SyntaxError`, always compile time.
    pub fn link_error(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            kind: IrDiagnosticKind::LinkError,
            phase: IrDiagnosticPhase::Resolution,
            code: Some(code),
            error_type: Some("SyntaxError"),
            span: None,
            message: message.into(),
        }
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self {
            kind: IrDiagnosticKind::Unsupported,
            phase: IrDiagnosticPhase::Lowering,
            code: None,
            error_type: None,
            span: None,
            message: message.into(),
        }
    }

    pub fn lowering(message: impl Into<String>) -> Self {
        Self {
            kind: IrDiagnosticKind::Lowering,
            phase: IrDiagnosticPhase::Lowering,
            code: None,
            error_type: None,
            span: None,
            message: message.into(),
        }
    }
}

use porffor_front::SourceSpan;

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
    EarlyError,
    Unsupported,
    Lowering,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IrDiagnosticPhase {
    Early,
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

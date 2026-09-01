use crate::early_error_code::rejection_kind;
use crate::{DynamicSourceGap, NativeErrorKind};
use lila_front::{EarlyErrorCode, ParseClassified, SourceSpan};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoweringStage {
    ParsedSource,
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

/// A compiler capability gap with a closed, consumer-visible identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnsupportedFeature {
    DynamicSource(DynamicSourceGap),
}

impl IrDiagnosticKind {
    /// When this rejection was decided.
    ///
    /// 16.1.4 `ParseScript` and 16.2.1.6.1 `ParseModule` fix the reporting phase
    /// per producing operation, so the phase is a function of the stage that
    /// produced the diagnostic and never a free choice at a call site. It used
    /// to be a field beside `kind`, which is what let one condition
    /// (`E_MODULE_DUPLICATE_EXPORT`) be reported as `Early` by one producer and
    /// `Resolution` by another.
    #[must_use]
    pub const fn phase(self) -> IrDiagnosticPhase {
        match self {
            Self::EarlyError => IrDiagnosticPhase::Early,
            Self::LinkError => IrDiagnosticPhase::Resolution,
            Self::Unsupported | Self::Lowering => IrDiagnosticPhase::Lowering,
        }
    }

    /// The error the program would have thrown, if the spec says it throws one.
    ///
    /// `ParseScript` and `ParseModule` return "a List of **SyntaxError**
    /// objects"; `InitializeEnvironment` throws a **SyntaxError**. Measured on
    /// the pinned suite: there is no `parse/` or `resolution/` negative of any
    /// other type, in either direction. `Unsupported` and `Lowering` are
    /// compiler gaps and must **not** claim a spec error type — `None` here is
    /// the difference between "ECMAScript rejects this program" and "this
    /// compiler could not compile it".
    #[must_use]
    pub const fn error_type(self) -> Option<NativeErrorKind> {
        match self {
            Self::EarlyError | Self::LinkError => Some(NativeErrorKind::SyntaxError),
            Self::Unsupported | Self::Lowering => None,
        }
    }
}

/// One compile-time rejection or gap report.
///
/// The private payload is the only stored classification authority. A coded
/// rejection, an unclassified compiler gap, a typed capability gap, and a
/// lowering failure are distinct variants, so no constructor or later mutation
/// can pair a code or feature with the wrong kind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDiagnostic {
    payload: IrDiagnosticPayload,
    pub span: Option<SourceSpan>,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum IrDiagnosticPayload {
    Rejected(EarlyErrorCode),
    Unsupported,
    UnsupportedFeature(UnsupportedFeature),
    Lowering,
}

impl IrDiagnostic {
    /// The **only** constructor that can produce a coded diagnostic: this
    /// program is rejected before any construct evaluates, under this condition.
    ///
    /// Replaces the old `early_error` / `link_error` pair, whose separation was
    /// precisely the opportunity for the two to disagree about one condition —
    /// and they did, for `E_MODULE_DUPLICATE_EXPORT`. Which of the two stages
    /// rejects a given code is now
    /// [`rejection_kind`](crate::early_error_code::rejection_kind)'s single
    /// exhaustive match, and the parse table's half of that agreement is
    /// assertion P7.
    pub fn rejected(
        code: EarlyErrorCode,
        message: impl Into<String>,
        span: Option<SourceSpan>,
    ) -> Self {
        Self {
            payload: IrDiagnosticPayload::Rejected(code),
            span,
            message: message.into(),
        }
    }

    /// [`Self::rejected`] for a **parse-stage** producer.
    ///
    /// Same derivation, narrower door: the code must be a
    /// [`ParseClassified`] — one the message-pattern table can actually yield — so a
    /// producer that runs while parsing cannot name a link-only condition such
    /// as `ModuleMissingExport` and have it reported at
    /// `IrDiagnosticPhase::Resolution` from a `ParseModule` stage. Assertion P7
    /// makes the *table* agree with `rejection_kind`; this makes the *call
    /// sites* agree with it, which P7 cannot see.
    pub fn rejected_at_parse(
        code: ParseClassified,
        message: impl Into<String>,
        span: Option<SourceSpan>,
    ) -> Self {
        Self::rejected(code.code(), message, span)
    }

    pub fn unsupported(message: impl Into<String>) -> Self {
        Self {
            payload: IrDiagnosticPayload::Unsupported,
            span: None,
            message: message.into(),
        }
    }

    /// Records dynamic-source debt without requiring consumers to parse the
    /// human-readable diagnostic text.
    pub fn unsupported_dynamic_source(gap: DynamicSourceGap) -> Self {
        Self {
            payload: IrDiagnosticPayload::UnsupportedFeature(UnsupportedFeature::DynamicSource(
                gap,
            )),
            span: None,
            message: format!(
                "unsupported in lila wasm-aot first slice: feature `{}` ({}) requires {}",
                gap.kind().feature_label(),
                gap.kind().operation_name(),
                gap.requirement().description(),
            ),
        }
    }

    pub fn lowering(message: impl Into<String>) -> Self {
        Self {
            payload: IrDiagnosticPayload::Lowering,
            span: None,
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> IrDiagnosticKind {
        match &self.payload {
            IrDiagnosticPayload::Rejected(code) => rejection_kind(*code),
            IrDiagnosticPayload::Unsupported | IrDiagnosticPayload::UnsupportedFeature(_) => {
                IrDiagnosticKind::Unsupported
            }
            IrDiagnosticPayload::Lowering => IrDiagnosticKind::Lowering,
        }
    }

    /// The condition this diagnostic reports, or `None` for a compiler gap.
    ///
    /// `code().is_some()` is exactly "this is a spec rejection", which is the
    /// predicate `lila-test262` needs and the one that survives folding the
    /// code into `IrDiagnosticKind` later.
    #[must_use]
    pub const fn code(&self) -> Option<EarlyErrorCode> {
        match &self.payload {
            IrDiagnosticPayload::Rejected(code) => Some(*code),
            IrDiagnosticPayload::Unsupported
            | IrDiagnosticPayload::UnsupportedFeature(_)
            | IrDiagnosticPayload::Lowering => None,
        }
    }

    /// The closed compiler capability, when this unsupported diagnostic has
    /// migrated away from string-based accounting.
    #[must_use]
    pub const fn unsupported_feature(&self) -> Option<UnsupportedFeature> {
        match &self.payload {
            IrDiagnosticPayload::UnsupportedFeature(feature) => Some(*feature),
            IrDiagnosticPayload::Rejected(_)
            | IrDiagnosticPayload::Unsupported
            | IrDiagnosticPayload::Lowering => None,
        }
    }

    #[must_use]
    pub const fn phase(&self) -> IrDiagnosticPhase {
        self.kind().phase()
    }

    #[must_use]
    pub const fn error_type(&self) -> Option<NativeErrorKind> {
        self.kind().error_type()
    }
}

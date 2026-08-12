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
/// `phase` and `error_type` are **not** fields; they are functions of `kind`,
/// and `kind` is a function of `code`. What used to be a four-field product held
/// consistent by there happening to be exactly four constructors is now one
/// stored discriminant plus payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IrDiagnostic {
    pub kind: IrDiagnosticKind,
    /// **Private, and that is the invariant.** The only writer is
    /// [`IrDiagnostic::rejected`], which derives `kind` from this value. A
    /// struct literal anywhere else in the workspace is `error[E0451]` (private
    /// field `code`), so a fifth constructor cannot be written except beside the
    /// other four, in this file, where the derivation lives.
    ///
    /// `None` means "not a spec rejection": an `Unsupported` or `Lowering`
    /// report. It is deliberately not an `EarlyErrorCode` variant — a code that
    /// named the absence of a code would let `code: Some(_)` mean "no code".
    code: Option<EarlyErrorCode>,
    unsupported_feature: Option<UnsupportedFeature>,
    pub span: Option<SourceSpan>,
    pub message: String,
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
            kind: rejection_kind(code),
            code: Some(code),
            unsupported_feature: None,
            span,
            message: message.into(),
        }
    }

    /// [`Self::rejected`] for a **parse-stage** producer.
    ///
    /// Same derivation, narrower door: the code must be a
    /// [`ParseClassified`] — one the fragment table can actually yield — so a
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
            kind: IrDiagnosticKind::Unsupported,
            code: None,
            unsupported_feature: None,
            span: None,
            message: message.into(),
        }
    }

    /// Records dynamic-source debt without requiring consumers to parse the
    /// human-readable diagnostic text.
    pub fn unsupported_dynamic_source(gap: DynamicSourceGap) -> Self {
        Self {
            kind: IrDiagnosticKind::Unsupported,
            code: None,
            unsupported_feature: Some(UnsupportedFeature::DynamicSource(gap)),
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
            kind: IrDiagnosticKind::Lowering,
            code: None,
            unsupported_feature: None,
            span: None,
            message: message.into(),
        }
    }

    /// The condition this diagnostic reports, or `None` for a compiler gap.
    ///
    /// `code().is_some()` is exactly "this is a spec rejection", which is the
    /// predicate `lila-test262` needs and the one that survives folding the
    /// code into `IrDiagnosticKind` later.
    #[must_use]
    pub const fn code(&self) -> Option<EarlyErrorCode> {
        self.code
    }

    /// The closed compiler capability, when this unsupported diagnostic has
    /// migrated away from string-based accounting.
    #[must_use]
    pub const fn unsupported_feature(&self) -> Option<UnsupportedFeature> {
        self.unsupported_feature
    }

    #[must_use]
    pub const fn phase(&self) -> IrDiagnosticPhase {
        self.kind.phase()
    }

    #[must_use]
    pub const fn error_type(&self) -> Option<NativeErrorKind> {
        self.kind.error_type()
    }
}

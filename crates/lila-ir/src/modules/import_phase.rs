use boa_ast::declaration::ImportPhase;

/// Phase of a module request (`import`, `import defer`, `import source`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub enum ImportPhaseIr {
    /// A normal eager request.
    #[default]
    Evaluation,
    /// `import defer * as ns from "m"`.
    Defer,
    /// `import source x from "m"`.
    Source,
}

impl ImportPhaseIr {
    /// Name used in diagnostics.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Evaluation => "evaluation",
            Self::Defer => "defer",
            Self::Source => "source",
        }
    }

    pub(super) const fn from_ast(phase: ImportPhase) -> Self {
        match phase {
            ImportPhase::Evaluation => Self::Evaluation,
            ImportPhase::Defer => Self::Defer,
            ImportPhase::Source => Self::Source,
        }
    }
}

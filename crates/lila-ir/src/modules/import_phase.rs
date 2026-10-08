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
    /// Whether loading this occurrence opens the target's dependencies.
    /// Source phase loads/parses one record. Evaluation and Defer recurse;
    /// a new phase must explicitly choose its actual loading behavior.
    #[must_use]
    pub const fn loads_dependencies(self) -> bool {
        match self {
            Self::Evaluation | Self::Defer => true,
            Self::Source => false,
        }
    }

    pub(crate) const fn namespace_mode(self) -> Option<crate::ModuleNamespaceModeIr> {
        match self {
            Self::Evaluation => Some(crate::ModuleNamespaceModeIr::Eager),
            Self::Defer => Some(crate::ModuleNamespaceModeIr::Deferred),
            Self::Source => None,
        }
    }

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

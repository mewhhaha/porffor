/// When, if ever, a unit's body runs in the merged script.
///
/// Fixed by
/// [`classify_evaluation_modes`](super::graph_evaluation_classification::classify_evaluation_modes)
/// from the *phases* of the requests that reach a unit, and consumed by
/// `modules::link` (which body text to emit) and `modules::namespace` (which
/// object to build).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ModuleEvaluationModeIr {
    /// The body is emitted inline, in evaluation order. The default, and what
    /// every module of an unphased graph gets.
    #[default]
    Eager,
    /// `import defer`: the body is emitted as a thunk that the module's
    /// namespace object calls on the first read of any export.
    Deferred,
    /// `import source`: the module is loaded, parsed and linked, but its body
    /// is never emitted. Only a module source object is handed out.
    NotEvaluated,
}

/// How a linked unit participates in runtime source generation.
///
/// `NotEvaluated` deliberately has no inhabitant here: a source-phase-only
/// unit stays in the loaded and linked graph, but no runtime collector may
/// receive it. Keeping this type private prevents callers from manufacturing a
/// namespace or dispatcher for a unit whose body is absent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ModuleMaterializationModeIr {
    /// The unit's body is emitted inline.
    Eager,
    /// The unit's body is emitted as a deferred thunk.
    Deferred,
}

impl ModuleEvaluationModeIr {
    /// Diagnostic spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Eager => "eager",
            Self::Deferred => "deferred",
            Self::NotEvaluated => "not evaluated",
        }
    }

    /// Runtime source-generation participation for this evaluation mode.
    ///
    /// This is the single exhaustive crossing from graph classification into
    /// artifact materialization. A new evaluation mode must decide here
    /// whether it contributes runtime state instead of inheriting a boolean
    /// default at one of the collectors.
    #[must_use]
    pub(super) const fn materialization(self) -> Option<ModuleMaterializationModeIr> {
        match self {
            Self::Eager => Some(ModuleMaterializationModeIr::Eager),
            Self::Deferred => Some(ModuleMaterializationModeIr::Deferred),
            Self::NotEvaluated => None,
        }
    }
}

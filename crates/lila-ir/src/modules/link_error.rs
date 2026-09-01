use crate::{EarlyErrorCode, ExportName, IrDiagnostic, MAX_LINKABLE_MODULE_UNIT_ID};

use super::import_phase::ImportPhaseIr;
use super::module_key::ModuleKey;
use super::record::{ModuleRequestIr, ModuleRequestKeyIr, ModuleUnitId};

/// A linking failure. Every variant is a `SyntaxError` reported at compile
/// time, which is what test262's `phase: resolution` negatives expect — with
/// one exception this enum does not get to make: `DuplicateExport` names a
/// 16.2.3.1 *early* error that happens to have a producer here too, and
/// `rejection_kind` reports it at `phase: parse` from both producers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ModuleLinkErrorIr {
    /// The host could not resolve a request to a module.
    UnresolvedModule {
        /// Module that made the request.
        referrer: ModuleUnitId,
        /// The unresolved request.
        request: ModuleRequestKeyIr,
    },
    /// The requested module does not export the imported name.
    MissingExport {
        /// Module that made the request.
        referrer: ModuleUnitId,
        /// The request.
        request: ModuleRequestIr,
        /// Name that could not be resolved.
        ///
        /// An `[[ImportName]]`, which is the requested module's
        /// `[[ExportName]]` read from this side — the same domain, so the same
        /// type. Filling this from a `[[LocalName]]` is `E0308`.
        import_name: ExportName,
    },
    /// Two `export *` paths reached different bindings for one name.
    AmbiguousExport {
        /// Module whose export is ambiguous.
        module: ModuleUnitId,
        /// The ambiguous export name.
        export_name: ExportName,
    },
    /// The same `[[ExportName]]` is declared twice (16.2.3.1, an early error).
    DuplicateExport {
        /// Module declaring the duplicate.
        module: ModuleUnitId,
        /// The duplicated export name.
        export_name: ExportName,
    },
    /// One key was loaded twice with different source text.
    InconsistentLoad {
        /// The key loaded inconsistently.
        key: ModuleKey,
    },
    /// Public host rows resolved one phase-free request key to two targets.
    InconsistentResolution {
        /// Module that made the request.
        referrer: ModuleUnitId,
        /// Request whose host resolution contradicted itself.
        request: ModuleRequestKeyIr,
    },
    /// The closure holds more units than the source-text linker can name.
    ///
    /// Unit ids are spelled into two in-place rewrites whose replacements must
    /// not change a unit's byte length, which caps the decimal width of an id at
    /// four digits — see [`MAX_LINKABLE_MODULE_UNIT_ID`]. This is the runtime
    /// half of budgets B1/B2; const assertions V2 and V4 carry the format half.
    TooManyUnits {
        /// Number of module sources the host handed over.
        count: usize,
    },
    /// A phased request this stage cannot link, with the reason.
    ///
    /// `import defer` and `import source` link (see
    /// [`crate::ModuleEvaluationModeIr`]); what remains here are the shapes the
    /// source-text linker cannot express, chiefly a deferred module whose body
    /// would have to suspend.
    UnsupportedPhase {
        /// Module making the request.
        module: ModuleUnitId,
        /// The unsupported phase.
        phase: ImportPhaseIr,
        /// Why this particular request could not be linked.
        reason: String,
    },
}

impl ModuleLinkErrorIr {
    /// The condition this failure reports, in the one closed domain.
    ///
    /// `DuplicateExport` is deliberately not special-cased here: it names the
    /// same 16.2.3.1 condition `modules::early` names, and which *stage* that
    /// condition is reported at is `rejection_kind`'s decision, not this
    /// enum's. That is what makes it impossible for the two producers to
    /// disagree about its phase — they no longer each choose one.
    #[must_use]
    pub const fn code(&self) -> EarlyErrorCode {
        match self {
            Self::UnresolvedModule { .. } => EarlyErrorCode::ModuleUnresolved,
            Self::MissingExport { .. } => EarlyErrorCode::ModuleMissingExport,
            Self::AmbiguousExport { .. } => EarlyErrorCode::ModuleAmbiguousExport,
            Self::DuplicateExport { .. } => EarlyErrorCode::ModuleDuplicateExport,
            Self::InconsistentLoad { .. } => EarlyErrorCode::ModuleInconsistentLoad,
            Self::InconsistentResolution { .. } => EarlyErrorCode::ModuleInconsistentLoad,
            Self::UnsupportedPhase { .. } => EarlyErrorCode::ModuleUnsupportedPhase,
            Self::TooManyUnits { .. } => EarlyErrorCode::ModuleTooManyUnits,
        }
    }

    /// Human-readable message.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            Self::UnresolvedModule { request, .. } => {
                format!("unresolved module request: {}", request.specifier())
            }
            Self::MissingExport {
                request,
                import_name,
                ..
            } => format!(
                "module {} does not export {}",
                request.specifier(),
                import_name.as_str()
            ),
            Self::AmbiguousExport { export_name, .. } => {
                format!("ambiguous export name: {}", export_name.as_str())
            }
            Self::DuplicateExport { export_name, .. } => {
                format!("duplicate export name: {}", export_name.as_str())
            }
            Self::InconsistentLoad { key } => {
                format!("module loaded inconsistently: {}", key.as_str())
            }
            Self::InconsistentResolution { request, .. } => format!(
                "module request resolved inconsistently: {}",
                request.specifier()
            ),
            Self::UnsupportedPhase { phase, reason, .. } => format!(
                "unsupported in lila wasm-aot: {} phase module request: {reason}",
                phase.as_str()
            ),
            Self::TooManyUnits { count } => format!(
                "unsupported in lila wasm-aot: module graph has {count} units; the source-text \
                 linker can name unit ids up to {MAX_LINKABLE_MODULE_UNIT_ID}"
            ),
        }
    }

    /// The diagnostic this error becomes on `ProgramIr`.
    ///
    /// The kind and phase are not chosen here. `IrDiagnostic::rejected` derives
    /// them from the code, so `DuplicateExport` lands on `EarlyError`/`Early`
    /// — 16.2.3.1 makes it an early error and
    /// `test/language/module-code/early-dup-export-id.js` is `phase: parse` —
    /// while the genuine link conditions land on `LinkError`/`Resolution`.
    #[must_use]
    pub fn to_diagnostic(&self) -> IrDiagnostic {
        IrDiagnostic::rejected(self.code(), self.message(), None)
    }
}

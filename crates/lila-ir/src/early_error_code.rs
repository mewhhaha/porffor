//! The `EarlyErrorCode` → rejection-stage map, and the assertions that tie the
//! parse table in `lila-front` to it.
//!
//! The domain itself, the fragment table and the classifier live one crate down,
//! in `lila_front::early_error_code`, because every entry and dependency parse
//! is produced there and `lila-ir` depends on `lila-front`. This
//! module holds only what needs a `lila-ir` type. There is no second copy of
//! any table here; there used to be, and it had drifted.
//!
//! See `docs/rust-rewrite/contracts/early-error-taxonomy.md`.
//!
//! Note the neighbouring module `early_errors.rs` is unrelated despite the name:
//! it is derived-constructor validation over `ExprIr` arms. This module is the
//! diagnostic taxonomy.

use crate::{IrDiagnosticKind, IrDiagnosticPhase};

pub use lila_front::{EarlyErrorCode, ParseClassified};

/// Which stage rejects a program carrying this code.
///
/// The single `EarlyErrorCode → IrDiagnosticKind` map, exhaustive with no
/// catch-all: a new code is `error[E0004]` here, which is the point.
/// Phase and error type are then functions of the *kind*, so one code fixes all
/// three and nothing can be paired inconsistently.
///
/// `ModuleDuplicateExport` is an `EarlyError` because 16.2.3.1 makes it one and
/// `test/language/module-code/early-dup-export-id.js` is `phase: parse` — even
/// though `modules::graph` also raises it from the link stage. That producer now
/// gets `Early` too, and assertion P7 makes any other answer fail to build.
pub(crate) const fn rejection_kind(code: EarlyErrorCode) -> IrDiagnosticKind {
    match code {
        EarlyErrorCode::ObjectDuplicateProto
        | EarlyErrorCode::ObjectLiteralCoverInitializedName
        | EarlyErrorCode::DuplicateLexicalDeclaration
        | EarlyErrorCode::DuplicateFormalParameter
        | EarlyErrorCode::DuplicateCatchParameter
        | EarlyErrorCode::CatchBodyDeclarationConflict
        | EarlyErrorCode::DuplicateClassConstructor
        | EarlyErrorCode::ClassConstructorGeneratorMethod
        | EarlyErrorCode::ClassConstructorAsyncMethod
        | EarlyErrorCode::ClassConstructorGetter
        | EarlyErrorCode::ClassConstructorSetter
        | EarlyErrorCode::ClassPrivateConstructorName
        | EarlyErrorCode::ClassStaticMethodPrototypeName
        | EarlyErrorCode::ClassDuplicatePrivateName
        | EarlyErrorCode::ClassStaticBlockContainsArguments
        | EarlyErrorCode::ClassStaticBlockContainsAwait
        | EarlyErrorCode::ClassFieldConstructorName
        | EarlyErrorCode::ClassStaticFieldConstructorOrPrototypeName
        | EarlyErrorCode::ClassFieldContainsArguments
        | EarlyErrorCode::StrictModeWithStatement
        | EarlyErrorCode::DuplicateLabel
        | EarlyErrorCode::UndefinedBreakTarget
        | EarlyErrorCode::UndefinedContinueTarget
        | EarlyErrorCode::IllegalBreak
        | EarlyErrorCode::IllegalContinue
        | EarlyErrorCode::InvalidPrivateIdentifier
        | EarlyErrorCode::ScriptTopLevelNewTarget
        | EarlyErrorCode::ScriptTopLevelUsingDeclaration
        | EarlyErrorCode::ForInUsingDeclaration
        | EarlyErrorCode::SwitchClauseUsingDeclaration
        | EarlyErrorCode::GeneratorDeclarationParametersContainYield
        | EarlyErrorCode::GeneratorExpressionParametersContainYield
        | EarlyErrorCode::AsyncGeneratorExpressionParametersContainYield
        | EarlyErrorCode::ModuleDuplicateExport
        | EarlyErrorCode::ModuleUndeclaredExport
        | EarlyErrorCode::ModuleTopLevelSuper
        | EarlyErrorCode::ModuleTopLevelNewTarget => IrDiagnosticKind::EarlyError,

        EarlyErrorCode::ModuleUnresolved
        | EarlyErrorCode::ModuleMissingExport
        | EarlyErrorCode::ModuleAmbiguousExport
        | EarlyErrorCode::ModuleInconsistentLoad
        | EarlyErrorCode::ModuleUnsupportedPhase
        | EarlyErrorCode::ModuleTooManyUnits => IrDiagnosticKind::LinkError,
    }
}

// ---------------------------------------------------------------------------
// Const assertions P7-P9.
// ---------------------------------------------------------------------------

const fn is_early_error(kind: IrDiagnosticKind) -> bool {
    match kind {
        IrDiagnosticKind::EarlyError => true,
        IrDiagnosticKind::LinkError
        | IrDiagnosticKind::Unsupported
        | IrDiagnosticKind::Lowering => false,
    }
}

const fn is_rejection(kind: IrDiagnosticKind) -> bool {
    match kind {
        IrDiagnosticKind::EarlyError | IrDiagnosticKind::LinkError => true,
        IrDiagnosticKind::Unsupported | IrDiagnosticKind::Lowering => false,
    }
}

const fn is_lowering_phase(phase: IrDiagnosticPhase) -> bool {
    match phase {
        IrDiagnosticPhase::Lowering => true,
        IrDiagnosticPhase::Early | IrDiagnosticPhase::Resolution => false,
    }
}

/// P7: every code the *parse* table can produce is an `EarlyError`.
///
/// This is the one join left after the two tables became one, and it is the
/// structural replacement for the doc comment that used to ask, in words, that
/// the module-goal fragments in `lila-front` stay byte-identical to the ones
/// in `lila-ir`. It catches both halves of the mistake: a boa parse failure
/// classified as a link error (which would report `phase: resolution` for a
/// `phase: parse` negative), and a link-only code wired into the parse table.
const fn parse_classified_codes_are_early_errors() -> bool {
    let all = EarlyErrorCode::ALL;
    let mut i = 0;
    while i < all.len() {
        if all[i].is_parse_classified() && !is_early_error(rejection_kind(all[i])) {
            return false;
        }
        i += 1;
    }
    true
}

/// P8: every code names a rejection, never a compiler gap.
///
/// A code mapped to `Unsupported` or `Lowering` would get `error_type() == None`
/// and become invisible to `compile_negative_error_matches`, silently failing
/// every test262 case that expects it.
const fn every_code_is_a_rejection() -> bool {
    let all = EarlyErrorCode::ALL;
    let mut i = 0;
    while i < all.len() {
        if !is_rejection(rejection_kind(all[i])) {
            return false;
        }
        i += 1;
    }
    true
}

/// P9: the same property as P8, checked on the *derived* side.
///
/// P8 constrains the mapping; this constrains what the mapping's answer yields
/// once `phase()` and `error_type()` have been applied, so a future edit to
/// either derivation cannot quietly give a coded diagnostic a lowering phase or
/// no error type.
const fn every_code_derives_a_reportable_pair() -> bool {
    let all = EarlyErrorCode::ALL;
    let mut i = 0;
    while i < all.len() {
        let kind = rejection_kind(all[i]);
        if is_lowering_phase(kind.phase()) || kind.error_type().is_none() {
            return false;
        }
        i += 1;
    }
    true
}

const _: () = assert!(
    parse_classified_codes_are_early_errors(),
    "P7: a code reachable from PARSE_FAILURE_RULES does not map to IrDiagnosticKind::EarlyError, so one condition would report under two phases depending on which parse path found it"
);
const _: () = assert!(
    every_code_is_a_rejection(),
    "P8: an EarlyErrorCode maps to Unsupported/Lowering, which would give it no error_type and hide it from negative-expectation matching"
);
const _: () = assert!(
    every_code_derives_a_reportable_pair(),
    "P9: an EarlyErrorCode derives a Lowering phase or a None error_type"
);

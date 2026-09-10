//! Closed capability accounting for textual dynamic source evaluation.
//!
//! See `docs/rust-rewrite/contracts/dynamic-source-capability.md`.

use crate::names::{
    BUILTIN_FUNCTION_FUNCTION_ID, DYNAMIC_ASYNC_FUNCTION_CONSTRUCTOR_FUNCTION_ID,
    DYNAMIC_ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_FUNCTION_ID,
    DYNAMIC_GENERATOR_FUNCTION_CONSTRUCTOR_FUNCTION_ID, DYNAMIC_REALM_EVAL_SCRIPT_FUNCTION_ID,
};
use crate::FunctionExecutionKind;

/// The four constructors covered by CreateDynamicFunction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum DynamicFunctionKind {
    Ordinary,
    Generator,
    Async,
    AsyncGenerator,
}

impl DynamicFunctionKind {
    pub const ALL: &'static [Self] = &[
        Self::Ordinary,
        Self::Generator,
        Self::Async,
        Self::AsyncGenerator,
    ];

    /// The CreateDynamicFunction kind implied by a function object's
    /// execution protocol. Ordinary functions do not expose a derived
    /// constructor through this seam: their prototype chain reaches the
    /// ordinary `%Function%` identity already owned by `StandardBuiltinId`.
    #[must_use]
    pub const fn from_derived_execution_kind(kind: FunctionExecutionKind) -> Option<Self> {
        match kind {
            FunctionExecutionKind::Ordinary => None,
            FunctionExecutionKind::Generator => Some(Self::Generator),
            FunctionExecutionKind::Async => Some(Self::Async),
            FunctionExecutionKind::AsyncGenerator => Some(Self::AsyncGenerator),
        }
    }
}

/// Closed dynamic-source intrinsic identities shared by lowering and backend
/// metadata. Zero-argument Function-family calls have real compiled empty
/// bodies; textual evaluation without a specialization remains unsupported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicSourceIntrinsic {
    Function(DynamicFunctionKind),
    RealmEvalScript,
}

impl DynamicSourceIntrinsic {
    pub const ALL: &'static [Self] = &[
        Self::Function(DynamicFunctionKind::Ordinary),
        Self::Function(DynamicFunctionKind::Generator),
        Self::Function(DynamicFunctionKind::Async),
        Self::Function(DynamicFunctionKind::AsyncGenerator),
        Self::RealmEvalScript,
    ];

    #[must_use]
    pub const fn source_kind(self) -> DynamicSourceKind {
        match self {
            Self::Function(kind) => DynamicSourceKind::Function(kind),
            Self::RealmEvalScript => DynamicSourceKind::RealmEvalScript,
        }
    }

    #[must_use]
    pub const fn function_id(self) -> &'static str {
        match self {
            Self::Function(DynamicFunctionKind::Ordinary) => BUILTIN_FUNCTION_FUNCTION_ID,
            Self::Function(DynamicFunctionKind::Generator) => {
                DYNAMIC_GENERATOR_FUNCTION_CONSTRUCTOR_FUNCTION_ID
            }
            Self::Function(DynamicFunctionKind::Async) => {
                DYNAMIC_ASYNC_FUNCTION_CONSTRUCTOR_FUNCTION_ID
            }
            Self::Function(DynamicFunctionKind::AsyncGenerator) => {
                DYNAMIC_ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_FUNCTION_ID
            }
            Self::RealmEvalScript => DYNAMIC_REALM_EVAL_SCRIPT_FUNCTION_ID,
        }
    }

    #[must_use]
    pub fn from_function_id(function_id: &str) -> Option<Self> {
        match function_id {
            BUILTIN_FUNCTION_FUNCTION_ID => Some(Self::Function(DynamicFunctionKind::Ordinary)),
            DYNAMIC_GENERATOR_FUNCTION_CONSTRUCTOR_FUNCTION_ID => {
                Some(Self::Function(DynamicFunctionKind::Generator))
            }
            DYNAMIC_ASYNC_FUNCTION_CONSTRUCTOR_FUNCTION_ID => {
                Some(Self::Function(DynamicFunctionKind::Async))
            }
            DYNAMIC_ASYNC_GENERATOR_FUNCTION_CONSTRUCTOR_FUNCTION_ID => {
                Some(Self::Function(DynamicFunctionKind::AsyncGenerator))
            }
            DYNAMIC_REALM_EVAL_SCRIPT_FUNCTION_ID => Some(Self::RealmEvalScript),
            _ => None,
        }
    }

    #[must_use]
    pub const fn constructable(self) -> bool {
        match self {
            Self::Function(
                DynamicFunctionKind::Ordinary
                | DynamicFunctionKind::Generator
                | DynamicFunctionKind::Async
                | DynamicFunctionKind::AsyncGenerator,
            ) => true,
            Self::RealmEvalScript => false,
        }
    }
}

/// A semantic source-evaluation operation, independent of how it was spelled.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicSourceKind {
    DirectEval,
    IndirectEval,
    RealmEvalScript,
    Function(DynamicFunctionKind),
}

impl DynamicSourceKind {
    /// Stable family label used by conformance accounting.
    #[must_use]
    pub const fn feature_label(self) -> &'static str {
        match self {
            Self::DirectEval | Self::IndirectEval => "eval dynamic source evaluation",
            Self::RealmEvalScript => "$262.evalScript dynamic source evaluation",
            Self::Function(
                DynamicFunctionKind::Ordinary
                | DynamicFunctionKind::Generator
                | DynamicFunctionKind::Async
                | DynamicFunctionKind::AsyncGenerator,
            ) => "Function constructor dynamic code generation",
        }
    }

    #[must_use]
    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::DirectEval => "direct eval",
            Self::IndirectEval => "indirect eval",
            Self::RealmEvalScript => "$262.evalScript",
            Self::Function(DynamicFunctionKind::Ordinary) => "Function",
            Self::Function(DynamicFunctionKind::Generator) => "GeneratorFunction",
            Self::Function(DynamicFunctionKind::Async) => "AsyncFunction",
            Self::Function(DynamicFunctionKind::AsyncGenerator) => "AsyncGeneratorFunction",
        }
    }
}

/// The missing compiler capability at the dynamic-source boundary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicSourceRequirement {
    RuntimeCompilation,
    CallerEnvironment,
    TargetRealmEnvironment,
}

impl DynamicSourceRequirement {
    #[must_use]
    pub const fn description(self) -> &'static str {
        match self {
            Self::RuntimeCompilation => "source compilation after AOT",
            Self::CallerEnvironment => "a caller-environment lowering seam",
            Self::TargetRealmEnvironment => "a target-realm environment lowering seam",
        }
    }
}

/// One unsupported dynamic-source operation and its derived requirement.
///
/// The fields are private so callers cannot pair Function construction with a
/// direct-eval caller environment, or otherwise manufacture a false reason.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DynamicSourceGap {
    kind: DynamicSourceKind,
    requirement: DynamicSourceRequirement,
}

impl DynamicSourceGap {
    /// Source is not proven until execution and would require a runtime parser.
    #[must_use]
    pub const fn runtime_source(kind: DynamicSourceKind) -> Self {
        Self {
            kind,
            requirement: DynamicSourceRequirement::RuntimeCompilation,
        }
    }

    /// Source is proven during AOT compilation, but its environment is not.
    #[must_use]
    pub const fn aot_known_source(kind: DynamicSourceKind) -> Self {
        let requirement = match kind {
            DynamicSourceKind::DirectEval => DynamicSourceRequirement::CallerEnvironment,
            DynamicSourceKind::IndirectEval | DynamicSourceKind::RealmEvalScript => {
                DynamicSourceRequirement::TargetRealmEnvironment
            }
            DynamicSourceKind::Function(
                DynamicFunctionKind::Ordinary
                | DynamicFunctionKind::Generator
                | DynamicFunctionKind::Async
                | DynamicFunctionKind::AsyncGenerator,
            ) => DynamicSourceRequirement::TargetRealmEnvironment,
        };
        Self { kind, requirement }
    }

    #[must_use]
    pub const fn kind(self) -> DynamicSourceKind {
        self.kind
    }

    #[must_use]
    pub const fn requirement(self) -> DynamicSourceRequirement {
        self.requirement
    }
}

/// An intrinsic selected during execution without a compiled source/environment
/// specialization. This deliberately does not infer direct-eval syntax or
/// classify source text as runtime-generated from an erased call-site fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DynamicSourceRuntimeOperation {
    Eval,
    RealmEvalScript,
    Function(DynamicFunctionKind),
}

impl DynamicSourceRuntimeOperation {
    pub const fn abi_code(self) -> i64 {
        match self {
            Self::Eval => 0,
            Self::RealmEvalScript => 1,
            Self::Function(DynamicFunctionKind::Ordinary) => 2,
            Self::Function(DynamicFunctionKind::Generator) => 3,
            Self::Function(DynamicFunctionKind::Async) => 4,
            Self::Function(DynamicFunctionKind::AsyncGenerator) => 5,
        }
    }

    pub const fn from_abi_code(code: i64) -> Option<Self> {
        match code {
            0 => Some(Self::Eval),
            1 => Some(Self::RealmEvalScript),
            2 => Some(Self::Function(DynamicFunctionKind::Ordinary)),
            3 => Some(Self::Function(DynamicFunctionKind::Generator)),
            4 => Some(Self::Function(DynamicFunctionKind::Async)),
            5 => Some(Self::Function(DynamicFunctionKind::AsyncGenerator)),
            _ => None,
        }
    }

    pub const fn operation_name(self) -> &'static str {
        match self {
            Self::Eval => "eval",
            Self::RealmEvalScript => "$262.evalScript",
            Self::Function(DynamicFunctionKind::Ordinary) => "Function",
            Self::Function(DynamicFunctionKind::Generator) => "GeneratorFunction",
            Self::Function(DynamicFunctionKind::Async) => "AsyncFunction",
            Self::Function(DynamicFunctionKind::AsyncGenerator) => "AsyncGeneratorFunction",
        }
    }
}

impl core::fmt::Display for DynamicSourceRuntimeOperation {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "unsupported dynamic-source operation `{}` selected during Wasm execution without a compiled source/environment specialization",
            self.operation_name(),
        )
    }
}

impl std::error::Error for DynamicSourceRuntimeOperation {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_operation_abi_decodes_only_assigned_codes() {
        let operations = [
            DynamicSourceRuntimeOperation::Eval,
            DynamicSourceRuntimeOperation::RealmEvalScript,
            DynamicSourceRuntimeOperation::Function(DynamicFunctionKind::Ordinary),
            DynamicSourceRuntimeOperation::Function(DynamicFunctionKind::Generator),
            DynamicSourceRuntimeOperation::Function(DynamicFunctionKind::Async),
            DynamicSourceRuntimeOperation::Function(DynamicFunctionKind::AsyncGenerator),
        ];
        for (code, operation) in operations.into_iter().enumerate() {
            assert_eq!(operation.abi_code(), code as i64);
            assert_eq!(
                DynamicSourceRuntimeOperation::from_abi_code(code as i64),
                Some(operation)
            );
        }
        for code in [i64::MIN, -1, 6, i64::MAX] {
            assert_eq!(DynamicSourceRuntimeOperation::from_abi_code(code), None);
        }
    }

    #[test]
    fn dynamic_source_intrinsic_catalog_round_trips_every_identity() {
        for intrinsic in DynamicSourceIntrinsic::ALL.iter().copied() {
            assert_eq!(
                DynamicSourceIntrinsic::from_function_id(intrinsic.function_id()),
                Some(intrinsic),
            );
        }
    }

    #[test]
    fn execution_protocol_maps_only_to_derived_function_kinds() {
        assert_eq!(
            DynamicFunctionKind::from_derived_execution_kind(FunctionExecutionKind::Ordinary),
            None,
        );
        assert_eq!(
            DynamicFunctionKind::from_derived_execution_kind(FunctionExecutionKind::Generator),
            Some(DynamicFunctionKind::Generator),
        );
        assert_eq!(
            DynamicFunctionKind::from_derived_execution_kind(FunctionExecutionKind::Async),
            Some(DynamicFunctionKind::Async),
        );
        assert_eq!(
            DynamicFunctionKind::from_derived_execution_kind(FunctionExecutionKind::AsyncGenerator,),
            Some(DynamicFunctionKind::AsyncGenerator),
        );
    }
}

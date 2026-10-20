use crate::{ConformanceOwner, DynamicFunctionKind, DynamicSourceRuntimeOperation};

/// The weak-reference and ephemeron facility of the selected product runtime.
/// This authority is consumed by both the Wasmtime policy and weak builtins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmWeakReachabilityCapability {
    Unavailable,
    CopyingCollector,
}

pub const PRODUCT_WASM_WEAK_REACHABILITY: WasmWeakReachabilityCapability =
    WasmWeakReachabilityCapability::CopyingCollector;

impl WasmWeakReachabilityCapability {
    pub const fn report(self) -> &'static str {
        match self {
            Self::Unavailable => "weak-references=unavailable ephemerons=unavailable",
            Self::CopyingCollector => {
                "weak-references=copying-collector ephemerons=owner-key-fixed-point"
            }
        }
    }
}

/// A required semantic facility absent from the selected runtime, rather than
/// a compiler implementation gap or a JavaScript exception.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeUnavailableCapability {
    WeakReachability,
    ExternalCallerImport,
    /// A host source representation was supplied without an evaluation body.
    HostModuleEvaluation,
}

impl RuntimeUnavailableCapability {
    pub const fn owner(self) -> ConformanceOwner {
        match self {
            Self::WeakReachability => ConformanceOwner::T05,
            Self::ExternalCallerImport | Self::HostModuleEvaluation => ConformanceOwner::T13,
        }
    }
}

impl core::fmt::Display for RuntimeUnavailableCapability {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::HostModuleEvaluation => write!(
                f,
                "{}: the host ModuleSource record supplies no module evaluation capability",
                self.owner(),
            ),
            Self::ExternalCallerImport => write!(
                f,
                "{}: native direct eval cannot yet execute import() through the external caller's Script Record",
                self.owner(),
            ),
            Self::WeakReachability => write!(
                f,
                "{}: weak references and ephemerons are unavailable in the selected Wasmtime runtime; WeakMap, WeakSet, WeakRef and FinalizationRegistry require this facility",
                self.owner(),
            ),
        }
    }
}

/// Compiler semantics not yet implemented on the product Wasm path.
/// These are distinct from required Wasmtime features and unavailable host data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeSemanticGap {
    TemporalNamedTimeZone,
    RegExpRuntimePatternCompilation,
    TemporalZonedRoundingWindow,
}

impl RuntimeSemanticGap {
    pub const fn owner(self) -> ConformanceOwner {
        match self {
            Self::TemporalNamedTimeZone => ConformanceOwner::T22,
            Self::RegExpRuntimePatternCompilation => ConformanceOwner::T19,
            Self::TemporalZonedRoundingWindow => ConformanceOwner::T22,
        }
    }
}

impl core::fmt::Display for RuntimeSemanticGap {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::TemporalNamedTimeZone => write!(
                f,
                "{}: Temporal named-time-zone inverse resolution and consumers are not implemented in the Wasm compiler; pinned named-zone data is available",
                self.owner(),
            ),
            Self::RegExpRuntimePatternCompilation => write!(
                f,
                "{}: this RegExp pattern is not implemented by the Wasm compiler",
                self.owner(),
            ),
            Self::TemporalZonedRoundingWindow => write!(
                f,
                "{}: unresolved zoned calendar rounding window (upstream Temporal issue 3310)",
                self.owner(),
            ),
        }
    }
}

/// Closed domain of the mandatory out-of-band rejection import.
/// A rejection cannot be caught as a JavaScript exception or satisfy a negative test.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeSemanticRejection {
    DynamicSource(DynamicSourceRuntimeOperation),
    Gap(RuntimeSemanticGap),
    UnavailableCapability(RuntimeUnavailableCapability),
}

impl RuntimeSemanticRejection {
    pub const fn abi_code(self) -> i64 {
        match self {
            Self::DynamicSource(DynamicSourceRuntimeOperation::Eval) => 0,
            Self::DynamicSource(DynamicSourceRuntimeOperation::RealmEvalScript) => 1,
            Self::DynamicSource(DynamicSourceRuntimeOperation::Function(
                DynamicFunctionKind::Ordinary,
            )) => 2,
            Self::DynamicSource(DynamicSourceRuntimeOperation::Function(
                DynamicFunctionKind::Generator,
            )) => 3,
            Self::DynamicSource(DynamicSourceRuntimeOperation::Function(
                DynamicFunctionKind::Async,
            )) => 4,
            Self::DynamicSource(DynamicSourceRuntimeOperation::Function(
                DynamicFunctionKind::AsyncGenerator,
            )) => 5,
            Self::Gap(RuntimeSemanticGap::TemporalNamedTimeZone) => 6,
            Self::Gap(RuntimeSemanticGap::RegExpRuntimePatternCompilation) => 7,
            Self::Gap(RuntimeSemanticGap::TemporalZonedRoundingWindow) => 8,
            Self::UnavailableCapability(RuntimeUnavailableCapability::WeakReachability) => 9,
            Self::DynamicSource(DynamicSourceRuntimeOperation::ShadowRealmEvaluate) => 10,
            Self::UnavailableCapability(RuntimeUnavailableCapability::ExternalCallerImport) => 11,
            Self::UnavailableCapability(RuntimeUnavailableCapability::HostModuleEvaluation) => 12,
        }
    }

    pub const fn from_abi_code(code: i64) -> Option<Self> {
        match code {
            0 => Some(Self::DynamicSource(DynamicSourceRuntimeOperation::Eval)),
            1 => Some(Self::DynamicSource(
                DynamicSourceRuntimeOperation::RealmEvalScript,
            )),
            2 => Some(Self::DynamicSource(
                DynamicSourceRuntimeOperation::Function(DynamicFunctionKind::Ordinary),
            )),
            3 => Some(Self::DynamicSource(
                DynamicSourceRuntimeOperation::Function(DynamicFunctionKind::Generator),
            )),
            4 => Some(Self::DynamicSource(
                DynamicSourceRuntimeOperation::Function(DynamicFunctionKind::Async),
            )),
            5 => Some(Self::DynamicSource(
                DynamicSourceRuntimeOperation::Function(DynamicFunctionKind::AsyncGenerator),
            )),
            6 => Some(Self::Gap(RuntimeSemanticGap::TemporalNamedTimeZone)),
            7 => Some(Self::Gap(
                RuntimeSemanticGap::RegExpRuntimePatternCompilation,
            )),
            8 => Some(Self::Gap(RuntimeSemanticGap::TemporalZonedRoundingWindow)),
            9 => Some(Self::UnavailableCapability(
                RuntimeUnavailableCapability::WeakReachability,
            )),
            10 => Some(Self::DynamicSource(
                DynamicSourceRuntimeOperation::ShadowRealmEvaluate,
            )),
            11 => Some(Self::UnavailableCapability(
                RuntimeUnavailableCapability::ExternalCallerImport,
            )),
            12 => Some(Self::UnavailableCapability(
                RuntimeUnavailableCapability::HostModuleEvaluation,
            )),
            _ => None,
        }
    }
}

impl core::fmt::Display for RuntimeSemanticRejection {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::DynamicSource(operation) => operation.fmt(f),
            Self::Gap(gap) => gap.fmt(f),
            Self::UnavailableCapability(capability) => capability.fmt(f),
        }
    }
}

impl std::error::Error for RuntimeSemanticRejection {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_rejection_wire_domain_has_unique_closed_reasons() {
        let reasons = [
            RuntimeSemanticRejection::DynamicSource(DynamicSourceRuntimeOperation::Eval),
            RuntimeSemanticRejection::DynamicSource(DynamicSourceRuntimeOperation::RealmEvalScript),
            RuntimeSemanticRejection::DynamicSource(DynamicSourceRuntimeOperation::Function(
                DynamicFunctionKind::Ordinary,
            )),
            RuntimeSemanticRejection::DynamicSource(DynamicSourceRuntimeOperation::Function(
                DynamicFunctionKind::Generator,
            )),
            RuntimeSemanticRejection::DynamicSource(DynamicSourceRuntimeOperation::Function(
                DynamicFunctionKind::Async,
            )),
            RuntimeSemanticRejection::DynamicSource(DynamicSourceRuntimeOperation::Function(
                DynamicFunctionKind::AsyncGenerator,
            )),
            RuntimeSemanticRejection::Gap(RuntimeSemanticGap::TemporalNamedTimeZone),
            RuntimeSemanticRejection::Gap(RuntimeSemanticGap::RegExpRuntimePatternCompilation),
            RuntimeSemanticRejection::Gap(RuntimeSemanticGap::TemporalZonedRoundingWindow),
            RuntimeSemanticRejection::UnavailableCapability(
                RuntimeUnavailableCapability::WeakReachability,
            ),
            RuntimeSemanticRejection::DynamicSource(
                DynamicSourceRuntimeOperation::ShadowRealmEvaluate,
            ),
            RuntimeSemanticRejection::UnavailableCapability(
                RuntimeUnavailableCapability::ExternalCallerImport,
            ),
            RuntimeSemanticRejection::UnavailableCapability(
                RuntimeUnavailableCapability::HostModuleEvaluation,
            ),
        ];
        assert_eq!(
            RuntimeUnavailableCapability::WeakReachability.owner(),
            ConformanceOwner::T05
        );
        for (code, reason) in reasons.into_iter().enumerate() {
            assert_eq!(reason.abi_code(), code as i64);
            assert_eq!(
                RuntimeSemanticRejection::from_abi_code(code as i64),
                Some(reason)
            );
        }
        for code in [i64::MIN, -1, 13, i64::MAX] {
            assert_eq!(RuntimeSemanticRejection::from_abi_code(code), None);
        }
        assert_eq!(
            RuntimeSemanticGap::TemporalNamedTimeZone.owner(),
            ConformanceOwner::T22
        );
        assert_eq!(
            RuntimeSemanticGap::RegExpRuntimePatternCompilation.owner(),
            ConformanceOwner::T19
        );
        assert_eq!(
            RuntimeSemanticGap::TemporalZonedRoundingWindow.owner(),
            ConformanceOwner::T22
        );
    }
}

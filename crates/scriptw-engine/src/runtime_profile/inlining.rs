//! Native Wasm inlining preserves every emitted JavaScript activation operation.
use super::*;

/// Closed diagnostic cohort; ordinary Engines retain disabled native inlining.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WasmProfileInlining {
    Disabled,
    Bounded,
}

/// Pinned Wasmtime47 source facts, not observed Engine threshold getters.
/// Its supported Config API exposes only the mode; every public Config starts
/// with these same private tunable defaults and cannot customize the bounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WasmProfileInliningLimits {
    small_callee_size: u32,
    sum_size_threshold: u32,
}

pub(crate) const PINNED_WASMTIME_INLINING_LIMITS: WasmProfileInliningLimits =
    WasmProfileInliningLimits {
        small_callee_size: 50,
        sum_size_threshold: 2000,
    };

impl WasmProfileInliningLimits {
    pub const fn small_callee_size(self) -> u32 {
        self.small_callee_size
    }
    pub const fn sum_size_threshold(self) -> u32 {
        self.sum_size_threshold
    }
}

impl WasmProfileInlining {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Bounded => "bounded",
        }
    }

    pub const fn limits(self) -> Option<WasmProfileInliningLimits> {
        match self {
            Self::Disabled => None,
            Self::Bounded => Some(PINNED_WASMTIME_INLINING_LIMITS),
        }
    }

    pub(crate) fn configure(self, config: &mut WasmtimeConfig) {
        config.compiler_inlining(match self {
            Self::Disabled => wasmtime::Inlining::No,
            Self::Bounded => wasmtime::Inlining::Yes,
        });
    }

    pub(crate) fn observe(engine: &WasmtimeEngine) -> Result<Self, EngineError> {
        match engine.get_compiler_inlining() {
            wasmtime::Inlining::No => Ok(Self::Disabled),
            wasmtime::Inlining::Yes => Ok(Self::Bounded),
            other => Err(EngineError::new(format!(
                "runtime profile cannot represent observed compiler inlining {other:?}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn actual_engine_inlining_is_observed_independently_and_other_modes_are_rejected() {
        let requested = WasmRuntimeProfileOptions::new(
            WasmProfileNativeMode::Speed,
            WasmProfileGcInitialCapacity::MiB16,
            WasmProfileRegisterAllocation::SinglePass,
        );
        let mut config = WasmtimeConfig::new();
        config.cranelift_opt_level(wasmtime::OptLevel::Speed);
        config.gc_heap_initial_size(16 * 1024 * 1024);
        config.cranelift_regalloc_algorithm(wasmtime::RegallocAlgorithm::SinglePass);
        config.compiler_inlining(wasmtime::Inlining::Yes);
        let engine = WasmtimeEngine::new(&config).unwrap();
        let observed = WasmRuntimeProfileOptions::observe_engine(&engine).unwrap();
        assert_ne!(observed, requested);
        assert_eq!(observed.native_inlining(), WasmProfileInlining::Bounded);
        assert_eq!(engine.get_compiler_inlining(), wasmtime::Inlining::Yes);
        assert_eq!(
            observed
                .native_inlining()
                .limits()
                .unwrap()
                .small_callee_size(),
            50
        );
        assert_eq!(
            observed
                .native_inlining()
                .limits()
                .unwrap()
                .sum_size_threshold(),
            2000
        );

        config.compiler_inlining(wasmtime::Inlining::Intrinsics);
        let unsupported = WasmtimeEngine::new(&config).unwrap();
        assert!(WasmRuntimeProfileOptions::observe_engine(&unsupported)
            .unwrap_err()
            .to_string()
            .contains("observed compiler inlining Intrinsics"));
    }

    #[test]
    fn shared_profile_engines_keep_inlining_identity_distinct_without_changing_speed_default() {
        let ordinary = WasmRuntimeProfileOptions::new(
            WasmProfileNativeMode::Speed,
            WasmProfileGcInitialCapacity::MiB16,
            WasmProfileRegisterAllocation::SinglePass,
        );
        let bounded = ordinary.with_native_inlining(WasmProfileInlining::Bounded);
        let a = shared_profile_wasm_engine(ordinary).unwrap();
        let b = shared_profile_wasm_engine(bounded).unwrap();
        assert!(!WasmtimeEngine::same(&a, &b));
        assert_eq!(a.get_compiler_inlining(), wasmtime::Inlining::No);
        assert_eq!(b.get_compiler_inlining(), wasmtime::Inlining::Yes);
        assert_eq!(
            WasmRuntimeProfileOptions::observe_engine(&b).unwrap(),
            bounded
        );
        assert!(WasmtimeEngine::same(
            &a,
            &shared_profile_wasm_engine(ordinary).unwrap()
        ));
        assert!(ordinary.native_inlining().limits().is_none());
    }

    #[test]
    fn native_images_require_matching_inlining_tunables() {
        let mut disabled = WasmtimeConfig::new();
        disabled.compiler_inlining(wasmtime::Inlining::No);
        let disabled = WasmtimeEngine::new(&disabled).unwrap();
        let module = WasmtimeModule::new(&disabled, b"\0asm\x01\0\0\0").unwrap();
        let bytes = module.serialize().unwrap();
        let mut bounded = WasmtimeConfig::new();
        bounded.compiler_inlining(wasmtime::Inlining::Yes);
        let bounded = WasmtimeEngine::new(&bounded).unwrap();
        // SAFETY: bytes are produced above by this exact pinned Wasmtime. The
        // test exercises its compatibility rejection, never caller-owned code.
        let mismatch = unsafe { WasmtimeModule::deserialize(&bounded, &bytes) }.unwrap_err();
        assert!(format!("{mismatch:#}").contains("inlining"), "{mismatch:#}");
    }
}

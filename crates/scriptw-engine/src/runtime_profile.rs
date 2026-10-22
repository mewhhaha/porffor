//! Observations of the product Store; no alternate collector or forced GC.
use super::*;
use std::time::Duration;
mod process_memory;
pub(super) use process_memory::ProcessMemorySampler;
pub use process_memory::{
    WasmProcessMemoryObservation, WasmProcessMemorySample, WasmProcessMemoryScope,
    WasmProcessMemoryUnavailable,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmRuntimeModuleCacheOutcome {
    Hit,
    Miss,
    Bypassed,
}
impl From<WasmModuleMemoryCacheOutcome> for WasmRuntimeModuleCacheOutcome {
    fn from(value: WasmModuleMemoryCacheOutcome) -> Self {
        match value {
            WasmModuleMemoryCacheOutcome::Hit => Self::Hit,
            WasmModuleMemoryCacheOutcome::Miss => Self::Miss,
            WasmModuleMemoryCacheOutcome::Bypassed => Self::Bypassed,
        }
    }
}

/// These spans follow the existing execution pipeline. Module includes native
/// compilation or cache lookup; execution includes host work and completion
/// decoding. Total also includes admission and gaps between these phases, and
/// the opt-in memory sampler's setup/retirement. It excludes worker creation
/// and Store destruction.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WasmRuntimeTimings {
    pub engine: Duration,
    pub module: Duration,
    pub store_and_linker: Duration,
    pub instantiate: Duration,
    pub export_lookup: Duration,
    pub execution_and_completion: Duration,
    pub total: Duration,
}

/// Capacity is reserved GC heap space, not allocated/live bytes or a peak.
/// The first snapshot follows instantiation; the second follows completion
/// decoding and dropping host completion roots. Neither requests a collection.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WasmRuntimeHeapObservation {
    pub capacity_before_execution_bytes: usize,
    pub capacity_after_execution_bytes: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmRuntimeMetricUnavailable {
    /// Pinned Wasmtime 47 exposes capacity but no public per-Store counter.
    WasmtimeHasNoPublicCounter,
    /// Boundary capacity snapshots do not establish a peak memory usage.
    NoPerInvocationPeakSampler,
}

/// The same required runtime policy used to construct both native profiles.
pub fn wasm_runtime_profile_policy() -> String {
    PRODUCT_WASMTIME_POLICY.report()
}

/// Explicit diagnostic cohort only. Ordinary execution retains automatic
/// dispatch and its CodeTooLarge retry. Forced cohorts fail instead of silently
/// changing mode; every other product Wasmtime capability/limit stays identical.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmProfileNativeMode {
    Fast,
    SizeOptimized,
}
impl WasmProfileNativeMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::SizeOptimized => "size-optimized",
        }
    }
    pub(crate) fn internal(self) -> WasmNativeCompilationMode {
        match self {
            Self::Fast => WasmNativeCompilationMode::Fast,
            Self::SizeOptimized => WasmNativeCompilationMode::SizeOptimized,
        }
    }
    pub(crate) fn from_internal(mode: WasmNativeCompilationMode) -> Self {
        match mode {
            WasmNativeCompilationMode::Fast => Self::Fast,
            WasmNativeCompilationMode::SizeOptimized => Self::SizeOptimized,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WasmRuntimeProfile {
    pub native_mode: WasmProfileNativeMode,
    pub outcome: ObservedRunOutcome,
    pub timings: WasmRuntimeTimings,
    pub module_cache: WasmRuntimeModuleCacheOutcome,
    pub heap: WasmRuntimeHeapObservation,
    pub runtime_policy: String,
    pub allocation_count: WasmRuntimeMetricUnavailable,
    pub allocation_bytes: WasmRuntimeMetricUnavailable,
    pub live_gc_bytes: WasmRuntimeMetricUnavailable,
    pub collection_count: WasmRuntimeMetricUnavailable,
    pub gc_pause_time: WasmRuntimeMetricUnavailable,
    pub peak_gc_heap_bytes: WasmRuntimeMetricUnavailable,
    /// Sampled current process RSS during this invocation, with explicit scope
    /// and sampling policy. This does not claim the unsampled true peak.
    pub process_memory: WasmProcessMemoryObservation,
}

pub(super) struct RuntimeObservation {
    pub(super) requested_native_mode: Option<WasmProfileNativeMode>,
    pub(super) actual_native_mode: Option<WasmProfileNativeMode>,
    pub(super) timings: WasmRuntimeTimings,
    pub(super) module_cache: WasmRuntimeModuleCacheOutcome,
    pub(super) heap: WasmRuntimeHeapObservation,
    pub(super) process_memory: Option<WasmProcessMemoryObservation>,
}
impl Default for RuntimeObservation {
    fn default() -> Self {
        Self {
            requested_native_mode: None,
            actual_native_mode: None,
            timings: WasmRuntimeTimings::default(),
            heap: WasmRuntimeHeapObservation::default(),
            process_memory: None,
            module_cache: WasmRuntimeModuleCacheOutcome::Bypassed,
        }
    }
}

impl Engine {
    /// Execute already emitted product Wasm in a fresh Store under the same
    /// policy, imports, native-module cache and timeout as ordinary execution.
    /// JavaScript throws remain structured outcomes; compiler/host/trap failures
    /// remain errors. Parsing, lowering and emission are outside these spans.
    pub fn profile_wasm_execution(
        &self,
        artifact: &Artifact,
        timeout_ms: Option<u64>,
        can_block: bool,
    ) -> Result<WasmRuntimeProfile, EngineError> {
        self.profile_wasm_execution_mode(artifact, timeout_ms, can_block, None)
    }

    /// Measure a separately labelled native compiler-policy cohort. This does
    /// not change product automatic policy, JS state isolation or cache keys.
    pub fn profile_wasm_execution_with_native_mode(
        &self,
        artifact: &Artifact,
        timeout_ms: Option<u64>,
        can_block: bool,
        native_mode: WasmProfileNativeMode,
    ) -> Result<WasmRuntimeProfile, EngineError> {
        self.profile_wasm_execution_mode(artifact, timeout_ms, can_block, Some(native_mode))
    }

    fn profile_wasm_execution_mode(
        &self,
        artifact: &Artifact,
        timeout_ms: Option<u64>,
        can_block: bool,
        native_mode: Option<WasmProfileNativeMode>,
    ) -> Result<WasmRuntimeProfile, EngineError> {
        run_on_sized_stack(|| {
            let mut observation = RuntimeObservation {
                requested_native_mode: native_mode,
                ..RuntimeObservation::default()
            };
            let outcome = self
                .execute_with_wasm_bytes_profiled(
                    artifact.program(),
                    timeout_ms,
                    can_block,
                    WasmModuleMemoryCachePolicy::Retain,
                    None,
                    None,
                    &WasmExecutionMode::Structured,
                    Some(&mut observation),
                )?
                .into_structured()?;
            let process_memory = observation.process_memory.ok_or_else(|| {
                EngineError::new(
                    "completed runtime profile omitted its invocation memory observation",
                )
            })?;
            let unavailable = WasmRuntimeMetricUnavailable::WasmtimeHasNoPublicCounter;
            let peak = WasmRuntimeMetricUnavailable::NoPerInvocationPeakSampler;
            let actual_mode = observation.actual_native_mode.ok_or_else(|| {
                EngineError::new(
                    "completed runtime profile omitted its actual native compiler mode",
                )
            })?;
            Ok(WasmRuntimeProfile {
                native_mode: actual_mode,
                outcome,
                timings: observation.timings,
                module_cache: observation.module_cache,
                heap: observation.heap,
                runtime_policy: wasm_runtime_profile_policy(),
                allocation_count: unavailable,
                allocation_bytes: unavailable,
                live_gc_bytes: unavailable,
                collection_count: unavailable,
                gc_pause_time: unavailable,
                peak_gc_heap_bytes: peak,
                process_memory,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn profiling_uses_product_wasm_completion_output_and_capacity_without_forced_collection() {
        let engine = Engine::new(RealmBuilder::new().build());
        let compilation = engine
            .profile_script_compilation(
                "var x = {value: 42}; print(x.value); x.value;",
                CompileOptions::default(),
            )
            .unwrap();
        let profile = engine
            .profile_wasm_execution(&compilation.artifact, None, false)
            .unwrap();
        assert_eq!(profile.outcome.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            profile.outcome.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(42.0)))
        );
        assert_eq!(
            profile.outcome.output_events,
            [HostOutputEvent::PrintLine("42".into())]
        );
        // Instantiation may leave the Store's GC heap uninitialized. The
        // program allocates its object only when main executes.
        assert!(
            profile.heap.capacity_before_execution_bytes
                <= profile.heap.capacity_after_execution_bytes
        );
        assert!(profile.heap.capacity_after_execution_bytes > 0);
        assert!(profile.timings.total >= profile.timings.execution_and_completion);
        assert_eq!(
            profile.collection_count,
            WasmRuntimeMetricUnavailable::WasmtimeHasNoPublicCounter
        );
        assert_eq!(
            profile.peak_gc_heap_bytes,
            WasmRuntimeMetricUnavailable::NoPerInvocationPeakSampler
        );
        if let WasmProcessMemoryObservation::Sampled(sample) = &profile.process_memory {
            assert_eq!(
                sample.scope(),
                WasmProcessMemoryScope::RuntimeInvocationProcessRss
            );
            assert!(sample.sample_count() >= 2);
            assert!(sample.maximum_rss_bytes() >= sample.initial_rss_bytes());
            assert!(sample.maximum_rss_bytes() >= sample.final_rss_bytes());
        }
        assert_eq!(profile.runtime_policy, PRODUCT_WASMTIME_POLICY.report());
    }

    #[test]
    fn runtime_profiling_preserves_throw_and_backend_failure_boundaries() {
        let engine = Engine::new(RealmBuilder::new().build());
        let compilation = engine
            .profile_script_compilation("throw 7;", CompileOptions::default())
            .unwrap();
        let profile = engine
            .profile_wasm_execution(&compilation.artifact, None, false)
            .unwrap();
        assert_eq!(
            profile.outcome.completion,
            ObservedCompletion::Throw(ObservedJsValue::Number(ObservedNumber::from_f64(7.0)))
        );
        assert!(engine
            .profile_wasm_execution(
                &Artifact {
                    bytes: b"invalid wasm".to_vec(),
                    description: String::new(),
                    debug_dump: String::new(),
                    runtime: None,
                    compilation_policy: None,
                },
                None,
                false,
            )
            .is_err());
    }
}

#[cfg(test)]
mod native_mode_cohort_tests {
    use super::*;

    #[test]
    fn forced_modes_are_reported_exactly_and_reuse_only_their_own_cached_modules() {
        let engine = Engine::new(RealmBuilder::new().build());
        let artifact = engine
            .profile_script_compilation(
                "let n=0; for(let i=0;i<8;i++) n+=i; print(n); n;",
                CompileOptions::default(),
            )
            .unwrap()
            .artifact;
        for requested in [
            WasmProfileNativeMode::Fast,
            WasmProfileNativeMode::SizeOptimized,
        ] {
            for repeat in 0..2 {
                let profile = engine
                    .profile_wasm_execution_with_native_mode(&artifact, None, false, requested)
                    .unwrap();
                assert_eq!(profile.native_mode, requested);
                assert_eq!(
                    profile.outcome.completion,
                    ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(
                        28.0
                    )))
                );
                assert_eq!(
                    profile.outcome.output_events,
                    [HostOutputEvent::PrintLine("28".into())]
                );
                if repeat == 1 {
                    assert_eq!(profile.module_cache, WasmRuntimeModuleCacheOutcome::Hit);
                }
                assert_eq!(profile.runtime_policy, wasm_runtime_profile_policy());
            }
        }
        assert_eq!(
            engine
                .profile_wasm_execution(&artifact, None, false)
                .unwrap()
                .native_mode,
            WasmProfileNativeMode::Fast
        );
    }
}

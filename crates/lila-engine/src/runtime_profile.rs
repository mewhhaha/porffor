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

#[derive(Debug, Clone)]
pub struct WasmRuntimeProfile {
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
    pub(super) timings: WasmRuntimeTimings,
    pub(super) module_cache: WasmRuntimeModuleCacheOutcome,
    pub(super) heap: WasmRuntimeHeapObservation,
    pub(super) process_memory: Option<WasmProcessMemoryObservation>,
}
impl Default for RuntimeObservation {
    fn default() -> Self {
        Self {
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
        run_on_sized_stack(|| {
            let mut observation = RuntimeObservation::default();
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
            Ok(WasmRuntimeProfile {
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
                    kind: ArtifactKind::Wasm,
                    bytes: b"invalid wasm".to_vec(),
                    description: String::new(),
                    debug_dump: String::new(),
                    runtime: None,
                },
                None,
                false,
            )
            .is_err());
    }
}

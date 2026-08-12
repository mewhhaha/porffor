use wasmtime::{Collector, Config};

/// The strongest garbage-collection capability provided by the pinned product
/// runtime.
///
/// Naming the missing property in the only variant keeps the current lower
/// bound honest: Wasmtime 38's DRC collector reclaims acyclic garbage but
/// cannot collect cycles. Adding a cycle-capable collector must add a new
/// variant and update the exhaustive configuration/reporting matches below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmGcCapability {
    DeferredReferenceCountingWithoutCycleCollection,
}

impl WasmGcCapability {
    pub const fn report(self) -> &'static str {
        match self {
            Self::DeferredReferenceCountingWithoutCycleCollection => {
                "collector=deferred-reference-counting cycle-collection=unavailable"
            }
        }
    }
}

/// The weak-reachability capability provided by the pinned product runtime.
///
/// This is deliberately separate from [`WasmGcCapability`]. Wasm GC and its
/// collector can manage strong references without exposing the weak-reference
/// and ephemeron operations required by WeakRef, FinalizationRegistry, WeakMap
/// and WeakSet. Adding such a facility must add a variant and update the
/// exhaustive product-policy matches below.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmWeakReachabilityCapability {
    Unavailable,
}

impl WasmWeakReachabilityCapability {
    pub const fn report(self) -> &'static str {
        match self {
            Self::Unavailable => "weak-references=unavailable ephemerons=unavailable",
        }
    }
}

/// Complete proposal/collector policy for every product Wasmtime engine.
///
/// Its field is private and the product constant below is the only value. A
/// second engine profile may tune native compilation, but it cannot silently
/// choose a different Wasm feature surface or collector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct WasmtimeRuntimePolicy {
    gc: WasmGcCapability,
    weak_reachability: WasmWeakReachabilityCapability,
}

pub(crate) const PRODUCT_WASMTIME_POLICY: WasmtimeRuntimePolicy = WasmtimeRuntimePolicy {
    gc: WasmGcCapability::DeferredReferenceCountingWithoutCycleCollection,
    weak_reachability: WasmWeakReachabilityCapability::Unavailable,
};

impl WasmtimeRuntimePolicy {
    pub(crate) const fn gc_capability(self) -> WasmGcCapability {
        self.gc
    }

    pub(crate) const fn weak_reachability_capability(self) -> WasmWeakReachabilityCapability {
        self.weak_reachability
    }

    pub(crate) fn report(self) -> String {
        format!(
            "reference-types=required function-references=required gc=required exceptions=required {} {}",
            self.gc.report(),
            self.weak_reachability.report(),
        )
    }

    pub(crate) fn configure(self, config: &mut Config) {
        config.wasm_threads(true);
        config.wasm_multi_memory(true);
        config.wasm_reference_types(true);
        config.wasm_function_references(true);
        config.wasm_gc(true);
        config.wasm_exceptions(true);
        config.wasm_tail_call(true);

        match self.gc {
            WasmGcCapability::DeferredReferenceCountingWithoutCycleCollection => {
                config.collector(Collector::DeferredReferenceCounting);
            }
        }

        match self.weak_reachability {
            WasmWeakReachabilityCapability::Unavailable => {}
        }
    }
}

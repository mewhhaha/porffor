use wasmtime::{Collector, Config};

/// The strongest garbage-collection capability provided by the pinned product
/// runtime.
///
/// The copying collector traces native Wasm GC references and reclaims cycles.
/// This describes the runtime capability, not whether every compiler-managed
/// allocation has migrated to the native GC heap.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmGcCapability {
    CopyingWithCycleCollection,
}

impl WasmGcCapability {
    pub const fn report(self) -> &'static str {
        match self {
            Self::CopyingWithCycleCollection => "collector=copying cycle-collection=available",
        }
    }
}

/// The weak-reachability capability provided by the pinned product runtime.
///
/// This is deliberately separate from [`WasmGcCapability`]. The native weak
/// operations are a vendored Wasmtime extension, not standard Wasm GC. This
/// capability does not claim that the compiler's JavaScript semantic heap has
/// migrated to native references or connected its weak builtins to this API.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WasmWeakReachabilityCapability {
    NativeCopyingExtension,
}

impl WasmWeakReachabilityCapability {
    pub const fn report(self) -> &'static str {
        match self {
            Self::NativeCopyingExtension => {
                "native-weak-references=available native-ephemerons=available native-finalization-queue=available"
            }
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
    gc: WasmGcCapability::CopyingWithCycleCollection,
    weak_reachability: WasmWeakReachabilityCapability::NativeCopyingExtension,
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
        // Wasmtime 49 gates SharedMemory creation separately from Wasm threads.
        config.shared_memory(true);
        config.wasm_multi_memory(true);
        config.wasm_reference_types(true);
        config.wasm_function_references(true);
        config.wasm_gc(true);
        config.wasm_exceptions(true);
        config.wasm_tail_call(true);

        match self.gc {
            WasmGcCapability::CopyingWithCycleCollection => {
                config.collector(Collector::Copying);
            }
        }

        match self.weak_reachability {
            WasmWeakReachabilityCapability::NativeCopyingExtension => {
                // Fail to build against upstream Wasmtime without our native
                // weak-edge extension, rather than silently losing a required
                // runtime capability. The collector selection above is required.
                const { assert!(wasmtime::NATIVE_WEAK_GC_ABI_VERSION == 1) };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };
    use wasmtime::{
        Engine, ExternRef, FieldType, Global, GlobalType, Mutability, RootScope, StorageType,
        Store, StructRef, StructRefPre, StructType, Val, ValType,
    };

    struct DropWitness(Arc<AtomicUsize>);

    impl Drop for DropWitness {
        fn drop(&mut self) {
            self.0.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn product_collector_reclaims_cycles_and_preserves_live_roots() {
        let config = crate::product_wasmtime_config(crate::WasmNativeCompilationMode::Fast)
            .expect("product runtime configuration");
        let engine = Engine::new(&config).unwrap();
        let mut store = Store::new(&engine, ());
        let ty = StructType::new(
            &engine,
            [
                FieldType::new(Mutability::Var, StorageType::ValType(ValType::ANYREF)),
                FieldType::new(Mutability::Const, StorageType::ValType(ValType::EXTERNREF)),
            ],
        )
        .unwrap();
        let allocator = StructRefPre::new(&mut store, ty);
        let root = Global::new(
            &mut store,
            GlobalType::new(ValType::ANYREF, Mutability::Var),
            Val::AnyRef(None),
        )
        .unwrap();
        let dropped = Arc::new(AtomicUsize::new(0));
        for rooted in [false, true] {
            let mut scope = RootScope::new(&mut store);
            let witness = ExternRef::new(&mut scope, DropWitness(dropped.clone())).unwrap();
            let cycle = StructRef::new(
                &mut scope,
                &allocator,
                &[Val::AnyRef(None), Val::ExternRef(Some(witness))],
            )
            .unwrap();
            cycle
                .set_field(&mut scope, 0, Val::AnyRef(Some(cycle.into())))
                .unwrap();
            if rooted {
                root.set(&mut scope, Val::AnyRef(Some(cycle.into())))
                    .unwrap();
            }
        }
        store.gc(None).unwrap();
        assert_eq!(dropped.load(Ordering::SeqCst), 1);
        root.set(&mut store, Val::AnyRef(None)).unwrap();
        store.gc(None).unwrap();
        assert_eq!(dropped.load(Ordering::SeqCst), 2);
        // The Store stays alive throughout both assertions: dropping the Store
        // would reclaim even a leaking cycle and would not test collection.
    }
}

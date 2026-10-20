//! Cargo's native image compiler. Wasmtime owns exact-Wasm cache admission.

use crate::wasmtime_config::configure_native_version;
use crate::wasmtime_policy::PRODUCT_WASMTIME_POLICY;
use scriptw_aot_wasm::RuntimeBuildArtifact;
use wasmtime::{Cache, Config, Engine, Module};

pub(crate) struct RuntimeImageCompiler {
    engine: Engine,
    module_cache: Option<Cache>,
}

impl RuntimeImageCompiler {
    pub(crate) fn new(
        mut config: Config,
        module_cache: Option<Cache>,
        native_identity: &str,
    ) -> wasmtime::Result<Self> {
        configure_native_version(&mut config, native_identity);
        config.cache(module_cache.clone());
        let engine = Engine::new(&config)?;
        PRODUCT_WASMTIME_POLICY
            .verify_engine(&engine)
            .map_err(wasmtime::Error::msg)?;
        Ok(Self {
            engine,
            module_cache,
        })
    }

    /// The build producer supplies its freshly emitted, fully admitted raw R.
    /// Current-source package/manifest publication stays with that producer.
    pub(crate) fn compile(&self, runtime: &RuntimeBuildArtifact) -> wasmtime::Result<Vec<u8>> {
        self.compile_wasm(runtime.wasm())
    }

    fn compile_wasm(&self, wasm: &[u8]) -> wasmtime::Result<Vec<u8>> {
        if self.module_cache.is_some() {
            // Unlike precompile_module, Module::new uses Wasmtime's existing
            // exact-Wasm/target/ISA/compiler/tunables/features/version cache key.
            // This temporary mapping belongs to the build Engine and is dropped
            // after serialization. It never becomes a product retention owner.
            Module::new(&self.engine, wasm)?.serialize()
        } else {
            // Cross-target builds and unavailable caches retain the ordinary
            // precompile path, which does not require a native-host mapping.
            self.engine.precompile_module(wasm)
        }
    }

    pub(crate) fn module_cache_counters(&self) -> Option<(usize, usize)> {
        self.module_cache
            .as_ref()
            .map(|cache| (cache.cache_hits(), cache.cache_misses()))
    }
}

#[cfg(test)]
#[path = "compile_tests.rs"]
mod tests;

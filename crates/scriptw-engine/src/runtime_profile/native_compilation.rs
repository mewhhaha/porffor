//! Cold native-P compilation, separate from R production and execution.
use super::*;

/// One native compilation of already emitted P. Cache lookup, Engine creation,
/// serialization and JavaScript compilation are outside `compile_and_validate`.
#[derive(Debug, Clone)]
pub struct WasmProgramNativeCompilationProfile {
    pub options: WasmRuntimeProfileOptions,
    pub engine_creation: Duration,
    /// Wasmtime validates and compiles P; no R module is loaded or compiled.
    pub compile_and_validate: Duration,
    pub wasm_bytes: usize,
    pub native_text_bytes: usize,
    pub serialized_native_bytes: usize,
}

impl Engine {
    /// Measure cold P compilation under a closed diagnostic policy. A new
    /// Engine has neither a module cache nor an incremental compiler cache.
    /// This never executes JavaScript or changes ordinary execution defaults.
    pub fn profile_program_native_compilation(
        &self,
        artifact: &Artifact,
        options: WasmRuntimeProfileOptions,
    ) -> Result<WasmProgramNativeCompilationProfile, EngineError> {
        run_on_sized_stack(|| {
            let mut config = wasmtime_config::base_config(
                options.native_mode().internal(),
                wasmtime::WasmBacktraceDetails::Environment,
            );
            config.parallel_compilation(false);
            config.cache(None);
            config.gc_heap_initial_size(options.initial_gc_capacity().bytes() as u64);
            config.cranelift_regalloc_algorithm(options.register_allocation().algorithm());
            options.configure_native_inlining(&mut config);
            let started = std::time::Instant::now();
            let native = WasmtimeEngine::new(&config).map_err(|error| {
                EngineError::new(format!("native P compilation Engine: {error:#}"))
            })?;
            let engine_creation = started.elapsed();
            PRODUCT_WASMTIME_POLICY
                .verify_engine(&native)
                .map_err(EngineError::new)?;
            let observed = WasmRuntimeProfileOptions::observe_engine(&native)?;
            if observed != options {
                return Err(EngineError::new(
                    "native P compilation policy differs from the request",
                ));
            }
            let started = std::time::Instant::now();
            let module = WasmtimeModule::new(&native, &artifact.bytes).map_err(|error| {
                EngineError::new(format!("native P validation/compilation: {error:#}"))
            })?;
            let compile_and_validate = started.elapsed();
            let serialized_native_bytes = module
                .serialize()
                .map_err(|error| EngineError::new(format!("native P serialization: {error:#}")))?
                .len();
            Ok(WasmProgramNativeCompilationProfile {
                options: observed,
                engine_creation,
                compile_and_validate,
                wasm_bytes: artifact.bytes.len(),
                native_text_bytes: module.text().len(),
                serialized_native_bytes,
            })
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cold_program_compilation_does_not_need_a_runtime_or_execute_the_program() {
        let engine = Engine::new(RealmBuilder::new().build());
        let mut artifact = engine
            .profile_script_compilation(
                "throw new Error('must not execute');",
                CompileOptions::default(),
            )
            .unwrap()
            .artifact;
        assert!(artifact.runtime.take().is_some(), "P normally links R");
        for mode in [
            WasmProfileNativeMode::Fast,
            WasmProfileNativeMode::Speed,
            WasmProfileNativeMode::SizeOptimized,
        ] {
            let options = WasmRuntimeProfileOptions::new(
                mode,
                WasmProfileGcInitialCapacity::MiB16,
                WasmProfileRegisterAllocation::SinglePass,
            );
            let profile = engine
                .profile_program_native_compilation(&artifact, options)
                .unwrap();
            assert_eq!(profile.options, options);
            assert_eq!(profile.wasm_bytes, artifact.bytes.len());
            assert!(profile.native_text_bytes > 0);
            assert!(profile.serialized_native_bytes >= profile.native_text_bytes);
        }
    }

    #[test]
    fn malformed_program_fails_native_validation_without_running_a_fallback() {
        let engine = Engine::new(RealmBuilder::new().build());
        let mut artifact = engine
            .profile_script_compilation("0;", CompileOptions::default())
            .unwrap()
            .artifact;
        artifact.bytes = b"not wasm".to_vec();
        let options = WasmRuntimeProfileOptions::new(
            WasmProfileNativeMode::Fast,
            WasmProfileGcInitialCapacity::MiB64,
            WasmProfileRegisterAllocation::Backtracking,
        );
        assert!(engine
            .profile_program_native_compilation(&artifact, options)
            .unwrap_err()
            .to_string()
            .contains("validation/compilation"));
    }
}

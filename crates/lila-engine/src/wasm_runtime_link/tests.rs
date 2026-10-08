use super::*;

#[test]
fn linked_runtime_native_modules_honor_both_bypass_retention_paths() {
    run_on_sized_stack(|| {
        let engine = Engine::new(RealmBuilder::new().build());
        let unit = engine.compile_script(
            "const linkedRuntimeBypass = {value: 4815162342}; linkedRuntimeBypass.value;",
            CompileOptions::default(),
        )?;
        let artifact = engine.emit_wasm(&unit)?;
        let runtime = &artifact.runtime.as_ref().expect("fixture must link R").0;
        let program = WasmProgramRef::new(&artifact.bytes, Some(runtime));
        let mode = plan_native_compilation(program).mode;
        let native = match mode {
            WasmNativeCompilationMode::Fast => shared_wasm_engine()?,
            WasmNativeCompilationMode::SizeOptimized => shared_size_optimized_wasm_engine()?,
        };
        for (policy, agent) in [
            (WasmModuleMemoryCachePolicy::BypassRetention, false),
            (WasmModuleMemoryCachePolicy::Retain, true),
        ] {
            let first = compile_modules(&native, program, policy, agent, mode)?;
            let second = compile_modules(&native, program, policy, agent, mode)?;
            assert!(matches!(
                first.memory_cache_outcome,
                WasmModuleMemoryCacheOutcome::Bypassed
            ));
            assert!(matches!(
                second.memory_cache_outcome,
                WasmModuleMemoryCacheOutcome::Bypassed
            ));
            // Keep both modules alive and repeat each path independently: a
            // hidden R cache must not retain either an explicit bypass or an
            // agent execution. Disk hits own distinct live image mappings.
            assert_ne!(
                first.runtime.as_ref().unwrap().image_range().start,
                second.runtime.as_ref().unwrap().image_range().start,
                "bypassed executions must not reuse a hidden retained R module (agent={agent})"
            );
        }
        Ok::<_, EngineError>(())
    })
    .expect("both linked modules compile with retention bypassed");
}

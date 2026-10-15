use super::*;

#[test]
fn program_cache_decoder_reattaches_only_the_recorded_runtime() {
    let root = std::env::temp_dir().join(format!(
        "lila-engine-linked-cache-decoder-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let cache = cache::FunctionCache::new(root.clone(), 64 * 1024 * 1024).unwrap();
    let profile = IntlCompilationProfile::default();
    let selection = IntlDataSelection::new(profile.clone());
    let runtime_cache = RuntimeWasmCache::new(&cache).expect("loaded test compiler identity");
    let runtime = lila_aot_wasm::runtime_artifact_with_inputs(
        &selection,
        embedded_runtime::inputs(Some(&runtime_cache)),
    )
    .unwrap();
    for mode in [
        WasmNativeCompilationMode::Fast,
        WasmNativeCompilationMode::SizeOptimized,
    ] {
        assert_eq!(
            wasm_module_memory_cache_key(WasmModuleSource::Runtime(&runtime), mode),
            wasm_module_memory_cache_key(WasmModuleSource::Program(runtime.bytes()), mode),
            "retained runtime digests must preserve the actual-byte cache identity"
        );
    }
    let program = b"\0asm\x01\0\0\0";
    let entry = encode_cache_entry(WasmProgramRef::new(program, Some(&runtime)));
    let (decoded, linked) = decode_cache_entry(&entry, &profile, &cache).unwrap();
    assert_eq!(decoded.as_ref(), program);
    assert_eq!(linked.unwrap().key(), runtime.key());
    assert!(decode_standalone_cache_entry(&entry).is_none());

    let mut different_runtime = entry.clone();
    different_runtime[1] ^= 1;
    assert!(decode_cache_entry(&different_runtime, &profile, &cache).is_none());
    assert!(decode_cache_entry(&entry[..KEY_BYTES], &profile, &cache).is_none());
    assert!(decode_cache_entry(&[], &profile, &cache).is_none());
    assert!(decode_cache_entry(&[0xff], &profile, &cache).is_none());

    let standalone = encode_cache_entry(WasmProgramRef::standalone(program));
    let (decoded, linked) = decode_cache_entry(&standalone, &profile, &cache).unwrap();
    assert_eq!(decoded.as_ref(), program);
    assert!(linked.is_none());
    assert_eq!(
        decode_standalone_cache_entry(&standalone),
        Some(program.as_slice())
    );
    let _ = std::fs::remove_dir_all(root);
}

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

#[test]
fn runtime_only_retention_reuses_runtime_and_bypasses_program_modules() {
    run_on_sized_stack(|| {
        let engine = Engine::new(RealmBuilder::new().build());
        let unit = engine.compile_script(
            "const runtimeOnlyProgram = {answer: 42}; runtimeOnlyProgram.answer;",
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
        let first = compile_modules(
            &native,
            program,
            WasmModuleMemoryCachePolicy::RetainRuntime,
            false,
            mode,
        )?;
        // Oversized runtime images execute without admission. Observe the
        // actual bounded cache instead of assuming every valid budget fits R.
        let key = wasm_module_memory_cache_key(WasmModuleSource::Runtime(runtime), mode);
        let retained = memory_wasm_modules()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .get(&key);
        let second = compile_modules(
            &native,
            program,
            WasmModuleMemoryCachePolicy::RetainRuntime,
            false,
            mode,
        )?;
        if let Some(retained) = retained {
            assert_eq!(
                first.runtime.as_ref().unwrap().image_range().start,
                retained.image_range().start,
                "the retained runtime must be the executed runtime Module"
            );
            assert_eq!(
                retained.image_range().start,
                second.runtime.as_ref().unwrap().image_range().start,
                "persistent requests should reuse an admitted immutable runtime Module"
            );
        } else {
            assert_ne!(
                first.runtime.as_ref().unwrap().image_range().start,
                second.runtime.as_ref().unwrap().image_range().start,
                "a runtime image outside the bounded cache must execute without hidden retention"
            );
        }
        assert_ne!(
            first.program.image_range().start,
            second.program.image_range().start
        );
        assert!(matches!(
            first.memory_cache_outcome,
            WasmModuleMemoryCacheOutcome::Bypassed
        ));
        assert!(matches!(
            second.memory_cache_outcome,
            WasmModuleMemoryCacheOutcome::Bypassed
        ));
        assert!(!memory_wasm_module_is_cached(&artifact.bytes));
        Ok::<_, EngineError>(())
    })
    .expect("runtime-only retention must leave program images unretained");
}

#[test]
fn agent_override_bypasses_runtime_only_retention() {
    run_on_sized_stack(|| {
        let engine = Engine::new(RealmBuilder::new().build());
        let unit = engine.compile_script(
            "const runtimeOnlyAgent = {answer: 19}; runtimeOnlyAgent.answer;",
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
        let first = compile_modules(
            &native,
            program,
            WasmModuleMemoryCachePolicy::RetainRuntime,
            true,
            mode,
        )?;
        let second = compile_modules(
            &native,
            program,
            WasmModuleMemoryCachePolicy::RetainRuntime,
            true,
            mode,
        )?;
        assert_ne!(
            first.runtime.as_ref().unwrap().image_range().start,
            second.runtime.as_ref().unwrap().image_range().start,
            "an agent-owned Engine must not borrow the global runtime cache"
        );
        assert_ne!(
            first.program.image_range().start,
            second.program.image_range().start
        );
        assert!(matches!(
            first.memory_cache_outcome,
            WasmModuleMemoryCacheOutcome::Bypassed
        ));
        assert!(matches!(
            second.memory_cache_outcome,
            WasmModuleMemoryCacheOutcome::Bypassed
        ));
        Ok::<_, EngineError>(())
    })
    .expect("agent execution must preserve the full bypass policy");
}

#[test]
fn persistent_script_and_module_entries_keep_fresh_globals_with_shared_runtime_code() {
    run_on_sized_stack(|| {
        let source = "globalThis.runtimeOnlyFreshGlobal = \
            (globalThis.runtimeOnlyFreshGlobal || 0) + 1; globalThis.runtimeOnlyFreshGlobal;";
        for _ in 0..2 {
            let engine = Engine::new(RealmBuilder::new().build());
            let outcome = engine.run_wasm_aot_script_on_current_thread(
                source,
                CompileOptions::default(),
                None,
                true,
            )?;
            assert!(outcome.note.contains("number(1)"), "{outcome:?}");
            let engine = Engine::new(RealmBuilder::new().build());
            let outcome = engine.run_wasm_aot_module_on_current_thread(
                "if (globalThis.runtimeOnlyFreshGlobal !== undefined) \
                     throw new Error('prior case global leaked'); \
                 globalThis.runtimeOnlyFreshGlobal = 99; export const answer = 42;",
                CompileOptions::default(),
                None,
                true,
            )?;
            assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        }
        Ok::<_, EngineError>(())
    })
    .expect("runtime Module reuse must preserve fresh Script and Module case state");
}

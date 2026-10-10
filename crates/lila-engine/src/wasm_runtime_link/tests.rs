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

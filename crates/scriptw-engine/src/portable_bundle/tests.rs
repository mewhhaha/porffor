use super::*;

fn fixture() -> &'static Vec<u8> {
    static BUNDLE: OnceLock<Vec<u8>> = OnceLock::new();
    BUNDLE.get_or_init(|| {
        let engine = Engine::new(RealmBuilder::new().build());
        let unit = engine.compile_script(
            "var make=function(){var n=39;return function(){return ++n;};};var f=make();print(f()+f()-39);",
            CompileOptions::default(),
        ).unwrap();
        engine.emit_wasm(&unit).unwrap().encode_portable_bundle(&IntlCompilationProfile::Minimal).unwrap()
    })
}

fn resign(bundle: &mut [u8]) {
    let end = bundle.len() - DIGEST_BYTES;
    let digest = Sha256::digest(&bundle[..end]);
    bundle[end..].copy_from_slice(&digest);
}

#[test]
fn loaded_bundle_executes_real_product_wasm_without_source_or_lowering_policy() {
    let engine = Engine::new(RealmBuilder::new().build());
    let loaded = engine.load_portable_bundle(fixture()).unwrap();
    assert!(loaded.compilation_policy.is_none());
    assert!(
        loaded.runtime.is_some(),
        "closure fixture requires actual R"
    );
    for _ in 0..2 {
        let profile = engine
            .profile_wasm_execution(&loaded, Some(30_000), false)
            .unwrap();
        assert_eq!(profile.outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(matches!(
            profile.outcome.completion,
            ObservedCompletion::Normal(_)
        ));
        assert_eq!(
            profile.outcome.output_events,
            vec![HostOutputEvent::PrintLine("42".into())]
        );
    }
}

#[test]
fn strict_bundle_loader_rejects_corruption_missing_runtime_and_resigned_identity_changes() {
    let engine = Engine::new(RealmBuilder::new().build());
    let original = fixture();
    for end in [0, 8, original.len() - 1] {
        assert!(engine.load_portable_bundle(&original[..end]).is_err());
    }
    let mut corrupted = original.clone();
    corrupted[original.len() / 2] ^= 1;
    assert!(engine.load_portable_bundle(&corrupted).is_err());
    // Re-signing never bypasses independently checked source identity, exact
    // runtime key, Wasm bytes, or the nested safe R package digest.
    for offset in [9, 57, 89, original.len() - 33] {
        let mut altered = original.clone();
        altered[offset] ^= 1;
        resign(&mut altered);
        assert!(
            engine.load_portable_bundle(&altered).is_err(),
            "offset {offset}"
        );
    }
    let mut missing = original.clone();
    missing[49..57].copy_from_slice(&0u64.to_le_bytes());
    resign(&mut missing);
    assert!(engine.load_portable_bundle(&missing).is_err());
}

#[test]
fn loaded_portable_bundle_cannot_fall_back_to_execution_time_javascript_compilation() {
    let engine = Engine::new(RealmBuilder::new().build());
    let unit = engine
        .compile_script(
            "Function('return 42; /*' + Math.random() + '*/')();",
            CompileOptions::default(),
        )
        .unwrap();
    let bundle = engine
        .emit_wasm(&unit)
        .unwrap()
        .encode_portable_bundle(&IntlCompilationProfile::Minimal)
        .unwrap();
    let loaded = engine.load_portable_bundle(&bundle).unwrap();
    let error = engine
        .profile_wasm_execution(&loaded, Some(30_000), false)
        .unwrap_err();
    assert_eq!(
        error.wasm_execution_failure_kind(),
        Some(WasmExecutionFailureKind::DynamicSource)
    );
    assert!(error.parse_diagnostic().is_none());
}

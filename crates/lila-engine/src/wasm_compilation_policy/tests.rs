use super::*;
use crate::{cache, wasm_runtime_link, Engine, ParseGoal, RealmBuilder, WasmProgramRef};
use std::sync::Arc;

fn selected_options() -> CompileOptions {
    CompileOptions {
        host_surface_policy: HostSurfacePolicy::Test262,
        intl_profile: IntlCompilationProfile::Conformance,
        ..CompileOptions::default()
    }
}

#[test]
fn lowering_retains_authority_before_source_mentions_any_host_operation() {
    let engine = Engine::new(RealmBuilder::new().build());
    for options in [CompileOptions::default(), selected_options()] {
        let expected = WasmCompilationPolicy::from_options(&options);
        let unit = engine.compile_script("0;", options).unwrap();
        assert_eq!(WasmCompilationPolicy::from_unit(&unit), expected);
    }
}

#[test]
fn actual_emission_retains_the_lowering_policy_for_later_native_sources() {
    let engine = Engine::new(RealmBuilder::new().build());
    let options = selected_options();
    let expected = WasmCompilationPolicy::from_options(&options);
    let unit = engine.compile_script("0;", options).unwrap();
    let artifact = engine.emit_wasm(&unit).unwrap();
    assert_eq!(artifact.compilation_policy.as_ref(), Some(&expected));
    assert_eq!(artifact.program().compilation_policy, Some(&expected));
}

#[test]
fn cache_adoption_preserves_the_exact_requested_policy_in_the_execution_ref() {
    let directory = std::env::temp_dir().join(format!(
        "lila-native-source-policy-cache-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    assert!(!directory.exists());
    let cache = Arc::new(cache::FunctionCache::new(directory.clone(), 4096).unwrap());
    let engine = Engine::new(RealmBuilder::new().build());
    // This valid empty module exercises cache ownership only; it is never
    // executed and does not stand in for a native Function acceptance result.
    let bytes = crate::tests::portable_cache_fixture();
    let mut keys = Vec::new();
    for options in [
        CompileOptions::default(),
        selected_options(),
        CompileOptions {
            language_features: crate::LanguageFeaturePolicy::default()
                .with_private_element_extensibility(
                    crate::PrivateElementExtensibility::NonExtensibleAppliesToPrivate,
                ),
            ..CompileOptions::default()
        },
    ] {
        let key = crate::program_wasm_cache_key("0;", ParseGoal::Script, &options).unwrap();
        assert!(cache.write(
            &key,
            wasm_runtime_link::encode_cache_entry(WasmProgramRef::standalone(&bytes))
        ));
        let artifact = engine
            .load_or_compile_program_wasm_on_current_thread(
                "0;",
                ParseGoal::Script,
                options.clone(),
                Some(Arc::clone(&cache)),
            )
            .unwrap();
        assert!(artifact.cached_entry.is_some());
        assert_eq!(artifact.bytes.as_ref(), bytes.as_slice());
        let expected = WasmCompilationPolicy::from_options(&options);
        assert_eq!(artifact.compilation_policy, expected);
        assert_eq!(artifact.program().compilation_policy, Some(&expected));
        keys.push(key);
    }
    assert_ne!(keys[0], keys[1]);
    assert_ne!(keys[0], keys[2]);
    assert_ne!(keys[1], keys[2]);
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn language_feature_policy_survives_lowering_emission_cache_and_agent_handoff() {
    let base = CompileOptions::default();
    let enabled = CompileOptions {
        language_features: crate::LanguageFeaturePolicy::default()
            .with_private_element_extensibility(
                crate::PrivateElementExtensibility::NonExtensibleAppliesToPrivate,
            ),
        ..CompileOptions::default()
    };
    assert_ne!(
        crate::program_wasm_cache_key("0;", ParseGoal::Script, &base),
        crate::program_wasm_cache_key("0;", ParseGoal::Script, &enabled),
    );
    for options in [base, enabled] {
        let expected = WasmCompilationPolicy::from_options(&options);
        let worker = crate::WasmAgentCompilePolicy::from_root(&options).worker_options();
        assert_eq!(worker.language_features, options.language_features);
        let unit = Engine::new(RealmBuilder::new().build())
            .compile_script("0;", options)
            .unwrap();
        assert_eq!(WasmCompilationPolicy::from_unit(&unit), expected);
        let artifact = Engine::new(RealmBuilder::new().build())
            .emit_wasm(&unit)
            .unwrap();
        assert_eq!(artifact.compilation_policy.as_ref(), Some(&expected));
    }
}

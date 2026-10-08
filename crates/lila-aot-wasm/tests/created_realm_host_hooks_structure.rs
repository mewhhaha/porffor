use lila_aot_wasm::emit;
use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower_with_host_surface_policy, HostBuiltinId, HostSurfacePolicy, StandardBuiltinId,
};
use wasmparser::{Validator, WasmFeatures};

#[test]
fn created_realm_global_carries_dollar262_detach() {
    std::thread::Builder::new()
        .name("created-realm-transitive-host-roots".to_owned())
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            // Neither the implicit detach hook nor hidden intrinsic bodies are
            // named by source. Creating the Realm must retain real code for them.
            let parsed = parse("__lilaCreateRealm().global;", ParseOptions::script())
                .expect("created Realm source parses");
            let program = lower_with_host_surface_policy(&parsed, HostSurfacePolicy::Test262);
            let script = program
                .script
                .as_ref()
                .expect("created Realm source lowers");
            assert!(script.host_builtins.contains(&HostBuiltinId::CreateRealm));
            assert!(!script
                .host_builtins
                .contains(&HostBuiltinId::DetachArrayBuffer));
            let artifact = emit(&program).expect("created Realm and transitive bodies emit");
            let mut features = WasmFeatures::default();
            for feature in [
                WasmFeatures::THREADS,
                WasmFeatures::MULTI_MEMORY,
                WasmFeatures::REFERENCE_TYPES,
                WasmFeatures::FUNCTION_REFERENCES,
                WasmFeatures::GC,
                WasmFeatures::EXCEPTIONS,
                WasmFeatures::TAIL_CALL,
            ] {
                features.set(feature, true);
            }
            Validator::new_with_features(features)
                .validate_all(&artifact.bytes)
                .expect("created Realm callable entries preserve valid Wasm");
            for (name, category) in [
                (
                    format!("host::{}", HostBuiltinId::DetachArrayBuffer.as_str()),
                    "host-builtin",
                ),
                (
                    format!(
                        "builtin::{}",
                        StandardBuiltinId::AbstractModuleSourceConstructor.debug_name()
                    ),
                    "builtin",
                ),
                (
                    format!(
                        "builtin::{}",
                        StandardBuiltinId::AbstractModuleSourcePrototypeToStringTagGetter
                            .debug_name()
                    ),
                    "builtin",
                ),
            ] {
                assert!(
                    artifact
                        .function_sizes
                        .iter()
                        .any(|body| body.name == name && body.category == category),
                    "implicit Realm callable has no real body: {name}"
                );
            }
        })
        .expect("compiler source worker starts")
        .join()
        .expect("created Realm source worker completes");
}

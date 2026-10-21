use super::*;

#[test]
fn wasm_agent_compile_policy_preserves_root_module_authority() {
    let root = CompileOptions {
        host_surface_policy: HostSurfacePolicy::Test262,
        promise_rejection_policy: PromiseRejectionPolicy::Ignore,
        module_loading_policy: ModuleLoadingPolicy::RejectAll,
        intl_profile: IntlCompilationProfile::Custom(
            CustomProfileId::parse("worker-selection").unwrap(),
        ),
        ..CompileOptions::default()
    };
    let worker = WasmAgentCompilePolicy::from_root(&root).worker_options();

    assert_eq!(worker.intl_profile, root.intl_profile);
    assert_eq!(worker.host_surface_policy, HostSurfacePolicy::Test262);
    assert_eq!(worker.module_loading_policy, ModuleLoadingPolicy::RejectAll);
    assert_eq!(
        worker.promise_rejection_policy,
        PromiseRejectionPolicy::Ignore
    );
}

#[test]
fn module_prelude_preserves_independent_parse_goals() {
    let engine = Engine::new(RealmBuilder::new().build());
    let options = CompileOptions {
        module_prelude: Some("with ({}) {}".into()),
        ..CompileOptions::default()
    };
    engine
        .compile_module("export {};", options.clone())
        .expect("sloppy Script prelude");
    assert!(engine
        .compile_module("with ({}) {}", options.clone())
        .is_err());
    engine
        .compile_script("\"use strict\"; 0;", options)
        .expect("independently sloppy Script prelude before strict Script entry");
    let invalid = CompileOptions {
        module_prelude: Some("export {};".into()),
        ..CompileOptions::default()
    };
    let error = engine
        .compile_module("export {};", invalid.clone())
        .unwrap_err();
    assert!(error.is_global_prelude_failure());
    assert!(engine
        .compile_script("0;", invalid)
        .unwrap_err()
        .is_global_prelude_failure());
}

#[test]
fn compile_module_marks_module_goal() {
    let unit = engine()
        .compile_module("export {};", CompileOptions::default())
        .expect("module compile stub should succeed");
    assert_eq!(unit.source.goal, ParseGoal::Module);
}

#[test]
fn wasm_backend_module_root_this_is_undefined_through_lexical_arrows() {
    let outcome = engine()
        .run_module(
            r#"
if (this !== undefined) throw "direct root this";
if ((() => this)() !== undefined) throw "root arrow this";
if ((() => () => this)()() !== undefined) throw "nested root arrow this";
let undefinedThisError;
try {
  this.propertyIsEnumerable("Infinity");
} catch (error) {
  undefinedThisError = error;
}
if (!(undefinedThisError instanceof TypeError)) throw "module this was folded as globalThis";
function ordinary() { return this; }
if (ordinary.call(globalThis) !== globalThis) throw "ordinary activation this";
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("module root and activation this bindings should remain distinct");
    // A synchronous Module entry completes normally with `undefined`; see
    // docs/rust-rewrite/contracts/module-entry-completion.md.
    assert!(
        outcome.note.contains("undefined(undefined)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_module_split_anonymous_default_evaluates_in_place() {
    let outcome = engine()
        .run_module(
            "let seen = 0;\nexport\ndefault (seen = 42);\n\
                 if (seen !== 42) throw \"split default was not evaluated in place\";",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("split anonymous default should compile and run");
    // A synchronous Module entry completes normally with `undefined`.
    assert!(
        outcome.note.contains("undefined(undefined)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_supports_host_print_from_shared_memory_module() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let outcome = engine_with_captured_prints(Arc::clone(&lines))
            .run_script(
                "const values = new Int32Array(new SharedArrayBuffer(4)); values[0] = 7; const old = Atomics.add(values, 0, 5); print(old + ':' + values[0]);",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("host print should read strings from shared Wasm memory");
    assert!(outcome.note.contains("undefined"), "{}", outcome.note);
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &["7:12".to_string()]
    );
}

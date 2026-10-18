use super::*;

fn run(source: &str, prelude: &str) -> Result<RunOutcome, EngineError> {
    Engine::new(RealmBuilder::new().build()).run_script(
        source,
        CompileOptions {
            module_prelude: Some(prelude.into()),
            ..CompileOptions::default()
        },
        RunOptions {
            backend: ExecutionBackend::WasmAot,
            timeout_ms: Some(5_000),
            ..RunOptions::default()
        },
    )
}
fn normal_true(source: &str, prelude: &str) {
    let outcome = run(source, prelude).unwrap();
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn sloppy_harness_and_strict_test_keep_their_actual_direct_eval_strictness() {
    normal_true(
        r#"'use strict';
        var caught = false;
        try { eval('010'); } catch (error) { caught = error instanceof SyntaxError; }
        caught && lenient('010') === 8 && (0, eval)('010') === 8 && Function('return 010;')() === 8 && harnessThis() === globalThis;
        "#,
        "function lenient(code) { return eval(code); } function harnessThis() { return this; }",
    );
}

#[test]
fn harness_runs_before_test_declarations_and_shares_the_actual_global_environment() {
    normal_true(
        "'use strict'; let testLexical = 2; var prior; trace.push('test'); before === 'undefined' && prior === 11 && harnessLexical === 4 && trace.join(',') === 'harness,test';",
        "var before = typeof testLexical; var prior = 11; let harnessLexical = 4; var trace = ['harness'];",
    );
    normal_true(
        "'use strict'; eval('1') === 'replaced';",
        "eval = function () { return 'replaced'; };",
    );
}

#[test]
fn prelude_errors_cannot_claim_entry_negative_provenance() {
    let engine = Engine::new(RealmBuilder::new().build());
    let options = CompileOptions {
        module_prelude: Some("with (".into()),
        ..CompileOptions::default()
    };
    let prelude = engine.compile_script("0;", options.clone()).unwrap_err();
    assert!(prelude.is_global_prelude_failure());
    assert!(prelude.parse_diagnostic().is_some());
    let entry = engine.compile_script("let =", options).unwrap_err();
    assert!(!entry.is_global_prelude_failure());
    assert!(entry.parse_diagnostic().is_some());
    for name in ["TypeError", "SyntaxError"] {
        let error = run(
            "throw new Error('entry must not execute');",
            &format!("throw new {name}('harness');"),
        )
        .unwrap_err();
        assert!(error.is_global_prelude_failure());
        assert!(error.message().contains("harness"), "{error}");
        assert_eq!(
            error.wasm_execution_failure_kind(),
            Some(WasmExecutionFailureKind::JavaScriptException)
        );
        assert_eq!(
            error.wasm_javascript_exception_constructor_name(),
            Some(name)
        );
    }
    let error = run("throw new TypeError('entry');", "var installed = true;").unwrap_err();
    assert!(!error.is_global_prelude_failure());
    assert_eq!(
        error.wasm_javascript_exception_constructor_name(),
        Some("TypeError")
    );
}

#[test]
fn prelude_bytes_are_framed_in_script_cache_identity_and_phase_is_only_in_p() {
    let engine = Engine::new(RealmBuilder::new().build());
    let first = CompileOptions {
        module_prelude: Some("var x = 1;".into()),
        ..CompileOptions::default()
    };
    let second = CompileOptions {
        module_prelude: Some("var x = 2;".into()),
        ..CompileOptions::default()
    };
    assert_ne!(
        program_wasm_cache_key("x;", ParseGoal::Script, &first),
        program_wasm_cache_key("x;", ParseGoal::Script, &second)
    );
    let unit = engine.compile_script("'use strict'; x;", first).unwrap();
    let artifact = engine.emit_wasm(&unit).unwrap();
    let (_, program_exports) = export_names(&artifact.bytes);
    assert!(program_exports
        .iter()
        .any(|name| name == lila_aot_wasm::GLOBAL_SCRIPT_PRELUDE_PHASE_EXPORT));
    let runtime = artifact.runtime.as_ref().unwrap();
    let (_, runtime_exports) = export_names(runtime.bytes());
    assert!(!runtime_exports
        .iter()
        .any(|name| name == lila_aot_wasm::GLOBAL_SCRIPT_PRELUDE_PHASE_EXPORT));
    let baseline_unit = engine
        .compile_script("'use strict'; x;", CompileOptions::default())
        .unwrap();
    let baseline = engine.emit_wasm(&baseline_unit).unwrap();
    let baseline_runtime = baseline.runtime.as_ref().unwrap();
    assert!(
        runtime.bytes() == baseline_runtime.bytes(),
        "a program prelude must not change shared runtime bytes"
    );
    let validate = |runtime| {
        lila_aot_wasm::ValidatedRuntimeArtifact::new(
            Arc::clone(runtime),
            IntlCompilationProfile::Minimal,
        )
        .unwrap()
    };
    assert_eq!(
        validate(&runtime.0).layout_version(),
        validate(&baseline_runtime.0).layout_version()
    );
}

fn export_names(bytes: &[u8]) -> ((), Vec<String>) {
    let exports = wasmparser::Parser::new(0)
        .parse_all(bytes)
        .filter_map(|payload| match payload.unwrap() {
            wasmparser::Payload::ExportSection(exports) => Some(
                exports
                    .into_iter()
                    .map(|export| export.unwrap().name.to_owned())
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        })
        .flatten()
        .collect();
    ((), exports)
}

#[test]
fn canonical_globals_precede_entry_function_replacement_with_standard_attributes() {
    normal_true(
        r#"var d = Object.getOwnPropertyDescriptor(globalThis, 'Array');
        function Array() { return 'entry'; }
        before === 'function' && beforeValue !== Array && Array() === 'entry' &&
        prior.writable && !prior.enumerable && prior.configurable &&
        d.value === Array && d.writable && d.enumerable && !d.configurable &&
        typeof JSON === 'object' && typeof Reflect === 'object' &&
        typeof Promise === 'function' && typeof RegExp === 'function';"#,
        "var before = typeof Array; var beforeValue = Array; var prior = Object.getOwnPropertyDescriptor(globalThis, 'Array');",
    );
}

#[test]
fn side_effectful_function_source_coercion_keeps_entry_globals_after_prelude() {
    normal_true(
        r#"var i = 0; var p = { toString: function() { return 'a' + (++i); } };
        var obj = {}; new Function(p, p, p, 'this.shifted=a3;').apply(obj,
        (function() { return arguments; })('a', 'b', 'c'));
        i === 3 && obj.shifted === 'c' && typeof globalThis.shifted === 'undefined' &&
        typeof Function === 'function' && typeof Math === 'object' && marker === 7;"#,
        "var marker = 7;",
    );
}

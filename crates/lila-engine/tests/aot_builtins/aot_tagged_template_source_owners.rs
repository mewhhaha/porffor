use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};
use lila_ir::DynamicFunctionKind;

fn assert_template_source(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("template owner source compiles and executes through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn independently_prepared_units_retain_their_own_template_sites() {
    assert_template_source(include_str!(
        "../fixtures/tagged_template_owners/independent_units.js"
    ));
}
#[test]
fn fresh_parses_and_escaping_functions_keep_distinct_execution_owners() {
    assert_template_source(include_str!(
        "../fixtures/tagged_template_owners/fresh_invocations_and_escaping_closures.js"
    ));
}
#[test]
fn borrowed_functions_and_realm_scripts_use_defining_realm_template_arrays() {
    assert_template_source(include_str!(
        "../fixtures/tagged_template_owners/defining_realm.js"
    ));
}
#[test]
fn class_and_resumable_contexts_retain_their_original_template_owner() {
    assert_template_source(include_str!(
        "../fixtures/tagged_template_owners/class_and_generator_contexts.js"
    ));
}

#[test]
fn async_resumption_reuses_the_original_captured_template_owner() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_script(
            include_str!("../fixtures/tagged_template_owners/async_contexts.js"),
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("compiled async template owner source");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(matches!(observed.completion, ObservedCompletion::Normal(_)));
    assert_eq!(
        observed.output_events,
        vec![HostOutputEvent::PrintLine("async-template-ok".into())]
    );
}

#[test]
fn repeated_template_source_hints_execute_live_cells_and_conversion_hooks() {
    let source = r#"
function direct(text) {
    function captured() { return text; }
    return eval(`'${text}${text}${text}${text}${text}${text}${text}${text}${text}';`);
}
function compile(text) {
    function captured() { return text; }
    return Function(`return '${text}${text}${text}${text}${text}${text}${text}${text}${text}';`)();
}
if (direct('a') !== 'aaaaaaaaa' || direct('b') !== 'bbbbbbbbb' || direct('c') !== 'ccccccccc')
    throw 'captured eval source';
if (compile('a') !== 'aaaaaaaaa' || compile('b') !== 'bbbbbbbbb' || compile('c') !== 'ccccccccc')
    throw 'captured Function source';
let reads = 0, conversions = 0;
let box = { toString() { return 'a'; } };
Object.defineProperty(box, 'toString', { get() {
    reads++;
    return function () { conversions++; return 'a'; };
} });
if (compile(box) !== 'aaaaaaaaa' || reads !== 9 || conversions !== 9)
    throw 'Get and ToString must run once for every substitution';
function changed(text) {
    return Function(`return '${text}${(text = text === 'a' ? 'b' : 'a', '')}${text}';`)();
}
if (changed('a') !== 'ab' || changed('b') !== 'ba') throw 'reassignment alternatives';
let calls = 0;
function changedByCall(text) {
    function mutate() { calls++; text = 'b'; }
    return Function(`return '${text}${(mutate(), '')}${text}';`)();
}
if (changedByCall('a') !== 'ab' || changedByCall('b') !== 'bb' || calls !== 2)
    throw 'intervening calls remain live';
function independent(left, right) {
    return Function(`return '${left}${right}${left}${right}${left}${right}${left}${right}${left}${right}';`)();
}
if (independent('a', 'x') !== 'axaxaxaxax' || independent('a', 'y') !== 'ayayayayay'
    || independent('b', 'x') !== 'bxbxbxbxbx' || independent('b', 'y') !== 'bybybybyby')
    throw 'independent bindings';
function shadow(text) {
    function inner(text) {
        return Function(`return '${text}${text}${text}${text}${text}${text}${text}${text}${text}';`)();
    }
    return inner('x') + inner('y') + inner('z');
}
shadow('a') === 'xxxxxxxxxyyyyyyyyyzzzzzzzzz' && shadow('b') === 'xxxxxxxxxyyyyyyyyyzzzzzzzzz';
"#;
    for prefix in ["", "'use strict';"] {
        assert_template_source(&format!("{prefix}{source}"));
    }
}

#[test]
fn correlated_template_hints_never_dispatch_a_different_effectful_runtime_source() {
    let source = r#"
let reads = 0, conversions = 0;
function compile(text, checkEffects) {
    let body = `return '${text}${text}${text}${text}${text}${text}${text}${text}${text}';`;
    if (checkEffects && (reads !== 9 || conversions !== 9)) throw 'template effects were skipped';
    return Function(body)();
}
if (compile('a', false) !== 'aaaaaaaaa' || compile('b', false) !== 'bbbbbbbbb'
    || compile('c', false) !== 'ccccccccc') throw 'prepared primitive hints';
let box = { toString() { return 'a'; } };
Object.defineProperty(box, 'toString', { get() {
    reads++;
    return function () { conversions++; return conversions % 2 ? 'a' : 'b'; };
} });
let mixed = compile(box, true);
if (mixed !== 'ababababa' || reads !== 9 || conversions !== 9)
    throw 'native mixed source or template effects';
true;
"#;
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for prefix in ["", "'use strict';"] {
        let source = format!("{prefix}{source}");
        let unit = Engine::new(RealmBuilder::new().build())
            .compile_script(
                &source,
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
            )
            .expect("outer template source lowers without broadening prepared admission");
        let script = unit.ir.script.as_ref().expect("Script IR");
        assert!(script.prepared_dynamic_functions.len() <= 256);
        assert!(
            !script
                .prepared_dynamic_functions
                .iter()
                .any(|entry| entry.kind == DynamicFunctionKind::Ordinary
                    && entry
                        .arguments
                        .iter()
                        .map(String::as_str)
                        .eq(["return 'ababababa';"])),
            "the mixed effectful tuple must not select a primitive correlated hint"
        );
        let outcome = Engine::new(RealmBuilder::new().build())
            .run_script(
                &source,
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("the actual mixed source compiles through the native Function path");
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }
}

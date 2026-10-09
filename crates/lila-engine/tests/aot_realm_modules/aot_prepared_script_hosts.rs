use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_script(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
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
        .expect("prepared source compiles and executes through Wasm");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}: {}",
        observed.completion,
        observed.note
    );
    assert_eq!(
        observed.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).into()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn host_used_only_by_a_prepared_callback_is_available_to_queued_jobs() {
    assert_script(
        r#"
(0, eval)("function finish(value) { print('finished:' + value); }");
const original = finish;
finish = function(value) { original(value); };
Promise.resolve(7).then(finish);
"#,
        &["finished:7"],
    );
}

#[test]
fn realm_script_can_use_a_host_absent_from_the_entry_source() {
    assert_script(
        "var $262 = { evalScript: __lilaRealmEvalScript };\n\
         $262.evalScript(\"print('realm script');\");",
        &["realm script"],
    );
}

#[test]
fn prepared_function_can_use_a_host_absent_from_the_entry_source() {
    assert_script(
        "Function(\"print('prepared function');\")();",
        &["prepared function"],
    );
}

#[test]
fn entry_var_declaration_does_not_erase_the_host_global() {
    assert_script(
        "var print; (0, eval)(\"print('existing host');\");",
        &["existing host"],
    );
}

#[test]
fn entry_source_function_overrides_the_host_global() {
    assert_script(
        r#"
var received;
function print(value) { received = value; }
(0, eval)("print(7);");
if (received !== 7) throw 'source function was replaced';
"#,
        &[],
    );
}

#[test]
fn prepared_execution_keeps_runtime_host_replacement_and_deletion() {
    assert_script(
        r#"
var received;
globalThis.print = function(value) { received = value; };
(0, eval)("print(9);");
if (received !== 9) throw 'replacement was overwritten';
delete globalThis.print;
if ((0, eval)("typeof print;") !== 'undefined') throw 'deleted host was restored';
"#,
        &[],
    );
}

#[test]
fn prepared_source_declarations_wait_until_execution() {
    assert_script(
        r#"
if (Object.prototype.hasOwnProperty.call(globalThis, 'later')) throw 'early var';
if (Object.prototype.hasOwnProperty.call(globalThis, 'finish')) throw 'early function';
(0, eval)("var later = 7; function finish() { print(later); } let privateName = 9;");
if (typeof privateName !== 'undefined') throw 'eval lexical escaped';
finish();
"#,
        &["7"],
    );
}

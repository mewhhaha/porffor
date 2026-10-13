use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_modes(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
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
            .expect("callable entry roles execute through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{source}"
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(expected.into())],
            "{source}"
        );
    }
}

#[test]
fn new_target_is_separate_from_resumable_activations_and_lexical_captures() {
    assert_modes(
        include_str!("../fixtures/callable_entry_roles/new_target_and_resumption.js"),
        "callable-entry-roles:ok",
    );
}

#[test]
fn prepared_realm_scripts_publish_the_right_callable_family_and_context() {
    assert_modes(
        include_str!("../fixtures/callable_entry_roles/prepared_realm_entries.js"),
        "prepared-callable-entry-roles:ok",
    );
}

fn assert_sloppy_arguments(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler jobs");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("Annex B arguments binding executes through real Wasm");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn annex_b_arguments_copy_updates_the_existing_binding_only_when_selected() {
    assert_sloppy_arguments(
        r#"
function selected(value) {
  var before = arguments;
  var read = () => arguments;
  if (typeof before !== 'object' || before[0] !== 17) throw 'arguments missing at entry';
  if (false) { function arguments() {} }
  if (read() !== before) throw 'unselected declaration replaced arguments';
  { function arguments() { return 19; } }
  return typeof arguments === 'function' && read() === arguments && arguments() === 19;
}
function parameter(arguments) {
  { function arguments() {} }
  return arguments === 23;
}
function lexical() {
  let arguments = 29;
  { function arguments() {} }
  return arguments === 29;
}
selected(17) && parameter(23) && lexical();
"#,
    );
}

#[test]
fn annex_b_arguments_late_body_binding_preserves_parameter_and_arrow_environments() {
    assert_sloppy_arguments(
        r#"
function defaults(read = () => arguments, alter = () => arguments = 31) {
  let before = read();
  let later = () => arguments;
  if (later() !== before || typeof before !== 'object') throw 'body binding created at entry';
  { function arguments() { return 37; } }
  let body = arguments;
  return later() === body && read() === before && body() === 37
    && alter() === 31 && read() === 31 && later() === body;
}
function enclosing(value) {
  let before = arguments;
  let arrow = () => {
    let read = () => arguments;
    if (read() !== before) throw 'arrow created own arguments at entry';
    { function arguments() { return 41; } }
    return read() === arguments && arguments() === 41;
  };
  return arrow() && arguments === before && before[0] === value;
}
defaults() && enclosing(43);
"#,
    );
}

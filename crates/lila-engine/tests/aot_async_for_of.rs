use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_suspended_iteration_values(mode: &str) {
    let source = r#"
async function* stream(source) {
  for await (BINDING_MODE value of source) {
    yield value * 2;
    print("after:" + value);
    yield value + 1;
  }
}
var iterator = stream([3, 7]);
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) {
  report(result); return iterator.next();
}).then(function (result) {
  report(result); return iterator.next();
}).then(function (result) {
  report(result); return iterator.next();
}).then(function (result) {
  report(result); return iterator.next();
}).then(report);
void 0;
"#
    .replace("BINDING_MODE", mode);
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            &source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("suspending for-await must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    let expected = [
        "6:false",
        "after:3",
        "4:false",
        "14:false",
        "after:7",
        "8:false",
        "undefined:true",
    ]
    .into_iter()
    .map(|line| HostOutputEvent::PrintLine(line.to_string()))
    .collect::<Vec<_>>();
    assert_eq!(outcome.output_events, expected, "{mode} head");
}

#[test]
fn const_head_survives_multiple_body_suspensions() {
    assert_suspended_iteration_values("const");
}

#[test]
fn let_head_survives_multiple_body_suspensions() {
    assert_suspended_iteration_values("let");
}

#[test]
fn var_head_survives_multiple_body_suspensions() {
    assert_suspended_iteration_values("var");
}

#[test]
fn captured_let_head_reuses_one_cell_after_resume_and_fresh_cells_between_iterations() {
    let source = r#"
var captures = [];
async function* stream(source) {
  for await (let value of source) {
    captures.push(function () { return value; });
    yield value;
    value = value + 10;
    yield captures[captures.length - 1]();
  }
  print("captures:" + captures[0]() + ":" + captures[1]());
}
var iterator = stream([1, 2]);
function report(result) { print(result.value + ":" + result.done); }
iterator.next().then(function (result) {
  report(result); return iterator.next();
}).then(function (result) {
  report(result); return iterator.next();
}).then(function (result) {
  report(result); return iterator.next();
}).then(function (result) {
  report(result); return iterator.next();
}).then(report);
void 0;
"#;
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("captured for-await head must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    let expected = [
        "1:false",
        "11:false",
        "2:false",
        "12:false",
        "captures:11:12",
        "undefined:true",
    ]
    .into_iter()
    .map(|line| HostOutputEvent::PrintLine(line.to_string()))
    .collect::<Vec<_>>();
    assert_eq!(outcome.output_events, expected);
}

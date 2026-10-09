use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_wasm_lines(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
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
        .expect("for-await rejection closing must execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    assert_eq!(
        outcome.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn rejected_sync_value_closes_once_without_reading_the_return_result() {
    assert_wasm_lines(
        r#"
var marker = {}, closes = 0, gets = 0, receivers = true;
var iterator = {
  next() { return { value: Promise.reject(marker), done: false }; },
  return() {
    closes++; receivers = receivers && this === iterator;
    return {
      get done() { gets++; throw 'must not read done'; },
      get value() { gets++; throw 'must not read value'; },
      get then() { gets++; throw 'must not await return'; }
    };
  }
};
var source = { [Symbol.iterator]() { return iterator; } };
async function consume() {
  try { for await (let value of source) { print('unexpected body'); } }
  catch (error) { print('original:' + (error === marker)); }
  print('closes:' + closes + ':gets:' + gets + ':receivers:' + receivers);
}
consume().catch(function(error) { print('rejected:' + error); });
void 0;
"#,
        &["original:true", "closes:1:gets:0:receivers:true"],
    );
}

#[test]
fn rejected_generator_yield_runs_finally_once_before_async_catch() {
    assert_wasm_lines(
        r#"
var marker = {}, finalizations = 0, trace = '';
function* source() {
  try { yield Promise.reject(marker); }
  finally { finalizations++; trace += 'finally;'; }
}
async function consume() {
  try { for await (const value of source()); }
  catch (error) { trace += error === marker ? 'caught;' : 'wrong-error;'; }
  print(trace + finalizations);
}
consume().catch(function(error) { print('rejected:' + error); });
void 0;
"#,
        &["finally;caught;1"],
    );
}

#[test]
fn original_rejection_wins_over_close_errors_and_done_avoids_closing() {
    assert_wasm_lines(
        r#"
var marker = {}, closes = 0;
async function check(label, close, done) {
  var source = { [Symbol.iterator]() {
    return { next() { return { value: Promise.reject(marker), done: done }; }, return: close };
  } };
  try { for await (const value of source) { print('unexpected body'); } }
  catch (error) { print(label + ':' + (error === marker) + ':' + closes); }
}
async function run() {
  await check('absent', undefined, false);
  await check('null', null, false);
  await check('non-callable', 7, false);
  await check('throwing', function() { closes++; throw 'close-error'; }, false);
  await check('primitive', function() { closes++; return 7; }, false);
  await check('done', function() { closes++; return {}; }, true);
}
run().catch(function(error) { print('rejected:' + error); });
void 0;
"#,
        &[
            "absent:true:0",
            "null:true:0",
            "non-callable:true:0",
            "throwing:true:1",
            "primitive:true:2",
            "done:true:2",
        ],
    );
}

fn assert_body_await_modes(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let outcome = Engine::new(RealmBuilder::new().build())
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
            .expect("for-await body Await fixture compiles and executes through Wasm AOT");
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            outcome.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{source}"
        );
        assert_eq!(
            outcome.output_events,
            vec![HostOutputEvent::PrintLine(expected.into())],
            "{source}"
        );
    }
}

#[test]
fn awaited_next_and_body_keep_cached_methods_and_iteration_cells() {
    assert_body_await_modes(
        include_str!("../fixtures/for_await_body_await/next_and_body.js"),
        "for-await-next-body:ok",
    );
}

#[test]
fn body_control_waits_for_finalizers_and_closes_once() {
    assert_body_await_modes(
        include_str!("../fixtures/for_await_body_await/control_and_close.js"),
        "for-await-control-close:ok",
    );
}

#[test]
fn body_and_protocol_failures_keep_original_values_and_execution_realms() {
    assert_body_await_modes(
        include_str!("../fixtures/for_await_body_await/abrupt_and_realms.js"),
        "for-await-abrupt-realms:ok",
    );
}

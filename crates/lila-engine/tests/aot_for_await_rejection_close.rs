use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
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

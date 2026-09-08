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
        .expect("async loop bindings must compile and execute through Wasm AOT");
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
fn async_for_retains_lexical_index_through_await_update_and_condition() {
    assert_wasm_lines(
        r#"
async function collect() {
  const results = [];
  for (let index = 0; index < 3; index++) {
    results.push(await Promise.resolve(index));
    results.push(await Promise.resolve(index + 10));
  }
  print(results.length + ':' + results.join(','));
}
collect().catch(function(error) { print('rejected:' + error); });
void 0;
"#,
        &["6:0,10,1,11,2,12"],
    );
}

#[test]
fn async_while_retains_updated_lexical_binding_through_await() {
    assert_wasm_lines(
        r#"
async function collect() {
  let index = 0;
  const results = [];
  while (index < 3) {
    results.push(await Promise.resolve(index));
    index++;
  }
  print(index + ':' + results.join(','));
}
collect().catch(function(error) { print('rejected:' + error); });
void 0;
"#,
        &["3:0,1,2"],
    );
}

#[test]
fn async_for_of_retains_wait_async_view_and_index_bindings() {
    assert_wasm_lines(
        r#"
async function check(view, zero, different) {
  var indices = [view => -0, view => '-0', view => view.length - 1,
    view => ({ valueOf() { return 0; } }),
    view => ({ toString() { return '0'; }, valueOf: false })];
  print(await Atomics.waitAsync(view, 0, zero, 0).value);
  print(await Atomics.waitAsync(view, 0, different, 0).value);
  const results = [];
  for (let indexOf of indices) {
    let index = indexOf(view);
    view.fill(zero);
    Atomics.store(view, index, different);
    results.push(await Atomics.waitAsync(view, index, zero).value);
  }
  print(results.length + ':' + results.join(','));
}
async function run() {
  await check(new Int32Array(new SharedArrayBuffer(1024), 32, 20), 0, 37);
  await check(new BigInt64Array(new SharedArrayBuffer(1024), 32, 20), 0n, 37n);
}
run().catch(function(error) { print('rejected:' + error); });
void 0;
"#,
        &[
            "timed-out",
            "not-equal",
            "5:not-equal,not-equal,not-equal,not-equal,not-equal",
            "timed-out",
            "not-equal",
            "5:not-equal,not-equal,not-equal,not-equal,not-equal",
        ],
    );
}

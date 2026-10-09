use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

#[test]
fn poisoned_promise_constructor_rejects_delegation_and_closes_once() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            r#"
function start() {
  const marker = {};
  let closed = 0;
  function* source() {
    try {
      const promise = Promise.resolve('unused');
      Object.defineProperty(promise, 'constructor', { get() { throw marker; } });
      yield promise;
    } finally { closed++; }
  }
  async function* delegate() { yield* source(); }
  delegate().next().then(
    () => print('unexpected fulfillment'),
    error => print((error === marker) + ':' + closed)
  );
}
start();
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(60_000),
                ..RunOptions::default()
            },
        )
        .expect("Promise rejection must execute through Wasm without trapping");
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    assert_eq!(
        outcome.output_events,
        [HostOutputEvent::PrintLine("true:1".to_string())]
    );
}

#[test]
fn poisoned_promise_constructor_in_plain_await_preserves_error_identity() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            r#"
function start() {
  const marker = {};
  async function run() {
    const promise = Promise.resolve('unused');
    Object.defineProperty(promise, 'constructor', { get() { throw marker; } });
    try { await promise; print('unexpected fulfillment'); }
    catch (error) { print(error === marker); }
  }
  run();
}
start();
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(60_000),
                ..RunOptions::default()
            },
        )
        .expect("abrupt PromiseResolve must become an ordinary awaited rejection");
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    assert_eq!(
        outcome.output_events,
        [HostOutputEvent::PrintLine("true".to_string())]
    );
}

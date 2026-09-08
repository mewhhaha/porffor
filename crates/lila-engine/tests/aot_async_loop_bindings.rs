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

#[test]
fn async_while_and_for_of_resume_each_await_without_repeating_effects() {
    assert_wasm_lines(
        r#"
async function collect() {
  let index = 0;
  const trace = [];
  while (index < 2) {
    trace.push('before' + index);
    await Promise.resolve(index);
    trace.push('middle' + index);
    await Promise.resolve(index + 10);
    trace.push('after' + index);
    index++;
  }
  print(index + ':' + trace.join(','));
  const callbacks = [];
  const values = [];
  for (let value of [3, 7]) {
    let doubled = value * 2;
    callbacks.push(() => value);
    values.push('before' + value);
    await Promise.resolve(value);
    values.push('middle' + doubled);
    await Promise.resolve(doubled);
    values.push('after' + value);
  }
  print(values.join(','));
  print(callbacks.length + ':' + callbacks[0]() + ',' + callbacks[1]());
  const nested = [];
  {
    let outer = 17;
    nested.push(() => outer);
    {
      let inner = 'inner';
      nested.push(() => inner);
      let retained = 'saved';
      const marker = {};
      try { throw marker; }
      catch (error) {
        print(retained + ':' + (error === marker) + ':' + nested[0]() + ':' + nested[1]());
      }
    }
  }
}
collect().catch(function(error) { print('rejected:' + error); });
void 0;
"#,
        &[
            "2:before0,middle0,after0,before1,middle1,after1",
            "before3,middle6,after3,before7,middle14,after7",
            "2:3,7",
            "saved:true:17:inner",
        ],
    );
}

#[test]
fn async_second_loop_await_rejection_preserves_finally_and_iterator_close() {
    assert_wasm_lines(
        r#"
async function check() {
  const marker = {};
  let updates = 0;
  let caught = false;
  const trace = [];
  try {
    for (let index = 0; index < 2; updates++, index++) {
      trace.push('before');
      await Promise.resolve(index);
      trace.push('middle');
      await Promise.reject(marker);
      trace.push('after');
    }
  } catch (error) {
    caught = error === marker;
  } finally {
    trace.push('finally');
  }
  print('for:' + caught + ':' + updates + ':' + trace.join(','));
  let nextCalls = 0;
  let closeCalls = 0;
  let iteratorCaught = false;
  const iteratorTrace = [];
  const iterable = {
    [Symbol.iterator]() { return this; },
    next() { nextCalls++; return { value: nextCalls, done: false }; },
    return() { closeCalls++; return {}; }
  };
  try {
    for (const value of iterable) {
      iteratorTrace.push('before' + value);
      await Promise.resolve(value);
      iteratorTrace.push('middle' + value);
      await Promise.reject(marker);
      iteratorTrace.push('after');
    }
  } catch (error) {
    iteratorCaught = error === marker;
  } finally {
    iteratorTrace.push('finally');
  }
  print('for-of:' + iteratorCaught + ':' + nextCalls + ':' + closeCalls + ':' + iteratorTrace.join(','));
}
check().catch(function(error) { print('rejected:' + error); });
void 0;
"#,
        &[
            "for:true:0:before,middle,finally",
            "for-of:true:1:1:before1,middle1,finally",
        ],
    );
}

#[test]
fn async_generator_loop_resumes_multiple_awaits_before_the_following_yield() {
    assert_wasm_lines(
        r#"
async function* sequence() {
  for (let index = 0; index < 2; index++) {
    print('before' + index);
    await Promise.resolve(index);
    print('middle' + index);
    await Promise.resolve(index + 10);
    print('after' + index);
  }
  yield 42;
  return 9;
}
async function check() {
  const iterator = sequence();
  const first = await iterator.next();
  print(first.value + ':' + first.done);
  const last = await iterator.next();
  print(last.value + ':' + last.done);
}
check().catch(function(error) { print('rejected:' + error); });
void 0;
"#,
        &[
            "before0", "middle0", "after0", "before1", "middle1", "after1", "42:false", "9:true",
        ],
    );
}

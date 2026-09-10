use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_iterator_trace(source: &str, expected: &[&str]) {
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
        .expect("iterator consumer must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        outcome.completion
    );
    let expected = expected
        .iter()
        .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
        .collect::<Vec<_>>();
    assert_eq!(outcome.output_events, expected, "source:\n{source}");
}

#[test]
fn iterator_consumers_supply_undefined_before_callback_this_binding() {
    assert_iterator_trace(
        r#"
function* values() { yield 1; }
function strictCallback() {
  'use strict';
  print('strict:' + (this === undefined));
  return true;
}
var globalReceiver = function () { return this; }.call(undefined);
function sloppyCallback() {
  print('sloppy:' + (this === globalReceiver));
  return true;
}
values().forEach(strictCallback);
values().every(strictCallback);
values().some(strictCallback);
values().find(strictCallback);
values().reduce(strictCallback, 0);
values().forEach(sloppyCallback);
values().every(sloppyCallback);
values().some(sloppyCallback);
values().find(sloppyCallback);
values().reduce(sloppyCallback, 0);
"#,
        &[
            "strict:true",
            "strict:true",
            "strict:true",
            "strict:true",
            "strict:true",
            "sloppy:true",
            "sloppy:true",
            "sloppy:true",
            "sloppy:true",
            "sloppy:true",
        ],
    );
}

#[test]
fn iterator_records_cache_getters_while_consuming_every_generator_iteration() {
    assert_iterator_trace(
        r#"
var nextGets = 0;
var nextCalls = 0;
class CountingIterator {
  get next() {
    ++nextGets;
    let iterator = (function* () {
      for (let index = 1; index < 5; ++index) { yield index; }
    })();
    return function () { ++nextCalls; return iterator.next(); };
  }
}
var wrapped = Iterator.from(new CountingIterator());
print(nextGets + ':' + nextCalls);
print(wrapped.toArray().join(','));
print(nextGets + ':' + nextCalls);
class DroppedIterator extends Iterator {
  get next() {
    ++nextGets;
    let iterator = (function* () {
      for (let index = 1; index < 5; ++index) { yield index; }
    })();
    return function () { ++nextCalls; return iterator.next(); };
  }
}
nextGets = 0;
nextCalls = 0;
var dropped = new DroppedIterator().drop(2);
print(nextGets + ':' + nextCalls);
for (const value of dropped) { print(value); }
print(nextGets + ':' + nextCalls);
"#,
        &["1:0", "1,2,3,4", "1:5", "1:0", "3", "4", "1:5"],
    );
}

#[test]
fn drop_and_take_observe_underlying_generator_advancement() {
    assert_iterator_trace(
        r#"
function* values() {
  for (let index = 0; index < 5; ++index) { yield index; }
}
function report(result) { print(result.value + ':' + result.done); }
var iterator = values();
var dropped = iterator.drop(2);
report(iterator.next());
report(dropped.next());
report(dropped.next());
report(dropped.next());
iterator = values();
var taken = iterator.take(2);
report(iterator.next());
report(taken.next());
report(taken.next());
report(taken.next());
report(iterator.next());
"#,
        &[
            "0:false",
            "3:false",
            "4:false",
            "undefined:true",
            "0:false",
            "1:false",
            "2:false",
            "undefined:true",
            "undefined:true",
        ],
    );
}

#[test]
fn predicate_consumers_convert_results_after_every_resumed_iteration() {
    assert_iterator_trace(
        r#"
function* values() {
  try {
    yield 0;
    yield 1;
    yield 2;
    yield 3;
    yield 4;
  } finally { print('closed'); }
}
var calls = 0;
print(values().every(function (value) { calls++; return value < 3 ? {} : 0; }));
print(calls);
calls = 0;
print(values().some(function (value) { calls++; return value < 3 ? '' : 'yes'; }));
print(calls);
calls = 0;
print(values().find(function (value) { calls++; return value < 3 ? null : []; }));
print(calls);
"#,
        &[
            "closed", "false", "4", "closed", "true", "4", "closed", "3", "4",
        ],
    );
}

#[test]
fn flat_map_gets_symbol_iterator_and_falls_back_only_for_nullish_methods() {
    assert_iterator_trace(
        r#"
function* outer() { yield 0; }
function* inner() { yield 0; yield 1; yield 2; }
var mapped = outer().flatMap(function () {
  let iterator = inner();
  return { [Symbol.iterator]: 0, next: () => iterator.next() };
});
try { mapped.next(); print('missed error'); }
catch (error) { print(error instanceof TypeError); }
mapped = outer().flatMap(function () {
  let iterator = inner();
  return { [Symbol.iterator]: null, next: () => iterator.next() };
});
print(Array.from(mapped).join(','));
mapped = outer().flatMap(function () {
  let iterator = inner();
  return { [Symbol.iterator]: undefined, next: () => iterator.next() };
});
print(Array.from(mapped).join(','));
"#,
        &["true", "0,1,2", "0,1,2"],
    );
}

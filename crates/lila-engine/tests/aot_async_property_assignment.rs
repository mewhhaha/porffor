use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_trace(source: &str, expected: &[&str]) {
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
        .expect("async assignments must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}\noutput: {:?}\nsource:\n{source}",
        outcome.completion,
        outcome.output_events
    );
    assert_eq!(
        outcome.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
            .collect::<Vec<_>>(),
        "source:\n{source}"
    );
}

#[test]
fn awaited_keys_capture_the_receiver_before_key_evaluation() {
    assert_trace(
        r#"
async function check() {
  let first = {value: 0}, second = {value: 0}, receiver = first;
  receiver[await (receiver = second, "value")] = 1;
  print(first.value + ":" + second.value);
  receiver = first;
  receiver[String(await (receiver = second, "value"))] = 2;
  print(first.value + ":" + second.value);
}
check();
"#,
        &["1:0", "2:0"],
    );
}

#[test]
fn awaited_rhs_assignments_keep_the_original_reference_and_result() {
    assert_trace(
        r#"
async function check() {
  let first = {value: 0}, second = {value: 0}, receiver = first;
  receiver.value = await (receiver = second, 7);
  print(first.value + ":" + second.value);
  receiver = first;
  let key = "value";
  receiver[key] = await (receiver = second, key = "other", 8);
  print(first.value + ":" + second.value + ":" + (first.other === undefined));
  let marker = {};
  receiver = first;
  let assign = async () => receiver.value = await (receiver = second, marker);
  let result = await assign();
  print((result === marker) + ":" + (first.value === marker) + ":" + second.value);
  function consume(value) { print(value === marker); return value; }
  receiver = first;
  let nested = consume(receiver.value = await (receiver = second, marker));
  print(nested === marker);
  receiver = first;
  let makeReader = async () => () => receiver;
  let read = await makeReader();
  print("empty parent:" + (read() === first));
  receiver = second;
  print("live capture:" + (read() === second));
  let makeNamed = async () => async function reader() { return receiver; };
  let named = await makeNamed();
  print("named capture:" + ((await named()) === second));
}
check();
"#,
        &[
            "7:0",
            "8:0:true",
            "true:true:0",
            "true",
            "true",
            "empty parent:true",
            "live capture:true",
            "named capture:true",
        ],
    );
}

#[test]
fn base_key_and_rhs_suspensions_precede_raw_key_conversion_and_set() {
    assert_trace(
        r#"
async function check() {
  let target = {
    get value() { print("unexpected get"); },
    set value(value) { print("set:" + (this === target) + ":" + value); }
  };
  let spelling = "before";
  let key = {toString() { print("coerce:" + spelling); return "value"; }};
  function base() { print("base"); return target; }
  function name() { print("key"); return key; }
  function rhs() { print("rhs"); spelling = "after"; return 9; }
  let result = ((await base())[await name()] = await rhs());
  print("result:" + result);
}
check().then(function () {
  let release;
  let ready = new Promise(resolve => { release = resolve; });
  let target = {set value(value) { print("unexpected old setter"); }};
  let rawKey = {toString() { print("unexpected old key"); return "value"; }};
  let receiver = target, key = rawKey;
  async function assign() { return (receiver[key] = await ready); }
  let pending = assign();
  receiver = {};
  key = {toString() { print("unexpected replacement key"); return "other"; }};
  rawKey.toString = function () {
    print("live key:" + (this === rawKey));
    return "value";
  };
  Object.defineProperty(target, "value", {
    set(value) { print("live setter:" + (this === target) + ":" + value); }
  });
  release(10);
  pending.then(value => print("live result:" + value));
});
"#,
        &[
            "base",
            "key",
            "rhs",
            "coerce:after",
            "set:true:9",
            "result:9",
            "live key:true",
            "live setter:true:10",
            "live result:10",
        ],
    );
}

#[test]
fn rejected_awaits_skip_later_reference_effects_and_keep_finally() {
    assert_trace(
        r#"
async function check() {
  let marker = {};
  let key = {toString() { print("unexpected key coercion"); return "value"; }};
  let target = {set value(value) { print("unexpected setter"); }};
  try {
    target[key] = await Promise.reject(marker);
  } catch (error) {
    print("rhs:" + (error === marker));
  } finally {
    print("rhs:finally");
  }
  try {
    target[await Promise.reject(marker)] = (print("unexpected rhs"), 1);
  } catch (error) {
    print("key:" + (error === marker));
  }
  try {
    null[key] = await (print("null rhs"), 1);
  } catch (error) {
    print("null:" + (error instanceof TypeError));
  }
  try {
    null[key] = await Promise.reject(marker);
  } catch (error) {
    print("null reject:" + (error === marker));
  }
}
check();
"#,
        &[
            "rhs:true",
            "rhs:finally",
            "key:true",
            "null rhs",
            "null:true",
            "null reject:true",
        ],
    );
}

#[test]
fn resumed_sets_preserve_strictness_symbol_keys_and_setter_receivers() {
    assert_trace(
        r#"
let symbol = Symbol("key");
let marker = {};
let prototype = {set [symbol](value) { print("setter:" + (this === target) + ":" + value); throw marker; }};
let target = Object.create(prototype);
let locked = {};
Object.defineProperty(locked, "value", {value: 1, writable: false});
async function sloppy() {
  let result = (locked.value = await 2);
  print("sloppy:" + result + ":" + locked.value);
}
async function strict() {
  "use strict";
  try {
    locked.value = await 3;
  } catch (error) {
    print("strict:" + (error instanceof TypeError) + ":" + locked.value);
  }
  try {
    target[symbol] = await 4;
  } catch (error) {
    print("throw:" + (error === marker));
  } finally {
    print("setter:finally");
  }
}
sloppy().then(strict);
"#,
        &[
            "sloppy:2:1",
            "strict:true:1",
            "setter:true:4",
            "throw:true",
            "setter:finally",
        ],
    );
}

#[test]
fn interleaved_activations_keep_distinct_base_key_and_value_slots() {
    assert_trace(
        r#"
let releaseFirst, releaseSecond;
let firstReady = new Promise(resolve => { releaseFirst = resolve; });
let secondReady = new Promise(resolve => { releaseSecond = resolve; });
let first = {}, second = {};
async function assign(target, key, ready) {
  return (target[await key] = await ready);
}
let one = assign(first, "one", firstReady);
let two = assign(second, "two", secondReady);
releaseSecond(22);
releaseFirst(11);
Promise.all([one, two]).then(values => {
  print(first.one + ":" + second.two + ":" + values.join(","));
  print((first.two === undefined) + ":" + (second.one === undefined));
});
"#,
        &["11:22:11,22", "true:true"],
    );
}

#[test]
fn async_generator_awaits_use_the_same_property_reference() {
    assert_trace(
        r#"
let first = {value: 0}, second = {value: 0};
async function* sequence() {
  let receiver = first;
  receiver[await (receiver = second, "value")] = await 5;
  yield first.value;
  return second.value;
}
let iterator = sequence();
iterator.next().then(result => print(result.value + ":" + result.done));
iterator.next().then(result => print(result.value + ":" + result.done));
"#,
        &["5:false", "0:true"],
    );
}

#[test]
fn await_using_disposer_captures_cross_empty_activation_and_tdz_environments() {
    assert_trace(
        r#"
async function probe(trace, invalid) {
  try {
    await using registered = {
      get [Symbol.asyncDispose]() {
        trace.push("get");
        try {
          registered;
          trace.push("unexpected initialized resource");
        } catch (error) {
          trace.push("tdz:" + (error instanceof ReferenceError));
        }
        return async () => {
          trace.push("dispose before");
          await 0;
          trace.push("dispose after");
        };
      }
    };
    await using rejected = invalid;
    trace.push("unexpected second resource");
  } catch (error) {
    trace.push("caught:" + (error instanceof TypeError));
  }
}
let trace = [];
probe(trace, 1).then(() => print(trace.join("|")));
"#,
        &["get|tdz:true|dispose before|dispose after|caught:true"],
    );
}

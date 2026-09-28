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
                timeout_ms: Some(60_000),
                ..RunOptions::default()
            },
        )
        .expect("GC argument transport must compile and execute through Wasm AOT");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}: {}\n{source}",
        outcome.completion,
        outcome.note,
    );
    assert_eq!(
        outcome.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).into()))
            .collect::<Vec<_>>(),
        "{source}",
    );
}

#[test]
fn fixed_arguments_preserve_values_before_nested_calls_reuse_temporaries() {
    assert_trace(
        r#"
var value = 1, object = {}, symbol = Symbol('argument');
function change() { value = 3; return 2; }
function inner(a, b) { return a * 10 + b; }
function collect(a, b, c, d, e, f, g, h) {
  print(a, b, c, d, e === object, f === symbol, g === 123n, h, arguments.length);
}
collect(value, change(), value, inner(inner(4, 5), inner(6, 7)), object, symbol, 123n, undefined);
collect();
"#,
        &[
            "1 2 3 517 true true true undefined 8",
            "undefined undefined undefined undefined false false false undefined 0",
        ],
    );
}

#[test]
fn proxy_argument_arrays_remain_observable_and_independent_after_calls() {
    assert_trace(
        r#"
var saved = [], object = {};
function target(a, b) { return a === object && b; }
var proxy = new Proxy(target, {
  apply(fn, that, args) {
    saved.push(args);
    print(Array.isArray(args), args.length, args[0] === object);
    args[1] += 10;
    return Reflect.apply(fn, that, args);
  }
});
print(proxy(object, 1));
print(proxy(object, 2));
saved[0][0] = 'changed';
print(saved[0][1], saved[1][0] === object, saved[1][1], saved[0] !== saved[1]);
"#,
        &["true 2 true", "11", "true 2 true", "12", "11 true 12 true"],
    );
}

#[test]
fn apply_and_spread_observe_getters_and_iterator_steps_once_in_order() {
    assert_trace(
        r#"
var trace = '', prototype = { get 1() { trace += 'one;'; return 8; } };
var source = Object.create(prototype);
Object.defineProperty(source, 'length', { get() { trace += 'length;'; return 3; } });
Object.defineProperty(source, '0', { get() { trace += 'zero;'; return 7; } });
function target(a, b, c) { print(a, b, c, arguments.length); }
Reflect.apply(target, null, source);
print(trace);
var iterable = {
  [Symbol.iterator]() {
    trace += 'iterator;';
    var index = 0;
    return { next() { trace += 'next;'; return { value: ++index, done: index > 2 }; } };
  }
};
trace = '';
target(...iterable, 9);
print(trace);
"#,
        &[
            "7 8 undefined 3",
            "length;zero;one;",
            "1 2 9 3",
            "iterator;next;next;next;",
        ],
    );
}

#[test]
fn bound_constructor_arguments_keep_prefix_and_new_target() {
    assert_trace(
        r#"
var marker = {};
function Constructor(a, b, c) {
  this.ok = a === marker && b === 2 && c === 3 && new.target === Constructor;
}
var Bound = Constructor.bind(null, marker, 2);
var instance = new Bound(3);
print(instance.ok, instance instanceof Constructor);
var saved;
var proxy = new Proxy(Constructor, {
  construct(target, args, newTarget) {
    saved = args;
    return Reflect.construct(target, args, newTarget);
  }
});
var result = Reflect.construct(proxy, [marker, 2, 3], Constructor);
print(result.ok, saved.length, saved[0] === marker, saved[1], saved[2]);
"#,
        &["true true", "true 3 true 2 3"],
    );
}

#[test]
fn generator_and_async_activations_keep_argument_snapshots_across_resumption() {
    assert_trace(
        r#"
var marker = {};
function* generator(first, second) {
  yield first === marker;
  return second + arguments.length;
}
var iterator = generator(marker, 40);
print(iterator.next().value);
print(iterator.next().value);
async function pending(first, second) {
  await 0;
  return first === marker && second + arguments.length;
}
pending(marker, 40).then(function(value) { print('async', value); });
async function* asynchronous(first, second) {
  yield first === marker;
  return second + arguments.length;
}
var asyncIterator = asynchronous(marker, 40);
asyncIterator.next().then(function(result) {
  print('yield', result.value);
  return asyncIterator.next();
}).then(function(result) { print('return', result.value); });
"#,
        &["true", "42", "async 42", "yield true", "return 42"],
    );
}

#[test]
fn abrupt_argument_evaluation_does_not_invoke_the_callee() {
    assert_trace(
        r#"
var marker = {}, calls = 0, order = '';
function target(a, b, c) { calls++; }
function argument(value) { order += value; if (value === 'b') throw marker; return value; }
try { target(argument('a'), argument('b'), argument('c')); }
catch (error) { print(error === marker); }
finally { print(order, calls); }
target(1, 2, 3);
print(calls);
"#,
        &["true", "ab 0", "1"],
    );
}

#[test]
fn repeated_fixed_calls_do_not_exhaust_the_linear_heap_with_argument_arrays() {
    // The previous two-argument carrier used a linear Array record plus slots
    // on every call. This loop exceeds the product's unchanged 1 GiB linear
    // memory limit with that representation, even without observable arrays.
    // An arrow isolates carrier lifetime from ordinary functions' eager
    // arguments objects and mapped parameter environments.
    assert_trace(
        r#"
var add = (left, right) => left + right;
var result = 0;
for (var index = 0; index < 4000000; index++) result = add(result, 1);
print(result);
"#,
        &["4000000"],
    );
}

#[test]
fn hidden_argument_lists_do_not_invoke_array_prototype_setters() {
    assert_trace(
        r#"
var setters = 0, seen = '';
function target(first, second) { seen += first + ':' + second + ';'; }
var arrayLike = { length: 2, 0: 7, 1: 8 };
var iterable = {
  [Symbol.iterator]() {
    var next = 7;
    return { next() { return { value: next, done: next++ > 8 }; } };
  }
};
Object.defineProperty(Array.prototype, '0', {
  configurable: true,
  set(value) { setters++; }
});
try {
  target(...iterable);
  Reflect.apply(target, null, arrayLike);
  target.call(null, 7, 8);
  target.bind(null, 7)(8);
} finally {
  delete Array.prototype[0];
}
print(seen, setters);
"#,
        &["7:8;7:8;7:8;7:8; 0"],
    );
}

#[test]
fn builtin_arguments_remain_rooted_during_reentrant_allocation() {
    // Math.max must still read its later arguments after ToNumber re-enters
    // Wasm and allocates enough short-lived vectors to exercise native GC.
    assert_trace(
        r#"
function add(left, right) { return left + right; }
function churn() {
  var total = 0;
  for (var index = 0; index < 400000; index++) total = add(total, 1);
  return total === 400000 ? 7 : -1;
}
print(Math.max({ valueOf: churn }, 13, 17));
"#,
        &["17"],
    );
}

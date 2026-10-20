use scriptw_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, ObservedJsValue,
    PromiseRejectionPolicy, RealmBuilder, RunOptions,
};

fn assert_wasm_true(source: &str) {
    scriptw_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Arguments concat must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn spreadability_assignment_preserves_raw_symbol_property_and_argument_values() {
    assert_wasm_true(
        r#"
function mapped(a, b) { a = 3; return arguments; }
function unmapped(a, b) { 'use strict'; a = 3; return arguments; }
function duplicates(a, a) { a = 3; return arguments; }
function check(source, expected) {
  source[Symbol.isConcatSpreadable] = 'spread';
  var descriptor = Object.getOwnPropertyDescriptor(source, Symbol.isConcatSpreadable);
  return source[Symbol.isConcatSpreadable] === 'spread' && descriptor.value === 'spread' &&
    descriptor.writable && descriptor.enumerable && descriptor.configurable &&
    [].concat(source, source).join(':') === expected;
}
check(mapped(1, 2), '3:2:3:2') && check(unmapped(1, 2), '1:2:1:2') &&
  check(duplicates(1, 2), '1:3:1:3');
"#,
    );
}

#[test]
fn spreadability_uses_ordinary_descriptors_reflect_set_and_deletion() {
    assert_wasm_true(
        r#"
function source() { return arguments; }
var args = source(4, 5), ok = [].concat(args)[0] === args;
args['Symbol.isConcatSpreadable'] = true;
ok = ok && [].concat(args)[0] === args && args[Symbol.isConcatSpreadable] === undefined;
Object.defineProperty(args, Symbol.isConcatSpreadable, { value: 7, writable: true, configurable: true });
ok = ok && args[Symbol.isConcatSpreadable] === 7 && [].concat(args).join(':') === '4:5';
ok = ok && Reflect.set(args, Symbol.isConcatSpreadable, 0) && args[Symbol.isConcatSpreadable] === 0 &&
  [].concat(args)[0] === args;
Object.defineProperty(args, Symbol.isConcatSpreadable, { writable: false });
ok = ok && !Reflect.set(args, Symbol.isConcatSpreadable, true) && args[Symbol.isConcatSpreadable] === 0;
delete args[Symbol.isConcatSpreadable];
ok && args[Symbol.isConcatSpreadable] === undefined && [].concat(args)[0] === args;
"#,
    );
}

#[test]
fn spreadability_getters_observe_receiver_and_propagate_abrupt_completion() {
    assert_wasm_true(
        r#"
function source() { return arguments; }
var args = source(4, 5), gets = 0, ok = true, marker = {};
var prototype = {};
Object.defineProperty(prototype, Symbol.isConcatSpreadable, { get() {
  gets++; ok = ok && this === args; return true;
} });
Object.setPrototypeOf(args, prototype);
ok = ok && [].concat(args).join(':') === '4:5' && gets === 1;
Object.defineProperty(args, Symbol.isConcatSpreadable, { configurable: true, get() {
  gets++; ok = ok && this === args; throw marker;
} });
try { [].concat(args); ok = false; } catch (error) { ok = ok && error === marker; }
ok && gets === 2;
"#,
    );
}

#[test]
fn spreading_preserves_holes_and_reads_redefined_length_and_index_getters() {
    assert_wasm_true(
        r#"
function source(a, b, c) { return arguments; }
var args = source(4, 5, 6), marker = {}, calls = 0;
args[Symbol.isConcatSpreadable] = true;
delete args[1];
Object.defineProperty(args, 'length', { value: 5 });
var result = [].concat(args), ok = result.length === 5 && result[0] === 4 && result[2] === 6 &&
  !(1 in result) && !(3 in result) && !(4 in result);
Object.defineProperty(args, '0', { get() { calls++; throw marker; } });
try { [].concat(args); ok = false; } catch (error) { ok = ok && error === marker; }
ok && calls === 1;
"#,
    );
}

#[test]
fn arguments_binding_precedes_globals_and_var_hoisting_keeps_its_initial_value() {
    assert_wasm_true(
        r#"
var arguments = 'outer';
function own(value) { return arguments[0] === value && arguments !== 'outer'; }
function hoisted(value) {
  return typeof arguments === 'object' && arguments[0] === value;
  var arguments = 'unreachable';
}
function parameter(arguments) { return arguments === 47; }
function local() { let arguments = 53; return arguments === 53; }
own(41) && hoisted(43) && parameter(47) && local();
"#,
    );
}

#[test]
fn arguments_reassignment_and_destructuring_share_one_live_closure_binding() {
    assert_wasm_true(
        r#"
function shared(value) {
  var read = () => arguments;
  var original = arguments;
  [arguments] = [59];
  if (read() !== 59 || arguments !== 59 || original[0] !== value) return false;
  ({ replacement: arguments } = { replacement: 61 });
  if (read() !== 61) return false;
  var object = { arguments: 67 };
  with (object) { arguments = 71; }
  if (object.arguments !== 71 || read() !== 61) return false;
  arguments = 73;
  return read;
}
var escaped = shared(57);
escaped() === 73;
"#,
    );
}

#[test]
fn async_arguments_targets_keep_the_binding_across_await_and_resumption() {
    scriptw_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_script(
            r#"
var ordinaryDone = false;
var generatorDone = false;
async function ordinary() {
  var read = () => arguments;
  for await ([arguments = 79] of [[]]) {
    if (arguments !== 79 || read() !== 79) throw 'async destructuring wrote another binding';
  }
  ordinaryDone = true;
}
async function* generator() {
  var read = () => arguments;
  for await ([arguments] of [[83]]) {
    yield await 0;
    if (arguments !== 83 || read() !== 83) throw 'generator resumed with old arguments';
  }
  generatorDone = true;
}
ordinary().then(() => {
  if (!ordinaryDone) throw 'async arguments job missing';
  print('ordinary');
});
var iterator = generator();
iterator.next().then(() => iterator.next()).then(() => {
  if (!generatorDone) throw 'generator arguments job missing';
  print('generator');
});
true;
"#,
            CompileOptions {
                promise_rejection_policy: PromiseRejectionPolicy::FailRun,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("async arguments targets run through Wasm and drain all jobs");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert_eq!(
        observed.completion,
        ObservedCompletion::Normal(ObservedJsValue::Boolean(true))
    );
    assert_eq!(observed.output_events.len(), 2);
    for marker in ["ordinary", "generator"] {
        assert!(observed
            .output_events
            .contains(&HostOutputEvent::PrintLine(marker.into())));
    }
}

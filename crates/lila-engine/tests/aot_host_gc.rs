use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_trace(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(60_000),
                ..RunOptions::default()
            },
        )
        .expect("host collection must preserve Wasm AOT execution");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}: {}\n{source}",
        outcome.completion,
        outcome.note
    );
    assert_eq!(
        outcome.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).into()))
            .collect::<Vec<_>>(),
        "{source}"
    );
}

#[test]
fn collection_returns_undefined_without_coercing_unused_arguments() {
    assert_trace(
        r#"
var unused = { valueOf() { throw 'coerced'; }, toString() { throw 'coerced'; } };
print(gc(unused) === undefined, Reflect.apply(gc, null, [unused]) === undefined);
gc(); gc();
print('collected');
"#,
        &["true true", "collected"],
    );
}

#[test]
fn collection_preserves_pending_call_arguments_during_evaluation_and_coercion() {
    assert_trace(
        r#"
var first = { label: 'first' }, last = { label: 'last' };
function take(a, b, c) { 'use strict'; print(a === first, b, c === last, arguments.length); }
take(first, (gc(), 11), last);
print(Math.max({ valueOf() { gc(); return 7; } }, 13, 17));
function* values() { yield first; gc(); yield 19; gc(); yield last; }
take(...values());
"#,
        &["true 11 true 3", "17", "true 19 true 3"],
    );
}

#[test]
fn collection_preserves_deferred_arguments_and_recursive_frames() {
    assert_trace(
        r#"
function inspect(f) { gc(); return Object.getOwnPropertyDescriptor(f, 'arguments').value; }
function owner(value) {
  gc();
  var saved = inspect(owner);
  saved[0] = 23;
  gc();
  print(value, saved === owner.arguments, saved.callee === owner);
  return saved;
}
var saved = owner(5);
gc();
print(saved[0], owner.arguments === null);
function recursive(depth) {
  if (depth) recursive(depth - 1);
  gc();
  print(recursive.arguments[0]);
}
recursive(2);
"#,
        &["23 true true", "23 true", "0", "1", "2"],
    );
}

#[test]
fn collection_preserves_generator_continuations_and_promise_jobs() {
    assert_trace(
        r#"
var marker = { value: 29 };
function* sequence(value) { gc(); yield value; gc(); return value; }
var iterator = sequence(marker);
print(iterator.next().value === marker);
gc();
print(iterator.next().value === marker);
Promise.resolve(marker).then(function(value) { gc(); print(value === marker, value.value); });
gc();
"#,
        &["true", "true", "true 29"],
    );
}

#[test]
fn foreign_realm_collection_preserves_the_calling_realm_and_live_values() {
    assert_trace(
        r#"
var other = __lilaCreateRealm();
other.evalScript('function collect(value) { $262.gc(); return value; }');
var marker = {};
print(other.global.collect(marker) === marker, other.gc() === undefined);
var buffer = new other.global.ArrayBuffer(8);
other.detachArrayBuffer(buffer);
var nested = other.createRealm();
print(buffer.byteLength, nested.gc() === undefined);
gc();
print(Object.getPrototypeOf(marker) === Object.prototype);
"#,
        &["true true", "0 true", "true"],
    );
}

#[test]
fn collection_in_finally_preserves_the_pending_thrown_value() {
    assert_trace(
        r#"
var marker = { value: 31 };
function thrower() { try { throw marker; } finally { gc(); } }
try { thrower(); } catch (error) { gc(); print(error === marker, error.value); }
print(gc() === undefined);
"#,
        &["true 31", "true"],
    );
}

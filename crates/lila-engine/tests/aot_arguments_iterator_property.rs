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
        .expect("Arguments iterator properties must execute through Wasm AOT");
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
fn iterator_is_an_own_intrinsic_even_after_array_prototype_mutation() {
    assert_trace(
        include_str!("fixtures/arguments-iterator-intrinsic.js"),
        &[
            "true true true false true 1",
            "true true false 0",
            "true true true false true 1",
            "true true false 0",
            "true true true false true 1",
            "true true false 0",
            "true",
        ],
    );
}

#[test]
fn iterator_descriptors_and_proxy_invariants_use_the_actual_property() {
    assert_trace(
        r#"
function mapped(value) { return arguments; }
function unmapped(value) { 'use strict'; return arguments; }
function deferred(value) { return deferred.arguments; }
var sources = [mapped(1), unmapped(2), deferred(3)];
for (var i = 0; i < sources.length; i++) {
  var source = sources[i];
  var custom = function () { return 'custom'; };
  Object.defineProperty(source, Symbol.iterator, {value: custom, enumerable: true});
  var d = Object.getOwnPropertyDescriptor(source, Symbol.iterator);
  print(d.value === custom, d.writable, d.enumerable, d.configurable,
    new Proxy(source, {}).hasOwnProperty(Symbol.iterator));
  delete source[Symbol.iterator];
  var receiver = {};
  var gets = 0;
  var proto = {};
  Object.defineProperty(proto, Symbol.iterator, {get() { gets++; return this; }});
  Object.setPrototypeOf(source, proto);
  print(Reflect.get(source, Symbol.iterator, receiver) === receiver, gets,
    Object.getOwnPropertyDescriptor(source, Symbol.iterator) === undefined);
  Object.defineProperty(source, Symbol.iterator, {value: custom, writable: false, configurable: false});
  var getRejected = false;
  try { new Proxy(source, {get() { return undefined; }})[Symbol.iterator]; }
  catch (e) { getRejected = e instanceof TypeError; }
  var ownKeysRejected = false;
  try { Reflect.ownKeys(new Proxy(source, {ownKeys() { return ['0', 'length', 'callee']; }})); }
  catch (e) { ownKeysRejected = e instanceof TypeError; }
  print(getRejected, ownKeysRejected,
    Object.getOwnPropertyDescriptor(new Proxy(source, {}), Symbol.iterator).value === custom);
}
"#,
        &[
            "true true true true true",
            "true 1 true",
            "true true true",
            "true true true true true",
            "true 1 true",
            "true true true",
            "true true true true true",
            "true 1 true",
            "true true true",
        ],
    );
}

#[test]
fn indexed_exotic_own_keys_checks_all_required_keys_and_extensibility() {
    assert_trace(
        r#"
function mapped(value) { return arguments; }
function unmapped(value) { 'use strict'; return arguments; }
function deferred(value) { return deferred.arguments; }
function keysFrom(source, keys) {
  return Reflect.ownKeys(new Proxy(source, {ownKeys() { return keys; }}));
}
function rejects(source, keys) {
  try { keysFrom(source, keys); }
  catch (error) { return error instanceof TypeError; }
  return false;
}
function without(keys, omitted) {
  return keys.filter(function(key) { return key !== omitted; });
}
var sources = [[1], mapped(2), unmapped(3), deferred(4)];
for (var i = 0; i < sources.length; i++) {
  var source = sources[i];
  var keys = Reflect.ownKeys(source);
  print(rejects(source, without(keys, 'length')) === (i === 0));
  Object.defineProperty(source, '0', {configurable: false});
  var symbol = Symbol('required');
  Object.defineProperty(source, symbol, {get() { throw 'must not read'; }});
  Object.defineProperty(source, 'named', {value: 42});
  keys = Reflect.ownKeys(source);
  print(rejects(source, without(keys, '0')),
    rejects(source, without(keys, symbol)), rejects(source, without(keys, 'named')));
  print(keysFrom(source, keys.concat(['extra'])).length === keys.length + 1);
  if (i !== 0) delete source.length;
  Object.preventExtensions(source);
  keys = Reflect.ownKeys(source);
  print(keysFrom(source, keys.slice().reverse()).length === keys.length,
    rejects(source, keys.concat(['extra'])));
  var removed = keys[0];
  print(rejects(source, without(keys, removed)),
    rejects(source, without(keys, removed).concat(['extra'])));
}
"#,
        &[
            "true",
            "true true true",
            "true",
            "true true",
            "true true",
            "true",
            "true true true",
            "true",
            "true true",
            "true true",
            "true",
            "true true true",
            "true",
            "true true",
            "true true",
            "true",
            "true true true",
            "true",
            "true true",
            "true true",
        ],
    );
}

#[test]
fn eager_and_deferred_iterators_belong_to_the_defining_realm() {
    assert_trace(
        r#"
var other = __lilaCreateRealm();
other.evalScript('function eager(value) { return arguments; } function lazy(value, callback) { return callback(lazy); }');
var intrinsic = other.global.Array.prototype.values;
other.global.Array.prototype.values = function() {};
other.global.Array.prototype[Symbol.iterator] = function() {};
var eager = other.global.eager(7);
var lazy = other.global.lazy(11, function(f) { return f.arguments; });
print(eager[Symbol.iterator] === intrinsic, lazy[Symbol.iterator] === intrinsic,
  intrinsic !== Array.prototype.values);
print(Object.getOwnPropertyDescriptor(eager, Symbol.iterator).value === intrinsic,
  Object.getOwnPropertyDescriptor(lazy, Symbol.iterator).value === intrinsic);
var iterator = lazy[Symbol.iterator]();
print(iterator.next().value, Object.getPrototypeOf(iterator) ===
  Object.getPrototypeOf(intrinsic.call([])));
"#,
        &["true true true", "true true", "11 true"],
    );
}

#[test]
fn the_string_symbol_iterator_is_not_the_well_known_symbol() {
    assert_trace(
        r#"
function check(value) {
  var intrinsic = arguments[Symbol.iterator];
  print(arguments['Symbol.iterator'] === undefined);
  arguments['Symbol.iterator'] = 17;
  print(arguments['Symbol.iterator'], arguments[Symbol.iterator] === intrinsic);
  delete arguments['Symbol.iterator'];
  print(arguments['Symbol.iterator'] === undefined, arguments[Symbol.iterator] === intrinsic);
}
check(1);
"#,
        &["true", "17 true", "true true"],
    );
}

//! 6.2.6.4 FromPropertyDescriptor creates every field with
//! CreateDataPropertyOrThrow: a writable, enumerable, configurable data
//! property, in the order value, writable, get, set, enumerable, configurable.
//!
//! Descriptor objects used to carry builtin (non-enumerable) fields, so
//! `Object.keys`, `for-in` and `JSON.stringify` saw an empty object.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

/// Checks one descriptor object's own keys, enumeration, field attributes,
/// prototype and (optionally) JSON text.
const CHECK: &str = r#"
function check(label, descriptor, keys, json) {
  if (typeof descriptor !== 'object' || descriptor === null) throw label + ': not an object';
  if (Object.getPrototypeOf(descriptor) !== Object.prototype) throw label + ': prototype';
  if (Reflect.ownKeys(descriptor).join() !== keys) throw label + ': own keys ' + Reflect.ownKeys(descriptor).join();
  if (Object.keys(descriptor).join() !== keys) throw label + ': Object.keys ' + Object.keys(descriptor).join();
  var enumerated = [];
  for (var key in descriptor) enumerated.push(key);
  if (enumerated.join() !== keys) throw label + ': for-in ' + enumerated.join();
  var fields = keys.split(',');
  for (var index = 0; index < fields.length; index++) {
    var field = Object.getOwnPropertyDescriptor(descriptor, fields[index]);
    if (!field.writable || !field.enumerable || !field.configurable) throw label + ': attributes of ' + fields[index];
    if (!Object.prototype.propertyIsEnumerable.call(descriptor, fields[index])) throw label + ': propertyIsEnumerable ' + fields[index];
  }
  if (json !== undefined && JSON.stringify(descriptor) !== json) throw label + ': JSON ' + JSON.stringify(descriptor);
}
var DATA = 'value,writable,enumerable,configurable';
var ACCESSOR = 'get,set,enumerable,configurable';
"#;

fn observe_script(body: &str) -> Vec<HostOutputEvent> {
    let source = format!("{CHECK}{body}");
    lila_engine::configure_compilation_jobs(1).expect("one bounded compiler worker");
    let observation = Engine::new(RealmBuilder::new().build())
        .observe_script(
            &source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("descriptor fixture compiles and executes through Wasm AOT");
    assert_eq!(observation.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion
    );
    observation.output_events
}

fn assert_prints(body: &str, expected: &[&str]) {
    assert_eq!(
        observe_script(body),
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).into()))
            .collect::<Vec<_>>(),
        "{body}"
    );
}

#[test]
fn object_get_own_property_descriptor_fields_are_enumerable_data_properties() {
    assert_prints(
        r#"
var d = Object.getOwnPropertyDescriptor({x: 1}, 'x');
print(Object.keys(d).join(','), JSON.stringify(d));
var a = Object.getOwnPropertyDescriptor({get y() { return 1; }}, 'y');
print(Object.keys(a).join(','));
for (var k in d) print('in', k);
print(Object.getOwnPropertyDescriptor(d, 'value').enumerable);
"#,
        &[
            "value,writable,enumerable,configurable {\"value\":1,\"writable\":true,\"enumerable\":true,\"configurable\":true}",
            "get,set,enumerable,configurable",
            "in value",
            "in writable",
            "in enumerable",
            "in configurable",
            "true",
        ],
    );
}

#[test]
fn every_object_get_own_property_descriptor_representation_publishes_enumerable_fields() {
    assert_prints(
        r#"
var symbol = Symbol('key');
var ordinary = {x: 1, [symbol]: 's'};
Object.defineProperty(ordinary, 'hidden', {value: 2});
Object.defineProperty(ordinary, 'setterOnly', {set: function(v) {}, enumerable: true});
check('ordinary data', Object.getOwnPropertyDescriptor(ordinary, 'x'), DATA,
  '{"value":1,"writable":true,"enumerable":true,"configurable":true}');
check('symbol key', Object.getOwnPropertyDescriptor(ordinary, symbol), DATA,
  '{"value":"s","writable":true,"enumerable":true,"configurable":true}');
check('frozen attributes', Object.getOwnPropertyDescriptor(ordinary, 'hidden'), DATA,
  '{"value":2,"writable":false,"enumerable":false,"configurable":false}');
check('accessor', Object.getOwnPropertyDescriptor({get y() { return 1; }}, 'y'), ACCESSOR,
  '{"enumerable":true,"configurable":true}');
var setterOnly = Object.getOwnPropertyDescriptor(ordinary, 'setterOnly');
check('setter only', setterOnly, ACCESSOR, '{"enumerable":true,"configurable":false}');
if (setterOnly.get !== undefined || typeof setterOnly.set !== 'function') throw 'setter only fields';
check('builtin method', Object.getOwnPropertyDescriptor(Array.prototype, 'map'), DATA,
  '{"writable":true,"enumerable":false,"configurable":true}');
check('array index', Object.getOwnPropertyDescriptor([5], '0'), DATA,
  '{"value":5,"writable":true,"enumerable":true,"configurable":true}');
check('array length', Object.getOwnPropertyDescriptor([1, 2], 'length'), DATA,
  '{"value":2,"writable":true,"enumerable":false,"configurable":false}');
var array = [];
array.named = 'n';
Object.defineProperty(array, 'accessor', {get: function() { return 1; }, configurable: true});
check('array named data', Object.getOwnPropertyDescriptor(array, 'named'), DATA,
  '{"value":"n","writable":true,"enumerable":true,"configurable":true}');
check('array named accessor', Object.getOwnPropertyDescriptor(array, 'accessor'), ACCESSOR,
  '{"enumerable":false,"configurable":true}');
var indexedAccessor = [];
Object.defineProperty(indexedAccessor, '0', {get: function() { return 1; }, enumerable: true});
check('array index accessor', Object.getOwnPropertyDescriptor(indexedAccessor, '0'), ACCESSOR,
  '{"enumerable":true,"configurable":false}');
check('string index', Object.getOwnPropertyDescriptor('ab', '1'), DATA,
  '{"value":"b","writable":false,"enumerable":true,"configurable":false}');
check('string length', Object.getOwnPropertyDescriptor('ab', 'length'), DATA,
  '{"value":2,"writable":false,"enumerable":false,"configurable":false}');
check('String object index', Object.getOwnPropertyDescriptor(new String('ab'), '0'), DATA,
  '{"value":"a","writable":false,"enumerable":true,"configurable":false}');
check('String object length', Object.getOwnPropertyDescriptor(new String('ab'), 'length'), DATA,
  '{"value":2,"writable":false,"enumerable":false,"configurable":false}');
check('typed array index', Object.getOwnPropertyDescriptor(new Uint8Array([3]), '0'), DATA,
  '{"value":3,"writable":true,"enumerable":true,"configurable":true}');
check('function prototype', Object.getOwnPropertyDescriptor(function() {}, 'prototype'), DATA,
  '{"value":{},"writable":true,"enumerable":false,"configurable":false}');
(function() {
  check('arguments index', Object.getOwnPropertyDescriptor(arguments, '0'), DATA,
    '{"value":7,"writable":true,"enumerable":true,"configurable":true}');
  check('arguments length', Object.getOwnPropertyDescriptor(arguments, 'length'), DATA,
    '{"value":1,"writable":true,"enumerable":false,"configurable":true}');
  check('arguments callee', Object.getOwnPropertyDescriptor(arguments, 'callee'), DATA,
    '{"writable":true,"enumerable":false,"configurable":true}');
})(7);
(function() {
  'use strict';
  check('strict arguments callee', Object.getOwnPropertyDescriptor(arguments, 'callee'), ACCESSOR,
    '{"enumerable":false,"configurable":false}');
})();
print('ok');
"#,
        &["ok"],
    );
}

#[test]
fn reflect_and_bulk_descriptor_producers_publish_enumerable_fields() {
    assert_prints(
        r#"
check('Reflect data', Reflect.getOwnPropertyDescriptor({x: 1}, 'x'), DATA,
  '{"value":1,"writable":true,"enumerable":true,"configurable":true}');
check('Reflect accessor', Reflect.getOwnPropertyDescriptor({set z(v) {}}, 'z'), ACCESSOR,
  '{"enumerable":true,"configurable":true}');
var all = Object.getOwnPropertyDescriptors({x: 1, get y() { return 2; }});
if (Object.keys(all).join() !== 'x,y') throw 'descriptor table keys';
check('descriptors data', all.x, DATA, '{"value":1,"writable":true,"enumerable":true,"configurable":true}');
check('descriptors accessor', all.y, ACCESSOR, '{"enumerable":true,"configurable":true}');
if (JSON.stringify(Object.getOwnPropertyDescriptors({a: 'A'})) !==
    '{"a":{"value":"A","writable":true,"enumerable":true,"configurable":true}}') throw 'descriptor table JSON';
var copy = Object.assign({}, Object.getOwnPropertyDescriptor({x: 9}, 'x'));
if (JSON.stringify(copy) !== '{"value":9,"writable":true,"enumerable":true,"configurable":true}') throw 'Object.assign copy';
var spread = {...Object.getOwnPropertyDescriptor({x: 8}, 'x')};
if (Object.keys(spread).join() !== DATA) throw 'spread copy';
var roundTrip = Object.defineProperties({}, Object.getOwnPropertyDescriptors({p: 1, get q() { return 2; }}));
if (roundTrip.p !== 1 || roundTrip.q !== 2 || Object.keys(roundTrip).join() !== 'p,q') throw 'round trip';
print('ok');
"#,
        &["ok"],
    );
}

#[test]
fn proxy_trap_descriptor_arguments_publish_enumerable_fields_in_order() {
    assert_prints(
        r#"
var seen = [];
function recorder(target) {
  return new Proxy(target, {
    defineProperty(t, key, descriptor) {
      seen.push([String(key), descriptor]);
      return Reflect.defineProperty(t, key, descriptor);
    }
  });
}
function take(key) {
  for (var index = 0; index < seen.length; index++) {
    if (seen[index][0] === key) {
      var descriptor = seen[index][1];
      seen.splice(index, 1);
      return descriptor;
    }
  }
  throw 'no trap call for ' + key;
}

var getter = function() { return 1; };
var viaObject = recorder({});
Object.defineProperty(viaObject, 'partial', {enumerable: 1, value: 3});
check('Object.defineProperty partial', take('partial'), 'value,enumerable', '{"value":3,"enumerable":true}');
Object.defineProperty(viaObject, 'accessor', {configurable: false, get: getter});
check('Object.defineProperty accessor', take('accessor'), 'get,configurable', '{"configurable":false}');
Object.defineProperties(viaObject, {many: {writable: false, value: 'm', configurable: true}});
check('Object.defineProperties', take('many'), 'value,writable,configurable',
  '{"value":"m","writable":false,"configurable":true}');

var viaReflect = recorder({});
Reflect.defineProperty(viaReflect, 'r', {set: getter, get: undefined, enumerable: false});
check('Reflect.defineProperty', take('r'), 'get,set,enumerable', '{"enumerable":false}');

var receiver = recorder({});
Reflect.set({}, 'created', 1, receiver);
check('OrdinarySet CreateDataProperty', take('created'), DATA,
  '{"value":1,"writable":true,"enumerable":true,"configurable":true}');
Reflect.set({}, 'created', 2, receiver);
check('OrdinarySet existing receiver property', take('created'), 'value', '{"value":2}');

var fielded = recorder({});
class Base { constructor() { return fielded; } }
class Derived extends Base { field = 'f'; }
new Derived();
check('DefineField', take('field'), DATA,
  '{"value":"f","writable":true,"enumerable":true,"configurable":true}');

var legacy = recorder({});
legacy.__defineGetter__('legacyGet', getter);
check('__defineGetter__', take('legacyGet'), ACCESSOR.replace('set,', ''), '{"enumerable":true,"configurable":true}');

var sealed = recorder({s: 1});
Object.seal(sealed);
check('Object.seal', take('s'), 'configurable', '{"configurable":false}');
var frozen = recorder({f: 1});
Object.freeze(frozen);
check('Object.freeze', take('f'), 'writable,configurable', '{"writable":false,"configurable":false}');
if (seen.length !== 0) throw 'unexpected trap calls: ' + seen.map(function(entry) { return entry[0]; }).join();
print('ok');
"#,
        &["ok"],
    );
}

#[test]
fn engine_private_descriptor_arguments_do_not_inherit_object_prototype_fields() {
    // 6.2.6.5 ToPropertyDescriptor uses HasProperty, so an internal
    // descriptor argument with %Object.prototype% would pick up an inherited
    // `get` next to its own `value` and throw. Internal CreateDataProperty and
    // [[DefineOwnProperty]] requests must not observe Object.prototype at all.
    // The handlers have null prototypes: an inherited `get` on an ordinary
    // handler *is* the Proxy [[Get]] trap.
    assert_prints(
        r#"
Object.defineProperty(Object.prototype, 'get', {
  value: function() { throw 'inherited get was consulted'; },
  writable: true,
  configurable: true
});
try {
  var plain = new Proxy({}, Object.create(null));
  Reflect.set({}, 'p', 1, plain);
  Reflect.set({}, 'p', 2, plain);
  class Base { constructor() { return plain; } }
  class Derived extends Base { field = 3; }
  new Derived();
  var sealed = new Proxy({s: 1}, Object.create(null));
  Object.seal(sealed);
  var frozen = new Proxy({f: 1}, Object.create(null));
  Object.freeze(frozen);
  plain.__defineSetter__('legacySet', function(v) {});
  if (plain.p !== 2 || plain.field !== 3) throw 'definitions';
  if (Object.getOwnPropertyDescriptor(sealed, 's').configurable !== false) throw 'seal';
  if (Object.getOwnPropertyDescriptor(frozen, 'f').writable !== false) throw 'freeze';
  if (typeof Object.getOwnPropertyDescriptor(plain, 'legacySet').set !== 'function') throw 'legacy setter';
  print('ok');
} finally {
  delete Object.prototype.get;
}
"#,
        &["ok"],
    );
}

struct NamespaceModules(PathBuf);

impl Drop for NamespaceModules {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn module_namespace_descriptors_publish_enumerable_fields() {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let fixture = NamespaceModules(std::env::temp_dir().join(format!(
        "lila-namespace-descriptor-fields-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    std::fs::create_dir_all(&fixture.0).expect("create module fixture");
    let entry = format!(
        "{CHECK}{}",
        r#"
import * as ns from './values.js';
check('namespace binding', Object.getOwnPropertyDescriptor(ns, 'answer'), DATA,
  '{"value":42,"writable":true,"enumerable":true,"configurable":false}');
check('namespace Reflect', Reflect.getOwnPropertyDescriptor(ns, 'answer'), DATA,
  '{"value":42,"writable":true,"enumerable":true,"configurable":false}');
check('namespace toStringTag', Object.getOwnPropertyDescriptor(ns, Symbol.toStringTag), DATA,
  '{"value":"Module","writable":false,"enumerable":false,"configurable":false}');
check('namespace table', Object.getOwnPropertyDescriptors(ns).answer, DATA);
print('ok');
"#
    );
    for (name, source) in [
        ("entry.js", entry.as_str()),
        ("values.js", "export const answer = 42;\n"),
    ] {
        std::fs::write(fixture.0.join(name), source).expect("write module fixture");
    }
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_module(
            &entry,
            CompileOptions {
                filename: Some(fixture.0.join("entry.js").to_str().unwrap().into()),
                module_root: Some(fixture.0.to_str().unwrap().into()),
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("namespace descriptor fixture compiles and executes through Wasm");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        observed.completion
    );
    assert_eq!(
        observed.output_events,
        vec![HostOutputEvent::PrintLine("ok".into())]
    );
}

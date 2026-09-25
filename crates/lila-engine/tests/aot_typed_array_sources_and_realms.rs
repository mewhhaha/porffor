//! Primitive source property lookup and realm-owned binary-data intrinsics.
use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_wasm(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observation = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
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
        .expect("binary-data source/realm regression must compile and execute");
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        observation.completion
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine("true".to_string())]
    );
}

#[test]
fn typed_array_from_reads_primitive_iterators_and_arraylike_strings() {
    assert_wasm(
        r#"
function same(actual, expected, label) {
  if (actual !== expected) throw label + ': ' + actual + ' !== ' + expected;
}
var from = Uint32Array.from.bind(Uint32Array);
var code = s => s.codePointAt(0);
same(from('123').join(), '1,2,3', 'numeric string');
same(from('\ud834\udd1e G', code).join(), '119070,32,71', 'code point iteration');
var hits = 0, receiver = {};
from('abc', function () { 'use strict'; same(this, receiver, 'mapper receiver'); hits++; return 0; }, receiver);
same(hits, 3, 'mapper calls');
var saved = String.prototype[Symbol.iterator];
Object.defineProperty(String.prototype, Symbol.iterator, {
  configurable: true,
  get: function () {
    'use strict';
    same(this, 'source', 'GetMethod receiver');
    return function () { return [7, 8][Symbol.iterator](); };
  }
});
same(from('source').join(), '7,8', 'overridden iterator');
delete String.prototype[Symbol.iterator];
same(from('\ud834\udd1e', code).join(), '55348,56606', 'arraylike UTF16');
Object.defineProperty(String.prototype, Symbol.iterator, { value: saved, writable: true, configurable: true });
for (var primitive of [true, 3, 4n, Symbol()]) same(from(primitive).length, 0, 'empty primitive');
Object.defineProperty(Boolean.prototype, Symbol.iterator, {
  configurable: true,
  get: function () { 'use strict'; same(this, true, 'boolean receiver'); return 1; }
});
var caught = false;
try { from(true); } catch (error) { caught = error instanceof TypeError; }
if (!caught) throw 'noncallable primitive iterator';
print(true);
"#,
    );
}

#[test]
fn array_from_boxes_arraylike_primitives_and_snapshots_array_length() {
    assert_wasm(
        r#"
function check(value, label) { if (!value) throw label; }
function exercise(value, prototype, label) {
  var reads = '';
  function fromWrapper(key, result) {
    return function () {
      'use strict';
      check(typeof this === 'object' && Object.getPrototypeOf(this) === prototype,
            label + ' getter receiver');
      check(this.valueOf() === value, label + ' boxed value');
      reads += key;
      return result;
    };
  }
  Object.defineProperty(prototype, 'length', {
    configurable: true, get: fromWrapper('L', 2)
  });
  Object.defineProperty(prototype, '0', {
    configurable: true, get: fromWrapper('0', 17)
  });
  Object.defineProperty(prototype, '1', {
    configurable: true, get: fromWrapper('1', 19)
  });
  var result = Array.from(value);
  check(result.length === 2 && result[0] === 17 && result[1] === 19,
        label + ' elements');
  check(reads === 'L01', label + ' read order');
  delete prototype.length;
  delete prototype[0];
  delete prototype[1];
}
exercise(7, Number.prototype, 'number');
exercise(true, Boolean.prototype, 'boolean');
exercise(4n, BigInt.prototype, 'bigint');
exercise(Symbol('source'), Symbol.prototype, 'symbol');

var source = [1, 2, 3], calls = 0;
Object.setPrototypeOf(source, { 1: 42 });
var copy = Array.from(source, function (value, index) {
  calls++;
  if (index === 0) source.length = 1;
  return value;
});
check(calls === 3, 'array-like length snapshot');
check(copy.length === 3 && copy[0] === 1 && copy[1] === 42 && copy[2] === undefined,
      'array-like prototype and undefined tail');
print(true);
"#,
    );
}

#[test]
fn created_realms_publish_typed_array_statics_and_their_host_record() {
    assert_wasm(
        r#"
function check(value, label) { if (!value) throw label; }
var realm = __lilaCreateRealm(), other = realm.global;
check(other.$262 === realm && realm.global === other, 'host record identity');
check(typeof realm.createRealm === 'function' && typeof realm.gc === 'function', 'host members');
for (var name of ['Uint8Array', 'Int16Array', 'Float64Array']) {
  var C = other[name], home = globalThis[name];
  var copied = C.from([1, 2, 3]), made = C.of(4, 5);
  check(Object.getPrototypeOf(copied) === C.prototype && !(copied instanceof home), 'from realm');
  check(Object.getPrototypeOf(made) === C.prototype && made.join() === '4,5', 'of realm');
  check(home.from.call(C, [6])[0] === 6, 'foreign constructor from');
  check(C.of.call(home, 7) instanceof home, 'foreign constructor of');
  var target = new home(3), source = C.of(8, 9);
  Object.defineProperty(source, 'length', { get: function () { throw 'source length'; } });
  target.set(source, 1);
  check(target.join() === '0,8,9', 'cross realm set');
  realm.detachArrayBuffer(source.buffer);
  var threw = false;
  try { target.set(source); } catch (error) { threw = error instanceof TypeError; }
  check(threw, 'detached source');
}
var buffer = new other.ArrayBuffer(2), view = new Uint8Array(buffer);
var iterator = view.values();
other.$262.detachArrayBuffer(buffer);
var detached = false;
try { iterator.next(); } catch (error) { detached = error instanceof TypeError; }
check(detached, 'iterator detects cross realm detachment');
var nested = realm.createRealm();
check(nested.global.$262 === nested && nested.global !== other, 'nested realm host record');
print(true);
"#,
    );
}

#[test]
fn dataview_missing_buffer_is_a_catchable_runtime_type_error() {
    assert_wasm(
        r#"
var caught = false;
try { new DataView(); } catch (error) { caught = error instanceof TypeError; }
if (!caught) throw 'missing buffer must throw TypeError';
if (false) new DataView();
print(true);
"#,
    );
}

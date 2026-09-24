//! Immutable ArrayBuffers (proposal-immutable-arraybuffer) through Wasm AOT.
//!
//! The backing buffer's `Immutable` flag is consulted by one predicate on
//! every write path: ValidateTypedArray with `write`, the integer-indexed
//! `[[Set]]`/`[[DefineOwnProperty]]`/`[[GetOwnProperty]]`, SetViewValue,
//! Atomics read-modify-write, DetachArrayBuffer and ArrayBufferCopyAndDetach.

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_prints_true(source: &str) {
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
                timeout_ms: Some(60_000),
                ..RunOptions::default()
            },
        )
        .expect("immutable ArrayBuffer regression must execute through Wasm AOT");
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion,
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine("true".to_string())],
        "{source}",
    );
}

#[test]
fn immutable_buffers_report_their_state_and_refuse_detachment_and_resizing() {
    assert_prints_true(
        r#"
function throwsTypeError(f) {
  try { f(); } catch (error) { return error instanceof TypeError; }
  return false;
}
var getter = Object.getOwnPropertyDescriptor(ArrayBuffer.prototype, 'immutable').get;
if (typeof getter !== 'function' || getter.name !== 'get immutable') throw 'getter identity';
if (!throwsTypeError(function() { getter.call({}); })) throw 'getter brand';
if (!throwsTypeError(function() { getter.call(new SharedArrayBuffer(1)); })) throw 'getter shared';

var source = new ArrayBuffer(4);
new Uint8Array(source).set([1, 2, 3, 4]);
if (source.immutable !== false) throw 'mutable buffer';
var buffer = source.transferToImmutable(6);
if (!source.detached || buffer.immutable !== true) throw 'transfer state';
if (buffer.byteLength !== 6 || buffer.maxByteLength !== 6 || buffer.resizable) throw 'fixed length';
var bytes = new Uint8Array(buffer);
if (bytes.join() !== '1,2,3,4,0,0') throw 'copied contents';

var coercions = 0;
var length = { valueOf() { coercions++; return 1; } };
for (var method of ['transfer', 'transferToFixedLength', 'transferToImmutable']) {
  if (!throwsTypeError(function() { buffer[method](length); })) throw method;
}
if (coercions !== 3) throw 'ToIndex precedes the immutable rejection';
if (!throwsTypeError(function() { buffer.resize(1); })) throw 'resize';
if (!throwsTypeError(function() { __lilaDetachArrayBuffer(buffer); })) throw 'detach';
if (buffer.detached || buffer.byteLength !== 6) throw 'still attached';

var slice = buffer.sliceToImmutable(1, 3);
if (!slice.immutable || new Uint8Array(slice).join() !== '2,3') throw 'sliceToImmutable';
var copy = buffer.slice(1, 3);
if (copy.immutable || new Uint8Array(copy).join() !== '2,3') throw 'slice copies mutably';
print(true);
"#,
    );
}

#[test]
fn integer_indexed_set_and_define_reject_writes_without_coercing() {
    assert_prints_true(
        r#"
function throwsTypeError(f) {
  try { f(); } catch (error) { return error instanceof TypeError; }
  return false;
}
var ta = new Int16Array(new Int16Array([5, 6]).buffer.transferToImmutable());
var coercions = 0;
var value = { valueOf() { coercions++; return 9; } };

(function() { ta[0] = value; ta[7] = value; ta['-0'] = value; })();
if (!throwsTypeError(function() { 'use strict'; ta[0] = value; })) throw 'strict index';
if (!throwsTypeError(function() { 'use strict'; ta[7] = value; })) throw 'strict out of bounds';
if (!throwsTypeError(function() { 'use strict'; ta['1.5'] = value; })) throw 'strict numeric string';
if (Reflect.set(ta, '0', value) !== false) throw 'Reflect.set same receiver';
if (Reflect.set(ta, '0', value, {}) !== false) throw 'Reflect.set other receiver';
if (Reflect.set({}, '0', value, ta) !== false) throw 'receiver element is non-writable';
var inheritor = Object.create(ta);
if (!throwsTypeError(function() { 'use strict'; inheritor[0] = value; })) throw 'inherited';
if (inheritor.hasOwnProperty('0')) throw 'inherited write created a property';
if (coercions !== 0) throw 'rejected writes must not coerce';
if (ta[0] !== 5 || ta[1] !== 6) throw 'contents changed';

var descriptor = Object.getOwnPropertyDescriptor(ta, '1');
if (descriptor.value !== 6 || descriptor.writable || !descriptor.enumerable ||
    descriptor.configurable) throw 'element descriptor';
if (Reflect.defineProperty(ta, '1', { value: 6 }) !== true) throw 'same value';
if (Reflect.defineProperty(ta, '1', { value: 6, writable: false, enumerable: true,
    configurable: false }) !== true) throw 'identical descriptor';
if (Reflect.defineProperty(ta, '1', { value: 7 }) !== false) throw 'different value';
if (Reflect.defineProperty(ta, '1', { value: '6' }) !== false) throw 'no coercion';
if (Reflect.defineProperty(ta, '1', { writable: true }) !== false) throw 'writable';
if (Reflect.defineProperty(ta, '1', { get() {} }) !== false) throw 'accessor';
if (Reflect.defineProperty(ta, '2', { value: 6 }) !== false) throw 'invalid index';
if (!throwsTypeError(function() { Object.defineProperty(ta, '0', { value: 1 }); })) throw 'throw';
if (Object.freeze(ta) !== ta || !Object.isFrozen(ta)) throw 'freeze';

var mutable = new Int16Array(2);
mutable[0] = value;
if (mutable[0] !== 9 || coercions !== 1) throw 'mutable arrays still coerce and store';
print(true);
"#,
    );
}

#[test]
fn typed_array_writers_reject_immutable_receivers_and_species_results() {
    assert_prints_true(
        r#"
function throwsTypeError(f) {
  try { f(); } catch (error) { return error instanceof TypeError; }
  return false;
}
var coercions = 0;
var arg = { valueOf() { coercions++; return 0; } };
var ta = new Uint8Array(new Uint8Array([3, 1, 2]).buffer.transferToImmutable());
if (!throwsTypeError(function() { ta.fill(arg); })) throw 'fill';
if (!throwsTypeError(function() { ta.copyWithin(arg, arg); })) throw 'copyWithin';
if (!throwsTypeError(function() { ta.set([1], arg); })) throw 'set';
if (!throwsTypeError(function() { ta.reverse(); })) throw 'reverse';
if (!throwsTypeError(function() { ta.sort(); })) throw 'sort';
if (!throwsTypeError(function() { new Uint8Array(ta.buffer, 3).fill(1); })) throw 'empty fill';
if (!throwsTypeError(function() { Array.prototype.reverse.call(ta); })) throw 'Array reverse';
if (!throwsTypeError(function() { Array.prototype.fill.call(ta, 0); })) throw 'Array fill';
if (!throwsTypeError(function() { Array.prototype.copyWithin.call(ta, 0, 1); })) throw 'Array copyWithin';
if (!throwsTypeError(function() { Array.prototype.sort.call(ta); })) throw 'Array sort';
if (coercions !== 0) throw 'arguments coerced before the immutable rejection';
if (ta.join() !== '3,1,2') throw 'contents changed';

if (ta.slice(1).join() !== '1,2' || ta.map(function(x) { return x * 2; }).join() !== '6,2,4' ||
    ta.toSorted().join() !== '1,2,3' || ta.with(0, 9).join() !== '9,1,2' ||
    ta.indexOf(2) !== 2 || ta.subarray(1).buffer !== ta.buffer) throw 'readers';
var copy = new Uint8Array(ta);
copy[0] = 7;
if (copy[0] !== 7 || copy.buffer.immutable) throw 'copies are mutable';

var immutableResult = new Uint8Array(new ArrayBuffer(3).transferToImmutable());
ta.constructor = {};
ta.constructor[Symbol.species] = function() { return immutableResult; };
for (var method of ['map', 'filter', 'slice']) {
  if (!throwsTypeError(function() { ta[method](function() { return true; }); })) throw method;
}
var ctor = function() { return immutableResult; };
if (!throwsTypeError(function() { Uint8Array.from.call(ctor, [1]); })) throw 'from';
if (!throwsTypeError(function() { Uint8Array.of.call(ctor, 1); })) throw 'of';
print(true);
"#,
    );
}

#[test]
fn data_view_and_atomics_writers_reject_immutable_buffers_before_coercion() {
    assert_prints_true(
        r#"
function throwsTypeError(f) {
  try { f(); } catch (error) { return error instanceof TypeError; }
  return false;
}
var coercions = 0;
var arg = { valueOf() { coercions++; return 0; } };
var buffer = new Int32Array([7, 8]).buffer.transferToImmutable();
var view = new DataView(buffer);
if (!throwsTypeError(function() { view.setInt8(arg, arg); })) throw 'setInt8';
if (!throwsTypeError(function() { view.setFloat64(arg, arg); })) throw 'setFloat64';
if (!throwsTypeError(function() { view.setBigInt64(arg, 1n); })) throw 'setBigInt64';
if (view.getInt32(0, true) !== 7) throw 'getter reads';

var ta = new Int32Array(buffer);
for (var name of ['add', 'and', 'exchange', 'or', 'store', 'sub', 'xor']) {
  if (!throwsTypeError(function() { Atomics[name](ta, arg, arg); })) throw name;
}
if (!throwsTypeError(function() { Atomics.compareExchange(ta, arg, arg, arg); })) throw 'cmpxchg';
if (coercions !== 0) throw 'coerced before the immutable rejection';
if (Atomics.load(ta, 1) !== 8) throw 'load reads';
if (Atomics.notify(ta, arg, arg) !== 0 || coercions !== 2) throw 'notify reads';
print(true);
"#,
    );
}

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_binary_modes(source: &str, line: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
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
            .expect("finite BinaryData native control compiles and executes through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{source}"
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(line.into())],
            "{source}"
        );
    }
}

#[test]
fn gc_buffers_preserve_constructor_phases_resize_and_species_copy() {
    assert_binary_modes(
        r#"
function check(ok, label) { if (!ok) throw new Error(label); }
var trace = [];
var prototype = {};
var target = new Proxy(function Target() {}, { get: function (object, key) {
  if (key === 'prototype') { trace.push('prototype'); return prototype; }
  return object[key];
} });
var length = { valueOf: function () { trace.push('length'); return 4; } };
var options = { get maxByteLength() {
  trace.push('maximum:get'); return { valueOf: function () { trace.push('maximum:value'); return 12; } };
} };
var buffer = Reflect.construct(ArrayBuffer, [length, options], target);
check(Object.getPrototypeOf(buffer) === prototype && trace.join(',') === 'length,maximum:get,maximum:value,prototype', 'constructor phase order');
var rab = new ArrayBuffer(8, { maxByteLength: 16 });
var tracking = new Uint8Array(rab);
var fixed = new Uint8Array(rab, 2, 4);
tracking[0] = 9; tracking[3] = 44; tracking[7] = 88;
rab.resize(3);
check(rab.byteLength === 3 && tracking.length === 3 && fixed.length === 0 && fixed.byteOffset === 0, 'logical length and out of bounds accessor');
rab.resize(8);
check(tracking[0] === 9 && tracking[2] === 0 && tracking[3] === 0 && tracking[7] === 0 && fixed.length === 4, 'growth clears discarded bytes');
var copied = rab.transfer(10);
check(rab.detached && rab.byteLength === 0 && copied.resizable && copied.maxByteLength === 16 && new Uint8Array(copied)[0] === 9, 'transfer retained policy and bytes');
var frozenLength = copied.transferToFixedLength(6);
check(copied.detached && !frozenLength.resizable && frozenLength.maxByteLength === 6, 'fixed transfer');
var source = new ArrayBuffer(6, { maxByteLength: 8 });
var bytes = new Uint8Array(source); bytes[0] = 1; bytes[1] = 2; bytes[2] = 3; bytes[3] = 4;
trace = [];
source.constructor = { get [Symbol.species]() { trace.push('species'); return function (size) {
  trace.push('construct:' + size); source.resize(3); return new ArrayBuffer(size);
}; } };
var slice = source.slice({ valueOf: function () { trace.push('start'); return 1; } }, 5);
check(trace.join(',') === 'start,species,construct:4' && new Uint8Array(slice).join(',') === '2,3,0,0', 'late slice source reobservation');
var sab = new SharedArrayBuffer(4, { maxByteLength: 8 });
new Uint8Array(sab)[0] = 17; sab.grow(8);
check(sab.growable && sab.byteLength === 8 && sab.maxByteLength === 8 && new Uint8Array(sab)[0] === 17 && new Uint8Array(sab)[7] === 0, 'shared retained resource growth');
check(ArrayBuffer.isView(new DataView(sab)) && ArrayBuffer.isView(new Int32Array(sab)) && !ArrayBuffer.isView(new Proxy(new Uint8Array(sab), {})), 'concrete brands');
print('gc-binary-buffers:ok');
262;
"#,
        "gc-binary-buffers:ok",
    );
}

#[test]
fn gc_data_view_uses_exact_endian_words_and_late_coercion_witnesses() {
    assert_binary_modes(
        r#"
function check(ok, label) { if (!ok) throw new Error(label); }
var buffer = new ArrayBuffer(32, { maxByteLength: 40 });
var view = new DataView(buffer);
view.setUint32(0, 0x12345678);
check(view.getUint8(0) === 0x12 && view.getUint32(0, true) === 0x78563412, 'big endian default and little endian read');
view.setInt16(4, -1234, true); check(view.getInt16(4, true) === -1234, 'signed width');
view.setBigInt64(8, -9223372036854775808n, true); check(view.getBigInt64(8, true) === -9223372036854775808n, 'signed BigInt minimum');
view.setBigUint64(16, 18446744073709551615n); check(view.getBigUint64(16) === 18446744073709551615n, 'unsigned BigInt full domain');
view.setFloat16(24, 1.5, true); check(view.getFloat16(24, true) === 1.5, 'shared half rounding kernel');
view.setFloat32(24, -0, true); check(Object.is(view.getFloat32(24, true), -0), 'float32 negative zero');
var trace = [];
var marker = {};
var original = marker;
try { view.setUint16({ valueOf: function () { trace.push('index'); return 1; } }, {
  valueOf: function () { trace.push('value'); buffer.resize(1); return 7; }
}, { valueOf: function () { throw new Error('ToBoolean must not coerce endian'); } }); }
catch (error) { original = error; }
check(original instanceof RangeError && trace.join(',') === 'index,value', 'late view size after value and no endian hook');
var detached = new ArrayBuffer(4);
trace = [];
var target = new Proxy(function Target() {}, { get: function (object, key) {
  if (key === 'prototype') { trace.push('prototype'); return DataView.prototype; } return object[key];
} });
original = marker;
try { Reflect.construct(DataView, [detached, 0, { valueOf: function () {
  trace.push('length'); detached.transfer(); return 1;
} }], target); } catch (error) { original = error; }
check(original instanceof TypeError && trace.join(',') === 'length,prototype', 'prototype precedes final detachment check');
var immutable = new ArrayBuffer(4).transferToImmutable();
var immutableView = new DataView(immutable);
trace = [];
try { immutableView.setUint8({ valueOf: function () { trace.push('index'); return 0; } }, {
  valueOf: function () { trace.push('value'); return 1; }
}); throw new Error('immutable accepted'); } catch (error) { check(error instanceof TypeError && trace.length === 0, 'immutable before setter hooks'); }
var getter = Object.getOwnPropertyDescriptor(DataView.prototype, 'buffer').get;
check(getter.call(new DataView(new ArrayBuffer(0))) instanceof ArrayBuffer, 'buffer accessor complete reference');
print('gc-binary-data-view:ok');
262;
"#,
        "gc-binary-data-view:ok",
    );
}

#[test]
fn gc_typed_array_construction_retains_iteration_bits_and_species_arity() {
    assert_binary_modes(
        r#"
function check(ok, label) { if (!ok) throw new Error(label); }
// Aligned offsets admit both explicit and implicit lengths. Nonzero
// remainders reject before supplied-length coercion; fixed implicit lengths
// require whole-buffer alignment, while tracking views retain their tail.
var alignmentConstructors = [Uint8Array, Uint16Array, Uint32Array, Float64Array, BigInt64Array];
for (var backingKind = 0; backingKind < 2; ++backingKind) {
  for (var alignmentIndex = 0; alignmentIndex < alignmentConstructors.length; ++alignmentIndex) {
    var AlignmentCtor = alignmentConstructors[alignmentIndex];
    var width = AlignmentCtor.BYTES_PER_ELEMENT;
    var alignedBuffer = backingKind === 0 ? new ArrayBuffer(width * 3) : new SharedArrayBuffer(width * 3);
    var alignedImplicit = new AlignmentCtor(alignedBuffer, width);
    var alignedExplicit = new AlignmentCtor(alignedBuffer, width, 1);
    var alignedEmpty = new AlignmentCtor(alignedBuffer, 0, 0);
    check(alignedImplicit.byteOffset === width && alignedImplicit.length === 2 && alignedExplicit.length === 1 && alignedEmpty.length === 0, 'zero alignment remainders admit view construction');
    if (width > 1) {
      var alignmentTrace = [], alignmentError = undefined;
      try {
        new AlignmentCtor(alignedBuffer, {valueOf: function () { alignmentTrace.push('offset'); return 1; }}, {
          valueOf: function () { alignmentTrace.push('length'); return 1; }
        });
      } catch (caught) { alignmentError = caught; }
      check(alignmentError instanceof RangeError && alignmentTrace.join(',') === 'offset', 'nonzero offset remainder rejects before length conversion');
      var tailBuffer = backingKind === 0 ? new ArrayBuffer(width * 3 + 1) : new SharedArrayBuffer(width * 3 + 1);
      alignmentError = undefined;
      try { new AlignmentCtor(tailBuffer); } catch (caught) { alignmentError = caught; }
      check(alignmentError instanceof RangeError, 'fixed absent-length view rejects nonzero buffer remainder');
      check(new AlignmentCtor(tailBuffer, 0, 2).length === 2, 'explicit length does not require whole fixed buffer alignment');
      var growingBuffer = backingKind === 0 ? new ArrayBuffer(width * 3 + 1, {maxByteLength: width * 5}) : new SharedArrayBuffer(width * 3 + 1, {maxByteLength: width * 5});
      check(new AlignmentCtor(growingBuffer, width).length === 2, 'tracking view retains allowed partial logical tail');
    }
  }
}
var trace = [], input, calls = 0;
var next = new Proxy(function () {}, { apply: function (fn, receiver, args) {
  check(receiver === input && args.length === 0, 'next receiver and no arguments');
  trace.push('next'); return ++calls === 1 ? { value: { valueOf: function () { trace.push('number'); return 7; } }, done: false } : { done: true, get value() { throw new Error('terminal value read'); } };
} });
input = { get next() { trace.push('get:next'); return next; } };
var source = { get [Symbol.iterator]() { trace.push('get:iterator'); return new Proxy(function () {}, { apply: function (fn, receiver, args) {
  check(receiver === source && args.length === 0, 'iterator receiver'); trace.push('iterator'); return input;
} }); } };
var array = new Uint16Array(source);
check(array.length === 1 && array[0] === 7 && trace.join(',') === 'get:iterator,iterator,get:next,next,next,number', 'list completes before element coercion');
trace = [];
var from = Uint8Array.from({ [Symbol.iterator]: undefined, get length() { trace.push('length'); return 2; }, get 0() { trace.push('0'); return 8; }, get 1() { trace.push('1'); return 9; } }, function (value, index) { trace.push('map:' + index); return value + index; });
check(from.join(',') === '8,10' && trace.join(',') === 'length,0,map:0,1,map:1', 'array-like mapping order');
check(BigInt64Array.of(-1n, 2n).join(',') === '-1,2', 'Of actual argv and GC BigInt');
var raw = new ArrayBuffer(8); new DataView(raw).setBigUint64(0, 0x7ff8000000000042n, true);
var clone = new Float64Array(new Float64Array(raw));
check(new DataView(clone.buffer).getBigUint64(0, true) === 0x7ff8000000000042n, 'same kind preserves NaN bytes');
var rab = new ArrayBuffer(8, { maxByteLength: 16 });
var tracking = new Uint16Array(rab), fixed = new Uint16Array(rab, 0, 2), arities = [];
function Species(buffer, offset, length) { arities.push(arguments.length); return arguments.length === 2 ? new Uint16Array(buffer, offset) : new Uint16Array(buffer, offset, length); }
tracking.constructor = { [Symbol.species]: Species }; fixed.constructor = { [Symbol.species]: Species };
var a = tracking.subarray(1), b = tracking.subarray(1, 3), c = fixed.subarray(1);
check(arities.join(',') === '2,3,3' && a.length === 3 && b.length === 2 && c.length === 1, 'tracking Subarray original argument arity');
rab.resize(12); check(a.length === 5 && b.length === 2, 'tracking result follows resize');
var immutable = new ArrayBuffer(0).transferToImmutable();
function ImmutableResult() { return new Uint8Array(immutable); }
for (var i = 0; i < 2; ++i) {
  var error = undefined;
  try { if (i === 0) Uint8Array.from.call(ImmutableResult, []); else Uint8Array.of.call(ImmutableResult); } catch (caught) { error = caught; }
  check(error instanceof TypeError, 'custom empty target requires write admission');
}
print('gc-binary-typed-array:ok');
262;
"#,
        "gc-binary-typed-array:ok",
    );
}

#[test]
fn gc_atomics_keep_whole_abrupts_finite_waits_and_called_realms() {
    assert_binary_modes(
        r#"
function check(ok, label) { if (!ok) throw new Error(label); }
var kinds = [Int8Array, Uint8Array, Int16Array, Uint16Array, Int32Array, Uint32Array, BigInt64Array, BigUint64Array];
for (var shared = 0; shared < 2; ++shared) {
  for (var i = 0; i < kinds.length; ++i) {
    var ctor = kinds[i], ta = new ctor(shared ? new SharedArrayBuffer(16) : new ArrayBuffer(16));
    var one = i < 6 ? 1 : 1n, two = i < 6 ? 2 : 2n, three = i < 6 ? 3 : 3n, zero = i < 6 ? 0 : 0n;
    check(Atomics.store(ta, 0, one) === one && Atomics.add(ta, 0, two) === one && Atomics.load(ta, 0) === three, 'store/add/load width');
    check(Atomics.compareExchange(ta, 0, three, two) === three && Atomics.exchange(ta, 0, one) === two, 'compare and exchange width');
    check(Atomics.or(ta, 0, two) === one && Atomics.xor(ta, 0, one) === three && Atomics.and(ta, 0, one) === two && Atomics.sub(ta, 0, one) === zero, 'bitwise and subtract width');
  }
}
var wait = new Int32Array(new SharedArrayBuffer(4));
var mismatch = Atomics.waitAsync(wait, 0, 1, 0), immediate = Atomics.waitAsync(wait, 0, 0, 0);
check(!mismatch.async && mismatch.value === 'not-equal' && !immediate.async && immediate.value === 'timed-out', 'finite waitAsync outcomes');
check(Atomics.notify(wait, 0, 0) === 0 && Atomics.notify(new Int32Array(1), 0) === 0, 'finite notify and nonshared admission');
check(Atomics.isLockFree(4) && !Atomics.isLockFree(3) && Atomics.pause(0) === undefined && Atomics.pause(Symbol('ignored'), { get valueOf() { throw new Error('ignored pause hook'); } }) === undefined, 'closed pure outcomes and ignored pause operands');
var buffer = new ArrayBuffer(8, { maxByteLength: 16 }), array = new Int32Array(buffer), trace = [];
var caught;
try { Atomics.compareExchange(array, { valueOf: function () { trace.push('index'); return 1; } },
  { valueOf: function () { trace.push('expected'); buffer.resize(4); return 0; } },
  { valueOf: function () { trace.push('replacement'); return 1; } }); }
catch (error) { caught = error; }
check(caught instanceof RangeError && trace.join(',') === 'index,expected,replacement', 'both value hooks before absolute-start revalidation');
var foreign = $262.createRealm().global, marker = new foreign.Error('marker');
var localError = TypeError, foreignError = foreign.TypeError;
var localProto = localError.prototype, foreignProto = foreignError.prototype, getPrototypeOf = Object.getPrototypeOf;
var localCtor = ArrayBuffer, foreignCtor = foreign.ArrayBuffer;
var localMethod = DataView.prototype.getUint8, foreignMethod = foreign.DataView.prototype.getUint8;
try {
  TypeError = function () { throw marker; }; foreign.TypeError = function () { throw marker; };
  ArrayBuffer = function () { throw marker; }; foreign.ArrayBuffer = function () { throw marker; };
  check(getPrototypeOf(Reflect.construct(localCtor, [1], foreignCtor)) === foreignCtor.prototype, 'foreign NewTarget controls local buffer prototype');
  check(getPrototypeOf(Reflect.construct(foreignCtor, [1], localCtor)) === localCtor.prototype, 'local NewTarget controls foreign buffer prototype');
  for (var i = 0; i < 2; ++i) {
    var method = i === 0 ? localMethod : foreignMethod;
    caught = undefined; try { method.call({}); } catch (error) { caught = error; }
    check(getPrototypeOf(caught) === (i === 0 ? localProto : foreignProto), 'borrowed callee intrinsic error Realm');
    var index = { valueOf: function () { throw marker; } }, prior = marker, finallyCount = 0;
    caught = undefined;
    try { prior = method.call(new DataView(new localCtor(1)), index); } catch (error) { caught = error; } finally { ++finallyCount; }
    check(caught === marker && prior === marker && finallyCount === 1, 'original index Throw preserved');
  }
} finally { TypeError = localError; foreign.TypeError = foreignError; ArrayBuffer = localCtor; foreign.ArrayBuffer = foreignCtor; }
var registered = Atomics.waitAsync(wait, 0, 0, 10_000);
check(registered.async && registered.value instanceof Promise && Atomics.notify(wait, 0, 1) === 1, 'registered finite waiter retained native resource');
registered.value.then(function (value) { check(value === 'ok', 'notify settles retained Promise once'); print('gc-binary-atomics:ok'); });
262;
"#,
        "gc-binary-atomics:ok",
    );
}

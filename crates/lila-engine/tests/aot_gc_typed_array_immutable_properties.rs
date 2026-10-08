use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_immutable_modes(source: &str, line: &str) {
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
            .expect(
                "finite integer-indexed property control compiles and executes through Wasm AOT",
            );
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
fn immutable_set_rejects_before_value_and_receiver_work() {
    assert_immutable_modes(
        r#"
function check(ok, label) { if (!ok) throw new Error(label); }
var source = new Uint8Array(2); source[0] = 7;
var array = new Uint8Array(source.buffer.transferToImmutable());
var calls = 0, input = { valueOf: function () { ++calls; return 99; } };
var receiverCalls = 0, receiver = new Proxy({}, {
  getOwnPropertyDescriptor: function () { ++receiverCalls; throw new Error('receiver read'); },
  defineProperty: function () { ++receiverCalls; throw new Error('receiver define'); }
});
var keys = ['0', '2', '-0', '-1', '1.5', 'NaN', 'Infinity'];
for (var i = 0; i < keys.length; ++i) {
  check(Reflect.set(array, keys[i], input) === false, 'immutable direct Set ' + keys[i]);
  check(Reflect.set(array, keys[i], input, receiver) === false, 'immutable receiver Set ' + keys[i]);
}
check(calls === 0 && receiverCalls === 0 && array[0] === 7, 'no conversion or receiver work');
var argumentTrace = [];
check(Reflect.set(array, { [Symbol.toPrimitive]: function () { argumentTrace.push('key'); return '0'; } },
  (argumentTrace.push('rhs'), input)) === false && argumentTrace.join(',') === 'rhs,key' && calls === 0, 'full arguments precede key conversion and immutable rejection');
var strict = (function () { return this; })() === undefined, threw = false;
try { array[0] = input; } catch (error) { threw = true; check(error instanceof TypeError, 'strict Set native error'); }
check(threw === strict && calls === 0, 'strict versus sloppy result without conversion');
var symbol = Symbol('ordinary');
check(Reflect.set(array, '01', input) && Reflect.set(array, '1.0', input) && Reflect.set(array, symbol, input), 'noncanonical ordinary keys');
check(array['01'] === input && array['1.0'] === input && array[symbol] === input && calls === 0, 'ordinary properties retain original value');
var mutable = new Uint8Array(1), destination = {};
check(Reflect.set(mutable, '0', input, destination) && destination[0] === input && mutable[0] === 0 && calls === 0, 'distinct valid receiver uses ordinary definition');
check(Reflect.set(mutable, '5', input, receiver) && receiverCalls === 0 && calls === 0, 'mutable invalid distinct receiver ignored');
check(Reflect.set(mutable, '5', input) && calls === 1, 'mutable invalid own receiver still coerces');
var bigintError;
try { Reflect.set(new BigInt64Array(0), '0', Symbol('not bigint')); } catch (error) { bigintError = error; }
check(bigintError instanceof TypeError, 'mutable invalid BigInt receiver still converts');
print('gc-immutable-set:ok');
262;
"#,
        "gc-immutable-set:ok",
    );
}

#[test]
fn immutable_descriptors_use_same_value_without_numeric_cast() {
    assert_immutable_modes(
        r#"
function check(ok, label) { if (!ok) throw new Error(label); }
var kinds = [Int8Array, Uint8Array, Uint8ClampedArray, Int16Array, Uint16Array, Int32Array, Uint32Array, Float16Array, Float32Array, Float64Array, BigInt64Array, BigUint64Array];
var hooks = 0, input = { valueOf: function () { ++hooks; return 1; } };
for (var i = 0; i < kinds.length; ++i) {
  var ctor = kinds[i], original = new ctor(1), one = i < 10 ? 1 : 1n;
  original[0] = one;
  var array = new ctor(original.buffer.transferToImmutable());
  var descriptor = Object.getOwnPropertyDescriptor(array, '0');
  check(descriptor.value === one && descriptor.writable === false && descriptor.enumerable === true && descriptor.configurable === false, 'immutable complete descriptor ' + i);
  check(Reflect.defineProperty(array, '0', {}) && Reflect.defineProperty(array, '0', { value: one, writable: false, enumerable: true, configurable: false }), 'compatible unchanged descriptor ' + i);
  check(!Reflect.defineProperty(array, '0', { value: input }) && !Reflect.defineProperty(array, '0', { value: i < 10 ? 2 : 2n }), 'uncoerced value mismatch ' + i);
  check(!Reflect.defineProperty(array, '0', { writable: true }) && !Reflect.defineProperty(array, '0', { configurable: true }) && !Reflect.defineProperty(array, '0', { enumerable: false }) && !Reflect.defineProperty(array, '0', { get: function () {} }), 'incompatible attributes ' + i);
  check(!Reflect.defineProperty(array, '1', { value: input }) && !Reflect.defineProperty(array, '-0', {}), 'invalid index rejects ' + i);
  var descriptorGets = 0;
  check(!Reflect.defineProperty(array, '0', { get value() { ++descriptorGets; return input; } }) && descriptorGets === 1, 'descriptor converted once before compatibility ' + i);
  check(array[0] === one && hooks === 0, 'no mutation or numeric conversion ' + i);
}
var zeroSource = new Float64Array(1); zeroSource[0] = -0;
var zero = new Float64Array(zeroSource.buffer.transferToImmutable());
check(Reflect.defineProperty(zero, '0', { value: -0 }) && !Reflect.defineProperty(zero, '0', { value: 0 }), 'SameValue distinguishes signed zero');
var nanSource = new Float64Array(1); nanSource[0] = NaN;
var nan = new Float64Array(nanSource.buffer.transferToImmutable());
check(Reflect.defineProperty(nan, '0', { value: NaN }), 'SameValue accepts NaN');
check(Object.freeze(zero) === zero && Object.isFrozen(zero) && Object.isSealed(zero), 'immutable integrity consumes false configurable/writable');
var mutable = new Uint8Array(1), error;
try { Object.freeze(mutable); } catch (caught) { error = caught; }
check(error instanceof TypeError && !Object.isExtensible(mutable), 'mutable integrity retains prior preventExtensions effect');
print('gc-immutable-descriptors:ok');
262;
"#,
        "gc-immutable-descriptors:ok",
    );
}

#[test]
fn immutable_proxy_invariants_and_native_errors_keep_called_realm() {
    assert_immutable_modes(
        r#"
function check(ok, label) { if (!ok) throw new Error(label); }
function expectTypeError(fn, label) { var error; try { fn(); } catch (caught) { error = caught; } check(error instanceof TypeError, label); }
var original = new Uint8Array(1); original[0] = 7;
var array = new Uint8Array(original.buffer.transferToImmutable());
var trace = [], hooks = 0, input = { valueOf: function () { ++hooks; return 7; } };
var proxy = new Proxy(array, { get: function () { trace.push('get'); return 8; }, set: function () { trace.push('set'); return true; }, defineProperty: function () { trace.push('define'); return true; }, deleteProperty: function () { trace.push('delete'); return true; } });
expectTypeError(function () { return proxy[0]; }, 'Get nonwritable/nonconfigurable invariant');
check(Reflect.set(proxy, '0', 7), 'SameValue trap success is valid');
expectTypeError(function () { Reflect.set(proxy, '0', input); }, 'Set compares raw value');
check(Reflect.defineProperty(proxy, '0', { value: 7 }), 'compatible Define trap success');
expectTypeError(function () { Reflect.defineProperty(proxy, '0', { value: input }); }, 'Define compatibility invariant');
expectTypeError(function () { Reflect.deleteProperty(proxy, '0'); }, 'nonconfigurable Delete invariant');
check(hooks === 0 && trace.join(',') === 'get,set,set,define,define,delete' && array[0] === 7, 'shared descriptor authority and trap effects');
var foreign = $262.createRealm().global, localType = TypeError, foreignType = foreign.TypeError;
var localDefine = Object.defineProperty, foreignDefine = foreign.Object.defineProperty;
var getPrototypeOf = Object.getPrototypeOf, localProto = localType.prototype, foreignProto = foreignType.prototype;
var marker = new foreign.Error('original'), prior = marker, finallyCount = 0;
try {
  TypeError = function () { throw marker; }; foreign.TypeError = function () { throw marker; };
  for (var i = 0; i < 2; ++i) {
    var define = i === 0 ? localDefine : foreignDefine, error = undefined;
    try { define(array, '0', { value: 8 }); } catch (caught) { error = caught; }
    check(getPrototypeOf(error) === (i === 0 ? localProto : foreignProto), 'borrowed Define intrinsic Realm');
  }
  var mutable = new Uint8Array(1), thrown;
  try { prior = Reflect.defineProperty(mutable, '0', { value: { valueOf: function () { throw marker; } } }); }
  catch (caught) { thrown = caught; } finally { ++finallyCount; }
  check(thrown === marker && prior === marker && finallyCount === 1 && mutable[0] === 0, 'original conversion Throw preserved');
} finally { TypeError = localType; foreign.TypeError = foreignType; }
print('gc-immutable-proxy-realms:ok');
262;
"#,
        "gc-immutable-proxy-realms:ok",
    );
}

#[test]
fn writable_species_admission_precedes_empty_or_populated_result_loops() {
    assert_immutable_modes(
        r#"
function check(ok, label) { if (!ok) throw new Error(label); }
function expectTypeError(fn, label) { var error; try { fn(); } catch (caught) { error = caught; } check(error instanceof TypeError, label); }
for (var length = 0; length < 2; ++length) {
  var original = new Uint8Array(length); if (length) original[0] = 7;
  var immutable = new Uint8Array(original.buffer.transferToImmutable());
  var source = new Uint8Array(length), trace = [], callbacks = 0;
  source.constructor = { [Symbol.species]: function (count) { trace.push('construct:' + count); return immutable; } };
  expectTypeError(function () { source.map(function (value) { ++callbacks; return value; }); }, 'Map immutable target');
  check(callbacks === 0 && trace.join(',') === 'construct:' + length, 'Map validates target before callback loop');
  trace = []; callbacks = 0;
  expectTypeError(function () { source.filter(function () { ++callbacks; trace.push('callback'); return false; }); }, 'Filter immutable empty target');
  check(callbacks === length && trace.join(',') === (length ? 'callback,construct:0' : 'construct:0'), 'Filter observes all predicates before write target admission');
  trace = [];
  expectTypeError(function () { source.slice({ valueOf: function () { trace.push('start'); return 0; } }, { valueOf: function () { trace.push('end'); return 0; } }); }, 'Slice immutable empty target');
  check(trace.join(',') === 'start,end,construct:0', 'Slice bounds hooks before write target admission');
  var subarray = immutable.subarray(0);
  check(subarray.length === length && (!length || subarray[0] === 7), 'real Subarray keeps read admission');
}
print('gc-immutable-species:ok');
262;
"#,
        "gc-immutable-species:ok",
    );
}

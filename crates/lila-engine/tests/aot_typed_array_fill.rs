use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_typed_array_fill(source: &str) {
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
        .expect("TypedArray.fill regression must execute through Wasm AOT");
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
fn fill_has_its_own_builtin_identity_and_requires_typed_array_slots() {
    assert_typed_array_fill(
        r#"
var prototype = Object.getPrototypeOf(Uint8Array.prototype);
var fill = prototype.fill;
var descriptor = Object.getOwnPropertyDescriptor(prototype, 'fill');
if (fill === Array.prototype.fill || fill !== Int16Array.prototype.fill ||
    fill.name !== 'fill' || fill.length !== 1 ||
    !descriptor.writable || descriptor.enumerable || !descriptor.configurable) {
  throw 'builtin identity and descriptor';
}
var calls = 0;
var value = { valueOf() { calls++; return 7; } };
for (var receiver of [undefined, null, true, 1, 'x', {}, [],
    new Proxy(new Uint8Array(1), {})]) {
  var received = undefined;
  try { fill.call(receiver, value); } catch (error) { received = error; }
  if (!(received instanceof TypeError)) throw 'receiver brand';
}
if (calls !== 0) throw 'coercion before receiver validation';
var constructed;
try { new fill(1); } catch (error) { constructed = error; }
if (!(constructed instanceof TypeError)) throw 'fill is not a constructor';
var values = new Uint8Array(1);
if (fill.call(values, 7) !== values || values[0] !== 7) throw 'return receiver';
print(true);
"#,
    );
}

#[test]
fn numeric_fill_converts_once_and_preserves_the_payload_for_every_element() {
    assert_typed_array_fill(
        r#"
for (var Constructor of [Int8Array, Uint8Array, Int16Array, Uint16Array,
    Int32Array, Uint32Array, Uint8ClampedArray, Float32Array, Float64Array]) {
  var calls = 0, values = new Constructor(3);
  values.fill({ valueOf() { calls++; return 7; } });
  if (calls !== 1 || values[0] !== 7 || values[1] !== 7 || values[2] !== 7) {
    throw 'converted payload must survive every store';
  }
  calls = 0;
  new Constructor(0).fill({ valueOf() { calls++; return 7; } });
  if (calls !== 1) throw 'empty view still converts value';
}
var signed = new Int8Array(2).fill(257.9);
var unsigned = new Uint16Array(2).fill(-1.9);
var clamped = new Uint8ClampedArray(2).fill(2.5);
var negativeZero = new Float64Array(2).fill(-0);
if (signed[0] !== 1 || signed[1] !== 1 || unsigned[0] !== 65535 ||
    unsigned[1] !== 65535 || clamped[0] !== 2 || clamped[1] !== 2 ||
    !Object.is(negativeZero[0], -0) || !Object.is(negativeZero[1], -0)) {
  throw 'element storage conversion';
}
print(true);
"#,
    );
}

#[test]
fn bigint_fill_uses_to_bigint_once_and_rejects_mixed_numeric_kinds() {
    assert_typed_array_fill(
        r#"
for (var Constructor of [BigInt64Array, BigUint64Array]) {
  var calls = 0, values = new Constructor(3);
  values.fill({ valueOf() { calls++; return 18446744073709551623n; } });
  if (calls !== 1 || values[0] !== 7n || values[1] !== 7n || values[2] !== 7n) {
    throw 'BigInt conversion and modulo storage';
  }
  values.fill('9');
  if (values[0] !== 9n || values[2] !== 9n) throw 'string ToBigInt';
  values.fill(true);
  if (values[0] !== 1n || values[2] !== 1n) throw 'boolean ToBigInt';
  var endCalls = 0, received = undefined;
  try { values.fill(1, { valueOf() { endCalls++; return 0; } }); }
  catch (error) { received = error; }
  if (!(received instanceof TypeError) || endCalls !== 0 || values[0] !== 1n) {
    throw 'Number rejection before start';
  }
}
var signed = new BigInt64Array(2).fill(18446744073709551615n);
if (signed[0] !== -1n || signed[1] !== -1n) throw 'signed BigInt storage';
var received;
try { new Uint8Array(0).fill(1n); } catch (error) { received = error; }
if (!(received instanceof TypeError)) throw 'BigInt rejection for numeric empty view';
print(true);
"#,
    );
}

#[test]
fn typed_array_fill_ignores_length_properties_and_borrowed_array_fill_observes_them() {
    assert_typed_array_fill(
        r#"
var prototype = Object.getPrototypeOf(Uint8Array.prototype);
var values = new Uint8Array(3), lengthReads = 0, calls = 0;
Object.defineProperty(prototype, 'length', {
  configurable: true, get() { lengthReads++; return 1; }
});
Object.defineProperty(Uint8Array.prototype, 'length', {
  configurable: true, get() { lengthReads++; return 1; }
});
Object.defineProperty(values, 'length', {
  configurable: true, get() { lengthReads++; return 2; }
});
var value = { valueOf() { calls++; return calls; } };
values.fill(value);
if (lengthReads !== 0 || calls !== 1 || values[0] !== 1 || values[2] !== 1) {
  throw 'TypedArray.fill must use slots and one value conversion';
}
calls = 0;
if (Array.prototype.fill.call(values, value) !== values || lengthReads !== 1 ||
    calls !== 2 || values[0] !== 1 || values[1] !== 2 || values[2] !== 1) {
  throw 'borrowed Array.fill must use Get length and Set each element';
}
print(true);
"#,
    );
}

#[test]
fn fill_coercions_are_ordered_and_preserve_abrupt_values() {
    assert_typed_array_fill(
        r#"
var values = new Uint8Array(3), trace = '', marker = {}, received;
function argument(label, result) {
  return { valueOf() { trace += label; return result; } };
}
values.fill(argument('v', 7), argument('s', 0), argument('e', 2));
if (trace !== 'vse' || values[0] !== 7 || values[1] !== 7 || values[2] !== 0) {
  throw 'value/start/end conversion order';
}
for (var abruptAt of ['v', 's', 'e']) {
  trace = ''; received = undefined;
  function abruptArgument(label, result) {
    return { valueOf() { trace += label; if (label === abruptAt) throw marker; return result; } };
  }
  try { values.fill(abruptArgument('v', 9), abruptArgument('s', 0), abruptArgument('e', 2)); }
  catch (error) { received = error; }
  var expected = abruptAt === 'v' ? 'v' : abruptAt === 's' ? 'vs' : 'vse';
  if (received !== marker || trace !== expected || values[0] !== 7) {
    throw 'abrupt argument identity and order';
  }
}
print(true);
"#,
    );
}

#[test]
fn fill_validates_detachment_at_entry_and_after_all_coercions_even_for_empty_ranges() {
    assert_typed_array_fill(
        r#"
var empty = new Uint8Array(0), calls = 0, received;
__lilaDetachArrayBuffer(empty.buffer);
try { empty.fill({ valueOf() { calls++; return 1; } }); }
catch (error) { received = error; }
if (!(received instanceof TypeError) || calls !== 0) throw 'detached entry';
for (var detachAt of ['v', 's', 'e']) {
  var values = new Uint8Array(2), trace = '';
  function argument(label, result) {
    return { valueOf() {
      trace += label;
      if (label === detachAt) __lilaDetachArrayBuffer(values.buffer);
      return result;
    } };
  }
  received = undefined;
  try { values.fill(argument('v', 7), argument('s', 0), argument('e', 0)); }
  catch (error) { received = error; }
  if (!(received instanceof TypeError) || trace !== 'vse') {
    throw 'final detached validation includes empty range';
  }
}
var values = new Uint8Array(2), marker = {};
try {
  values.fill({ valueOf() { __lilaDetachArrayBuffer(values.buffer); return 1; } },
    { valueOf() { throw marker; } });
} catch (error) { received = error; }
if (received !== marker) throw 'start abrupt completion precedes late validation';
print(true);
"#,
    );
}

#[test]
fn fill_checks_fixed_view_bounds_after_coercion_and_allows_restored_bounds() {
    assert_typed_array_fill(
        r#"
for (var resizeAt of ['v', 's', 'e']) {
  var buffer = new ArrayBuffer(4, { maxByteLength: 8 });
  var values = new Uint8Array(buffer, 0, 4), trace = '', received = undefined;
  function argument(label, result) {
    return { valueOf() { trace += label; if (label === resizeAt) buffer.resize(2); return result; } };
  }
  try { values.fill(argument('v', 7), argument('s', 0), argument('e', 0)); }
  catch (error) { received = error; }
  if (!(received instanceof TypeError) || trace !== 'vse') throw 'fixed view out of bounds';
}
var buffer = new ArrayBuffer(4, { maxByteLength: 8 });
var values = new Uint8Array(buffer, 0, 4);
values.fill(9,
  { valueOf() { buffer.resize(2); return 0; } },
  { valueOf() { buffer.resize(4); return 4; } });
if (values.length !== 4 || values[0] !== 9 || values[3] !== 9) throw 'restored fixed bounds';
buffer.resize(2);
var calls = 0, received;
try { values.fill({ valueOf() { calls++; return 7; } }); }
catch (error) { received = error; }
if (!(received instanceof TypeError) || calls !== 0) throw 'out of bounds at entry';
print(true);
"#,
    );
}

#[test]
fn fill_tracking_views_clamp_initial_bounds_and_refresh_storage_after_resize() {
    assert_typed_array_fill(
        r#"
var buffer = new ArrayBuffer(4, { maxByteLength: 8 });
var values = new Uint8Array(buffer);
values.fill({ valueOf() { buffer.resize(2); return 7; } });
if (values.length !== 2 || values[0] !== 7 || values[1] !== 7) throw 'tracking shrink';
buffer.resize(1);
values.fill({ valueOf() { buffer.resize(8); return 9; } });
if (values.length !== 8 || values[0] !== 9 || values[1] !== 0 || values[7] !== 0) {
  throw 'growth retains initial end and uses new storage';
}
buffer.resize(4);
values.fill(0);
values.fill(3, { valueOf() { buffer.resize(8); return -1; } });
if (values[2] !== 0 || values[3] !== 3 || values[4] !== 0) throw 'negative start uses initial length';
buffer.resize(4);
var empty = new Uint8Array(buffer, 4);
empty.fill(5, { valueOf() { buffer.resize(4); return 0; } });
if (empty.length !== 0) throw 'valid empty tracking view';
var received;
try { empty.fill(5, { valueOf() { buffer.resize(2); return 0; } }); }
catch (error) { received = error; }
if (!(received instanceof TypeError)) throw 'tracking byte offset out of bounds';
print(true);
"#,
    );
}

#[test]
fn fill_rejects_immutable_backing_buffers_before_any_argument_conversion() {
    assert_typed_array_fill(
        r#"
for (var Constructor of [Uint8Array, BigInt64Array]) {
  for (var count of [0, 2]) {
    var source = new ArrayBuffer(count * Constructor.BYTES_PER_ELEMENT);
    var immutable = source.transferToImmutable();
    var values = new Constructor(immutable), calls = 0, received;
    var argument = { valueOf() { calls++; return 0; } };
    received = undefined;
    try { values.fill(argument, argument, argument); }
    catch (error) { received = error; }
    if (!(received instanceof TypeError) || calls !== 0 || values.length !== count) {
      throw 'immutable receiver validation precedes coercion, including empty views';
    }
    if (count !== 0 && values[0] !== (Constructor === BigInt64Array ? 0n : 0)) {
      throw 'immutable contents changed';
    }
  }
}
var other = __lilaCreateRealm().global;
var immutable = new ArrayBuffer(2).sliceToImmutable();
var values = new Uint8Array(immutable), received;
try { other.Uint8Array.prototype.fill.call(values, 7); }
catch (error) { received = error; }
if (!(received instanceof other.TypeError) || received instanceof TypeError || values[0] !== 0) {
  throw 'immutable rejection uses invoked builtin realm';
}
print(true);
"#,
    );
}

#[test]
fn fill_normalizes_range_bounds_without_writing_outside_a_view() {
    assert_typed_array_fill(
        r#"
var buffer = new ArrayBuffer(6), all = new Uint8Array(buffer);
all[0] = 11; all[5] = 13;
var values = new Uint8Array(buffer, 1, 4);
values.fill(7, -0, Infinity);
if (all[0] !== 11 || all[5] !== 13 || values[0] !== 7 || values[3] !== 7) throw 'view extent';
values.fill(3, -2.9, -0);
if (values[2] !== 7 || values[3] !== 7) throw 'negative zero end';
values.fill(3, -2.9, undefined);
if (values[1] !== 7 || values[2] !== 3 || values[3] !== 3) throw 'fractional negative start';
values.fill(5, -Infinity, 1.9);
values.fill(9, NaN, NaN);
values.fill(9, Infinity);
if (values[0] !== 5 || values[1] !== 7 || values[2] !== 3 || values[3] !== 3) throw 'normalized range';
print(true);
"#,
    );
}

#[test]
fn fill_publication_and_type_errors_use_the_invoked_builtin_realm() {
    assert_typed_array_fill(
        r#"
var other = __lilaCreateRealm().global;
var fill = other.Uint8Array.prototype.fill;
if (fill === Uint8Array.prototype.fill || fill === other.Array.prototype.fill ||
    fill !== other.BigInt64Array.prototype.fill ||
    Object.getPrototypeOf(fill) !== other.Function.prototype) throw 'realm builtin identity';
var values = new Uint8Array(2);
if (fill.call(values, 7) !== values || values[1] !== 7) throw 'foreign builtin receiver';
var received;
try { fill.call([], 1); } catch (error) { received = error; }
if (!(received instanceof other.TypeError) || received instanceof TypeError) throw 'brand error realm';
received = undefined;
try { fill.call(values, 1n); } catch (error) { received = error; }
if (!(received instanceof other.TypeError) || received instanceof TypeError) throw 'coercion error realm';
__lilaDetachArrayBuffer(values.buffer);
received = undefined;
try { fill.call(values, 1); } catch (error) { received = error; }
if (!(received instanceof other.TypeError) || received instanceof TypeError) throw 'entry validation error realm';
var buffer = new ArrayBuffer(2, { maxByteLength: 4 });
values = new Uint8Array(buffer, 0, 2);
received = undefined;
try { fill.call(values, 1, { valueOf() { buffer.resize(0); return 0; } }, 0); }
catch (error) { received = error; }
if (!(received instanceof other.TypeError) || received instanceof TypeError) throw 'late validation error realm';
print(true);
"#,
    );
}

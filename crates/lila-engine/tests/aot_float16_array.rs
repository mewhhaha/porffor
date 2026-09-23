use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_float16_array(source: &str) {
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
        .expect("Float16Array regression must execute through Wasm AOT");
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine("true".to_string())],
        "{source}"
    );
}

#[test]
fn constructor_identity_descriptors_and_shared_prototype() {
    assert_float16_array(
        r#"
var constructor = Float16Array, prototype = Float16Array.prototype;
if (globalThis.Float16Array !== constructor || constructor.name !== 'Float16Array' ||
    constructor.length !== 3 || constructor.BYTES_PER_ELEMENT !== 2 ||
    prototype.BYTES_PER_ELEMENT !== 2 || prototype.constructor !== constructor ||
    Object.getPrototypeOf(constructor) !== Object.getPrototypeOf(Float32Array) ||
    Object.getPrototypeOf(prototype) !== Object.getPrototypeOf(Float32Array.prototype) ||
    constructor[Symbol.species] !== constructor) throw 'constructor graph';
for (var target of [constructor, prototype]) {
  var descriptor = Object.getOwnPropertyDescriptor(target, 'BYTES_PER_ELEMENT');
  if (descriptor.value !== 2 || descriptor.writable || descriptor.enumerable ||
      descriptor.configurable) throw 'element-width descriptor';
}
var globalDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'Float16Array');
if (!globalDescriptor.writable || globalDescriptor.enumerable ||
    !globalDescriptor.configurable) throw 'global descriptor';
var caught;
try { Float16Array(1); } catch (error) { caught = error; }
if (!(caught instanceof TypeError)) throw 'requires new';
var values = new constructor(2);
if (values.length !== 2 || values.byteLength !== 4 || values.byteOffset !== 0 ||
    !ArrayBuffer.isView(values) || Object.prototype.toString.call(values) !== '[object Float16Array]' ||
    values[Symbol.toStringTag] !== 'Float16Array' || values[0] !== 0 || values[1] !== 0) {
  throw 'instance identity';
}
print(true);
"#,
    );
}

#[test]
fn rounds_directly_from_f64_at_midpoints_subnormals_and_overflow() {
    assert_float16_array(
        r#"
var midpoint = 1 + 2 ** -11;
var source = [0, -0, 1, -2, Infinity, -Infinity,
  2 ** -24, 2 ** -25, 2 ** -25 + 2 ** -55, 3 * 2 ** -25,
  1023 * 2 ** -24, 2 ** -14,
  midpoint, midpoint + 2 ** -40, midpoint - 2 ** -40,
  -midpoint, -midpoint - 2 ** -40, -midpoint + 2 ** -40,
  1 + 3 * 2 ** -11, 65504, 65520, 65520 - 2 ** -20,
  -65520, -65520 + 2 ** -20];
var expected = [0, 0x8000, 0x3c00, 0xc000, 0x7c00, 0xfc00,
  1, 0, 1, 2, 0x3ff, 0x400,
  0x3c00, 0x3c01, 0x3c00, 0xbc00, 0xbc01, 0xbc00,
  0x3c02, 0x7bff, 0x7c00, 0x7bff, 0xfc00, 0xfbff];
var values = new Float16Array(source), words = new Uint16Array(values.buffer);
for (var i = 0; i < source.length; i++) {
  if (words[i] !== expected[i]) throw 'constructor rounding at ' + i;
  values[i] = source[i];
  if (words[i] !== expected[i]) throw 'indexed rounding at ' + i;
}
if (!Object.is(values[1], -0)) throw 'negative zero';
values[0] = NaN;
if (!Number.isNaN(values[0]) || (words[0] & 0x7c00) !== 0x7c00 ||
    (words[0] & 0x3ff) === 0) throw 'NaN';
print(true);
"#,
    );
}

#[test]
fn every_binary16_encoding_decodes_and_round_trips() {
    assert_float16_array(
        r#"
var buffer = new ArrayBuffer(2), words = new Uint16Array(buffer);
var values = new Float16Array(buffer), copy = new Float16Array(1);
var copiedWords = new Uint16Array(copy.buffer);
for (var bits = 0; bits < 65536; bits++) {
  words[0] = bits;
  var value = values[0];
  if ((bits & 0x7c00) === 0x7c00 && (bits & 0x3ff) !== 0) {
    if (!Number.isNaN(value)) throw 'NaN decode';
  } else {
    copy[0] = value;
    if (copiedWords[0] !== bits) throw 'round trip ' + bits;
  }
}
print(true);
"#,
    );
}

#[test]
fn constructs_from_iterable_array_like_and_numeric_typed_array_sources() {
    assert_float16_array(
        r#"
var iterable = { [Symbol.iterator]() {
  var index = 0;
  return { next() { return index < 3 ? { value: ++index + 0.5, done: false } : { done: true }; } };
} };
for (var values of [new Float16Array(iterable),
    new Float16Array({ 0: 1.5, 1: 2.5, 2: 3.5, length: 3 }),
    new Float16Array(new Float32Array([1.5, 2.5, 3.5]))]) {
  if (values.join(',') !== '1.5,2.5,3.5') throw 'source construction';
}
if (Float16Array.from([1, 2], x => x + 0.5).join(',') !== '1.5,2.5' ||
    Float16Array.of(1.5, 2.5).join(',') !== '1.5,2.5') throw 'static methods';
var calls = 0, values = new Float16Array(3);
values.fill({ valueOf() { calls++; return 1 + 2 ** -11 + 2 ** -40; } });
if (calls !== 1 || values.join(',') !== '1.0009765625,1.0009765625,1.0009765625') {
  throw 'fill direct rounding and one conversion';
}
values.set(new Float64Array([2.5, 3.5]), 1);
if (values[1] !== 2.5 || values[2] !== 3.5) throw 'cross-kind set';
var integers = new Int16Array(3); integers.set(values);
if (integers.join(',') !== '1,2,3') throw 'Number content transfer';
print(true);
"#,
    );
}

#[test]
fn views_share_buffers_and_follow_resize_and_detachment() {
    assert_float16_array(
        r#"
var buffer = new ArrayBuffer(8, { maxByteLength: 16 });
var tracking = new Float16Array(buffer, 2), fixed = new Float16Array(buffer, 2, 2);
tracking[0] = 1.5;
if (fixed[0] !== 1.5 || tracking.length !== 3 || fixed.length !== 2 || fixed.byteOffset !== 2) {
  throw 'shared view';
}
buffer.resize(12);
if (tracking.length !== 5 || fixed.length !== 2 || tracking[4] !== 0) throw 'growth';
buffer.resize(4);
if (tracking.length !== 1 || fixed.length !== 0 || fixed.byteLength !== 0 ||
    fixed[0] !== undefined) throw 'out of bounds';
buffer.resize(8);
if (fixed.length !== 2 || fixed[0] !== 1.5 || fixed[1] !== 0) throw 'restored view';
var caught;
try { new Float16Array(buffer, 1); } catch (error) { caught = error; }
if (!(caught instanceof RangeError)) throw 'alignment';
__lilaDetachArrayBuffer(buffer);
if (tracking.length !== 0 || tracking[0] !== undefined) throw 'detached read';
caught = undefined;
try { tracking.slice(); } catch (error) { caught = error; }
if (!(caught instanceof TypeError)) throw 'detached method';
var shared = new SharedArrayBuffer(4), first = new Float16Array(shared);
var second = new Float16Array(shared); first[1] = -1.5;
if (second[1] !== -1.5 || first.buffer !== shared) throw 'shared backing memory';
print(true);
"#,
    );
}

#[test]
fn generic_callbacks_search_iteration_and_change_by_copy_use_number_values() {
    assert_float16_array(
        r#"
var values = new Float16Array([1.5, 2.5, 3.5]);
if (values.at(-1) !== 3.5 || !values.includes(2.5) || values.indexOf(2.5) !== 1 ||
    values.lastIndexOf(2.5) !== 1 || values.find(x => x > 2) !== 2.5 ||
    values.findIndex(x => x > 2) !== 1 || values.findLast(x => x > 2) !== 3.5 ||
    values.findLastIndex(x => x > 2) !== 2 || !values.every(x => x > 1) ||
    !values.some(x => x === 2.5)) throw 'search methods';
var sum = 0; values.forEach(x => { sum += x; });
if (sum !== 7.5 || values.reduce((a, b) => a + b, 0) !== 7.5 ||
    values.reduceRight((a, b) => a - b, 10) !== 2.5) throw 'callback methods';
if (values.map(x => x * 2).join(',') !== '3,5,7' ||
    values.filter(x => x > 2).join(',') !== '2.5,3.5') throw 'derived typed arrays';
if (Array.from(values.values()).join(',') !== '1.5,2.5,3.5' ||
    Array.from(values.keys()).join(',') !== '0,1,2' ||
    Array.from(values.entries())[1].join(',') !== '1,2.5') throw 'iterators';
if (values.toReversed().join(',') !== '3.5,2.5,1.5' ||
    values.with(1, 9.5).join(',') !== '1.5,9.5,3.5' ||
    values.join(',') !== '1.5,2.5,3.5') throw 'change by copy';
values.copyWithin(1, 0, 2);
if (values.join(',') !== '1.5,1.5,2.5') throw 'copyWithin overlap';
values.reverse();
if (values.toString() !== '2.5,1.5,1.5') throw 'reverse/toString';
if (new Float16Array([1, 2]).toLocaleString() !== '1,2') throw 'toLocaleString';
print(true);
"#,
    );
}

#[test]
fn numeric_sort_handles_nan_signed_zero_and_comparator_callbacks() {
    assert_float16_array(
        r#"
var original = new Float16Array([NaN, 10, -0, 2, 0, -Infinity, Infinity]);
var values = original.toSorted();
if (values[0] !== -Infinity || !Object.is(values[1], -0) || !Object.is(values[2], 0) ||
    values[3] !== 2 || values[4] !== 10 || values[5] !== Infinity ||
    !Number.isNaN(values[6]) || !Number.isNaN(original[0])) throw 'default numeric order';
if (original.sort() !== original || !Object.is(original[1], -0) ||
    !Number.isNaN(original[6])) throw 'in place sort';
var calls = 0;
values = new Float16Array([1.5, 3.5, 2.5]);
values.sort((a, b) => { calls++; return b - a; });
if (calls === 0 || values.join(',') !== '3.5,2.5,1.5') throw 'custom sort';
print(true);
"#,
    );
}

#[test]
fn species_and_subclasses_keep_float16_storage_and_allow_other_number_kinds() {
    assert_float16_array(
        r#"
class Half extends Float16Array {}
var original = new Half([1.5, 2.5, 3.5]);
for (var values of [original.slice(1), original.subarray(1), original.map(x => x),
    original.filter(x => x > 1)]) {
  if (!(values instanceof Half) || values.BYTES_PER_ELEMENT !== 2) throw 'subclass result';
}
original.constructor = { [Symbol.species]: Float32Array };
if (!(original.slice(1) instanceof Float32Array) ||
    original.map(x => x + 0.25).join(',') !== '1.75,2.75,3.75') throw 'Number species';
original.constructor = { [Symbol.species]: BigInt64Array };
for (var operation of [() => original.slice(0, 0), () => original.subarray(0, 0),
    () => original.map(x => x), () => original.filter(x => false)]) {
  var caught;
  try { operation(); } catch (error) { caught = error; }
  if (!(caught instanceof TypeError)) throw 'BigInt species content';
}
print(true);
"#,
    );
}

#[test]
fn foreign_constructor_and_new_target_prototypes_use_their_realms() {
    assert_float16_array(
        r#"
var other = __lilaCreateRealm().global;
var ForeignHalf = other.Float16Array;
if (ForeignHalf === Float16Array || ForeignHalf.prototype === Float16Array.prototype ||
    ForeignHalf.BYTES_PER_ELEMENT !== 2) throw 'foreign constructor identity';
var values = new ForeignHalf([1.5]);
if (Object.getPrototypeOf(values) !== ForeignHalf.prototype || values[0] !== 1.5 ||
    values.constructor !== ForeignHalf) throw 'foreign construction';
var boundTarget = other.Function.bind(null);
boundTarget.prototype = 1;
values = Reflect.construct(Float16Array, [2], boundTarget);
if (Object.getPrototypeOf(values) !== ForeignHalf.prototype || values.length !== 2) {
  throw 'new target realm fallback';
}
var caught;
try { ForeignHalf(1); } catch (error) { caught = error; }
if (!(caught instanceof other.TypeError) || caught instanceof TypeError) throw 'foreign new error';
print(true);
"#,
    );
}

#[test]
fn atomics_rejects_float16_before_index_or_value_coercion() {
    assert_float16_array(
        r#"
var values = new Float16Array(new SharedArrayBuffer(4));
var calls = 0, argument = { valueOf() { calls++; return 0; } };
for (var operation of [
  () => Atomics.add(values, argument, argument),
  () => Atomics.and(values, argument, argument),
  () => Atomics.compareExchange(values, argument, argument, argument),
  () => Atomics.exchange(values, argument, argument),
  () => Atomics.load(values, argument),
  () => Atomics.or(values, argument, argument),
  () => Atomics.store(values, argument, argument),
  () => Atomics.sub(values, argument, argument),
  () => Atomics.xor(values, argument, argument),
  () => Atomics.wait(values, argument, argument, 0),
  () => Atomics.waitAsync(values, argument, argument, 0),
  () => Atomics.notify(values, argument, argument)
]) {
  var caught;
  try { operation(); } catch (error) { caught = error; }
  if (!(caught instanceof TypeError)) throw 'Atomics receiver restriction';
}
if (calls !== 0) throw 'Atomics rejection ordering';
print(true);
"#,
    );
}

#[test]
fn constructor_rejects_empty_and_populated_cross_content_sources() {
    assert_float16_array(
        r#"
for (var NumberArray of [Float16Array, Float32Array, Int16Array]) {
  for (var length of [0, 1]) {
    for (var operation of [
      () => new NumberArray(new BigInt64Array(length)),
      () => new BigInt64Array(new NumberArray(length))
    ]) {
      var caught;
      try { operation(); } catch (error) { caught = error; }
      if (!(caught instanceof TypeError)) throw 'cross-content constructor';
    }
  }
  if (new NumberArray(new Float16Array(0)).length !== 0 ||
      new Float16Array(new NumberArray([1, 2])).join(',') !== '1,2') {
    throw 'same-content controls';
  }
}
if (new BigInt64Array(new BigUint64Array(0)).length !== 0 ||
    new BigInt64Array(new BigUint64Array([1n]))[0] !== 1n) throw 'BigInt controls';
print(true);
"#,
    );
}

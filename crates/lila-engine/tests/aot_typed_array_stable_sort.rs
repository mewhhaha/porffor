use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_sort(source: &str) {
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
        .expect("TypedArray sort regression must execute through Wasm AOT");
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
fn large_reverse_and_pseudorandom_inputs_sort_numerically() {
    assert_sort(
        r#"
const count = 32768;
const reverse = new Uint32Array(count);
for (let i = 0; i < count; i++) reverse[i] = count - i;
if (reverse.sort() !== reverse) throw 'sort return value';
for (let i = 0; i < count; i++) {
  if (reverse[i] !== i + 1) throw 'reverse at ' + i;
}
const random = new Int32Array(count);
let seed = 0x12345678, originalSum = 0;
for (let i = 0; i < count; i++) {
  seed = (Math.imul(seed, 1664525) + 1013904223) | 0;
  random[i] = seed;
  originalSum += seed;
}
const first = random[0], last = random[count - 1];
const copy = random.toSorted();
let sortedSum = copy[0];
for (let i = 1; i < count; i++) {
  if (copy[i - 1] > copy[i]) throw 'random order at ' + i;
  sortedSum += copy[i];
}
if (sortedSum !== originalSum || random[0] !== first || random[count - 1] !== last) {
  throw 'random contents';
}
print(true);
"#,
    );
}

#[test]
fn equal_comparator_keys_preserve_original_order() {
    assert_sort(
        r#"
const values = new Int32Array(2048);
for (let i = 0; i < values.length; i++) values[i] = (i % 7) * 10000 + i;
const original = values.slice();
const sorted = values.toSorted((a, b) => Math.trunc(a / 10000) - Math.trunc(b / 10000));
if (values.join() !== original.join()) throw 'toSorted mutated source';
let lastKey = -1;
const lastIndex = new Int32Array(7).fill(-1);
for (const item of sorted) {
  const key = Math.trunc(item / 10000), index = item % 10000;
  if (key < lastKey || index <= lastIndex[key]) throw 'unstable equal keys';
  lastKey = key;
  lastIndex[key] = index;
}
print(true);
"#,
    );
}

#[test]
fn default_order_handles_nan_signed_zero_and_bigint_signs() {
    assert_sort(
        r#"
const numbers = new Float64Array([NaN, 0, -0, Infinity, -Infinity, 3, NaN, -2]);
const sorted = numbers.toSorted();
if (sorted[0] !== -Infinity || sorted[1] !== -2 || !Object.is(sorted[2], -0) ||
    !Object.is(sorted[3], 0) || sorted[4] !== 3 || sorted[5] !== Infinity ||
    !Number.isNaN(sorted[6]) || !Number.isNaN(sorted[7]) ||
    !Number.isNaN(numbers[0])) throw 'floating order';
const signed = new BigInt64Array([3n, -1n, -9223372036854775808n, 0n, 9223372036854775807n]);
signed.sort();
if (signed.join() !== '-9223372036854775808,-1,0,3,9223372036854775807') {
  throw 'signed BigInt order';
}
const unsigned = new BigUint64Array([-1n, 0n, 2n, 9223372036854775808n]);
if (unsigned.toSorted().join() !== '0,2,9223372036854775808,18446744073709551615') {
  throw 'unsigned BigInt order';
}
print(true);
"#,
    );
}

#[test]
fn comparator_coercion_mutation_throw_detach_and_resize() {
    assert_sort(
        r#"
const mutated = new Int32Array([4, 1, 3, 2]);
let comparisons = 0, coercions = 0;
mutated.sort((a, b) => {
  comparisons++;
  return { valueOf() { coercions++; mutated.fill(99); return a - b; } };
});
if (!comparisons || coercions !== comparisons || mutated.join() !== '1,2,3,4') {
  throw 'snapshot or comparator conversion';
}
const marker = new Error('compare');
const throwing = new Int32Array([4, 3, 2, 1]);
let caught;
try { throwing.sort(() => { throw marker; }); } catch (error) { caught = error; }
if (caught !== marker || throwing.join() !== '4,3,2,1') throw 'comparator throw';
caught = undefined;
try { throwing.sort(() => ({ valueOf() { throw marker; } })); }
catch (error) { caught = error; }
if (caught !== marker || throwing.join() !== '4,3,2,1') throw 'conversion throw';

const detached = new Int32Array([8, 7, 6, 5, 4, 3, 2, 1]);
let detachedCalls = 0;
detached.sort((a, b) => {
  if (detachedCalls++ === 0) __lilaDetachArrayBuffer(detached.buffer);
  return a - b;
});
if (detachedCalls < 4 || detached.length !== 0) throw 'detached comparison continuation';

const fixedBuffer = new ArrayBuffer(16, { maxByteLength: 32 });
const fixed = new Int32Array(fixedBuffer, 0, 4);
fixed.set([4, 3, 2, 1]);
let shrunkCalls = 0;
fixed.sort((a, b) => {
  if (shrunkCalls++ === 0) fixedBuffer.resize(8);
  return a - b;
});
if (shrunkCalls < 2 || new Int32Array(fixedBuffer).join() !== '4,3') {
  throw 'fixed view shrink';
}

const trackingBuffer = new ArrayBuffer(16, { maxByteLength: 32 });
const tracking = new Int32Array(trackingBuffer);
tracking.set([4, 3, 2, 1]);
tracking.sort((a, b) => { trackingBuffer.resize(24); return a - b; });
if (tracking.join() !== '1,2,3,4,0,0') throw 'tracking view growth';
print(true);
"#,
    );
}

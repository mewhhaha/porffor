use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_hex(source: &str, host_surface_policy: HostSurfacePolicy) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions {
                host_surface_policy,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("hexadecimal conversion failed: {error}\n{source}"));
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn computed_hex_roundtrips_all_bytes_and_allocates_exact_fresh_arrays() {
    assert_hex(
        r#"
const digits = '0123456789abcdef';
let input = '';
const bytes = new Uint8Array(256);
for (let value = 0; value < 256; value++) {
  input += digits[value >> 4] + digits[value & 15];
  bytes[value] = value;
}
if (bytes.toHex() !== input) throw new Error('all byte encodings');
const decoded = Uint8Array.fromHex(input);
const upper = Uint8Array.fromHex(input.toUpperCase());
if (decoded === upper || decoded.buffer === upper.buffer) throw new Error('shared result');
for (const result of [decoded, upper]) {
  if (result.length !== 256 || result.byteOffset !== 0 || result.buffer.byteLength !== 256 || result.buffer.resizable) throw new Error('allocation extent');
  if (Object.getPrototypeOf(result) !== Uint8Array.prototype || Object.getPrototypeOf(result.buffer) !== ArrayBuffer.prototype) throw new Error('allocation prototypes');
  for (let value = 0; value < 256; value++) {
    if (result[value] !== value) throw new Error('decoded byte ' + value);
  }
}
decoded[0] = 255;
if (upper[0] !== 0 || bytes[0] !== 0) throw new Error('result aliases');
const empty = Uint8Array.fromHex('');
if (empty.length !== 0 || empty.buffer.byteLength !== 0 || empty.toHex() !== '') throw new Error('empty result');
if (Uint8Array.fromHex('a' + 'A' + '0f').toHex() !== 'aa0f') throw new Error('mixed case');
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn strict_string_and_uint8_brands_do_not_coerce_or_consult_static_receivers() {
    assert_hex(
        r#"
function rejects(operation) {
  let error;
  try { operation(); } catch (received) { error = received; }
  if (!(error instanceof TypeError)) throw new Error('required type rejection');
}
let hooks = 0;
const coercible = new Proxy({}, {get() { hooks++; throw new Error('input coercion'); }});
const target = new Uint8Array(2);
for (const input of [undefined, null, false, 1, 1n, Symbol(), new String('aa'), coercible]) {
  rejects(() => Uint8Array.fromHex(input));
  rejects(() => target.setFromHex(input));
}
if (hooks !== 0) throw new Error('source hooks');
for (const receiver of [undefined, null, false, 1, 1n, Symbol(), {}, [], Uint8Array.prototype,
  new Int8Array(2), new Uint8ClampedArray(2), new Uint16Array(2), new BigInt64Array(2), new Proxy(target, {})]) {
  rejects(() => Uint8Array.prototype.setFromHex.call(receiver, 'aa'));
  rejects(() => Uint8Array.prototype.toHex.call(receiver));
}
class Child extends Uint8Array {}
const child = new Child(1);
child.setFromHex('fe');
if (child[0] !== 254 || child.toHex() !== 'fe') throw new Error('subclass brand');
class Unused extends Uint8Array {
  constructor() { throw new Error('subclass constructor'); }
  static get [Symbol.species]() { throw new Error('species'); }
}
const from = Uint8Array.fromHex;
const ignored = new Proxy(function() {}, {get() { throw new Error('static receiver read'); }});
for (const result of [from('aa'), from.call(null, 'aa'), from.call(ignored, 'aa'), Unused.fromHex('aa')]) {
  if (Object.getPrototypeOf(result) !== Uint8Array.prototype || result[0] !== 170) throw new Error('static receiver');
}
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn utf16_parity_capacity_and_partial_pairs_control_exact_writes() {
    assert_hex(
        r#"
function syntax(operation) {
  let error;
  try { operation(); } catch (received) { error = received; }
  if (!(error instanceof SyntaxError)) throw new Error('required syntax rejection');
}
for (const input of ['aaa', 'aaé', 'aa\uD800']) {
  const target = new Uint8Array([255, 255]);
  syntax(() => target.setFromHex(input));
  if (target[0] !== 255 || target[1] !== 255) throw new Error('odd input wrote prefix');
}
for (const input of ['aagg', 'aaa ', 'aa\0b', 'aaéa']) {
  const target = new Uint8Array([255, 255]);
  syntax(() => target.setFromHex(input));
  if (target[0] !== 170 || target[1] !== 255) throw new Error('partial pair writes');
}
for (const input of ['aagg', 'aa😀', 'aa\uD800a']) {
  const target = new Uint8Array([255]);
  const result = target.setFromHex(input);
  if (result.read !== 2 || result.written !== 1 || target[0] !== 170) throw new Error('capacity suffix');
}
const empty = new Uint8Array(0);
for (const input of ['gg', 'éa', '😀', '\uD800a']) {
  const result = empty.setFromHex(input);
  if (result.read !== 0 || result.written !== 0) throw new Error('empty capacity');
}
for (const input of ['a', 'aaé', '\uD800']) syntax(() => empty.setFromHex(input));
for (const input of ['gg', '0x', ' a', 'éa', '😀', '\uD800a', 'aa😀']) syntax(() => Uint8Array.fromHex(input));
const base = new Uint8Array([255, 255, 255, 255, 255, 255, 255]);
const view = base.subarray(2, 5);
const result = view.setFromHex('aabbccdd');
if (result.read !== 6 || result.written !== 3 || view.toHex() !== 'aabbcc') throw new Error('subarray result');
for (const [index, expected] of [[0,255], [1,255], [2,170], [3,187], [4,204], [5,255], [6,255]]) {
  if (base[index] !== expected) throw new Error('subarray boundary');
}
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn detached_resized_and_immutable_views_use_current_buffer_boundaries() {
    assert_hex(
        r#"
function typeError(operation) {
  let error;
  try { operation(); } catch (received) { error = received; }
  if (!(error instanceof TypeError)) throw new Error('required buffer rejection');
}
const buffer = new ArrayBuffer(6, {maxByteLength:12});
const whole = new Uint8Array(buffer);
const fixed = new Uint8Array(buffer, 2, 2);
const tracking = new Uint8Array(buffer, 2);
whole.setFromHex('010203040506');
buffer.resize(3);
typeError(() => fixed.toHex());
typeError(() => fixed.setFromHex(''));
typeError(() => fixed.setFromHex('a'));
if (tracking.toHex() !== '03') throw new Error('tracking shrink');
const result = tracking.setFromHex('aabb');
if (result.read !== 2 || result.written !== 1 || whole[2] !== 170) throw new Error('tracking capacity');
buffer.resize(1);
typeError(() => tracking.toHex());
typeError(() => tracking.setFromHex('00'));
buffer.resize(8);
if (fixed.toHex() !== '0000' || tracking.toHex() !== '000000000000') throw new Error('regrowth extent');
fixed.setFromHex('aabb');
if (whole[2] !== 170 || whole[3] !== 187 || whole[4] !== 0) throw new Error('regrown writes');
const detached = new ArrayBuffer(2);
const detachedView = new Uint8Array(detached);
const detachedEmpty = new Uint8Array(detached, 0, 0);
const moved = detached.transfer();
if (moved.byteLength !== 2) throw new Error('transfer control');
for (const view of [detachedView, detachedEmpty]) {
  typeError(() => view.toHex());
  typeError(() => view.setFromHex(''));
  typeError(() => view.setFromHex('a'));
}
const mutable = new ArrayBuffer(2);
new Uint8Array(mutable).setFromHex('abcd');
const immutable = new Uint8Array(mutable.transferToImmutable());
if (immutable.toHex() !== 'abcd') throw new Error('immutable read');
typeError(() => immutable.setFromHex(''));
typeError(() => immutable.setFromHex('0000'));
if (immutable[0] !== 171 || immutable[1] !== 205) throw new Error('immutable mutation');
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn shared_and_growing_backing_stores_use_the_buffer_memory_and_view_offsets() {
    assert_hex(
        r#"
const buffer = new SharedArrayBuffer(4, {maxByteLength:8});
const whole = new Uint8Array(buffer);
const fixed = new Uint8Array(buffer, 1, 2);
const tracking = new Uint8Array(buffer, 1);
whole.setFromHex('10203040');
if (fixed.toHex() !== '2030' || tracking.toHex() !== '203040') throw new Error('shared read');
buffer.grow(6);
if (tracking.toHex() !== '2030400000' || fixed.toHex() !== '2030') throw new Error('shared growth');
const result = tracking.setFromHex('aabbccddeeff');
if (result.read !== 10 || result.written !== 5 || fixed[0] !== 170 || fixed[1] !== 187) throw new Error('shared write');
if (whole.toHex() !== '10aabbccddee') throw new Error('shared extent');
const decoded = Uint8Array.fromHex(whole.toHex());
if (decoded.buffer instanceof SharedArrayBuffer || decoded.buffer.byteLength !== 6) throw new Error('private allocation in split memory');
for (let index = 0; index < 6; index++) {
  if (decoded[index] !== whole[index]) throw new Error('split memory copy');
}
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn descriptors_results_and_private_slot_access_ignore_public_property_hooks() {
    assert_hex(
        r#"
for (const [owner, name, length] of [[Uint8Array,'fromHex',1], [Uint8Array.prototype,'setFromHex',1], [Uint8Array.prototype,'toHex',0]]) {
  const descriptor = Object.getOwnPropertyDescriptor(owner, name);
  const method = descriptor.value;
  if (!descriptor.writable || descriptor.enumerable || !descriptor.configurable || method.length !== length || method.name !== name) throw new Error('method descriptor');
  for (const key of ['length', 'name']) {
    const property = Object.getOwnPropertyDescriptor(method, key);
    if (property.writable || property.enumerable || !property.configurable) throw new Error('function descriptor');
  }
  if (Object.getPrototypeOf(method) !== Function.prototype || Object.prototype.hasOwnProperty.call(method, 'prototype')) throw new Error('function prototype');
  let received;
  try { new method(''); } catch (error) { received = error; }
  if (!(received instanceof TypeError)) throw new Error('constructable method');
}
const target = new Uint8Array([1, 2, 3]);
for (const key of ['length', 'byteOffset', 'byteLength', 'buffer', 'constructor']) {
  Object.defineProperty(target, key, {get() { throw new Error('public receiver property ' + key); }});
}
let setters = 0;
for (const key of ['read', 'written']) {
  Object.defineProperty(Object.prototype, key, {set() { setters++; throw new Error('result setter'); }, configurable:true});
}
const result = target.setFromHex('aabb');
for (const key of ['read', 'written']) delete Object.prototype[key];
if (setters !== 0 || target.toHex() !== 'aabb03') throw new Error('private receiver or result');
if (Object.getPrototypeOf(result) !== Object.prototype || Object.keys(result).join(',') !== 'read,written') throw new Error('result object');
for (const [key, expected] of [['read',4], ['written',2]]) {
  const property = Object.getOwnPropertyDescriptor(result, key);
  if (property.value !== expected || !property.writable || !property.enumerable || !property.configurable) throw new Error('result flags');
}
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn foreign_hex_methods_allocate_results_buffers_and_errors_in_their_defining_realm() {
    assert_hex(
        r#"
const other = __lilaCreateRealm().global;
const foreignConstructor = other.Uint8Array;
const foreignBuffer = other.ArrayBuffer;
const from = foreignConstructor.fromHex;
const set = foreignConstructor.prototype.setFromHex;
const to = foreignConstructor.prototype.toHex;
other.Uint8Array = function() { throw new Error('mutable constructor binding'); };
other.ArrayBuffer = function() { throw new Error('mutable buffer binding'); };
const foreign = from.call(Uint8Array, 'aabb');
if (Object.getPrototypeOf(foreign) !== foreignConstructor.prototype || Object.getPrototypeOf(foreign.buffer) !== foreignBuffer.prototype) throw new Error('foreign allocation prototypes');
if (foreign[0] !== 170 || foreign[1] !== 187 || foreign.buffer.byteLength !== 2) throw new Error('foreign allocation bytes');
const local = Uint8Array.fromHex.call(foreignConstructor, 'ccdd');
if (Object.getPrototypeOf(local) !== Uint8Array.prototype || Object.getPrototypeOf(local.buffer) !== ArrayBuffer.prototype) throw new Error('local allocation prototypes');
const result = set.call(local, '0102');
if (Object.getPrototypeOf(result) !== other.Object.prototype || result.read !== 4 || result.written !== 2) throw new Error('foreign result realm');
if (to.call(local) !== '0102' || Uint8Array.prototype.toHex.call(foreign) !== 'aabb') throw new Error('borrowed methods');
const localResult = Uint8Array.prototype.setFromHex.call(foreign, '0304');
if (Object.getPrototypeOf(localResult) !== Object.prototype || foreign[0] !== 3 || foreign[1] !== 4) throw new Error('local result realm');
for (const [operation, prototype] of [
  [() => from(1), other.TypeError.prototype],
  [() => from('gg'), other.SyntaxError.prototype],
  [() => from('a'), other.SyntaxError.prototype],
  [() => set.call({}, ''), other.TypeError.prototype],
  [() => set.call(local, {}), other.TypeError.prototype],
  [() => set.call(local, 'gg'), other.SyntaxError.prototype],
  [() => to.call({}), other.TypeError.prototype]
]) {
  let received;
  try { operation(); } catch (error) { received = error; }
  if (received === undefined || Object.getPrototypeOf(received) !== prototype) throw new Error('foreign error realm');
}
for (const method of [from, set, to]) {
  if (Object.getPrototypeOf(method) !== other.Function.prototype) throw new Error('foreign function realm');
}
true;
"#,
        HostSurfacePolicy::Test262,
    );
}

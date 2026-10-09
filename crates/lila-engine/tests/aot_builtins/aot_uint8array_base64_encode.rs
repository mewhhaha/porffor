use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_base64_encoding(source: &str, host_surface_policy: HostSurfacePolicy) {
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
        .expect("Uint8Array Base64 encoding must execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn initializes_default_options_without_source_literals() {
    assert_base64_encoding(
        "new Uint8Array(0).toBase64() === ''; ",
        HostSurfacePolicy::Product,
    );
}

#[test]
fn encodes_all_bytes_and_both_tail_lengths_with_each_alphabet() {
    assert_base64_encoding(
        r#"
const vectors = [
  [[], ''], [[102], 'Zg=='], [[102, 111], 'Zm8='],
  [[102, 111, 111], 'Zm9v'], [[102, 111, 111, 98], 'Zm9vYg=='],
  [[102, 111, 111, 98, 97], 'Zm9vYmE='],
  [[102, 111, 111, 98, 97, 114], 'Zm9vYmFy']
];
for (const [bytes, expected] of vectors) {
  const array = new Uint8Array(bytes);
  if (array.toBase64() !== expected) throw new Error('RFC 4648 vector');
  if (array.toBase64({omitPadding:true}) !== expected.replaceAll('=', ''))
    throw new Error('omitted padding');
}
const bytes = new Uint8Array(256);
for (let index = 0; index < bytes.length; index++) bytes[index] = index;
const expected = 'AAECAwQFBgcICQoLDA0ODxAREhMUFRYXGBkaGxwdHh8gISIjJCUmJygpKissLS4vMDEyMzQ1Njc4OTo7PD0+P0BBQkNERUZHSElKS0xNTk9QUVJTVFVWV1hZWltcXV5fYGFiY2RlZmdoaWprbG1ub3BxcnN0dXZ3eHl6e3x9fn+AgYKDhIWGh4iJiouMjY6PkJGSk5SVlpeYmZqbnJ2en6ChoqOkpaanqKmqq6ytrq+wsbKztLW2t7i5uru8vb6/wMHCw8TFxsfIycrLzM3Oz9DR0tPU1dbX2Nna29zd3t/g4eLj5OXm5+jp6uvs7e7v8PHy8/T19vf4+fr7/P3+/w==';
if (bytes.toBase64() !== expected) throw new Error('all byte values');
const urlExpected = expected.replaceAll('+', '-').replaceAll('/', '_');
if (bytes.toBase64({alphabet:'base64url'}) !== urlExpected) throw new Error('URL alphabet');
if (bytes.toBase64({alphabet:'base64url', omitPadding:true}) !== urlExpected.slice(0, -2))
  throw new Error('URL padding');
for (const [input, expectedStandard, expectedUrl] of [
  [[255], '/w==', '_w=='], [[199, 239], 'x+8=', 'x-8='], [[199, 239, 242], 'x+/y', 'x-_y']
]) {
  const value = new Uint8Array(input);
  if (value.toBase64() !== expectedStandard || value.toBase64({alphabet:'base64url'}) !== expectedUrl)
    throw new Error('alphabet punctuation');
}
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn validates_receiver_before_options_without_coercing_alphabet() {
    assert_base64_encoding(
        r#"
let accesses = 0;
const options = {get alphabet() { accesses++; throw new Error('unexpected Get'); }};
const encode = Uint8Array.prototype.toBase64;
for (const receiver of [undefined, null, 1, [], {}, new Uint8ClampedArray(1),
  new Int8Array(1), new Uint16Array(1), new BigUint64Array(1),
  new Proxy(new Uint8Array(1), {})]) {
  let received;
  try { encode.call(receiver, options); } catch (error) { received = error; }
  if (!(received instanceof TypeError)) throw new Error('Uint8Array brand');
}
if (accesses !== 0) throw new Error('receiver validation order');
const bytes = new Uint8Array([255]);
for (const options of [null, true, 1, 'base64', Symbol(), 1n]) {
  let received;
  try { bytes.toBase64(options); } catch (error) { received = error; }
  if (!(received instanceof TypeError)) throw new Error('options object');
}
let coerced = 0;
for (const alphabet of [null, true, 1, Symbol(), Object('base64'),
  {[Symbol.toPrimitive]() { coerced++; return 'base64'; }}]) {
  let received;
  try { bytes.toBase64({alphabet, get omitPadding() { accesses++; }}); }
  catch (error) { received = error; }
  if (!(received instanceof TypeError)) throw new Error('primitive alphabet');
}
if (coerced !== 0 || accesses !== 0) throw new Error('alphabet validation order');
function callableOptions() {}
callableOptions.alphabet = 'base64url';
if (bytes.toBase64(callableOptions) !== '_w==') throw new Error('callable options');
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn observes_option_gets_once_before_bytes_and_preserves_abrupt_identity() {
    assert_base64_encoding(
        r#"
const bytes = new Uint8Array([0]);
let order = '';
const truthy = {[Symbol.toPrimitive]() { throw new Error('ToBoolean coerced'); }};
const result = bytes.toBase64({
  get alphabet() { order += 'A'; bytes[0] = 1; return 'base64'; },
  get omitPadding() { order += 'P'; bytes[0] = 255; return truthy; }
});
if (result !== '/w' || order !== 'AP') throw new Error('option and byte order');
const marker = {};
for (const property of ['alphabet', 'omitPadding']) {
  let received;
  const options = Object.defineProperty({}, property, {get() { throw marker; }});
  try { bytes.toBase64(options); } catch (error) { received = error; }
  if (received !== marker) throw new Error('getter abrupt identity');
}
for (const [omitPadding, expected] of [[undefined, '/w=='], [null, '/w=='],
  [0, '/w=='], [NaN, '/w=='], ['', '/w=='], [0n, '/w=='], [1n, '/w'],
  [Symbol(), '/w'], [false, '/w=='], [true, '/w'], [{}, '/w']]) {
  if (bytes.toBase64({omitPadding}) !== expected) throw new Error('ToBoolean');
}
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn reads_the_late_resizable_view_and_rejects_detachment_after_options() {
    assert_base64_encoding(
        r#"
const buffer = new ArrayBuffer(4, {maxByteLength:8});
const tracking = new Uint8Array(buffer, 1);
const fixed = new Uint8Array(buffer, 1, 2);
new Uint8Array(buffer).set([9, 102, 111, 111]);
if (fixed.toBase64() !== 'Zm8=') throw new Error('byte offset');
const grown = tracking.toBase64({get alphabet() {
  buffer.resize(5); new Uint8Array(buffer)[4] = 98; return 'base64';
}});
if (grown !== 'Zm9vYg==') throw new Error('late tracking length');
if (fixed.toBase64({get omitPadding() { buffer.resize(3); return false; }}) !== 'Zm8=')
  throw new Error('fixed view length');
let received;
try { fixed.toBase64({get omitPadding() { buffer.resize(2); }}); }
catch (error) { received = error; }
if (!(received instanceof TypeError)) throw new Error('late out-of-bounds');
if (fixed.toBase64({get alphabet() { buffer.resize(3); return 'base64'; }}) !== 'ZgA=')
  throw new Error('out-of-bounds view revived by options');
const detached = new Uint8Array([1]);
detached.buffer.transfer();
let order = '';
received = undefined;
try { detached.toBase64({
  get alphabet() { order += 'A'; return 'base64'; },
  get omitPadding() { order += 'P'; return false; }
}); } catch (error) { received = error; }
if (!(received instanceof TypeError) || order !== 'AP') throw new Error('detached option order');
const marker = {};
received = undefined;
try { detached.toBase64({get omitPadding() { throw marker; }}); }
catch (error) { received = error; }
if (received !== marker) throw new Error('detached abrupt precedence');
const detaching = new Uint8Array([1]);
received = undefined;
try { detaching.toBase64({get omitPadding() { detaching.buffer.transfer(); }}); }
catch (error) { received = error; }
if (!(received instanceof TypeError)) throw new Error('option detachment');
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn encodes_shared_and_immutable_bytes_without_public_property_reads() {
    assert_base64_encoding(
        r#"
const shared = new SharedArrayBuffer(4, {maxByteLength:8});
const bytes = new Uint8Array(shared);
bytes.set([9, 102, 111, 111]);
const view = new Uint8Array(shared, 1);
const encoded = view.toBase64({get alphabet() {
  shared.grow(5); new Uint8Array(shared)[4] = 98; return 'base64';
}});
if (encoded !== 'Zm9vYg==') throw new Error('shared buffer bytes');
const immutable = new ArrayBuffer(3);
new Uint8Array(immutable).set([102, 111, 111]);
const frozenBytes = new Uint8Array(immutable.transferToImmutable());
if (frozenBytes.toBase64() !== 'Zm9v') throw new Error('immutable reading');
for (const key of ['buffer', 'byteOffset', 'byteLength', 'length', 'constructor']) {
  Object.defineProperty(frozenBytes, key, {get() { throw new Error('public property read'); }});
}
if (frozenBytes.toBase64() !== 'Zm9v') throw new Error('internal view fields');
class Sub extends Uint8Array {}
const subclass = new Sub([255]);
if (subclass.toBase64() !== '/w==') throw new Error('subclass receiver');
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn uses_method_realm_errors_and_the_standard_property_contract() {
    assert_base64_encoding(
        r#"
const descriptor = Object.getOwnPropertyDescriptor(Uint8Array.prototype, 'toBase64');
if (!descriptor.writable || descriptor.enumerable || !descriptor.configurable)
  throw new Error('method descriptor');
const encode = descriptor.value;
if (encode.name !== 'toBase64' || encode.length !== 0 || Object.hasOwn(encode, 'prototype'))
  throw new Error('function contract');
let received;
try { new encode(); } catch (error) { received = error; }
if (!(received instanceof TypeError)) throw new Error('nonconstructor');
const realm = __lilaCreateRealm();
const foreign = realm.global.Uint8Array.prototype.toBase64;
for (const [receiver, options] of [[{}, undefined], [new Uint8Array(1), {alphabet:'invalid'}]]) {
  received = undefined;
  try { foreign.call(receiver, options); } catch (error) { received = error; }
  if (!(received instanceof realm.global.TypeError) || received instanceof TypeError)
    throw new Error('method error realm');
}
const foreignBytes = new realm.global.Uint8Array([255]);
if (encode.call(foreignBytes) !== '/w==' || foreign.call(new Uint8Array([255])) !== '/w==')
  throw new Error('cross-realm receivers');
true;
"#,
        HostSurfacePolicy::Test262,
    );
}

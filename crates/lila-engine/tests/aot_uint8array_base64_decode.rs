use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_base64_decode(source: &str, host_surface_policy: HostSurfacePolicy) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
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
        .expect("Uint8Array Base64 decoding must execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn static_decoding_preserves_exact_bytes_buffers_and_intrinsic_identity() {
    assert_base64_decode(
        r#"
const vectors = [
  ['', []], ['Zg==', [102]], ['Zm8=', [102, 111]],
  ['Zm9v', [102, 111, 111]], ['Zm9vYg==', [102, 111, 111, 98]],
  ['Zm9vYmE=', [102, 111, 111, 98, 97]], ['Zm9vYmFy', [102, 111, 111, 98, 97, 114]]
];
for (const pair of vectors) {
  const decoded = Uint8Array.fromBase64(pair[0]);
  if (decoded.length !== pair[1].length || decoded.buffer.byteLength !== pair[1].length ||
      decoded.buffer.detached || decoded.byteOffset !== 0 ||
      Object.getPrototypeOf(decoded) !== Uint8Array.prototype ||
      Object.getPrototypeOf(decoded.buffer) !== ArrayBuffer.prototype) throw new Error('allocation');
  for (let i = 0; i < decoded.length; i++) {
    if (decoded[i] !== pair[1][i]) throw new Error('decoded byte');
  }
}
let source = '';
for (const character of ['x', '-', '_', 'y']) source += character;
const url = Uint8Array.fromBase64(source, {alphabet: 'base64url'});
const ordinary = Uint8Array.fromBase64('x+/y');
if (url[0] !== 199 || url[1] !== 239 || url[2] !== 242 || ordinary[2] !== 242) throw new Error('alphabet');
class Subclass extends Uint8Array { constructor() { throw new Error('subclass construction'); } }
const inherited = Subclass.fromBase64('AQID');
const detachedMethod = Uint8Array.fromBase64;
const detachedCall = detachedMethod.call(() => { throw new Error('receiver called'); }, 'AQID');
if (Object.getPrototypeOf(inherited) !== Uint8Array.prototype ||
    Object.getPrototypeOf(detachedCall) !== Uint8Array.prototype || detachedCall[2] !== 3) throw new Error('static receiver');
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn final_chunks_distinguish_modes_padding_and_overflow_bits() {
    assert_base64_decode(
        r#"
function syntax(source, mode) {
  let caught;
  try { Uint8Array.fromBase64(source, {lastChunkHandling: mode}); } catch (error) { caught = error; }
  if (!(caught instanceof SyntaxError)) throw new Error('syntax accepted');
}
for (const source of ['Zg==', 'Z g\t=\n=\r\f ']) {
  for (const mode of ['loose', 'strict', 'stop-before-partial']) {
    const decoded = Uint8Array.fromBase64(source, {lastChunkHandling: mode});
    if (decoded.length !== 1 || decoded[0] !== 102) throw new Error('padded final');
  }
}
for (const source of ['Zh==', 'AAB=']) {
  syntax(source, 'strict');
  const loose = Uint8Array.fromBase64(source);
  const stopped = Uint8Array.fromBase64(source, {lastChunkHandling: 'stop-before-partial'});
  if (loose.length !== stopped.length || loose[0] !== stopped[0]) throw new Error('unused bits');
}
if (Uint8Array.fromBase64('Zg').length !== 1 || Uint8Array.fromBase64('Zm8')[1] !== 111) throw new Error('loose unpadded');
for (const source of ['A', 'AA=', 'ABCDA', 'ABCDAA=']) {
  syntax(source, 'loose');
  syntax(source, 'strict');
  const stopped = Uint8Array.fromBase64(source, {lastChunkHandling: 'stop-before-partial'});
  if (stopped.length !== (source.startsWith('ABCD') ? 3 : 0)) throw new Error('partial stop');
}
for (const source of ['=', 'A=', 'A==', 'AA===', 'AAA==', 'AAAA=', 'AA=A', 'Zg==x', 'Zg\v==', 'Zg\u00a0==', 'Zg\u2028==', 'Z−==', 'Z＋==']) {
  for (const mode of ['loose', 'strict', 'stop-before-partial']) syntax(source, mode);
}
syntax('Zg', 'strict');
syntax('Zm8', 'strict');
let wrongAlphabet;
try { Uint8Array.fromBase64('x+/y', {alphabet: 'base64url'}); } catch (error) { wrongAlphabet = error; }
if (!(wrongAlphabet instanceof SyntaxError)) throw new Error('mixed alphabet');
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn capacity_stops_preserve_committed_read_counts_and_validate_lookahead() {
    assert_base64_decode(
        r#"
function check(source, capacity, mode, read, written) {
  const target = new Uint8Array(capacity);
  target.fill(255);
  const result = target.setFromBase64(source, {lastChunkHandling: mode});
  if (result.read !== read || result.written !== written) throw new Error('read or written');
  for (let i = 0; i < capacity; i++) {
    if (target[i] !== (i < written ? 0 : 255)) throw new Error('write boundary');
  }
  if (Object.getPrototypeOf(result) !== Object.prototype || Object.keys(result).join(',') !== 'read,written') throw new Error('result object');
  for (const key of ['read', 'written']) {
    const descriptor = Object.getOwnPropertyDescriptor(result, key);
    if (!descriptor.writable || !descriptor.enumerable || !descriptor.configurable) throw new Error('result descriptor');
  }
}
for (const mode of ['loose', 'strict', 'stop-before-partial']) {
  check('#', 0, mode, 0, 0);
  check('AAAA#', 3, mode, 4, 3);
  check('AAA', 1, mode, 0, 0);
  check('AAAA', 2, mode, 0, 0);
  check('AA==', 1, mode, 4, 1);
  check('AAA=', 2, mode, 4, 2);
  check('AAAA  ', 3, mode, 4, 3);
  check('AAAA  ', 4, mode, 6, 3);
  for (const pair of [['AA#', 1], ['AAA#', 2], ['AA==#', 1]]) {
    const target = new Uint8Array(pair[1]);
    target.fill(255);
    let caught;
    try { target.setFromBase64(pair[0], {lastChunkHandling: mode}); } catch (error) { caught = error; }
    if (!(caught instanceof SyntaxError) || target[0] !== 255) throw new Error('lookahead validation');
  }
}
check('AA', 1, 'loose', 2, 1);
check('AAA', 2, 'loose', 3, 2);
check('AA=', 1, 'stop-before-partial', 0, 0);
check('AAAA AA', 5, 'stop-before-partial', 4, 3);
check('AAAA AA=', 5, 'stop-before-partial', 4, 3);
check('AAAA AA==', 4, 'strict', 9, 4);
check(' \tAA==', 1, 'strict', 6, 1);
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn syntax_errors_leave_prior_chunks_written_and_the_rejected_chunk_untouched() {
    assert_base64_decode(
        r#"
for (const pair of [
  ['MjYyZg===', 'loose'], ['MjYyZg', 'strict'], ['MjYyZh==', 'strict'],
  ['MjYyZm.9v', 'loose'], ['MjYyAA#', 'stop-before-partial']
]) {
  const backing = new Uint8Array([17, 255, 255, 255, 255, 255, 19]);
  const target = backing.subarray(1, 6);
  let caught;
  try { target.setFromBase64(pair[0], {lastChunkHandling: pair[1]}); } catch (error) { caught = error; }
  if (!(caught instanceof SyntaxError)) throw new Error('expected syntax error');
  if (backing[0] !== 17 || backing[1] !== 50 || backing[2] !== 54 || backing[3] !== 50 ||
      backing[4] !== 255 || backing[5] !== 255 || backing[6] !== 19) throw new Error('prefix publication');
}
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn receiver_source_and_options_follow_get_order_without_string_coercion() {
    assert_base64_decode(
        r#"
function typeError(action) {
  let caught;
  try { action(); } catch (error) { caught = error; }
  if (!(caught instanceof TypeError)) throw new Error('expected type error');
}
let trace = '';
const options = {
  get alphabet() { trace += 'A'; return 'base64'; },
  get lastChunkHandling() { trace += 'L'; return 'loose'; }
};
const receiver = new Uint8Array(3);
receiver.setFromBase64('AQID', options);
if (trace !== 'AL' || receiver[2] !== 3) throw new Error('option order');
trace = '';
const boxed = {toString() { trace += 'S'; return 'AAAA'; }};
typeError(() => Uint8Array.fromBase64(boxed, options));
typeError(() => receiver.setFromBase64(boxed, options));
for (const invalid of [[], new Int8Array(3), new Uint8ClampedArray(3), new Proxy(receiver, {})]) {
  typeError(() => Uint8Array.prototype.setFromBase64.call(invalid, boxed, options));
}
if (trace !== '') throw new Error('premature coercion or options');
for (const value of [null, false, 1, 'options']) typeError(() => Uint8Array.fromBase64('AAAA', value));
typeError(() => receiver.setFromBase64('AAAA', {alphabet: boxed, get lastChunkHandling() { trace += 'L'; }}));
typeError(() => receiver.setFromBase64('AAAA', {alphabet: 'base64', lastChunkHandling: boxed}));
if (trace !== '') throw new Error('option coercion');
const marker = {};
let caught;
try { receiver.setFromBase64('AAAA', {get alphabet() { throw marker; }, get lastChunkHandling() { trace += 'L'; }}); }
catch (error) { caught = error; }
if (caught !== marker || trace !== '') throw new Error('alphabet throw identity');
caught = undefined;
try { Uint8Array.fromBase64('AAAA', {get alphabet() { trace += 'A'; return 'base64'; }, get lastChunkHandling() { throw marker; }}); }
catch (error) { caught = error; }
if (caught !== marker || trace !== 'A') throw new Error('last chunk throw identity');
const detached = new Uint8Array(0);
detached.buffer.transfer();
trace = '';
typeError(() => detached.setFromBase64('', options));
if (trace !== 'AL') throw new Error('buffer validated before options');
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn buffer_witness_observes_getter_resizes_and_detachment_with_fixed_view_recovery() {
    assert_base64_decode(
        r#"
function typeError(action) {
  let caught;
  try { action(); } catch (error) { caught = error; }
  if (!(caught instanceof TypeError)) throw new Error('expected invalid view');
}
const growing = new ArrayBuffer(6, {maxByteLength: 12});
const tracking = new Uint8Array(growing, 2);
let reads = 0;
const progress = tracking.setFromBase64('Zm9vYmFy', {
  get alphabet() { reads++; growing.resize(8); return 'base64'; },
  get lastChunkHandling() { reads++; return 'loose'; }
});
if (reads !== 2 || progress.read !== 8 || progress.written !== 6 || tracking[0] !== 102 || tracking[5] !== 114) throw new Error('growth witness');
const shrinking = new ArrayBuffer(6, {maxByteLength: 12});
const short = new Uint8Array(shrinking, 2);
const shortened = short.setFromBase64('Zm8=', {get lastChunkHandling() { shrinking.resize(4); return 'strict'; }});
if (shortened.written !== 2 || short[0] !== 102 || short[1] !== 111) throw new Error('shrink witness');
const fixedBuffer = new ArrayBuffer(6, {maxByteLength: 12});
const fixed = new Uint8Array(fixedBuffer, 2, 3);
reads = 0;
typeError(() => fixed.setFromBase64('AAAA', {
  get alphabet() { reads++; fixedBuffer.resize(4); return 'base64'; },
  get lastChunkHandling() { reads++; return 'loose'; }
}));
if (reads !== 2) throw new Error('fixed view validation order');
fixedBuffer.resize(6);
if (fixed.setFromBase64('AQID').written !== 3 || fixed[2] !== 3) throw new Error('fixed view recovery');
const detaching = new Uint8Array(3);
typeError(() => detaching.setFromBase64('AQID', {get lastChunkHandling() { detaching.buffer.transfer(); return 'loose'; }}));
const shadowed = new Uint8Array(3);
Object.defineProperty(shadowed, 'length', {get() { throw new Error('public length'); }});
Object.defineProperty(shadowed, 'byteOffset', {get() { throw new Error('public offset'); }});
Object.defineProperty(shadowed, 'buffer', {get() { throw new Error('public buffer'); }});
if (shadowed.setFromBase64('AQID').written !== 3 || shadowed[2] !== 3) throw new Error('private slots');
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn temporary_and_shared_destinations_use_their_own_memories_and_preserve_view_bounds() {
    assert_base64_decode(
        r#"
const shared = new SharedArrayBuffer(8, {maxByteLength: 12});
const bytes = new Uint8Array(shared);
bytes.fill(255);
const view = new Uint8Array(shared, 2, 3);
const result = view.setFromBase64('AQIDBAUG');
if (result.read !== 4 || result.written !== 3 || bytes[1] !== 255 || bytes[2] !== 1 ||
    bytes[3] !== 2 || bytes[4] !== 3 || bytes[5] !== 255) throw new Error('shared view');
const plain = Uint8Array.fromBase64('AQIDBAUG');
if (plain.length !== 6 || plain.buffer.byteLength !== 6 || plain[0] !== 1 || plain[5] !== 6) throw new Error('temporary memory');
const tracking = new Uint8Array(shared, 8);
const grown = tracking.setFromBase64('AQID', {get alphabet() { shared.grow(11); return 'base64'; }});
if (grown.written !== 3 || tracking[2] !== 3 || plain[5] !== 6) throw new Error('shared growth');
const immutable = new Uint8Array(new ArrayBuffer(3).transferToImmutable());
let caught;
try { immutable.setFromBase64('AQID'); } catch (error) { caught = error; }
if (!(caught instanceof TypeError) || immutable[0] !== 0) throw new Error('immutable write');
true;
"#,
        HostSurfacePolicy::Product,
    );
}

#[test]
fn retained_foreign_methods_own_allocations_results_and_errors_after_global_replacement() {
    assert_base64_decode(
        r#"
const foreign = __lilaCreateRealm().global;
const decode = foreign.Uint8Array.fromBase64;
const set = foreign.Uint8Array.prototype.setFromBase64;
const prototype = foreign.Uint8Array.prototype;
const bufferPrototype = foreign.ArrayBuffer.prototype;
const objectPrototype = foreign.Object.prototype;
const syntaxPrototype = foreign.SyntaxError.prototype;
const typePrototype = foreign.TypeError.prototype;
foreign.Uint8Array = function replacement() { throw new Error('replacement Uint8Array'); };
foreign.ArrayBuffer = function replacement() { throw new Error('replacement ArrayBuffer'); };
foreign.SyntaxError = function replacement() { throw new Error('replacement SyntaxError'); };
foreign.TypeError = function replacement() { throw new Error('replacement TypeError'); };
const decoded = decode.call(null, 'AQID');
if (Object.getPrototypeOf(decoded) !== prototype || Object.getPrototypeOf(decoded.buffer) !== bufferPrototype || decoded[2] !== 3) throw new Error('foreign allocation');
const target = new Uint8Array(4);
const progress = set.call(target, 'AQID');
if (Object.getPrototypeOf(progress) !== objectPrototype || progress.read !== 4 || target[2] !== 3) throw new Error('foreign result');
let caught;
try { decode('#'); } catch (error) { caught = error; }
if (Object.getPrototypeOf(caught) !== syntaxPrototype) throw new Error('foreign syntax error');
caught = undefined;
try { decode('AAAA', {lastChunkHandling: 'invalid'}); } catch (error) { caught = error; }
if (Object.getPrototypeOf(caught) !== typePrototype) throw new Error('foreign option error');
target.fill(255);
caught = undefined;
try { set.call(target, 'AQID#'); } catch (error) { caught = error; }
if (Object.getPrototypeOf(caught) !== syntaxPrototype || target[0] !== 1 || target[2] !== 3 || target[3] !== 255) throw new Error('foreign partial throw');
true;
"#,
        HostSurfacePolicy::Test262,
    );
}

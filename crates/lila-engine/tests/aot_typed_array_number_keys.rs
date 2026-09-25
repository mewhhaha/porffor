//! A runtime Number key reaches a TypedArray's integer-indexed `[[Get]]`,
//! `[[Set]]` and `[[HasProperty]]` without its String ever being built, and
//! String-keyed access runs CanonicalNumericIndexString without allocating.
//!
//! Lila's heap is a bump allocator with no collector and a 1 GiB Wasm store
//! cap, so a per-access allocation is a leak that eventually traps. Before the
//! Number-key path existed, the copy loop below (the shape of Test262's
//! `harness/testTypedArray.js` `copyIntoArrayBuffer`) allocated several hundred
//! bytes per element and exhausted the heap after a few million copies.

use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_wasm_true(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(120_000),
                ..RunOptions::default()
            },
        )
        .expect("TypedArray Number-key regression must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn copying_millions_of_elements_stays_within_the_bounded_heap() {
    // 40 rounds over 100,000 bytes: 4,000,000 element reads and writes plus
    // as many `in` tests, `length` reads and compound updates, all through
    // dynamically typed views and function-local bindings.
    assert_wasm_true(
        r#"
function copyInto(destBuffer, srcBuffer) {
  var destView = new Uint8Array(destBuffer);
  var srcView = new Uint8Array(srcBuffer);
  for (var i = 0; i < srcView.length; i++) destView[i] = srcView[i];
  return destBuffer;
}
function touch(view) {
  var present = 0;
  for (var i = 0; i < view.length; i++) {
    if (i in view) present++;
    view[i] += 1;
    view[i]++;
  }
  return present;
}
function run() {
  var source = new ArrayBuffer(100000);
  var bytes = new Uint8Array(source);
  for (var i = 0; i < bytes.length; i++) bytes[i] = i & 0xff;
  var ok = true;
  for (var round = 0; round < 40; round++) {
    var dest = copyInto(new ArrayBuffer(100000, { maxByteLength: 200000 }), source);
    var view = new Uint8Array(dest);
    ok = ok && view[99999] === (99999 & 0xff) && view[12345] === (12345 & 0xff);
  }
  var counted = touch(bytes);
  return ok && counted === 100000 && bytes[0] === 2 && bytes[255] === 1;
}
run();
"#,
    );
}

#[test]
fn number_keys_keep_canonical_numeric_index_semantics() {
    assert_wasm_true(
        r#"
function check() {
  var ta = new Int16Array([10, 20, 30]);
  var keys = [-0, 0, 2, 3, -1, 1.5, NaN, Infinity, -Infinity, 4294967296, 1e21];
  var got = [];
  for (var i = 0; i < keys.length; i++) {
    var k = keys[i];
    got.push(String(ta[k]) + ':' + (k in ta));
  }
  var ok = got.join(',') ===
    '10:true,10:true,30:true,undefined:false,undefined:false,undefined:false,' +
    'undefined:false,undefined:false,undefined:false,undefined:false,undefined:false';

  // -0 names element 0; every other non-index Number is a canonical numeric
  // key that is not a valid integer index: the value is still coerced, nothing
  // is stored, and no ordinary property appears.
  var coerced = 0;
  var value = { valueOf: function () { coerced++; return 7; } };
  ta[-0] = value;
  var invalid = [-1, 1.5, NaN, Infinity, 3, 1e21];
  for (var j = 0; j < invalid.length; j++) ta[invalid[j]] = value;
  ok = ok && ta[0] === 7 && coerced === 7 && Object.keys(ta).join() === '0,1,2';
  ok = ok && Object.getOwnPropertyNames(ta).length === 3 && !ta.hasOwnProperty('-1');

  // String keys: "-0" is canonical and invalid; "01" and "1.0" are ordinary.
  ta['-0'] = 99;
  ta['01'] = 5;
  ta['1.0'] = 6;
  ok = ok && ta['-0'] === undefined && !('-0' in ta) && ta['01'] === 5 && ta['1.0'] === 6;
  ok = ok && ta['1'] === 20 && ('2' in ta) && !('3' in ta) && ta['1e21'] === undefined;

  // Compound assignment reads and writes the same element.
  var k2 = 2;
  ta[k2] += 5;
  ta[k2]++;
  ok = ok && ta[2] === 36;

  // BigInt arrays reject a Number value even at an invalid index.
  var big = new BigInt64Array(1);
  var bigThrew = false;
  try { big[5] = 1; } catch (error) { bigThrew = error instanceof TypeError; }
  big[0] = 3n;
  ok = ok && bigThrew && big[0] === 3n && big[-0] === 3n;
  return ok;
}
check();
"#,
    );
}

#[test]
fn number_keys_respect_detached_immutable_proxy_and_prototype_receivers() {
    assert_wasm_true(
        r#"
function check() {
  var ok = true;

  // Detached: reads are undefined, `in` is false, writes coerce then no-op.
  var buffer = new ArrayBuffer(4);
  var ta = new Uint8Array(buffer);
  ta[1] = 9;
  buffer.transfer();
  var coerced = false;
  ta[1] = { valueOf: function () { coerced = true; return 3; } };
  var i = 1;
  ok = ok && ta[i] === undefined && !(i in ta) && coerced && ta.length === 0;

  // A value coercion that detaches the buffer makes the store a no-op.
  var buffer2 = new ArrayBuffer(4);
  var ta2 = new Uint8Array(buffer2);
  ta2[0] = { valueOf: function () { buffer2.transfer(); return 5; } };
  ok = ok && ta2.length === 0 && ta2[0] === undefined;

  // Immutable: [[Set]] returns false before the value is coerced.
  var frozen = new Uint8Array(new Uint8Array([1, 2]).buffer.transferToImmutable());
  var touched = false;
  var probe = { valueOf: function () { touched = true; return 8; } };
  frozen[0] = probe;
  var strictThrew = false;
  try { (function () { 'use strict'; frozen[1] = probe; })(); }
  catch (error) { strictThrew = error instanceof TypeError; }
  ok = ok && frozen[0] === 1 && frozen[1] === 2 && !touched && strictThrew;

  // A TypedArray on the prototype chain: integer-indexed [[Get]]/[[HasProperty]]
  // answer through the prototype, and [[Set]] with a different receiver defines
  // an own property on the receiver only for a valid index.
  var proto = new Uint8Array([4, 5]);
  var child = Object.create(proto);
  var j = 1;
  ok = ok && child[j] === 5 && (j in child) && !(2 in child);
  child[j] = 6;
  child[7] = 1;
  ok = ok && child.hasOwnProperty('1') && child[1] === 6 && proto[1] === 5;
  ok = ok && !child.hasOwnProperty('7') && child[7] === undefined;

  // A Proxy around a TypedArray sees the String key in its traps.
  var seen = [];
  var proxy = new Proxy(new Uint8Array(2), {
    get: function (target, key, receiver) { seen.push(typeof key + ':' + key); return target[key]; },
    set: function (target, key, value) { seen.push('set:' + typeof key + ':' + key); target[key] = value; return true; },
    has: function (target, key) { seen.push('has:' + typeof key + ':' + key); return key in target; }
  });
  var n = 1;
  proxy[n] = 3;
  var read = proxy[n];
  var has = n in proxy;
  ok = ok && read === 3 && has && seen.join(',') === 'set:string:1,string:1,has:string:1';
  return ok;
}
check();
"#,
    );
}

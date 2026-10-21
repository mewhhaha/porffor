use super::*;

#[test]
fn wasm_agents_share_buffers_and_deliver_reports() {
    let source = format!(
        r#"{TEST262_AGENT_PRELUDE}
$262.agent.start("$262.agent.receiveBroadcast(function (sab) {{ var view = new Int32Array(sab); $262.agent.report(Atomics.add(view, 0, 1) + 1); $262.agent.leaving(); }});");
var sab = new SharedArrayBuffer(4);
new Int32Array(sab)[0] = 41;
$262.agent.broadcast(sab);
var report = null;
for (var attempt = 0; attempt < 10000 && report === null; attempt++) {{
  $262.agent.sleep(1);
  report = $262.agent.getReport();
}}
report;
"#
    );
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_wasm_aot_script_with_agents(
            &source,
            test262_compile_options(),
            Some(SHARED_AGENT_STARTUP_EXECUTION_TIMEOUT_MS),
            true,
            TEST262_AGENT_PRELUDE.to_string(),
        )
        .expect("agent should mutate the shared buffer and report its result");

    assert!(outcome.note.contains("42"), "{}", outcome.note);
}

#[test]
fn observed_wasm_completion_is_typed_and_captures_print_once() {
    let delegated_lines = Arc::new(Mutex::new(Vec::new()));
    let engine = engine_with_captured_prints(Arc::clone(&delegated_lines));
    let outcome = engine
        .observe_script(
            r#"
                print("observed line");
                ({
                  secret: "t25-private-property",
                  toString() { print("coerced"); return "wrong"; }
                });
                "#,
            CompileOptions::default(),
            RunOptions::default(),
        )
        .expect("an object completion should be observed without coercion");

    assert_eq!(
        outcome.completion,
        ObservedCompletion::Normal(ObservedJsValue::Object)
    );
    assert_eq!(
        outcome.output_events,
        vec![HostOutputEvent::PrintLine("observed line".to_string())]
    );
    assert_eq!(outcome.note, "wasm-aot normal completion (object)");
    let debug = format!("{outcome:?}");
    assert!(!debug.contains("handle@"));
    assert!(!debug.contains("t25-private-property"));
    assert_eq!(
        *delegated_lines.lock().expect("capture mutex poisoned"),
        vec!["observed line".to_string()]
    );
}

#[test]
fn wasm_backend_json_parse_grows_dynamic_arrays_amortized() {
    let outcome = engine()
        .run_script(
            r#"
var text = "[";
for (var i = 0; i < 257; i++) text += (i === 0 ? "" : ",") + i;
text += "]";
var value = JSON.parse(text, false);
value.length === 257 && value[0] === 0 && value[127] === 127 && value[256] === 256;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("dynamic JSON arrays should grow without reparsing or fixed limits");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_json_stringify_snapshots_replacer_array_before_serializing_values() {
    let outcome = engine()
        .run_script(
            r#"
var conversions = 0;
var accesses = [];
var boxedKey = new String("a");
boxedKey.toString = function() {
  conversions += 1;
  return "a";
};
var target = [boxedKey];
var replacer = new Proxy(target, {
  get: function(target, key) {
    accesses.push(key);
    return target[key];
  }
});
var value = {};
Object.defineProperty(value, "a", {
  enumerable: true,
  get: function() {
    target[0] = "b";
    return { a: 1, b: 2 };
  }
});
[
  JSON.stringify(value, replacer),
  accesses.join(","),
  conversions
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("JSON stringify should snapshot an array replacer once");
    assert!(
        outcome
            .note
            .contains("string({\"a\":{\"a\":1}}|length,0|1)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_catches_arraybuffer_dataview_constructor_toindex_abrupts() {
    for source in [
        "try { new ArrayBuffer({ valueOf: function() { throw \"x\"; } }); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
        "try { new ArrayBuffer(0, { maxByteLength: { valueOf: function() { throw \"x\"; } } }); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
        "try { new DataView(new ArrayBuffer(8), -1); 0; } catch (e) { e.name === \"RangeError\" ? 123 : 1; }",
        "try { new DataView(new ArrayBuffer(8), 0, -1); 0; } catch (e) { e.name === \"RangeError\" ? 123 : 1; }",
        "try { new Uint8Array(new ArrayBuffer(8), { valueOf: function() { throw \"x\"; } }); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
        "try { new Uint8Array(new ArrayBuffer(8), 0, { valueOf: function() { throw \"x\"; } }); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
        "var view = new DataView(new ArrayBuffer(8)); try { view.getUint8({ valueOf: function() { throw \"x\"; } }); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
        "var view = new DataView(new ArrayBuffer(8)); try { view.setUint16({ valueOf: function() { throw \"x\"; } }, 1); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
        "var view = new DataView(new ArrayBuffer(8)); try { view.getBigInt64({ valueOf: function() { throw \"x\"; } }); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
        "try { new ArrayBuffer(1, { maxByteLength: 4 }).resize({ valueOf: function() { throw \"x\"; } }); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
        "try { new ArrayBuffer(1).transfer({ valueOf: function() { throw \"x\"; } }); 0; } catch (e) { e === \"x\" ? 123 : 1; }",
    ] {
        let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!("constructor ToIndex abrupt case should run for `{source}`: {err:?}")
            });
        assert!(
            outcome.note.contains("number(123"),
            "source: {source}, note: {}",
            outcome.note
        );
    }
}

#[test]
fn wasm_backend_typed_array_buffer_arguments_use_private_current_state() {
    let source = r#"
const buffer = new ArrayBuffer(8);
buffer.$ArrayBufferDataPtr = 0;
buffer.$ArrayBufferByteLength = 0;
Object.defineProperty(buffer, "length", {
  get() { throw "buffer length property must not be read"; }
});
Object.defineProperty(buffer, Symbol.iterator, {
  get() { throw "buffer iterator property must not be read"; }
});
const privateView = new Uint16Array(buffer);

const forged = {
  $ArrayBufferDataPtr: 1,
  $ArrayBufferByteLength: 8,
  length: 2,
  0: 7,
  1: 8
};
const forgedView = new Uint8Array(forged);

const detached = new ArrayBuffer(4);
const conversionOrder = [];
let detachedTypeError = false;
try {
  new Uint8Array(
    detached,
    { valueOf() { conversionOrder.push("offset"); __scriptwDetachArrayBuffer(detached); return 0; } },
    { valueOf() { conversionOrder.push("length"); return 1; } }
  );
} catch (error) {
  detachedTypeError = error instanceof TypeError;
}

const shrinking = new ArrayBuffer(8, { maxByteLength: 8 });
let shrinkRangeError = false;
try {
  new Uint8Array(shrinking, 0, {
    valueOf() { shrinking.resize(1); return 2; }
  });
} catch (error) {
  shrinkRangeError = error instanceof RangeError;
}

const growing = new ArrayBuffer(1, { maxByteLength: 4 });
const grownView = new Uint8Array(growing, 0, {
  valueOf() { growing.resize(4); return 4; }
});
const tracking = new Uint8Array(growing, 0, undefined);
growing.resize(2);
const fixed = new Uint8Array(growing, 0, 1);
growing.resize(4);

const shared = new BigInt64Array(new SharedArrayBuffer(16), 8);
[
  privateView.length,
  forgedView.join(","),
  conversionOrder.join(","),
  detachedTypeError,
  shrinkRangeError,
  grownView.length,
  tracking.length,
  fixed.length,
  shared.length
].join("|");
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray buffer arguments should use branded private current state");
    assert!(
        outcome
            .note
            .contains("string(4|7,8|offset,length|true|true|4|4|1|1)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_buffer_maximum_capacity_preserves_constructor_effects() {
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/testdata/arraybuffer-maximum-capacity.js"
    ));
    for directive in ["", "\"use strict\";\n"] {
        let outcome = engine()
            .run_script(
                &format!("{directive}{source}"),
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("ArrayBuffer maximum admission preserves constructor ordering and backing");
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }
}

#[test]
fn wasm_backend_typed_array_length_reports_backing_store_allocation_failure() {
    let source = r#"
const huge = 1024 * 1024 * 1024;
function checkRangeError(operation) {
  let caught;
  try { operation(); } catch (error) { caught = error; }
  if (!(caught instanceof RangeError)) throw "missing catchable allocation failure";
}
checkRangeError(function () { new Uint8Array(huge); });
checkRangeError(function () { new Float64Array(huge / 8); });
let prototypeReads = 0;
const target = new Proxy(function Target() {}, {
  get(object, key, receiver) {
    if (key === "prototype") { prototypeReads++; return Object.prototype; }
    return Reflect.get(object, key, receiver);
  }
});
checkRangeError(function () { Reflect.construct(ArrayBuffer, [huge], target); });
if (prototypeReads !== 1) throw "allocation failure preceded prototype selection";
const original = {};
const abruptTarget = new Proxy(function Target() {}, {
  get(object, key, receiver) {
    if (key === "prototype") throw original;
    return Reflect.get(object, key, receiver);
  }
});
let prototypeError;
try { Reflect.construct(ArrayBuffer, [huge], abruptTarget); }
catch (error) { prototypeError = error; }
if (prototypeError !== original) throw "allocation replaced the original prototype exception";
const buffer = new ArrayBuffer(8, {maxByteLength: huge});
const view = new Uint8Array(buffer);
view[0] = 41;
checkRangeError(function () { buffer.resize(huge); });
if (buffer.byteLength !== 8 || view[0] !== 41) throw "failed resize changed the original backing";
checkRangeError(function () { buffer.transfer(huge); });
if (buffer.byteLength !== 8 || view[0] !== 41) throw "failed transfer detached the original backing";
const small = new Uint8Array(16);
if (small.length !== 16 || small[0] !== 0 || small[15] !== 0) throw "successful backing lost zero initialization";
true;
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray backing-store allocation failure should be catchable");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_length_zeroes_reused_heap_storage() {
    let source = r#"
let text = "12345678901234567890";
text += "1234567890";
const parsed = Number(text);
const values = new Uint8Array(8);
parsed > 0 &&
  values[0] === 0 && values[1] === 0 && values[2] === 0 && values[3] === 0 &&
  values[4] === 0 && values[5] === 0 && values[6] === 0 && values[7] === 0;
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray length construction should zero reused heap storage");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_bigint_typed_array_sources_copy_into_array_buffer_backed_views() {
    let source = r#"
const shared = new SharedArrayBuffer(8);
const signedSource = new BigInt64Array(shared);
signedSource[0] = 7n;
const signedToSigned = new BigInt64Array(signedSource);
const signedToUnsigned = new BigUint64Array(signedSource);

const unsignedSource = new BigUint64Array(shared);
const unsignedToSigned = new BigInt64Array(unsignedSource);
const unsignedToUnsigned = new BigUint64Array(unsignedSource);

signedToSigned.buffer.constructor === ArrayBuffer && signedToSigned[0] === 7n &&
signedToUnsigned.buffer.constructor === ArrayBuffer && signedToUnsigned[0] === 7n &&
unsignedToSigned.buffer.constructor === ArrayBuffer && unsignedToSigned[0] === 7n &&
unsignedToUnsigned.buffer.constructor === ArrayBuffer && unsignedToUnsigned[0] === 7n;
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("BigInt TypedArray sources should copy into ArrayBuffer-backed views");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_numeric_typed_array_constructor_collects_generator_iterable() {
    let source = r#"
const numbers = new Uint8Array((function* () {
  yield 7;
  yield 42;
})());
[numbers.length, numbers.join(",")].join("|");
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("numeric TypedArray constructor should collect a generator iterable");
    assert!(
        outcome.note.contains("string(2|7,42)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_bigint_typed_array_constructor_collects_generator_iterable() {
    let source = r#"
const bigints = new BigInt64Array((function* () {
  yield 7n;
  yield 42n;
})());
[bigints.length, bigints.join(",")].join("|");
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("BigInt TypedArray constructor should collect a generator iterable");
    assert!(
        outcome.note.contains("string(2|7,42)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_numeric_typed_array_constructor_copies_array_argument() {
    let outcome = engine()
        .run_script(
            "const values = new Uint8Array([7, 42]); [values.length, values.join(',')].join('|');",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("numeric TypedArray constructor should copy an array argument");
    assert!(
        outcome.note.contains("string(2|7,42)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_bigint_typed_array_constructor_copies_array_argument() {
    let outcome = engine()
            .run_script(
                "const values = new BigInt64Array([7n, 42n]); [values.length, values.join(',')].join('|');",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("BigInt TypedArray constructor should copy an array argument");
    assert!(
        outcome.note.contains("string(2|7,42)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_bigint_typed_array_constructors_reduce_multi_limb_values_modulo_2_64() {
    let source = r#"
const positive = 18446744073709551618n;
const negative = -340282366920938463463374607431768211458n;
const signedBoundary = 9223372036854775810n;
const signed = new BigInt64Array([positive, negative, signedBoundary]);
const unsigned = new BigUint64Array([positive, negative, signedBoundary]);
[
  signed[0] === 2n,
  signed[1] === -2n,
  signed[2] === -9223372036854775806n
].join(",") + "|" + [
  unsigned[0] === 2n,
  unsigned[1] === 18446744073709551614n,
  unsigned[2] === 9223372036854775810n
].join(",");
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("BigInt TypedArray constructors should reduce values modulo 2^64");
    assert!(
        outcome
            .note
            .contains("string(true,true,true|true,true,true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_constructor_uses_modified_array_iterator_next() {
    let source = r#"
const iteratorPrototype = Object.getPrototypeOf([].values());
let remainingValues = [1, 2, 3, 4];
iteratorPrototype.next = function() {
  const done = remainingValues.length === 0;
  const value = remainingValues.pop();
  return { value, done };
};
const values = new Uint8Array([0]);
[values.length, values.join(",")].join("|");
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray constructor should use a modified Array iterator next method");
    assert!(
        outcome.note.contains("string(4|4,3,2,1)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_source_copy_uses_current_resizable_bounds() {
    let source = r#"
const buffer = new ArrayBuffer(4, { maxByteLength: 4 });
const fixed = new Uint8Array(buffer, 0, 4);
const tracking = new Uint8Array(buffer, 0);
tracking[0] = 1;
tracking[1] = 2;
tracking[2] = 3;
tracking[3] = 4;
const offsetCopy = new Uint8Array(new Uint8Array(buffer, 2, 2));

buffer.resize(2);
let fixedThrows = false;
try {
  new Uint8Array(fixed);
} catch (error) {
  fixedThrows = error instanceof TypeError;
}
const shortened = new Uint8Array(tracking);

buffer.resize(0);
const empty = new Uint8Array(tracking);

[
  offsetCopy.byteOffset,
  offsetCopy.join(","),
  fixedThrows,
  shortened.length,
  shortened.join(","),
  empty.length
].join("|");
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray source copies should use current resizable buffer bounds");
    assert!(
        outcome.note.contains("string(0|3,4|true|2|1,2|0)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_detaches_only_branded_array_buffers_with_private_state() {
    let source = r#"
const buffer = new ArrayBuffer(8);
const view = new DataView(buffer);
const typed = new Uint8Array(buffer);
buffer.$ArrayBufferDataPtr = 0;
buffer.$ArrayBufferByteLength = 99;
let privateBefore = "unset";
try {
  privateBefore = [buffer.byteLength, view.byteLength, typed.length].join(":");
} catch (error) {
  privateBefore = error.name;
}

let forgedTypeError = false;
try {
  __scriptwDetachArrayBuffer({ $ArrayBufferByteLength: 8 });
} catch (error) {
  forgedTypeError = error instanceof TypeError;
}
let sharedTypeError = false;
try {
  __scriptwDetachArrayBuffer(new SharedArrayBuffer(8));
} catch (error) {
  sharedTypeError = error instanceof TypeError;
}
let wrongKeyTypeError = false;
try {
  __scriptwDetachArrayBuffer(buffer, {});
} catch (error) {
  wrongKeyTypeError = error instanceof TypeError;
}

let firstDetach = true;
try { __scriptwDetachArrayBuffer(buffer); } catch (error) { firstDetach = error.name; }
let viewAccessorTypeError = false;
try { view.byteLength; } catch (error) { viewAccessorTypeError = error instanceof TypeError; }
let viewMethodTypeError = false;
try { view.getUint8(0); } catch (error) { viewMethodTypeError = error instanceof TypeError; }
let detachedState = "unset";
try {
  detachedState = [
    buffer.detached,
    buffer.byteLength,
    buffer.maxByteLength,
    buffer.resizable
  ].join(":");
} catch (error) { detachedState = error.name; }
let typedState = "unset";
try { typedState = [typed.length, typed[0] === undefined].join(":"); }
catch (error) { typedState = error.name; }
let typedToStringTypeError = false;
try { typed.toString(); } catch (error) { typedToStringTypeError = error instanceof TypeError; }
let typedToLocaleStringTypeError = false;
try { typed.toLocaleString(); }
catch (error) { typedToLocaleStringTypeError = error instanceof TypeError; }

buffer.$ArrayBufferDataPtr = 123;
buffer.$ArrayBufferByteLength = 123;
let repeatedDetach = true;
try { __scriptwDetachArrayBuffer(buffer); } catch (error) { repeatedDetach = error.name; }
let privateAfterRewrite = false;
try { privateAfterRewrite = buffer.detached && buffer.byteLength === 0 && typed.length === 0; }
catch (error) { privateAfterRewrite = error.name; }
const resizableBuffer = new ArrayBuffer(1, { maxByteLength: 2 });
__scriptwDetachArrayBuffer(resizableBuffer);
const detachedResizableState = [
  resizableBuffer.resizable,
  resizableBuffer.maxByteLength
].join(":");

[
  privateBefore,
  forgedTypeError,
  sharedTypeError,
  wrongKeyTypeError,
  firstDetach,
  detachedState,
  viewAccessorTypeError && viewMethodTypeError,
  typedState,
  typedToStringTypeError && typedToLocaleStringTypeError,
  repeatedDetach,
  privateAfterRewrite,
  detachedResizableState
].join("|");
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("ArrayBuffer detachment should use private state and realm-correct errors");
    assert!(
        outcome.note.contains(
            "string(8:8:8|true|true|true|true|true:0:0:false|true|0:true|true|true|true|true:0)"
        ),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_detaches_numeric_typed_array_owned_buffer() {
    let outcome = engine()
            .run_script(
                "const view = new Uint8Array(1); const buffer = view.buffer; __scriptwDetachArrayBuffer(buffer); buffer.detached && buffer.byteLength === 0 && view.length === 0;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("numeric TypedArray owned buffer should detach");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_detaches_bigint_typed_array_owned_buffer() {
    let outcome = engine()
            .run_script(
                "const view = new BigInt64Array(1); const buffer = view.buffer; __scriptwDetachArrayBuffer(buffer); buffer.detached && buffer.byteLength === 0 && view.length === 0;",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("BigInt TypedArray owned buffer should detach");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_growable_shared_array_buffer_grows_in_place_and_updates_live_views() {
    let source = r#"
const buffer = new SharedArrayBuffer(2, { maxByteLength: 8 });
const tracking = new Uint8Array(buffer);
const fixed = new Uint8Array(buffer, 0, 2);
const view = new DataView(buffer);
tracking[0] = 9;
tracking[1] = 7;

const result = buffer.grow(6);
const zeroInitialized = tracking[2] === 0 && tracking[3] === 0 &&
  tracking[4] === 0 && tracking[5] === 0;
tracking[5] = 11;
const sameSize = buffer.grow(6);

let shrinkRangeError = false;
try { buffer.grow(5); } catch (error) {
  shrinkRangeError = error instanceof RangeError;
}
let excessiveRangeError = false;
try { buffer.grow(9); } catch (error) {
  excessiveRangeError = error instanceof RangeError;
}
let argumentCoerced = false;
let fixedTypeError = false;
try {
  new SharedArrayBuffer(1).grow({
    valueOf() { argumentCoerced = true; return 2; }
  });
} catch (error) {
  fixedTypeError = error instanceof TypeError;
}
let inheritedArgumentCoerced = false;
let inheritedTypeError = false;
try {
  SharedArrayBuffer.prototype.grow.call(Object.create(buffer), {
    valueOf() { inheritedArgumentCoerced = true; return 7; }
  });
} catch (error) {
  inheritedTypeError = error instanceof TypeError;
}

const nested = new SharedArrayBuffer(0, { maxByteLength: 8 });
let nestedRangeError = false;
try {
  nested.grow({
    valueOf() { nested.grow(6); return 4; }
  });
} catch (error) {
  nestedRangeError = error instanceof RangeError;
}

const fractional = new SharedArrayBuffer(0, { maxByteLength: 4 });
const negativeFraction = fractional.grow(-0.5);
const positiveFraction = fractional.grow(1.9);

[
  result === undefined,
  buffer.byteLength,
  buffer.maxByteLength,
  buffer.growable,
  tracking.length,
  fixed.length,
  view.byteLength,
  tracking[0],
  tracking[1],
  zeroInitialized,
  tracking[5],
  fixed[0],
  sameSize === undefined,
  shrinkRangeError,
  excessiveRangeError,
  fixedTypeError,
  argumentCoerced,
  inheritedTypeError,
  inheritedArgumentCoerced,
  nestedRangeError,
  nested.byteLength,
  negativeFraction === undefined,
  positiveFraction === undefined,
  fractional.byteLength
].join("|");
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("growable SharedArrayBuffer should update live views without reallocating");
    assert!(
            outcome.note.contains(
                "string(true|6|8|true|6|2|6|9|7|true|11|9|true|true|true|true|false|true|false|true|6|true|true|1)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_shared_array_buffer_max_byte_length_uses_to_index() {
    let source = r#"
const nan = new SharedArrayBuffer(0, { maxByteLength: NaN });
const negativeFraction = new SharedArrayBuffer(0, { maxByteLength: -0.5 });
let coercions = 0;
const coerced = new SharedArrayBuffer(2, {
  maxByteLength: {
    valueOf() { coercions += 1; return 3.9; }
  }
});
let negativeRangeError = false;
try {
  new SharedArrayBuffer(0, { maxByteLength: -1 });
} catch (error) {
  negativeRangeError = error instanceof RangeError;
}
[
  nan.byteLength,
  nan.maxByteLength,
  nan.growable,
  negativeFraction.maxByteLength,
  coerced.byteLength,
  coerced.maxByteLength,
  coercions,
  negativeRangeError
].join("|");
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("SharedArrayBuffer maxByteLength should use ToIndex semantics");
    assert!(
        outcome.note.contains("string(0|0|true|0|2|3|1|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_shared_array_buffer_grow_throws_errors_from_its_function_realm() {
    let source = r#"
const other = __scriptwCreateRealm().global;
const grow = other.SharedArrayBuffer.prototype.grow;
const byteLength = Object.getOwnPropertyDescriptor(
  other.SharedArrayBuffer.prototype,
  "byteLength"
).get;
const fixed = new SharedArrayBuffer(1);
let typeRealm = "none";
try {
  grow.call(fixed, 2);
} catch (error) {
  typeRealm = [
    Object.getPrototypeOf(error) === other.TypeError.prototype,
    error instanceof other.TypeError,
    error instanceof TypeError
  ].join(":");
}

const growable = new other.SharedArrayBuffer(1, { maxByteLength: 2 });
let rangeRealm = "none";
try {
  growable.grow(0);
} catch (error) {
  rangeRealm = [
    Object.getPrototypeOf(error) === other.RangeError.prototype,
    error instanceof other.RangeError,
    error instanceof RangeError
  ].join(":");
}

let getterRealm = "none";
try {
  byteLength.call(new ArrayBuffer(1));
} catch (error) {
  getterRealm = [
    Object.getPrototypeOf(error) === other.TypeError.prototype,
    error instanceof other.TypeError,
    error instanceof TypeError
  ].join(":");
}
typeRealm + "|" + rangeRealm + "|" + getterRealm;
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("SharedArrayBuffer grow errors should use the method's defining realm");
    assert!(
        outcome
            .note
            .contains("string(true:true:false|true:true:false|true:true:false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_supports_array_write_and_read() {
    let outcome = engine()
        .run_script(
            "let a = [1]; a[2] = 4; a[2];",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should run array write and read");
    assert!(outcome.note.contains("number(4"));
}

#[test]
fn wasm_backend_supports_high_sparse_array_indexes() {
    let outcome = engine()
            .run_script(
                "let a = [1]; a[50000] = 2; let b = a.map(function(v) { return v + 1; }); b.length + \"|\" + b[0] + \"|\" + b[50000];",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("wasm backend should preserve high sparse array indexes");
    assert!(
        outcome.note.contains("string(50001|2|3)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_supports_array_function_calls_and_global_default_this() {
    let array_outcome = engine()
        .run_script(
            "function inc(x) { return x + 1; } let a = [inc]; a[0](2);",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should run array function call");
    assert!(array_outcome.note.contains("number(3"));

    let bare_this_outcome = engine()
        .run_script(
            "function check() { return this; } check();",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("wasm backend should default bare-call this to global object");
    assert!(bare_this_outcome
        .note
        .contains("wasm-aot completion: object"));
}

#[test]
fn wasm_backend_typed_array_iterators_keep_state_in_private_slots() {
    // The forged slots are written under the exact names `hidden` probes.
    // Spelling them `iterator.$ArrayIterator.array = []` instead reads the
    // absent `$ArrayIterator` property first, so PutValue's
    // ToObject(undefined) throws a TypeError before any slot is forged
    // (13.15.2 step 1.d via 6.2.5.6 PutValue step 3.a).
    let source = "const iterator = new Uint8Array([7, 8]).values(); const slotNames = ['$ArrayIterator.array', '$ArrayIterator.index', '$ArrayIterator.done', '$ArrayIterator.kind']; const hidden = slotNames.every(name => !Object.hasOwn(iterator, name)); iterator['$ArrayIterator.array'] = []; iterator['$ArrayIterator.index'] = 100; iterator['$ArrayIterator.done'] = true; iterator['$ArrayIterator.kind'] = 0; const first = iterator.next(); const second = iterator.next(); const done = iterator.next(); const forgedTarget = { length: 1, 0: 42, $TypedArrayViewedArrayBuffer: new ArrayBuffer(1), $TypedArrayByteOffset: 0, $TypedArrayByteLength: 0, $TypedArrayBytesPerElement: 1, $TypedArrayLengthTracking: false }; const genericValue = Array.prototype.values.call(forgedTarget).next().value; hidden + '|' + first.value + ':' + first.done + '|' + second.value + ':' + second.done + '|' + (done.value === undefined) + ':' + done.done + '|' + genericValue;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray iterators should keep state in private slots");
    assert!(
        outcome
            .note
            .contains("string(true|7:false|8:false|true:true|42)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_iterators_follow_resizable_buffer_bounds() {
    let source = "const trackingBuffer = new ArrayBuffer(4, { maxByteLength: 8 }); const tracking = new Uint8Array(trackingBuffer); tracking[0] = 1; tracking[1] = 2; tracking[2] = 3; tracking[3] = 4; const values = tracking.values(); const first = values.next().value; trackingBuffer.resize(6); tracking[4] = 5; tracking[5] = 6; const grown = [values.next().value, values.next().value, values.next().value, values.next().value, values.next().value].join(','); const grownDone = values.next().done; const fixedBuffer = new ArrayBuffer(4, { maxByteLength: 8 }); const fixed = new Uint8Array(fixedBuffer, 0, 4); const entries = fixed.entries(); entries.next(); fixedBuffer.resize(2); let fixedError = 'missing'; try { entries.next(); } catch (error) { fixedError = error.name; } const shrinkingBuffer = new ArrayBuffer(4, { maxByteLength: 8 }); const shrinking = new Uint8Array(shrinkingBuffer); const keys = shrinking.keys(); const key = keys.next().value; shrinkingBuffer.resize(1); const shrinkDone = keys.next().done; first + '|' + grown + ':' + grownDone + '|' + fixedError + '|' + key + ':' + shrinkDone;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray iterators should follow resizable buffer bounds");
    assert!(
        outcome
            .note
            .contains("string(1|2,3,4,5,6:true|TypeError|0:true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_iterators_remain_exhausted_after_resize() {
    let source = "const trackingBuffer = new ArrayBuffer(0, { maxByteLength: 4 }); const tracking = new Uint8Array(trackingBuffer); const trackingIterator = tracking.values(); const firstDone = trackingIterator.next().done; trackingBuffer.resize(1); tracking[0] = 9; const grown = trackingIterator.next(); const fixedBuffer = new ArrayBuffer(1, { maxByteLength: 2 }); const fixed = new Uint8Array(fixedBuffer, 0, 1); const fixedIterator = fixed.values(); fixedIterator.next(); const fixedDone = fixedIterator.next().done; fixedBuffer.resize(0); const shrunk = fixedIterator.next(); firstDone + '|' + (grown.value === undefined) + ':' + grown.done + '|' + fixedDone + '|' + (shrunk.value === undefined) + ':' + shrunk.done;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("exhausted TypedArray iterators should ignore later resizes");
    assert!(
        outcome
            .note
            .contains("string(true|true:true|true|true:true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_push_respects_non_writable_array_length() {
    let outcome = engine()
        .run_script(
            r#"
var zero = [];
Object.defineProperty(zero, "length", { writable: false });
var zeroThrow = "no";
try {
  zero.push();
} catch (e) {
  zeroThrow = e.name;
}

var one = [];
Object.defineProperty(one, "length", { writable: false });
var oneThrow = "no";
try {
  one.push(1);
} catch (e) {
  oneThrow = e.name;
}

zeroThrow + ":" + zero.length + "|" + oneThrow + ":" + one.length + ":" + (one[0] === undefined);
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.push should catch non-writable length TypeErrors");
    assert!(
        outcome
            .note
            .contains("string(TypeError:0|TypeError:0:true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_push_calls_inherited_index_setter_before_length_write() {
    let outcome = engine()
        .run_script(
            r#"
var array = [];
var callCount = 0;
var receiverMatches = false;
var receivedValue = 0;
Object.defineProperty(Array.prototype, "0", {
  set: function(value) {
    callCount += 1;
    receiverMatches = this === array;
    receivedValue = value;
    Object.defineProperty(array, "length", { writable: false });
  },
  configurable: true
});

var caught = "no";
try {
  array.push(41);
} catch (e) {
  caught = e.name;
}
delete Array.prototype[0];
caught + ":" + array.length + ":" + callCount + ":" + receiverMatches + ":" +
  receivedValue + ":" + (array[0] === undefined);
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.push should honor inherited index setters");
    assert!(
        outcome.note.contains("string(TypeError:0:1:true:41:true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_assignment_calls_inherited_numeric_setter() {
    let outcome = engine()
        .run_script(
            r#"
var array = [];
var callCount = 0;
var receiverMatches = false;
var receivedValue = 0;
Object.defineProperty(Array.prototype, "0", {
  set: function(value) {
    callCount += 1;
    receiverMatches = this === array;
    receivedValue = value;
  },
  configurable: true
});

var assignedValue = array[0] = 23;
delete Array.prototype[0];
assignedValue + ":" + callCount + ":" + receiverMatches + ":" + receivedValue + ":" +
  array.length + ":" + (array[0] === undefined);
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("array assignment should honor inherited numeric setters");
    assert!(
        outcome.note.contains("string(23:1:true:23:0:true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_push_calls_inherited_setter_with_non_writable_length() {
    let outcome = engine()
        .run_script(
            r#"
var array = [];
Object.defineProperty(array, "length", { writable: false });
var callCount = 0;
Object.defineProperty(Array.prototype, "0", {
  set: function(value) {
    callCount += value;
  },
  configurable: true
});

var caught = "no";
try {
  array.push(2);
} catch (e) {
  caught = e.name;
}
delete Array.prototype[0];
caught + ":" + callCount + ":" + array.length + ":" + (array[0] === undefined);
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("push should run an inherited setter before its final length write");
    assert!(
        outcome.note.contains("string(TypeError:2:0:true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_push_respects_frozen_array_length() {
    let outcome = engine()
        .run_script(
            r#"
var empty = [];
Object.freeze(empty);
var emptyThrow = "no";
try {
  empty.push();
} catch (e) {
  emptyThrow = e.name;
}

var array = [];
var callCount = 0;
Object.defineProperty(Array.prototype, "0", {
  set: function(value) {
    Object.freeze(array);
    callCount += 1;
  },
  configurable: true
});

var inheritedThrow = "no";
try {
  array.push(1);
} catch (e) {
  inheritedThrow = e.name;
}
delete Array.prototype[0];
emptyThrow + ":" + empty.length + ":" + (empty[0] === undefined) + "|" +
  inheritedThrow + ":" + array.length + ":" + callCount + ":" + (array[0] === undefined);
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.push should respect Object.freeze length integrity");
    assert!(
        outcome
            .note
            .contains("string(TypeError:0:true|TypeError:0:1:true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_shift_moves_generic_array_like_properties_and_holes() {
    let outcome = engine()
        .run_script(
            r#"
var sparse = ["a", , "c"];
var sparseFirst = sparse.shift();
var sparseResult = sparseFirst + ":" + sparse.length + ":" + (0 in sparse) + ":" + sparse[1];

var calls = [];
var object;
var prototype = {};
Object.defineProperty(prototype, "0", {
  get: function() {
    calls.push("get 0");
    return "first";
  },
  set: function(value) {
    calls.push("set 0 " + value + " " + (this === object));
  },
  configurable: true
});
Object.defineProperty(prototype, "1", {
  get: function() {
    calls.push("get 1");
    return "second";
  },
  configurable: true
});
object = Object.create(prototype);
object.length = 2;
object.shift = Array.prototype.shift;
var objectFirst = object.shift();
var objectResult = objectFirst + ":" + object.length + ":" + calls.join(",");

var savedShift = Array.prototype.shift;
Array.prototype.shift = function() { return "custom " + this.length; };
var mutated = [9];
var mutatedResult = mutated.shift() + ":" + mutated.length;
Array.prototype.shift = savedShift;
var extracted = [8];
var extractedResult = savedShift.call(extracted) + ":" + extracted.length;

sparseResult + "|" + objectResult + "|" + mutatedResult + "|" + extractedResult;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.shift generic array-like cases should run");
    assert!(
        outcome
            .note
            .contains("string(a:2:false:c|first:1:get 0,get 1,set 0 second true|custom 1:1|8:0)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_shift_enforces_strict_writes_and_builtin_realm_errors() {
    let outcome = engine()
        .run_script(
            r#"
var frozen = [];
Object.freeze(frozen);
var frozenResult = "missing";
try {
  frozen.shift();
} catch (error) {
  frozenResult = error.name + ":" + frozen.length;
}

var stringResults = [];
for (var value of ["", "abc"]) {
  try {
    Array.prototype.shift.call(value);
    stringResults.push("missing");
  } catch (error) {
    stringResults.push(error.name);
  }
}

var other = __scriptwCreateRealm();
var otherShift = other.global.Array.prototype.shift;
var realmResult = "missing";
try {
  otherShift.call(null);
} catch (error) {
  realmResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

frozenResult + "|" + stringResults.join(",") + "|" + realmResult;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.shift should enforce strict writes in its defining realm");
    assert!(
        outcome
            .note
            .contains("string(TypeError:0|TypeError,TypeError|true:true:false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_unshift_moves_generic_array_like_properties_and_holes() {
    let outcome = engine()
        .run_script(
            r#"
var object = { 0: "a", 2: "c", length: 3 };
var objectLength = Array.prototype.unshift.call(object, "x", "y");
var objectResult = objectLength + ":" + object.length + ":" + object[0] + ":" +
  object[1] + ":" + object[2] + ":" + (3 in object) + ":" + object[4];

var array = [];
array[1] = "b";
array.length = 3;
var arrayLength = array.unshift("x", "y");
var arrayResult = arrayLength + ":" + array.length + ":" + array[0] + ":" +
  array[1] + ":" + (2 in array) + ":" + array[3] + ":" + (4 in array);

var many = ["tail"];
var manyLength = many.unshift(0, 1, 2, 3, 4, 5, 6, 7, 8, 9);
var manyResult = manyLength + ":" + many.length + ":" + many[0] + ":" +
  many[8] + ":" + many[9] + ":" + many[10];

objectResult + "|" + arrayResult + "|" + manyResult;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.unshift generic array-like cases should run");
    assert!(
        outcome
            .note
            .contains("string(5:5:x:y:a:false:c|5:5:x:y:false:b:false|11:11:0:8:9:tail)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_unshift_propagates_an_abrupt_source_getter() {
    let error = engine()
        .run_script(
            r#"
var object = { length: 2 };
Object.defineProperty(object, "1", {
  get: function() {
    return "moved";
  },
  configurable: true
});
Object.defineProperty(object, "0", {
  get: function() {
    throw "stop";
  },
  configurable: true
});
Array.prototype.unshift.call(object, "new");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect_err("Array.prototype.unshift should propagate abrupt source getters");
    assert!(
        error.message().contains("uncaught throw") && error.message().contains("string(stop)"),
        "error: {error:?}"
    );
}

#[test]
fn wasm_backend_array_unshift_calls_inherited_setter_before_length_write() {
    let outcome = engine()
        .run_script(
            r#"
var array = [];
var callCount = 0;
var receiverMatches = false;
var receivedValue = 0;
Object.defineProperty(Array.prototype, "0", {
  set: function(value) {
    callCount += 1;
    receiverMatches = this === array;
    receivedValue = value;
  },
  configurable: true
});
Object.defineProperty(array, "length", { writable: false });

var nonzeroThrow = "no";
try {
  array.unshift(41);
} catch (error) {
  nonzeroThrow = error.name;
}

var frozen = [];
Object.freeze(frozen);
var zeroThrow = "no";
try {
  frozen.unshift();
} catch (error) {
  zeroThrow = error.name;
}
delete Array.prototype[0];
nonzeroThrow + ":" + array.length + ":" + callCount + ":" + receiverMatches + ":" +
  receivedValue + ":" + (array[0] === undefined) + "|" + zeroThrow + ":" + frozen.length;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.unshift should honor setters and length integrity");
    assert!(
        outcome
            .note
            .contains("string(TypeError:0:1:true:41:true|TypeError:0)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_cross_realm_array_species_create_uses_receiver_realm_constructor() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let array = new other.Array(2);
array[0] = 41;
array[1] = 7;
let mapped = other.Array.prototype.map.call(array, function(value) { return value + 1; });
let filtered = other.Array.prototype.filter.call(array, function(value) { return value > 10; });
[
  array.constructor === other.Array,
  other.Array.prototype.constructor === other.Array,
  other.Array[Symbol.species] === other.Array,
  Object.getPrototypeOf(mapped) === other.Array.prototype,
  Object.getPrototypeOf(mapped) === Array.prototype,
  mapped.constructor === other.Array,
  mapped.length,
  mapped[0],
  mapped[1],
  Object.getPrototypeOf(filtered) === other.Array.prototype,
  Object.getPrototypeOf(filtered) === Array.prototype,
  filtered.constructor === other.Array,
  filtered.length,
  filtered[0]
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("ArraySpeciesCreate should allocate with the receiver realm constructor");
    assert!(
        outcome
            .note
            .contains("string(true|true|true|true|false|true|2|42|8|true|false|true|1|41)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_create_realm_uses_realm_local_typed_array_constructor_family() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let names = [
  "Int8Array", "Uint8Array", "Uint8ClampedArray",
  "Int16Array", "Uint16Array", "Int32Array", "Uint32Array",
  "Float32Array", "Float64Array", "BigInt64Array", "BigUint64Array"
];
let otherTypedArrayPrototype = Object.getPrototypeOf(other.Uint8Array.prototype);
let ok = true;

for (let i = 0; i < names.length; i++) {
  let name = names[i];
  let C = other[name];
  let ThisC = globalThis[name];
  let isBigInt = name === "BigInt64Array" || name === "BigUint64Array";
  let view = new C(isBigInt ? [0n] : [0]);
  ok =
    ok &&
    C !== ThisC &&
    C.prototype !== ThisC.prototype &&
    Object.getPrototypeOf(C.prototype) === otherTypedArrayPrototype &&
    Object.getPrototypeOf(view) === C.prototype &&
    typeof view[0] === (isBigInt ? "bigint" : "number") &&
    view instanceof C &&
    !(view instanceof ThisC);
}

ok;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm should use realm-local typed array constructor family");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_cross_realm_array_flat_methods_use_receiver_realm_species() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let array = new other.Array(1);
let nested = new other.Array(1);
nested[0] = 5;
array[0] = nested;
let flat = other.Array.prototype.flat.call(array);
let flatMapped = other.Array.prototype.flatMap.call(array, function(value) { return value; });
[
  typeof other.Array.prototype.flat,
  Object.getPrototypeOf(flat) === other.Array.prototype,
  Object.getPrototypeOf(flat) === Array.prototype,
  flat.constructor === other.Array,
  flat.length,
  flat[0],
  typeof other.Array.prototype.flatMap,
  Object.getPrototypeOf(flatMapped) === other.Array.prototype,
  Object.getPrototypeOf(flatMapped) === Array.prototype,
  flatMapped.constructor === other.Array,
  flatMapped.length,
  flatMapped[0]
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("cross-realm flat/flatMap should use receiver realm species constructors");
    assert!(
        outcome
            .note
            .contains("string(function|true|false|true|1|5|function|true|false|true|1|5)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_cross_realm_array_flat_methods_throw_other_realm_type_error() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm();
let results = [];
let flat = other.global.Array.prototype.flat;
let flatMap = other.global.Array.prototype.flatMap;
let boundFlat = flat.bind(null);
let boundFlatMapNullish = flatMap.bind(null, function(value) { return value; });
let boundFlatMapBadMapper = flatMap.bind([], null);
let calls = [
  function() { return flat.call(null); },
  function() { return boundFlat(); },
  function() { return flatMap.call(null, function(value) { return value; }); },
  function() { return boundFlatMapNullish(); },
  function() { return flatMap.call([], null); },
  function() { return boundFlatMapBadMapper(); }
];
for (let i = 0; i < calls.length; i++) {
  try {
    calls[i]();
    results.push("missing");
  } catch (error) {
    results.push(
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError)
    );
  }
}
results.join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("cross-realm flat/flatMap throws should use the defining realm TypeError");
    assert!(
            outcome.note.contains(
                "string(true:true:false|true:true:false|true:true:false|true:true:false|true:true:false|true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_cross_realm_array_concat_uses_receiver_realm_species() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let array = new other.Array(1);
array[0] = 3;
let result = other.Array.prototype.concat.call(array, 4);
[
  typeof other.Array.prototype.concat,
  Object.getPrototypeOf(result) === other.Array.prototype,
  Object.getPrototypeOf(result) === Array.prototype,
  result.constructor === other.Array,
  result.length,
  result[0],
  result[1]
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("cross-realm concat should use receiver realm species constructors");
    assert!(
        outcome
            .note
            .contains("string(function|true|false|true|2|3|4)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_cross_realm_array_concat_throws_other_realm_type_error() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm();
let concat = other.global.Array.prototype.concat;
let boundNullish = concat.bind(null);
let calls = [
  function() { return concat.call(null); },
  function() { return boundNullish(); }
];
let results = [];
for (let i = 0; i < calls.length; i++) {
  try {
    calls[i]();
    results.push("missing");
  } catch (error) {
    results.push(
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError)
    );
  }
}
results.join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("cross-realm concat throws should use the defining realm TypeError");
    assert!(
        outcome
            .note
            .contains("string(true:true:false|true:true:false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_create_realm_exposes_array_static_methods_with_receiver_realm_results() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm().global;
let otherArray = new other.Array(1);
otherArray[0] = 9;
let fromResult = other.Array.from([1, 2]);
let ofResult = other.Array.of(3, 4);
[
  typeof other.Array.from,
  typeof other.Array.of,
  typeof other.Array.isArray,
  other.Array.isArray(otherArray),
  Array.isArray(otherArray),
  Object.getPrototypeOf(fromResult) === other.Array.prototype,
  Object.getPrototypeOf(fromResult) === Array.prototype,
  fromResult.constructor === other.Array,
  fromResult.length,
  fromResult[0],
  fromResult[1],
  Object.getPrototypeOf(ofResult) === other.Array.prototype,
  Object.getPrototypeOf(ofResult) === Array.prototype,
  ofResult.constructor === other.Array,
  ofResult.length,
  ofResult[0],
  ofResult[1]
].join("|");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("synthetic realm Array statics should use the receiver realm constructor");
    assert!(
            outcome.note.contains(
                "string(function|function|function|true|true|true|false|true|2|1|2|true|false|true|2|3|4)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_cross_realm_array_at_throws_other_realm_type_error() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm();
let method = other.global.Array.prototype.at;
let bound = method.bind(null, 0);
let direct = "missing";
let boundResult = "missing";

try {
  method.call(null, 0);
} catch (error) {
  direct =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

try {
  bound();
} catch (error) {
  boundResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

direct + "|" + boundResult;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("borrowed and bound cross-realm Array.prototype.at should throw in defining realm");
    assert!(
        outcome
            .note
            .contains("string(true:true:false|true:true:false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_cross_realm_array_includes_throws_other_realm_type_error() {
    let outcome = engine()
            .run_script(
                r#"
let other = __scriptwCreateRealm();
let method = other.global.Array.prototype.includes;
let bound = method.bind(null, 1);
let direct = "missing";
let boundResult = "missing";

try {
  method.call(null, 1);
} catch (error) {
  direct =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

try {
  bound();
} catch (error) {
  boundResult =
    (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
    (error instanceof other.global.TypeError) + ":" +
    (error instanceof TypeError);
}

direct + "|" + boundResult;
"#,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect(
                "borrowed and bound cross-realm Array.prototype.includes should throw in defining realm",
            );
    assert!(
        outcome
            .note
            .contains("string(true:true:false|true:true:false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_cross_realm_array_helper_methods_throw_other_realm_type_error() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm();
let names = ["indexOf", "lastIndexOf", "forEach"];
let results = [];

for (let i = 0; i < names.length; i++) {
  let method = other.global.Array.prototype[names[i]];
  let bound = method.bind(null, 1);
  let direct = "missing";
  let boundResult = "missing";

  try {
    method.call(null, 1);
  } catch (error) {
    direct =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    bound();
  } catch (error) {
    boundResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  results.push(names[i] + "=" + direct + "|" + boundResult);
}

results.join(",");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect(
            "borrowed and bound cross-realm Array helper methods should throw in defining realm",
        );
    assert!(
            outcome.note.contains(
                "string(indexOf=true:true:false|true:true:false,lastIndexOf=true:true:false|true:true:false,forEach=true:true:false|true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_cross_realm_array_find_like_methods_throw_other_realm_type_error() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm();
let names = ["find", "findIndex", "findLast", "findLastIndex"];
let results = [];

for (let i = 0; i < names.length; i++) {
  let method = other.global.Array.prototype[names[i]];
  let boundNullish = method.bind(null, function() { return true; });
  let boundBadPredicate = method.bind([], null);
  let directNullish = "missing";
  let boundNullishResult = "missing";
  let directBadPredicate = "missing";
  let boundBadPredicateResult = "missing";

  try {
    method.call(null, function() { return true; });
  } catch (error) {
    directNullish =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    boundNullish();
  } catch (error) {
    boundNullishResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    method.call([], null);
  } catch (error) {
    directBadPredicate =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    boundBadPredicate();
  } catch (error) {
    boundBadPredicateResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  results.push(
    names[i] + "=" +
    directNullish + "|" +
    boundNullishResult + "|" +
    directBadPredicate + "|" +
    boundBadPredicateResult
  );
}

results.join(",");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect(
            "borrowed and bound cross-realm Array find-like methods should throw in defining realm",
        );
    assert!(
            outcome.note.contains(
                "string(find=true:true:false|true:true:false|true:true:false|true:true:false,findIndex=true:true:false|true:true:false|true:true:false|true:true:false,findLast=true:true:false|true:true:false|true:true:false|true:true:false,findLastIndex=true:true:false|true:true:false|true:true:false|true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_cross_realm_array_callback_methods_throw_other_realm_type_error() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm();
let names = ["map", "filter", "every", "some"];
let results = [];

for (let i = 0; i < names.length; i++) {
  let method = other.global.Array.prototype[names[i]];
  let boundNullish = method.bind(null, function(value) { return value; });
  let boundBadCallback = method.bind([], null);
  let directNullish = "missing";
  let boundNullishResult = "missing";
  let directBadCallback = "missing";
  let boundBadCallbackResult = "missing";

  try {
    method.call(null, function(value) { return value; });
  } catch (error) {
    directNullish =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    boundNullish();
  } catch (error) {
    boundNullishResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    method.call([], null);
  } catch (error) {
    directBadCallback =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    boundBadCallback();
  } catch (error) {
    boundBadCallbackResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  results.push(
    names[i] + "=" +
    directNullish + "|" +
    boundNullishResult + "|" +
    directBadCallback + "|" +
    boundBadCallbackResult
  );
}

results.join(",");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect(
            "borrowed and bound cross-realm Array callback methods should throw in defining realm",
        );
    assert!(
            outcome.note.contains(
                "string(map=true:true:false|true:true:false|true:true:false|true:true:false,filter=true:true:false|true:true:false|true:true:false|true:true:false,every=true:true:false|true:true:false|true:true:false|true:true:false,some=true:true:false|true:true:false|true:true:false|true:true:false)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_cross_realm_array_iterator_methods_throw_other_realm_type_error() {
    let outcome = engine()
        .run_script(
            r#"
let other = __scriptwCreateRealm();
let names = ["keys", "entries", "values"];
let results = [];

for (let i = 0; i < names.length; i++) {
  let method = other.global.Array.prototype[names[i]];
  let bound = method.bind(null);
  let direct = "missing";
  let boundResult = "missing";

  try {
    method.call(null);
  } catch (error) {
    direct =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    bound();
  } catch (error) {
    boundResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  results.push(names[i] + "=" + direct + "|" + boundResult);
}

{
  let iterator = other.global.Array.prototype.values.call(new other.global.Array(1));
  let method = Object.getPrototypeOf(iterator).next;
  let bound = method.bind({});
  let direct = "missing";
  let boundResult = "missing";
  let nextResult = "missing";

  try {
    method.call({});
  } catch (error) {
    direct =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  try {
    bound();
  } catch (error) {
    boundResult =
      (Object.getPrototypeOf(error) === other.global.TypeError.prototype) + ":" +
      (error instanceof other.global.TypeError) + ":" +
      (error instanceof TypeError);
  }

  nextResult = method.call(iterator);
  results.push(
    "next=" + direct + "|" + boundResult + "|" +
    (Object.getPrototypeOf(nextResult) === other.global.Object.prototype) + ":" +
    (Object.getPrototypeOf(nextResult) === Object.prototype)
  );
}

results.join(",");
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect(
            "borrowed and bound cross-realm Array iterator methods should throw in defining realm",
        );
    assert!(
            outcome.note.contains(
                "string(keys=true:true:false|true:true:false,entries=true:true:false|true:true:false,values=true:true:false|true:true:false,next=true:true:false|true:true:false|true:false)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_runtime_throws_for_non_callable_method_and_reads_array_length_brackets() {
    let method_err = engine()
        .run_script(
            "let obj = { f: 1 }; obj.f();",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect_err("non-callable method should throw");
    assert!(method_err.message().contains("uncaught throw: TypeError"));

    let length_outcome = engine()
        .run_script(
            "let a = [1]; a[\"length\"];",
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("array length bracket should run");
    assert!(length_outcome.note.contains("number(1)"));
}

#[test]
fn wasm_backend_captured_var_observes_typed_array_assigned_after_closure_creation() {
    let outcome = engine()
        .run_script(
            r#"
function copyWithinAfterDetach(TA) {
  var view;
  function detachAndReturnEnd() {
    __scriptwDetachArrayBuffer(view.buffer);
    return 4;
  }
  view = new TA(4);
  try {
    view.copyWithin(0, 1, { valueOf: detachAndReturnEnd });
  } catch (error) {
    return error instanceof TypeError;
  }
  return false;
}
copyWithinAfterDetach(Uint8Array);
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("captured var should retain the runtime kind of its later typed array value");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_emits_arraybuffer_resize_for_closure_method_call() {
    let source = "const rab = new ArrayBuffer(64, { maxByteLength: 1024 }); let called = false; let threw = false; try { (() => rab.resize({ valueOf() { called = true; throw new TypeError(); }}))(); } catch (e) { threw = e instanceof TypeError; } called + '|' + threw;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| {
            panic!("ArrayBuffer resize method call in closure should run: {err:?}")
        });
    assert!(
        outcome.note.contains("string(true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_writes_captured_typed_array_indexes() {
    let source = "const view = new Uint8Array(new ArrayBuffer(4)); function write() { view[1] = 9; } write(); view[1];";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("captured typed array write should run: {err:?}"));
    assert!(outcome.note.contains("number(9)"), "note: {}", outcome.note);
}

#[test]
fn wasm_backend_array_map_result_indexes_are_ordinary_data_properties() {
    let source = "var r = [1, 2, 3].map(function(v, i) { return i + 10; }); var before = Object.keys(r).join(','); var d = Object.getOwnPropertyDescriptor(r, 1); r[1] = 42; delete r[2]; before + '|' + r[1] + '|' + r[2] + '|' + d.writable + '|' + d.enumerable + '|' + d.configurable + '|' + Object.keys(r).join(',');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("Array.prototype.map result descriptors should run: {err:?}"));
    assert!(
        outcome
            .note
            .contains("string(0,1,2|42|undefined|true|true|true|0,1)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_map_remains_generic_for_inherited_array_methods() {
    let source = "function Receiver() {} Receiver.prototype = new Array(1, 2, 3); var receiver = new Receiver(); receiver.length = 1; var calls = 0; var result = receiver.map(function(value, index, array) { calls += 1; return array === receiver && index === 0 && value === 1; }); Array.isArray(result) + '|' + result.length + '|' + result[0] + '|' + calls;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.map should remain generic on non-array receivers");
    assert!(
        outcome.note.contains("string(true|1|true|1)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_callback_methods_remain_generic_when_inherited() {
    let source = "function Receiver() {} Receiver.prototype = new Array(1, 2, 3); var receiver = new Receiver(); receiver.length = 1; receiver.every(function(value) { return value === 1; }) + '|' + receiver.some(function(value) { return value === 1; }) + '|' + receiver.filter(function(value) { return value === 1; }).join(',');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("inherited Array callback methods should remain generic");
    assert!(
        outcome.note.contains("string(true|true|1)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_methods_use_observable_arguments_length() {
    let source = "function check(a, b) { arguments[2] = 9; var flat = Array.prototype.flat.call(arguments); var flatMapped = Array.prototype.flatMap.call(arguments, function(value) { return [value]; }); var mapped = Array.prototype.map.call(arguments, function(value) { return value; }); var filtered = Array.prototype.filter.call(arguments, function() { return true; }); var every = Array.prototype.every.call(arguments, function(value) { return value > 10; }); var some = Array.prototype.some.call(arguments, function(value) { return value === 9; }); var index = Array.prototype.indexOf.call(arguments, 9); var lastIndex = Array.prototype.lastIndexOf.call(arguments, 9); return flat.length + '|' + flatMapped.length + '|' + mapped.length + '|' + filtered.length + '|' + every + '|' + some + '|' + index + '|' + lastIndex; } check(12, 11);";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array methods should use the observable arguments length");
    assert!(
        outcome.note.contains("string(2|2|2|2|true|false|-1|-1)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_search_methods_stop_after_index_getter_throws() {
    let source = "var forwardAccessed = false; var forward = []; Object.defineProperty(forward, '0', { get: function() { throw new TypeError(); } }); Object.defineProperty(forward, '1', { get: function() { forwardAccessed = true; return true; } }); var forwardCaught = false; try { forward.indexOf(true); } catch (error) { forwardCaught = error instanceof TypeError; } var reverseAccessed = false; var reverse = []; Object.defineProperty(reverse, '0', { get: function() { reverseAccessed = true; return true; } }); Object.defineProperty(reverse, '1', { get: function() { throw new TypeError(); } }); var reverseCaught = false; try { reverse.lastIndexOf(true); } catch (error) { reverseCaught = error instanceof TypeError; } forwardCaught + '|' + forwardAccessed + '|' + reverseCaught + '|' + reverseAccessed;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array search methods should propagate indexed getter exceptions");
    assert!(
        outcome.note.contains("string(true|false|true|false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_concat_stops_after_index_getter_throws() {
    let source = "var token = {}; var accessed = false; var values = []; Object.defineProperty(values, '0', { get: function() { throw token; } }); Object.defineProperty(values, '1', { get: function() { accessed = true; return true; } }); var caught = false; try { values.concat([]); } catch (error) { caught = error === token; } caught + '|' + accessed;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.concat should propagate indexed getter exceptions");
    assert!(
        outcome.note.contains("string(true|false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_to_string_observes_overridden_join_getters() {
    let source = "const method = Uint8Array.prototype.toString; const values = new Uint8Array([1, 2]); const throwing = new Uint8Array(); const token = {}; const events = []; Object.defineProperty(values, 'join', { get() { events.push('get'); return function() { events.push(this === values ? 'call' : 'wrong receiver'); return 'custom'; }; } }); Object.defineProperty(throwing, 'join', { get() { events.push('throw'); throw token; } }); const result = method.call(values); let caught = false; try { method.call(throwing); } catch (error) { caught = error === token; } result + '|' + events.join(',') + '|' + caught;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.toString should observe overridden join getters");
    assert!(
        outcome.note.contains("string(custom|get,call,throw|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_to_string_is_generic_for_non_typed_array_receivers() {
    let source = "const method = Uint8Array.prototype.toString; const receiver = { marker: 'generic', join() { return this.marker; } }; const fallback = { join: 0, [Symbol.toStringTag]: 'Fallback' }; method.call(receiver) + '|' + method.call(fallback) + '|' + method.call([1, 2]) + '|' + method.call('text');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.toString should accept generic receivers");
    assert!(
        outcome
            .note
            .contains("string(generic|[object Fallback]|1,2|[object String])"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_from_and_of_use_generic_constructor_semantics() {
    let source = "const events = []; const source = { get [Symbol.iterator]() { events.push('iterator'); return null; }, get length() { events.push('length'); return 2.9; }, get 0() { events.push('get:0'); return 4; }, get 1() { events.push('get:1'); return 5; } }; const Constructor = new Proxy(function(length) { events.push('construct:' + length); return new Uint8Array(length); }, {}); const mapper = new Proxy(function(value, index) { events.push('map:' + index); return value + index; }, {}); const from = Uint8Array.from.call(Constructor, source, mapper); const of = Uint8Array.of.call(Constructor, 1, 258); let forged = 'missing'; try { Uint8Array.of.call(function() { return { $TypedArrayViewedArrayBuffer: {}, $TypedArrayByteOffset: 0, $TypedArrayByteLength: 2, $TypedArrayBytesPerElement: 1, $TypedArrayElementKind: 1 }; }, 1, 2); } catch (error) { forged = error.name; } from.join(',') + '|' + of.join(',') + '|' + forged + '|' + events.join(',');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray.from and TypedArray.of should use generic constructors");
    assert!(
            outcome.note.contains("string(4,6|1,2|TypeError|iterator,length,construct:2,get:0,map:0,get:1,map:1,construct:2)"),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_array_for_each_observes_inherited_indexes_after_deletion() {
    let source = "Array.prototype[4] = 9; var array = [1, 2, 3, 4, 5]; var values = []; array.forEach(function(value, index) { if (index === 0) delete array[4]; values.push(index + ':' + value); }); delete Array.prototype[4]; values.join(',');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.forEach should observe inherited indexes after deletion");
    assert!(
        outcome.note.contains("string(0:1,1:2,2:3,3:4,4:9)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_compound_assigns_constant_array_indexes() {
    let source = "var array = [10]; var result = (array[0] += 2); var strings = ['a']; strings[0] += 2; var bigints = [10n]; bigints[0] += 2n; result + '|' + array[0] + '|' + strings[0] + '|' + bigints[0];";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("compound assignment should support constant computed array indexes");
    assert!(
        outcome.note.contains("string(12|12|a2|12)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_to_reversed_snapshots_length_and_reads_in_descending_order() {
    let source = "var order = []; var array = [0, 1, 2, 3, 4]; Array.prototype[1] = 5; Object.defineProperty(array, '3', { get: function() { order.push(3); array.length = 1; return 3; } }); var result = array.toReversed(); delete Array.prototype[1]; result.join(',') + '|' + order.join(',');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.toReversed should use its length snapshot");
    assert!(
        outcome.note.contains("string(4,3,,5,0|3)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_with_replaces_without_reading_the_selected_index() {
    let source = "var reads = []; var source = { length: 3, get 0() { reads.push(0); return 1; }, get 1() { throw 'selected index was read'; }, get 2() { reads.push(2); return 3; } }; var result = Array.prototype.with.call(source, -2, 9); result.join(',') + '|' + reads.join(',');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.with should replace the selected index without reading it");
    assert!(
        outcome.note.contains("string(1,9,3|0,2)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_to_spliced_skips_deleted_elements() {
    let source = "var reads = []; var source = { length: 4, get 0() { reads.push(0); return 1; }, get 1() { throw 'deleted element was read'; }, get 2() { throw 'deleted element was read'; }, get 3() { reads.push(3); return 4; } }; var result = Array.prototype.toSpliced.call(source, 1, 2, 8, 9); result.join(',') + '|' + reads.join(',');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.toSpliced should not read deleted elements");
    assert!(
        outcome.note.contains("string(1,8,9,4|0,3)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_to_sorted_reads_all_elements_before_comparing() {
    let source = "var reads = []; var comparedAfterReads = true; var source = { length: 3, get 0() { reads.push(0); return 3; }, get 1() { reads.push(1); return 1; }, get 2() { reads.push(2); return 2; } }; var result = Array.prototype.toSorted.call(source, function(a, b) { if (reads.length !== 3) comparedAfterReads = false; return a - b; }); result.join(',') + '|' + reads.join(',') + '|' + comparedAfterReads;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.toSorted should collect values before comparing");
    assert!(
        outcome.note.contains("string(1,2,3|0,1,2|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_reverse_handles_resizable_typed_array_views() {
    let source = "const buffer = new ArrayBuffer(4, { maxByteLength: 8 }); const fixed = new Uint8Array(buffer, 0, 4); const tracking = new Uint8Array(buffer); fixed[0] = 1; fixed[1] = 2; fixed[2] = 3; fixed[3] = 4; Array.prototype.reverse.call(fixed); const first = tracking.join(','); buffer.resize(3); tracking[0] = 5; tracking[1] = 6; tracking[2] = 7; Array.prototype.reverse.call(fixed); Array.prototype.reverse.call(tracking); first + '|' + tracking.join(',');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.reverse should handle resizable typed array views");
    assert!(
        outcome.note.contains("string(4,3,2,1|7,6,5)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_join_validates_receiver_and_uses_current_view() {
    let source = "const buffer = new ArrayBuffer(4, { maxByteLength: 8 }); const values = new Uint8Array(buffer); values[0] = 1; values[1] = 2; values[2] = 3; values[3] = 4; const before = values.join('-'); buffer.resize(2); const after = values.join(','); let lengthReads = 0; Object.defineProperty(values, 'length', { get() { lengthReads++; return 0; } }); const internalLength = values.join(':'); let borrowed = 'missing'; try { Uint8Array.prototype.join.call({ length: 1, 0: 1 }); } catch (error) { borrowed = error.name; } before + '|' + after + '|' + internalLength + '|' + lengthReads + '|' + borrowed + '|' + (values.join === Array.prototype.join);";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray.prototype.join should use TypedArray semantics");
    assert!(
        outcome
            .note
            .contains("string(1-2-3-4|1,2|1:2|0|TypeError|false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_join_validation_does_not_change_generic_array_join() {
    let source = "const buffer = new ArrayBuffer(4, { maxByteLength: 8 }); const fixed = new Uint8Array(buffer, 1, 2); buffer.resize(2); let typedResult = 'missing'; try { Uint8Array.prototype.join.call(fixed); } catch (error) { typedResult = error.name; } const genericResult = Array.prototype.join.call(fixed); let objectResult = 'missing'; try { Uint8Array.prototype.join.call({ length: 1, 0: 1 }); } catch (error) { objectResult = error.name; } const genericObjectResult = Array.prototype.join.call({ length: 1, 0: 1 }); typedResult + '|' + genericResult + '|' + objectResult + '|' + genericObjectResult;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray join validation should not change generic Array join");
    assert!(
        outcome.note.contains("string(TypeError||TypeError|1)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_subarray_uses_species_and_preserves_the_backing_buffer() {
    let source = "const buffer = new ArrayBuffer(8); const source = new Uint16Array(buffer); source[0] = 1; source[1] = 2; source[2] = 3; source[3] = 4; let observed = ''; source.constructor = { [Symbol.species]: function(speciesBuffer, byteOffset, length) { observed = (speciesBuffer === buffer) + ':' + byteOffset + ':' + length; return new Uint8Array(speciesBuffer, byteOffset, length); } }; const result = source.subarray(1, 3); const replacement = new Uint8Array([9, 8, 7]); source.constructor = { [Symbol.species]: function() { return replacement; } }; const preserved = source.subarray(0, 0); observed + '|' + (result.buffer === buffer) + ':' + result.byteOffset + ':' + result.length + ':' + result[0] + '|' + (preserved === replacement);";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray.prototype.subarray should use the selected species constructor");
    assert!(
        outcome.note.contains("string(true:2:2|true:2:2:2|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_subarray_observes_resize_and_detach_boundaries() {
    let source = "const buffer = new ArrayBuffer(4, { maxByteLength: 8 }); const tracking = new Uint8Array(buffer); tracking[0] = 1; tracking[1] = 2; tracking[2] = 3; tracking[3] = 4; const result = tracking.subarray(1); buffer.resize(6); const tracked = result.byteOffset + ':' + result.length; const detached = new Uint8Array([1, 2]); let conversions = ''; const begin = { valueOf() { conversions += 'b'; return 0; } }; const end = { valueOf() { conversions += 'e'; return 1; } }; __scriptwDetachArrayBuffer(detached.buffer); let errorName = 'missing'; try { detached.subarray(begin, end); } catch (error) { errorName = error.name; } tracked + '|' + conversions + ':' + errorName;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray.prototype.subarray should preserve resize and detach ordering");
    assert!(
        outcome.note.contains("string(1:5|be:TypeError)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_at_validates_out_of_bounds_dynamic_receivers() {
    let source = "function readFirst(view) { return view.at(0); } const typedArrayAt = Object.getPrototypeOf(Uint8Array).prototype.at; const buffer = new ArrayBuffer(4, { maxByteLength: 8 }); const fixed = new Uint8Array(buffer, 1, 2); buffer.resize(2); let methodResult = 'missing'; try { readFirst(fixed); } catch (error) { methodResult = error.name; } let intrinsicResult = 'missing'; try { typedArrayAt.call(fixed, 0); } catch (error) { intrinsicResult = error.name; } let objectResult = 'missing'; try { typedArrayAt.call({ length: 1, 0: 1 }, 0); } catch (error) { objectResult = error.name; } let arrayResult = 'missing'; try { typedArrayAt.call([1], 0); } catch (error) { arrayResult = error.name; } const genericResult = Array.prototype.at.call(fixed, 0); methodResult + '|' + intrinsicResult + '|' + objectResult + '|' + arrayResult + '|' + (genericResult === undefined) + '|' + (Uint8Array.prototype.at === Array.prototype.at);";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray.prototype.at should validate out-of-bounds dynamic receivers");
    assert!(
        outcome
            .note
            .contains("string(TypeError|TypeError|TypeError|TypeError|true|false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_at_coerces_index_on_empty_receivers() {
    let source = "const values = new Uint8Array(); const typedArrayAt = Object.getPrototypeOf(Uint8Array).prototype.at; const events = []; const directIndex = { valueOf() { events.push('direct'); return 0; } }; const intrinsicIndex = { valueOf() { events.push('intrinsic'); return 0; } }; const direct = values.at(directIndex); const intrinsic = typedArrayAt.call(values, intrinsicIndex); (direct === undefined) + '|' + (intrinsic === undefined) + '|' + events.join(',');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray.prototype.at should coerce indexes on empty receivers");
    assert!(
        outcome.note.contains("string(true|true|direct,intrinsic)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_at_propagates_index_coercion_from_empty_receivers() {
    let source = "const values = new Uint8Array(); const typedArrayAt = Object.getPrototypeOf(Uint8Array).prototype.at; const directError = {}; const intrinsicError = {}; let directCaught = false; let intrinsicCaught = false; try { values.at({ valueOf() { throw directError; } }); } catch (error) { directCaught = error === directError; } try { typedArrayAt.call(values, { valueOf() { throw intrinsicError; } }); } catch (error) { intrinsicCaught = error === intrinsicError; } directCaught + '|' + intrinsicCaught;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray.prototype.at should propagate index coercion failures");
    assert!(
        outcome.note.contains("string(true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_join_formats_bigint_elements() {
    let source = "const signed = new BigInt64Array([1n, 0n, 2n, -3n]); const unsigned = new BigUint64Array([1n, 42n]); signed.join(',') + '|' + signed.join(null) + '|' + unsigned.join('-');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray.prototype.join should format BigInt elements");
    assert!(
        outcome
            .note
            .contains("string(1,0,2,-3|1null0null2null-3|1-42)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_typed_array_fill_accepts_number_and_bigint_elements() {
    let source = "const numbers = new Int32Array([1, 2, 3]); const bigints = new BigInt64Array([1n, 2n]); numbers.fill(7, 1); bigints.fill(-5n); numbers.join(',') + '|' + bigints.join(',');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("TypedArray.prototype.fill should write Number and BigInt elements");
    assert!(
        outcome.note.contains("string(1,7,7|-5,-5)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_integer_typed_array_writes_apply_modulo_conversions() {
    let source = r#"
const u32 = new Uint32Array(6);
u32[0] = 4294967295;
u32[1] = 4294967296;
u32[2] = -1;
u32[3] = -4294967295;
u32[4] = Infinity;
u32[5] = NaN;
const i32 = new Int32Array(4);
i32[0] = 4294967295;
i32[1] = 2147483648;
i32[2] = -2147483649;
i32[3] = 4294967297.9;
const u16 = new Uint16Array(4);
u16[0] = 65535;
u16[1] = 65536;
u16[2] = -1;
u16[3] = -65535;
const i16 = new Int16Array(3);
i16[0] = 65535;
i16[1] = 32768;
i16[2] = -32769;
const u8 = new Uint8Array(4);
u8[0] = 255;
u8[1] = 256;
u8[2] = -1;
u8[3] = -255;
const i8 = new Int8Array(3);
i8[0] = 255;
i8[1] = 128;
i8[2] = -129;
const clamped = new Uint8ClampedArray(4);
clamped[0] = 300;
clamped[1] = -1;
clamped[2] = 0.5;
clamped[3] = 1.5;
u32.join(",") + "|" + i32.join(",") + "|" +
  u16.join(",") + "|" + i16.join(",") + "|" +
  u8.join(",") + "|" + i8.join(",") + "|" + clamped.join(",");
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("integer typed array writes should apply modulo conversions");
    assert!(
            outcome.note.contains(
                "string(4294967295,0,4294967295,1,0,0|-1,-2147483648,2147483647,1|65535,0,65535,1|-1,-32768,32767|255,0,255,1|-1,-128,127|255,0,0,2)"
            ),
            "note: {}",
            outcome.note
        );
}

#[test]
fn wasm_backend_dataview_get_biguint64_returns_unsigned_bigints() {
    let source = r#"
const buffer = new ArrayBuffer(8);
const view = new DataView(buffer);
for (let index = 0; index < 8; index++) view.setUint8(index, 255);
const maximum = view.getBigUint64(0);
const reversed = view.getBigUint64(0, true);
(maximum === 18446744073709551615n) + "|" +
  (reversed === 18446744073709551615n) + "|" +
  (maximum === 9223372036854775807n) + "|" +
  (18446744073709551615n === 18446744073709551615n) + "|" +
  (typeof maximum) + "|" + (!!maximum);
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("getBigUint64 should preserve the full unsigned 64-bit range");
    assert!(
        outcome
            .note
            .contains("string(true|true|false|true|bigint|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_dataview_number_getters_ignore_legacy_mirrors_across_widths() {
    let source = r#"
const buffer = new ArrayBuffer(26);
const view = new DataView(buffer, 2, 22);
view.setInt8(0, -1);
view.setUint16(1, 0x1234, false);
view.setInt32(3, -2, true);
view.setFloat32(7, 1.5, true);
view.setFloat64(11, -2.25, false);
view.setUint16(20, 0x3c00, false);
view.buffer = new ArrayBuffer(1);
view.$DataViewDataPtr = 0;
view.$DataViewByteOffset = 0;
view.$DataViewByteLength = 1;
view.$DataViewLengthTracking = true;
[
  view.getInt8(0),
  view.getUint8(0),
  view.getUint16(1, false),
  view.getInt16(1, false),
  view.getUint32(3, true),
  view.getInt32(3, true),
  view.getFloat16(20, false),
  view.getFloat32(7, true),
  view.getFloat64(11, false)
].join(",");
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("DataView number getters should read private state across widths");
    assert!(
        outcome
            .note
            .contains("string(-1,255,4660,4660,4294967294,-2,1,1.5,-2.25)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_dataview_number_getters_preserve_validation_and_toindex_order() {
    let source = r#"
const forged = {
  buffer: new ArrayBuffer(16),
  $DataViewDataPtr: 1,
  $DataViewByteOffset: 0,
  $DataViewByteLength: 16,
  $DataViewLengthTracking: false
};
const getters = [
  DataView.prototype.getInt8,
  DataView.prototype.getUint16,
  DataView.prototype.getInt32,
  DataView.prototype.getFloat16,
  DataView.prototype.getFloat32,
  DataView.prototype.getFloat64
];
let forgedErrors = 0;
let forgedCoercions = 0;
for (const getter of getters) {
  try {
    getter.call(forged, {
      valueOf() {
        forgedCoercions++;
        return 0;
      }
    });
  } catch (error) {
    if (error.name === "TypeError") forgedErrors++;
  }
}
const detachedBuffer = new ArrayBuffer(8);
const detached = new DataView(detachedBuffer);
__scriptwDetachArrayBuffer(detachedBuffer);
const offsetError = {};
let abruptOffset = false;
try {
  detached.getFloat64({
    valueOf() {
      throw offsetError;
    }
  });
} catch (error) {
  abruptOffset = error === offsetError;
}
let detachedError = "missing";
try {
  detached.getUint16(0);
} catch (error) {
  detachedError = error.name;
}
const fixedBuffer = new ArrayBuffer(8, { maxByteLength: 16 });
const fixed = new DataView(fixedBuffer, 0, 8);
fixedBuffer.resize(4);
let fixedError = "missing";
try {
  fixed.getInt32(0);
} catch (error) {
  fixedError = error.name;
}
const trackingBuffer = new ArrayBuffer(8, { maxByteLength: 16 });
const tracking = new DataView(trackingBuffer);
trackingBuffer.resize(16);
const grownValue = tracking.getFloat64(8);
forgedErrors + ":" + forgedCoercions + "|" + abruptOffset + ":" +
  detachedError + "|" + fixedError + "|" + grownValue;
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("DataView number getters should preserve validation and ToIndex order");
    assert!(
        outcome
            .note
            .contains("string(6:0|true:TypeError|TypeError|0)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_dataview_setters_ignore_legacy_mirrors_across_widths() {
    let source = r#"
const buffer = new ArrayBuffer(18);
const view = new DataView(buffer, 2, 16);
Object.defineProperty(view, "buffer", { value: new ArrayBuffer(1), writable: true });
view.$DataViewDataPtr = 0;
view.$DataViewByteOffset = 0;
view.$DataViewByteLength = 1;
view.$DataViewLengthTracking = true;
const results = [];
results.push(view.setInt8(0, -1) === undefined && view.getInt8(0) === -1);
results.push(view.setUint8(0, 255) === undefined && view.getUint8(0) === 255);
results.push(view.setInt16(0, -2, true) === undefined && view.getInt16(0, true) === -2);
results.push(view.setUint16(0, 0x1234, false) === undefined &&
  view.getUint16(0, false) === 0x1234);
results.push(view.setInt32(0, -3, true) === undefined && view.getInt32(0, true) === -3);
results.push(view.setUint32(0, 0x89abcdef, false) === undefined &&
  view.getUint32(0, false) === 0x89abcdef);
results.push(view.setFloat16(0, 1.5, true) === undefined &&
  view.getFloat16(0, true) === 1.5);
results.push(view.setFloat32(0, -2.25, false) === undefined &&
  view.getFloat32(0, false) === -2.25);
results.push(view.setFloat64(0, 3.5, true) === undefined &&
  view.getFloat64(0, true) === 3.5);
results.push(view.setBigInt64(0, -4n, false) === undefined &&
  view.getBigInt64(0, false) === -4n);
results.push(view.setBigUint64(0, 18446744073709551615n, true) === undefined &&
  view.getBigUint64(0, true) === 18446744073709551615n);
results.join(",");
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("DataView setters should write through private state across widths");
    assert!(
        outcome
            .note
            .contains("string(true,true,true,true,true,true,true,true,true,true,true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_dataview_setters_validate_receiver_and_preserve_coercion_order() {
    let source = r#"
const forged = {
  buffer: new ArrayBuffer(16),
  $DataViewDataPtr: 1,
  $DataViewByteOffset: 0,
  $DataViewByteLength: 16,
  $DataViewLengthTracking: false
};
const setters = [
  DataView.prototype.setInt8,
  DataView.prototype.setUint8,
  DataView.prototype.setInt16,
  DataView.prototype.setUint16,
  DataView.prototype.setInt32,
  DataView.prototype.setUint32,
  DataView.prototype.setFloat16,
  DataView.prototype.setFloat32,
  DataView.prototype.setFloat64,
  DataView.prototype.setBigInt64,
  DataView.prototype.setBigUint64
];
let receiverErrors = 0;
let receiverCoercions = 0;
for (const setter of setters) {
  try {
    setter.call(
      forged,
      { valueOf() { receiverCoercions++; return 0; } },
      { valueOf() { receiverCoercions++; return 0; } }
    );
  } catch (error) {
    if (error.name === "TypeError") receiverErrors++;
  }
}
const view = new DataView(new ArrayBuffer(8));
const numberError = {};
let numberOrder = "";
let abruptNumber = false;
try {
  view.setUint32(
    { valueOf() { numberOrder += "o"; return 100; } },
    { valueOf() { numberOrder += "v"; throw numberError; } }
  );
} catch (error) {
  abruptNumber = error === numberError;
}
const bigintError = {};
let bigintOrder = "";
let abruptBigint = false;
try {
  view.setBigInt64(
    { valueOf() { bigintOrder += "o"; return 100; } },
    { valueOf() { bigintOrder += "v"; throw bigintError; } }
  );
} catch (error) {
  abruptBigint = error === bigintError;
}
let negativeValueCoercions = 0;
let negativeError = "missing";
try {
  view.setFloat64(-1, {
    valueOf() {
      negativeValueCoercions++;
      return 1;
    }
  });
} catch (error) {
  negativeError = error.name;
}
let unsafeOffsetErrors = 0;
for (const offset of [9007199254740992, -1e300]) {
  try {
    view.setInt8(offset, {
      valueOf() {
        negativeValueCoercions++;
        return 1;
      }
    });
  } catch (error) {
    if (error.name === "RangeError") unsafeOffsetErrors++;
  }
}
let numericBigintError = "missing";
try {
  view.setBigUint64(0, 1);
} catch (error) {
  numericBigintError = error.name;
}
let bigintNumberError = "missing";
try {
  view.setInt16(0, 1n);
} catch (error) {
  bigintNumberError = error.name;
}
receiverErrors + ":" + receiverCoercions + "|" +
  abruptNumber + ":" + numberOrder + "|" + abruptBigint + ":" + bigintOrder + "|" +
  negativeError + ":" + unsafeOffsetErrors + ":" + negativeValueCoercions + "|" +
  numericBigintError + ":" + bigintNumberError;
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("DataView setters should preserve receiver, offset, and value coercion order");
    assert!(
        outcome
            .note
            .contains("string(11:0|true:ov|true:ov|RangeError:2:0|TypeError:TypeError)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_dataview_setters_observe_buffer_changes_during_value_coercion() {
    let source = r#"
const growingBuffer = new ArrayBuffer(8, { maxByteLength: 16 });
const growing = new DataView(growingBuffer);
growing.setFloat64(8, {
  valueOf() {
    growingBuffer.resize(16);
    return 1.5;
  }
}, true);
const grown = growing.getFloat64(8, true);

const fixedBuffer = new ArrayBuffer(8, { maxByteLength: 16 });
const fixed = new DataView(fixedBuffer, 0, 8);
let fixedError = "missing";
try {
  fixed.setInt32(0, {
    valueOf() {
      fixedBuffer.resize(4);
      return 1;
    }
  });
} catch (error) {
  fixedError = error.name;
}

const shrinkingBuffer = new ArrayBuffer(8, { maxByteLength: 16 });
const shrinking = new DataView(shrinkingBuffer);
let shrinkingError = "missing";
try {
  shrinking.setUint32(0, {
    valueOf() {
      shrinkingBuffer.resize(2);
      return 1;
    }
  });
} catch (error) {
  shrinkingError = error.name;
}

const detachedBuffer = new ArrayBuffer(8);
const detached = new DataView(detachedBuffer);
const valueError = {};
let abruptValue = false;
try {
  detached.setBigInt64(0, {
    valueOf() {
      __scriptwDetachArrayBuffer(detachedBuffer);
      throw valueError;
    }
  });
} catch (error) {
  abruptValue = error === valueError;
}
let detachedError = "missing";
try {
  detached.setBigInt64(0, 1n);
} catch (error) {
  detachedError = error.name;
}
grown + "|" + fixedError + "|" + shrinkingError + "|" +
  abruptValue + ":" + detachedError;
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("DataView setters should validate current buffer state after value coercion");
    assert!(
        outcome
            .note
            .contains("string(1.5|TypeError|RangeError|true:TypeError)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_buffer_is_view_uses_private_brands() {
    let source = r#"
const forged = {
  $DataViewDataPtr: 1,
  $TypedArrayViewedArrayBuffer: new ArrayBuffer(1)
};
const view = new DataView(new ArrayBuffer(1));
const typed = new Uint8Array(1);
view.$DataViewDataPtr = 0;
typed.$TypedArrayViewedArrayBuffer = undefined;
[
  ArrayBuffer.isView(forged),
  ArrayBuffer.isView(view),
  ArrayBuffer.isView(typed),
  ArrayBuffer.isView(new Proxy(view, {})),
  ArrayBuffer.isView(new Proxy(typed, {}))
].join(",");
"#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("ArrayBuffer.isView should use private view brands");
    assert!(
        outcome.note.contains("string(false,true,true,false,false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_slice_uses_current_resizable_typed_array_bounds() {
    let source = "const buffer = new ArrayBuffer(4, { maxByteLength: 8 }); const tracking = new Uint8Array(buffer); tracking[0] = 1; tracking[1] = 2; tracking[2] = 3; tracking[3] = 4; const grow = { valueOf() { buffer.resize(6); return 0; } }; Array.prototype.slice.call(tracking, grow); const grown = Array.prototype.slice.call(tracking, 3, 5); const fixed = new Uint8Array(buffer, 0, 4); const shrink = { valueOf() { buffer.resize(2); return 0; } }; const fixedResult = Array.prototype.slice.call(fixed, shrink); const trackingResult = Array.prototype.slice.call(tracking); grown.join(',') + '|' + fixedResult.length + '|' + fixedResult.hasOwnProperty(0) + '|' + trackingResult.join(',');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.slice should use current resizable typed array bounds");
    assert!(
        outcome.note.contains("string(4,0|4|false|1,2)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_pop_is_generic_and_preserves_inherited_indexes() {
    let source = "const object = { 0: 'a', length: 1 }; const value = Array.prototype.pop.call(object); const empty = {}; const emptyResult = Array.prototype.pop.call(empty); let stringThrows = false; try { Array.prototype.pop.call(''); } catch (error) { stringThrows = error instanceof TypeError; } Array.prototype[1] = 7; const array = [1]; array.length = 2; const inherited = array.pop(); const remains = array[1]; delete Array.prototype[1]; value + '|' + object.length + '|' + !object.hasOwnProperty('0') + '|' + (emptyResult === undefined) + '|' + empty.length + '|' + (Array.prototype.pop.call(true) === undefined) + '|' + stringThrows + '|' + inherited + '|' + remains;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.pop should work on generic receivers");
    assert!(
        outcome
            .note
            .contains("string(a|0|true|true|0|true|true|7|7)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_copy_within_preserves_overlap_and_holes() {
    let source = "var values = [0, 1, , 3]; var result = values.copyWithin(1, 0, 3); (result === values) + '|' + values.join(',') + '|' + values.hasOwnProperty(3);";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.copyWithin should preserve overlap and holes");
    assert!(
        outcome.note.contains("string(true|0,0,1,|false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_concat_rejects_result_above_safe_integer_limit() {
    let source = "var source = { length: Number.MAX_SAFE_INTEGER }; source[Symbol.isConcatSpreadable] = true; var threw = false; try { [1].concat(source); } catch (error) { threw = error instanceof TypeError; } threw;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.concat should reject an oversized result");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_map_visits_arguments_indexes() {
    let source = "function callbackfn(val, idx, obj) { return val + ':' + idx + ':' + Object.prototype.toString.call(obj); } var r = (function(a, b) { return Array.prototype.map.call(arguments, callbackfn); })(9, 11); r.length + '|' + r[0] + '|' + r[1] + '|' + Object.keys(r).join(',');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| {
            panic!("Array.prototype.map should visit Arguments indexes: {err:?}")
        });
    assert!(
        outcome
            .note
            .contains("string(2|9:0:[object Arguments]|11:1:[object Arguments]|0,1)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_flat_map_uses_real_builtin_and_compact_target_indexes() {
    let source = "var a = [1, 2].flatMap(function(e) { return [e, e * 2]; }); var b = [1, 2].flatMap(function(e) { return e * 2; }); var c = [, 1].flatMap(function(e) { return e; }); function E() {} var threw = false; try { [].flatMap.call({ get length() { throw new E(); } }, function(e) { return e; }); } catch (e) { threw = e instanceof E; } var d = Array.prototype.flatMap.call(true, function() { return 1; }); a.length + '|' + a[0] + '|' + a[1] + '|' + a[2] + '|' + a[3] + '|' + b.length + '|' + b[0] + '|' + b[1] + '|' + c.length + '|' + c[0] + '|' + c[1] + '|' + Object.keys(c).join(',') + '|' + threw + '|' + d.length;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| {
            panic!("Array.prototype.flatMap should run through the real Wasm builtin: {err:?}")
        });
    assert!(
        outcome
            .note
            .contains("string(4|1|2|2|4|2|2|4|1|1|undefined|0|true|0)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_concat_boxes_boolean_receiver() {
    let source = "var t = Array.prototype.concat.call(true); var f = Array.prototype.concat.call(false); t.length + '|' + (t[0] instanceof Boolean) + '|' + f.length + '|' + (f[0] instanceof Boolean);";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| {
            panic!("Array.prototype.concat should box Boolean receivers: {err:?}")
        });
    assert!(
        outcome.note.contains("string(1|true|1|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_concat_spreads_arguments_without_changing_length() {
    let source = "var args = (function(a, b, c) { return arguments; })(1, 2, 3); args[Symbol.isConcatSpreadable] = true; var r = [].concat(args, args); Object.defineProperty(args, 'length', { value: 6 }); var s = [].concat(args); r.length + '|' + r[0] + '|' + r[1] + '|' + r[2] + '|' + r[3] + '|' + r[4] + '|' + r[5] + '|' + args.length + '|' + s.length + '|' + s[0] + '|' + s[1] + '|' + s[2] + '|' + s[3] + '|' + s[4] + '|' + s[5];";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| {
            panic!("Array.prototype.concat should spread Arguments objects: {err:?}")
        });
    assert!(
        outcome
            .note
            .contains("string(6|1|2|3|1|2|3|6|6|1|2|3|undefined|undefined|undefined)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_concat_spreads_string_wrapper_by_utf16_code_unit() {
    let source = "var str = new String('yuck\\uD83D\\uDCA9'); str[Symbol.isConcatSpreadable] = true; var s = [].concat(str); s.length + '|' + s[0] + s[1] + s[2] + s[3] + '|' + (s[4] === '\\uD83D') + '|' + (s[5] === '\\uDCA9');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| {
            panic!(
                "Array.prototype.concat should spread String wrappers by UTF-16 code unit: {err:?}"
            )
        });
    assert!(
        outcome.note.contains("string(6|yuck|true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_concat_preserves_inherited_function_indexes_across_calls() {
    let source = "var own = function(a, b, c) {}; own[Symbol.isConcatSpreadable] = true; own[0] = 1; own[1] = 2; own[2] = 3; var first = [].concat(own); Function.prototype[Symbol.isConcatSpreadable] = true; var holes = [].concat(function(a, b, c) {}); Function.prototype[0] = 1; Function.prototype[1] = 2; Function.prototype[2] = 3; var inherited = function(a, b, c) {}; var second = [].concat(inherited); first.join(',') + '|' + holes.length + '|' + second.join(',') + '|' + Object.keys(second).join(',');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| {
            panic!("Array.prototype.concat should preserve inherited Function indexes: {err:?}")
        });
    assert!(
        outcome.note.contains("string(1,2,3|3|1,2,3|0,1,2)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_concat_copies_inherited_array_indexes_as_own_properties() {
    let source = "Array.prototype[1] = 1; var x = [0]; x.length = 2; var a = x.concat(); Object.prototype[1] = 1; Object.prototype.length = 2; Object.prototype.concat = Array.prototype.concat; var y = { 0: 0 }; var b = y.concat(); a[0] + '|' + a[1] + '|' + a.hasOwnProperty('1') + '|' + (b[0] === y) + '|' + b[1] + '|' + b.hasOwnProperty('1');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| {
            panic!("Array.prototype.concat should copy inherited array indexes: {err:?}")
        });
    assert!(
        outcome.note.contains("string(0|1|true|true|1|false)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_concat_propagates_is_concat_spreadable_getter_throw() {
    let source = "function E() {} var o = {}; Object.defineProperty(o, Symbol.isConcatSpreadable, { get: function() { throw new E(); } }); var caught = false; try { Array.prototype.concat.call(o); } catch (e) { caught = e instanceof E; } caught;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| {
            panic!("Array.prototype.concat should propagate spreadability getter throws: {err:?}")
        });
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_concat_propagates_arguments_index_getter_throw() {
    let source = "function E() {} var args = (function(a) { return arguments; })(1, 2, 3); Object.defineProperty(args, 0, { get: function() { throw new E(); } }); args[Symbol.isConcatSpreadable] = true; var caught = false; try { [].concat(args, args); } catch (error) { caught = error instanceof E; } caught;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("Array.prototype.concat should propagate arguments index getter throws");
    assert!(
        outcome.note.contains("boolean(true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_concat_spreads_typed_array_from_dynamic_constructor() {
    let source = "function check(type) { var ta = new type(2); for (var i = 0; i < 2; ++i) { ta[i] = i + 1; } ta[Symbol.isConcatSpreadable] = true; var r = [].concat(ta); return r.length === 2 && r[0] === 1 && r[1] === 2 && r.hasOwnProperty('1'); } check(Uint16Array) + '|' + check(Uint32Array) + '|' + check(Float64Array);";
    let outcome = engine()
            .run_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|err| {
                panic!(
                    "Array.prototype.concat should spread typed arrays built through dynamic constructors: {err:?}"
                )
            });
    assert!(
        outcome.note.contains("string(true|true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_reads_typed_array_indexes_from_callback_constructor() {
    let source = "var values = []; [Uint8Array, Uint16Array, Uint32Array, Float32Array, Float64Array].forEach(function(type) { var array = new type(1); values.push(array[0]); }); values.join(':');";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("dynamic property reads should preserve TypedArray integer-index semantics");
    assert!(
        outcome.note.contains("string(0:0:0:0:0)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_array_concat_ignores_cross_realm_array_species() {
    let source = "var array = []; var callCount = 0; var OArray = __scriptwCreateRealm().global.Array; var speciesDesc = { get: function() { callCount += 1; } }; array.constructor = OArray; Object.defineProperty(Array, Symbol.species, speciesDesc); Object.defineProperty(OArray, Symbol.species, speciesDesc); var result = array.concat(); (Object.getPrototypeOf(result) === Array.prototype) + '|' + callCount;";
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| {
            panic!(
                "Array.prototype.concat should ignore cross-realm intrinsic Array species: {err:?}"
            )
        });
    assert!(
        outcome.note.contains("string(true|0)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_iterates_captured_array_concat_result() {
    let outcome = engine()
        .run_script(
            r#"
const baseValues = [1, 2];
const values = baseValues.concat(3);

function sumValues() {
  let sum = 0;
  for (const value of values) {
    sum += value;
  }
  return sum;
}

sumValues();
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("captured Array.prototype.concat results should remain iterable arrays");
    assert!(outcome.note.contains("number(6)"), "note: {}", outcome.note);
}

#[test]
fn wasm_backend_set_constructs_from_array_and_string_iterables() {
    let outcome = engine()
            .run_script(
                "var numbers = new Set([1, 2, 2, -0]); var letters = new Set('aba'); [numbers.size, numbers.has(0), letters.size, letters.has('a'), letters.has('b')].join('|');",
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("Set should consume array and string iterables");
    assert!(
        outcome.note.contains("string(3|true|2|true|true)"),
        "note: {}",
        outcome.note
    );
}

#[test]
fn wasm_backend_for_await_array_awaits_values_and_preserves_iteration_bindings() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            async function collect() {
                const readers = [];
                for await (const value of [Promise.resolve(1), 2]) {
                    readers.push(function () { return value; });
                }
                return readers[0]() + readers[1]();
            }
            const result = collect();
            print(result instanceof Promise);
            result.then(function (value) { print("sum:" + value); });
            print("sync");
        "#;
    engine_with_captured_prints(Arc::clone(&lines))
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("array-backed for-await-of should execute: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &["true".to_string(), "sync".to_string(), "sum:3".to_string(),]
    );
}

#[test]
fn wasm_backend_for_await_array_propagates_break_and_return() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            async function breakAfterOne() {
                let count = 0;
                for await (const value of [1, 2]) {
                    count += value;
                    break;
                }
                return count;
            }
            async function returnFirst() {
                for await (const value of [3, 4]) {
                    return value;
                }
            }
            breakAfterOne().then(function (value) { print("break:" + value); });
            returnFirst().then(function (value) { print("return:" + value); });
            print("sync");
        "#;
    engine_with_captured_prints(Arc::clone(&lines))
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("for-await-of completions should propagate: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &[
            "sync".to_string(),
            "break:1".to_string(),
            "return:3".to_string(),
        ]
    );
}

#[test]
fn wasm_backend_for_await_array_rejections_enter_async_catch() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            async function catchesRejection() {
                try {
                    for await (const value of [Promise.reject("boom")]) {
                        print("unreachable:" + value);
                    }
                } catch (error) {
                    return "caught:" + error;
                }
                return "unreachable";
            }
            catchesRejection().then(function (value) { print(value); });
            print("sync");
        "#;
    engine_with_captured_prints(Arc::clone(&lines))
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("for-await-of rejection should be catchable: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &["sync".to_string(), "caught:boom".to_string()]
    );
}

#[test]
fn wasm_backend_for_await_array_uses_observable_sync_iterator_protocol() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            let calls = "";
            let source = [1, 2, 3];
            Array.prototype[Symbol.iterator] = function () {
                "use strict";
                calls += this === source ? "i" : "w";
                let index = 0;
                let iterator = {
                    get next() {
                        calls += "g";
                        return function () {
                            calls += "n";
                            index += 1;
                            return index <= 3
                                ? { value: Promise.resolve(index), done: false }
                                : { value: undefined, done: true };
                        };
                    },
                    return: function () {
                        calls += this === iterator ? "r" : "w";
                        return { done: true };
                    }
                };
                return iterator;
            };
            async function collect() {
                let sum = 0;
                for await (const value of source) {
                    calls += value;
                    sum += value;
                    if (sum === 3) break;
                }
                return sum;
            }
            collect().then(function (result) { print(result + ":" + calls); });
            print("sync");
        "#;
    engine_with_captured_prints(Arc::clone(&lines))
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("array iterator protocol should execute: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &["sync".to_string(), "3:ign1n2r".to_string()]
    );
}

#[test]
fn wasm_backend_awaited_lexical_array_preserves_element_evaluation_order() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            function record(label) {
                print(label);
                return label;
            }
            function throwBetweenAwaits() {
                print("throw-between");
                throw "stop";
            }
            async function collect() {
                const values = [
                    record("before"),
                    await Promise.resolve("a"),
                    record("between"),
                    await Promise.resolve("b"),
                    record("after"),
                ];
                print(values.join("|"));
                const sparse = [, await Promise.resolve("held"), ,];
                print(
                    sparse.length + ":" +
                    (0 in sparse) + ":" +
                    sparse[1] + ":" +
                    (2 in sparse)
                );
            }
            async function failBetweenAwaits() {
                const values = [
                    record("fail-before"),
                    await Promise.resolve("first"),
                    throwBetweenAwaits(),
                    await Promise.resolve("second"),
                    record("unreachable"),
                ];
                return values;
            }
            collect()
                .then(function () { return failBetweenAwaits(); })
                .then(undefined, function (reason) { print("rejected:" + reason); });
            print("sync");
        "#;
    engine_with_captured_prints(Arc::clone(&lines))
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| {
            panic!("awaited lexical array elements should resume in source order: {err:?}")
        });
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &[
            "before".to_string(),
            "sync".to_string(),
            "between".to_string(),
            "after".to_string(),
            "before|a|between|b|after".to_string(),
            "3:false:held:false".to_string(),
            "fail-before".to_string(),
            "throw-between".to_string(),
            "rejected:stop".to_string(),
        ]
    );
}

#[test]
fn wasm_backend_async_generator_array_spread_resumes_with_an_iterable() {
    let lines = Arc::new(Mutex::new(Vec::new()));
    let source = r#"
            async function* stream() { yield [0, ...yield, 3]; }
            let iterator = stream();
            iterator.next().then(function () {
                return iterator.next("ab");
            }).then(function (result) {
                print(result.value.join(",") + ":" + result.done);
            });
        "#;
    engine_with_captured_prints(Arc::clone(&lines))
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("async-generator array spread should run: {err:?}"));
    assert_eq!(
        lines.lock().expect("capture mutex poisoned").as_slice(),
        &["0,a,b,3:false".to_string()]
    );
}

#[test]
fn wasm_backend_array_accumulation_observes_iterator_and_direct_fresh_writes() {
    let source = r#"
            const values = [10, 20];
            const marker = {};
            let events = "";
            let closeCalls = 0;
            let setterCalls = 0;
            let index = 0;

            Array.prototype[Symbol.iterator] = function () {
                "use strict";
                events += this === values ? "i" : "w";
                const receiver = this;
                return {
                    next() {
                        events += "n";
                        return index < receiver.length
                            ? { value: receiver[index++], done: false }
                            : { done: true };
                    },
                    return() {
                        closeCalls++;
                        return {};
                    }
                };
            };
            Object.defineProperty(Array.prototype, "0", {
                configurable: true,
                set() { setterCalls++; }
            });

            function before() { events += "b"; return 1; }
            function after() { events += "a"; return 3; }
            const result = [before(), , ...values, after()];
            delete Array.prototype["0"];

            const abrupt = {
                [Symbol.iterator]() {
                    return {
                        next() {
                            return { get value() { throw marker; }, done: false };
                        },
                        return() { closeCalls++; return {}; }
                    };
                }
            };
            let caught = false;
            try { [...abrupt]; } catch (error) { caught = error === marker; }

            result.length === 5
                && result[0] === 1
                && !(1 in result)
                && result[2] === 10 && result[3] === 20 && result[4] === 3
                && events === "binnna"
                && setterCalls === 0
                && closeCalls === 0
                && caught;
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("array accumulation protocol should run: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_generator_array_accumulation_commits_prefix_before_yield() {
    let source = r#"
            let events = "";
            function before() { events += "b"; return 1; }
            function after() { events += "a"; return 3; }
            function* stream() { yield [before(), ...yield, after()]; }

            const iterator = stream();
            const suspended = iterator.next();
            const prefixWasCommitted = events === "b";
            const yielded = iterator.next("xy");
            const value = yielded.value;

            suspended.value === undefined && suspended.done === false
                && prefixWasCommitted
                && events === "ba"
                && yielded.done === false
                && value.length === 4
                && value[0] === 1 && value[1] === "x"
                && value[2] === "y" && value[3] === 3;
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("generator array accumulation order should run: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_with_typed_array_reads_outer_infinity_binding() {
    let source = r#"
            function observe(view) {
                with (view) {
                    return Infinity === Infinity;
                }
            }

            const numeric = new Uint8Array([1]);
            const bigint = new BigInt64Array([1n]);
            __scriptwDetachArrayBuffer(numeric.buffer);
            __scriptwDetachArrayBuffer(bigint.buffer);
            observe(numeric) && observe(bigint);
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("typed-array with Infinity fallback should compile: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn wasm_backend_with_typed_array_reads_outer_function_binding() {
    let source = r#"
            function readFallback() { return 17; }
            function observe(view) {
                with (view) {
                    return readFallback() === 17;
                }
            }

            const numeric = new Uint8Array([1]);
            __scriptwDetachArrayBuffer(numeric.buffer);
            observe(numeric);
        "#;
    let outcome = engine()
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|err| panic!("typed-array with function fallback should compile: {err:?}"));
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

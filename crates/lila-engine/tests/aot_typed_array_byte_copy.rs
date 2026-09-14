use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_typed_array_byte_copy(source: &str) {
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
        .expect("TypedArray byte-copy regression must execute through Wasm AOT");
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
fn same_kind_construction_and_set_preserve_all_binary16_nan_encodings() {
    assert_typed_array_byte_copy(
        r#"
var source = new Float16Array(2046), words = new Uint16Array(source.buffer);
for (var i = 1; i < 1024; i++) {
  words[i - 1] = 0x7c00 + i;
  words[i + 1022] = 0xfc00 + i;
}
var constructed = new Float16Array(source), assigned = new Float16Array(source.length);
assigned.set(source);
for (var result of [constructed, assigned]) {
  var copied = new Uint16Array(result.buffer);
  for (var i = 0; i < words.length; i++) {
    if (copied[i] !== words[i]) throw 'NaN bit copy at ' + i;
  }
}
print(true);
"#,
    );
}

#[test]
fn float32_and_float64_copies_preserve_signaling_nan_bits_and_offsets() {
    assert_typed_array_byte_copy(
        r#"
for (var Constructor of [Float32Array, Float64Array]) {
  var width = Constructor.BYTES_PER_ELEMENT;
  var buffer = new ArrayBuffer(4 * width);
  var words = new Uint32Array(buffer);
  for (var i = 0; i < words.length; i++) words[i] = i % 2 ? 0xff801234 : 0x7f800001;
  if (width === 8) {
    // Wasm backing stores are little endian. These two f64s are signaling NaNs.
    words[2] = 1; words[3] = 0x7ff00000;
    words[4] = 0x1234; words[5] = 0xfff00000;
  }
  var source = new Constructor(buffer, width, 2);
  var constructed = new Constructor(source);
  var assigned = new Constructor(4); assigned.set(source, 1);
  var expected = new Uint8Array(buffer, width, 2 * width);
  for (var actual of [new Uint8Array(constructed.buffer),
      new Uint8Array(assigned.buffer, width, 2 * width)]) {
    for (var i = 0; i < expected.length; i++) {
      if (actual[i] !== expected[i]) throw 'signaling NaN encoding';
    }
  }
  if (assigned[0] !== 0 || assigned[3] !== 0) throw 'set writes only requested range';
}
print(true);
"#,
    );
}

#[test]
fn same_kind_set_snapshots_overlap_in_both_directions_and_shared_memory() {
    assert_typed_array_byte_copy(
        r#"
for (var Buffer of [ArrayBuffer, SharedArrayBuffer]) {
  for (var Constructor of [Float16Array, Float32Array, Float64Array, Int8Array,
      Uint8Array, Uint8ClampedArray, Int16Array, Uint16Array, Int32Array, Uint32Array,
      BigInt64Array, BigUint64Array]) {
    var width = Constructor.BYTES_PER_ELEMENT;
    for (var backwards of [false, true]) {
      var buffer = new Buffer(6 * width), bytes = new Uint8Array(buffer);
      var original = [];
      for (var i = 0; i < bytes.length; i++) {
        bytes[i] = (i * 73 + 19) % 256; original.push(bytes[i]);
      }
      var sourceOffset = backwards ? 2 * width : width;
      var targetOffset = backwards ? width : 2 * width;
      var source = new Constructor(buffer, sourceOffset, 3);
      var target = new Constructor(buffer, targetOffset, 3);
      if (target.set(source) !== undefined) throw 'set completion';
      for (var i = 0; i < bytes.length; i++) {
        var expected = i >= targetOffset && i < targetOffset + 3 * width
          ? original[sourceOffset + i - targetOffset] : original[i];
        if (bytes[i] !== expected) throw 'overlapping copy';
      }
    }
  }
}
print(true);
"#,
    );
}

#[test]
fn copies_between_shared_and_resizable_buffers_preserve_bytes_and_witnesses() {
    assert_typed_array_byte_copy(
        r#"
var shared = new SharedArrayBuffer(4), source = new Float16Array(shared);
var words = new Uint16Array(shared); words[0] = 0x7c01; words[1] = 0xfe13;
var clone = new Float16Array(source);
if (clone.buffer instanceof SharedArrayBuffer || clone.buffer === shared) throw 'private clone';
if (new Uint16Array(clone.buffer)[0] !== 0x7c01 ||
    new Uint16Array(clone.buffer)[1] !== 0xfe13) throw 'shared source clone bytes';
var resizable = new ArrayBuffer(8, { maxByteLength: 16 });
var target = new Float16Array(resizable);
target.set(source, { valueOf() { resizable.resize(4); return 0; } });
if (new Uint16Array(resizable)[0] !== 0x7c01 ||
    new Uint16Array(resizable)[1] !== 0xfe13) throw 'live resized target';
words[0] = 0; words[1] = 0; source.set(target);
if (words[0] !== 0x7c01 || words[1] !== 0xfe13) throw 'private to shared';
var caught = undefined;
try {
  target.set(source, { valueOf() { __lilaDetachArrayBuffer(resizable); return 0; } });
} catch (error) { caught = error; }
if (!(caught instanceof TypeError)) throw 'detachment before raw write';
print(true);
"#,
    );
}

#[test]
fn set_checks_bounds_before_content_type_and_retains_numeric_conversion() {
    assert_typed_array_byte_copy(
        r#"
for (var Constructor of [Float16Array, Float32Array, Int16Array]) {
  for (var operation of [
    () => new Constructor(0).set(new BigInt64Array(1)),
    () => new BigInt64Array(0).set(new Constructor(1)),
    () => new Constructor(0).set(new BigInt64Array(0), 1)
  ]) {
    var caught = undefined;
    try { operation(); } catch (error) { caught = error; }
    if (!(caught instanceof RangeError)) throw 'capacity before content';
  }
  var caught = undefined;
  try { new Constructor(1).set(new BigInt64Array(0)); } catch (error) { caught = error; }
  if (!(caught instanceof TypeError)) throw 'content check even for empty source';
}
var detached = new Float16Array(1); __lilaDetachArrayBuffer(detached.buffer);
var caught = undefined;
try { new BigInt64Array(0).set(detached); } catch (error) { caught = error; }
if (!(caught instanceof TypeError)) throw 'source witness before capacity';
var source = new Float64Array([1 + 2 ** -11 + 2 ** -40, -2.5]);
var target = new Float16Array(2); target.set(source);
if (new Uint16Array(target.buffer)[0] !== 0x3c01 || target[1] !== -2.5) throw 'different-kind conversion';
var integers = new Int16Array(target);
if (integers[0] !== 1 || integers[1] !== -2) throw 'Number conversion';
print(true);
"#,
    );
}

#[test]
fn constructor_owned_backing_buffers_use_active_constructor_realm() {
    assert_typed_array_byte_copy(
        r#"
var other = __lilaCreateRealm().global;
var ForeignBuffer = other.ArrayBuffer;
for (var pair of [[Float16Array, other.Float16Array], [Float32Array, other.Float32Array],
    [BigInt64Array, other.BigInt64Array]]) {
  var Local = pair[0], Foreign = pair[1];
  var foreignPrototype = other.ArrayBuffer.prototype;
  for (var argument of [1, [], new Local(1)]) {
    var values = new Foreign(argument);
    if (Object.getPrototypeOf(values.buffer) !== foreignPrototype) throw 'foreign backing buffer';
  }
  var values = Reflect.construct(Foreign, [1], Local);
  if (Object.getPrototypeOf(values) !== Local.prototype ||
      Object.getPrototypeOf(values.buffer) !== foreignPrototype) throw 'constructor realm versus newTarget';
  values = Reflect.construct(Local, [1], Foreign);
  if (Object.getPrototypeOf(values) !== Foreign.prototype ||
      Object.getPrototypeOf(values.buffer) !== ArrayBuffer.prototype) throw 'entry constructor realm';
  var supplied = new ArrayBuffer(8);
  if (new Foreign(supplied).buffer !== supplied) throw 'supplied buffer identity';
  other.ArrayBuffer = function Replacement() { throw 'mutable global constructor'; };
  if (Object.getPrototypeOf(new Foreign(1).buffer) !== foreignPrototype) throw 'intrinsic retained';
  other.ArrayBuffer = ForeignBuffer;
}
print(true);
"#,
    );
}

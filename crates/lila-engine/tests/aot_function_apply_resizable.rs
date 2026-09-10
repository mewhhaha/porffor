use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

#[test]
fn apply_reads_current_resizable_typed_array_lengths_after_shrinking_and_growing() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let source = r#"
class Uint8Subclass extends Uint8Array {}
class Float32Subclass extends Float32Array {}
class BigInt64Subclass extends BigInt64Array {}
var constructors = [
  Uint8Array, Int8Array, Uint16Array, Int16Array, Uint32Array, Int32Array,
  Float32Array, Float64Array, Uint8ClampedArray, BigUint64Array, BigInt64Array,
  Uint8Subclass, Float32Subclass, BigInt64Subclass
];
function collect(...args) { return [...args]; }
function values(view) { return collect.apply(null, view).join(','); }
for (var ctor of constructors) {
  var width = ctor.BYTES_PER_ELEMENT;
  var buffer = new ArrayBuffer(4 * width, { maxByteLength: 8 * width });
  var fixed = new ctor(buffer, 0, 4);
  var fixedOffset = new ctor(buffer, 2 * width, 2);
  var tracking = new ctor(buffer);
  var trackingOffset = new ctor(buffer, 2 * width);
  for (var index = 0; index < 4; index++) {
    tracking[index] = tracking instanceof BigInt64Array || tracking instanceof BigUint64Array
      ? BigInt(index) : index;
  }
  var trace = [];
  trace.push([values(fixed), values(fixedOffset), values(tracking), values(trackingOffset)].join('|'));
  for (var size of [3, 1, 0, 6]) {
    buffer.resize(size * width);
    trace.push([values(fixed), values(fixedOffset), values(tracking), values(trackingOffset)].join('|'));
  }
  var expected = [
    '0,1,2,3|2,3|0,1,2,3|2,3',
    '||0,1,2|2',
    '||0|',
    '|||',
    '0,0,0,0|0,0|0,0,0,0,0,0|0,0,0,0'
  ];
  if (trace.join(';') !== expected.join(';')) throw ctor.name + ': ' + trace.join(';');
}
print('all views preserved');
var control = new Uint8Array([7, 8]);
function Target(...args) { this.args = args; }
print(Reflect.apply(collect, null, control).join(',') === '7,8'
  && Reflect.construct(Target, control).args.join(',') === '7,8');
var buffer = new ArrayBuffer(4, { maxByteLength: 8 });
var view = new Uint8Array(buffer);
view[0] = 7;
var reads = 0;
Object.defineProperty(view, 'length', { get() { reads++; buffer.resize(1); return 4; } });
var args = collect.apply(null, view);
print(reads === 1 && args.length === 4 && args[0] === 7 && args[1] === undefined && args[3] === undefined);
void 0;
"#;
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(60_000),
                ..RunOptions::default()
            },
        )
        .expect("resizable typed array argument lists execute through Wasm AOT");
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        outcome.completion
    );
    assert_eq!(
        outcome.output_events,
        ["all views preserved", "true", "true"]
            .map(|line| HostOutputEvent::PrintLine(line.to_string()))
            .to_vec()
    );
}

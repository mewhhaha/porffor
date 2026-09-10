use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_wasm_true(source: &str) {
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("buffer allocation regression must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn transfers_allocate_in_the_same_memory_as_buffer_accesses() {
    assert_wasm_true(
        r#"
var ok = true;
for (var method of ["transfer", "transferToFixedLength", "transferToImmutable"]) {
  var source = new ArrayBuffer(4, { maxByteLength: 16 });
  new Uint8Array(source).set([3, 5, 7, 9]);
  var target = source[method](6);
  var bytes = new Uint8Array(target);
  ok = ok && source.detached && target.byteLength === 6 &&
    bytes[0] === 3 && bytes[3] === 9 && bytes[4] === 0 && bytes[5] === 0;
  if (method === "transfer") {
    target.resize(16);
    bytes[15] = 42;
    ok = ok && bytes[15] === 42 && bytes[6] === 0;
  }
}
ok;
"#,
    );
}

#[test]
fn slices_preserve_bytes_and_independent_backing_stores() {
    assert_wasm_true(
        r#"
var source = new ArrayBuffer(4);
new Uint8Array(source).set([3, 5, 7, 9]);
var copied = source.slice(1, 3);
var immutable = source.sliceToImmutable(1, 3);
new Uint8Array(source)[1] = 42;
new Uint8Array(copied)[0] = 17;
new Uint8Array(copied)[0] === 17 && new Uint8Array(copied)[1] === 7 &&
new Uint8Array(immutable)[0] === 5 && new Uint8Array(immutable)[1] === 7 &&
immutable.immutable && !source.detached;
"#,
    );
}

#[test]
fn zero_length_transfers_keep_a_live_buffer() {
    assert_wasm_true(
        r#"
var source = new ArrayBuffer(0, { maxByteLength: 8 });
var target = source.transfer();
target.resize(8);
var bytes = new Uint8Array(target);
bytes[7] = 19;
source.detached && !target.detached && bytes[0] === 0 && bytes[7] === 19;
"#,
    );
}

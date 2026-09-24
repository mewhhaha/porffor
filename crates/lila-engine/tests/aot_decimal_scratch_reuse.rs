//! The decimal-to-binary64 helper rewinds the bump allocator after using a
//! scratch region; the next allocation must still see zeroed memory.

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

#[test]
fn long_decimal_strings_leave_no_stale_scratch_for_the_next_allocation() {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let source = r#"
var values = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9];
for (var text of ["4503599627370496", "9007199254740992", "123456789012345678901234567890"]) {
  var sliced = values.slice(0, text);
  if (sliced.length !== 10 || sliced[9] !== 9) throw 'slice with ' + text;
  var fresh = values.slice(0, Number(text));
  fresh.push(10);
  if (fresh.length !== 11 || !Object.isExtensible(fresh)) throw 'fresh array ' + text;
  var descriptor = Object.getOwnPropertyDescriptor(fresh, '0');
  if (!descriptor.writable || !descriptor.configurable) throw 'element descriptor ' + text;
}
print(true);
"#;
    let observation = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(60_000),
                ..RunOptions::default()
            },
        )
        .expect("decimal scratch regression must execute through Wasm AOT");
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        observation.completion,
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine("true".to_string())],
    );
}

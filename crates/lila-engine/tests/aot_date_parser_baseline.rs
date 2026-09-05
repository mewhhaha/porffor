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
        .expect("baseline witness must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

// These assert the desired semantics, not the old bugs. All three are expected
// to fail on the unchanged product code at 821fccaa69ebd62a048543a70ed8480dd6a69841.
#[test]
fn non_epoch_utc_display_round_trip() {
    assert_wasm_true("Date.parse(new Date(1000).toUTCString()) === 1000;");
}

#[test]
fn reduced_iso_date_followed_by_time() {
    assert_wasm_true("Date.parse(\"1970T12:34Z\") === 45240000;");
}

#[test]
fn iso_end_of_day_notation() {
    assert_wasm_true("Date.parse(\"1995-02-04T24:00Z\") === 791942400000;");
}

//! A new or caught error reaches `toString` through `%NativeError.prototype%`
//! and `%Error.prototype%`. Only `%Error.prototype%` owns `toString`, and the
//! chain an error is typed with is the one recorded at that program point, so
//! replacing `Error.prototype.toString` is visible through every error kind,
//! whether the error was created before or after the write.
//!
//! The replacement returns a number, so a stale `String` result kind would
//! turn `+ 1` into concatenation.
use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_wasm_string(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
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
        .expect(
            "replaced Error.prototype.toString program must compile and execute through Wasm AOT",
        );
    let completion = format!("string({expected})");
    assert!(
        outcome.note.contains(&completion),
        "expected completion {completion}, got {}",
        outcome.note
    );
}

#[test]
fn replaced_error_prototype_to_string_reaches_new_type_errors_and_errors() {
    assert_wasm_string(
        r#"
Error.prototype.toString = function () { return 8; };
let s = TypeError("y").toString();
let e = Error("x");
let r = e.toString();
typeof s + "|" + (s + 1) + "|" + typeof r + "|" + (r + 1);
"#,
        "number|9|number|9",
    );
}

#[test]
fn replaced_error_prototype_to_string_reaches_errors_created_before_the_write() {
    assert_wasm_string(
        r#"
let early = new RangeError("r");
let plain = new Error("p");
Error.prototype.toString = function () { return 8; };
let s = early.toString();
let t = plain.toString();
typeof s + "|" + (s + 1) + "|" + (t + 1);
"#,
        "number|9|9",
    );
}

#[test]
fn replaced_error_prototype_to_string_reaches_a_caught_runtime_error() {
    assert_wasm_string(
        r#"
Error.prototype.toString = function () { return 8; };
let r;
try { null.x; } catch (e) { r = e.toString(); }
typeof r + "|" + (r + 1);
"#,
        "number|9",
    );
}

//! Error objects are typed with the prototype chain recorded at their
//! creation point, not with a fresh-realm snapshot, and a NativeError
//! prototype does not own `toString`. So `toString` on a new, early or caught
//! error keeps its `String` result only while `%Error.prototype%.toString` is
//! provably the builtin.
use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ScriptIr, ValueKind};

const WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;

fn lower_script(source: &'static str) -> ScriptIr {
    std::thread::Builder::new()
        .stack_size(WORKER_STACK_BYTES)
        .spawn(move || {
            let parsed = parse(source, ParseOptions::script()).expect("script should parse");
            let program = lower(&parsed);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            program.script.expect("script IR")
        })
        .expect("worker thread should spawn")
        .join()
        .expect("lowering should not panic")
}

#[test]
fn unmodified_error_to_string_keeps_its_string_result() {
    let script = lower_script("TypeError('y').toString();");
    assert_eq!(script.result_kind(), ValueKind::String);
}

#[test]
fn replaced_error_to_string_claims_no_string_result() {
    for source in [
        "Error.prototype.toString = function () { return 8; };\nTypeError('y').toString();",
        "let e = new RangeError('r');\n\
         Error.prototype.toString = function () { return 8; };\ne.toString();",
        "Error.prototype.toString = function () { return 8; };\n\
         let r; try { null.x; } catch (e) { r = e.toString(); } r;",
    ] {
        let script = lower_script(source);
        assert_ne!(script.result_kind(), ValueKind::String, "{source}");
    }
}

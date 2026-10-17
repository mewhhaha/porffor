use super::*;
use lila_front::{parse, ParseOptions};
use lila_ir::lower;
use wasmparser::{Validator, WasmFeatures};

#[test]
fn large_repeated_read_body_retains_bounded_valid_wasm_code() {
    std::thread::Builder::new()
        .name("repeated-identifier-read-body".into())
        .stack_size(64 * 1024 * 1024)
        .spawn(|| {
            let source = format!(
                "function readMany() {{ if (x) ; else if (y) n(); else {{ {} }} }} readMany;",
                "e;".repeat(10_000)
            );
            let parsed = parse(&source, ParseOptions::script()).expect("large source parses");
            let ir = lower(&parsed);
            assert!(ir.is_wasm_supported(), "{:?}", ir.diagnostics);
            let artifact = emit(&ir).expect("live reads compile to real Wasm");
            let features = WasmFeatures::default()
                | WasmFeatures::THREADS
                | WasmFeatures::FUNCTION_REFERENCES
                | WasmFeatures::GC
                | WasmFeatures::EXCEPTIONS;
            Validator::new_with_features(features)
                .validate_all(&artifact.bytes)
                .expect("compact repeated reads have valid Wasm control flow");
            let body = artifact
                .function_sizes
                .iter()
                .find(|body| body.name.starts_with("js::readMany#"))
                .expect("actual source function is attributed")
                .body_bytes
                .bytes();
            assert!(body < 32 * 1024, "10,000 live reads emit {body} body bytes");
        })
        .expect("bounded compiler worker starts")
        .join()
        .expect("large body emission succeeds");
}

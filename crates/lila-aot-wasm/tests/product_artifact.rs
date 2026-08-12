//! Product-artifact boundary for T27.
//!
//! Lila may retain source metadata for diagnostics, but emitted Wasm must not
//! carry user source as input to an evaluator. This test uses a comment as an
//! execution-irrelevant source marker, then independently inspects the binary
//! imports and operators.

use lila_aot_wasm::emit;
use lila_front::{parse, ParseOptions};
use lila_ir::lower;
use wasmparser::{Operator, Parser, Payload, TypeRef};

const WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;
const SOURCE_MARKER: &str = "LILA_SOURCE_MUST_NOT_FEED_A_RUNTIME_EVALUATOR_8A8D20E9";
const VALUE_MARKER: f64 = 424_242.0;

fn emit_representative_program() -> Vec<u8> {
    std::thread::Builder::new()
        .stack_size(WORKER_STACK_BYTES)
        .spawn(|| {
            let source = format!(
                "/* {SOURCE_MARKER} */ function answer() {{ return {VALUE_MARKER}; }} print(answer());"
            );
            let parsed = parse(&source, ParseOptions::script()).expect("fixture should parse");
            emit(&lower(&parsed))
                .expect("fixture should compile directly to Wasm")
                .bytes
        })
        .expect("compiler worker should spawn")
        .join()
        .expect("compiler worker should not panic")
}

#[test]
fn product_wasm_contains_compiled_semantics_without_a_source_evaluator() {
    let bytes = emit_representative_program();
    assert!(
        !bytes
            .windows(SOURCE_MARKER.len())
            .any(|window| window == SOURCE_MARKER.as_bytes()),
        "execution-irrelevant user source must not be embedded in the artifact"
    );

    let mut saw_compiled_value = false;
    let mut code_bodies = 0usize;
    for payload in Parser::new(0).parse_all(&bytes) {
        match payload.expect("emitted Wasm should parse") {
            Payload::ImportSection(reader) => {
                for import in reader.into_imports() {
                    let import = import.expect("import should decode");
                    let boundary =
                        format!("{}.{}", import.module, import.name).to_ascii_lowercase();
                    for forbidden in [
                        "eval",
                        "interpreter",
                        "javascript",
                        "parse_source",
                        "run_source",
                    ] {
                        assert!(
                            !boundary.contains(forbidden),
                            "product artifact imports forbidden evaluator boundary {boundary}"
                        );
                    }
                    if matches!(import.ty, TypeRef::Func(_) | TypeRef::FuncExact(_)) {
                        assert_eq!(
                            import.module, "lila_host",
                            "product function imports must cross the typed Lila host ABI"
                        );
                    }
                }
            }
            Payload::CodeSectionEntry(body) => {
                code_bodies += 1;
                for operator in body
                    .get_operators_reader()
                    .expect("operator reader should construct")
                {
                    match operator.expect("operator should decode") {
                        Operator::F64Const { value } if value.bits() == VALUE_MARKER.to_bits() => {
                            saw_compiled_value = true;
                        }
                        Operator::I64Const { value }
                            if value == VALUE_MARKER.to_bits() as i64
                                || value == VALUE_MARKER as i64 =>
                        {
                            saw_compiled_value = true;
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    assert!(
        code_bodies > 0,
        "product artifact must contain compiled code"
    );
    assert!(
        saw_compiled_value,
        "the user program's numeric semantics must appear in emitted instructions"
    );
}

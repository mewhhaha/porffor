//! Product-artifact boundary for T27.
//!
//! Lila may retain source metadata for diagnostics, but emitted Wasm must not
//! carry user source as input to an evaluator. Validate the binary's types as
//! well as its imports and operators: successful decoding alone is not proof
//! that the emitted program is a valid WebAssembly module.

use lila_aot_wasm::emit;
use lila_front::{parse, ParseOptions};
use lila_ir::lower;
use wasmparser::{Operator, Parser, Payload, TypeRef, Validator, WasmFeatures};

const WORKER_STACK_BYTES: usize = 64 * 1024 * 1024;
const SOURCE_MARKER: &str = "LILA_SOURCE_MUST_NOT_FEED_A_RUNTIME_EVALUATOR_8A8D20E9";
const VALUE_MARKER: f64 = 424_242.0;

fn emit_program(name: &'static str, source: String) -> Vec<u8> {
    std::thread::Builder::new()
        .name(format!("product-artifact-{name}"))
        .stack_size(WORKER_STACK_BYTES)
        .spawn(move || {
            let parsed = parse(&source, ParseOptions::script()).expect("fixture should parse");
            emit(&lower(&parsed))
                .expect("fixture should compile directly to Wasm")
                .bytes
        })
        .expect("compiler worker should spawn")
        .join()
        .expect("compiler worker should not panic")
}

fn product_validator() -> Validator {
    let mut features = WasmFeatures::default();
    // The product deliberately targets these Wasm capabilities. Do not use
    // all(): that would silently allow unrelated experimental proposals.
    for feature in [
        WasmFeatures::THREADS,
        WasmFeatures::MULTI_MEMORY,
        WasmFeatures::REFERENCE_TYPES,
        WasmFeatures::FUNCTION_REFERENCES,
        WasmFeatures::GC,
        WasmFeatures::EXCEPTIONS,
        WasmFeatures::TAIL_CALL,
    ] {
        features.set(feature, true);
    }
    Validator::new_with_features(features)
}

fn assert_product_artifact(name: &str, bytes: &[u8], expected_value: Option<f64>) {
    product_validator()
        .validate_all(bytes)
        .unwrap_or_else(|error| panic!("{name}: emitted Wasm failed validation: {error}"));
    assert!(
        !bytes
            .windows(SOURCE_MARKER.len())
            .any(|window| window == SOURCE_MARKER.as_bytes()),
        "{name}: execution-irrelevant user source must not be embedded in the artifact"
    );

    let mut saw_compiled_value = false;
    let mut code_bodies = 0usize;
    for payload in Parser::new(0).parse_all(bytes) {
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
                            "{name}: product imports forbidden evaluator boundary {boundary}"
                        );
                    }
                    if matches!(import.ty, TypeRef::Func(_) | TypeRef::FuncExact(_)) {
                        assert_eq!(
                            import.module, "lila_host",
                            "{name}: function imports must cross the typed Lila host ABI"
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
                        Operator::F64Const { value } => {
                            if let Some(expected) = expected_value {
                                saw_compiled_value |= value.bits() == expected.to_bits();
                            }
                        }
                        Operator::I64Const { value } => {
                            if let Some(expected) = expected_value {
                                saw_compiled_value |= value == expected.to_bits() as i64
                                    || value == expected as i64;
                            }
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
        "{name}: product artifact must contain compiled code"
    );
    if expected_value.is_some() {
        assert!(
            saw_compiled_value,
            "{name}: the user program's numeric semantics must appear in emitted instructions"
        );
    }
}

#[test]
fn product_wasm_contains_compiled_semantics_without_a_source_evaluator() {
    let source = format!(
        "/* {SOURCE_MARKER} */ function answer() {{ return {VALUE_MARKER}; }} print(answer());"
    );
    let bytes = emit_program("numeric-marker", source);
    assert_product_artifact("numeric-marker", &bytes, Some(VALUE_MARKER));
}

#[test]
fn representative_language_families_produce_valid_aot_artifacts() {
    for (name, source) in [
        (
            "loop-branches",
            "let total = 0; for (let i = 0; i < 10; i++) { if (i === 3) continue; if (i === 8) break; total += i; } print(total);",
        ),
        (
            "closure-capture",
            "function makeAdder(x) { return function(y) { return x + y; }; } const add = makeAdder(4); print(add(5));",
        ),
        (
            "abrupt-completion",
            "function checked() { try { throw 7; } catch (value) { return value + 1; } finally { print('done'); } } print(checked());",
        ),
        (
            "heap-aggregates",
            "const values = [1, 2, 3]; const record = { items: values }; print(record.items[1]);",
        ),
        (
            "bigint-and-strings",
            "const value = 12345678901234567890n + 1n; print(String(value)); print('x😀'.length);",
        ),
    ] {
        let bytes = emit_program(name, source.to_owned());
        assert_product_artifact(name, &bytes, None);
    }
}

#[test]
fn validator_rejects_a_parseable_function_result_type_mismatch() {
    // (module (func (result i32) i32.const 7))
    let valid = [
        0, 97, 115, 109, 1, 0, 0, 0, 1, 5, 1, 96, 0, 1, 127, 3, 2, 1, 0, 10, 6, 1, 4, 0, 65, 7, 11,
    ];
    assert!(product_validator().validate_all(&valid).is_ok());
    let mut invalid = valid;
    assert_eq!(invalid[24], 0x41);
    invalid[24] = 0x42; // i64.const 7, while the function still promises i32.

    for payload in Parser::new(0).parse_all(&invalid) {
        if let Payload::CodeSectionEntry(body) = payload.expect("control should remain parseable") {
            for operator in body
                .get_operators_reader()
                .expect("control operators should decode")
            {
                operator.expect("control instruction should remain parseable");
            }
        }
    }
    assert!(
        product_validator().validate_all(&invalid).is_err(),
        "decodable but ill-typed Wasm must not pass the product artifact gate"
    );
}

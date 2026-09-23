use lila_front::{parse, ParseOptions};
use lila_ir::{lower, FunctionIr, StatementIr, ValueKind};

fn lower_sample(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("conversion source parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .expect("script IR")
        .functions
        .into_iter()
        .find(|function| function.name == "sample")
        .expect("sample function")
}

#[test]
fn indexed_receiver_coercion_keeps_bigint_in_the_numeric_result_domain() {
    for source in [
        "function sample() { var value = [1]; value.valueOf = function () { return 7n; }; return (value - 1n) + 1n; }",
        "function sample() { arguments.valueOf = function () { return 7n; }; return (arguments - 1n) + 1n; }",
        "function sample() { var value = [1]; value.valueOf = function () { return 7n; }; return (~value) + 1n; }",
    ] {
        let function = lower_sample(source);
        let StatementIr::Return(result) = function.body.statements.last().expect("return") else {
            panic!("final return");
        };
        assert!(result.possible_kinds.contains(ValueKind::BigInt), "{source}");
        assert!(!result.possible_kinds.contains(ValueKind::String), "{source}");
    }
}

#[test]
fn addition_does_not_narrow_mutable_array_or_arguments_hooks_to_strings() {
    for source in [
        "function sample() { var value = [1]; value.valueOf = function () { return 7; }; return value + 1; }",
        "function sample() { arguments.valueOf = function () { return 7; }; return arguments + 1; }",
        "function sample() { var value = [1]; value.join = function () { return 7n; }; return value + 1n; }",
    ] {
        let function = lower_sample(source);
        let StatementIr::Return(result) = function.body.statements.last().expect("return") else {
            panic!("final return");
        };
        assert!(result.possible_kinds.contains(ValueKind::Number), "{source}");
        assert!(result.possible_kinds.contains(ValueKind::BigInt), "{source}");
    }
}

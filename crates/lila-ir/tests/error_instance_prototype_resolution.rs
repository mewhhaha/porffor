//! Error construction can invalidate prototype facts. Calls still retain the
//! real receiver and inherited method read; a possible native target cannot
//! license a String result when the live descriptor is no longer proved.
use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ExprIr, PropertyKeyIr, ScriptIr, SpecOperationIr, StandardBuiltinId, StatementIr,
    ValueKind,
};

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
fn unmodified_error_to_string_keeps_its_actual_receiver_read_and_native_candidate() {
    let script = lower_script("TypeError('y').toString();");
    let Some(StatementIr::Expression(result)) = script.body.statements.last() else {
        panic!("the toString call remains the result");
    };
    assert!(result.possible_kinds.contains(ValueKind::String));
    let ExprIr::MaterializeBinding { name, value, body } = &result.expr else {
        panic!("the Error receiver must be acquired once: {result:?}");
    };
    let ExprIr::CallIndirect {
        args: construction_args,
        ..
    } = &value.expr
    else {
        panic!("the original TypeError invocation remains: {value:?}");
    };
    assert_eq!(construction_args.len(), 1);
    assert!(matches!(&construction_args[0].expr, ExprIr::String(message) if message == "y"));
    let ExprIr::CallIndirect {
        callee,
        this_arg: Some(receiver),
        args,
        ..
    } = &body.expr
    else {
        panic!("the acquired method is called: {body:?}");
    };
    assert!(args.is_empty());
    assert!(matches!(&receiver.expr, ExprIr::Identifier(storage) if storage == name));
    assert!(callee
        .function_targets
        .known_targets()
        .contains(&StandardBuiltinId::ErrorPrototypeToString.function_id()));
    let target = match &callee.expr {
        ExprIr::PropertyRead {
            target,
            key: PropertyKeyIr::StaticString(key),
        } => {
            assert_eq!(key, "toString");
            target.as_ref()
        }
        ExprIr::SpecOperation {
            operation: SpecOperationIr::GetV,
            operands,
        } => {
            assert_eq!(operands.len(), 2);
            assert!(matches!(&operands[1].expr, ExprIr::String(key) if key == "toString"));
            &operands[0]
        }
        _ => panic!("toString must retain its live inherited Get: {callee:?}"),
    };
    assert!(matches!(&target.expr, ExprIr::Identifier(storage) if storage == name));
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

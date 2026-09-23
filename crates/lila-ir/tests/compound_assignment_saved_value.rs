use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ExprIr, FunctionIr, KindSet, StatementIr, TypedExpr, ValueKind};

fn lower_sample(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("saved-value fixture parses");
    let lowered = lower(&parsed);
    assert!(lowered.is_wasm_supported(), "{:?}", lowered.diagnostics);
    lowered
        .script
        .expect("script IR")
        .functions
        .into_iter()
        .find(|function| function.name == "sample")
        .expect("sample function")
}

fn assignment(function: &FunctionIr) -> &TypedExpr {
    function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Expression(expression) => Some(expression),
            _ => None,
        })
        .expect("compound assignment")
}

#[test]
fn primitive_assignment_results_follow_the_saved_left_value() {
    for (source, kind) in [
        (
            "function sample(){let value=1; value += (value='changed',2); return value+1;}",
            ValueKind::Number,
        ),
        (
            "function sample(){let value='x'; value += (value=10,2); return value+1;}",
            ValueKind::String,
        ),
        (
            "function sample(){let value=3n; value -= (value='changed',1n); return value+1n;}",
            ValueKind::BigInt,
        ),
    ] {
        let function = lower_sample(source);
        assert_eq!(
            assignment(&function).possible_kinds,
            KindSet::from_kind(kind)
        );
        let StatementIr::Return(result) = function.body.statements.last().expect("return") else {
            panic!("final return");
        };
        assert_eq!(result.possible_kinds, KindSet::from_kind(kind));
    }
}

#[test]
fn bigint_addition_retains_the_original_operand_domain() {
    let function = lower_sample(
        "function sample(){let value=1n; value += (value='changed',2n); return value+1n;}",
    );
    let ExprIr::AssignIdentifier { value, .. } = &assignment(&function).expr else {
        panic!("binding write")
    };
    let ExprIr::CoerciveAdd { lhs, rhs } = &value.expr else {
        panic!("ordered addition")
    };
    assert_eq!(lhs.possible_kinds, KindSet::from_kind(ValueKind::BigInt));
    assert_eq!(rhs.possible_kinds, KindSet::from_kind(ValueKind::BigInt));
    assert!(matches!(lhs.expr, ExprIr::Identifier(_)));
}

#[test]
fn mutable_conversion_hooks_do_not_survive_as_saved_shape_proofs() {
    for operator in ["-=", "&="] {
        let function=lower_sample(&format!(
            "function sample(){{let value={{valueOf(){{return 1;}}}}; let original=value; value {operator} (original.valueOf=()=>7n,value='replaced',1n); return value;}}"
        ));
        let ExprIr::AssignIdentifier { value, .. } = &assignment(&function).expr else {
            panic!("binding write")
        };
        let lhs = match &value.expr {
            ExprIr::CoerciveBinaryNumber { lhs, .. } | ExprIr::BitwiseNumeric { lhs, .. } => lhs,
            _ => panic!("ordered numeric operation"),
        };
        assert_eq!(lhs.kind, ValueKind::Dynamic);
        assert!(lhs.heap_shape.is_none());
        assert_eq!(lhs.possible_kinds, KindSet::all_runtime_tags());
        assert!(value.possible_kinds.contains(ValueKind::BigInt));
    }
}

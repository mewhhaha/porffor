#[test]
fn source_identity_unbound_bpe_uses_ordinary_identifier_resolution() {
    let program = lower_script("BPE; missingName; typeof BPE;");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR");
    for (statement, expected) in script.body.statements.iter().zip(["BPE", "missingName"]) {
        assert!(
            matches!(statement, StatementIr::Expression(TypedExpr {
                expr: ExprIr::GlobalIdentifierRead { name }, ..
            }) if name == expected),
            "unbound {expected} must retain runtime ResolveBinding: {statement:?}"
        );
    }
    assert!(matches!(
        script.body.statements.last(),
        Some(StatementIr::Expression(TypedExpr {
            expr: ExprIr::TypeOfUnresolvedIdentifier { .. },
            ..
        }))
    ));
}

#[test]
fn source_identity_named_numeric_receivers_retain_exponentiation_operands() {
    let program =
        lower_script("function power(Number, Math) { return Number.EPSILON ** Math.PI; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR");
    let function = script
        .functions
        .iter()
        .find(|function| function.name == "power")
        .expect("power body");
    let result = function_return(function).expect("power return");
    let (lhs, rhs) = match &result.expr {
        ExprIr::BinaryNumber {
            op: ArithmeticBinaryOp::Exp,
            lhs,
            rhs,
        }
        | ExprIr::CoerciveBinaryNumber {
            op: ArithmeticBinaryOp::Exp,
            lhs,
            rhs,
        } => (lhs, rhs),
        other => panic!("receiver names cannot replace runtime operands: {other:?}"),
    };
    assert!(!matches!(lhs.expr, ExprIr::Number(_)), "{lhs:?}");
    assert!(!matches!(rhs.expr, ExprIr::Number(_)), "{rhs:?}");
}

#[test]
fn source_identity_generator_marker_properties_keep_actual_method_calls() {
    for method in ["next", "return", "throw"] {
        let program = lower_script(&format!(
            "var receiver = {{ \
                $LilaYieldStarGenerator: true, \
                $LilaYieldStarReturnNonObject: true, \
                $LilaYieldStarThrowNonObject: true, \
                {method}(value) {{ return value; }} \
            }}; receiver.{method}(7);"
        ));
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR");
        let Some(StatementIr::Expression(expression)) = script.body.statements.last() else {
            panic!("expected {method} expression");
        };
        assert!(
            indirect_call_body(expression).is_some(),
            "ordinary properties cannot authorize synthetic generator behavior: {expression:?}"
        );
    }
}

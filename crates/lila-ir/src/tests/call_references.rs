#[test]
fn lowers_object_keys_join_before_control_as_runtime_property_call() {
    let program = lower_script(r#"var o = { a: 1 }; Object.keys(o).join(""); if (false) {} 1;"#);
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    assert!(script.body.statements.iter().any(|statement| {
        let StatementIr::Expression(expression) = statement else {
            return false;
        };
        matches!(
            indirect_call_body(expression).map(|call| &call.expr),
            Some(ExprIr::CallIndirect { callee, .. }) if matches!(
                &callee.expr,
                ExprIr::SpecOperation {
                    operation: SpecOperationIr::GetV,
                    operands,
                } if operands.len() == 2
                    && matches!(&operands[1].expr, ExprIr::String(name) if name == "join")
            )
        )
    }));
}

#[test]
fn lowers_mutable_array_prototype_method_call_through_runtime_getv() {
    let program = lower_script(
            "var alias = Array.prototype; alias.join = function () { return 'alias'; }; Array.prototype.join();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    assert!(script.body.statements.iter().any(|statement| {
        let StatementIr::Expression(expression) = statement else {
            return false;
        };
        matches!(
            indirect_call_body(expression).map(|call| &call.expr),
            Some(ExprIr::CallIndirect { callee, .. }) if matches!(
                &callee.expr,
                ExprIr::SpecOperation {
                    operation: SpecOperationIr::GetV,
                    operands,
                } if operands.len() == 2
                    && matches!(&operands[1].expr, ExprIr::String(name) if name == "join")
            )
        )
    }));
}

#[test]
fn lowers_copied_mutable_array_prototype_method_as_indirect_call() {
    let program = lower_script("var obj = {}; obj.join = Array.prototype.join; obj.join();");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    assert!(script.body.statements.iter().any(|statement| {
        let StatementIr::Expression(expression) = statement else {
            return false;
        };
        indirect_call_body(expression)
            .is_some_and(|call| call.possible_kinds == KindSet::all_runtime_tags())
    }));
}

#[test]
fn non_generic_primitive_method_calls_retain_acquired_callee_identity() {
    struct MethodCase {
        owner: &'static str,
        name: &'static str,
        builtin: StandardBuiltinId,
        valid_receiver: &'static str,
        expected_kind: ValueKind,
    }

    let methods = [
        MethodCase {
            owner: "Boolean",
            name: "toString",
            builtin: StandardBuiltinId::BooleanPrototypeToString,
            valid_receiver: "new Boolean()",
            expected_kind: ValueKind::String,
        },
        MethodCase {
            owner: "Boolean",
            name: "valueOf",
            builtin: StandardBuiltinId::BooleanPrototypeValueOf,
            valid_receiver: "new Boolean()",
            expected_kind: ValueKind::Boolean,
        },
        MethodCase {
            owner: "Number",
            name: "toExponential",
            builtin: StandardBuiltinId::NumberPrototypeToExponential,
            valid_receiver: "new Number()",
            expected_kind: ValueKind::String,
        },
        MethodCase {
            owner: "Number",
            name: "toFixed",
            builtin: StandardBuiltinId::NumberPrototypeToFixed,
            valid_receiver: "new Number()",
            expected_kind: ValueKind::String,
        },
        MethodCase {
            owner: "Number",
            name: "toLocaleString",
            builtin: StandardBuiltinId::NumberPrototypeToLocaleString,
            valid_receiver: "new Number()",
            expected_kind: ValueKind::String,
        },
        MethodCase {
            owner: "Number",
            name: "toPrecision",
            builtin: StandardBuiltinId::NumberPrototypeToPrecision,
            valid_receiver: "new Number()",
            expected_kind: ValueKind::String,
        },
        MethodCase {
            owner: "Number",
            name: "toString",
            builtin: StandardBuiltinId::NumberPrototypeToString,
            valid_receiver: "new Number()",
            expected_kind: ValueKind::String,
        },
        MethodCase {
            owner: "Number",
            name: "valueOf",
            builtin: StandardBuiltinId::NumberPrototypeValueOf,
            valid_receiver: "new Number()",
            expected_kind: ValueKind::Number,
        },
        MethodCase {
            owner: "BigInt",
            name: "toString",
            builtin: StandardBuiltinId::BigIntPrototypeToString,
            valid_receiver: "Object(1n)",
            expected_kind: ValueKind::String,
        },
        MethodCase {
            owner: "BigInt",
            name: "toLocaleString",
            builtin: StandardBuiltinId::BigIntPrototypeToLocaleString,
            valid_receiver: "Object(1n)",
            expected_kind: ValueKind::String,
        },
        MethodCase {
            owner: "BigInt",
            name: "valueOf",
            builtin: StandardBuiltinId::BigIntPrototypeValueOf,
            valid_receiver: "Object(1n)",
            expected_kind: ValueKind::BigInt,
        },
        MethodCase {
            owner: "String",
            name: "toString",
            builtin: StandardBuiltinId::StringPrototypeToString,
            valid_receiver: "new String()",
            expected_kind: ValueKind::String,
        },
        MethodCase {
            owner: "String",
            name: "valueOf",
            builtin: StandardBuiltinId::StringPrototypeValueOf,
            valid_receiver: "new String()",
            expected_kind: ValueKind::String,
        },
    ];
    let mut source = String::new();
    let mut expected_calls = Vec::new();
    for method in methods {
        let mut calls = vec![
            (method.valid_receiver, "transferred"),
            ("new Object()", method.name),
            ("new Object()", "transferred"),
        ];
        if method.owner == "Number" {
            calls.push(("new Boolean()", method.name));
        }
        for (receiver, destination) in calls {
            let index = expected_calls.len() + 1;
            source.push_str(&format!(
                    "var value{index} = {receiver}; value{index}.{destination} = {}.prototype.{}; value{index}.{destination}();",
                    method.owner, method.name
                ));
            expected_calls.push((method.builtin, destination, method.expected_kind));
        }
    }
    let program = lower_script(&source);
    assert!(
        program.is_wasm_supported(),
        "expected supported transferred primitive calls: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script ir should exist");
    let calls = script
        .body
        .statements
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::Expression(expression)
                if matches!(
                    expression.expr,
                    ExprIr::MaterializeBinding { ref body, .. }
                        if matches!(body.expr, ExprIr::CallIndirect { .. })
                ) =>
            {
                Some(expression)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(calls.len(), expected_calls.len());
    for (expression, (expected_builtin, expected_key, expected_kind)) in
        calls.into_iter().zip(expected_calls)
    {
        assert_eq!(
            expression.kind,
            ValueKind::Dynamic,
            "transferred {expected_builtin:?} at property {expected_key}"
        );
        let ExprIr::MaterializeBinding {
            name, body: call, ..
        } = &expression.expr
        else {
            unreachable!("the call collection accepts only materialized calls");
        };
        let ExprIr::CallIndirect {
            callee,
            this_arg: Some(this_arg),
            ..
        } = &call.expr
        else {
            panic!("expected callee and this argument: {call:?}");
        };
        assert_eq!(
            call.kind,
            ValueKind::Dynamic,
            "transferred {expected_builtin:?} at property {expected_key}"
        );
        assert!(matches!(
            this_arg.expr,
            ExprIr::Identifier(ref this_name) if this_name == name
        ));
        // The preceding property Put can invoke a prototype setter. A later
        // Get cannot inherit the transferred builtin's result domain.
        assert!(matches!(
            callee.function_targets,
            FunctionTargetKnowledge::Open(_)
        ));
        assert_eq!(
            call.possible_kinds,
            KindSet::all_runtime_tags(),
            "unguarded {expected_kind:?} result"
        );
        assert!(
            match &callee.expr {
                ExprIr::PropertyRead { target, key } => matches!(
                    (&target.expr, key),
                    (
                        ExprIr::Identifier(target_name),
                        PropertyKeyIr::StaticString(key)
                    ) if target_name == name && key.as_str() == expected_key
                ),
                ExprIr::SpecOperation {
                    operation: SpecOperationIr::GetV,
                    operands,
                } =>
                    operands.len() == 2
                        && matches!(operands[0].expr, ExprIr::Identifier(ref target_name) if target_name == name)
                        && matches!(operands[1].expr, ExprIr::String(ref key) if key.as_str() == expected_key),
                _ => false,
            },
            "expected retained property read: {callee:?}"
        );
    }
}

#[test]
fn boolean_method_fold_and_shape_are_invalidated_through_binding_aliases() {
    let program = lower_script(
            "var b = new Boolean(); var alias = b; alias.toString = Number.prototype.toString; b.toString();",
        );
    assert!(
        program.is_wasm_supported(),
        "expected supported aliased Boolean method call: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(expression) =
        script.body.statements.last().expect("call expression")
    else {
        panic!("expected final call expression");
    };
    let ExprIr::MaterializeBinding {
        name,
        value,
        body: call,
    } = &expression.expr
    else {
        panic!("expected acquired-callee receiver materialization: {expression:?}");
    };
    assert!(
        matches!(
            value.expr,
            ExprIr::GlobalIdentifierRead { name: ref value_name }
                if value_name == "b"
        ),
        "expected materialized source receiver b: {value:?}"
    );
    let ExprIr::CallIndirect {
        callee,
        this_arg: Some(this_arg),
        ..
    } = &call.expr
    else {
        panic!("expected acquired-callee indirect call: {call:?}");
    };
    assert!(callee.function_targets.exact_targets().is_none());
    assert!(matches!(
        this_arg.expr,
        ExprIr::Identifier(ref this_name) if this_name == name
    ));
    assert!(
        match &callee.expr {
            ExprIr::PropertyRead { target, key } => {
                matches!(target.expr, ExprIr::Identifier(ref target_name) if target_name == name)
                    && matches!(key, PropertyKeyIr::StaticString(key) if key == "toString")
            }
            ExprIr::SpecOperation {
                operation: SpecOperationIr::GetV,
                operands,
            } => {
                operands.len() == 2
                    && matches!(operands[0].expr, ExprIr::Identifier(ref target_name) if target_name == name)
                    && matches!(operands[1].expr, ExprIr::String(ref key) if key == "toString")
            }
            _ => false,
        },
        "expected retained runtime property read: {callee:?}"
    );
}

#[test]
fn preserves_call_spreads_in_source_argument_order() {
    let program =
        lower_script("function collect() {} let values = [2, 3]; collect(42, ...[1], ...values,);");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(call) = script.body.statements.last().expect("call expression")
    else {
        panic!("expected call expression");
    };
    let args = match &call.expr {
        ExprIr::CallNamed { args, .. } | ExprIr::CallIndirect { args, .. } => args,
        other => panic!("expected function call, got {other:?}"),
    };

    assert_eq!(args.len(), 3);
    assert!(matches!(args[0].expr, ExprIr::Number(_)));
    assert!(matches!(
        args[1].expr,
        ExprIr::SpreadArgument(ref spread)
            if matches!(spread.value.expr, ExprIr::ArrayLiteral(_))
    ));
    assert!(matches!(
        args[2].expr,
        ExprIr::SpreadArgument(ref spread)
            if matches!(spread.value.expr, ExprIr::Identifier(ref name) if name == "values")
    ));
}

#[test]
fn lowers_computed_array_subclass_method_through_runtime_getv() {
    let program = lower_script(
            "class Derived extends Array { push() { return 'derived'; } } var key = 'push'; new Derived()[key]();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let Some(StatementIr::Expression(final_expr)) = script.body.statements.last() else {
        panic!("expected final expression statement");
    };
    let call = indirect_call_body(final_expr).expect("expected materialized indirect call");
    let ExprIr::CallIndirect { callee, .. } = &call.expr else {
        panic!("expected indirect call, got {:?}", final_expr.expr);
    };
    let ExprIr::SpecOperation {
        operation: SpecOperationIr::GetV,
        operands,
    } = &callee.expr
    else {
        panic!(
            "expected GetV callee, got {:?} with targets {:?}",
            callee.expr, callee.function_targets
        );
    };
    assert_eq!(operands.len(), 2);
    assert!(
        matches!(
            &operands[1].expr,
            ExprIr::String(name) if name == "push"
        ) || matches!(
            &operands[1].expr,
            ExprIr::GlobalIdentifierRead { name } if name == "key"
        ),
        "unexpected key operand: {:?}",
        operands[1]
    );
}

#[test]
fn lowers_coercion_core_ir() {
    let program = lower_script("1 == \"1\"; \"2\" - 1; \"10\" > \"2\"; void 1; (1, 2);");
    assert!(program.is_wasm_supported());
    let summary = program.ir_summary();
    assert!(summary.contains("spec_operations=1"));
    assert!(summary.contains("coercive_numeric_ops=1"));
    assert!(summary.contains("coercive_relational_ops=1"));
    assert!(summary.contains("void_uses=1"));
    assert!(summary.contains("comma_ops=1"));
}

#[test]
fn lowers_object_function_property_call_arg_coercion_ir() {
    let program =
        lower_script(r#"var h = { get: function(_, key) { return key * 10; } }; h.get({}, "1");"#);
    assert!(program.is_wasm_supported());
    let summary = program.ir_summary();
    assert!(summary.contains("coercive_numeric_ops=2"));
}

#[test]
fn lowers_object_seal_with_argument_result_shape() {
    let program = lower_script("var target = { value: 1 }; Object.seal(target);");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(expression) = &script.body.statements[1] else {
        panic!("expected Object.seal expression");
    };
    assert_eq!(expression.kind, ValueKind::Object);
    let function_id = StandardBuiltinId::ObjectSeal.function_id();
    let call = indirect_call_body(expression)
        .unwrap_or_else(|| panic!("expected Object.seal call: {expression:?}"));
    let ExprIr::CallIndirect { callee, .. } = &call.expr else {
        unreachable!("indirect_call_body only returns indirect calls");
    };
    assert_eq!(
        callee.function_targets.exact_single_target(),
        Some(&function_id)
    );
}

#[test]
fn lowers_heap_loose_equality_ir() {
    let program = lower_script("let object = {}; object == undefined; null != object;");
    assert!(program.is_wasm_supported());
    let summary = program.ir_summary();
    assert!(summary.contains("spec_operations=2"));
}

#[test]
fn lowers_typeof_unresolved_identifier() {
    let program = lower_script("typeof missingName;");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(expr) = &script.body.statements[0] else {
        panic!("expected expression statement");
    };
    assert!(matches!(
        expr.expr,
        ExprIr::TypeOfUnresolvedIdentifier { .. }
    ));
    assert_eq!(expr.kind, ValueKind::String);
}

#[test]
fn lowers_builtin_typeof_through_runtime_globals_after_arbitrary_effects() {
    // An exact empty function has no arbitrary effects. Keep this witness
    // open so its call can delete or replace any configurable global binding.
    let program = lower_script("unknownHook(); typeof Number; typeof Symbol; typeof BigInt;");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let runtime_global_names = script
        .body
        .statements
        .iter()
        .filter_map(|statement| {
            let StatementIr::Expression(TypedExpr {
                expr: ExprIr::TypeOfUnresolvedIdentifier { name },
                ..
            }) = statement
            else {
                return None;
            };
            Some(name.as_str())
        })
        .collect::<Vec<_>>();

    assert_eq!(runtime_global_names, ["Number", "Symbol", "BigInt"]);
}

#[test]
fn an_effect_free_exact_source_call_keeps_untracked_typeof_unresolved() {
    let program = lower_script("function observe() {} observe(); typeof createdAfterCall;");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let typeof_expression = script
        .body
        .statements
        .iter()
        .find_map(|statement| {
            let StatementIr::Expression(expression) = statement else {
                return None;
            };
            matches!(&expression.expr, ExprIr::TypeOfUnresolvedIdentifier { .. })
                .then_some(expression)
        })
        .expect("untracked identifier should retain unresolved typeof lowering");

    assert_eq!(typeof_expression.kind, ValueKind::String);
}

#[test]
fn precise_definition_and_deletion_make_untracked_global_typeof_runtime_reads() {
    let program = lower_script(
        r#"Object.defineProperty(globalThis, "accessorAfterCall", { get: function() { return 1; } });
typeof accessorAfterCall;
delete globalThis.accessorAfterCall;
typeof accessorAfterCall;"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let runtime_global_names = script
        .body
        .statements
        .iter()
        .filter_map(|statement| {
            let StatementIr::Expression(TypedExpr {
                expr: ExprIr::TypeOfUnresolvedIdentifier { name },
                ..
            }) = statement
            else {
                return None;
            };
            Some(name.as_str())
        })
        .collect::<Vec<_>>();

    assert_eq!(
        runtime_global_names,
        ["accessorAfterCall", "accessorAfterCall"]
    );
}

#[test]
fn lowers_conditionally_deleted_global_typeof_through_runtime_reference() {
    let program = lower_script(
            "let remove = globalThis.removeNumber; if (remove) delete globalThis.Number; typeof Number;",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let runtime_global_name = script.body.statements.iter().find_map(|statement| {
        let StatementIr::Expression(TypedExpr {
            expr: ExprIr::TypeOfUnresolvedIdentifier { name },
            ..
        }) = statement
        else {
            return None;
        };
        Some(name.as_str())
    });

    assert_eq!(runtime_global_name, Some("Number"));
}

#[test]
fn lowers_with_typeof_global_created_during_unscopables_resolution() {
    let program = lower_script(
        r#"let globalName = "createdDuringWithTypeof";
with ({
    createdDuringWithTypeof: 0,
    get [Symbol.unscopables]() {
        globalThis[globalName] = 1;
        return { createdDuringWithTypeof: true };
    }
}) {
    typeof createdDuringWithTypeof;
}"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let with_block = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::LexicalBlock(statements) => Some(statements),
            _ => None,
        })
        .expect("with statement should lower through a lexical block");
    let StatementIr::Block(with_scope) = &with_block[1] else {
        panic!("with lexical block should contain its body block");
    };
    assert!(matches!(
        &with_scope.statements[1],
        StatementIr::Expression(TypedExpr {
            expr: ExprIr::Undefined,
            ..
        })
    ));
    let StatementIr::Block(with_body) = &with_scope.statements[2] else {
        panic!("with scope should contain its statement body");
    };
    let expr = with_body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Expression(expr) if matches!(&expr.expr, ExprIr::Conditional { .. }) => {
                Some(expr)
            }
            _ => None,
        })
        .expect("with body should contain its typeof expression");
    let ExprIr::Conditional {
        then_expr,
        else_expr,
        ..
    } = &expr.expr
    else {
        panic!("with lookup should select between its object and global fallback");
    };

    assert_eq!(expr.kind, ValueKind::String);
    assert!(matches!(&then_expr.expr, ExprIr::TypeOf { .. }));
    assert!(matches!(
        &else_expr.expr,
        ExprIr::TypeOfUnresolvedIdentifier { name } if name == "createdDuringWithTypeof"
    ));
}

#[test]
fn lowers_symbol_key_for_through_real_method() {
    // `Symbol.keyFor` now resolves through the real `Symbol` constructor
    // object's own `keyFor` method (backed by a runtime registry) rather
    // than a compile-time fold, so the call dispatches indirectly and its
    // result is typed `String | undefined`.
    let program = lower_script("Symbol.keyFor(Symbol.iterator);");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(expr) = &script.body.statements[0] else {
        panic!("expected expression statement");
    };
    assert!(indirect_call_body(expr).is_some());
    assert!(expr.possible_kinds.contains(ValueKind::String));
    assert!(expr.possible_kinds.contains(ValueKind::Undefined));
}

#[test]
fn lowers_symbol_description_property() {
    let program = lower_script("Symbol.iterator.description.startsWith(\"Symbol.\");");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(expr) = &script.body.statements[0] else {
        panic!("expected expression statement");
    };
    assert_eq!(expr.kind, ValueKind::Boolean);
}

#[test]
fn lowers_ascii_identifier_helper_through_its_actual_regexp_method() {
    let program = lower_script(
        "const ASCII_IDENTIFIER = /^[$_a-zA-Z][$_a-zA-Z0-9]*$/u; ASCII_IDENTIFIER.test(\"next\");",
    );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(expr) = &script.body.statements[1] else {
        panic!("expected expression statement");
    };
    assert_eq!(expr.kind, ValueKind::Boolean);
    assert!(indirect_call_body(expr).is_some());
}

#[test]
fn lowers_temporal_zoned_date_time_from_with_instance_result_shape() {
    let program = lower_script("Temporal.ZonedDateTime.from(\"1970-01-01T00:00Z[UTC]\");");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(expression) = &script.body.statements[0] else {
        panic!("expected Temporal.ZonedDateTime.from expression");
    };
    assert_eq!(expression.kind, ValueKind::Object);
    assert!(expression.heap_shape.is_some());
    let function_id = StandardBuiltinId::TemporalZonedDateTimeFrom.function_id();
    let call = indirect_call_body(expression)
        .unwrap_or_else(|| panic!("expected Temporal.ZonedDateTime.from call: {expression:?}"));
    let ExprIr::CallIndirect { callee, .. } = &call.expr else {
        unreachable!("indirect_call_body only returns indirect calls");
    };
    assert_eq!(
        callee.function_targets.exact_single_target(),
        Some(&function_id)
    );
}

#[test]
fn temporal_zoned_date_time_fixed_offset_accessors_retain_property_reads() {
    let program = lower_script(
        "const value = new Temporal.ZonedDateTime(0n, \"+01:30\"); \
             value.offset; value.offsetNanoseconds;",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(offset) = &script.body.statements[1] else {
        panic!("expected Temporal.ZonedDateTime offset expression");
    };
    let StatementIr::Expression(offset_nanoseconds) = &script.body.statements[2] else {
        panic!("expected Temporal.ZonedDateTime offsetNanoseconds expression");
    };
    for (expression, name, kind) in [
        (offset, "offset", ValueKind::String),
        (offset_nanoseconds, "offsetNanoseconds", ValueKind::Number),
    ] {
        assert!(expression.possible_kinds.contains(kind));
        assert!(
            match &expression.expr {
                ExprIr::PropertyRead {
                    key: PropertyKeyIr::StaticString(key),
                    ..
                } => key == name,
                ExprIr::SpecOperation {
                    operation: SpecOperationIr::GetV,
                    operands,
                } =>
                    operands.len() == 2
                        && matches!(&operands[1].expr, ExprIr::String(key) if key == name),
                _ => false,
            },
            "the accessor must use the live prototype: {expression:?}"
        );
    }
}

#[test]
fn temporal_instant_equals_retains_the_acquired_method_and_constructed_argument() {
    let program = lower_script("new Temporal.Instant(1n).equals(new Temporal.Instant(1n));");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(expression) = &script.body.statements[0] else {
        panic!("expected Temporal.Instant.prototype.equals expression");
    };
    assert!(expression.possible_kinds.contains(ValueKind::Boolean));
    let function_id = StandardBuiltinId::TemporalInstantPrototypeEquals.function_id();
    let ExprIr::MaterializeBinding { value, .. } = &expression.expr else {
        panic!("the original receiver must be evaluated once: {expression:?}");
    };
    assert!(matches!(value.expr, ExprIr::Construct { .. }));
    let (callee, args) = retained_indexed_collection_call(expression, "equals");
    assert!(callee
        .function_targets
        .known_targets()
        .contains(&function_id));
    assert_eq!(args.len(), 1);
    assert!(matches!(args[0].expr, ExprIr::Construct { .. }));
}

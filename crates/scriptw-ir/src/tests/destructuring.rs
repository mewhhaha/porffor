#[test]
fn lowers_simple_destructuring_assignment_patterns() {
    let program =
        lower_script("var x; var base = {}; ([x = 1, base.y = 3] = [2, 4]); ({x = 5} = {x: 6});");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(TypedExpr {
        expr: ExprIr::ArrayDestructure { pattern, .. },
        ..
    }) = &script.body.statements[2]
    else {
        panic!("expected semantic array destructuring assignment");
    };
    assert!(matches!(
        pattern.elements[1],
        ArrayDestructuringElementIr::Target {
            target: DestructuringTargetIr::AssignmentProperty { .. },
            ..
        }
    ));
}

#[test]
fn lowers_simple_destructuring_lexical_bindings() {
    let program = lower_script("const [x = 1] = [2]; const { y = 3 } = { y: 4 }; x + y;");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    assert!(matches!(
        script.body.statements[0],
        StatementIr::DeclarationEvaluation(TypedExpr {
            expr: ExprIr::ArrayDestructure {
                evaluation: ArrayDestructuringEvaluationIr::BindingInitialization,
                ..
            },
            ..
        })
    ));
    assert!(matches!(
        script.body.statements[1],
        StatementIr::LexicalBlock(_)
    ));
}

#[test]
fn lowers_array_var_bindings_with_binding_initialization_evaluation() {
    let program = lower_script("var [value] = [1]; value;");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::LexicalBlock(statements) = &script.body.statements[0] else {
        panic!("expected materialized var destructuring block");
    };
    assert!(matches!(
        statements.get(1),
        Some(StatementIr::DeclarationEvaluation(TypedExpr {
            expr: ExprIr::ArrayDestructure {
                evaluation: ArrayDestructuringEvaluationIr::BindingInitialization,
                ..
            },
            ..
        }))
    ));
}

#[test]
fn lowers_object_destructuring_rhs_once_before_ordered_property_reads() {
    let program = lower_script(
        "let { first: renamed = 10, second, missing, } = source(); renamed + second + missing;",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR");
    let StatementIr::LexicalBlock(statements) = &script.body.statements[0] else {
        panic!("expected RHS materialization and one BindingInitialization");
    };
    let [StatementIr::Lexical {
        name: temporary,
        init,
        ..
    }, initialization] = statements.as_slice()
    else {
        panic!("{statements:?}");
    };
    assert!(matches!(
        init.expr,
        ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
    ));
    let StatementIr::DeclarationEvaluation(TypedExpr {
        expr: ExprIr::ObjectDestructure { value, pattern },
        ..
    }) = initialization
    else {
        panic!("{initialization:?}");
    };
    assert!(matches!(&value.expr, ExprIr::Identifier(name) if name == temporary));
    assert_eq!(pattern.properties.len(), 3);
    for (property, (key, name)) in pattern.properties.iter().zip([
        ("first", "renamed"),
        ("second", "second"),
        ("missing", "missing"),
    ]) {
        assert_eq!(property.key, DestructuringPropertyKeyIr::Static(key.into()));
        assert!(
            matches!(&property.target, DestructuringTargetIr::Binding { mode: BindingMode::Let, name: target } if target == name)
        );
        assert_eq!(property.default.is_some(), name == "renamed");
    }
    assert!(pattern.rest.is_none());
}

#[test]
fn materializes_literal_array_before_pattern_defaults_and_bindings() {
    let program =
        lower_script("let [, selected = fallback()] = [leading(), , trailing()]; selected;");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::DeclarationEvaluation(TypedExpr {
        expr:
            ExprIr::ArrayDestructure {
                value,
                pattern,
                evaluation: ArrayDestructuringEvaluationIr::BindingInitialization,
            },
        ..
    }) = &script.body.statements[0]
    else {
        panic!("expected semantic array destructuring expression");
    };
    let ExprIr::ArrayLiteral(elements) = &value.expr else {
        panic!("expected the complete literal array RHS");
    };
    assert_eq!(elements.len(), 3);
    assert!(matches!(
        elements[0].expr,
        ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
    ));
    assert!(matches!(elements[1].expr, ExprIr::ArrayHole));
    assert!(matches!(
        elements[2].expr,
        ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
    ));
    assert!(matches!(
        pattern.elements[0],
        ArrayDestructuringElementIr::Elision
    ));
    let ArrayDestructuringElementIr::Target {
        target: DestructuringTargetIr::Binding { name, .. },
        default: Some(default),
    } = &pattern.elements[1]
    else {
        panic!("expected selected binding with a default initializer");
    };
    assert_eq!(name, "selected");
    assert!(matches!(
        default.expr,
        ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
    ));
}

#[test]
fn materializes_literal_array_assignment_before_target_writes() {
    let program = lower_script(
        "var selected; var result = ([, selected] = [leading(), chosen(), trailing()]); result;",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Var(declarators) = &script.body.statements[1] else {
        panic!("expected result declaration");
    };
    let init = declarators[0]
        .init
        .as_ref()
        .expect("result should have an initializer");
    let ExprIr::ArrayDestructure {
        value,
        pattern,
        evaluation: ArrayDestructuringEvaluationIr::AssignmentEvaluation,
    } = &init.expr
    else {
        panic!("expected semantic array assignment");
    };
    let ExprIr::ArrayLiteral(elements) = &value.expr else {
        panic!("expected the complete literal array RHS");
    };
    assert_eq!(elements.len(), 3);
    assert!(matches!(
        pattern.elements[0],
        ArrayDestructuringElementIr::Elision
    ));
    assert!(matches!(
        pattern.elements[1],
        ArrayDestructuringElementIr::Target {
            target: DestructuringTargetIr::AssignmentIdentifier(..),
            ..
        }
    ));
}

#[test]
fn lowers_for_of_array_assignment_pattern_through_generic_iterator() {
    let program = lower_script("var x; for ([x] of [[1]]) {}");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::ForOfIterator {
        head:
            ForOfIteratorHeadIr::Assignment {
                async_plan: None,
                protocol,
                ..
            },
        body,
        ..
    } = &script.body.statements[1]
    else {
        panic!("expected generic for-of statement");
    };
    assert_eq!(*protocol, IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL);
    let StatementIr::Block(block) = body.as_ref() else {
        panic!("expected assignment prefix block");
    };
    let StatementIr::DeclarationEvaluation(TypedExpr {
        expr:
            ExprIr::ArrayDestructure {
                value,
                evaluation: ArrayDestructuringEvaluationIr::AssignmentEvaluation,
                ..
            },
        ..
    }) = &block.statements[0]
    else {
        panic!("expected semantic array assignment");
    };
    assert_eq!(value.kind, ValueKind::Dynamic);
    assert_eq!(value.possible_kinds, KindSet::all_runtime_tags());
}

#[test]
fn lowers_for_of_array_lexical_pattern_through_generic_iterator() {
    let program = lower_script("for (let [value] of [[1]]) {}");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::ForOfIterator {
        head: ForOfIteratorHeadIr::Assignment {
            async_plan: None, ..
        },
        body,
        ..
    } = &script.body.statements[0]
    else {
        panic!("expected generic for-of statement");
    };
    let StatementIr::Block(block) = body.as_ref() else {
        panic!("expected lexical binding prefix block");
    };
    assert!(matches!(
        block.statements.first(),
        Some(StatementIr::DeclarationEvaluation(TypedExpr {
            expr: ExprIr::ArrayDestructure {
                evaluation: ArrayDestructuringEvaluationIr::BindingInitialization,
                ..
            },
            ..
        }))
    ));
}

#[test]
fn lowers_synchronous_string_for_of_through_generic_iterator_with_dynamic_values() {
    let program = lower_script("for (const value of \"ab\") { value; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::ForOfIterator {
        head:
            ForOfIteratorHeadIr::Assignment {
                async_plan: None,
                protocol,
                ..
            },
        iterable,
        body,
        ..
    } = &script.body.statements[0]
    else {
        panic!("expected generic for-of statement");
    };
    assert_eq!(*protocol, IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL);
    assert_eq!(iterable.kind, ValueKind::String);

    let StatementIr::Block(block) = body.as_ref() else {
        panic!("expected loop body block");
    };
    let StatementIr::Expression(value) = &block.statements[0] else {
        panic!("expected loop binding read");
    };
    assert_eq!(value.kind, ValueKind::Dynamic);
    assert_eq!(value.possible_kinds, KindSet::all_runtime_tags());
}

#[test]
fn lowers_private_loop_heads_as_per_iteration_writes() {
    let program = lower_script(
        "class C {
                #value;
                assign() {
                    for (this.#value of [1, 2]) {}
                    for (this.#value in { first: 1, second: 2 }) {}
                }
            }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let function = script
        .functions
        .iter()
        .find(|function| function.name == "C.assign")
        .expect("assign method should be lowered");

    let mut private_loop_count = 0;
    for statement in &function.body.statements {
        let body = match statement {
            StatementIr::ForOfIterator { body, .. } | StatementIr::ForInObject { body, .. } => body,
            _ => continue,
        };
        private_loop_count += 1;
        let StatementIr::Block(block) = body.as_ref() else {
            panic!("private loop head should add an iteration prefix");
        };
        assert!(matches!(
            block.statements.first(),
            Some(StatementIr::DeclarationEvaluation(TypedExpr {
                expr: ExprIr::PrivateWrite { .. },
                ..
            }))
        ));
    }
    assert_eq!(private_loop_count, 2);
}

#[test]
fn preserves_private_array_assignment_targets_in_destructuring_ir() {
    let program = lower_script(
        "class C {
                #value;
                assign() { [this.#value, ...this.#value] = [1, 2, 3]; }
            }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let function = script
        .functions
        .iter()
        .find(|function| function.name == "C.assign")
        .expect("assign method should be lowered");
    let pattern = function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::ArrayDestructure { pattern, .. },
                ..
            }) => Some(pattern),
            _ => None,
        })
        .expect("array assignment should be lowered");

    assert!(matches!(
        pattern.elements.as_slice(),
        [
            ArrayDestructuringElementIr::Target {
                target: DestructuringTargetIr::AssignmentPrivate { .. },
                ..
            },
            ArrayDestructuringElementIr::Rest {
                target: DestructuringTargetIr::AssignmentPrivate { .. }
            }
        ]
    ));
}

#[test]
fn lowers_object_assignment_properties_and_private_rest_target() {
    let program = lower_script(
        "class C {
                #value;
                assign(source, key, target) {
                    ({ [key]: target.value, fallback = 1, ...this.#value } = source);
                }
            }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let function = script
        .functions
        .iter()
        .find(|function| function.name == "C.assign")
        .expect("assign method should be lowered");
    let pattern = function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::ObjectDestructure { pattern, .. },
                ..
            }) => Some(pattern),
            _ => None,
        })
        .expect("object assignment should be lowered");

    assert_eq!(pattern.properties.len(), 2);
    assert!(matches!(
        pattern.properties[0],
        ObjectDestructuringPropertyIr {
            key: DestructuringPropertyKeyIr::Computed(_),
            target: DestructuringTargetIr::AssignmentProperty { .. },
            default: None,
        }
    ));
    assert!(pattern.properties[1].default.is_some());
    assert!(matches!(
        pattern.rest,
        Some(DestructuringTargetIr::AssignmentPrivate { .. })
    ));
}

#[test]
fn plans_functions_in_array_pattern_defaults() {
    let program = lower_script(
            "let [first = function() { return 1; }] = []; var second; [second = function() { return 2; }] = [];",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    assert_eq!(
        script
            .functions
            .iter()
            .filter(|function| function.is_expression)
            .count(),
        2
    );
}

#[test]
fn array_assignment_targets_are_captured_in_nested_functions() {
    let program = lower_script(
            "function owner(iterable) { let outer = 0; function assign() { [outer] = iterable; return outer; } return assign; }",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let assign = script
        .functions
        .iter()
        .find(|function| function.name == "assign")
        .expect("nested assign function should be lowered");
    assert!(assign
        .captured_bindings
        .iter()
        .any(|binding| binding.name == "outer"));
}

#[test]
fn captured_const_array_assignment_target_remains_immutable() {
    let program =
        lower_script("const outer = 0; function assign(iterable) { [outer] = iterable; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let assign = script
        .functions
        .iter()
        .find(|function| function.name == "assign")
        .expect("assign function should be lowered");
    assert!(
        matches!(
            &assign.body.statements[0],
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::ArrayDestructure { pattern, .. },
                ..
            }) if matches!(
                &pattern.elements[0],
                ArrayDestructuringElementIr::Target {
                    target: DestructuringTargetIr::AssignmentIdentifier(reference),
                    ..
                } if matches!(
                    reference.write_disposition(),
                    IdentifierWriteDisposition::Throw {
                        error: IdentifierWriteErrorIr::ImmutableBinding
                            | IdentifierWriteErrorIr::ImmutableClassName,
                    }
                )
            )
        ),
        "{:?}",
        assign.body
    );
}

#[test]
fn depth_two_function_const_capture_preserves_array_target_immutability() {
    let program = lower_script(
            "function outer() { const value = 0; function middle() { function inner() { [value] = [1]; } return inner; } return middle; }",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let inner = script
        .functions
        .iter()
        .find(|function| function.name == "inner")
        .expect("inner function should be lowered");
    assert!(inner
        .captured_bindings
        .iter()
        .any(|binding| { binding.source_name == "value" && binding.mode == BindingMode::Const }));
    assert!(
        matches!(
            &inner.body.statements[0],
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::ArrayDestructure { pattern, .. },
                ..
            }) if matches!(
                &pattern.elements[0],
                ArrayDestructuringElementIr::Target {
                    target: DestructuringTargetIr::AssignmentIdentifier(reference),
                    ..
                } if matches!(
                    reference.write_disposition(),
                    IdentifierWriteDisposition::Throw {
                        error: IdentifierWriteErrorIr::ImmutableBinding
                            | IdentifierWriteErrorIr::ImmutableClassName,
                    }
                )
            )
        ),
        "{:?}",
        inner.body
    );
}

#[test]
fn depth_two_block_const_capture_uses_selected_environment_mode() {
    let source = "function outer() { { const value = 0; function middle() { function inner() { [value] = [1]; } return inner; } return middle; } }";
    with_script_analysis(source, |analysis| {
        let inner = analysis
            .function_plans
            .values()
            .find(|function| function.name == "inner")
            .expect("inner function should be planned");
        let (storage_name, capture) = inner
            .captures
            .iter()
            .find(|(_, capture)| capture.source_name == "value")
            .expect("inner should capture the block const binding");
        let environment = &analysis.environment_plans[&capture.environment_id];
        assert_eq!(environment.kind, EnvironmentKind::Block);
        assert_ne!(storage_name, "value");
        assert!(
            capture.environment_id
                != analysis.owner_plans[&capture.owner_id].activation_environment_id
        );
        assert_eq!(capture.mode, BindingMode::Const);
        assert_eq!(
            environment.binding_modes.get(storage_name),
            Some(&BindingMode::Const)
        );
    });
}

#[test]
fn generated_class_method_capture_preserves_planned_const_mode() {
    let program = lower_script(
            "function outer() { const value = 0; class Holder { assign() { [value] = [1]; } } return Holder; }",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let method = script
        .functions
        .iter()
        .find(|function| function.name.ends_with(".assign"))
        .unwrap_or_else(|| {
            panic!(
                "class method should be lowered: {:?}",
                script
                    .functions
                    .iter()
                    .map(|function| function.name.as_str())
                    .collect::<Vec<_>>()
            )
        });
    assert!(method
        .captured_bindings
        .iter()
        .any(|binding| { binding.source_name == "value" && binding.mode == BindingMode::Const }));
    assert!(
        matches!(
            &method.body.statements[0],
            StatementIr::Expression(TypedExpr {
                expr: ExprIr::ArrayDestructure { pattern, .. },
                ..
            }) if matches!(
                &pattern.elements[0],
                ArrayDestructuringElementIr::Target {
                    target: DestructuringTargetIr::AssignmentIdentifier(reference),
                    ..
                } if matches!(
                    reference.write_disposition(),
                    IdentifierWriteDisposition::Throw {
                        error: IdentifierWriteErrorIr::ImmutableBinding
                            | IdentifierWriteErrorIr::ImmutableClassName,
                    }
                )
            )
        ),
        "{:?}",
        method.body
    );
}

#[test]
fn capture_planning_preserves_annex_b_and_mutable_binding_modes() {
    with_script_analysis("if (true) { function copied() {} }", |analysis| {
        let activation = &analysis.environment_plans
            [&analysis.owner_plans[SCRIPT_OWNER_ID].activation_environment_id];
        assert_eq!(
            activation.binding_modes.get("copied"),
            Some(&BindingMode::Var)
        );
        assert!(analysis.environment_plans.values().any(|environment| {
            environment
                .binding_modes
                .iter()
                .any(|(name, mode)| name.starts_with("$annexb.block.") && *mode == BindingMode::Let)
        }));
    });

    let program =
        lower_script("function outer() { let value = 0; function read() { return value; } }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let read = script
        .functions
        .iter()
        .find(|function| function.name == "read")
        .expect("reader function should be lowered");
    assert!(read
        .captured_bindings
        .iter()
        .any(|binding| { binding.source_name == "value" && binding.mode == BindingMode::Let }));
}

#[test]
fn captured_var_assignments_share_the_owners_environment_slot() {
    let program = lower_script(
            "function outer(TA) { var ta; function read() { return ta.buffer; } ta = new TA(1); return read(); } outer(Float64Array);",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let outer = script
        .functions
        .iter()
        .find(|function| function.name == "outer")
        .expect("outer function should be lowered");
    let read = script
        .functions
        .iter()
        .find(|function| function.name == "read")
        .expect("read function should be lowered");
    let owned = outer
        .owned_env_bindings
        .iter()
        .find(|binding| binding.name == "ta")
        .expect("outer function should own the captured var");
    let captured = read
        .captured_bindings
        .iter()
        .find(|binding| binding.name == "ta")
        .expect("read function should capture the var");
    assert_eq!(captured.slot, owned.slot);
    let StatementIr::Return(TypedExpr {
        expr: ExprIr::SpecOperation { operands, .. },
        ..
    }) = &read.body.statements[0]
    else {
        panic!("captured property read should lower through GetV");
    };
    assert_eq!(
        operands[0].possible_kinds,
        KindSet::all_runtime_tags(),
        "a mutable capture must not retain its pre-assignment undefined type"
    );
}

#[test]
fn materializes_defaulted_object_var_property_reads_once() {
    let program = lower_script(
            "var getterHits = 0; var receiver = { get value() { getterHits += 1; return undefined; } }; function fallback() { return 1; } var { value = fallback() } = receiver; value;",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR");
    let statements = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::LexicalBlock(statements)
                if matches!(
                    statements.last(),
                    Some(StatementIr::DeclarationEvaluation(TypedExpr {
                        expr: ExprIr::ObjectDestructure { .. },
                        ..
                    }))
                ) =>
            {
                Some(statements)
            }
            _ => None,
        })
        .expect("object binding block");
    let [StatementIr::Lexical {
        name: temporary,
        init,
        ..
    }, initialization] = statements.as_slice()
    else {
        panic!("{statements:?}");
    };
    assert!(matches!(&init.expr, ExprIr::GlobalIdentifierRead { name } if name == "receiver"));
    let StatementIr::DeclarationEvaluation(TypedExpr {
        expr: ExprIr::ObjectDestructure { value, pattern },
        ..
    }) = initialization
    else {
        panic!("{initialization:?}");
    };
    assert!(matches!(&value.expr, ExprIr::Identifier(name) if name == temporary));
    let [property] = pattern.properties.as_slice() else {
        panic!("{pattern:?}");
    };
    assert_eq!(
        property.key,
        DestructuringPropertyKeyIr::Static("value".into())
    );
    assert!(
        matches!(&property.target, DestructuringTargetIr::Binding { mode: BindingMode::Var, name } if name == "value")
    );
    assert!(matches!(
        &property.default.as_ref().expect("one default").expr,
        ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
    ));
}

#[test]
fn hoists_object_var_bindings_in_for_of_loops() {
    let program = lower_script(
        "for (var { iterator, error } of [{ iterator: 1, error: 2 }]) {} iterator + error;",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR");
    for name in ["iterator", "error"] {
        assert!(script
            .global_bindings
            .iter()
            .any(|binding| binding.name == name
                && binding.declarations == GlobalDeclarationSetIr::Var));
    }
    let StatementIr::ForOfIterator { body, .. } = &script.body.statements[0] else {
        panic!("iterator loop");
    };
    let StatementIr::Block(block) = body.as_ref() else {
        panic!("head prefix block");
    };
    let StatementIr::DeclarationEvaluation(TypedExpr {
        expr: ExprIr::ObjectDestructure { pattern, .. },
        ..
    }) = &block.statements[0]
    else {
        panic!("semantic object binding");
    };
    let mut bindings = Vec::new();
    pattern.visit_bindings(&mut |mode, name| bindings.push((mode, name.to_owned())));
    assert_eq!(
        bindings,
        [
            (BindingMode::Var, "iterator".into()),
            (BindingMode::Var, "error".into())
        ]
    );
}

#[test]
fn materializes_defaulted_object_var_property_reads_in_for_of_loops() {
    let program =
        lower_script("for (var { value = fallback() } of [{ value: undefined }]) {} value;");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR");
    let StatementIr::ForOfIterator { body, .. } = &script.body.statements[0] else {
        panic!("iterator loop");
    };
    let StatementIr::Block(block) = body.as_ref() else {
        panic!("head prefix block");
    };
    let StatementIr::DeclarationEvaluation(TypedExpr {
        expr: ExprIr::ObjectDestructure { pattern, .. },
        ..
    }) = &block.statements[0]
    else {
        panic!("semantic object binding");
    };
    let [property] = pattern.properties.as_slice() else {
        panic!("{pattern:?}");
    };
    assert!(
        matches!(&property.target, DestructuringTargetIr::Binding { mode: BindingMode::Var, name } if name == "value")
    );
    assert!(matches!(
        &property.default.as_ref().expect("one default").expr,
        ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
    ));
}

#[test]
fn hoists_object_var_bindings_from_for_of_loops_in_functions() {
    let program = lower_script(
        "function values() { for (var { value } of [{ value: 1 }]) {} return value; } values();",
    );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    assert!(!script
        .global_bindings
        .iter()
        .any(|binding| binding.name == "value"));
    let function = script
        .functions
        .iter()
        .find(|function| function.name == "values")
        .expect("function should be lowered");
    assert!(matches!(
        function.body.statements.last(),
        Some(StatementIr::Return(TypedExpr {
            expr: ExprIr::Identifier(name),
            ..
        })) if name == "value"
    ));
}

#[test]
fn object_destructuring_predeclares_tdz_and_coerces_empty_patterns() {
    let program = lower_script("let { value } = value; let {} = null; let {} = undefined;");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");

    let StatementIr::LexicalBlock(value_binding) = &script.body.statements[0] else {
        panic!("expected value destructuring block");
    };
    let StatementIr::Lexical { init, .. } = &value_binding[0] else {
        panic!("expected RHS materialization");
    };
    assert!(matches!(init.expr, ExprIr::RuntimeThrow { .. }));

    for statement in &script.body.statements[1..] {
        let StatementIr::LexicalBlock(bindings) = statement else {
            panic!("expected empty destructuring block");
        };
        assert_eq!(bindings.len(), 2);
        let StatementIr::DeclarationEvaluation(coercion) = &bindings[1] else {
            panic!("expected RequireObjectCoercible representation");
        };
        assert!(matches!(
            coercion.expr,
            ExprIr::ObjectDestructure { ref pattern, .. }
                if pattern.properties.is_empty() && pattern.rest.is_none()
        ));
    }
}

#[test]
fn object_var_destructuring_coerces_empty_patterns() {
    let program = lower_script("var {} = null;");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::LexicalBlock(bindings) = &script.body.statements[0] else {
        panic!("expected empty destructuring block");
    };
    assert_eq!(bindings.len(), 2);
    let StatementIr::DeclarationEvaluation(coercion) = &bindings[1] else {
        panic!("expected RequireObjectCoercible representation");
    };
    assert!(matches!(
        coercion.expr,
        ExprIr::ObjectDestructure { ref pattern, .. }
            if pattern.properties.is_empty() && pattern.rest.is_none()
    ));
}

#[test]
fn lowers_computed_key_object_destructuring_forms() {
    // A computed key contributes no bound name (8.6 BoundNames), so these bind
    // exactly what the literal-key spelling binds and lower through the semantic
    // `ObjectDestructure` node, which carries the key expression.
    for source in [
        "let { [key]: value } = source;",
        "let { ['value']: value } = source;",
        "const { [key]: value = 1 } = source;",
        "const { [key]: { nested } } = source;",
        "let { [key]: value, ...rest } = source;",
        "for (const { [key]: value } of source) print(value);",
    ] {
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "expected supported lowering for {source}: {:?}",
            program.diagnostics
        );
    }
}

#[test]
fn lowers_object_assignment_patterns_in_loop_heads() {
    // 13.15.5 destructuring assignment is legal in a `for-in`/`for-of` head, and
    // the object shape reaches the same assignment-pattern lowering the array
    // shape does, so property-access and rest targets work there too.
    for source in [
        "let a; for ({ a } of source) print(a);",
        "let a; for ({ a = 1 } of source) print(a);",
        "const o = {}; for ({ a: o.x } of source) print(o.x);",
        "let r; for ({ ...r } of source) print(r);",
        "let a; for ({ length: a } in source) print(a);",
        "let a; for ([a] in source) print(a);",
    ] {
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "expected supported lowering for {source}: {:?}",
            program.diagnostics
        );
    }
}

#[test]
fn lowers_nested_and_rest_object_destructuring_bindings() {
    for source in [
        "let { value: { nested } } = source;",
        "let { value, ...rest } = source;",
        "let { value: [first] } = source;",
        "var { value: [first], ...rest } = source;",
        "let a, b; ({ value: [a], ...b } = source);",
    ] {
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "expected supported lowering for {source}: {:?}",
            program.diagnostics
        );
    }
}

fn debug_of_functions(program: &ProgramIr) -> String {
    format!("{:?}", program.script.as_ref().unwrap().functions)
}

#[test]
fn super_property_destructuring_targets_capture_their_reference_before_the_element_value() {
    let program = lower_script(
        r#"
class base {}
class derived extends base {
  m(arr, o, k) {
    [super.a, super["b"]] = arr;
    ({ x: super[k], ...super.rest } = o);
    [[super.deep]] = arr;
  }
}
"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let body = debug_of_functions(&program);
    assert_eq!(body.matches("AssignmentSuper").count(), 5, "{body}");
    // The Reference is captured when the target is prepared, then written
    // through the captured slots; nothing re-resolves the base at PutValue.
    assert_eq!(body.matches("Capture(").count(), 5, "{body}");
    assert_eq!(body.matches("PutCaptured").count(), 5, "{body}");
    assert!(!body.contains("SuperPropertyWrite"), "{body}");
}

#[test]
fn parenthesized_super_destructuring_targets_lower_like_unparenthesized_ones() {
    let program = lower_script(
        r#"
var obj = { x() { var b; [(super.man) = 1, b] = [1, 2]; [(super[8 + {}]) = 'motel'] = [1]; } };
"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let body = debug_of_functions(&program);
    assert_eq!(body.matches("AssignmentSuper").count(), 2, "{body}");
}

#[test]
fn grouped_assignment_targets_preserve_property_and_rest_references() {
    for source in [
        "var x, o = {}; [(x), ((o.value)), ...((o.rest))] = source;",
        "var x, o = {}; ({value: (x), other: ((o.value)), ...((o.rest))} = source);",
        "var o = {}; for ([(o.value)] of source) {}",
        "var o = {}; for ({value: (o.value)} of source) {}",
    ] {
        let program = lower_script(source);
        assert!(program.is_wasm_supported(), "{source}: {:?}", program.diagnostics);
    }
    let program = lower_script(
        "var o = {m(source) { [(super.value)] = source; ({...((super.rest))} = source); }};",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let body = debug_of_functions(&program);
    assert_eq!(body.matches("AssignmentSuper").count(), 2, "{body}");
    assert_eq!(body.matches("Capture(").count(), 2, "{body}");
    assert_eq!(body.matches("PutCaptured").count(), 2, "{body}");
}

#[test]
fn super_property_for_of_and_for_in_heads_write_through_the_super_reference() {
    let program = lower_script(
        r#"
class base {}
class derived extends base {
  m(it, o, k) {
    for (super.x of it) {}
    for (super[k] of it) {}
    for (super.y in o) {}
  }
}
"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let body = debug_of_functions(&program);
    assert_eq!(body.matches("SuperPropertyWrite").count(), 3, "{body}");
}

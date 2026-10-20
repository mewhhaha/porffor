#[test]
fn lowers_optional_property_chain_as_one_ir_expression() {
    let program = lower_script("let a; a?.b;");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(expr) = &script.body.statements[1] else {
        panic!("expected optional-chain expression statement");
    };
    let ExprIr::OptionalPropertyChain { target, chain } = &expr.expr else {
        panic!("expected one optional-property-chain IR expression");
    };
    assert!(matches!(target.expr, ExprIr::Identifier(_)));
    assert_eq!(chain.len(), 1);
    let OptionalChainOperationIr::Property { key, shorted } = &chain[0] else {
        panic!("expected optional property operation");
    };
    assert_eq!(key, &PropertyKeyIr::StaticString("b".to_string()));
    assert!(*shorted);
}

#[test]
fn optional_property_chain_preserves_each_operation_shorted_flag() {
    for (source, expected_flags) in [
        ("let a; a?.b.c;", vec![true, false]),
        ("let a; a?.b?.c;", vec![true, true]),
    ] {
        let program = lower_script(source);
        assert!(program.is_wasm_supported(), "{source}");
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected optional-chain expression statement");
        };
        let ExprIr::OptionalPropertyChain { chain, .. } = &expr.expr else {
            panic!("expected optional-property-chain IR expression");
        };
        assert_eq!(
            chain
                .iter()
                .map(|operation| match operation {
                    OptionalChainOperationIr::Property { shorted, .. }
                    | OptionalChainOperationIr::PrivateProperty { shorted, .. }
                    | OptionalChainOperationIr::Call { shorted, .. } => *shorted,
                })
                .collect::<Vec<_>>(),
            expected_flags,
            "{source}"
        );
    }
}

#[test]
fn optional_private_access_preserves_chain_order_receiver_and_private_identity() {
    let source = "
            class A {
                #field = 1;
                #method() { return this; }
                read(o) { return o?.c.#field; }
                call(o) { return o?.#method(); }
            }
            class B {
                #field = 2;
                read(o) { return o?.c.#field; }
            }
        ";
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script ir should exist");
    let mut field_ids = Vec::new();

    for function_name in ["A.read", "B.read"] {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == function_name)
            .unwrap_or_else(|| panic!("missing `{function_name}`"));
        let StatementIr::Return(expr) = &function.body.statements[0] else {
            panic!("expected return from `{function_name}`");
        };
        let ExprIr::OptionalPropertyChain { chain, .. } = &expr.expr else {
            panic!("expected optional chain from `{function_name}`");
        };
        assert!(matches!(
            chain.first(),
            Some(OptionalChainOperationIr::Property {
                key: PropertyKeyIr::StaticString(key),
                shorted: true,
            }) if key == "c"
        ));
        let Some(OptionalChainOperationIr::PrivateProperty {
            private_name_id,
            shorted: false,
        }) = chain.get(1)
        else {
            panic!("expected private tail from `{function_name}`: {chain:?}");
        };
        field_ids.push(*private_name_id);
    }
    assert_ne!(field_ids[0], field_ids[1]);

    let call = script
        .functions
        .iter()
        .find(|function| function.name == "A.call")
        .expect("missing `A.call`");
    let StatementIr::Return(expr) = &call.body.statements[0] else {
        panic!("expected return from `A.call`");
    };
    let ExprIr::OptionalPropertyChain { chain, .. } = &expr.expr else {
        panic!("expected optional chain from `A.call`");
    };
    assert!(matches!(
        chain.as_slice(),
        [
            OptionalChainOperationIr::PrivateProperty { shorted: true, .. },
            OptionalChainOperationIr::Call {
                args,
                receiver: OptionalChainCallReceiverIr::ReferenceOrUndefined,
                shorted: false,
                boundary_before: false,
            },
        ] if args.is_empty()
    ));
}

#[test]
fn optional_property_chain_keeps_computed_key_inside_chain() {
    let program = lower_script("function key() { return 'x'; } let a; a?.[key()];");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let Some(StatementIr::Expression(expr)) = script.body.statements.last() else {
        panic!("expected optional-chain expression statement");
    };
    let ExprIr::OptionalPropertyChain { chain, .. } = &expr.expr else {
        panic!("expected optional-property-chain IR expression");
    };
    let OptionalChainOperationIr::Property { key, .. } = &chain[0] else {
        panic!("expected optional property operation, got {:?}", chain[0]);
    };
    let PropertyKeyIr::StringExpr(key) = key else {
        panic!("expected deferred computed key, got {key:?}");
    };
    assert!(
        matches!(
            key.expr,
            ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
        ),
        "expected the deferred key expression to contain the call, got {:?}",
        key.expr
    );
}

#[test]
fn leading_private_optional_calls_keep_mandatory_reference_read_before_guarded_arguments() {
    let program = lower_script(
        "function argument() { return 1; } \
         class C { \
           #method() { return this; } \
           direct(o) { return o.#method?.(argument()); } \
           grouped(o) { return ((o.#method))?.(argument()); } \
           nullReceiver() { return null.#method?.(argument()); } \
         }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR");
    let mut identities = Vec::new();
    for name in ["C.direct", "C.grouped", "C.nullReceiver"] {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == name)
            .unwrap_or_else(|| panic!("missing {name}"));
        let StatementIr::Return(value) = &function.body.statements[0] else {
            panic!("private optional call return in {name}");
        };
        let ExprIr::OptionalPropertyChain { target, chain } = &value.expr else {
            panic!("one ordered private Reference chain in {name}");
        };
        if name == "C.nullReceiver" {
            assert!(matches!(target.expr, ExprIr::Null));
        } else {
            assert!(matches!(target.expr, ExprIr::Identifier(_)));
        }
        let [OptionalChainOperationIr::PrivateProperty {
            private_name_id,
            shorted: false,
        }, OptionalChainOperationIr::Call {
            args,
            receiver: OptionalChainCallReceiverIr::ReferenceOrUndefined,
            shorted: true,
            boundary_before: false,
        }] = chain.as_slice()
        else {
            panic!("mandatory private Get followed by optional Call in {name}: {chain:?}");
        };
        assert_eq!(args.len(), 1);
        assert!(matches!(
            args[0].expr,
            ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
        ));
        identities.push(*private_name_id);
    }
    assert!(identities.windows(2).all(|pair| pair[0] == pair[1]));
}

#[test]
fn optional_call_is_retained_in_ordered_chain_ir() {
    let program = lower_script("function arg() { return 1; } let fn; fn?.(arg());");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let Some(StatementIr::Expression(expr)) = script.body.statements.last() else {
        panic!("expected optional-call expression statement");
    };
    let ExprIr::OptionalPropertyChain { target, chain } = &expr.expr else {
        panic!("expected ordered optional-chain IR expression");
    };
    assert!(matches!(target.expr, ExprIr::Identifier(_)));
    assert_eq!(chain.len(), 1);
    let OptionalChainOperationIr::Call {
        args,
        receiver,
        shorted,
        boundary_before,
    } = &chain[0]
    else {
        panic!("expected optional call operation, got {:?}", chain[0]);
    };
    assert!(*shorted);
    assert_eq!(*receiver, OptionalChainCallReceiverIr::ReferenceOrUndefined);
    assert!(!*boundary_before);
    assert_eq!(args.len(), 1);
    assert!(matches!(
        args[0].expr,
        ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
    ));
}

#[test]
fn optional_call_argument_effects_discard_the_captured_receiver_shape() {
    let source = "const o = { x: 1, f() { return this.x; } }; o?.f(o.x = 's') + 1;";
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
        panic!("expected optional-call addition");
    };
    assert!(
        matches!(result.expr, ExprIr::CoerciveAdd { .. }),
        "the argument write must invalidate the receiver shape before `f` observes `this`: {:?}",
        result.expr
    );
}

#[test]
fn optional_getter_effects_precede_argument_analysis() {
    let source = "const observed = { x: 1 }; function result() { return 0; } const o = { get f() { delete observed.x; return result; } }; o?.f(observed.x + 1);";
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Expression(call) = script.body.statements.last().unwrap() else {
        panic!("expected optional call");
    };
    let ExprIr::OptionalPropertyChain { chain, .. } = &call.expr else {
        panic!("expected optional-property-chain IR, got {:?}", call.expr);
    };
    let Some(OptionalChainOperationIr::Call { args, .. }) = chain.last() else {
        panic!("expected optional call operation, got {chain:?}");
    };
    assert!(
        matches!(
            args.as_slice(),
            [TypedExpr {
                expr: ExprIr::CoerciveAdd { .. },
                ..
            }]
        ),
        "the getter can invalidate `observed.x` before its argument is evaluated: {args:?}"
    );
}

#[test]
fn optional_method_calls_preserve_reference_and_shorted_flags() {
    for (source, expected) in [
        (
            "let obj; obj?.method();",
            vec![("property", true), ("call", false)],
        ),
        (
            "let obj; obj.method?.();",
            vec![("property", false), ("call", true)],
        ),
        (
            "let obj; obj?.method?.().x;",
            vec![("property", true), ("call", true), ("property", false)],
        ),
    ] {
        let program = lower_script(source);
        assert!(program.is_wasm_supported(), "{source}");
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected optional-chain expression for {source}");
        };
        let ExprIr::OptionalPropertyChain { target, chain } = &expr.expr else {
            panic!("expected ordered optional-chain IR for {source}");
        };
        assert!(
            matches!(target.expr, ExprIr::Identifier(_)),
            "member-call base must be stored once for {source}, got {:?}",
            target.expr
        );
        assert_eq!(chain.len(), expected.len(), "{source}");
        assert_eq!(
            chain
                .iter()
                .map(|operation| match operation {
                    OptionalChainOperationIr::Property { shorted, .. } => {
                        ("property", *shorted)
                    }
                    OptionalChainOperationIr::PrivateProperty { shorted, .. } => {
                        ("private", *shorted)
                    }
                    OptionalChainOperationIr::Call { shorted, .. } => ("call", *shorted),
                })
                .collect::<Vec<_>>(),
            expected,
            "{source}"
        );
        let OptionalChainOperationIr::Property { key, .. } = &chain[0] else {
            unreachable!();
        };
        assert_eq!(key, &PropertyKeyIr::StaticString("method".to_string()));
        if source.ends_with(".x;") {
            let OptionalChainOperationIr::Property { key, .. } = &chain[2] else {
                unreachable!();
            };
            assert_eq!(key, &PropertyKeyIr::StaticString("x".to_string()));
        }
    }
}

#[test]
fn optional_member_call_keeps_effectful_base_once() {
    let program = lower_script("function base() { return { method() {} }; } base().method?.();");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(expr) = script.body.statements.last().unwrap() else {
        panic!("expected optional-call expression statement");
    };
    let ExprIr::OptionalPropertyChain { target, chain } = &expr.expr else {
        panic!("expected ordered optional-chain IR expression");
    };
    assert!(matches!(
        target.expr,
        ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
    ));
    assert!(matches!(
        chain.as_slice(),
        [
            OptionalChainOperationIr::Property {
                key: PropertyKeyIr::StaticString(key),
                shorted: false,
            },
            OptionalChainOperationIr::Call {
                args,
                receiver: OptionalChainCallReceiverIr::ReferenceOrUndefined,
                shorted: true,
                boundary_before: false,
            },
        ] if key == "method" && args.is_empty()
    ));
}

#[test]
fn grouped_optional_chain_calls_mark_new_short_circuit_boundaries() {
    for (source, expected_call_flags) in [
        ("let a; (a?.b)();", vec![(false, true)]),
        ("let a; (a?.b)?.();", vec![(true, true)]),
        ("let a; ((a?.b)())();", vec![(false, true), (false, true)]),
    ] {
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(expr) = &script.body.statements[1] else {
            panic!("expected grouped optional-chain expression for {source}");
        };
        let ExprIr::OptionalPropertyChain { target, chain } = &expr.expr else {
            panic!("expected flattened optional-chain IR for {source}");
        };
        assert!(matches!(target.expr, ExprIr::Identifier(_)), "{source}");
        assert!(matches!(
            chain.first(),
            Some(OptionalChainOperationIr::Property {
                key: PropertyKeyIr::StaticString(key),
                shorted: true,
            }) if key == "b"
        ));
        assert_eq!(
            chain
                .iter()
                .filter_map(|operation| match operation {
                    OptionalChainOperationIr::Call {
                        shorted,
                        boundary_before,
                        ..
                    } => Some((*shorted, *boundary_before)),
                    OptionalChainOperationIr::Property { .. }
                    | OptionalChainOperationIr::PrivateProperty { .. } => None,
                })
                .collect::<Vec<_>>(),
            expected_call_flags,
            "{source}"
        );
    }
}

#[test]
fn grouped_ordinary_call_keeps_arguments_after_optional_segment_boundary() {
    let program = lower_script("function arg() { return 1; } let a; (a?.b)(arg());");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let Some(StatementIr::Expression(expr)) = script.body.statements.last() else {
        panic!("expected grouped optional-chain expression");
    };
    let ExprIr::OptionalPropertyChain { chain, .. } = &expr.expr else {
        panic!("expected flattened optional-chain IR");
    };
    let Some(OptionalChainOperationIr::Call {
        args,
        receiver: OptionalChainCallReceiverIr::ReferenceOrUndefined,
        shorted: false,
        boundary_before: true,
    }) = chain.last()
    else {
        panic!("expected ordinary grouped call boundary, got {chain:?}");
    };
    assert!(matches!(
        args.as_slice(),
        [TypedExpr {
            expr: ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. },
            ..
        }]
    ));
}

#[test]
fn optional_primitive_method_calls_preserve_strict_this_and_the_raw_receiver() {
    for (source, function_name, expected_kind) in [
            (
                "String.prototype.q = function stringQ() { 'use strict'; return this === 'z'; }; 'z'?.q();",
                "stringQ",
                ValueKind::String,
            ),
            (
                "Number.prototype.q = function numberQ() { 'use strict'; return this === 3; }; (3)?.q();",
                "numberQ",
                ValueKind::Number,
            ),
        ] {
            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script ir should exist");
            let function = script
                .functions
                .iter()
                .find(|function| function.name == function_name)
                .unwrap_or_else(|| panic!("missing {function_name} for {source}"));
            let this_operand = function.body.statements.iter().find_map(|statement| {
                let StatementIr::Return(TypedExpr {
                    expr:
                        ExprIr::SpecOperation {
                            operation: SpecOperationIr::StrictEqualityComparison,
                            operands,
                        },
                    ..
                }) = statement
                else {
                    return None;
                };
                operands
                    .iter()
                    .find(|operand| matches!(operand.expr, ExprIr::This))
            });
            assert!(function.strict);
            assert!(this_operand
                .expect("strict method retains This")
                .possible_kinds
                .contains(expected_kind));
            let Some(StatementIr::Expression(TypedExpr {
                expr: ExprIr::OptionalPropertyChain { target, chain }, ..
            })) = script.body.statements.last() else {
                panic!("primitive method keeps its optional Reference: {source}");
            };
            assert_eq!(target.kind, expected_kind);
            assert!(matches!(chain.as_slice(), [
                OptionalChainOperationIr::Property {
                    key: PropertyKeyIr::StaticString(name), shorted: true
                },
                OptionalChainOperationIr::Call {
                    receiver: OptionalChainCallReceiverIr::ReferenceOrUndefined,
                    shorted: false, boundary_before: false, args
                }
            ] if name == "q" && args.is_empty()));
        }
}

#[test]
fn optional_factory_calls_retain_the_returned_value_as_the_method_receiver() {
    for (source, factory_name, function_name, expected_kind) in [
            (
                "String.prototype.q = function stringFactoryQ() { 'use strict'; return this === 's'; }; function makeString() { return 's'; } makeString?.().q();",
                "makeString",
                "stringFactoryQ",
                ValueKind::String,
            ),
            (
                "Number.prototype.q = function numberFactoryQ() { 'use strict'; return this === 3; }; function makeNumber() { return 3; } makeNumber?.().q();",
                "makeNumber",
                "numberFactoryQ",
                ValueKind::Number,
            ),
            (
                "Boolean.prototype.q = function booleanFactoryQ() { 'use strict'; return this === true; }; function makeBoolean() { return true; } makeBoolean?.().q();",
                "makeBoolean",
                "booleanFactoryQ",
                ValueKind::Boolean,
            ),
            (
                "BigInt.prototype.q = function bigintFactoryQ() { 'use strict'; return this === 3n; }; function makeBigInt() { return 3n; } makeBigInt?.().q();",
                "makeBigInt",
                "bigintFactoryQ",
                ValueKind::BigInt,
            ),
            (
                "Symbol.prototype.q = function symbolFactoryQ() { 'use strict'; return this === this; }; function makeSymbol() { return Symbol('marker'); } makeSymbol?.().q();",
                "makeSymbol",
                "symbolFactoryQ",
                ValueKind::Symbol,
            ),
        ] {
            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script ir should exist");
            let factory = script
                .functions
                .iter()
                .find(|function| function.name == factory_name)
                .unwrap_or_else(|| panic!("missing {factory_name} for {source}"));
            if expected_kind == ValueKind::Symbol {
                // Installing q can invoke inherited hooks, including a
                // replacement of the mutable global Symbol constructor.
                assert!(function_return(factory)
                    .expect("factory return")
                    .possible_kinds
                    .contains(expected_kind));
            } else {
                assert_eq!(factory.return_kind, expected_kind, "factory return for {source}");
            }
            let function = script
                .functions
                .iter()
                .find(|function| function.name == function_name)
                .unwrap_or_else(|| panic!("missing {function_name} for {source}"));
            let this_operand = function.body.statements.iter().find_map(|statement| {
                let StatementIr::Return(TypedExpr {
                    expr:
                        ExprIr::SpecOperation {
                            operation: SpecOperationIr::StrictEqualityComparison,
                            operands,
                        },
                    ..
                }) = statement
                else {
                    return None;
                };
                operands
                    .iter()
                    .find(|operand| matches!(operand.expr, ExprIr::This))
            });
            assert!(function.strict);
            assert!(this_operand
                .expect("strict method retains This")
                .possible_kinds
                .contains(expected_kind));
            let Some(StatementIr::Expression(TypedExpr {
                expr: ExprIr::OptionalPropertyChain { target, chain }, ..
            })) = script.body.statements.last() else {
                panic!("factory and method must share one optional chain: {source}");
            };
            assert!(target.function_targets.known_targets().contains(&factory.id));
            assert!(matches!(chain.as_slice(), [
                OptionalChainOperationIr::Call { shorted: true, args: factory_args, .. },
                OptionalChainOperationIr::Property {
                    key: PropertyKeyIr::StaticString(name), shorted: false
                },
                OptionalChainOperationIr::Call {
                    receiver: OptionalChainCallReceiverIr::ReferenceOrUndefined,
                    shorted: false, boundary_before: false, args
                }
            ] if name == "q" && factory_args.is_empty() && args.is_empty()));
        }
}

#[test]
fn optional_eval_call_registers_an_indirect_script() {
    let program = lower_script("eval?.('source');");
    assert_prepared_script(&program, PreparedScriptKind::IndirectEval);
}

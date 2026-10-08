#[test]
fn marks_explicit_extending_class_constructor_as_derived() {
    let program = lower_script("class A {} class B extends A { constructor() { super(); } } B;");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let derived = script
        .functions
        .iter()
        .find(|function| function.name == "B")
        .expect("derived constructor should be lowered");
    assert!(derived.is_derived_constructor);
    assert!(derived.super_constructor_target.is_some());
    assert_canonical_derived_activation(derived);
}

#[test]
fn gives_default_derived_constructor_canonical_activation_but_not_base_constructor() {
    let program = lower_script("class A {} class B extends A {} B;");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let base = script
        .functions
        .iter()
        .find(|function| function.name == "A")
        .expect("base constructor should be lowered");
    assert!(!base.is_derived_constructor);
    assert!(base.lexical_derived_activation.is_none());
    assert!(base.owned_env_bindings.iter().all(|binding| {
        ![
            DERIVED_ACTIVATION_THIS_NAME,
            DERIVED_ACTIVATION_THIS_STATUS_NAME,
            DERIVED_ACTIVATION_NEW_TARGET_NAME,
            DERIVED_ACTIVATION_FUNCTION_NAME,
        ]
        .contains(&binding.name.as_str())
    }));

    let derived = script
        .functions
        .iter()
        .find(|function| function.name == "B")
        .expect("default derived constructor should be lowered");
    assert!(derived.is_derived_constructor);
    assert!(derived.is_synthetic_default_derived_constructor);
    assert_canonical_derived_activation(derived);
}

#[test]
fn lowers_immediate_arrow_super_with_derived_activation_capture() {
    let program =
        lower_script("class A {} class B extends A { constructor() { (() => super())(); } }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let constructor = script
        .functions
        .iter()
        .find(|function| function.is_derived_constructor)
        .unwrap();
    let activation = constructor.lexical_derived_activation.as_ref().unwrap();
    assert_eq!(activation.owner_function_id, constructor.id);
    let arrow = script
        .functions
        .iter()
        .find(|function| function.protocol.flavor() == FunctionFlavor::Arrow)
        .unwrap();
    assert!(arrow.uses_super);
    assert!(arrow
        .captured_bindings
        .iter()
        .any(|binding| binding.name == DERIVED_ACTIVATION_FUNCTION_NAME));
    assert!(arrow.body.statements.iter().any(|statement| matches!(
        statement,
        StatementIr::Return(TypedExpr {
            expr: ExprIr::SuperConstruct { .. },
            ..
        })
    )));
}

#[test]
fn nested_arrows_share_derived_activation_this_and_new_target() {
    let program = lower_script(
            "class A {} class B extends A { constructor() { (() => (() => [this, new.target, super()]))(); } }",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let constructor = script
        .functions
        .iter()
        .find(|function| function.is_derived_constructor)
        .unwrap();
    let activation = constructor.lexical_derived_activation.as_ref().unwrap();
    for name in [
        &activation.this_binding,
        &activation.this_status_binding,
        &activation.new_target_binding,
        &activation.active_function_binding,
    ] {
        assert!(constructor
            .owned_env_bindings
            .iter()
            .any(|binding| &binding.name == name));
    }
    let arrows = script
        .functions
        .iter()
        .filter(|function| function.protocol.flavor() == FunctionFlavor::Arrow)
        .collect::<Vec<_>>();
    assert_eq!(arrows.len(), 2);
    let innermost = arrows.iter().find(|function| function.uses_super).unwrap();
    assert!(innermost.captures_lexical_this);
    assert!(innermost
        .captured_bindings
        .iter()
        .any(|binding| binding.name == LEXICAL_NEW_TARGET_NAME));
    assert!(innermost
        .captured_bindings
        .iter()
        .any(|binding| binding.name == DERIVED_ACTIVATION_THIS_STATUS_NAME));
}

#[test]
fn derived_arrow_this_and_new_target_capture_activation_without_super_flag() {
    for source in [
        "class A {} class B extends A { constructor() { (() => this)(); } }",
        "class A {} class B extends A { constructor() { (() => new.target)(); } }",
    ] {
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().unwrap();
        let arrow = script
            .functions
            .iter()
            .find(|function| function.protocol.flavor() == FunctionFlavor::Arrow)
            .unwrap();
        assert!(!arrow.uses_super);
        for name in [
            DERIVED_ACTIVATION_FUNCTION_NAME,
            DERIVED_ACTIVATION_NEW_TARGET_NAME,
            DERIVED_ACTIVATION_THIS_NAME,
            DERIVED_ACTIVATION_THIS_STATUS_NAME,
        ] {
            assert!(arrow
                .captured_bindings
                .iter()
                .any(|binding| binding.name == name));
        }
    }
}

#[test]
fn derived_arrow_super_property_captures_activation_and_preserves_slots() {
    let program = lower_script(
            "class A { get x() { return 1; } } class B extends A { constructor() { let $a = 1; (() => (() => super.x + $a))(); } }",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().unwrap();
    let constructor = script
        .functions
        .iter()
        .find(|function| function.is_derived_constructor)
        .unwrap();
    let slots = constructor
        .owned_env_bindings
        .iter()
        .map(|binding| (binding.name.as_str(), binding.slot))
        .collect::<BTreeMap<_, _>>();
    assert_eq!(slots.get(DERIVED_ACTIVATION_FUNCTION_NAME), Some(&0));
    assert_eq!(slots.get(DERIVED_ACTIVATION_NEW_TARGET_NAME), Some(&1));
    assert_eq!(slots.get(DERIVED_ACTIVATION_THIS_NAME), Some(&2));
    assert_eq!(slots.get(DERIVED_ACTIVATION_THIS_STATUS_NAME), Some(&3));
    assert_eq!(slots.get("$a"), Some(&4));
    let arrows = script
        .functions
        .iter()
        .filter(|function| function.protocol.flavor() == FunctionFlavor::Arrow)
        .collect::<Vec<_>>();
    assert_eq!(arrows.len(), 2);
    assert!(arrows.iter().any(|function| function.uses_super));
}

#[test]
fn derived_arrow_dynamic_super_method_call_uses_lexical_this() {
    let source = "class A { increment() {} } class B extends A { constructor() { super(); (() => super.increment())(); } }";
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script ir should exist");
    let arrow = script
        .functions
        .iter()
        .find(|function| function.protocol.flavor() == FunctionFlavor::Arrow)
        .expect("arrow should be lowered");
    let Some(StatementIr::Return(TypedExpr {
        expr:
            ExprIr::CallIndirect {
                callee,
                this_arg: Some(this_arg),
                args,
                ..
            },
        ..
    })) = arrow.body.statements.first()
    else {
        panic!("expected indirect super method call: {:?}", arrow.body);
    };
    assert!(matches!(
        callee.expr,
        ExprIr::SuperPropertyRead {
            key: PropertyKeyIr::StaticString(ref key),
            ..
        } if key == "increment"
    ));
    assert!(matches!(this_arg.expr, ExprIr::This));
    assert!(args.is_empty());
}

#[test]
fn class_method_arrow_captures_lexical_home_object_and_this_for_super() {
    let source =
        "class A { method() {} } class B extends A { make() { return () => super.method(); } }";
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script ir should exist");
    let method = script
        .functions
        .iter()
        .find(|function| function.name == "B.make")
        .expect("class method should be lowered");
    for name in [LEXICAL_THIS_NAME, LEXICAL_HOME_OBJECT_NAME] {
        assert!(method
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == name));
    }
    let arrow = script
        .functions
        .iter()
        .find(|function| function.protocol.flavor() == FunctionFlavor::Arrow)
        .expect("arrow should be lowered");
    for name in [LEXICAL_THIS_NAME, LEXICAL_HOME_OBJECT_NAME] {
        assert!(arrow
            .captured_bindings
            .iter()
            .any(|binding| binding.name == name));
    }
    assert!(arrow.captures_lexical_this);
}

#[test]
fn object_method_arrow_super_captures_paired_home_object_authority() {
    let source = r#"
            const object = {
                parameter(value = (() => super.seed)()) {},
                body() { return () => super.seed; },
                nested() { return () => () => super.seed; }
            };
        "#;
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script ir should exist");
    let methods = script
        .functions
        .iter()
        .filter(|function| function.protocol.is_object_literal_method())
        .collect::<Vec<_>>();
    assert_eq!(methods.len(), 3);
    for method in methods {
        for name in [LEXICAL_THIS_NAME, LEXICAL_HOME_OBJECT_NAME] {
            assert!(method
                .owned_env_bindings
                .iter()
                .any(|binding| binding.name == name));
        }
    }

    let arrows = script
        .functions
        .iter()
        .filter(|function| function.protocol.flavor() == FunctionFlavor::Arrow)
        .collect::<Vec<_>>();
    assert_eq!(arrows.len(), 4);
    let lexical_super_arrows = arrows
        .into_iter()
        .filter(|function| function.captures_lexical_this)
        .collect::<Vec<_>>();
    assert!(lexical_super_arrows.len() >= 3);
    for arrow in lexical_super_arrows {
        let captured = arrow
            .captured_bindings
            .iter()
            .map(|binding| binding.source_name.as_str())
            .collect::<BTreeSet<_>>();
        assert!(captured.contains(LEXICAL_THIS_NAME));
        assert!(captured.contains(LEXICAL_HOME_OBJECT_NAME));
    }
}

#[test]
fn exact_context_specialization_preserves_escaped_closure_environment() {
    let source = "class B { make() { let x = 7; return () => () => x; } } let b = new B(); let outer = b.make(); let inner = outer(); inner();";
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script ir should exist");
    let inner_init = script.body.statements.iter().find_map(|statement| {
        let StatementIr::Lexical { name, init, .. } = statement else {
            return None;
        };
        (name == "inner").then_some(init)
    });
    let Some(TypedExpr {
        expr: ExprIr::CallIndirect { callee, .. },
        ..
    }) = inner_init
    else {
        panic!("expected escaped outer closure call: {:?}", script.body);
    };
    assert!(matches!(
        callee.expr,
        ExprIr::Identifier(ref name) if name == "outer"
    ));
}

#[test]
fn exact_context_specialization_preserves_escaped_callback_argument() {
    let source = "function invoke(callback) { return callback(); } function make() { let x = 1; return () => x; } let callback = make(); invoke(callback);";
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let script = program.script.as_ref().expect("script ir should exist");
    let callback_arg = script.body.statements.iter().find_map(|statement| {
        let StatementIr::Expression(TypedExpr {
            expr: ExprIr::CallIndirect { args, .. },
            ..
        }) = statement
        else {
            return None;
        };
        args.first()
    });
    assert!(matches!(
        callback_arg,
        Some(TypedExpr {
            expr: ExprIr::Identifier(name),
            ..
        }) if name == "callback"
    ));
}

#[test]
fn array_callback_exact_context_preserves_this_argument_shape() {
    let source = "['source', 'flags'].forEach(function (key) { Object.defineProperty(this, key, { value: '' }); }, this);";
    let program = lower_script(source);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
}

#[test]
fn base_arrow_does_not_capture_derived_activation() {
    let program = lower_script("class B { constructor() { (() => this)(); } }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().unwrap();
    let arrow = script
        .functions
        .iter()
        .find(|function| function.protocol.flavor() == FunctionFlavor::Arrow)
        .unwrap();
    assert!(!arrow.uses_super);
    assert!(arrow.lexical_derived_activation.is_none());
    assert!(!arrow
        .captured_bindings
        .iter()
        .any(|binding| binding.name == DERIVED_ACTIVATION_FUNCTION_NAME));
}

#[test]
fn ordinary_function_is_a_lexical_super_boundary() {
    let source = "class A {} class B extends A { constructor() { (function () { return () => super(); })(); } }";
    assert!(parse(source, ParseOptions::script()).is_err());
}

#[test]
fn infers_array_kind_for_direct_and_nested_default_subclasses() {
    for source in [
        "class Ar extends Array {} new Ar();",
        "class A extends Array {} class B extends A {} new B();",
    ] {
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script ir should exist");
        let StatementIr::Expression(instance) = script.body.statements.last().unwrap() else {
            panic!("expected constructed instance");
        };
        assert_eq!(instance.kind, ValueKind::Array, "{source}");
        assert!(matches!(
            instance.heap_shape.as_deref(),
            Some(HeapShape::Array(_))
        ));
    }

    let ordinary = lower_script("class C {} new C();");
    let script = ordinary.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(instance) = script.body.statements.last().unwrap() else {
        panic!("expected constructed instance");
    };
    assert_eq!(instance.kind, ValueKind::Object);
    assert!(matches!(
        instance.heap_shape.as_deref(),
        Some(HeapShape::Object(_))
    ));
}

#[test]
fn object_create_does_not_infer_a_null_prototype_from_a_mixed_prototype() {
    let program =
        lower_script("const create = Object.create; var prototype = globalThis.flag ? null : {}; create(prototype);");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(instance) = script.body.statements.last().unwrap() else {
        panic!("expected Object.create result");
    };
    assert_eq!(instance.kind, ValueKind::Object);
    assert!(instance.heap_shape.is_none());
}

#[test]
fn preserves_private_brand_shape_for_array_subclass_instances() {
    let program =
        lower_script("class A extends Array { #x; has() { return #x in this; } } new A();");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
        panic!("expected class declaration");
    };
    let ExprIr::ClassDefinition(class) = &init.expr else {
        panic!("expected class definition");
    };
    let private_name_id = *class
        .private_name_ids
        .get("x")
        .expect("private brand should be assigned");
    let StatementIr::Expression(instance) = script.body.statements.last().unwrap() else {
        panic!("expected constructed instance");
    };
    let HeapShape::Array(shape) = instance.heap_shape.as_deref().expect("array shape") else {
        panic!("expected ArrayShape");
    };
    assert!(shape
        .properties
        .contains_key(&private_brand_key(private_name_id)));
}

#[test]
fn private_in_rhs_preserves_shift_precedence_and_runtime_global_resolution() {
    for (source, expected_rhs) in [
        (
            "class C { #field; probe() { try { #field in {} << 0; } catch (error) {} } }",
            "shift",
        ),
        (
            "class C { #field; probe() { try { #field in missingName; } catch (error) {} } }",
            "global-resolution",
        ),
    ] {
        let program = lower_script(source);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let probe = script
            .functions
            .iter()
            .find(|function| function.name == "C.probe")
            .expect("private-in probe should be lowered");
        let StatementIr::TryCatch { try_block, .. } = &probe.body.statements[0] else {
            panic!("expected private-in try/catch");
        };
        let StatementIr::Expression(TypedExpr {
            expr: ExprIr::PrivateIn { rhs, .. },
            ..
        }) = &try_block.statements[0]
        else {
            panic!("expected private-in expression");
        };

        match expected_rhs {
            "shift" => assert!(matches!(
                &rhs.expr,
                ExprIr::BitwiseNumeric {
                    op: BitwiseBinaryOp::Shl,
                    ..
                }
            )),
            "global-resolution" => assert!(matches!(
                &rhs.expr,
                ExprIr::GlobalIdentifierRead { name } if name == "missingName"
            )),
            _ => unreachable!(),
        }
    }
}

#[test]
fn private_in_captured_error_widens_the_enclosing_binding() {
    let program = lower_script(
        "let caught = null;
             class C {
                 #field;
                 constructor() {
                     try { #field in 0; } catch (error) { caught = error; }
                 }
             }
             new C();
             caught.constructor;",
    );

    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
}

#[test]
fn same_spelling_in_distinct_nested_classes_has_distinct_private_identity() {
    let program = lower_script(
        "function first() { return class { #value; }; }
             function second() { return class { #value; }; }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let private_name_id = |function_name: &str| {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == function_name)
            .unwrap_or_else(|| panic!("{function_name} should be lowered"));
        let class = function
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Return(TypedExpr {
                    expr: ExprIr::ClassDefinition(class),
                    ..
                }) => Some(class),
                _ => None,
            })
            .unwrap_or_else(|| panic!("{function_name} should return a class"));
        *class
            .private_name_ids
            .get("value")
            .unwrap_or_else(|| panic!("{function_name} should declare #value"))
    };

    assert_ne!(private_name_id("first"), private_name_id("second"));
}

#[test]
fn resolves_private_names_through_nested_function_and_class_boundaries() {
    let program = lower_script(
        "class Outer {
                #value;
                make() {
                    function nested(receiver) { return receiver.#value; }
                    return class Inner extends (function Heritage(receiver) {
                        return receiver.#value;
                    }) {
                        #value;
                        read(receiver) { return receiver.#value; }
                    };
                }
            }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
        panic!("expected outer class declaration");
    };
    let ExprIr::ClassDefinition(outer_class) = &init.expr else {
        panic!("expected outer class definition");
    };
    let outer_private_name_id = outer_class.private_name_ids["value"];

    let make = script
        .functions
        .iter()
        .find(|function| function.name == "Outer.make")
        .expect("outer method should be lowered");
    let inner_class = make
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Return(TypedExpr {
                expr: ExprIr::ClassDefinition(class),
                ..
            }) => Some(class),
            _ => None,
        })
        .expect("outer method should return the inner class");
    let inner_private_name_id = inner_class.private_name_ids["value"];
    assert_ne!(inner_private_name_id, outer_private_name_id);

    let returned_private_name_id = |function_name: &str| {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == function_name)
            .unwrap_or_else(|| panic!("function `{function_name}` should be lowered"));
        function
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Return(TypedExpr {
                    expr:
                        ExprIr::PrivateRead {
                            private_name_id, ..
                        },
                    ..
                }) => Some(*private_name_id),
                _ => None,
            })
            .unwrap_or_else(|| panic!("function `{function_name}` should read a private name"))
    };

    assert_eq!(returned_private_name_id("nested"), outer_private_name_id);
    assert_eq!(returned_private_name_id("Heritage"), outer_private_name_id);
    assert_eq!(
        returned_private_name_id("Inner.read"),
        inner_private_name_id
    );
}

#[test]
fn resolves_shadowed_private_names_in_a_nested_class_field_initializer() {
    let program = lower_script(
        "class Outer {
                set #value(next) {}
                field = class Inner {
                    #value;
                    write(receiver, next) { receiver.#value = next; }
                    read() { return this.#value; }
                };
            }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");

    let outer_setter = script
        .functions
        .iter()
        .find(|function| function.name == "set #value")
        .expect("outer setter should be lowered");
    let write = script
        .functions
        .iter()
        .find(|function| function.name == "Inner.write")
        .expect("inner writer should be lowered");
    let read = script
        .functions
        .iter()
        .find(|function| function.name == "Inner.read")
        .expect("inner reader should be lowered");

    let write_private_name_id = write
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Expression(TypedExpr {
                expr:
                    ExprIr::PrivateWrite {
                        private_name_id, ..
                    },
                ..
            }) => Some(*private_name_id),
            _ => None,
        })
        .expect("inner writer should write a private name");
    let read_private_name_id = read
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Return(TypedExpr {
                expr:
                    ExprIr::PrivateRead {
                        private_name_id, ..
                    },
                ..
            }) => Some(*private_name_id),
            _ => None,
        })
        .expect("inner reader should read a private name");

    assert_eq!(write_private_name_id, read_private_name_id);
    assert_eq!(write.private_name_ids["value"], write_private_name_id);
    assert_eq!(read.private_name_ids["value"], read_private_name_id);
    assert_ne!(outer_setter.private_name_ids["value"], read_private_name_id);
    assert_eq!(read.return_kind, ValueKind::Dynamic);
}

#[test]
fn resolves_private_names_in_field_initializers_and_static_blocks() {
    let program = lower_script(
        "class C {
                #instance;
                field = this.#instance;
                static #staticValue;
                static field = this.#staticValue;
                static { this.#staticValue; }
            }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
        panic!("expected class declaration");
    };
    let ExprIr::ClassDefinition(class) = &init.expr else {
        panic!("expected class definition");
    };

    let private_read_id = |execution_kind: ClassElementExecutionKind| {
        let function = script
            .functions
            .iter()
            .find(|function| function.class_element_execution_kind == execution_kind)
            .unwrap_or_else(|| panic!("{execution_kind:?} should be lowered"));
        function
            .body
            .statements
            .iter()
            .find_map(|statement| {
                let expression = match statement {
                    StatementIr::Expression(expression) | StatementIr::Return(expression) => {
                        expression
                    }
                    _ => return None,
                };
                match &expression.expr {
                    ExprIr::PrivateRead {
                        private_name_id, ..
                    } => Some(*private_name_id),
                    _ => None,
                }
            })
            .unwrap_or_else(|| panic!("{execution_kind:?} should read a private name"))
    };

    assert_eq!(
        private_read_id(ClassElementExecutionKind::InstanceFieldInitializer),
        class.private_name_ids["instance"]
    );
    assert_eq!(
        private_read_id(ClassElementExecutionKind::StaticFieldInitializer),
        class.private_name_ids["staticValue"]
    );
    assert_eq!(
        private_read_id(ClassElementExecutionKind::StaticBlock),
        class.private_name_ids["staticValue"]
    );
}

#[test]
fn class_declarations_have_mutable_lexical_bindings() {
    let program = lower_script("class C {} C = 1;");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    assert!(matches!(
        script.body.statements[0],
        StatementIr::Lexical {
            mode: BindingMode::Let,
            ..
        }
    ));
}

#[test]
fn script_level_class_names_join_the_global_lexical_plan() {
    // `GlobalDeclarationInstantiation` collision checks (notably a later
    // `$262.evalScript('let C;')`) read the plan, so a script-level class
    // name must be there as a mutable lexical with an owned cell, exactly
    // like a `let`.
    let program = lower_script("class C {}");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    assert_eq!(
        script.global_bindings.lexical_bindings().get("C"),
        Some(&GlobalLexicalBindingModeIr::Mutable)
    );
    assert!(
        script
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == "C"),
        "the global lexical entry needs its analyzed cell"
    );
}

#[test]
fn class_setters_capture_the_immutable_inner_name_binding() {
    let program = lower_script("var C2; class C { set value(next) { C2 = C; } }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Lexical { init, .. } = &script.body.statements[1] else {
        panic!("expected class declaration");
    };
    let ExprIr::ClassDefinition(class) = &init.expr else {
        panic!("expected class definition");
    };
    let name_binding = class
        .name_binding
        .as_ref()
        .expect("named class should own an inner name binding");
    let setter = script
        .functions
        .iter()
        .find(|function| function.protocol.class_kind() == ClassFunctionKind::Setter)
        .expect("class setter should be lowered");

    assert_eq!(name_binding.environment.bindings.len(), 1);
    assert_eq!(setter.captured_bindings.len(), 1);
    let capture = &setter.captured_bindings[0];
    assert_eq!(capture.source_name, "C");
    assert_eq!(capture.name, name_binding.storage_name);
    assert_eq!(capture.mode, BindingMode::Const);
}

#[test]
fn class_shape_retains_paired_getter_and_setter() {
    let program = lower_script("class C { get value() { return C; } set value(next) {} }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
        panic!("expected class declaration");
    };
    let Some(HeapShape::Object(class_shape)) = init.heap_shape.as_deref() else {
        panic!("expected class shape");
    };
    let Some(ObjectShapeProperty::Data(prototype)) = class_shape.properties.get("prototype") else {
        panic!("expected class prototype");
    };
    let Some(HeapShape::Object(prototype_shape)) = prototype.heap_shape.as_deref() else {
        panic!("expected prototype shape");
    };

    assert!(matches!(
        prototype_shape.properties.get("value"),
        Some(ObjectShapeProperty::Accessor {
            getter: Some(_),
            setter: Some(_),
        })
    ));
}

#[test]
fn array_subclass_overrides_lower_as_runtime_property_calls() {
    let program = lower_script(
        "class A extends Array {
                push() { return 'custom push'; }
                join() { return 'custom join'; }
             }
             const a = new A();
             a.push(1);
             a.join();",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");

    for (statement, expected_key) in script
        .body
        .statements
        .iter()
        .rev()
        .take(2)
        .rev()
        .zip(["push", "join"])
    {
        let StatementIr::Expression(expression) = statement else {
            panic!("expected runtime indirect call for {expected_key}: {statement:?}");
        };
        assert_eq!(
            expression.kind,
            ValueKind::String,
            "expected the {expected_key} override result, got {expression:?}"
        );
        let Some(TypedExpr {
            expr:
                ExprIr::CallIndirect {
                    callee,
                    this_arg: Some(this_arg),
                    ..
                },
            ..
        }) = indirect_call_body(expression)
        else {
            panic!("expected runtime indirect call for {expected_key}: {statement:?}");
        };
        assert!(matches!(
            &callee.expr,
            ExprIr::SpecOperation {
                operation: SpecOperationIr::GetV,
                operands,
            } if operands.len() == 2
                && matches!(&operands[1].expr, ExprIr::String(key) if key == expected_key)
        ));
        assert!(matches!(
            this_arg.heap_shape.as_deref(),
            Some(HeapShape::Array(shape)) if shape.prototype.is_some()
        ));
    }
}

#[test]
fn array_subclass_method_write_invalidates_a_later_override_result() {
    let program = lower_script(
        "class A extends Array {
                replaceJoin() { this.join = function replacement() { return 1; }; }
                join() { return 'custom join'; }
             }
             const a = new A();
             a.replaceJoin();
             a.join();",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(call) = script.body.statements.last().unwrap() else {
        panic!("expected the post-mutation join call");
    };

    assert_eq!(call.kind, ValueKind::Dynamic, "{call:?}");
}

#[test]
fn private_method_call_materializes_receiver_before_brand_checked_read() {
    let program = lower_script(
        "class A extends Array {
                #method() { return 1; }
                call() { return this.#method(); }
             }
             new A().call();",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let function = script
        .functions
        .iter()
        .find(|function| function.name == "A.call")
        .expect("class method should be lowered");
    let StatementIr::Return(TypedExpr {
        expr: ExprIr::MaterializeBinding { name, value, body },
        ..
    }) = &function.body.statements[0]
    else {
        panic!(
            "expected materialized private method call: {:?}",
            function.body
        );
    };
    assert!(matches!(value.expr, ExprIr::This));
    let ExprIr::CallIndirect {
        callee,
        this_arg: Some(this_arg),
        ..
    } = &body.expr
    else {
        panic!("expected indirect private method call body: {body:?}");
    };
    let ExprIr::PrivateRead { target, .. } = &callee.expr else {
        panic!("expected brand-checked private read: {callee:?}");
    };
    assert!(matches!(
        &target.expr,
        ExprIr::Identifier(target_name) if target_name == name
    ));
    assert!(matches!(
        &this_arg.expr,
        ExprIr::Identifier(this_name) if this_name == name
    ));
}

#[test]
fn ordinary_method_call_materializes_compound_receiver_before_reference_get() {
    let program =
        lower_script("function make() { return { method() { return this; } }; } make().method();");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(TypedExpr {
        expr: ExprIr::MaterializeBinding { name, value, body },
        ..
    }) = script
        .body
        .statements
        .last()
        .expect("method call statement should exist")
    else {
        panic!("expected materialized ordinary method call");
    };
    assert!(matches!(
        value.expr,
        ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
    ));
    let ExprIr::CallIndirect {
        callee,
        this_arg: Some(this_arg),
        ..
    } = &body.expr
    else {
        panic!("expected indirect method call body: {body:?}");
    };
    // A proven shaped receiver uses the typed PropertyRead carrier. Once
    // flow analysis has conservatively widened that shape, the same
    // Reference get is represented by the canonical GetV operation. Both
    // must consume the one materialized base before Call supplies `this`.
    let target = match &callee.expr {
        ExprIr::PropertyRead { target, .. } => target.as_ref(),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::GetV,
            operands,
        } => operands
            .first()
            .expect("GetV method reference should retain its base"),
        _ => panic!("expected property reference get callee: {callee:?}"),
    };
    assert!(matches!(
        &target.expr,
        ExprIr::Identifier(target_name) if target_name == name
    ));
    assert!(matches!(
        &this_arg.expr,
        ExprIr::Identifier(this_name) if this_name == name
    ));
}

#[test]
fn computed_method_call_materializes_base_before_key_evaluation() {
    let program = lower_script(
            "function make() { return { method() { return this; } }; } function key() { return 'method'; } make()[key()]();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(TypedExpr {
        expr: ExprIr::MaterializeBinding { name, body, .. },
        ..
    }) = script
        .body
        .statements
        .last()
        .expect("computed method call statement should exist")
    else {
        panic!("expected materialized computed method call");
    };
    let ExprIr::CallIndirect {
        callee,
        this_arg: Some(this_arg),
        ..
    } = &body.expr
    else {
        panic!("expected indirect computed method call body: {body:?}");
    };
    let ExprIr::SpecOperation {
        operation: SpecOperationIr::GetV,
        operands,
    } = &callee.expr
    else {
        panic!("expected GetV callee: {callee:?}");
    };
    assert!(matches!(
        &operands[0].expr,
        ExprIr::Identifier(target_name) if target_name == name
    ));
    assert!(matches!(
        &this_arg.expr,
        ExprIr::Identifier(this_name) if this_name == name
    ));
    assert!(matches!(
        operands[1].expr,
        ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
    ));
}

#[test]
fn carries_dynamic_class_heritage_to_runtime() {
    let program =
        lower_script("function make(Base) { class Derived extends Base {} return Derived; }");
    assert!(program.is_wasm_supported());
    let summary = program.ir_summary();
    assert!(summary.contains("class_extends=1"));
}

#[test]
fn narrows_identifier_used_as_object_property_key_to_string() {
    let program =
        lower_script("function get(obj, name) { return obj[name]; } get({ x: 1 }, \"x\");");
    assert!(program.is_wasm_supported());
    let summary = program.ir_summary();
    assert!(summary.contains("property_reads=2"));
}

#[test]
fn keeps_dynamic_string_property_key_on_possible_array_targets() {
    let program = lower_script(
            "function read(desc, name) { return desc.get[name]; } read({ get: function () {} }, \"length\");",
        );
    assert!(program.is_wasm_supported());
    let summary = program.ir_summary();
    assert!(summary.contains("property_reads=4"));
}

#[test]
fn treats_typed_array_constructor_parameter_bytes_per_element_as_number() {
    let program = lower_script("function f(TA) { return 4 * TA.BYTES_PER_ELEMENT; } f(Int8Array);");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    assert_eq!(script.result_kind(), ValueKind::Number);
}

#[test]
fn class_owned_functions_keep_unique_ids_across_helper_and_callback_contexts() {
    let program = lower_script(
        r#"
function make(value) {
  return class Same {
    field = value;
    static field = value;
    accessor item = value;
    #private = value;
    read() { return this.#private; }
    static read() { return this.field; }
  };
}
function invoke(callback, value) { return callback(value); }
const First = make(1);
const Second = make('second');
const callback = value => class Same { field = value; accessor item = value; };
const Third = invoke(callback, 3);
const Fourth = invoke(callback, 'fourth');
new First().field;
new Second().field;
new Third().item;
new Fourth().item;
"#,
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR");
    let mut ids = BTreeSet::new();
    let mut repeated_names = BTreeMap::<&str, usize>::new();
    for function in &script.functions {
        assert!(
            ids.insert(&function.id),
            "duplicate callable: {}",
            function.id
        );
        *repeated_names.entry(&function.name).or_default() += 1;
    }
    assert!(
        repeated_names.values().any(|count| *count > 1),
        "same displayed class member names still have distinct execution identities"
    );
    assert!(script.functions.iter().any(|function| {
        function.class_element_execution_kind == ClassElementExecutionKind::InstanceFieldInitializer
    }));
    assert!(script.functions.iter().any(|function| {
        function.class_element_execution_kind == ClassElementExecutionKind::StaticFieldInitializer
    }));
}

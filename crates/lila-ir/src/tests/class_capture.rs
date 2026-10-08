#[test]
fn captured_function_does_not_publish_a_stale_intrinsic_return_target() {
    let program = lower_script(
        "const rab = new ArrayBuffer(64, { maxByteLength: 1024 });
             const f = () => rab.resize;
             rab.resize = function replacement() {};
             f();",
    );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let intrinsic = StandardBuiltinId::ArrayBufferPrototypeResize.function_id();
    let arrow = script
        .functions
        .iter()
        .find(|function| function.protocol.flavor() == FunctionFlavor::Arrow)
        .expect("arrow function should be lowered");
    let Some(StatementIr::Return(expr)) = arrow.body.statements.first() else {
        panic!("expression arrow should lower to return");
    };
    assert!(!expr.function_targets.known_targets().contains(&intrinsic));
    assert!(!arrow.return_targets.known_targets().contains(&intrinsic));
    let StatementIr::Expression(call) = script.body.statements.last().unwrap() else {
        panic!("final statement should be the closure call");
    };
    assert!(!call.function_targets.known_targets().contains(&intrinsic));
}

#[test]
fn inherited_typed_array_to_string_keeps_its_possible_native_target_and_live_read() {
    let program = lower_script("Uint8Array.prototype.toString;");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Expression(expr) = script.body.statements.last().unwrap() else {
        panic!("property read should remain the script result");
    };
    assert!(
        expr.function_targets
            .known_targets()
            .contains(&StandardBuiltinId::TypedArrayPrototypeToString.function_id()),
        "unexpected property expression: {expr:?}"
    );
    assert!(expr.function_targets.exact_targets().is_none());
    assert!(matches!(&expr.expr, ExprIr::PropertyRead {
        key: PropertyKeyIr::StaticString(key), ..
    } if key == "toString"));
}

#[test]
fn lowers_script_class_closure_capture() {
    let program = lower_script("class C {} function f() { return C; } f();");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    assert!(script
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == "C"));
    assert!(script.functions.iter().any(|function| {
        function
            .captured_bindings
            .iter()
            .any(|binding| binding.name == "C")
    }));
}

#[test]
fn lowers_class_constructor_closure_capture() {
    let program = lower_script(
            "function outer(Base, args) { let called = 0; class Derived extends Base { constructor() { ++called; super(...args); } } return Derived; }",
        );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let constructor = script
        .functions
        .iter()
        .find(|function| function.name == "Derived")
        .expect("derived constructor should be lowered");
    assert!(constructor
        .captured_bindings
        .iter()
        .any(|binding| binding.name == "called"));
    assert!(constructor
        .captured_bindings
        .iter()
        .any(|binding| binding.name == "args"));
}

#[test]
fn class_members_preserve_scoped_capture_source_names() {
    for (source, member_name) in [
            (
                "function owner() { let x = \"outer\"; { let x = 2; class C { m() { return x + 3; } } x = \"later\"; return new C().m(); } } owner();",
                "C.m",
            ),
            (
                "function owner() { let x = \"outer\"; { let x = 2; class C { constructor() { this.value = x + 3; } } x = \"later\"; return new C().value; } } owner();",
                "C",
            ),
        ] {
            let program = lower_script(source);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.as_ref().expect("script IR should exist");
            let owner = script
                .functions
                .iter()
                .find(|function| function.name == "owner")
                .expect("owner function should be lowered");
            let member = script
                .functions
                .iter()
                .find(|function| function.name == member_name)
                .expect("class member should be lowered");
            let capture = member
                .captured_bindings
                .iter()
                .find(|binding| binding.source_name == "x")
                .expect("class member should capture the scoped binding");

            assert!(capture.name.starts_with("$scoped.lex."));
            assert_eq!(capture.mode, BindingMode::Let);
            assert_eq!(capture.hops, 1);
            assert!(
                !owner
                    .owned_env_bindings
                    .iter()
                    .any(|binding| binding.name == capture.name)
            );
            assert!(block_environment_owns_binding(
                &owner.body,
                &capture.name,
                capture.slot
            ));
            if member_name == "C.m" {
                assert_eq!(member.return_kind, ValueKind::Dynamic);
            } else {
                assert!(member.body.statements.iter().any(|statement| {
                    matches!(
                        statement,
                        StatementIr::Expression(TypedExpr {
                            expr: ExprIr::OrdinaryPropertyAssignment(assignment),
                            ..
                        }) if assignment.rhs().kind == ValueKind::Dynamic
                    )
                }));
            }
        }
}

#[test]
fn class_members_retain_transitive_root_function_captures() {
    for (source, member_name) in [
            (
                "function owner() { let x = \"outer\"; { let x = 2; class C { m() { function inner() { return x + 3; } return inner(); } } return new C().m(); } } owner();",
                "C.m",
            ),
            (
                "function owner() { let x = \"outer\"; { let x = 2; class C { constructor() { function inner() { return x + 3; } this.value = inner(); } } return new C().value; } } owner();",
                "C",
            ),
        ] {
            let program = lower_script(source);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.as_ref().expect("script IR should exist");
            let member = script
                .functions
                .iter()
                .find(|function| function.name == member_name)
                .expect("class member should be lowered");
            let member_capture = member
                .captured_bindings
                .iter()
                .find(|binding| binding.source_name == "x")
                .expect("class member should retain the nested root capture");
            let root_function = script
                .functions
                .iter()
                .find(|function| function.name == "inner")
                .expect("nested root function should be lowered");
            let root_capture = root_function
                .captured_bindings
                .iter()
                .find(|binding| binding.source_name == "x")
                .expect("nested root function should capture the scoped binding");

            assert!(member_capture.name.starts_with("$scoped.lex."));
            assert_eq!(member_capture.name, root_capture.name);
            assert_eq!(member_capture.slot, root_capture.slot);
            assert_eq!(member_capture.hops, 1);
            assert_eq!(root_capture.hops, 1);
            assert_eq!(root_function.return_kind, ValueKind::Dynamic);
        }
}

#[test]
fn instance_field_initializer_retains_transitive_block_capture_from_class_definition() {
    let program = lower_script(
            "function owner() { { const blockValue = 7; class C { value = (() => blockValue)(); } return C; } } owner();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let field = script
        .functions
        .iter()
        .find(|function| function.name == "C.field.value")
        .expect("field initializer should be lowered");
    let constructor = script
        .functions
        .iter()
        .find(|function| function.name == "C")
        .expect("default constructor should be lowered");
    let arrow = script
        .functions
        .iter()
        .find(|function| function.protocol.flavor() == FunctionFlavor::Arrow)
        .expect("nested arrow should be lowered");
    let field_capture = field
        .captured_bindings
        .iter()
        .find(|binding| binding.source_name == "blockValue")
        .expect("field initializer should retain the nested arrow capture");
    let arrow_capture = arrow
        .captured_bindings
        .iter()
        .find(|binding| binding.source_name == "blockValue")
        .expect("nested arrow should capture the block binding");

    assert_eq!(field_capture.mode, BindingMode::Const);
    assert_eq!(field_capture.name, arrow_capture.name);
    assert_eq!(field_capture.slot, arrow_capture.slot);
    assert!(!constructor
        .captured_bindings
        .iter()
        .any(|binding| binding.source_name == "blockValue"));
}

#[test]
fn instance_field_capture_uses_the_class_definition_environment() {
    let program = lower_script(
            "function owner() { let x = 7; class C { constructor() { let local = 1; (() => local)(); } value = x; } x = \"later\"; return C; } owner();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let field = script
        .functions
        .iter()
        .find(|function| function.name == "C.field.value")
        .expect("field initializer should be lowered");
    let constructor = script
        .functions
        .iter()
        .find(|function| function.name == "C")
        .expect("explicit constructor should be lowered");
    let field_capture = field
        .captured_bindings
        .iter()
        .find(|binding| binding.source_name == "x")
        .expect("field initializer should capture the outer binding");
    assert!(constructor
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == "local"));
    assert_eq!(field_capture.mode, BindingMode::Let);
    assert_eq!(field_capture.hops, 1);
    assert_eq!(field.return_kind, ValueKind::Dynamic);
    assert!(!constructor
        .captured_bindings
        .iter()
        .any(|binding| binding.source_name == "x"));
}

#[test]
fn static_field_initializer_captures_switch_environment_binding() {
    let program = lower_script(
        "switch (0) { case 0: let switchValue = 3; class C { static value = switchValue; } }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let field = script
        .functions
        .iter()
        .find(|function| function.name == "C.field.value")
        .expect("static field initializer should be lowered");
    let capture = field
        .captured_bindings
        .iter()
        .find(|binding| binding.source_name == "switchValue")
        .expect("static field should capture the switch binding");

    assert_eq!(capture.mode, BindingMode::Let);
    assert_ne!(capture.name, capture.source_name);
    assert_eq!(field.return_kind, ValueKind::Number);
}

#[test]
fn static_block_retains_nested_function_capture_from_catch_environment() {
    let program = lower_script(
            "try { throw 1; } catch (caught) { class C { static { function readCaught() { return caught; } this.value = readCaught(); } } }",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let static_block = script
        .functions
        .iter()
        .find(|function| function.name == "C.<static>")
        .expect("static block should be lowered");
    let nested = script
        .functions
        .iter()
        .find(|function| function.name == "readCaught")
        .expect("nested function should be lowered");
    let static_capture = static_block
        .captured_bindings
        .iter()
        .find(|binding| binding.source_name == "caught")
        .expect("static block should retain the nested function capture");
    let nested_capture = nested
        .captured_bindings
        .iter()
        .find(|binding| binding.source_name == "caught")
        .expect("nested function should capture the catch binding");

    assert_eq!(static_capture.mode, BindingMode::Let);
    assert_eq!(static_capture.name, nested_capture.name);
    assert_eq!(static_capture.slot, nested_capture.slot);
}

#[test]
fn class_execution_ids_are_distinct_for_multiple_fields_and_static_blocks() {
    let program = lower_script("class C { first = 1; second = 2; static {} static {} }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let class = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical {
                init:
                    TypedExpr {
                        expr: ExprIr::ClassDefinition(class),
                        ..
                    },
                ..
            } => Some(class.as_ref()),
            _ => None,
        })
        .expect("class definition should be lowered");
    let constructor = script
        .functions
        .iter()
        .find(|function| function.id == class.constructor_function_id)
        .expect("class constructor should be lowered");
    let instance_plan = constructor
        .class_instance_element_plan
        .as_ref()
        .expect("constructor should own its instance element plan");
    let execution_ids = instance_plan
        .elements
        .iter()
        .filter_map(|element| match element {
            ClassInstanceElementIr::Field(field) => field.init_function_id.clone(),
            ClassInstanceElementIr::AutoAccessorBacking(accessor) => {
                accessor.init_function_id.clone()
            }
        })
        .chain(
            class
                .element_plan
                .static_elements
                .iter()
                .filter_map(|element| match element {
                    ClassStaticElementIr::Field(field) => field.init_function_id.clone(),
                    ClassStaticElementIr::AutoAccessorBacking(accessor) => {
                        accessor.init_function_id.clone()
                    }
                    ClassStaticElementIr::Block(block) => Some(block.function_id.clone()),
                }),
        )
        .collect::<BTreeSet<_>>();

    assert_eq!(instance_plan.elements.len(), 2);
    assert_eq!(class.element_plan.static_elements.len(), 2);
    assert_eq!(execution_ids.len(), 4);
}

#[test]
fn class_element_plan_preserves_definition_and_field_source_order() {
    let program = lower_script(
        "function first() { return 'first'; }
             function second() { return 'second'; }
             function third() { return 'third'; }
             class C {
                 [first()]() {}
                 #method() {}
                 static publicField = 1;
                 static {}
                 static #privateField = 2;
                 get [second()]() {}
                 set [third()](value) {}
                 publicInstance = 3;
                 #privateInstance = 4;
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let class = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical {
                init:
                    TypedExpr {
                        expr: ExprIr::ClassDefinition(class),
                        ..
                    },
                ..
            } => Some(class.as_ref()),
            _ => None,
        })
        .expect("class definition should be lowered");

    assert!(matches!(
        class.element_plan.definitions.as_slice(),
        [
            ClassElementDefinitionIr::PublicMethod(first),
            ClassElementDefinitionIr::PrivateMethod(_),
            ClassElementDefinitionIr::PublicMethod(second),
            ClassElementDefinitionIr::PublicMethod(third),
        ] if first.kind == ClassMethodKindIr::Method
            && matches!(&first.key, PropertyKeyIr::StringExpr(_))
            && second.kind == ClassMethodKindIr::Getter
            && matches!(&second.key, PropertyKeyIr::StringExpr(_))
            && third.kind == ClassMethodKindIr::Setter
            && matches!(&third.key, PropertyKeyIr::StringExpr(_))
    ));
    assert!(matches!(
        class.element_plan.static_elements.as_slice(),
        [
            ClassStaticElementIr::Field(public),
            ClassStaticElementIr::Block(_),
            ClassStaticElementIr::Field(private),
        ] if matches!(&public.key, ClassFieldKeyIr::Public(key) if key == "publicField")
            && matches!(&private.key, ClassFieldKeyIr::Private(_))
    ));
    let constructor = script
        .functions
        .iter()
        .find(|function| function.id == class.constructor_function_id)
        .expect("class constructor should be lowered");
    let instance_plan = constructor
        .class_instance_element_plan
        .as_ref()
        .expect("constructor should own its instance element plan");
    assert!(matches!(
        instance_plan.elements.as_slice(),
        [ClassInstanceElementIr::Field(public), ClassInstanceElementIr::Field(private)]
            if matches!(&public.key, ClassFieldKeyIr::Public(key) if key == "publicInstance")
                && matches!(&private.key, ClassFieldKeyIr::Private(_))
    ));
    assert_eq!(instance_plan.private_method_brands.len(), 1);
}

#[test]
fn class_auto_accessors_have_distinct_exposed_and_backing_plans() {
    let program = lower_script(
        "class C {
                accessor publicValue = 1;
                static accessor staticValue;
                accessor #privateValue = 2;
                static accessor #privateStatic = 3;
            }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let class = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical {
                init:
                    TypedExpr {
                        expr: ExprIr::ClassDefinition(class),
                        ..
                    },
                ..
            } => Some(class.as_ref()),
            _ => None,
        })
        .expect("class definition should be lowered");
    let private_environment = class
        .private_environment
        .expect("auto-accessors require a class private environment");
    assert_eq!(class.private_name_ids.len(), 2);
    assert_eq!(private_environment.slot_count(), 6);

    let accessors = class
        .element_plan
        .definitions
        .iter()
        .filter_map(|definition| match definition {
            ClassElementDefinitionIr::AutoAccessor(accessor) => Some(accessor),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(accessors.len(), 4);
    let backing_names = accessors
        .iter()
        .map(|accessor| accessor.backing_name.private_name_id())
        .collect::<BTreeSet<_>>();
    assert_eq!(backing_names.len(), 4);
    assert!(backing_names.iter().all(|backing| {
        !class
            .private_name_ids
            .values()
            .any(|visible| visible == backing)
    }));
    for accessor in accessors {
        let getter = script
            .functions
            .iter()
            .find(|function| &function.id == accessor.functions.getter())
            .expect("generated getter should exist");
        let setter = script
            .functions
            .iter()
            .find(|function| &function.id == accessor.functions.setter())
            .expect("generated setter should exist");
        assert_eq!(getter.protocol, FunctionProtocolIr::ClassGetter);
        assert_eq!(setter.protocol, FunctionProtocolIr::ClassSetter);
        assert!(getter.strict && setter.strict);
        assert!(getter.params.is_empty());
        assert_eq!(setter.params.len(), 1);
    }

    let constructor = script
        .functions
        .iter()
        .find(|function| function.id == class.constructor_function_id)
        .expect("class constructor should be lowered");
    let instance_plan = constructor
        .class_instance_element_plan
        .as_ref()
        .expect("instance backing initializers should be planned");
    assert_eq!(instance_plan.elements.len(), 2);
    assert_eq!(class.element_plan.static_elements.len(), 2);
}

#[test]
fn computed_class_field_keys_use_definition_order_cache_slots() {
    let program = lower_script(
        "function first() { return 'instance'; }
             function second() { return 'static'; }
             class C { [first()] = 1; static [second()] = 2; }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let class = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical {
                init:
                    TypedExpr {
                        expr: ExprIr::ClassDefinition(class),
                        ..
                    },
                ..
            } => Some(class.as_ref()),
            _ => None,
        })
        .expect("class definition should be lowered");
    assert!(matches!(
        class.element_plan.definitions.as_slice(),
        [
            ClassElementDefinitionIr::ComputedFieldKey { slot: 0, .. },
            ClassElementDefinitionIr::ComputedFieldKey { slot: 1, .. },
        ]
    ));
    let constructor = script
        .functions
        .iter()
        .find(|function| function.id == class.constructor_function_id)
        .expect("class constructor should be lowered");
    let instance_plan = constructor
        .class_instance_element_plan
        .as_ref()
        .expect("constructor should own its instance field plan");
    assert!(matches!(
        instance_plan.elements.as_slice(),
        [ClassInstanceElementIr::Field(ClassFieldInitIr {
            key: ClassFieldKeyIr::ComputedPublic(0),
            ..
        })]
    ));
    assert!(matches!(
        class.element_plan.static_elements.as_slice(),
        [ClassStaticElementIr::Field(ClassFieldInitIr {
            key: ClassFieldKeyIr::ComputedPublic(1),
            ..
        })]
    ));
}

#[test]
fn computed_class_field_key_preserves_nested_runtime_global_resolution() {
    let program =
        lower_script("function evaluate() { class C { [missingComputedName] = 1; } } evaluate();");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let evaluate = script
        .functions
        .iter()
        .find(|function| function.name == "evaluate")
        .expect("evaluate function should be lowered");
    let class = evaluate
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical {
                init:
                    TypedExpr {
                        expr: ExprIr::ClassDefinition(class),
                        ..
                    },
                ..
            } => Some(class.as_ref()),
            _ => None,
        })
        .expect("class definition should be lowered");
    let [ClassElementDefinitionIr::ComputedFieldKey {
        key: PropertyKeyIr::StringExpr(key),
        ..
    }] = class.element_plan.definitions.as_slice()
    else {
        panic!("expected one computed field key");
    };
    assert!(matches!(
        &key.expr,
        ExprIr::GlobalIdentifierRead { name } if name == "missingComputedName"
    ));
}

#[test]
fn later_instance_field_can_call_an_earlier_private_function_field() {
    let program =
        lower_script("class C { #callable = () => 42; value = this.#callable(); } new C().value;");

    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
}

#[test]
fn generated_class_elements_record_their_execution_kind_and_strictness() {
    let program = lower_script(
        "function ordinary() {}
             class C {
                 instance = 1;
                 #private = 2;
                 static shared = 3;
                 static {}
                 method() {}
                 get value() {}
                 set value(next) {}
                 static sharedMethod() {}
                 static get sharedValue() {}
                 static set sharedValue(next) {}
             }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let class = script
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Lexical {
                init:
                    TypedExpr {
                        expr: ExprIr::ClassDefinition(class),
                        ..
                    },
                ..
            } => Some(class.as_ref()),
            _ => None,
        })
        .expect("class definition should be lowered");

    let constructor = script
        .functions
        .iter()
        .find(|function| function.id == class.constructor_function_id)
        .expect("class constructor should be lowered");
    let instance_plan = constructor
        .class_instance_element_plan
        .as_ref()
        .expect("constructor should own its instance element plan");
    for field in &instance_plan.elements {
        let ClassInstanceElementIr::Field(field) = field else {
            panic!("expected an ordinary field")
        };
        let initializer_id = field
            .init_function_id
            .as_ref()
            .expect("field should have an initializer");
        let initializer = script
            .functions
            .iter()
            .find(|function| &function.id == initializer_id)
            .expect("field initializer function should be lowered");
        assert_eq!(
            initializer.class_element_execution_kind,
            ClassElementExecutionKind::InstanceFieldInitializer
        );
        assert!(initializer.strict);
        assert_eq!(initializer.protocol.class_kind(), ClassFunctionKind::None);
    }
    for field in class
        .element_plan
        .static_elements
        .iter()
        .filter_map(|element| match element {
            ClassStaticElementIr::Field(field) => Some(field),
            ClassStaticElementIr::AutoAccessorBacking(_) | ClassStaticElementIr::Block(_) => None,
        })
    {
        let initializer_id = field
            .init_function_id
            .as_ref()
            .expect("field should have an initializer");
        let initializer = script
            .functions
            .iter()
            .find(|function| &function.id == initializer_id)
            .expect("field initializer function should be lowered");
        assert_eq!(
            initializer.class_element_execution_kind,
            ClassElementExecutionKind::StaticFieldInitializer
        );
        assert!(initializer.strict);
        assert_eq!(initializer.protocol.class_kind(), ClassFunctionKind::None);
    }

    let static_block_id = class
        .element_plan
        .static_elements
        .iter()
        .find_map(|element| match element {
            ClassStaticElementIr::Block(block) => Some(&block.function_id),
            ClassStaticElementIr::Field(_) | ClassStaticElementIr::AutoAccessorBacking(_) => None,
        })
        .expect("static block should be planned");
    let static_block = script
        .functions
        .iter()
        .find(|function| &function.id == static_block_id)
        .expect("static block function should be lowered");
    assert_eq!(
        static_block.class_element_execution_kind,
        ClassElementExecutionKind::StaticBlock
    );
    assert!(static_block.strict);

    for function in script.functions.iter().filter(|function| {
        function.id == class.constructor_function_id
            || class.element_plan.definitions.iter().any(|definition| {
                matches!(
                    definition,
                    ClassElementDefinitionIr::PublicMethod(method)
                        if method.function_id == function.id
                )
            })
            || function.name == "ordinary"
    }) {
        assert_eq!(
            function.class_element_execution_kind,
            ClassElementExecutionKind::None
        );
        if function.name == "ordinary" {
            assert!(!function.strict);
        } else {
            assert!(function.strict, "{} should be strict", function.name);
        }
    }
}

#[test]
fn nested_arrows_in_class_elements_capture_the_home_object() {
    let program = lower_script(
            "class Base {} Base.prototype.marker = 1; Base.marker = 2; class C extends Base { instance = (() => super.marker)(); static shared = (() => super.marker)(); static { this.block = (() => super.marker)(); } }",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let arrows = script
        .functions
        .iter()
        .filter(|function| function.protocol.flavor() == FunctionFlavor::Arrow)
        .collect::<Vec<_>>();

    assert_eq!(arrows.len(), 3);
    assert!(arrows.iter().all(|arrow| {
        arrow
            .captured_bindings
            .iter()
            .any(|binding| binding.source_name == LEXICAL_HOME_OBJECT_NAME)
    }));
}

#[test]
fn class_parameter_defaults_capture_outer_bindings() {
    let program = lower_script(
            "function owner() { const fallback = 1; class C { constructor(value = fallback) {} method(value = fallback) { return value; } } return C; }",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    for member_name in ["C", "C.method"] {
        let member = script
            .functions
            .iter()
            .find(|function| function.name == member_name)
            .unwrap_or_else(|| panic!("class member `{member_name}` should be lowered"));
        assert!(member.captured_bindings.iter().any(|binding| {
            binding.source_name == "fallback" && binding.mode == BindingMode::Const
        }));
    }
}

#[test]
fn private_class_callable_names_preserve_source_spelling_and_accessor_prefixes() {
    let program = lower_script(
        "class C {
                #instanceMethod() {}
                static #staticMethod() {}
                get #value() { return 1; }
                set #value(next) {}
                publicMethod() {}
            }
            class D { #instanceMethod() {} }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let function_names = script
        .functions
        .iter()
        .map(|function| function.name.as_str())
        .collect::<BTreeSet<_>>();

    for private_name in [
        "#instanceMethod",
        "#staticMethod",
        "get #value",
        "set #value",
    ] {
        assert!(
            function_names.contains(private_name),
            "missing private callable name `{private_name}` in {function_names:?}"
        );
    }
    assert!(function_names.contains("C.publicMethod"));

    let same_spelling = script
        .functions
        .iter()
        .filter(|function| function.name == "#instanceMethod")
        .collect::<Vec<_>>();
    assert_eq!(same_spelling.len(), 2);
    assert_ne!(same_spelling[0].id, same_spelling[1].id);
}

#[test]
fn lowers_script_global_update_from_class_constructor() {
    let program = lower_script(
        "var count = 0; class Base { constructor() { count++; } } class Derived extends Base { constructor() { (_ => super())(); } } new Derived();",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let base_constructor = script
        .functions
        .iter()
        .find(|function| function.name == "Base")
        .expect("base constructor should be lowered");
    assert!(matches!(
        base_constructor.body.statements.first(),
        Some(StatementIr::Expression(TypedExpr {
            expr: ExprIr::EnvironmentIdentifier(reference),
            ..
        })) if reference.name == "count"
            && reference.resolution_start() == EnvironmentIdentifierResolutionStart::GlobalEnvironment
            && matches!(reference.operation, EnvironmentIdentifierOperationIr::Update {
                operation: NumericUpdateOp::Increment,
                return_mode: UpdateReturnMode::Postfix,
            })
    ));
}

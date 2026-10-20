#[test]
fn allows_subclassing_iterator_constructor() {
    let program = lower_script("class SubIterator extends Iterator {}");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Lexical { init, .. } = &script.body.statements[0] else {
        panic!("expected class lexical declaration");
    };
    assert!(matches!(init.kind, ValueKind::Function));
}

#[test]
fn typed_array_constructor_prototype_keeps_the_hidden_intrinsic_identity() {
    let program = lower_script("var TypedArray = Object.getPrototypeOf(Int8Array);");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::Var(declarators) = &script.body.statements[0] else {
        panic!("expected variable declaration");
    };
    let init = declarators[0]
        .init
        .as_ref()
        .expect("TypedArray should have an initializer");
    assert_eq!(init.kind, ValueKind::Function);
    assert_eq!(init.possible_kinds, KindSet::from_kind(ValueKind::Function));
    assert_eq!(
        init.function_targets.exact_single_target(),
        Some(&StandardBuiltinId::TypedArrayConstructor.function_id())
    );
    assert!(StandardBuiltinId::TypedArrayConstructor.constructable());
    let Some(HeapShape::Object(constructor_shape)) = init.heap_shape.as_deref() else {
        panic!("hidden TypedArray constructor should have a function shape");
    };
    assert!(constructor_shape.properties.contains_key("from"));
    assert!(constructor_shape.properties.contains_key("of"));
    let Some(ObjectShapeProperty::Data(prototype)) = constructor_shape.properties.get("prototype")
    else {
        panic!("hidden TypedArray constructor should expose its prototype");
    };
    let Some(HeapShape::Object(prototype_shape)) = prototype.heap_shape.as_deref() else {
        panic!("TypedArray prototype should have an object shape");
    };
    let Some(ObjectShapeProperty::Data(constructor)) =
        prototype_shape.properties.get("constructor")
    else {
        panic!("TypedArray prototype should refer back to its constructor");
    };
    assert_eq!(
        constructor.function_targets.exact_single_target(),
        Some(&StandardBuiltinId::TypedArrayConstructor.function_id())
    );
}

#[test]
fn lowers_heap_shapes_and_array_length() {
    let program = lower_script(
            "function box() { let o = { inner: { x: 2 } }; return o; } let a = [1, 2, 3]; box().inner.x + a.length;",
        );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    assert_eq!(script.functions[0].return_kind, ValueKind::Object);
    assert!(script.functions[0].return_shape.is_some());
    let summary = program.ir_summary();
    assert!(summary.contains("array_lengths=1"));
    assert!(summary.contains("heap_shapes="));
}

#[test]
fn lowers_property_access_on_dynamic_after_kind_merge() {
    let program = lower_script("let v; if (true) { v = 1; } else { v = { x: 1 }; } v.x;");
    assert!(program.is_wasm_supported());
    assert_eq!(
        program
            .script
            .as_ref()
            .expect("script ir should exist")
            .result_kind(),
        ValueKind::Dynamic
    );
}

#[test]
fn lowers_nested_function_declaration() {
    let program =
        lower_script("function outer() { function inner() { return 1; } return inner(); }");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    assert_eq!(script.functions.len(), 3);
    assert!(script.functions.iter().any(|function| function.is_nested));
    let summary = program.ir_summary();
    assert!(summary.contains("nested_functions=2"));
}

#[test]
fn lowers_closure_capture_and_function_expression() {
    let program = lower_script(
            "function outer() { let x = 2; return function (y) { return x + y; }; } let f = outer(); f(3);",
        );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    assert_eq!(script.functions.len(), 3);
    assert!(script
        .functions
        .iter()
        .any(|function| function.is_expression));
    assert!(script
        .functions
        .iter()
        .any(|function| !function.captured_bindings.is_empty()));
    let summary = program.ir_summary();
    assert!(summary.contains("function_exprs=1"));
    assert!(summary.contains("closures=3"));
    assert!(summary.contains("captures=1"));
}

#[test]
fn lowers_script_closure_capture() {
    let program = lower_script("let x = 1; function f() { return x; } f();");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    assert_eq!(script.owned_env_bindings.len(), 1);
    assert_eq!(script.owned_env_bindings[0].name, "x");
    assert_eq!(script.functions[0].captured_bindings[0].name, "x");
    assert_eq!(
        script.functions[0].captured_bindings[0].slot,
        script.owned_env_bindings[0].slot
    );
    assert_eq!(script.functions[0].captured_bindings[0].hops, 0);
}

#[test]
fn named_function_captures_count_the_runtime_self_record_without_direct_eval() {
    for (function, hops) in [
        ("function named() { return value; }", 1),
        ("function* named() { yield value; }", 2),
        ("async function named() { return value; }", 2),
        ("async function* named() { yield value; }", 2),
    ] {
        let source = format!("function outer(value) {{ return {function}; }} outer(7);");
        let program = lower_script(&source);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let functions = &program.script.as_ref().unwrap().functions;
        let named = functions.iter().find(|function| function.name == "named").unwrap();
        assert!(named.is_named_expression);
        let capture = named.captured_bindings.iter().find(|binding| binding.source_name == "value").unwrap();
        assert_eq!(capture.hops, hops, "{source}");
    }
}

#[test]
fn nested_named_function_captures_count_each_physical_self_record() {
    let program = lower_script(
        "function outer(value) { return function first() { return function second() { return value; }; }; } outer(7)();",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let functions = &program.script.as_ref().unwrap().functions;
    let second = functions.iter().find(|function| function.name == "second").unwrap();
    let capture = second.captured_bindings.iter().find(|binding| binding.source_name == "value").unwrap();
    assert_eq!(capture.hops, 2);
}

#[test]
fn with_body_captures_outer_binding_for_property_fallback() {
    let program = lower_script(
            "const fallback = { value: 17 }; function observe(view) { with (view) { return fallback.value; } }",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let observe = script
        .functions
        .iter()
        .find(|function| function.name == "observe")
        .expect("observe function should be lowered");

    assert!(observe
        .captured_bindings
        .iter()
        .any(|binding| binding.name == "fallback"));
}

#[test]
fn supported_block_patterns_share_span_stable_capture_storage() {
    for declaration in ["let { value } = { value: 2 };", "let [value] = [2];"] {
        let source = format!(
                "function owner() {{ let value = 1; {{ {declaration} return (() => value)(); }} }} owner();"
            );
        assert_function_capture_storage_contract(&source, "owner", None, "$scoped.lex.");
    }
}

#[test]
fn object_catch_pattern_shares_span_stable_capture_storage() {
    assert_function_capture_storage_contract(
            "function owner() { let value = 1; try { throw { value: 2 }; } catch ({ value }) { return (() => value)(); } } owner();",
            "owner",
            None,
            "$scoped.lex.",
        );
}

#[test]
fn script_supported_patterns_keep_source_named_capture_storage() {
    for source in [
        "let { value } = { value: 2 }; (() => value)();",
        "let [value] = [2]; (() => value)();",
    ] {
        let program = lower_script(source);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let capture = script
            .functions
            .iter()
            .flat_map(|function| &function.captured_bindings)
            .find(|binding| binding.name == "value")
            .expect("arrow should capture the root lexical binding");
        assert!(script
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == capture.name && binding.slot == capture.slot));
        assert!(collect_binding_storage_names(&script.body).contains(&capture.name));
    }
}

#[test]
fn pattern_initializer_closures_capture_eventual_lexical_storage() {
    for source in [
        "let { value } = { value: (() => value)() };",
        "let [value] = [(() => value)()];",
    ] {
        let program = lower_script(source);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let capture = script
            .functions
            .iter()
            .flat_map(|function| &function.captured_bindings)
            .find(|binding| binding.name == "value")
            .expect("initializer arrow should capture the eventual lexical binding");
        assert!(script
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == capture.name && binding.slot == capture.slot));
    }
}

#[test]
fn object_var_pattern_keeps_owner_capture_storage() {
    assert_function_capture_storage_contract(
        "function owner() { var { value } = { value: 2 }; return () => value; } owner()();",
        "owner",
        None,
        "value",
    );
}

#[test]
fn object_for_of_pattern_shares_dedicated_loop_capture_storage() {
    assert_function_capture_storage_contract(
            "function owner() { let value = 1; let read; for (let { value } of [{ value: 2 }]) { read = () => value; break; } return read(); } owner();",
            "owner",
            None,
            "$forof.lex.",
        );
}

#[test]
fn classic_for_head_shares_span_stable_capture_storage() {
    assert_function_capture_storage_contract(
            "function owner() { let value = 1; let read; for (let value = 2; value < 3; value++) { read = () => value; break; } return read(); } owner();",
            "owner",
            None,
            "$scoped.lex.",
        );
}

#[test]
fn classic_for_update_targets_lexical_storage() {
    let program = lower_script("for (let value = 0; value < 2; value++) {}");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::For {
        init: Some(ForInitIr::Lexical { name, .. }),
        update: Some(update),
        ..
    } = &script.body.statements[0]
    else {
        panic!("expected classic for lexical initializer and update");
    };
    assert!(name.starts_with("$scoped.lex."));
    assert!(matches!(
        &update.expr,
        ExprIr::UpdateIdentifier {
            name: update_name,
            ..
        } if update_name == name
    ));
}

#[test]
fn classic_for_initializers_capture_eventual_physical_bindings() {
    for source in [
            "function owner() { let value = 1; let read; for (let value = 2, unused = read = () => value; false;) {} return read(); } owner();",
            "function owner() { let read; for (let inner = read = () => later, later = 2; false;) {} return read(); } owner();",
        ] {
            assert_function_capture_storage_contract(source, "owner", None, "$scoped.lex.");
        }
}

#[test]
fn classic_for_direct_self_initializer_read_uses_tdz() {
    let program = lower_script(
        "function owner() { let value = 1; for (let value = value;;) { break; } } owner();",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner function should be lowered");
    let init = owner
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::For {
                init: Some(ForInitIr::Lexical { init, .. }),
                ..
            } => Some(init),
            _ => None,
        })
        .expect("classic for lexical initializer should be lowered");
    assert!(matches!(
        &init.expr,
        ExprIr::RuntimeThrow {
            name: NativeErrorKind::ReferenceError,
            ..
        }
    ));
}

#[test]
fn nested_root_declarations_preserve_transitive_creation_aliases() {
    assert_function_capture_storage_contract(
            "function owner() { let value = 0; { let value = 2; function middle() { function inner() { return value; } return inner(); } return middle(); } } owner();",
            "owner",
            Some("inner"),
            "$scoped.lex.",
        );
}

#[test]
fn lowers_for_in_let_closure_capture_metadata() {
    let program = lower_script(
            "function fn(x) { let callbacks = []; for (let p in x) { callbacks.push(function () { return p; }); } return callbacks[0](); } fn({ a: 1 });",
        );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let outer = script
        .functions
        .iter()
        .find(|function| function.name == "fn")
        .expect("outer function should be lowered");
    let loop_binding = outer
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::ForInObject {
                lexical_environment: Some(environment),
                ..
            } => environment
                .iteration_environment
                .as_ref()
                .and_then(|environment| {
                    environment.bindings.iter().find(|binding| {
                        binding.name.starts_with("$forin.lex.") && binding.name.ends_with(".p")
                    })
                }),
            _ => None,
        })
        .expect("loop binding should own an iteration environment slot");
    let closure = script
        .functions
        .iter()
        .find(|function| function.is_expression)
        .expect("loop closure should be lowered");
    assert_eq!(closure.captured_bindings.len(), 1);
    assert_eq!(closure.captured_bindings[0].name, loop_binding.name);
    assert_eq!(closure.captured_bindings[0].slot, loop_binding.slot);
}

#[test]
fn lowers_shadowed_for_in_let_closure_to_loop_binding() {
    let program = lower_script(
        "let x = 'outside'; var f; for (let x in { a: 0 }) { f = function () { return x; }; } f();",
    );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let closure = script
        .functions
        .iter()
        .find(|function| function.is_expression)
        .expect("loop closure should be lowered");
    assert_eq!(closure.captured_bindings.len(), 1);
    let captured = &closure.captured_bindings[0];
    assert!(captured.name.starts_with("$forin.lex."));
    assert!(captured.name.ends_with(".x"));
    assert_ne!(captured.name, "x");
    let StatementIr::ForInObject {
        lexical_environment: Some(environment),
        ..
    } = &script.body.statements[2]
    else {
        panic!("expected captured for-in iteration environment");
    };
    assert!(environment
        .iteration_environment
        .as_ref()
        .is_some_and(|environment| environment
            .bindings
            .iter()
            .any(|binding| binding.name == captured.name && binding.slot == captured.slot)));
}

#[test]
fn lowers_for_in_head_lexical_tdz_for_target_expression() {
    fn has_reference_error_throw(expr: &TypedExpr) -> bool {
        match &expr.expr {
            ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                ..
            } => true,
            ExprIr::ObjectLiteral(properties) => properties.iter().any(|property| match property {
                ObjectPropertyIr::PrototypeSetter { value }
                | ObjectPropertyIr::Spread { source: value }
                | ObjectPropertyIr::Data { value, .. }
                | ObjectPropertyIr::NonEnumerableData { value, .. } => {
                    has_reference_error_throw(value)
                }
                ObjectPropertyIr::ComputedData { key, value, .. } => {
                    has_reference_error_throw(key) || has_reference_error_throw(value)
                }
                ObjectPropertyIr::ComputedMethod { key, .. }
                | ObjectPropertyIr::ComputedGetter { key, .. }
                | ObjectPropertyIr::ComputedSetter { key, .. } => has_reference_error_throw(key),
                ObjectPropertyIr::Method { .. }
                | ObjectPropertyIr::Getter { .. }
                | ObjectPropertyIr::Setter { .. } => false,
            }),
            ExprIr::TypeOf { expr } => has_reference_error_throw(expr),
            _ => false,
        }
    }

    let program = lower_script("let x = 1; for (let x in { x }) {}");
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let StatementIr::ForInObject { target, .. } = &script.body.statements[1] else {
        panic!("expected for-in object statement");
    };
    assert!(has_reference_error_throw(target));
}

#[test]
fn lowers_for_of_object_pattern_head_under_lexical_tdz() {
    let program = lower_script("let x = 1; for (let { x } of [{ x }]) {}");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let StatementIr::ForOfIterator { iterable, .. } = &script.body.statements[1] else {
        panic!("expected generic for-of statement");
    };
    let ExprIr::ArrayLiteral(elements) = &iterable.expr else {
        panic!("expected array iterable");
    };
    let ExprIr::ObjectLiteral(properties) = &elements[0].expr else {
        panic!("expected object element");
    };
    let ObjectPropertyIr::Data { value, .. } = &properties[0] else {
        panic!("expected shorthand data property");
    };
    assert!(matches!(
        value.expr,
        ExprIr::RuntimeThrow {
            name: NativeErrorKind::ReferenceError,
            ..
        }
    ));
}

#[test]
fn lowers_for_in_head_function_capture_to_tdz_binding() {
    let program = lower_script(
            "let x = 'outside'; var f; for (let x in { i: f = function () { return typeof x; } }) {} f();",
        );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let closure = script
        .functions
        .iter()
        .find(|function| function.is_expression)
        .expect("head closure should be lowered");
    assert_eq!(closure.captured_bindings.len(), 1);
    assert!(closure.captured_bindings[0]
        .name
        .starts_with(TDZ_BINDING_STORAGE_PREFIX));
    assert!(closure.captured_bindings[0].name.ends_with(".x"));
}

#[test]
fn infers_array_buffer_new_shape_for_closure_capture() {
    let program = lower_script(
        "const rab = new ArrayBuffer(64, { maxByteLength: 1024 }); const f = () => rab.resize;",
    );
    assert!(program.is_wasm_supported());
    let script = program.script.as_ref().expect("script ir should exist");
    let arrow = script
        .functions
        .iter()
        .find(|function| function.protocol.flavor() == FunctionFlavor::Arrow)
        .expect("arrow function should be lowered");
    let Some(StatementIr::Return(expr)) = arrow.body.statements.first() else {
        panic!("expression arrow should lower to return");
    };
    assert!(
        expr.function_targets
            .exact_single_target()
            .is_some_and(|target| {
                target == &StandardBuiltinId::ArrayBufferPrototypeResize.function_id()
            }),
        "captured ArrayBuffer shape must retain resize: {expr:?}"
    );
}

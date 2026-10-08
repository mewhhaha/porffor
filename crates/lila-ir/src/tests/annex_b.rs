#[test]
fn annex_b_block_functions_create_undefined_owner_bindings_and_copy_when_selected() {
    let program = lower_script(
        "if (false) { function unselected() {} } if (true) { function selected() {} }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    for name in ["unselected", "selected"] {
        assert!(script.global_bindings.iter().any(|binding| {
            binding.name == name && binding.declarations == GlobalDeclarationSetIr::Var
        }));
    }
    let copies = collect_annex_b_copies(&script.body);
    assert_eq!(copies.len(), 2);
    assert!(copies.iter().any(|(source, block, target)| {
        source == "unselected"
            && block.starts_with("$annexb.block.")
            && target
                == &AnnexBFunctionCopyTargetIr::ScriptGlobal {
                    name: source.clone(),
                }
    }));
    assert!(copies.iter().any(|(source, block, target)| {
        source == "selected"
            && block.starts_with("$annexb.block.")
            && target
                == &AnnexBFunctionCopyTargetIr::ScriptGlobal {
                    name: source.clone(),
                }
    }));
}

#[test]
fn annex_b_block_function_self_reference_captures_the_block_binding() {
    let program =
        lower_script("function owner() { { function f() { return f; } } return f; } owner();");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let function = script
        .functions
        .iter()
        .find(|function| function.name == "f")
        .expect("block function should be lowered");
    assert!(function
        .captured_bindings
        .iter()
        .any(|binding| binding.name.starts_with("$annexb.block.")));
}

#[test]
fn script_annex_b_block_function_self_reference_captures_the_block_binding() {
    let program = lower_script("if (true) function f() { return f; } f();");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let function = script
        .functions
        .iter()
        .find(|function| function.name == "f")
        .expect("block function should be lowered");
    assert!(function.captured_bindings.iter().any(|binding| {
        binding.name.starts_with("$annexb.block.") && binding.source_name == "f"
    }));
}

#[test]
fn annex_b_sibling_block_function_captures_the_block_binding() {
    let program = lower_script(
            "function owner() { let f = function () { return 2; }; { function f() { return 1; } function g() { return f(); } return g(); } } owner();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner function should be lowered");
    let sibling = script
        .functions
        .iter()
        .find(|function| function.name == "g")
        .expect("sibling block function should be lowered");
    let captured = sibling
        .captured_bindings
        .iter()
        .find(|binding| binding.name.starts_with("$annexb.block."))
        .expect("sibling should capture the block function binding");
    assert_ne!(captured.name, "f");
    assert!(!owner
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == captured.name));
    assert!(block_environment_owns_binding(
        &owner.body,
        &captured.name,
        captured.slot
    ));
}

#[test]
fn block_function_captures_same_block_let_binding() {
    let program = lower_script(
            "function owner() { 'use strict'; let value = 2; { let value = 1; function read() { return value; } return read(); } } owner();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner function should be lowered");
    let reader = script
        .functions
        .iter()
        .find(|function| function.name == "read")
        .expect("block function should be lowered");
    let captured = reader
        .captured_bindings
        .iter()
        .find(|binding| binding.name.starts_with("$scoped.lex."))
        .expect("block function should capture the block lexical binding");
    assert!(!owner
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == captured.name));
    assert!(block_environment_owns_binding(
        &owner.body,
        &captured.name,
        captured.slot
    ));
}

#[test]
fn block_function_captures_same_block_class_binding() {
    let program = lower_script(
            "function owner() { 'use strict'; class Outer {} { class Local {} function read() { return Local; } return read(); } } owner();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner function should be lowered");
    let reader = script
        .functions
        .iter()
        .find(|function| function.name == "read")
        .expect("block function should be lowered");
    let captured = reader
        .captured_bindings
        .iter()
        .find(|binding| binding.name.starts_with("$scoped.lex."))
        .expect("block function should capture the block class binding");
    assert!(!owner
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == captured.name));
    assert!(block_environment_owns_binding(
        &owner.body,
        &captured.name,
        captured.slot
    ));
}

#[test]
fn nested_block_function_capture_uses_the_nearest_shadowing_binding() {
    let program = lower_script(
            "function owner() { 'use strict'; let value = 0; { let value = 1; { const value = 2; function read() { return value; } return read(); } } } owner();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner function should be lowered");
    let reader = script
        .functions
        .iter()
        .find(|function| function.name == "read")
        .expect("nested block function should be lowered");
    let captured = reader
        .captured_bindings
        .iter()
        .find(|binding| binding.name.starts_with("$scoped.lex."))
        .expect("nested block function should capture a scoped lexical binding");
    assert_ne!(captured.name, "value");
    assert!(!owner
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == captured.name));
    assert!(block_environment_owns_binding(
        &owner.body,
        &captured.name,
        captured.slot
    ));
}

#[test]
fn captured_block_bindings_use_nested_environment_hops_without_owner_activation() {
    let program = lower_script(
            "function owner() { { let outer = 1; { let inner = 2; function read() { return outer + inner; } return read; } } } owner();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner function should be lowered");
    let reader = script
        .functions
        .iter()
        .find(|function| function.name == "read")
        .expect("reader function should be lowered");
    let outer = reader
        .captured_bindings
        .iter()
        .find(|binding| binding.source_name == "outer")
        .expect("outer block binding should be captured");
    let inner = reader
        .captured_bindings
        .iter()
        .find(|binding| binding.source_name == "inner")
        .expect("inner block binding should be captured");

    assert!(owner.owned_env_bindings.is_empty());
    assert_eq!(outer.hops, 1);
    assert_eq!(inner.hops, 0);
    assert!(block_environment_owns_binding(
        &owner.body,
        &outer.name,
        outer.slot
    ));
    assert!(block_environment_owns_binding(
        &owner.body,
        &inner.name,
        inner.slot
    ));
}

#[test]
fn captured_for_of_binding_uses_one_hop_from_the_body_block() {
    let program = lower_script(
            "function owner() { let saved; for (let value of [1]) { let body = 2; function read() { return value + body; } saved = read; } return saved; } owner();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let reader = script
        .functions
        .iter()
        .find(|function| function.name == "read")
        .expect("reader function should be lowered");
    let capture_hops = reader
        .captured_bindings
        .iter()
        .map(|binding| (binding.source_name.as_str(), binding.hops))
        .collect::<BTreeMap<_, _>>();

    assert_eq!(capture_hops.get("body"), Some(&0));
    assert_eq!(capture_hops.get("value"), Some(&1));
}

#[test]
fn captured_block_bindings_skip_to_owner_activation_when_it_exists() {
    let program = lower_script(
            "function owner(argument) { { let outer = 1; { let inner = 2; function read() { return argument + outer + inner; } return read; } } } owner(3);",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner function should be lowered");
    let reader = script
        .functions
        .iter()
        .find(|function| function.name == "read")
        .expect("reader function should be lowered");
    let capture_hops = reader
        .captured_bindings
        .iter()
        .map(|binding| (binding.source_name.as_str(), binding.hops))
        .collect::<BTreeMap<_, _>>();

    assert!(owner
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == "argument"));
    assert_eq!(capture_hops.get("inner"), Some(&0));
    assert_eq!(capture_hops.get("outer"), Some(&1));
    assert_eq!(capture_hops.get("argument"), Some(&2));
}

#[test]
fn switch_case_block_functions_share_lexical_capture_aliases() {
    let program = lower_script(
            "function owner() { 'use strict'; let value = 0; switch (1) { case 1: let value = 1; case 2: function read() { return value; } return read(); } } owner();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner function should be lowered");
    let reader = script
        .functions
        .iter()
        .find(|function| function.name == "read")
        .expect("case block function should be lowered");
    let captured = reader
        .captured_bindings
        .iter()
        .find(|binding| binding.name.starts_with("$scoped.lex."))
        .expect("case block function should capture the case lexical binding");
    assert!(owner.owned_env_bindings.is_empty());
    let StatementIr::Switch {
        lexical_environment,
        ..
    } = owner
        .body
        .statements
        .iter()
        .find(|statement| matches!(statement, StatementIr::Switch { .. }))
        .expect("owner should contain switch statement")
    else {
        panic!("expected switch statement");
    };
    assert!(lexical_environment.as_ref().is_some_and(|environment| {
        environment
            .bindings
            .iter()
            .any(|binding| binding.name == captured.name && binding.slot == captured.slot)
    }));
    assert!(block_environment_owns_binding(
        &owner.body,
        &captured.name,
        captured.slot
    ));
}

#[test]
fn switch_selector_reads_its_shared_lexical_environment_in_tdz() {
    let program = lower_script(
            "function select() { let value = 1; switch (value) { case value: let value = 2; } } select();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let select = script
        .functions
        .iter()
        .find(|function| function.name == "select")
        .expect("select function should be lowered");
    let StatementIr::Switch { cases, .. } = select
        .body
        .statements
        .iter()
        .find(|statement| matches!(statement, StatementIr::Switch { .. }))
        .expect("select should contain switch statement")
    else {
        panic!("expected switch statement");
    };
    let condition = cases[0]
        .condition
        .as_ref()
        .expect("case should have a selector");

    assert!(matches!(condition.expr, ExprIr::RuntimeThrow { .. }));
}

#[test]
fn block_shadow_read_before_declaration_uses_the_inner_tdz_binding() {
    let program =
        lower_script("function owner() { let value = 1; { value; let value = 2; } } owner();");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner function should be lowered");
    let block = owner
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Block(block) => Some(block),
            _ => None,
        })
        .expect("owner should contain the shadowing block");

    assert!(matches!(
        &block.statements[0],
        StatementIr::Expression(TypedExpr {
            expr: ExprIr::RuntimeThrow {
                name: NativeErrorKind::ReferenceError,
                ..
            },
            ..
        })
    ));
    let StatementIr::Lexical { name, .. } = &block.statements[1] else {
        panic!("expected the inner lexical declaration");
    };
    assert!(name.starts_with("$scoped.lex."));
}

#[test]
fn block_self_initializer_uses_the_inner_tdz_binding() {
    let program =
        lower_script("function owner() { let value = 1; { let value = value; } } owner();");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner function should be lowered");
    let block = owner
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Block(block) => Some(block),
            _ => None,
        })
        .expect("owner should contain the shadowing block");
    let StatementIr::Lexical { init, .. } = &block.statements[0] else {
        panic!("expected the inner lexical declaration");
    };

    assert!(matches!(
        &init.expr,
        ExprIr::RuntimeThrow {
            name: NativeErrorKind::ReferenceError,
            ..
        }
    ));
}

#[test]
fn switch_later_selector_reads_its_shared_lexical_environment_in_tdz() {
    let program = lower_script(
        "function select() { switch (1) { case 0: let value = 1; break; case value: } } select();",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let select = script
        .functions
        .iter()
        .find(|function| function.name == "select")
        .expect("select function should be lowered");
    let StatementIr::Switch { cases, .. } = select
        .body
        .statements
        .iter()
        .find(|statement| matches!(statement, StatementIr::Switch { .. }))
        .expect("select should contain switch statement")
    else {
        panic!("expected switch statement");
    };
    let condition = cases[1]
        .condition
        .as_ref()
        .expect("second case should have a selector");

    assert!(matches!(
        &condition.expr,
        ExprIr::RuntimeThrow {
            name: NativeErrorKind::ReferenceError,
            ..
        }
    ));
}

#[test]
fn catch_block_functions_capture_catch_scope_lexical_bindings() {
    let program = lower_script(
            "function owner() { 'use strict'; let value = 0; try { throw 1; } catch (error) { let value = 1; function read() { return value; } return read(); } } owner();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner function should be lowered");
    let reader = script
        .functions
        .iter()
        .find(|function| function.name == "read")
        .expect("catch block function should be lowered");
    let captured = reader
        .captured_bindings
        .iter()
        .find(|binding| binding.name.starts_with("$scoped.lex."))
        .expect("catch block function should capture the catch lexical binding");
    assert!(owner.owned_env_bindings.is_empty());
    assert!(block_environment_owns_binding(
        &owner.body,
        &captured.name,
        captured.slot
    ));
}

#[test]
fn lowers_parent_linked_try_catch_finally_environment_layouts_and_hops() {
    let program = lower_script(
            "function owner() { try { let tried = 1; function readTried() { return tried; } throw 2; } catch (error) { let handled = error; function readHandled() { return error + handled; } return readHandled; } finally { let cleaned = 3; function readCleaned() { return cleaned; } } } owner();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner function should be lowered");
    let StatementIr::TryCatchFinally {
        try_block,
        catch_parameter_environment,
        catch_block,
        finally_block,
        ..
    } = owner
        .body
        .statements
        .iter()
        .find(|statement| matches!(statement, StatementIr::TryCatchFinally { .. }))
        .expect("owner should contain try/catch/finally")
    else {
        panic!("expected try/catch/finally statement");
    };

    assert!(try_block.lexical_environment.is_some());
    assert!(catch_parameter_environment.is_some());
    assert!(catch_block.lexical_environment.is_some());
    assert!(finally_block.lexical_environment.is_some());

    let handled = script
        .functions
        .iter()
        .find(|function| function.name == "readHandled")
        .expect("catch reader should be lowered");
    let capture_hops = handled
        .captured_bindings
        .iter()
        .map(|binding| (binding.source_name.as_str(), binding.hops))
        .collect::<BTreeMap<_, _>>();

    assert_eq!(capture_hops.get("handled"), Some(&0));
    assert_eq!(capture_hops.get("error"), Some(&1));
}

#[test]
fn strict_block_bindings_do_not_leak_to_sibling_functions() {
    let program = lower_script(
            "function owner() { 'use strict'; { function hidden() { return 1; } } function outside() { return typeof hidden; } return outside(); } owner();",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let outside = script
        .functions
        .iter()
        .find(|function| function.name == "outside")
        .expect("sibling function should be lowered");
    assert!(outside.captured_bindings.is_empty());
}

#[test]
fn annex_b_function_owner_parameter_and_arguments_bindings_block_outer_copies() {
    for source in [
        "function owner(f) { { function f() {} } return f; }",
        "function owner() { { function arguments() {} } return arguments; }",
    ] {
        let program = lower_script(source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script IR should exist");
        assert!(script
            .functions
            .iter()
            .all(|function| collect_annex_b_copies(&function.body).is_empty()));
    }
}

#[test]
fn annex_b_top_level_lexical_binding_blocks_outer_copy() {
    let program = lower_script("let f = 1; { function f() {} } f;");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    assert!(collect_annex_b_copies(&script.body).is_empty());
    assert!(!script
        .global_bindings
        .iter()
        .any(|binding| binding.name == "f"));
}

#[test]
fn annex_b_blocked_candidates_do_not_create_script_owner_bindings() {
    for (shape, source) in [
            (
                "block",
                "{ let f; { function f() {} } } typeof f; f; function outside() { return typeof f; }",
            ),
            (
                "switch",
                "switch (0) { default: let f; { function f() {} } } typeof f; f; function outside() { return typeof f; }",
            ),
        ] {
            let mut interner = Interner::default();
            let scope = Scope::new_global();
            let parsed_script = Parser::new(Source::from_bytes(source.as_bytes()))
                .parse_script(&scope, &mut interner)
                .expect("script should parse");
            let analysis = AnalysisBuilder::default().finish(&parsed_script, &interner, source);
            assert!(
                !analysis.owner_plans[SCRIPT_OWNER_ID]
                    .root_bindings
                    .contains("f"),
                "{shape}: blocked Annex B candidate must not create a script owner binding"
            );

            let program = lower_script(source);
            assert!(
                program.is_wasm_supported(),
                "{shape}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script IR should exist");
            assert!(
                !script
                    .global_bindings
                    .iter()
                    .any(|binding| binding.name == "f"),
                "{shape}: blocked Annex B candidate must not create a script global binding"
            );
            assert!(
                collect_annex_b_copies(&script.body)
                    .iter()
                    .all(|(source, _, _)| source != "f"),
                "{shape}: blocked Annex B candidate must not copy to the variable environment"
            );
            assert!(
                script.body.statements.iter().any(|statement| matches!(
                    statement,
                    StatementIr::Expression(TypedExpr {
                        expr: ExprIr::TypeOfUnresolvedIdentifier { .. },
                        ..
                    })
                )),
                "{shape}: outer typeof f must be unresolved"
            );
            assert!(
                script.body.statements.iter().any(|statement| matches!(
                    statement,
                    StatementIr::Expression(TypedExpr {
                        expr: ExprIr::GlobalIdentifierRead { name },
                        ..
                    }) if name == "f"
                )),
                "{shape}: outer f read must use runtime global resolution"
            );

            let outside = script
                .functions
                .iter()
                .find(|function| function.name == "outside")
                .expect("outside function should be lowered");
            assert!(
                outside.captured_bindings.is_empty(),
                "{shape}: outside callback must not capture blocked f"
            );
            assert!(
                outside.body.statements.iter().any(|statement| matches!(
                    statement,
                    StatementIr::Return(TypedExpr {
                        expr: ExprIr::TypeOfUnresolvedIdentifier { .. },
                        ..
                    })
                )),
                "{shape}: outside callback typeof f must be unresolved"
            );
        }
}

#[test]
fn annex_b_existing_var_and_function_bindings_are_reused() {
    let program = lower_script(
            "function owner() { var f = 1; { function f() {} } function g() {} { function g() {} } return f; }",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner should be lowered");
    let copies = collect_annex_b_copies(&owner.body);
    assert_eq!(copies.len(), 2);
    assert!(copies.iter().any(|(source, _, target)| source == "f"
        && target
            == &AnnexBFunctionCopyTargetIr::OwnerBinding {
                storage_name: "f".to_string(),
            }));
    assert!(copies.iter().any(|(source, _, target)| source == "g"
        && target
            == &AnnexBFunctionCopyTargetIr::OwnerBinding {
                storage_name: "g".to_string(),
            }));
}

#[test]
fn annex_b_copy_bypasses_a_same_named_catch_binding() {
    let program = lower_script(
        "function owner() { try { throw 1; } catch (f) { { function f() {} } } return f; }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner should be lowered");
    let copies = collect_annex_b_copies(&owner.body);
    assert_eq!(copies.len(), 1);
    assert_eq!(copies[0].0, "f");
    assert_eq!(
        copies[0].2,
        AnnexBFunctionCopyTargetIr::OwnerBinding {
            storage_name: "f".to_string()
        }
    );
}

#[test]
fn annex_b_duplicate_declarations_share_the_last_block_binding() {
    let program = lower_script(
        "function owner() { { function f() { return 1; } function f() { return 2; } } return f; }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner should be lowered");
    let copies = collect_annex_b_copies(&owner.body);
    assert_eq!(copies.len(), 2);
    assert_eq!(copies[0].1, copies[1].1);
}

#[test]
fn annex_b_switch_declarations_share_one_case_block_binding() {
    let program = lower_script(
            "function owner(v) { switch (v) { case 0: function f() { return 1; } break; default: function f() { return 2; } } return f; }",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner should be lowered");
    let switch = owner
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Switch {
                lexical_declarations,
                cases,
                ..
            } => Some((lexical_declarations, cases)),
            _ => None,
        })
        .expect("switch should be lowered");
    assert_eq!(switch.0.len(), 1);
    assert_eq!(collect_annex_b_copies(&owner.body).len(), 2);
}

#[test]
fn nested_labelled_function_declaration_remains_block_scoped() {
    let program = lower_script(
            "function owner() { var result; { label: function f() { return 6; } result = f(); } return result; }",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner should be lowered");
    assert!(collect_annex_b_copies(&owner.body).is_empty());
    assert!(script.functions.iter().any(|function| function.name == "f"));
}

#[test]
fn annex_b_for_lexical_binding_blocks_outer_copy() {
    let program = lower_script(
            "function owner() { for (let f;;) { if (false) function _f() {} else function f() {} break; } return typeof f; }",
        );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let owner = script
        .functions
        .iter()
        .find(|function| function.name == "owner")
        .expect("owner should be lowered");
    let copies = collect_annex_b_copies(&owner.body);
    assert!(copies.iter().any(|(source, _, _)| source == "_f"));
    assert!(copies.iter().all(|(source, _, _)| source != "f"));
}

#[test]
fn annex_b_labelled_block_function_shares_storage_with_direct_redeclaration() {
    let program = lower_script(
        "function test() { { function f() { return 1; } l: function f() { return 2; } } return f(); } test(); \
         { function g() { return 1; } m: function g() { return 2; } }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.as_ref().expect("script IR should exist");
    let copies = collect_annex_b_copies(&script.body);
    assert!(copies.iter().any(|(source, _, _)| source == "g"));
}

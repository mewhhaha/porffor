use lila_front::{parse, ParseOptions};
use lila_ir::{lower, EvalBindingDeclarationIr, EvalEnvironmentRoleIr};

#[test]
fn eval_visible_activation_preserves_parameter_and_lexical_declaration_authority() {
    let parsed = parse(
        "function caller(parameter) { let lexical = 1; var variable = 2; eval(0); }",
        ParseOptions::script(),
    )
    .unwrap();
    let program = lower(&parsed);
    assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let caller = script
        .functions
        .iter()
        .find(|function| function.name == "caller")
        .unwrap();
    let Some(EvalEnvironmentRoleIr::Declarative { bindings, .. }) = &caller.eval_environment else {
        panic!("eval-visible function needs a named activation")
    };
    for (name, declaration) in [
        ("parameter", EvalBindingDeclarationIr::Parameter),
        ("lexical", EvalBindingDeclarationIr::Lexical),
        ("variable", EvalBindingDeclarationIr::Variable),
        ("arguments", EvalBindingDeclarationIr::Variable),
    ] {
        assert!(
            bindings
                .iter()
                .any(|binding| binding.source_name == name && binding.declaration == declaration),
            "missing {name}: {bindings:?}"
        );
    }
    assert!(bindings
        .iter()
        .all(|binding| !binding.source_name.starts_with('$')));
}

#[test]
fn an_empty_eval_visible_record_survives_without_static_slots() {
    let parsed = parse("(() => { eval(0); })();", ParseOptions::script()).unwrap();
    let program = lower(&parsed);
    assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    assert!(script
        .functions
        .iter()
        .any(|function| function.eval_environment.is_some()
            && function.owned_env_bindings.is_empty()));
}

#[test]
fn root_lexical_initialization_uses_the_same_cell_as_eval_after_global_shadowing() {
    let parsed = parse(
        "var shadow = 7; function caller(eval) { let shadow = 31; return eval('shadow;'); } caller(eval);",
        ParseOptions::script(),
    )
    .unwrap();
    let program = lower(&parsed);
    assert!(program.diagnostics.is_empty(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let caller = script
        .functions
        .iter()
        .find(|function| function.name == "caller")
        .unwrap();
    let Some(EvalEnvironmentRoleIr::Declarative { bindings, .. }) = &caller.eval_environment else {
        panic!("caller must expose its lexical binding to eval");
    };
    let named = bindings
        .iter()
        .find(|binding| binding.source_name == "shadow")
        .unwrap();
    let owned = caller
        .owned_env_bindings
        .iter()
        .find(|binding| binding.slot == named.slot)
        .unwrap();
    assert!(
        caller.body.statements.iter().any(|statement| {
            matches!(statement, lila_ir::StatementIr::Lexical { name, .. } if name == &owned.name)
        }),
        "the lexical declaration must initialize eval's physical cell"
    );
}

#[test]
fn parameter_expression_body_has_distinct_cells_and_explicit_initialization() {
    use lila_ir::{FunctionBodyBindingValueIr, LexicalEnvironmentInitializationIr, StatementIr};
    let parsed = parse(
        "function caller(parameter = eval('1')) { var parameter; var fresh; let lexical; return parameter; } caller();",
        ParseOptions::script(),
    ).unwrap();
    let lowered = lower(&parsed);
    assert!(lowered.diagnostics.is_empty(), "{:?}", lowered.diagnostics);
    let script = lowered.script.unwrap();
    let caller = script
        .functions
        .iter()
        .find(|function| function.name == "caller")
        .unwrap();
    let body = caller
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Block(body) => body.lexical_environment.as_ref(),
            _ => None,
        })
        .expect("parameter expressions require a separate body record");
    let parameter_slot = caller
        .owned_env_bindings
        .iter()
        .find(|binding| binding.name == "parameter")
        .unwrap()
        .slot;
    let body_slot = |name| {
        body.bindings
            .iter()
            .find(|binding| binding.name == name)
            .unwrap()
            .slot
    };
    let LexicalEnvironmentInitializationIr::FunctionBody { bindings } = &body.initialization else {
        panic!("body record must carry declaration initialization");
    };
    assert!(bindings
        .iter()
        .any(|binding| binding.slot == body_slot("parameter")
            && binding.value
                == FunctionBodyBindingValueIr::Parameter {
                    slot: parameter_slot
                }));
    assert!(bindings
        .iter()
        .any(|binding| binding.slot == body_slot("fresh")
            && binding.value == FunctionBodyBindingValueIr::Undefined));
    assert!(bindings
        .iter()
        .all(|binding| binding.slot != body_slot("lexical")));
    let Some(EvalEnvironmentRoleIr::Declarative { bindings, .. }) = &caller.eval_environment else {
        panic!()
    };
    assert!(bindings
        .iter()
        .all(|binding| binding.source_name != "fresh" && binding.source_name != "lexical"));
}

#[test]
fn prepared_eval_empty_blocks_lower_to_empty_completion_without_scope_records() {
    for prefix in ["", "'use strict';"] {
        for count in [1, 128, 4096] {
            let body = "{}".repeat(count);
            let source = format!("{prefix} eval({body:?});");
            let program = lower(&parse(&source, ParseOptions::script()).unwrap());
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.unwrap();
            let units: Vec<_> = script.prepared_script_units().collect();
            assert_eq!(units.len(), 2, "direct and indirect intrinsic branches");
            for unit in units {
                assert!(unit.body.statements.is_empty(), "count={count}");
            }
        }
    }
}

#[test]
fn prepared_eval_nonempty_blocks_keep_their_lexical_environment() {
    let source = "eval('{ let retained = 17; {} {} retained; }');";
    let program = lower(&parse(source, ParseOptions::script()).unwrap());
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let unit = script
        .prepared_script_units()
        .find(|unit| matches!(unit.kind, lila_ir::PreparedScriptKind::DirectEval(_)))
        .expect("the direct eval branch retains its observable lexical records");
    let [lila_ir::StatementIr::Block(block)] = unit.body.statements.as_slice() else {
        panic!("a nonempty lexical block must remain: {:?}", unit.body);
    };
    let environment = block.lexical_environment.as_ref().unwrap();
    assert!(!environment.bindings.is_empty());
    assert_eq!(
        block
            .statements
            .iter()
            .filter(|statement| matches!(statement, lila_ir::StatementIr::Empty))
            .count(),
        0
    );
    assert!(matches!(
        block.statements.last(),
        Some(lila_ir::StatementIr::Expression(_))
    ));
}

#[test]
fn prepared_direct_eval_empty_try_blocks_keep_the_catch_parameter_record() {
    let program = lower(
        &parse(
            "eval('try {} catch (error) {} finally {}');",
            ParseOptions::script(),
        )
        .unwrap(),
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    let unit = script
        .prepared_script_units()
        .find(|unit| matches!(unit.kind, lila_ir::PreparedScriptKind::DirectEval(_)))
        .unwrap();
    let [lila_ir::StatementIr::TryCatchFinally {
        try_block,
        catch_parameter_environment,
        catch_block,
        finally_block,
        ..
    }] = unit.body.statements.as_slice()
    else {
        panic!("try control owner must remain: {:?}", unit.body);
    };
    assert!(catch_parameter_environment.is_some());
    for block in [try_block, catch_block, finally_block] {
        assert!(block.statements.is_empty());
        assert!(block.lexical_environment.is_none());
    }
}

#[test]
fn inert_statement_lists_retain_constant_ir_across_root_function_block_and_switch() {
    use lila_ir::StatementIr;
    let mut baseline = None;
    for count in [1, 128, 4096] {
        let empty = "{};".repeat(count);
        let source = format!(
            "{empty} function owner(selector){{ {empty} {{{empty}}} switch(selector){{case 0:{empty}}} }}"
        );
        let lowered = lower(&parse(source, ParseOptions::script()).unwrap());
        assert!(lowered.is_wasm_supported(), "{:?}", lowered.diagnostics);
        let script = lowered.script.unwrap();
        assert!(script
            .body
            .statements
            .iter()
            .all(|s| !matches!(s, StatementIr::Empty)));
        let owner = script.functions.iter().find(|f| f.name == "owner").unwrap();
        let mut contexts = 0;
        for statement in &owner.body.statements {
            match statement {
                StatementIr::Block(block) => {
                    assert!(block.statements.is_empty());
                    contexts += 1;
                }
                StatementIr::Switch { cases, .. } => {
                    assert_eq!(cases.len(), 1);
                    assert!(cases[0].body.statements.is_empty());
                    contexts += 1;
                }
                _ => assert!(!matches!(statement, StatementIr::Empty)),
            }
        }
        assert_eq!(contexts, 2);
        let retained = (script.body.statements.len(), owner.body.statements.len());
        assert_eq!(retained, *baseline.get_or_insert(retained), "count={count}");
    }
}

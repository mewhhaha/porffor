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

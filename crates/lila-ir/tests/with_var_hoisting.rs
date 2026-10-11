use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, EnvironmentIdentifierOperationIr, EnvironmentIdentifierResolutionStart, ExprIr,
    ForInitIr, GlobalDeclarationSetIr, SpecOperationIr, StatementIr, Strictness, TypedExpr,
};

fn declaration_evaluations<'a>(
    statements: &'a [StatementIr],
    initializers: &mut Vec<&'a TypedExpr>,
) {
    for statement in statements {
        match statement {
            StatementIr::DeclarationEvaluation(expression) => initializers.push(expression),
            StatementIr::Block(block) => {
                declaration_evaluations(&block.statements, initializers);
            }
            StatementIr::LexicalBlock(statements) => {
                declaration_evaluations(statements, initializers);
            }
            StatementIr::For {
                init: Some(ForInitIr::Statements(statements)),
                ..
            } => declaration_evaluations(statements, initializers),
            _ => {}
        }
    }
}

#[test]
fn with_body_vars_participate_in_global_declaration_instantiation() {
    let source = r#"
if (false) {
  with ({}) {
    var direct;
    with ({}) {
      for (var loop = 0; loop < 1; loop++) {
        try { var [element] = [1]; } finally { var final; }
      }
    }
  }
}
"#;
    let program = lower(&parse(source, ParseOptions::script()).unwrap());
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    for name in ["direct", "loop", "element", "final"] {
        assert_eq!(
            script.global_bindings.get(name).unwrap().declarations,
            GlobalDeclarationSetIr::Var,
            "{name} must be declared even when its with statement never executes",
        );
    }
}

#[test]
fn function_with_body_vars_remain_in_the_function_variable_environment() {
    let program = lower(
        &parse(
            "function owner() { with ({}) { var captured = 7; } return function read() { return captured; }; } owner()();",
            ParseOptions::script(),
        )
        .unwrap(),
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let script = program.script.unwrap();
    assert!(script.global_bindings.get("captured").is_none());
    assert!(script
        .functions
        .iter()
        .any(|function| function.name == "owner"));
    assert!(script
        .functions
        .iter()
        .any(|function| function.name == "read"));
}

#[test]
fn with_var_initializer_retains_reference_selection_and_declaration_completion() {
    for declaration in [
        "var selected = delete scope.selected;",
        "for (var selected = delete scope.selected; false;) {}",
    ] {
        let source = format!("var scope = {{ selected: 1 }}; with (scope) {{ {declaration} }}");
        let program = lower(&parse(&source, ParseOptions::script()).unwrap());
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.unwrap();
        let mut initializers = Vec::new();
        declaration_evaluations(&script.body.statements, &mut initializers);
        let [initializer] = initializers.as_slice() else {
            panic!("one variable initializer must retain empty completion: {initializers:?}");
        };
        let ExprIr::EnvironmentIdentifier(identifier) = &initializer.expr else {
            panic!("the Object Environment Record selection must be retained: {initializer:?}");
        };
        assert_eq!(identifier.name, "selected");
        assert_eq!(identifier.strictness, Strictness::Sloppy);
        assert_eq!(
            identifier.resolution_start(),
            EnvironmentIdentifierResolutionStart::GlobalEnvironment,
        );
        let EnvironmentIdentifierOperationIr::AssignWithGlobalFallback { selection, value } =
            &identifier.operation
        else {
            panic!("the initializer must retain its complete with/global Reference");
        };
        assert!(
            matches!(&selection.expr, ExprIr::Conditional { condition, .. }
            if matches!(&condition.expr, ExprIr::SpecOperation {
                operation: SpecOperationIr::WithEnvironmentHasBinding, ..
            })),
            "the Object Environment Record is selected before evaluating the initializer",
        );
        let operands: Vec<_> = identifier.operation.operands().collect();
        assert_eq!(operands, [selection.as_ref(), value.as_ref()]);
        assert!(
            matches!(value.expr, ExprIr::DeleteProperty { .. }),
            "the initializer is evaluated once, after selection: {value:?}",
        );
        assert_eq!(
            script.global_bindings.get("selected").unwrap().declarations,
            GlobalDeclarationSetIr::Var,
        );
    }
}

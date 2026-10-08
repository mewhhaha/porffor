use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, BindingMode, ExprIr, FunctionIr, ResumableRegionProtocolIr, StatementIr, TypedExpr,
};

fn lower_run(source: &str) -> FunctionIr {
    let unit = parse(source, ParseOptions::script()).expect("fixture parses");
    let program = lower(&unit);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .unwrap()
        .functions
        .into_iter()
        .find(|function| function.name == "run")
        .expect("actual run owner")
}

fn binding_owner_rows(items: &[StatementIr]) -> Vec<&StatementIr> {
    fn visit<'a>(items: &'a [StatementIr], rows: &mut Vec<&'a StatementIr>) {
        for item in items {
            rows.push(item);
            match item {
                StatementIr::Block(block) => visit(&block.statements, rows),
                StatementIr::LexicalBlock(items) => visit(items, rows),
                StatementIr::EmptyStatementCompletion(item) => {
                    visit(std::slice::from_ref(item.statement()), rows)
                }
                _ => {}
            }
        }
    }
    let mut rows = Vec::new();
    visit(items, &mut rows);
    rows
}

#[test]
fn simple_loop_bindings_use_one_semantic_initialization_instead_of_cloned_gets() {
    for mode in ["let", "const", "var"] {
        for protocol in ["of", "in"] {
            let function = lower_run(&format!(
                "function run(source) {{ for ({mode} {{value = fallback()}} {protocol} source) {{ value; }} }}"
            ));
            let body = function
                .body
                .statements
                .iter()
                .find_map(|statement| match statement {
                    StatementIr::ForOfIterator { body, .. }
                    | StatementIr::ForInObject { body, .. } => Some(body),
                    _ => None,
                })
                .expect("actual loop owner");
            let StatementIr::Block(block) = body.as_ref() else {
                panic!("head prefix");
            };
            let StatementIr::DeclarationEvaluation(TypedExpr {
                expr: ExprIr::ObjectDestructure { pattern, .. },
                ..
            }) = &block.statements[0]
            else {
                panic!("one semantic BindingInitialization");
            };
            let [property] = pattern.properties.as_slice() else {
                panic!("{pattern:?}");
            };
            assert!(matches!(
                &property.default.as_ref().expect("one fallback").expr,
                ExprIr::CallNamed { .. } | ExprIr::CallIndirect { .. }
            ));
        }
    }
}

#[test]
fn simple_async_patterns_initialize_before_suspension_with_separate_value_storage() {
    for (mode, expected) in [
        ("let", BindingMode::Let),
        ("const", BindingMode::Const),
        ("var", BindingMode::Var),
    ] {
        let function = lower_run(&format!(
            "async function run(source) {{ for ({mode} {{selected = 7}} of source) {{ await 0; selected; }} }}"
        ));
        let plan = binding_owner_rows(&function.body.statements)
            .into_iter()
            .find_map(|statement| match statement {
                StatementIr::AsyncGeneratorForOf(plan) => Some(plan),
                _ => None,
            })
            .expect("actual async iterator plan");
        assert_eq!(plan.execution(), ResumableRegionProtocolIr::Async);
        assert!(function
            .owned_env_bindings
            .contains(plan.incoming_binding()));
        assert!(plan.initialization().end_state() < plan.body().entry_state());
        let statements = &plan.initialization().block().statements;
        let [StatementIr::DeclarationEvaluation(TypedExpr {
            expr: ExprIr::ObjectDestructure { value, pattern },
            ..
        })] = statements.as_slice()
        else {
            panic!("one initialization before await: {statements:?}");
        };
        assert!(
            matches!(&value.expr, ExprIr::Identifier(name) if name == &plan.incoming_binding().name)
        );
        assert!(binding_owner_rows(&plan.body().block().statements).into_iter().any(|statement|
            matches!(statement, StatementIr::AsyncAwait { suspend_state, resume_state, .. }
                if *suspend_state >= plan.body().entry_state() && *resume_state <= plan.body().end_state())));
        let mut targets = Vec::new();
        pattern.visit_bindings(&mut |mode, name| targets.push((mode, name.to_owned())));
        assert_eq!(targets.len(), 1);
        assert_eq!(targets[0].0, expected);
        assert_ne!(targets[0].1, plan.incoming_binding().name);
        match expected {
            BindingMode::Var => {
                assert!(plan.lexical_environment().is_none());
                assert!(function
                    .owned_env_bindings
                    .iter()
                    .any(|binding| binding.name == targets[0].1));
            }
            BindingMode::Let | BindingMode::Const => {
                let environment = plan
                    .lexical_environment()
                    .unwrap()
                    .iteration_environment
                    .as_ref()
                    .expect("fresh lexical cell");
                assert_eq!(environment.bindings.len(), 1);
                assert_eq!(environment.bindings[0].name, targets[0].1);
                assert!(!function
                    .owned_env_bindings
                    .iter()
                    .any(|binding| binding.name == targets[0].1));
            }
        }
    }
}

use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ExprIr, FunctionIr, StatementIr, Strictness, TypedExpr};

fn values(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("optional region source parses");
    let program = lower(&parsed);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program
        .script
        .unwrap()
        .functions
        .into_iter()
        .find(|function| function.name == "values" || function.name.ends_with(".values"))
        .unwrap()
}

fn walk<'a>(source: &'a [StatementIr], result: &mut Vec<&'a StatementIr>) {
    for statement in source {
        result.push(statement);
        match statement {
            StatementIr::EmptyStatementCompletion(item) => {
                walk(std::slice::from_ref(item.statement()), result)
            }
            StatementIr::Block(block) => walk(&block.statements, result),
            StatementIr::LexicalBlock(statements) => walk(statements, result),
            StatementIr::If {
                then_branch,
                else_branch,
                ..
            } => {
                walk(std::slice::from_ref(then_branch.as_ref()), result);
                if let Some(branch) = else_branch {
                    walk(std::slice::from_ref(branch.as_ref()), result);
                }
            }
            StatementIr::OrdinaryGeneratorIf(branch) => {
                walk(&branch.then_branch().block().statements, result);
                walk(&branch.else_branch().block().statements, result);
            }
            _ => {}
        }
    }
}

fn expression(statement: &StatementIr) -> Option<&TypedExpr> {
    match statement {
        StatementIr::Lexical { init, .. } => Some(init),
        StatementIr::Expression(value) | StatementIr::Return(value) => Some(value),
        _ => None,
    }
}

#[test]
fn complete_optional_base_precedes_disjoint_multi_yield_key_and_argument_regions() {
    let function = values("function* values(object) { return (yield 1, yield 2)?.[yield 3, yield 4]?.method(yield* object); }");
    let plan = function.generator_plan.as_ref().unwrap();
    assert_eq!(plan.suspension_points.len(), 5);
    assert_eq!(
        (
            plan.suspension_points[0].suspend_state,
            plan.suspension_points[0].resume_state
        ),
        (0, 1)
    );
    assert_eq!(
        (
            plan.suspension_points[1].suspend_state,
            plan.suspension_points[1].resume_state
        ),
        (1, 2)
    );
    let mut statements = Vec::new();
    walk(&function.body.statements, &mut statements);
    let branches = statements
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::OrdinaryGeneratorIf(branch) => Some(branch.as_ref()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(branches.len(), 2);
    for branch in branches {
        assert!(branch.entry_state() >= 2);
        assert_eq!(branch.then_branch().entry_state(), branch.entry_state() + 1);
        assert_eq!(
            branch.else_branch().entry_state(),
            branch.then_branch().end_state() + 1
        );
        assert_eq!(branch.exit_state(), branch.else_branch().end_state() + 1);
        let mut skipped = Vec::new();
        walk(&branch.then_branch().block().statements, &mut skipped);
        assert!(!skipped
            .iter()
            .any(|statement| matches!(statement, StatementIr::GeneratorYield { .. })));
        let mut selected = Vec::new();
        walk(&branch.else_branch().block().statements, &mut selected);
        assert!(selected
            .iter()
            .any(|statement| matches!(statement, StatementIr::GeneratorYield { .. })));
    }
    assert!(!statements
        .iter()
        .any(|statement| matches!(statement, StatementIr::GeneratorIf { .. })));
    let mut slots = std::collections::BTreeSet::new();
    for binding in function.owned_env_bindings.iter().filter(|binding| {
        binding.name.starts_with("$generator.") || binding.name.starts_with("$call.")
    }) {
        assert!(
            slots.insert(binding.slot),
            "retained operands must not alias activation cells"
        );
    }
}

#[test]
fn ordinary_and_suspended_optional_delete_keep_the_terminal_property_reference() {
    let ordinary =
        values("function values(object, key) { 'use strict'; return delete (((object?.[key]))); }");
    let mut statements = Vec::new();
    walk(&ordinary.body.statements, &mut statements);
    let deletion = statements
        .iter()
        .filter_map(|statement| expression(statement))
        .find_map(|value| match &value.expr {
            ExprIr::DeleteOptionalPropertyChain(reference) => Some(reference.as_ref()),
            _ => None,
        })
        .expect("real terminal Delete carrier");
    assert!(deletion.prefix().is_empty());
    assert!(deletion.shorted());
    assert_eq!(deletion.strictness(), Strictness::Strict);
    let suspended =
        values("function* values(object) { return delete ((yield object)?.[yield 'key']); }");
    let mut statements = Vec::new();
    walk(&suspended.body.statements, &mut statements);
    assert_eq!(
        suspended
            .generator_plan
            .as_ref()
            .unwrap()
            .suspension_points
            .len(),
        2
    );
    let expressions = statements
        .iter()
        .filter_map(|statement| expression(statement))
        .collect::<Vec<_>>();
    assert_eq!(
        expressions
            .iter()
            .filter(|value| matches!(value.expr, ExprIr::DeleteProperty { .. }))
            .count(),
        1
    );
    assert!(!expressions.iter().any(|value| matches!(
        value.expr,
        ExprIr::OptionalPropertyChain { .. } | ExprIr::CaptureOptionalCallReference(_)
    )));
}

#[test]
fn nested_optional_regions_keep_private_link_super_and_async_boundaries_explicit() {
    for source in [
        "function* values(object, flag) { return (yield object)?.method(flag ? (yield 1, yield 2) : (yield* [3])); }",
        "function* values(object) { return delete ((yield object)?.method(yield 1, yield 2)); }",
        "function* values(object) { return (yield object)?.[object?.[yield 1]]; }",
        "function* values(object) { return ((yield object)?.[yield 'method'])(yield 1); }",
        "async function* values(object) { return object?.method(yield await 1); }",
        "class C { #method; *values(object) { return object?.#method(yield 1); } }",
        "class C extends Object { *values() { return super.method?.(yield 1); } }",
    ] { values(source); }
    for source in [
        "function* values(object, list) { for (const item of list) { object?.method(yield item); } }",
    ] {
        assert!(!values(source).generator_plan.unwrap().suspension_points.is_empty());
    }
}

use std::collections::BTreeSet;

use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ExprIr, FunctionIr, GeneratorResumeModeIr, StatementIr, TypedExpr, YieldForm,
};

fn values(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("generator branch source parses");
    let program = lower(&parsed);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    program
        .script
        .expect("script")
        .functions
        .into_iter()
        .find(|function| function.name == "values")
        .expect("ordinary generator")
}

fn flatten<'a>(source: &'a [StatementIr], statements: &mut Vec<&'a StatementIr>) {
    for statement in source {
        match statement {
            StatementIr::EmptyStatementCompletion(item) => {
                flatten(std::slice::from_ref(item.statement()), statements)
            }
            StatementIr::LexicalBlock(source) => flatten(source, statements),
            StatementIr::Block(source) => flatten(&source.statements, statements),
            StatementIr::OrdinaryGeneratorIf(source) => {
                statements.push(statement);
                flatten(&source.then_branch().block().statements, statements);
                flatten(&source.else_branch().block().statements, statements);
            }
            StatementIr::If {
                then_branch,
                else_branch,
                ..
            } => {
                statements.push(statement);
                flatten(std::slice::from_ref(then_branch.as_ref()), statements);
                if let Some(branch) = else_branch {
                    flatten(std::slice::from_ref(branch.as_ref()), statements);
                }
            }
            statement => statements.push(statement),
        }
    }
}

fn publication(statements: &[StatementIr]) -> &str {
    let Some(StatementIr::Expression(TypedExpr {
        expr: ExprIr::AssignIdentifier { name, .. },
        ..
    })) = statements.last()
    else {
        panic!("completed arm publishes to its result cell");
    };
    name
}

fn owned_slot(function: &FunctionIr, name: &str) -> u32 {
    let bindings = function
        .owned_env_bindings
        .iter()
        .filter(|binding| binding.name == name)
        .collect::<Vec<_>>();
    assert_eq!(
        bindings.len(),
        1,
        "{name}: {:?}",
        function.owned_env_bindings
    );
    bindings[0].slot
}

#[test]
fn selected_resumes_and_fresh_joins_match_the_source_plan_across_all_value_routes() {
    let function = values(
        "function* values(flag, left, base, f) {
        let a = flag ? (yield 1) : (yield 2);
        const b = left || (yield 3);
        var c = base?.[(yield 4)].value;
        (left ?? (yield 5));
        return f(flag ? (yield 6) : 0, (yield 7));
    }",
    );
    let plan = function
        .generator_plan
        .as_ref()
        .expect("generator source plan");
    assert_eq!(plan.entry_state, 0);
    assert_eq!(plan.state_count, 23);
    assert_eq!(
        plan.suspension_points
            .iter()
            .map(|point| (point.suspend_state, point.resume_state))
            .collect::<Vec<_>>(),
        [
            (1, 2),
            (3, 4),
            (7, 8),
            (11, 12),
            (14, 15),
            (18, 19),
            (21, 22)
        ]
    );

    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let mut complete_layouts = Vec::new();
    let mut result_names = BTreeSet::new();
    let mut received_names = BTreeSet::new();
    for branch in &statements {
        if let StatementIr::OrdinaryGeneratorIf(source) = branch {
            complete_layouts.push((
                source.entry_state(),
                source.then_branch().entry_state(),
                source.then_branch().end_state(),
                source.else_branch().entry_state(),
                source.else_branch().end_state(),
                source.exit_state(),
            ));
            let then_result = publication(&source.then_branch().block().statements);
            let else_result = publication(&source.else_branch().block().statements);
            assert_eq!(then_result, else_result);
            assert!(then_result.starts_with("$generator.branch.result."));
            assert!(result_names.insert(then_result.to_string()));
            owned_slot(&function, then_result);
            assert!(
                statements.iter().any(|statement| matches!(statement,
                StatementIr::Lexical { name, init: TypedExpr { expr: ExprIr::Undefined, .. }, .. }
                    if name == then_result)),
                "both complete arms share an eager result cell"
            );
            for arm in [source.then_branch(), source.else_branch()] {
                let mut body = Vec::new();
                flatten(&arm.block().statements, &mut body);
                for (index, statement) in body.iter().enumerate() {
                    if let StatementIr::GeneratorYield {
                        form: YieldForm::Plain,
                        suspend_state,
                        resume_state,
                        resume_mode: GeneratorResumeModeIr::AssignIdentifier(received),
                        ..
                    } = statement
                    {
                        assert!(*suspend_state >= arm.entry_state());
                        assert!(*resume_state <= arm.end_state());
                        assert!(received_names.insert(received.clone()));
                        owned_slot(&function, received);
                        assert!(body[..index].iter().any(|statement| matches!(statement,
                            StatementIr::Lexical { name, .. } if name == received)));
                        assert!(!body[..=index].iter().any(|statement| matches!(statement,
                            StatementIr::Expression(TypedExpr { expr: ExprIr::AssignIdentifier { name, .. }, .. })
                                if name == then_result)), "publication follows Normal resumption");
                    }
                }
            }
            continue;
        }
        assert!(
            !matches!(branch, StatementIr::GeneratorIf { .. }),
            "every selected value operand owns complete ordinary regions"
        );
    }
    assert_eq!(
        complete_layouts,
        [
            (0, 1, 2, 3, 4, 5),
            (5, 6, 6, 7, 8, 9),
            (9, 10, 10, 11, 12, 13),
            (13, 14, 15, 16, 16, 17),
            (17, 18, 19, 20, 20, 21),
        ]
    );
    assert_eq!(received_names.len(), 6);
    let saved = statements
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::Lexical { name, .. } if name.starts_with("$generator.branch.saved.") => {
                Some(name)
            }
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        saved.len(),
        8,
        "selectors, left values, optional base, retained skipped value and both completed Gets are captured once"
    );
    let slots = result_names
        .iter()
        .chain(received_names.iter())
        .map(|name| owned_slot(&function, name))
        .chain(saved.into_iter().map(|name| owned_slot(&function, name)))
        .collect::<BTreeSet<_>>();
    assert_eq!(
        slots.len(),
        19,
        "selected results, retained values and per-arm received cells cannot alias"
    );
    assert!(statements.iter().any(|statement| matches!(
        statement,
        StatementIr::GeneratorYield {
            suspend_state: 21,
            resume_state: 22,
            ..
        }
    )));
}

#[test]
fn existing_completed_reference_owners_stage_values_without_replaying_the_reference() {
    for source in [
        "function* values(flag, target) { return target.method(flag ? (yield 1) : 0); }",
        "function* values(flag, Constructor) { return new Constructor(flag ? 0 : (yield 1)); }",
        "function* values(flag, tag) { return tag`${flag ? (yield 1) : 0}`; }",
        "function* values(left, target, key) { return target[key] = left && (yield 1); }",
        "function* values(base) { return (base?.[(yield 1)]).method(2); }",
        "function* values(flag) { { var result = flag ? (yield 1) : 0; } return result; }",
        "function* values(flag, left, base) { try { var result = left || (yield 1); return result; } catch (error) { return error; } finally { 0; } }",
        "function* values(flag, base) { try { const result = base?.[(yield 1)].value; return result; } finally { return 2; } }",
        "function* values(base) { return (base?.[(yield 1)])(2); }",
        "function* values(base) { return (base?.[(yield 1)])`tag`; }",
    ] {
        let function = values(source);
        assert_eq!(function.generator_plan.as_ref().expect("plan").suspension_points.len(), 1, "{source}");
    }
}

#[test]
fn complete_selectors_and_arms_own_multiple_nested_and_delegated_suspensions() {
    for source in [
        "function* values(flag) { return (yield flag) ? (yield 1) : 0; }",
        "function* values(flag) { return (yield flag) || (yield 1); }",
        "function* values(flag) { return flag ? f((yield 1), (yield 2)) : 0; }",
        "function* values(flag) { return flag ? (yield* [1]) : 0; }",
        "function* values(flag) { return flag ? (yield (yield 1)) : 0; }",
        "function* values(flag, other) { return flag ? (other ? (yield 1) : 0) : 0; }",
        "function* values(flag) { return flag ? (side(), (yield 1)) : 0; }",
        "function* values(left) { left ||= (yield 1); }",
        "function* values(flag) { if (flag ? (yield 1) : 0) return 2; }",
        "function* values(flag) { if (flag) { const value = flag ? (yield 1) : 0; } }",
        "function* values(flag) { while (flag ? (yield 1) : 0) { yield 2; } }",
        "function* values(flag) { for (let value = flag ? (yield 1) : 0; value; ) { yield 2; } }",
        "function* values(flag) { for (; flag || (yield 1); ) { yield 2; } }",
        "function* values(flag) { for (; flag; flag && (yield 1)) { yield 2; } }",
        "function* values(flag) { while (flag) { flag ? (yield 1) : 0; } }",
        "function* values(flag) { for (; flag; ) { const value = flag || (yield 1); } }",
        "function* values(flag) { do { flag && (yield 1); } while (flag); }",
    ] {
        let function = values(source);
        assert!(
            !function
                .generator_plan
                .as_ref()
                .expect("plan")
                .suspension_points
                .is_empty(),
            "{source}"
        );
    }
}

#[test]
fn unowned_selectors_references_protocols_and_iterator_regions_remain_refused() {
    // The original ordered source cohort retains explicit acceptance and
    // refusal obligations as complete optional regions acquire real consumers.
    for (source, supported) in [
        ("function* values(base) { return (yield base)?.[(yield 1)]; }", true),
        ("function* values(base) { return delete base?.[(yield 1)]; }", true),
        ("function* values(flag, list) { for (const item of list) { const value = flag ? (yield 1) : 0; } }", true),
        ("async function* values(flag) { return flag ? (yield 1) : 0; }", true),
        ("async function* values(flag) { return flag ? (yield await 1) : 0; }", true),
    ] {
        if supported {
            values(source);
            continue;
        }
        let parsed = parse(source, ParseOptions::script()).expect("refused generator source parses");
        let program = lower(&parsed);
        assert!(!program.is_wasm_supported(), "incorrectly admitted {source}");
        assert!(program.diagnostics.iter().any(|diagnostic|
            diagnostic.message.contains("generator") || diagnostic.message.contains("suspension")),
            "{source}: {:?}", program.diagnostics);
    }
}

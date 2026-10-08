use lila_front::{parse, ParseOptions};

#[path = "common/statement_awaits.rs"]
mod statement_awaits;
use lila_ir::{
    lower, AsyncFunctionIfPlanIr, EqualityBinaryOp, ExprIr, FunctionIr, LogicalBinaryOp,
    StatementIr, TypedExpr,
};

fn function(source: &str) -> FunctionIr {
    let source = parse(source, ParseOptions::script()).expect("logical await source parses");
    let program = lower(&source);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .expect("script")
        .functions
        .into_iter()
        .find(|function| function.name == "choose")
        .expect("async function")
}

fn flatten<'a>(statements: &'a [StatementIr], result: &mut Vec<&'a StatementIr>) {
    for statement in statements {
        match statement {
            StatementIr::EmptyStatementCompletion(item) => {
                flatten(std::slice::from_ref(item.statement()), result)
            }
            StatementIr::LexicalBlock(statements) => flatten(statements, result),
            StatementIr::Block(block) => flatten(&block.statements, result),
            statement => result.push(statement),
        }
    }
}

fn branch_write(branch: &StatementIr) -> (&str, &TypedExpr) {
    let StatementIr::LexicalBlock(statements) = branch else {
        panic!("checked branch prefix");
    };
    let Some(StatementIr::Expression(TypedExpr {
        expr: ExprIr::AssignIdentifier { name, value },
        ..
    })) = statements.last()
    else {
        panic!("branch commits one result");
    };
    (name, value)
}

fn branch<'a>(
    statements: &[&'a StatementIr],
) -> (
    &'a TypedExpr,
    &'a StatementIr,
    &'a StatementIr,
    AsyncFunctionIfPlanIr,
) {
    statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncFunctionIf {
                condition,
                then_branch,
                else_branch: Some(else_branch),
                plan,
            } => Some((condition, then_branch.as_ref(), else_branch.as_ref(), *plan)),
            _ => None,
        })
        .expect("logical value owns branches")
}

#[test]
fn all_three_operators_publish_the_original_captured_left_in_the_correct_skipped_arm() {
    for (operator, skipped_then, expected_states) in [
        ("&&", false, [0, 1, 3, 4]),
        ("||", true, [0, 1, 2, 4]),
        ("??", false, [0, 1, 3, 4]),
    ] {
        let function = function(&format!("async function choose(value, rhs) {{ const result = value {operator} await rhs; await 0; return result; }}"));
        let mut statements = Vec::new();
        flatten(&function.body.statements, &mut statements);
        let (condition, then_branch, else_branch, plan) = branch(&statements);
        assert_eq!(
            [
                plan.entry_state(),
                plan.then_entry_state(),
                plan.else_entry_state(),
                plan.exit_state()
            ],
            expected_states
        );
        let captured = statements
            .iter()
            .filter_map(|statement| match statement {
                StatementIr::Lexical {
                    name,
                    init:
                        TypedExpr {
                            expr: ExprIr::Identifier(original),
                            ..
                        },
                    ..
                } if name.starts_with("$async.logical.left.") && original == "value" => Some(name),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(captured.len(), 1);
        let left = captured[0];
        let (result, skipped_value) = branch_write(if skipped_then {
            then_branch
        } else {
            else_branch
        });
        assert_eq!(branch_write(then_branch).0, branch_write(else_branch).0);
        assert!(matches!(&skipped_value.expr, ExprIr::Identifier(name) if name == left));
        let StatementIr::LexicalBlock(skipped) = (if skipped_then {
            then_branch
        } else {
            else_branch
        }) else {
            unreachable!()
        };
        assert_eq!(skipped.len(), 1, "skipped arm has no Await/Promise segment");
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| &binding.name == left));
        assert!(function
            .owned_env_bindings
            .iter()
            .any(|binding| binding.name == result));
        if operator != "??" {
            assert!(matches!(&condition.expr, ExprIr::Identifier(name) if name == left));
        } else {
            let ExprIr::LogicalShortCircuit {
                op: LogicalBinaryOp::Or,
                lhs,
                rhs,
            } = &condition.expr
            else {
                panic!("pure nullish predicate");
            };
            for (comparison, undefined) in [(lhs.as_ref(), false), (rhs.as_ref(), true)] {
                let ExprIr::StrictEquality {
                    op: EqualityBinaryOp::StrictEqual,
                    lhs,
                    rhs,
                } = &comparison.expr
                else {
                    panic!("strict saved-value test");
                };
                assert!(matches!(&lhs.expr, ExprIr::Identifier(name) if name == left));
                assert!(if undefined {
                    matches!(rhs.expr, ExprIr::Undefined)
                } else {
                    matches!(rhs.expr, ExprIr::Null)
                });
            }
        }
        assert!(statements.iter().any(|statement| matches!(
            statement,
            StatementIr::AsyncAwait {
                suspend_state: 4,
                resume_state: 5,
                ..
            }
        )));
    }
}

#[test]
fn awaited_left_completes_before_the_saved_value_and_rhs_branch_entry() {
    let function =
        function("async function choose(lhs, rhs) { return (await lhs()) ?? await rhs(); }");
    let mut statements = Vec::new();
    flatten(&function.body.statements, &mut statements);
    let (_, _, _, plan) = branch(&statements);
    assert_eq!(plan.entry_state(), 1);
    let left_await = statements
        .iter()
        .position(|statement| {
            matches!(
                statement,
                StatementIr::AsyncAwait {
                    suspend_state: 0,
                    resume_state: 1,
                    ..
                }
            )
        })
        .expect("left Await");
    let capture = statements
        .iter()
        .position(|statement| {
            matches!(statement,
        StatementIr::Lexical { name, .. } if name.starts_with("$async.logical.left."))
        })
        .expect("completed left cell");
    let selection = statements
        .iter()
        .position(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. }))
        .expect("RHS branch selection");
    assert!(left_await < capture && capture < selection);
}

#[test]
fn nested_logical_conditional_and_invocation_owners_compose() {
    for source in [
        "async function choose(flag, lhs, rhs, f) { return f(lhs || (flag ? await rhs : (lhs ?? await 2))); }",
        "async function choose(lhs, C) { return new C(lhs ?? (false || await 2)); }",
        "async function choose(lhs, tag) { return tag`${lhs && (true ? await 1 : 2)}`; }",
        "async function choose(lhs) { var result = await (lhs || await 1); return result; }",
        "async function choose(lhs) { if (lhs && await 1) return 3; return 4; }",
    ] { function(source); }
}

#[test]
fn compound_optional_generator_and_loop_owners_remain_explicit_boundaries() {
    for (source, awaits) in [
        ("async function choose(lhs) { missing &&= await 1; }", 1),
        (
            "async function choose(lhs) { const result = (lhs[await 0] ??= await 1); }",
            2,
        ),
        ("async function* choose(lhs) { lhs || await 1; }", 1),
        (
            "async function choose(lhs) { while (lhs && await true) { await 0; } }",
            2,
        ),
        (
            "async function choose(lhs) { do { break; } while (lhs ?? await false); }",
            1,
        ),
        (
            "async function choose(lhs) { while (lhs) { lhs || await 1; } }",
            1,
        ),
        (
            "async function choose(lhs) { for (; lhs || await true;) { break; } }",
            1,
        ),
    ] {
        statement_awaits::assert_count(&function(source), awaits);
    }
}

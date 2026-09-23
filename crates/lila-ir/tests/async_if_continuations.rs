use lila_front::{parse, ParseOptions};
use lila_ir::{lower, AsyncFunctionIfPlanIr, FunctionIr, StatementIr};

fn lower_function(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("async conditional parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .expect("script IR")
        .functions
        .into_iter()
        .find(|function| function.name == "choose")
        .expect("conditional function")
}

fn continuations(function: &FunctionIr) -> (Vec<[u32; 4]>, Vec<(u32, u32)>) {
    fn collect(
        statement: &StatementIr,
        plans: &mut Vec<AsyncFunctionIfPlanIr>,
        awaits: &mut Vec<(u32, u32)>,
    ) {
        match statement {
            StatementIr::AsyncFunctionIf {
                then_branch,
                else_branch,
                plan,
                ..
            } => {
                plans.push(*plan);
                collect(then_branch, plans, awaits);
                if let Some(else_branch) = else_branch {
                    collect(else_branch, plans, awaits);
                }
            }
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => awaits.push((*suspend_state, *resume_state)),
            StatementIr::Block(block) => {
                for statement in &block.statements {
                    collect(statement, plans, awaits);
                }
            }
            StatementIr::LexicalBlock(statements) => {
                for statement in statements {
                    collect(statement, plans, awaits);
                }
            }
            _ => {}
        }
    }
    let mut plans = Vec::new();
    let mut awaits = Vec::new();
    for statement in &function.body.statements {
        collect(statement, &mut plans, &mut awaits);
    }
    (
        plans
            .into_iter()
            .map(|plan| {
                [
                    plan.entry_state(),
                    plan.then_entry_state(),
                    plan.else_entry_state(),
                    plan.exit_state(),
                ]
            })
            .collect(),
        awaits,
    )
}

#[test]
fn two_awaiting_branches_own_disjoint_ranges_after_the_preceding_await() {
    let function = lower_function(
        "async function choose(flag) { await 0; if(flag) { let value=await 1; return value+1; } else { let value=await 4; return value+1; } }",
    );
    assert_eq!(
        continuations(&function),
        (vec![[1, 2, 4, 6]], vec![(0, 1), (2, 3), (4, 5)])
    );
}

#[test]
fn an_empty_branch_still_joins_before_the_following_await() {
    let function =
        lower_function("async function choose(flag) { if(flag) {} else await 4; return await 9; }");
    assert_eq!(
        continuations(&function),
        (vec![[0, 1, 2, 4]], vec![(2, 3), (4, 5)])
    );
}

#[test]
fn condition_awaits_precede_branch_states_and_the_join_continues_the_sequence() {
    let function = lower_function(
        "async function choose(flag) { if(await flag) { await 1; } else { await 2; } await 3; }",
    );
    assert_eq!(
        continuations(&function),
        (vec![[1, 2, 4, 6]], vec![(0, 1), (2, 3), (4, 5), (6, 7)])
    );
}

#[test]
fn nested_branches_compose_with_awaits_before_and_after_each_join() {
    let function = lower_function(
        "async function choose(first, second) { if(first) { await 0; if(second) { await 1; } else await 2; await 3; } else { await 4; } await 5; }",
    );
    assert_eq!(
        continuations(&function),
        (
            vec![[0, 1, 9, 11], [2, 3, 5, 7]],
            vec![(1, 2), (3, 4), (5, 6), (7, 8), (9, 10), (11, 12)],
        )
    );
}

#[test]
fn ordinary_branches_do_not_leave_unused_states_before_an_await() {
    let function = lower_function(
        "async function choose(flag) { if(flag) { flag=0; } else { flag=1; } await 3; }",
    );
    assert_eq!(continuations(&function), (vec![], vec![(0, 1)]));
}

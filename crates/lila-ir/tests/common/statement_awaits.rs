use super::{FunctionIr, StatementIr};

// Function metadata need not own plain Async states. Inspect the executable
// statement owners without entering nested callable bodies or flattening an
// operand's selected branch into its parent's unconditional evaluation order.
fn collect(statement: &StatementIr, points: &mut Vec<(u32, u32)>) {
    let before = points.len();
    match statement {
        StatementIr::AsyncAwait {
            suspend_state,
            resume_state,
            ..
        } => {
            assert!(suspend_state < resume_state);
            points.push((*suspend_state, *resume_state));
        }
        StatementIr::EmptyStatementCompletion(item) => collect(item.statement(), points),
        StatementIr::LexicalBlock(statements) => collect_items(statements, points),
        StatementIr::Block(block) => collect_items(&block.statements, points),
        StatementIr::AsyncFunctionIf {
            then_branch,
            else_branch,
            plan,
            ..
        } => {
            collect(then_branch, points);
            assert!(points[before..].iter().all(|&(suspend, resume)| {
                plan.then_entry_state() <= suspend && resume < plan.else_entry_state()
            }));
            let before_else = points.len();
            if let Some(branch) = else_branch {
                collect(branch, points);
            }
            assert!(points[before_else..].iter().all(|&(suspend, resume)| {
                plan.else_entry_state() <= suspend && resume < plan.exit_state()
            }));
        }
        StatementIr::If {
            then_branch,
            else_branch,
            ..
        } => {
            collect(then_branch, points);
            if let Some(branch) = else_branch {
                collect(branch, points);
            }
        }
        StatementIr::AsyncFunctionWhile(plan) => {
            collect_items(plan.condition_prefix(), points);
            collect(plan.body(), points);
        }
        StatementIr::AsyncGeneratorLoop(plan) => {
            for region in plan.regions() {
                let first = points.len();
                collect_items(&region.block().statements, points);
                assert!(points[first..].iter().all(|&(suspend, resume)| {
                    region.entry_state() <= suspend && resume <= region.end_state()
                }));
            }
        }
        StatementIr::AsyncGeneratorIf(plan) => {
            for region in plan.regions() {
                let first = points.len();
                collect_items(&region.block().statements, points);
                assert!(points[first..].iter().all(|&(suspend, resume)| {
                    region.entry_state() <= suspend && resume <= region.end_state()
                }));
            }
        }
        StatementIr::While { body, .. }
        | StatementIr::DoWhile { body, .. }
        | StatementIr::For { body, .. }
        | StatementIr::Labelled {
            statement: body, ..
        } => collect(body, points),
        _ => {}
    }
}

fn collect_items(statements: &[StatementIr], points: &mut Vec<(u32, u32)>) {
    for statement in statements {
        collect(statement, points);
    }
}

pub(super) fn assert_count(function: &FunctionIr, expected: usize) {
    let mut points = Vec::new();
    collect_items(&function.body.statements, &mut points);
    assert_eq!(points.len(), expected, "actual Await owners: {points:?}");
    let resumes = points
        .iter()
        .map(|&(_, resume)| resume)
        .collect::<std::collections::BTreeSet<_>>();
    assert_eq!(resumes.len(), expected, "Await resumes must be distinct");
}

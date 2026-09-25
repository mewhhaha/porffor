use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ResumableLoopIterationEnvironmentIr, StatementIr};

#[test]
fn nested_branch_loop_and_iterator_have_disjoint_resume_intervals() {
    let parsed = parse(
        "function* values(xs) {
            if (xs.length === 0) { yield 0; }
            else { for (let i = 0; i < xs.length; i++) {
                for (let x of xs) { yield i + x; }
            } }
        }",
        ParseOptions::script(),
    )
    .expect("generator parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .expect("script IR")
        .functions
        .into_iter()
        .find(|function| function.name == "values")
        .expect("generator IR");
    let [StatementIr::GeneratorStructuredIf { plan, .. }] = function.body.statements.as_slice()
    else {
        panic!("structured branch expected");
    };
    assert_eq!(
        function
            .generator_plan
            .as_ref()
            .expect("generator plan")
            .state_count,
        plan.exit_state() + 1
    );
    assert_eq!(plan.then_entry_state(), plan.entry_state() + 1);
    assert!(plan.then_exit_state() < plan.else_entry_state());
    assert!(plan.else_exit_state() < plan.exit_state());
    assert!(matches!(plan.then_branch(), StatementIr::Block(_)));
    let StatementIr::Block(else_block) = plan.else_branch().expect("else branch") else {
        panic!("else block expected");
    };
    let StatementIr::GeneratorStructuredLoop {
        plan: loop_plan, ..
    } = &else_block.statements[0]
    else {
        panic!("structured classic for expected");
    };
    assert_eq!(loop_plan.entry_state(), plan.else_entry_state());
    let StatementIr::Block(loop_block) = loop_plan.body() else {
        panic!("loop block expected");
    };
    let StatementIr::GeneratorForOfIterator {
        plan: iterator_plan,
        ..
    } = loop_block.statements.last().expect("for-of")
    else {
        panic!("resumable for-of expected");
    };
    assert_eq!(iterator_plan.entry_state(), loop_plan.body_entry_state());
    assert!(iterator_plan.body_exit_state() < iterator_plan.exit_state());
    assert_eq!(iterator_plan.exit_state(), loop_plan.body_exit_state());
}

#[test]
fn captured_for_of_head_owns_a_fresh_iteration_environment() {
    let parsed = parse(
        "function* values() { for (let x of [3, 4]) { const read = () => x; yield read; } }",
        ParseOptions::script(),
    )
    .expect("generator parses");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .expect("script IR")
        .functions
        .into_iter()
        .find(|function| function.name == "values")
        .expect("generator IR");
    let StatementIr::GeneratorForOfIterator { plan, .. } = &function.body.statements[0] else {
        panic!("resumable iterator expected");
    };
    assert_eq!(
        function
            .generator_plan
            .as_ref()
            .expect("generator plan")
            .state_count,
        plan.exit_state() + 1
    );
    assert!(matches!(
        plan.iteration_environment(),
        ResumableLoopIterationEnvironmentIr::FreshPerIteration(_)
    ));
}

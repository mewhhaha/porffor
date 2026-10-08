use lila_front::{parse, ParseOptions};
use lila_ir::{lower, FunctionIr, StatementIr};

fn function(source: &str) -> FunctionIr {
    let source = parse(source, ParseOptions::script()).expect("async while source parses");
    let program = lower(&source);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .expect("script")
        .functions
        .into_iter()
        .find(|function| function.name == "run")
        .expect("run function")
}

#[test]
fn condition_prefix_is_activation_owned_and_following_await_starts_at_exit() {
    let function =
        function("async function run() { while (await await true) { break; } await 3; }");
    let plan = function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncFunctionWhile(plan) => Some(plan),
            _ => None,
        })
        .expect("checked awaited condition");
    assert_eq!(plan.ready_state() - plan.entry_state(), 2);
    assert!(function
        .body
        .statements
        .iter()
        .any(|statement| matches!(statement,
        StatementIr::AsyncAwait { suspend_state, .. } if *suspend_state == plan.exit_state())));
    assert!(plan
        .condition_prefix()
        .iter()
        .filter_map(|statement| match statement {
            StatementIr::AsyncAwait {
                resume_mode: lila_ir::AsyncResumeModeIr::AssignIdentifier(name),
                ..
            } => Some(name),
            _ => None,
        })
        .all(|name| function
            .owned_env_bindings
            .iter()
            .any(|binding| &binding.name == name)));
}

#[test]
fn eager_finally_stays_on_the_real_break_and_continue_completion_path() {
    let function = function(
        "async function run() { while (await true) { try { continue; } finally { break; } } }",
    );
    let plan = function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncFunctionWhile(plan) => Some(plan),
            _ => None,
        })
        .expect("eager completion body accepted");
    let body = match plan.body() {
        StatementIr::Block(block) => &block.statements,
        other => panic!("expected source block, got {other:?}"),
    };
    assert!(body.iter().any(|statement| matches!(
        statement,
        StatementIr::TryFinally {
            async_plan: None,
            generator_plan: None,
            ..
        }
    )));
}

#[test]
fn awaited_body_and_other_loop_heads_keep_their_explicit_boundaries() {
    for source in [
        "async function run() { while (await true) { await 1; } }",
        "async function run() { for (; await true;) { break; } }",
        "async function run() { do { break; } while (await true); }",
    ] {
        let parsed = parse(source, ParseOptions::script()).expect("valid source");
        let program = lower(&parsed);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
    }
}

#[test]
fn branch_conditions_own_their_join_and_the_following_await() {
    for source in [
        "async function run(flag) { while (flag && await true) { break; } await 0; }",
        "async function run(flag) { while (flag || await true) { break; } await 0; }",
        "async function run(flag) { while (flag ?? await true) { break; } await 0; }",
        "async function run(flag) { while (flag ? await true : await false) { break; } await 0; }",
        "async function run(object) { while (object?.[await 'ready']) { break; } await 0; }",
        "async function run(flag, object) { while ((await flag) ? (object?.[await 'ready'] ?? await false) : (false || await true)) { break; } await 0; }",
    ] {
        let function = function(source);
        let plan = function.body.statements.iter().find_map(|statement| match statement {
            StatementIr::AsyncFunctionWhile(plan) => Some(plan),
            _ => None,
        }).expect("checked branch condition");
        assert!(plan.condition_prefix().iter().any(|statement| matches!(statement,
            StatementIr::AsyncFunctionIf { .. })));
        assert!(plan.ready_state() > plan.entry_state());
        assert_eq!(plan.exit_state(), plan.ready_state() + 1);
        assert!(function.body.statements.iter().any(|statement| matches!(statement,
            StatementIr::AsyncAwait { suspend_state, .. } if *suspend_state == plan.exit_state())));
        assert!(!function.owned_env_bindings.is_empty(), "branch values survive re-entry");
    }
}

#[test]
fn erased_optional_awaits_keep_eager_prefixes_inside_the_restartable_condition() {
    let function = function(
        "async function run() { let n = 0; while ((++n < 3) && (null?.[await missing()] === undefined)) { continue; } await 0; }",
    );
    let plan = function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncFunctionWhile(plan) => Some(plan),
            _ => None,
        })
        .expect("eager condition prefix has a loop owner");
    assert_eq!(plan.entry_state(), 0);
    assert!(
        plan.ready_state() > plan.entry_state(),
        "the skipped branch retains its checked source phases"
    );
    assert_eq!(plan.exit_state(), plan.ready_state() + 1);
    assert!(
        !plan.condition_prefix().is_empty(),
        "the ++n effect stays inside the loop"
    );
    assert!(plan
        .condition_prefix()
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncFunctionIf { .. })));
    assert!(function.body.statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait { suspend_state, resume_state, .. }
            if *suspend_state == plan.exit_state() && *resume_state == plan.exit_state() + 1
    )));
}

#[test]
fn condition_branch_context_does_not_escape_into_other_loops_or_child_functions() {
    for source in [
        "async function run(flag) { while (flag && await false) { break; } for (; flag && await true;) { break; } }",
        "async function run(flag) { while (flag && await false) { break; } do {} while (flag ? await false : false); }",
        "async function run(flag) { while (flag && await true) { flag && await false; } }",
        "async function run(flag) { while (await (async function child() { for (; flag && await false;) {} })()) {} }",
        "async function run(flag) { while (await (async function child() { do {} while (flag && await false); })()) {} }",
        "async function run(flag) { while (await (async function* child() { while (flag && await false) { yield 1; } })()) {} }",
        "async function run(flag) { ancestor: while (flag) { while (flag && await false) {} continue ancestor; } }",
        "async function run(object) { while (object?.(await true)) { break; } }",
        "async function run(object) { while (object?.[await 'x'](1)) { break; } }",
        "async function run() { let value = true; while (value &&= await false) { break; } }",
        "async function run(object) { while (object.value ??= await false) { break; } }",
        "async function run() { while (class { [await 0]() {} }) { break; } }",
    ] {
        let parsed = parse(source, ParseOptions::script()).expect("valid scoped-boundary source");
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{source}: {:?}", program.diagnostics);
    }
}

#[test]
fn eager_body_conditions_keep_reference_prefixes_and_route_class_lifetimes_to_complete_regions() {
    for source in [
        "async function run(object) { while (object?.[await 'method'](await 1)) { break; } await 0; }",
        "async function run(object) { while (object[await 'value'] &&= await false) { break; } await 0; }",
    ] {
        let function = function(source);
        let plan = function.body.statements.iter().find_map(|statement| match statement {
            StatementIr::AsyncFunctionWhile(plan) => Some(plan), _ => None,
        }).expect("the checked condition prefix retains the Reference owner");
        assert!(plan.ready_state() > plan.entry_state());
        assert!(function.body.statements.iter().any(|statement| matches!(statement,
            StatementIr::AsyncAwait { suspend_state, .. } if *suspend_state == plan.exit_state())));
    }
    let function = function(
        "async function run() { while (class Named { [await 'key']() {} }) { break; } await 0; }",
    );
    let plan = function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncGeneratorLoop(plan) => Some(plan),
            _ => None,
        })
        .expect("the class lifecycle requires the complete loop owner");
    assert_eq!(plan.execution(), lila_ir::ResumableRegionProtocolIr::Async);
    assert!(function
        .body
        .statements
        .iter()
        .any(|statement| matches!(statement,
        StatementIr::AsyncAwait { suspend_state, .. } if *suspend_state == plan.exit_state())));
}

#[test]
fn directly_labelled_body_await_loops_keep_the_established_loop_owner() {
    for source in [
        "async function run() { let n = 0; outer: inner: while (n++ < 1) { await 0; } await 0; }",
        "async function run() { let n = 0; outer: inner: for (; n < 1; ++n) { await 0; } await 0; }",
    ] {
        let function = function(source);
        let statement = function.body.statements.iter().find_map(|statement| match statement {
            StatementIr::Labelled { labels, statement, async_plan: None } if labels.len() == 2 => Some(statement.as_ref()),
            _ => None,
        }).expect("direct loop label chain");
        let StatementIr::AsyncGeneratorLoop(plan) = statement else { panic!("complete awaited-body loop owner"); };
        assert_eq!(plan.execution(), lila_ir::ResumableRegionProtocolIr::Async);
        assert_eq!(plan.entry_state(), 0);
        assert!(plan.exit_state() > plan.entry_state());
        assert!(function.body.statements.iter().any(|statement| matches!(statement,
            StatementIr::AsyncAwait { suspend_state, .. } if *suspend_state == plan.exit_state())));
    }
}

fn label_region(function: &FunctionIr) -> (lila_ir::AsyncFunctionLabelledPlanIr, &StatementIr) {
    function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Labelled {
                statement,
                async_plan: Some(plan),
                ..
            } => Some((*plan, statement.as_ref())),
            _ => None,
        })
        .expect("checked non-loop label region")
}

fn assert_region(source: &str, expected: [u32; 3]) {
    let function = function(source);
    let (plan, _) = label_region(&function);
    assert_eq!(
        [
            plan.entry_state(),
            plan.body_exit_state(),
            plan.exit_state()
        ],
        expected
    );
    assert!(function
        .body
        .statements
        .iter()
        .any(|statement| matches!(statement,
        StatementIr::AsyncAwait { suspend_state, .. } if *suspend_state == plan.exit_state())));
}

#[test]
fn labelled_block_owns_an_exit_before_a_following_await() {
    assert_region(
        "async function run() { outer: { while (await true) { break outer; } } await 0; }",
        [0, 2, 3],
    );
}

#[test]
fn labelled_block_continues_the_earlier_await_and_owns_the_following_one() {
    assert_region(
        "async function run() { await 0; outer: { while (await true) { break outer; } } await 0; }",
        [1, 3, 4],
    );
}

#[test]
fn early_labelled_break_has_a_distinct_exit_even_when_the_condition_is_bypassed() {
    assert_region(
        "async function run() { outer: { break outer; while (await true) {} } await 0; }",
        [0, 2, 3],
    );
}

#[test]
fn owning_labels_reuse_nested_block_branch_and_try_plans() {
    for source in [
        "async function run(flag) { outer: { { while (await true) { break outer; } } } await 0; }",
        "async function run(flag) { outer: { if (flag) { while (await true) { break outer; } } } await 0; }",
        "async function run(flag) { outer: { try { while (await true) { break outer; } } finally {} } await 0; }",
        "async function run(flag) { outer: if (flag) { while (await true) { break outer; } } await 0; }",
        "async function run(flag) { outer: try { while (await true) { break outer; } } finally {} await 0; }",
    ] {
        let function = function(source);
        let (plan, _) = label_region(&function);
        assert_eq!(plan.exit_state(), plan.body_exit_state() + 1);
        assert!(function.body.statements.iter().any(|statement| matches!(statement,
            StatementIr::AsyncAwait { suspend_state, .. } if *suspend_state == plan.exit_state())));
    }
}

#[test]
fn pending_label_break_passes_the_awaited_finalizer_before_the_label_exit() {
    let function = function("async function run() { outer: try { break outer; while (await true) {} } finally { await 0; } await 0; }");
    let (plan, statement) = label_region(&function);
    assert_eq!(
        [
            plan.entry_state(),
            plan.body_exit_state(),
            plan.exit_state()
        ],
        [0, 5, 6]
    );
    let StatementIr::TryFinally {
        async_plan: Some(inner),
        finally_block,
        ..
    } = statement
    else {
        panic!("existing async Try owner must remain explicit");
    };
    assert_eq!(
        (
            inner.entry_state,
            inner.try_exit_state,
            inner.finally_entry_state,
            inner.exit_state
        ),
        (0, 3, Some(3), 5)
    );
    assert!(finally_block.statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait {
            suspend_state: 3,
            resume_state: 4,
            ..
        }
    )));
    assert!(function.body.statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait {
            suspend_state: 6,
            resume_state: 7,
            ..
        }
    )));
}

#[test]
fn a_false_branch_has_its_if_exit_and_a_distinct_owning_label_exit() {
    let function = function("async function run(flag) { outer: if (flag) { while (await true) { break outer; } } await 0; }");
    let (plan, statement) = label_region(&function);
    let StatementIr::AsyncFunctionIf { plan: branch, .. } = statement else {
        panic!("existing If owner");
    };
    assert_eq!(
        (
            branch.entry_state(),
            branch.then_entry_state(),
            branch.else_entry_state(),
            branch.exit_state()
        ),
        (0, 1, 4, 5)
    );
    assert_eq!(
        [
            plan.entry_state(),
            plan.body_exit_state(),
            plan.exit_state()
        ],
        [0, 5, 6]
    );
}

#[test]
fn enclosing_loop_restrictions_remain_explicit_for_ancestor_control() {
    for source in [
        "async function run(flag) { ancestor: while (flag) { region: { while (await false) {} } continue ancestor; } }",
        "async function run(flag) { ancestor: while (flag) { region: { try { while (await false) {} } finally { await 0; break ancestor; } } } }",
        "async function run() { for await (const value of []) { region: { while (await false) {} } } }",
        "async function run() { for (const value of [1]) { region: { while (await false) {} } } }",
        "async function run() { ancestor: while (await true) { region: { while (await false) {} } } }",
    ] {
        let parsed = parse(source, ParseOptions::script()).expect("valid enclosing-loop source");
        let program = lower(&parsed);
        assert!(program.is_wasm_supported(), "{source}: {:?}", program.diagnostics);
    }
}

#[test]
fn direct_label_chains_keep_the_awaited_loop_state_owner() {
    let function = function(
        "async function run() { outer: inner: while (await true) { continue outer; } await 0; }",
    );
    let plan = function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::Labelled {
                labels, statement, ..
            } if labels.len() == 2 => match statement.as_ref() {
                StatementIr::AsyncFunctionWhile(plan) => Some(plan),
                _ => None,
            },
            _ => None,
        })
        .expect("direct label chain retains its checked loop");
    assert!(function.body.statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait { suspend_state, .. } if *suspend_state == plan.exit_state()
    )));
}

#[test]
fn a_nested_function_condition_does_not_belong_to_its_enclosing_label() {
    let parsed = parse(
        "async function run() { outer: { const inner = async function() { while (await true) { break; } }; break outer; } }",
        ParseOptions::script(),
    )
    .expect("nested async function source");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
}

fn assert_suspending_body_has_complete_owner(source: &str) {
    let parsed = parse(source, ParseOptions::script()).expect("valid implicit suspension source");
    let program = lower(&parsed);
    assert!(
        program.is_wasm_supported(),
        "{source}: {:?}",
        program.diagnostics
    );
    let function = program
        .script
        .as_ref()
        .unwrap()
        .functions
        .iter()
        .find(|function| function.name == "run")
        .unwrap();
    assert!(function.body.statements.iter().any(|statement|
        matches!(statement, StatementIr::AsyncGeneratorLoop(plan) if plan.execution() == lila_ir::ResumableRegionProtocolIr::Async)));
}

#[test]
fn implicit_await_using_body_is_refused_before_clearing_continuation_state() {
    assert_suspending_body_has_complete_owner(
        "async function run() { while (await true) { await using resource = null; break; } }",
    );
}

#[test]
fn implicit_for_await_body_is_refused_before_lowering_its_iterator_plan() {
    assert_suspending_body_has_complete_owner(
        "async function run() { while (await true) { for await (const value of []) {} break; } }",
    );
}

#[test]
fn implicit_await_using_iterator_head_is_not_an_eager_body() {
    assert_suspending_body_has_complete_owner(
        "async function run() { while (await true) { for (await using resource of []) {} break; } }",
    );
}

#[test]
fn implicit_await_using_classic_for_head_is_not_an_eager_body() {
    assert_suspending_body_has_complete_owner(
        "async function run() { while (await true) { for (await using resource = null; false;) {} break; } }",
    );
}

#[test]
fn nested_function_implicit_disposal_does_not_suspend_the_eager_body() {
    let parsed = parse(
        "async function run() { while (await true) { const inner = async function() { await using resource = null; }; break; } }",
        ParseOptions::script(),
    )
    .expect("nested independent disposal source");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
}

#[test]
fn switch_case_owns_a_condition_before_a_following_await() {
    let function = function(
        "async function run() { switch (0) { default: while (await true) { break; } } await 0; }",
    );
    assert!(function.body.statements.iter().any(|statement| matches!(statement, StatementIr::LexicalBlock(statements)
        if statements.iter().any(|statement| matches!(statement, StatementIr::AsyncFunctionSwitch(_))))));
}

#[test]
fn early_switch_break_owns_exit_without_scheduling_a_bypassed_condition() {
    let function = function(
        "async function run() { switch (0) { default: break; while (await true) {} } await 0; }",
    );
    assert!(function.body.statements.iter().any(|statement| matches!(statement, StatementIr::LexicalBlock(statements)
        if statements.iter().any(|statement| matches!(statement, StatementIr::AsyncFunctionSwitch(_))))));
}

#[test]
fn nested_function_condition_does_not_belong_to_its_enclosing_switch() {
    let parsed = parse(
        "async function run() { switch (0) { default: const inner = async function() { while (await true) { break; } }; break; } }",
        ParseOptions::script(),
    )
    .expect("nested independent switch function source");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
}

#[test]
fn direct_and_block_await_labels_own_the_following_await() {
    for source in [
        "async function run() { outer: await 0; await 0; }",
        "async function run() { outer: { await 0; } await 0; }",
    ] {
        assert_region(source, [0, 1, 2]);
    }
}

#[test]
fn labelled_if_and_try_retain_their_ordinary_await_plans() {
    assert_region(
        "async function run(flag) { outer: if (flag) { await 0; } await 0; }",
        [0, 4, 5],
    );
    assert_region(
        "async function run() { outer: try { await 0; } finally { await 1; } await 2; }",
        [0, 4, 5],
    );
}

#[test]
fn bypassed_ordinary_awaits_still_reserve_a_label_completion_exit() {
    assert_region(
        "async function run() { outer: { break outer; await 0; } await 0; }",
        [0, 1, 2],
    );
    assert_region(
        "async function run() { outer: { await 0; break outer; await 0; } await 0; }",
        [0, 2, 3],
    );
    assert_region("async function run() { outer: try { break outer; await 0; } finally { await 0; } await 0; }", [0, 4, 5]);
}

#[test]
fn independent_function_awaits_do_not_advance_the_enclosing_label_counter() {
    let function = function("async function run() { outer: { const helper = async function() { await 0; }; break outer; } await 0; }");
    assert!(function.body.statements.iter().any(|statement| matches!(
        statement,
        StatementIr::Labelled {
            async_plan: None,
            ..
        }
    )));
    assert!(function.body.statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait {
            suspend_state: 0,
            resume_state: 1,
            ..
        }
    )));
}

#[test]
fn switch_body_and_head_awaits_are_admitted_while_unowned_phases_are_refused() {
    for source in [
        "async function run() { outer: { switch (0) { default: await 0; } } await 0; }",
        "async function run() { outer: { switch (0) { default: break; await 0; } } await 0; }",
        "async function run() { outer: { switch (await 0) { default: break; } } await 0; }",
    ] {
        function(source);
    }
    for source in [
        "async function run(target) { outer: { switch (0) { case target[await 0] &&= await 0: break; } } await 0; }",
        "async function run() { outer: { switch (0) { default: { await using resource = null; break; } } } await 0; }",
        "async function run() { outer: { switch (0) { default: for await (const value of []) {} break; } } await 0; }",
    ] {
        function(source);
    }
}

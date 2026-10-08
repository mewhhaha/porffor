use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, AsyncDisposableFinalizerPlanIr, AsyncDisposableScopeExecutionIr,
    AsyncFunctionAsyncDisposableCapabilityIr, AsyncFunctionSwitchIr,
    AsyncGeneratorAsyncDisposableCapabilityIr, AsyncGeneratorResourceCapabilityIr,
    AsyncGeneratorSwitchIr, AsyncResumeModeIr, BindingMode, ExprIr, FunctionIr,
    ResumableRegionProtocolIr, StatementIr,
};

fn function(source: &str) -> FunctionIr {
    let parsed = parse(source, ParseOptions::script()).expect("valid async switch source");
    let program = lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .unwrap()
        .functions
        .into_iter()
        .find(|function| function.name == "run")
        .unwrap()
}

fn switches<'a>(statements: &'a [StatementIr], out: &mut Vec<&'a AsyncFunctionSwitchIr>) {
    for statement in statements {
        match statement {
            StatementIr::AsyncFunctionSwitch(plan) => {
                out.push(plan);
                for case in plan.cases() {
                    switches(case.condition_prefix(), out);
                    switches(&case.body().statements, out);
                }
            }
            StatementIr::LexicalBlock(statements) => switches(statements, out),
            StatementIr::Block(block) => switches(&block.statements, out),
            StatementIr::Labelled { statement, .. } => {
                switches(std::slice::from_ref(statement.as_ref()), out)
            }
            _ => {}
        }
    }
}

fn complete_switch(function: &FunctionIr) -> &AsyncGeneratorSwitchIr {
    fn find(statements: &[StatementIr]) -> Option<&AsyncGeneratorSwitchIr> {
        for statement in statements {
            let found = match statement {
                StatementIr::AsyncGeneratorSwitch(plan) => Some(plan.as_ref()),
                StatementIr::Block(block) => find(&block.statements),
                StatementIr::LexicalBlock(statements) => find(statements),
                StatementIr::EmptyStatementCompletion(item) => {
                    find(std::slice::from_ref(item.statement()))
                }
                StatementIr::Labelled { statement, .. } => {
                    find(std::slice::from_ref(statement.as_ref()))
                }
                _ => None,
            };
            if found.is_some() {
                return found;
            }
        }
        None
    }
    let plan = find(&function.body.statements).expect("actual complete async Switch");
    assert_eq!(plan.execution(), ResumableRegionProtocolIr::Async);
    plan
}

enum Disposal<'a> {
    Legacy(&'a AsyncFunctionAsyncDisposableCapabilityIr),
    Complete(&'a AsyncGeneratorAsyncDisposableCapabilityIr),
}
impl Disposal<'_> {
    fn binding_name(&self) -> &str {
        match self {
            Self::Legacy(capability) => capability.binding_name(),
            Self::Complete(capability) => capability.binding_name(),
        }
    }
    fn finalizer(&self) -> &AsyncDisposableFinalizerPlanIr {
        match self {
            Self::Legacy(capability) => capability.finalizer(),
            Self::Complete(capability) => capability.finalizer(),
        }
    }
}

fn block_disposals<'a>(statements: &'a [StatementIr], out: &mut Vec<(Disposal<'a>, usize)>) {
    for statement in statements {
        match statement {
            StatementIr::AsyncDisposableScope {
                execution,
                resources,
                body,
            } => {
                let AsyncDisposableScopeExecutionIr::AsyncFunction(capability) = execution else {
                    panic!("plain async case retains its actual disposal owner");
                };
                out.push((Disposal::Legacy(capability), resources.len()));
                block_disposals(&body.statements, out);
            }
            StatementIr::AsyncGeneratorResourceScope(plan) => {
                assert_eq!(plan.execution(), ResumableRegionProtocolIr::Async);
                let AsyncGeneratorResourceCapabilityIr::Async(capability) = plan.capability()
                else {
                    panic!("actual awaited disposal");
                };
                out.push((Disposal::Complete(capability), plan.capacity()));
                block_disposals(&plan.body().block().statements, out);
            }
            StatementIr::EmptyStatementCompletion(item) => {
                block_disposals(std::slice::from_ref(item.statement()), out)
            }
            StatementIr::Block(block) => block_disposals(&block.statements, out),
            StatementIr::LexicalBlock(statements) => block_disposals(statements, out),
            _ => {}
        }
    }
}

#[test]
fn implicit_only_block_disposal_retains_a_case_owner_and_following_await_entry() {
    let function = function("async function run() { switch (0) { default: { await using resource = null; break; } } await 0; }");
    let mut owners = Vec::new();
    switches(&function.body.statements, &mut owners);
    assert_eq!(owners.len(), 1);
    let plan = owners[0];
    let case = &plan.cases()[0];
    let mut disposals = Vec::new();
    block_disposals(&case.body().statements, &mut disposals);
    let [(capability, 1)] = disposals.as_slice() else {
        panic!("one resource stays in its nested block scope");
    };
    let finalizer = capability.finalizer();
    assert_eq!(
        (
            finalizer.entry_state(),
            finalizer.dispose_state(),
            finalizer.resume_state(),
            finalizer.exit_state(),
        ),
        (1, 2, 3, 4)
    );
    assert_eq!(case.next_entry_state(), 5);
    assert_eq!(plan.exit_state(), 5);
    assert!(function.body.statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait {
            suspend_state: 5,
            ..
        }
    )));
}

#[test]
fn nested_resources_finalize_after_actual_suffix_and_before_fallthrough() {
    let function = function("async function run() { switch (0) { default: { await using outer = null, second = null; await 0; { await using inner = null; await 1; } await 2; } case 1: await 3; break; } await 4; }");
    let mut owners = Vec::new();
    switches(&function.body.statements, &mut owners);
    let plan = owners[0];
    let mut disposals = Vec::new();
    block_disposals(&plan.cases()[0].body().statements, &mut disposals);
    let [(outer, 2), (inner, 1)] = disposals.as_slice() else {
        panic!("two outer registrations precede the nested capability");
    };
    assert_ne!(outer.binding_name(), inner.binding_name());
    assert_eq!(
        (
            inner.finalizer().entry_state(),
            inner.finalizer().dispose_state(),
            inner.finalizer().resume_state(),
            inner.finalizer().exit_state(),
        ),
        (2, 4, 5, 6)
    );
    assert_eq!(
        (
            outer.finalizer().entry_state(),
            outer.finalizer().dispose_state(),
            outer.finalizer().resume_state(),
            outer.finalizer().exit_state(),
        ),
        (1, 8, 9, 10)
    );
    assert_eq!(plan.cases()[1].entry_state(), 11);
    assert_eq!(plan.exit_state(), 13);
    assert!(function.body.statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait {
            suspend_state: 13,
            ..
        }
    )));
}

#[test]
fn block_disposal_composes_with_checked_case_branch_and_awaited_finally() {
    let function = function("async function run(flag) { switch (await 0) { default: try { if (flag) { await using resource = null; break; } } finally { await 1; } } await 2; }");
    let plan = complete_switch(&function);
    let case = &plan.cases()[0];
    let [StatementIr::TryFinally {
        try_block,
        async_plan: Some(try_plan),
        ..
    }] = case.body().block().statements.as_slice()
    else {
        panic!("case owns the existing try continuation");
    };
    let [StatementIr::AsyncFunctionIf {
        plan: branch,
        then_branch,
        ..
    }] = try_block.statements.as_slice()
    else {
        panic!("the conditional resource owns its selected branch");
    };
    let mut disposals = Vec::new();
    block_disposals(std::slice::from_ref(then_branch.as_ref()), &mut disposals);
    let [(capability, 1)] = disposals.as_slice() else {
        panic!("selected block retains one plain async capability");
    };
    assert_eq!(
        capability.finalizer().entry_state(),
        branch.then_entry_state()
    );
    assert!(capability.finalizer().exit_state() < branch.else_entry_state());
    assert!(branch.exit_state() < try_plan.try_exit_state);
    assert!(try_plan.exit_state < case.body().end_state() + 1);
}

#[test]
fn case_segments_own_skipped_eager_cases_and_following_await_entry() {
    let function = function("async function run() { switch (1) { case 0: break; case 1: await 0; default: break; } await 0; }");
    let mut owners = Vec::new();
    switches(&function.body.statements, &mut owners);
    let plan = owners[0];
    assert_eq!((plan.entry_state(), plan.exit_state()), (0, 5));
    assert_eq!(
        plan.cases()
            .iter()
            .map(|case| (case.entry_state(), case.next_entry_state()))
            .collect::<Vec<_>>(),
        [(1, 2), (2, 4), (4, 5)]
    );
    assert!(function.body.statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait {
            suspend_state: 5,
            ..
        }
    )));
}

#[test]
fn discriminant_prefix_precedes_caseblock_owner_and_does_not_capture_case_awaits() {
    let function =
        function("async function run() { switch (await 1) { default: await 2; } await 3; }");
    let plan = complete_switch(&function);
    let prefix = &plan.discriminant().region().block().statements;
    // General await staging retains its result cell before suspending. The
    // head await must feed the switch through that same activation-owned cell.
    let [StatementIr::Lexical {
        mode: BindingMode::Let,
        name: result_name,
        init,
    }, StatementIr::AsyncAwait {
        value: head_value,
        suspend_state: 0,
        resume_state: 1,
        resume_mode: AsyncResumeModeIr::AssignIdentifier(resume_name),
    }, StatementIr::Lexical {
        name: discriminant_name,
        init: discriminant_value,
        ..
    }] = prefix.as_slice()
    else {
        panic!("isolated result cell, head await and switch owner: {prefix:?}");
    };
    assert_eq!(init.expr, ExprIr::Undefined);
    assert_eq!(head_value.expr, ExprIr::Number(1.0_f64.to_bits()));
    assert_eq!(result_name, resume_name);
    assert!(matches!(
        &discriminant_value.expr,
        ExprIr::Identifier(name) if name == result_name
    ));
    assert_eq!(discriminant_name, &plan.discriminant_binding().name);
    assert!(
        matches!(&plan.discriminant().value().expr,ExprIr::Identifier(name) if name==discriminant_name)
    );
    assert_eq!(
        function
            .owned_env_bindings
            .iter()
            .filter(|binding| &binding.name == result_name)
            .count(),
        1
    );
    assert_eq!(plan.cases().len(), 1);
    assert!(plan.cases()[0].selector().is_none());
    let [StatementIr::AsyncAwait {
        value: case_value,
        suspend_state: 3,
        resume_state: 4,
        resume_mode: AsyncResumeModeIr::Ignore,
    }] = plan.cases()[0].body().block().statements.as_slice()
    else {
        panic!("case await must remain in its own continuation body");
    };
    assert_eq!(case_value.expr, ExprIr::Number(2.0_f64.to_bits()));
    let [_, StatementIr::AsyncAwait {
        value: following_value,
        suspend_state: 5,
        resume_state: 6,
        resume_mode: AsyncResumeModeIr::Ignore,
    }] = function.body.statements.as_slice()
    else {
        panic!("following await must remain outside the switch prefix");
    };
    assert_eq!(following_value.expr, ExprIr::Number(3.0_f64.to_bits()));
    assert_eq!(
        (
            plan.entry_state(),
            plan.cases()[0].body().entry_state(),
            plan.exit_state()
        ),
        (0, 3, 5)
    );
    assert_eq!(
        prefix
            .iter()
            .filter(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
            .count(),
        1
    );
}

#[test]
fn eager_try_in_a_case_still_requires_the_switch_completion_owner() {
    let function = function(
        "async function run() { switch (0) { default: try { break; } finally {} } await 0; }",
    );
    let mut owners = Vec::new();
    switches(&function.body.statements, &mut owners);
    assert_eq!(owners.len(), 1);
    assert!(owners[0].exit_state() > owners[0].cases()[0].entry_state() + 1);
}

#[test]
fn nested_switch_and_label_owners_compose_without_reusing_case_entries() {
    let function = function("async function run() { outer: switch (0) { default: switch (1) { default: await 0; break outer; } await 1; } await 2; }");
    let mut owners = Vec::new();
    switches(&function.body.statements, &mut owners);
    assert_eq!(owners.len(), 2);
    assert_eq!(owners[1].entry_state(), owners[0].cases()[0].entry_state());
    assert!(owners[1].exit_state() < owners[0].cases()[0].next_entry_state());
    assert!(matches!(
        function.body.statements[0],
        StatementIr::Labelled {
            async_plan: Some(_),
            ..
        }
    ));
}

#[test]
fn nested_async_function_does_not_claim_the_case_activation() {
    let function = function("async function run() { switch (0) { default: const helper = async () => { await 1; }; break; } await 0; }");
    let mut owners = Vec::new();
    switches(&function.body.statements, &mut owners);
    assert!(owners.is_empty());
    assert!(function.body.statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait {
            suspend_state: 0,
            ..
        }
    )));
}

#[test]
fn branch_sensitive_selectors_and_implicit_suspensions_fail_before_case_lowering() {
    for source in [
        "async function run(flag) { switch (0) { case flag[await 0] &&= await 0: break; } }",
        "async function run() { switch (0) { default: for await (const value of []) {} } }",
        "async function run() { switch (0) { default: for (await using resource of []) {} } }",
        "async function run() { switch (0) { default: for (await using resource = null; false;) {} } }",
        "async function run(flag) { switch (0) { default: while (flag) { await using resource = null; } } }",
        "async function run() { switch (0) { default: { await using resource = await 0; } } }",
        "async function run(flag) { switch (flag[await 0] &&= await 0) { default: break; } }",
        "async function run() { while (true) { switch (0) { default: try { break; } finally {} } } }",
    ] {
        let _=function(source);
    }
}

#[test]
fn awaited_selectors_retain_one_outer_discriminant_and_separate_selection_states() {
    let function = function("async function run(value) { switch (value) { case await 0: break; default: await 1; case await 2: break; } await 3; }");
    let plan = complete_switch(&function);
    let prefix = &plan.discriminant().region().block().statements;
    let [StatementIr::Lexical { name, init, .. }] = prefix.as_slice() else {
        panic!("one saved discriminant and its switch owner: {prefix:?}");
    };
    assert!(matches!(&init.expr, ExprIr::Identifier(original) if original == "value"));
    assert!(
        matches!(&plan.discriminant().value().expr, ExprIr::Identifier(saved) if saved == name)
    );
    assert_eq!(
        function
            .owned_env_bindings
            .iter()
            .filter(|binding| &binding.name == name)
            .count(),
        1
    );
    let selectors = plan
        .cases()
        .iter()
        .filter_map(|case| case.selector())
        .collect::<Vec<_>>();
    assert_eq!(
        selectors
            .iter()
            .map(|owner| (
                owner.region().entry_state(),
                owner.region().end_state(),
                owner.region().end_state() + 1
            ))
            .collect::<Vec<_>>(),
        [(1, 2, 3), (3, 4, 5)]
    );
    assert_eq!(plan.fallback_state(), 5);
    assert_eq!(
        plan.cases()
            .iter()
            .map(|case| (case.body().entry_state(), case.body().end_state() + 1))
            .collect::<Vec<_>>(),
        [(6, 7), (7, 9), (9, 10)]
    );
    assert_eq!(plan.exit_state(), 10);
    assert!(function.body.statements.iter().any(|statement| matches!(
        statement,
        StatementIr::AsyncAwait {
            suspend_state: 10,
            resume_state: 11,
            ..
        }
    )));
    for owner in selectors {
        for statement in &owner.region().block().statements {
            if let StatementIr::Lexical { name, .. } = statement {
                assert!(function
                    .owned_env_bindings
                    .iter()
                    .any(|binding| &binding.name == name));
            }
        }
    }
}

#[test]
fn selector_await_requires_a_switch_owner_even_with_eager_bodies() {
    let function = function(
        "async function run() { switch (await 1) { case await 1: break; default: break; } }",
    );
    let plan = complete_switch(&function);
    assert_eq!(plan.entry_state(), 0);
    assert_eq!(plan.case_block_entry_state(), 2);
    assert_eq!(plan.fallback_state(), 4);
    assert_eq!(plan.cases()[0].body().entry_state(), 5);
    assert_eq!(plan.exit_state(), 7);
}

#[test]
fn direct_case_using_keeps_front_early_error_while_nested_sync_using_is_admitted() {
    assert!(parse(
        "async function run() { switch (0) { default: await using resource = null; } }",
        ParseOptions::script()
    )
    .is_err());
    let function = function("async function run() { switch (0) { default: { using resource = null; await 0; break; } } await 0; }");
    let mut owners = Vec::new();
    switches(&function.body.statements, &mut owners);
    assert_eq!(owners.len(), 1);
}

use lila_front::{parse, ParseOptions};
use lila_ir::{lower, ExprIr, FunctionIr, StatementIr, ValueKind};

fn values(source: &str) -> FunctionIr {
    let program = lower(&parse(source, ParseOptions::script()).unwrap());
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
        .find(|function| function.name == "values")
        .unwrap()
}

fn rows<'a>(source: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for statement in source {
        output.push(statement);
        match statement {
            StatementIr::AsyncGeneratorForIn(plan) => {
                rows(&plan.head().region().block().statements, output);
                rows(&plan.initialization().statements, output);
                rows(&plan.body().block().statements, output);
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                rows(&plan.head().region().block().statements, output);
                rows(&plan.initialization().block().statements, output);
                rows(&plan.body().block().statements, output);
            }
            StatementIr::AsyncFunctionWith(plan) => {
                rows(&plan.head().statements, output);
                rows(&plan.body().statements, output);
            }
            StatementIr::AsyncFunctionArrayDestructuring(plan) => {
                rows(&plan.body().statements, output)
            }
            StatementIr::Block(block) => rows(&block.statements, output),
            StatementIr::LexicalBlock(body) => rows(body, output),
            StatementIr::If {
                then_branch,
                else_branch,
                ..
            }
            | StatementIr::AsyncFunctionIf {
                then_branch,
                else_branch,
                ..
            } => {
                rows(std::slice::from_ref(then_branch), output);
                if let Some(branch) = else_branch {
                    rows(std::slice::from_ref(branch), output);
                }
            }
            StatementIr::AsyncFunctionSwitch(plan) => {
                for case in plan.cases() {
                    rows(case.condition_prefix(), output);
                }
                rows(plan.lexical_declarations(), output);
                for case in plan.cases() {
                    rows(&case.body().statements, output);
                }
            }
            StatementIr::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                rows(&try_block.statements, output);
                rows(&catch_block.statements, output);
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                ..
            } => {
                rows(&try_block.statements, output);
                rows(&finally_block.statements, output);
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                ..
            } => {
                rows(&try_block.statements, output);
                rows(&catch_block.statements, output);
                rows(&finally_block.statements, output);
            }
            StatementIr::Labelled { statement, .. } => {
                rows(std::slice::from_ref(statement), output)
            }
            StatementIr::EmptyStatementCompletion(item) => {
                rows(std::slice::from_ref(item.statement()), output)
            }
            _ => {}
        }
    }
}

#[test]
fn async_for_in_owns_completed_head_four_distinct_cells_and_original_lexical_record() {
    let function = values("async function values(input,p){for(const key in await input){var read=()=>key;await p;read();}}");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    let plan = ordered
        .iter()
        .find_map(|row| match row {
            StatementIr::AsyncGeneratorForIn(plan) => Some(plan),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        (
            plan.entry_state(),
            plan.head().region().end_state(),
            plan.advance_state(),
            plan.body().entry_state(),
            plan.body().end_state(),
            plan.exit_state()
        ),
        (0, 1, 2, 4, 5, 6)
    );
    assert_eq!(plan.continue_state(), plan.advance_state());
    let bindings = [
        plan.head_binding(),
        plan.enumerator_binding(),
        plan.key_binding(),
        plan.value_binding(),
    ];
    assert_eq!(
        bindings
            .iter()
            .map(|binding| binding.name.as_str())
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
    assert_eq!(
        bindings
            .iter()
            .map(|binding| binding.slot)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        4
    );
    assert!(bindings
        .iter()
        .all(|binding| function.owned_env_bindings.contains(binding)));
    let ExprIr::Identifier(name) = &plan.head().value().expr else {
        panic!("actual retained head read");
    };
    assert_eq!(name, &plan.head_binding().name);
    let environment = plan.lexical_environment().unwrap();
    assert_eq!(environment.tdz_binding_names.len(), 1);
    let record = environment
        .iteration_environment
        .as_ref()
        .expect("captured key owns its original per-iteration record");
    assert!(record
        .bindings
        .iter()
        .all(|binding| !bindings.iter().any(|row| row.name == binding.name)));
    let StatementIr::Lexical { init, .. } = &plan.initialization().statements[0] else {
        panic!("original const BindingInitialization");
    };
    assert_eq!(init.kind, ValueKind::String);
    assert!(matches!(&init.expr,ExprIr::Identifier(name) if name==&plan.key_binding().name));
}

#[test]
fn async_for_in_consumes_nested_with_array_and_full_branch_source_ranges() {
    let function=values("async function values(input,view,p){for(const key in await input){if(true){with(view){for(var child in await input){const [received=await p]=[];}}}try{await p;}finally{await p;}}}");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    assert_eq!(
        ordered
            .iter()
            .filter(|row| matches!(row, StatementIr::AsyncGeneratorForIn(_)))
            .count(),
        2
    );
    assert!(ordered
        .iter()
        .any(|row| matches!(row, StatementIr::AsyncFunctionWith(_))));
    assert!(ordered
        .iter()
        .any(|row| matches!(row, StatementIr::AsyncFunctionArrayDestructuring(_))));
    assert!(ordered
        .iter()
        .any(|row| matches!(row, StatementIr::AsyncFunctionIf { .. })));
    let awaits = ordered
        .iter()
        .filter_map(|row| match row {
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => Some((*suspend_state, *resume_state)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(awaits.len(), 5);
    assert!(awaits.windows(2).all(|pair| pair[0].1 <= pair[1].0));
}

#[test]
fn eager_async_for_in_phases_are_owned_by_enclosing_if_switch_and_labelled_try() {
    let function=values("async function values(input,flag){if(true){for(var key in input){key;}}switch(flag){case 0:for(const key in input){key;}break;default:break;}label:{try{for(var key in input){break label;}}finally{flag;}}}");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    assert_eq!(
        ordered
            .iter()
            .filter(|row| matches!(row, StatementIr::AsyncGeneratorForIn(_)))
            .count(),
        3
    );
    assert!(!ordered
        .iter()
        .any(|row| matches!(row, StatementIr::AsyncAwait { .. })));
    assert!(ordered
        .iter()
        .any(|row| matches!(row, StatementIr::AsyncFunctionIf { .. })));
    assert!(ordered
        .iter()
        .any(|row| matches!(row, StatementIr::AsyncFunctionSwitch(_))));
}

#[test]
fn async_for_in_retains_source_empty_wrappers_and_original_eager_property_put() {
    let function=values("async function values(input,target,p){for(target.key in await input){var received=await p;if(true){const [other=await p]=[];}}}");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    let plan = ordered
        .iter()
        .find_map(|row| match row {
            StatementIr::AsyncGeneratorForIn(plan) => Some(plan),
            _ => None,
        })
        .unwrap();
    assert!(plan
        .initialization()
        .statements
        .iter()
        .any(|statement| matches!(statement, StatementIr::DeclarationEvaluation(_))));
    assert!(ordered
        .iter()
        .any(|row| matches!(row, StatementIr::EmptyStatementCompletion(_))));
    assert_eq!(
        ordered
            .iter()
            .filter(|row| matches!(row, StatementIr::AsyncAwait { .. }))
            .count(),
        3
    );
}

#[test]
fn async_for_in_keeps_complete_loop_and_suspended_per_key_head_domains_owned() {
    for source in [
        "async function* values(input,p){for(var key in input){await p;}}",
        "async function values(input,p){for(const item of [1]){for(var key in input){await p;}}}",
        "async function values(input,target,p){for(target[await p] in input){await p;}}",
        "async function values(input,p){for(const [key=await p] in input){await p;}}",
    ] {
        let program = lower(&parse(source, ParseOptions::script()).unwrap());
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
    }
    let function =
        values("async function values(input){for(const item of [1]){for(var key in input){key;}}}");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    assert_eq!(
        ordered
            .iter()
            .filter(|row| matches!(row, StatementIr::AsyncGeneratorForIn(_)))
            .count(),
        1
    );
}

#[test]
fn async_for_in_owns_awaited_prefix_target_default_and_body_as_one_tape() {
    let function=values("async function values(input,target,p){for(var key=await p in await input){await key;}for(target[await p] in input){await key;}for(let [first,second,missing=await p] in input){await missing;}}");
    let mut ordered = Vec::new();
    rows(&function.body.statements, &mut ordered);
    let plans = ordered
        .iter()
        .filter_map(|row| match row {
            StatementIr::AsyncGeneratorForIn(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(plans.len(), 3);
    assert!(plans
        .iter()
        .all(|plan| plan.execution() == lila_ir::ResumableRegionProtocolIr::Async));
    assert!(plans
        .iter()
        .skip(1)
        .all(|plan| plan.initialization_region().end_state()
            > plan.initialization_region().entry_state()));
    let awaits = ordered
        .iter()
        .filter_map(|row| match row {
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => Some((*suspend_state, *resume_state)),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(awaits.len(), 7);
    assert!(awaits.windows(2).all(|pair| pair[0].1 <= pair[1].0));
    for plan in plans {
        assert_eq!(
            plan.body().entry_state(),
            plan.initialization_region().end_state() + 1
        );
        assert_eq!(plan.continue_state(), plan.advance_state());
    }
}

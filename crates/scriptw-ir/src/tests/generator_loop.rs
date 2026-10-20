fn ordinary_classic_loop(function: &FunctionIr) -> &OrdinaryGeneratorLoopIr {
    fn find(statement: &StatementIr) -> Option<&OrdinaryGeneratorLoopIr> {
        match statement {
            StatementIr::OrdinaryGeneratorLoop(plan) => Some(plan),
            StatementIr::Labelled { statement, .. } => find(statement),
            _ => None,
        }
    }
    function
        .body
        .statements
        .iter()
        .find_map(find)
        .expect("checked classic loop owner")
}

#[test]
fn ordinary_generator_classic_phases_consume_source_heads_and_full_body_suspensions() {
    for (source, kind, suspension_count) in [
        ("function* g() { for (let i = yield 'init'; yield 'test'; yield 'update') { yield i; yield i; } yield 'tail'; }", GeneratorLoopKindIr::For, 6),
        ("function* g() { while (yield 'test') { yield 1; yield 2; } yield 'tail'; }", GeneratorLoopKindIr::While, 4),
        ("function* g() { do { yield 1; yield 2; } while (yield 'test'); yield 'tail'; }", GeneratorLoopKindIr::DoWhile, 4),
    ] {
        let program = lower_script(source);
        assert!(program.is_wasm_supported(), "{source}: {:?}", program.diagnostics);
        let function = program.script.as_ref().unwrap().functions.iter().find(|function| function.name == "g").unwrap();
        let plan = ordinary_classic_loop(function);
        assert_eq!(plan.kind(), kind);
        assert_eq!(function.generator_plan.as_ref().unwrap().suspension_points.len(), suspension_count);
        assert!(function.owned_env_bindings.contains(plan.value_binding()));
        let regions = plan.regions().collect::<Vec<_>>();
        for pair in regions.windows(2) { assert_eq!(pair[0].end_state().checked_add(1), Some(pair[1].entry_state())); }
        assert_eq!(regions.last().unwrap().end_state().checked_add(1), Some(plan.exit_state()));
        assert_eq!(plan.continue_state(), plan.update().map_or(plan.test().region().entry_state(), |update| update.region().entry_state()));
        let tail = function.generator_plan.as_ref().unwrap().suspension_points.last().unwrap();
        assert_eq!(tail.suspend_state, plan.exit_state(), "the loop commits its exit before the tail");
    }
}

#[test]
fn ordinary_generator_loop_branches_and_yielding_finally_keep_complete_control_regions() {
    let program = lower_script("function* g() { outer: for (let i = 0; i < 2; i++) { if (yield 'choose') { yield 1; yield 2; } else { try { yield 3; continue outer; } finally { yield 4; yield 5; } } break outer; } yield 'tail'; }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .unwrap()
        .functions
        .iter()
        .find(|function| function.name == "g")
        .unwrap();
    let loop_plan = ordinary_classic_loop(function);
    let [StatementIr::LexicalBlock(prefix), StatementIr::Break { label: Some(label) }] =
        loop_plan.body().block().statements.as_slice()
    else {
        panic!("complete body and labelled break must be retained");
    };
    assert_eq!(label, "outer");
    let StatementIr::OrdinaryGeneratorIf(branch) = prefix.last().unwrap() else {
        panic!("condition prefix ends in its range-owned branch");
    };
    assert_eq!(
        branch
            .then_branch()
            .block()
            .statements
            .iter()
            .filter(|statement| matches!(statement, StatementIr::GeneratorYield { .. }))
            .count(),
        2
    );
    let StatementIr::TryFinally {
        finally_block,
        generator_plan: Some(finalizer),
        async_plan: None,
        ..
    } = &branch.else_branch().block().statements[0]
    else {
        panic!("yielding finalizer owner retained inside else range");
    };
    assert_eq!(
        finally_block
            .statements
            .iter()
            .filter(|statement| matches!(statement, StatementIr::GeneratorYield { .. }))
            .count(),
        2
    );
    assert!(finalizer.finally_entry_state.unwrap() < finalizer.finally_exit_state.unwrap());
    assert_eq!(branch.else_branch().end_state(), finalizer.exit_state);
    let points = &function.generator_plan.as_ref().unwrap().suspension_points;
    assert_eq!(points.len(), 7);
    assert_eq!(
        points
            .iter()
            .map(|point| point.resume_state)
            .collect::<BTreeSet<_>>()
            .len(),
        points.len()
    );
}

#[test]
fn ordinary_generator_nested_classic_loops_retain_distinct_value_and_captured_iteration_cells() {
    let program = lower_script("function* g() { for (let i = 0; i < 2; i++) { let body = i; while (yield 'head') { yield function () { return [i, body]; }; break; } yield function () { return i; }; } }");
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .unwrap()
        .functions
        .iter()
        .find(|function| function.name == "g")
        .unwrap();
    let outer = ordinary_classic_loop(function);
    let environment = outer
        .lexical_environment()
        .expect("captured for-let environment");
    assert!(!environment.per_iteration_slots.is_empty());
    assert!(
        outer.body().block().lexical_environment.is_some(),
        "captured body cell retains its block record"
    );
    let inner = outer
        .body()
        .block()
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::OrdinaryGeneratorLoop(plan) => Some(plan),
            _ => None,
        })
        .expect("inner While owner");
    assert_ne!(outer.value_binding(), inner.value_binding());
    assert!(function.owned_env_bindings.contains(outer.value_binding()));
    assert!(function.owned_env_bindings.contains(inner.value_binding()));
}

#[test]
fn ordinary_generator_loop_admission_rejects_orphan_states_and_foreign_head_control() {
    use crate::generator_loop_control::{
        GeneratorLoopControlError, GeneratorLoopSourceRange, GeneratorLoopSourceStates,
    };
    let block = |statements| BlockIr {
        statements,
        result_kind: ValueKind::Undefined,
        lexical_environment: None,
    };
    let region = |entry, statements| {
        GeneratorLoopRegionIr::new(
            block(statements),
            GeneratorLoopSourceRange { entry, end: entry },
        )
        .unwrap()
    };
    let value = || TypedExpr::from_info(ValueInfo::new(ValueKind::Boolean), ExprIr::Boolean(true));
    let source = || GeneratorLoopSourceStates {
        kind: GeneratorLoopKindIr::While,
        initialization: None,
        test: GeneratorLoopSourceRange { entry: 0, end: 0 },
        body: GeneratorLoopSourceRange { entry: 1, end: 1 },
        update: None,
        exit: 2,
        suspensions: Vec::new(),
    };
    let binding = || OwnedEnvBindingIr {
        mutability: crate::EnvironmentBindingMutabilityIr::Mutable,
        name: "generator.loop.value.checked".into(),
        slot: 0,
    };
    let orphan = GeneratorLoopRegionIr::new(
        block(vec![StatementIr::GeneratorYield {
            value: TypedExpr::undefined(),
            form: YieldForm::Plain,
            suspend_state: 0,
            resume_state: 2,
            resume_mode: GeneratorResumeModeIr::Ignore,
        }]),
        GeneratorLoopSourceRange { entry: 0, end: 2 },
    );
    assert!(matches!(
        orphan,
        Err(GeneratorLoopControlError::StateMismatch {
            expected: 1,
            actual: 2
        })
    ));
    assert!(matches!(
        OrdinaryGeneratorLoopIr::new(
            source(),
            None,
            GeneratorLoopExpressionIr::new(
                region(0, vec![StatementIr::Continue { label: None }]),
                value()
            ),
            region(1, vec![]),
            None,
            None,
            binding()
        ),
        Err(GeneratorLoopControlError::ForeignContinuation)
    ));
    let mut unused = source();
    unused.suspensions.push(GeneratorSuspensionPointIr {
        suspend_state: 0,
        resume_state: 1,
    });
    assert!(matches!(
        OrdinaryGeneratorLoopIr::new(
            unused,
            None,
            GeneratorLoopExpressionIr::new(region(0, vec![]), value()),
            region(1, vec![]),
            None,
            None,
            binding()
        ),
        Err(GeneratorLoopControlError::UnconsumedSourceSuspension)
    ));
    let foreign = GeneratorLoopRegionIr::new(
        block(vec![StatementIr::AsyncAwait {
            value: TypedExpr::undefined(),
            suspend_state: 0,
            resume_state: 1,
            resume_mode: AsyncResumeModeIr::Ignore,
        }]),
        GeneratorLoopSourceRange { entry: 0, end: 1 },
    );
    assert!(matches!(
        foreign,
        Err(GeneratorLoopControlError::ForeignContinuation)
    ));
    assert!(crate::generator_loop_control::checked_next_state(u32::MAX).is_err());
}

use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, AsyncGeneratorLoopIr, FunctionIr, GeneratorLoopKindIr, ResumableResumeEnvironmentIr,
    ResumableSuspensionKindIr, StatementIr,
};

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

fn loops<'a>(items: &'a [StatementIr], output: &mut Vec<&'a AsyncGeneratorLoopIr>) {
    for item in items {
        match item {
            StatementIr::AsyncGeneratorLoop(plan) => {
                output.push(plan);
                for region in plan.regions() {
                    loops(&region.block().statements, output);
                }
            }
            StatementIr::AsyncGeneratorIf(plan) => {
                loops(&plan.condition().region().block().statements, output);
                loops(&plan.then_branch().block().statements, output);
                loops(&plan.else_branch().block().statements, output);
            }
            StatementIr::AsyncGeneratorForIn(plan) => {
                loops(&plan.head().region().block().statements, output);
                loops(&plan.initialization().statements, output);
                loops(&plan.body().block().statements, output);
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                loops(&plan.head().region().block().statements, output);
                loops(&plan.initialization().block().statements, output);
                loops(&plan.body().block().statements, output);
            }
            StatementIr::AsyncGeneratorResourceScope(plan) => {
                loops(&plan.body().block().statements, output)
            }
            StatementIr::Block(block) => loops(&block.statements, output),
            StatementIr::LexicalBlock(items) => loops(items, output),
            StatementIr::EmptyStatementCompletion(item) => {
                loops(std::slice::from_ref(item.statement()), output)
            }
            _ => {}
        }
    }
}

#[test]
fn mixed_classic_for_has_complete_contiguous_phases_and_exact_await_yield_tape() {
    let function = values("async function* values(input){for(let i=await input;i<await input;i=(yield i)+(await input)){const v=await input;yield v;}return 9;}");
    let [StatementIr::AsyncGeneratorLoop(plan), StatementIr::AsyncAwait { .. }] =
        function.body.statements.as_slice()
    else {
        panic!("{:#?}", function.body.statements);
    };
    assert_eq!(plan.kind(), GeneratorLoopKindIr::For);
    assert_eq!(
        (plan.entry_state(), plan.exit_state(), plan.continue_state()),
        (0, 10, 7)
    );
    assert_eq!(
        plan.regions()
            .map(|region| (region.entry_state(), region.end_state()))
            .collect::<Vec<_>>(),
        [(0, 1), (2, 3), (4, 6), (7, 9)]
    );
    assert!(function.owned_env_bindings.contains(plan.value_binding()));
    assert!(matches!(
        plan.body().block().statements.first(),
        Some(StatementIr::EmptyStatementCompletion(_))
    ));
    let tape = &function.resumable_plan.as_ref().unwrap().suspension_points;
    assert_eq!(
        tape.iter()
            .map(|point| (point.kind, point.suspend_state, point.resume_state))
            .collect::<Vec<_>>(),
        [
            (ResumableSuspensionKindIr::Await, 0, 1),
            (ResumableSuspensionKindIr::Await, 2, 3),
            (ResumableSuspensionKindIr::Await, 4, 5),
            (ResumableSuspensionKindIr::Yield, 5, 6),
            (ResumableSuspensionKindIr::Yield, 7, 8),
            (ResumableSuspensionKindIr::Await, 8, 9),
            (ResumableSuspensionKindIr::Await, 10, 11),
        ]
    );
    assert!(tape[..6]
        .iter()
        .all(|point| point.resume_environment == ResumableResumeEnvironmentIr::InvocationOuter));
    assert_eq!(
        tape[6].resume_environment,
        ResumableResumeEnvironmentIr::SavedLexicalChain
    );
}

#[test]
fn mixed_if_completes_condition_before_disjoint_full_arms_and_real_throw() {
    let function = values("async function* values(input){if(await (yield input)){const received=await input;yield received;}else{throw yield input;}return input;}");
    let StatementIr::AsyncGeneratorIf(plan) = &function.body.statements[0] else {
        panic!("checked mixed If");
    };
    assert_eq!(
        (
            plan.condition().region().entry_state(),
            plan.condition().region().end_state()
        ),
        (0, 2)
    );
    assert_eq!(
        (
            plan.then_branch().entry_state(),
            plan.then_branch().end_state()
        ),
        (3, 5)
    );
    assert_eq!(
        (
            plan.else_branch().entry_state(),
            plan.else_branch().end_state(),
            plan.exit_state()
        ),
        (6, 7, 8)
    );
    let tape = &function.resumable_plan.as_ref().unwrap().suspension_points;
    assert_eq!(
        tape.iter().map(|point| point.kind).collect::<Vec<_>>(),
        [
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await
        ]
    );
    assert!(tape[..5]
        .iter()
        .all(|point| point.resume_environment == ResumableResumeEnvironmentIr::InvocationOuter));
    assert_eq!(
        tape[5].resume_environment,
        ResumableResumeEnvironmentIr::SavedLexicalChain
    );
}

#[test]
fn mixed_try_and_legacy_for_await_keep_distinct_checked_resume_environments() {
    let function = values("async function* values(source,p){try{await p;yield 1;}finally{await p;}for await(const value of source){yield value;}await p;}");
    let plan = function.resumable_plan.unwrap();
    assert_eq!(plan.state_count, 16);
    assert_eq!(
        plan.suspension_points
            .iter()
            .map(|point| (point.suspend_state, point.resume_environment))
            .collect::<Vec<_>>(),
        [
            (0, ResumableResumeEnvironmentIr::InvocationOuter),
            (1, ResumableResumeEnvironmentIr::InvocationOuter),
            (3, ResumableResumeEnvironmentIr::InvocationOuter),
            (7, ResumableResumeEnvironmentIr::SavedLexicalChain),
            (10, ResumableResumeEnvironmentIr::InvocationOuter),
            (12, ResumableResumeEnvironmentIr::SavedLexicalChain),
            (14, ResumableResumeEnvironmentIr::SavedLexicalChain),
        ]
    );
}

#[test]
fn mixed_while_do_and_nested_eager_loops_own_their_actual_phase_ranges() {
    let function = values("async function* values(p){while(await p){for(let i=0;i<1;i++){yield i;}break;}do{yield 2;}while(await p);}");
    let mut found = Vec::new();
    loops(&function.body.statements, &mut found);
    assert_eq!(
        found.iter().map(|plan| plan.kind()).collect::<Vec<_>>(),
        [
            GeneratorLoopKindIr::While,
            GeneratorLoopKindIr::For,
            GeneratorLoopKindIr::DoWhile
        ]
    );
    for plan in found {
        let regions = plan.regions().collect::<Vec<_>>();
        assert!(regions
            .windows(2)
            .all(|pair| pair[0].end_state() + 1 == pair[1].entry_state()));
        assert_eq!(regions.last().unwrap().end_state() + 1, plan.exit_state());
        assert!(function.owned_env_bindings.contains(plan.value_binding()));
    }
}

#[test]
fn explicit_captured_block_certifies_its_pre_loop_point_and_preserves_foreign_points() {
    let function = values("async function* values(source,p){{let retained=p;var read=()=>retained;yield retained;for(let i=0;i<1;i++){await p;yield read();}for await(const value of source){yield value;}}}");
    let [StatementIr::Block(block)] = function.body.statements.as_slice() else {
        panic!("actual source Block");
    };
    assert!(block.lexical_environment.is_some());
    let plan = function.resumable_plan.as_ref().unwrap();
    assert_eq!(plan.state_count, 17);
    assert_eq!(
        plan.resume_environment_plan().invocation_resume_states(),
        [1, 4, 5, 13]
    );
    assert_eq!(
        plan.resume_environment_plan()
            .enclosing_scope_resume_states(),
        [1, 4, 5, 10, 13, 15]
    );
    assert_eq!(
        plan.suspension_points
            .iter()
            .map(|point| (point.suspend_state, point.resume_environment))
            .collect::<Vec<_>>(),
        [
            (0, ResumableResumeEnvironmentIr::InvocationOuter),
            (3, ResumableResumeEnvironmentIr::InvocationOuter),
            (4, ResumableResumeEnvironmentIr::InvocationOuter),
            (9, ResumableResumeEnvironmentIr::SavedLexicalChain),
            (12, ResumableResumeEnvironmentIr::InvocationOuter),
            (14, ResumableResumeEnvironmentIr::SavedLexicalChain),
        ]
    );
}

#[test]
fn original_foreign_if_relocates_later_checked_certificate_with_its_source_suffix() {
    let function = values("async function* values(source,flag){for(const value of source){if(flag)yield value;else yield 0;}for(let index=0;index<1;index++){yield 9;}}");
    let plan = function.resumable_plan.as_ref().unwrap();
    assert_eq!(plan.state_count, 16);
    assert_eq!(
        plan.resume_environment_plan().invocation_resume_states(),
        [6, 8, 13]
    );
    assert_eq!(
        plan.suspension_points
            .iter()
            .map(|point| (
                point.suspend_state,
                point.resume_state,
                point.resume_environment
            ))
            .collect::<Vec<_>>(),
        [
            (5, 6, ResumableResumeEnvironmentIr::InvocationOuter),
            (7, 8, ResumableResumeEnvironmentIr::InvocationOuter),
            (12, 13, ResumableResumeEnvironmentIr::InvocationOuter),
        ]
    );
}

#[test]
fn foreign_nested_block_preserves_saved_protocol_and_certifies_only_scope_ancestry() {
    let function = values("async function* values(source){for await(const value of source){{let retained=value;var read=()=>retained;yield read();}}}");
    let plan = function.resumable_plan.as_ref().unwrap();
    let [StatementIr::AsyncGeneratorForOf(iterator)] = function.body.statements.as_slice() else {
        panic!("the captured source body belongs to the complete iterator owner");
    };
    assert_eq!(plan.state_count, 10);
    assert_eq!(
        (
            iterator.acquisition_state(),
            iterator.advance_state(),
            iterator.initialization().entry_state(),
            iterator.initialization().end_state(),
            iterator.body().entry_state(),
            iterator.body().end_state(),
            iterator.exit_state(),
        ),
        (1, 2, 4, 4, 5, 6, 9)
    );
    assert_eq!(
        plan.resume_environment_plan().invocation_resume_states(),
        [6]
    );
    assert_eq!(
        plan.resume_environment_plan()
            .enclosing_scope_resume_states(),
        [6]
    );
    assert_eq!(
        plan.suspension_points
            .iter()
            .map(|point| (
                point.suspend_state,
                point.resume_state,
                point.resume_environment
            ))
            .collect::<Vec<_>>(),
        [
            (2, 3, ResumableResumeEnvironmentIr::SavedLexicalChain),
            (5, 6, ResumableResumeEnvironmentIr::InvocationOuter),
            (7, 8, ResumableResumeEnvironmentIr::SavedLexicalChain),
        ]
    );
}

#[test]
fn implicit_disposal_resume_certificate_comes_from_the_actual_finalizer_plan() {
    let function = values("async function* values(){{let retained=1;var read=()=>retained;await using resource=null;yield read();}}");
    let plan = function.resumable_plan.as_ref().unwrap();
    assert_eq!(plan.state_count, 5);
    assert_eq!(plan.suspension_points.len(), 1);
    assert_eq!(plan.suspension_points[0].resume_state, 1);
    assert_eq!(
        plan.suspension_points[0].resume_environment,
        ResumableResumeEnvironmentIr::InvocationOuter
    );
    assert_eq!(
        plan.resume_environment_plan().invocation_resume_states(),
        [1]
    );
    assert_eq!(
        plan.resume_environment_plan()
            .enclosing_scope_resume_states(),
        [1, 3]
    );
    assert!(plan
        .suspension_points
        .iter()
        .all(|point| point.resume_state != 3));
}

#[test]
fn mixed_owner_refuses_unowned_pattern_and_opposite_composite_protocols() {
    let function =
        values("async function* values(p){for(let i=0;i<1;i++){const [x=await p]=[];yield x;}}");
    let mut found = Vec::new();
    loops(&function.body.statements, &mut found);
    assert_eq!(found.len(), 1);
    for source in [
        "async function* values(p){while(true){yield {a:await p,b:yield 1};}}",
        "async function* values(p){while(true){delete object[await p];yield 1;}}",
    ] {
        let function = values(source);
        let mut found = Vec::new();
        loops(&function.body.statements, &mut found);
        assert_eq!(
            found.len(),
            1,
            "the actual mixed expression remains within its complete loop"
        );
    }
}

#[test]
fn mixed_for_in_body_owns_nested_complete_classic_loop() {
    let function = values("async function* values(items,p){for(const key in await (yield items)){while(yield key){await p;break;}}}");
    let mut found = Vec::new();
    loops(&function.body.statements, &mut found);
    assert_eq!(found.len(), 1);
    let StatementIr::AsyncGeneratorForIn(outer) = &function.body.statements[0] else {
        panic!("checked complete enumeration");
    };
    assert!(found[0].entry_state() >= outer.body().entry_state());
    assert!(found[0].exit_state() <= outer.body().end_state());
}

#[test]
fn mixed_var_initializer_keeps_its_entire_suspended_prefix_in_empty_completion() {
    let function = values(
        "async function* values(p){for(let i=0;i<1;i++){17;var saved=await (yield p);debugger;}}",
    );
    let mut found = Vec::new();
    loops(&function.body.statements, &mut found);
    let statements = &found[0].body().block().statements;
    assert!(matches!(
        statements.first(),
        Some(StatementIr::Expression(_))
    ));
    let StatementIr::EmptyStatementCompletion(item) = &statements[1] else {
        panic!("actual source var owns Empty completion");
    };
    let StatementIr::LexicalBlock(prefix) = item.statement() else {
        panic!("whole var prefix is retained");
    };
    assert!(prefix
        .iter()
        .any(|item| matches!(item, StatementIr::GeneratorYield { .. })));
    assert!(prefix
        .iter()
        .any(|item| matches!(item, StatementIr::AsyncAwait { .. })));
    assert!(matches!(
        prefix.last(),
        Some(StatementIr::DeclarationEvaluation(_))
    ));
    assert!(matches!(
        statements.last(),
        Some(StatementIr::EmptyStatementCompletion(_))
    ));
}

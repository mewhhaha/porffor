use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, ArrayDestructuringOperationKindIr, ExprIr, FunctionIr, ResumableResumeEnvironmentIr,
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
        .find(|function| function.name == "values" || function.name.ends_with(".values"))
        .unwrap()
}
fn walk<'a>(items: &'a [StatementIr], output: &mut Vec<&'a StatementIr>) {
    for item in items {
        output.push(item);
        match item {
            StatementIr::AsyncGeneratorArrayDestructuring(plan) => {
                walk(&plan.body().block().statements, output)
            }
            StatementIr::AsyncGeneratorIf(plan) => {
                for region in plan.regions() {
                    walk(&region.block().statements, output);
                }
            }
            StatementIr::AsyncGeneratorLoop(plan) => {
                for region in plan.regions() {
                    walk(&region.block().statements, output);
                }
            }
            StatementIr::AsyncGeneratorWith(plan) => {
                walk(&plan.head().region().block().statements, output);
                walk(&plan.body().block().statements, output);
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                walk(&plan.head().region().block().statements, output);
                walk(&plan.initialization().block().statements, output);
                walk(&plan.body().block().statements, output);
            }
            StatementIr::AsyncGeneratorResourceScope(plan) => {
                walk(&plan.body().block().statements, output)
            }
            StatementIr::Block(block) => walk(&block.statements, output),
            StatementIr::LexicalBlock(items) => walk(items, output),
            StatementIr::EmptyStatementCompletion(item) => {
                walk(std::slice::from_ref(item.statement()), output)
            }
            StatementIr::If {
                then_branch,
                else_branch,
                ..
            } => {
                walk(std::slice::from_ref(then_branch), output);
                if let Some(branch) = else_branch {
                    walk(std::slice::from_ref(branch), output);
                }
            }
            _ => {}
        }
    }
}

#[test]
fn mixed_array_pattern_owns_exact_iterator_operations_guarded_tape_and_raw_result() {
    let function = values("async function* values(input,p,q){var received;return ([,received=(yield p,await q)]=input);}");
    let mut items = Vec::new();
    walk(&function.body.statements, &mut items);
    let plan = items
        .iter()
        .find_map(|item| match item {
            StatementIr::AsyncGeneratorArrayDestructuring(plan) => Some(plan),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        (
            plan.entry_state(),
            plan.body_entry_state(),
            plan.body_end_state(),
            plan.exit_state()
        ),
        (0, 1, 6, 7)
    );
    assert_eq!(
        items
            .iter()
            .filter_map(|item| match item {
                StatementIr::ArrayDestructuringOperation(op) => Some(op.kind()),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [
            ArrayDestructuringOperationKindIr::Elision,
            ArrayDestructuringOperationKindIr::StepValue
        ]
    );
    let points = &function.resumable_plan.as_ref().unwrap().suspension_points;
    assert_eq!(
        points
            .iter()
            .map(|point| (point.kind, point.suspend_state, point.resume_state))
            .collect::<Vec<_>>(),
        [
            (ResumableSuspensionKindIr::Yield, 2, 3),
            (ResumableSuspensionKindIr::Await, 3, 4),
            (ResumableSuspensionKindIr::Await, 7, 8)
        ]
    );
    assert!(points[..2]
        .iter()
        .all(|point| point.resume_environment == ResumableResumeEnvironmentIr::InvocationOuter));
    let ExprIr::Identifier(raw) = &plan.raw_source().expr else {
        panic!("whole raw RHS cell");
    };
    assert_ne!(raw, &plan.storage().binding().name);
    assert_eq!(
        function
            .owned_env_bindings
            .iter()
            .filter(|row| &row.name == raw)
            .count(),
        1
    );
    assert!(function
        .owned_env_bindings
        .contains(plan.storage().binding()));
    let branch = items
        .iter()
        .find_map(|item| match item {
            StatementIr::AsyncGeneratorIf(plan) => Some(plan),
            _ => None,
        })
        .unwrap();
    assert_eq!(
        (
            branch.entry_state(),
            branch.then_branch().entry_state(),
            branch.then_branch().end_state(),
            branch.else_branch().entry_state(),
            branch.exit_state()
        ),
        (1, 2, 4, 5, 6)
    );
}

#[test]
fn mixed_nested_object_array_patterns_own_distinct_original_iterator_cells() {
    let function = values("async function* values(input,p){let [{[yield 'key']:[received=await p]}]=input;yield received;}");
    let mut items = Vec::new();
    walk(&function.body.statements, &mut items);
    let plans = items
        .iter()
        .filter_map(|item| match item {
            StatementIr::AsyncGeneratorArrayDestructuring(plan) => Some(plan),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(plans.len(), 2);
    assert_ne!(
        plans[0].storage().binding().slot,
        plans[1].storage().binding().slot
    );
    assert!(plans.iter().all(|plan| function
        .owned_env_bindings
        .contains(plan.storage().binding())));
    assert!(items.iter().any(|item| matches!(item, StatementIr::DeclarationEvaluation(value) if matches!(&value.expr, ExprIr::ObjectDestructuringOperation(_)))));
    assert_eq!(
        function
            .resumable_plan
            .as_ref()
            .unwrap()
            .suspension_points
            .iter()
            .map(|point| point.kind)
            .collect::<Vec<_>>(),
        [
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield
        ]
    );
}

#[test]
fn mixed_default_keeps_complete_conditional_arms_and_nested_callable_await_separate() {
    let function = values("async function* values(input,p,q){const [received=(yield true)?await p:await q]=input;yield received;}");
    let mut items = Vec::new();
    walk(&function.body.statements, &mut items);
    assert_eq!(
        items
            .iter()
            .filter(|item| matches!(item, StatementIr::AsyncGeneratorIf(_)))
            .count(),
        2
    );
    assert_eq!(
        function
            .resumable_plan
            .as_ref()
            .unwrap()
            .suspension_points
            .iter()
            .map(|point| point.kind)
            .collect::<Vec<_>>(),
        [
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield
        ]
    );
    let function = values("async function* values(input){const [named=async function(){return await 1;}]=input;yield named;}");
    assert_eq!(
        function
            .resumable_plan
            .as_ref()
            .unwrap()
            .suspension_points
            .len(),
        1
    );
}

#[test]
fn mixed_pattern_classic_for_retains_original_scoped_bound_names_and_whole_empty_item() {
    let function = values("async function* values(input,p){for(let [index=await(yield 'head'),step]=input;index<2;index++){var [v=await(yield index)]=input;yield ()=>[index,step,v];}}");
    let mut items = Vec::new();
    walk(&function.body.statements, &mut items);
    let plan = items
        .iter()
        .find_map(|item| match item {
            StatementIr::AsyncGeneratorLoop(plan) => Some(plan),
            _ => None,
        })
        .unwrap();
    assert!(plan.initialization().is_some());
    assert_eq!(
        items
            .iter()
            .filter(|item| matches!(item, StatementIr::AsyncGeneratorArrayDestructuring(_)))
            .count(),
        2
    );
    assert!(items
        .iter()
        .any(|item| matches!(item, StatementIr::EmptyStatementCompletion(_))));
    assert!(function
        .resumable_plan
        .as_ref()
        .unwrap()
        .suspension_points
        .iter()
        .any(|point| point.kind == ResumableSuspensionKindIr::Await));
}

#[test]
fn eager_pattern_used_mixed_rhs_keeps_original_owner_and_foreign_pattern_scopes_stay_refused() {
    let function = values(
        "async function* values(input){var received;return ([received]=await(yield input));}",
    );
    let mut items = Vec::new();
    walk(&function.body.statements, &mut items);
    assert!(!items
        .iter()
        .any(|item| matches!(item, StatementIr::AsyncGeneratorArrayDestructuring(_))));
    assert!(items
        .iter()
        .any(|item| matches!(item, StatementIr::AsyncAwait { .. })));
    for source in [
        "async function* values(input){for await(const item of input){let [received=await 1]=item;yield received;}}",
        "async function* values(input){for(const item of input){let [received=yield 1]=item;}}",
    ] {
        let function=values(source);let mut items=Vec::new();walk(&function.body.statements,&mut items);
        assert!(items.iter().any(|item| matches!(item,StatementIr::AsyncGeneratorForOf(_))));
        assert!(items.iter().any(|item| matches!(item,StatementIr::AsyncGeneratorArrayDestructuring(_))));
    }
    for source in ["class P{}class C extends P{async *values(input){[super[await 1]]=input;}}"] {
        let parsed = parse(source, ParseOptions::script()).unwrap();
        assert!(lower(&parsed).is_wasm_supported(), "{source}");
    }
}

#[test]
fn mixed_super_pattern_spends_one_reference_after_the_complete_default() {
    use lila_ir::{SuperPropertyCaptureMode, SuperPropertyMutationOperationIr};
    let function = values("class P{}class C extends P{async* values(input){return ([super[await(yield 'key')]=await(yield 'default')]=input);}}");
    let mut items = Vec::new();
    walk(&function.body.statements, &mut items);
    let mutations = items
        .iter()
        .filter_map(|item| match item {
            StatementIr::Expression(value) => match &value.expr {
                ExprIr::SuperPropertyMutation(mutation) => Some(mutation),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();
    let captures = mutations
        .iter()
        .filter_map(|mutation| match mutation.operation() {
            SuperPropertyMutationOperationIr::Capture(capture) => Some(capture),
            _ => None,
        })
        .collect::<Vec<_>>();
    let puts = mutations
        .iter()
        .filter_map(|mutation| match mutation.operation() {
            SuperPropertyMutationOperationIr::PutCaptured { capture, .. } => Some(capture),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(captures.len(), 1);
    assert_eq!(puts.len(), 1);
    assert_eq!(captures[0], puts[0]);
    assert_eq!(captures[0].mode(), SuperPropertyCaptureMode::WriteOnly);
    let names = [
        captures[0].receiver_storage_name(),
        captures[0].base_storage_name(),
        captures[0].referenced_name_storage_name(),
    ];
    for (index, name) in names.iter().enumerate() {
        assert_eq!(
            function
                .owned_env_bindings
                .iter()
                .filter(|binding| binding.name.as_str() == *name)
                .count(),
            1
        );
        assert!(names[..index].iter().all(|other| other != name));
    }
    assert!(items
        .iter()
        .any(|item| matches!(item, StatementIr::AsyncGeneratorArrayDestructuring(_))));
    let kinds = function
        .resumable_plan
        .as_ref()
        .unwrap()
        .suspension_points
        .iter()
        .map(|point| point.kind)
        .collect::<Vec<_>>();
    assert_eq!(
        &kinds[..4],
        [
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await,
            ResumableSuspensionKindIr::Yield,
            ResumableSuspensionKindIr::Await
        ]
    );
}

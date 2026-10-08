use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, AsyncGeneratorForOfIr, AsyncGeneratorIteratorProtocolIr, BindingMode, ExprIr,
    FunctionIr, ResumableResumeEnvironmentIr, ResumableSuspensionKindIr, StatementIr,
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
fn first(items: &[StatementIr]) -> &AsyncGeneratorForOfIr {
    for item in items {
        match item {
            StatementIr::AsyncGeneratorForOf(plan) => return plan,
            StatementIr::Block(block) if contains(&block.statements) => {
                return first(&block.statements)
            }
            StatementIr::LexicalBlock(items) if contains(items) => return first(items),
            _ => {}
        }
    }
    panic!("actual checked mixed iterator")
}
fn contains(items: &[StatementIr]) -> bool {
    items.iter().any(|item| match item {
        StatementIr::AsyncGeneratorForOf(_) => true,
        StatementIr::Block(block) => contains(&block.statements),
        StatementIr::LexicalBlock(items) => contains(items),
        _ => false,
    })
}

#[test]
fn mixed_sync_iterator_owns_complete_head_initialization_and_body() {
    let function = values(
        "async function* values(input,p){for(let item of await(yield input)){yield item;await p;}}",
    );
    let plan = first(&function.body.statements);
    assert_eq!(
        (
            plan.entry_state(),
            plan.head().region().end_state(),
            plan.acquisition_state(),
            plan.advance_state(),
            plan.initialization().entry_state(),
            plan.initialization().end_state(),
            plan.body().entry_state(),
            plan.body().end_state(),
            plan.exit_state()
        ),
        (0, 2, 3, 4, 5, 5, 6, 8, 9)
    );
    assert_eq!(plan.protocol(), AsyncGeneratorIteratorProtocolIr::Sync);
    assert_eq!(plan.head_mode(), BindingMode::Let);
    assert_eq!(plan.continue_state(), plan.advance_state());
    let cells = [
        plan.head_binding(),
        plan.incoming_binding(),
        plan.value_binding(),
    ];
    for (index, cell) in cells.iter().enumerate() {
        assert!(function.owned_env_bindings.contains(cell));
        for other in &cells[..index] {
            assert_ne!(cell.name, other.name);
            assert_ne!(cell.slot, other.slot);
        }
    }
    let [StatementIr::Lexical { name, init, .. }] =
        plan.initialization().block().statements.as_slice()
    else {
        panic!("original per-key initialization");
    };
    assert!(
        matches!(&init.expr, ExprIr::Identifier(read) if read == &plan.incoming_binding().name)
    );
    assert!(plan
        .lexical_environment()
        .unwrap()
        .iteration_environment
        .as_ref()
        .unwrap()
        .bindings
        .iter()
        .any(|binding| &binding.name == name));
    assert_eq!(function.resumable_plan.as_ref().unwrap().state_count, 10);
}

#[test]
fn mixed_awaited_iterator_has_exact_saved_next_and_close_sites() {
    let function = values("async function* values(input,p){for await(const item of await(yield input)){yield item;await p;}}");
    let plan = first(&function.body.statements);
    assert_eq!(
        plan.protocol(),
        AsyncGeneratorIteratorProtocolIr::Awaited {
            next_suspend_state: 4,
            next_resume_state: 5,
            close_suspend_state: 10,
            close_resume_state: 11,
        }
    );
    assert_eq!(
        (
            plan.initialization().entry_state(),
            plan.body().entry_state(),
            plan.body().end_state(),
            plan.exit_state()
        ),
        (6, 7, 9, 12)
    );
    let source = function.resumable_plan.as_ref().unwrap();
    assert_eq!(source.state_count, 13);
    assert_eq!(
        source
            .suspension_points
            .iter()
            .map(|point| (point.kind, point.suspend_state, point.resume_state))
            .collect::<Vec<_>>(),
        [
            (ResumableSuspensionKindIr::Yield, 0, 1),
            (ResumableSuspensionKindIr::Await, 1, 2),
            (ResumableSuspensionKindIr::ForAwaitNext, 4, 5),
            (ResumableSuspensionKindIr::Yield, 7, 8),
            (ResumableSuspensionKindIr::Await, 8, 9),
            (ResumableSuspensionKindIr::ForAwaitClose, 10, 11),
        ]
    );
    for point in &source.suspension_points {
        assert_eq!(
            point.resume_environment,
            if matches!(
                point.kind,
                ResumableSuspensionKindIr::ForAwaitNext | ResumableSuspensionKindIr::ForAwaitClose
            ) {
                ResumableResumeEnvironmentIr::SavedLexicalChain
            } else {
                ResumableResumeEnvironmentIr::InvocationOuter
            }
        );
    }
}

#[test]
fn mixed_iterator_initializer_retains_nested_array_and_original_cells() {
    let function = values("async function* values(input){for await(const [item=await(yield 'default')] of input){yield ()=>item;}}");
    let plan = first(&function.body.statements);
    assert!(plan
        .initialization()
        .block()
        .statements
        .iter()
        .any(|statement| matches!(statement, StatementIr::AsyncGeneratorArrayDestructuring(_))));
    assert!(plan.initialization().end_state() > plan.initialization().entry_state());
    assert!(plan.body().block().lexical_environment.is_none());
    let environment = plan.lexical_environment().unwrap();
    assert!(environment
        .iteration_environment
        .as_ref()
        .unwrap()
        .bindings
        .iter()
        .any(|binding| binding.name.contains("item")));
    assert!(
        matches!(&plan.head().value().expr, ExprIr::Identifier(name) if name == &plan.head_binding().name)
    );
}

#[test]
fn mixed_iterator_accepts_nested_whole_owners_and_keeps_resources_separate() {
    for source in [
        "async function* values(input){for(const item of input){for await(const next of item){yield next;await 1;}}}",
        "async function* values(input,scope){for await(const item of input){with(await(yield scope)){yield item;}}}",
        "async function* values(input,target){for((yield target)[await(yield 'key')] of input){yield 1;}}",
    ] { assert!(contains(&values(source).body.statements)); }
    let source = "async function* values(input){for await(const item of input){await using resource=null;yield item;}}";
    let function = values(source);
    let plan = first(&function.body.statements);
    let [StatementIr::Block(block)] = plan.body().block().statements.as_slice() else {
        panic!("original source Block");
    };
    assert!(block
        .statements
        .iter()
        .any(|item| matches!(item, StatementIr::AsyncGeneratorResourceScope(_))));
}

#[test]
fn mixed_resource_heads_own_registration_and_dispose_before_iterator_close() {
    for (source, asynchronous) in [
        (
            "async function* values(input){for(using item of input){yield ()=>item;}}",
            false,
        ),
        (
            "async function* values(input){for await(await using item of input){yield ()=>item;}}",
            true,
        ),
    ] {
        let function = values(source);
        let plan = first(&function.body.statements);
        let resource = plan.resource().expect("one original per-key capability");
        let [StatementIr::AsyncGeneratorResourceRegistration(registration)] =
            plan.initialization().block().statements.as_slice()
        else {
            panic!("actual head registration");
        };
        assert_eq!(
            registration.capability_binding(),
            resource.capability_binding()
        );
        assert!(
            matches!(&registration.initializer().expr,ExprIr::Identifier(name) if name==&plan.incoming_binding().name)
        );
        assert!(function
            .owned_env_bindings
            .contains(resource.capability_binding()));
        assert_ne!(
            resource.capability_binding().slot,
            plan.incoming_binding().slot
        );
        assert_eq!(plan.head_mode(), BindingMode::Const);
        if asynchronous {
            let lila_ir::AsyncGeneratorResourceCapabilityIr::Async(capability) =
                resource.capability()
            else {
                panic!("original async disposer");
            };
            let finalizer = capability.finalizer();
            assert_eq!(finalizer.dispose_state(), plan.body().end_state() + 1);
            assert_eq!(resource.exit_state(), finalizer.exit_state());
            let AsyncGeneratorIteratorProtocolIr::Awaited {
                close_suspend_state,
                ..
            } = plan.protocol()
            else {
                panic!("awaited iterator");
            };
            assert_eq!(close_suspend_state, resource.exit_state());
            let source = function.resumable_plan.as_ref().unwrap();
            assert!(source
                .resume_environment_plan()
                .enclosing_scope_resume_states()
                .contains(&finalizer.resume_state()));
            assert!(source
                .suspension_points
                .iter()
                .all(|point| point.resume_state != finalizer.resume_state()));
        } else {
            assert!(matches!(
                resource.capability(),
                lila_ir::AsyncGeneratorResourceCapabilityIr::Sync(_)
            ));
            assert_eq!(resource.exit_state(), plan.body().end_state() + 1);
            assert_eq!(plan.exit_state(), resource.exit_state());
        }
    }
}

#[test]
fn mixed_super_iterator_head_owns_its_complete_key_and_original_incoming_value() {
    use lila_ir::{SuperPropertyCaptureMode, SuperPropertyMutationOperationIr};
    let function = values("class P{}class C extends P{async* values(input){for await(super[await(yield 'key')] of input){yield this;}}}");
    let plan = first(&function.body.statements);
    let mutations = plan
        .initialization()
        .block()
        .statements
        .iter()
        .filter_map(|item| match item {
            StatementIr::Expression(value) => match &value.expr {
                ExprIr::SuperPropertyMutation(mutation) => Some(mutation),
                _ => None,
            },
            _ => None,
        })
        .collect::<Vec<_>>();
    let capture = mutations
        .iter()
        .find_map(|mutation| match mutation.operation() {
            SuperPropertyMutationOperationIr::Capture(capture) => Some(capture),
            _ => None,
        })
        .unwrap();
    let (put_capture, incoming) = mutations
        .iter()
        .find_map(|mutation| match mutation.operation() {
            SuperPropertyMutationOperationIr::PutCaptured { capture, value } => {
                Some((capture, value))
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(capture, put_capture);
    assert_eq!(capture.mode(), SuperPropertyCaptureMode::WriteOnly);
    assert!(
        matches!(&incoming.expr, ExprIr::Identifier(name) if name == &plan.incoming_binding().name)
    );
    assert!(plan.initialization().end_state() > plan.initialization().entry_state());
    assert!(function
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == capture.base_storage_name()));
}

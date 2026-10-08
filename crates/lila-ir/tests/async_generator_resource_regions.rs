use lila_front::{parse, ParseOptions};
use lila_ir::{
    lower, AsyncGeneratorResourceCapabilityIr, AsyncGeneratorResourceScopeIr, ExprIr, FunctionIr,
    ResourceDisposalHintIr, ResumableResumeEnvironmentIr, StatementIr,
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
fn scopes<'a>(items: &'a [StatementIr], output: &mut Vec<&'a AsyncGeneratorResourceScopeIr>) {
    for item in items {
        match item {
            StatementIr::AsyncGeneratorResourceScope(plan) => {
                output.push(plan);
                scopes(&plan.body().block().statements, output);
            }
            StatementIr::Block(block) => scopes(&block.statements, output),
            StatementIr::LexicalBlock(items) => scopes(items, output),
            StatementIr::EmptyStatementCompletion(item) => {
                scopes(std::slice::from_ref(item.statement()), output)
            }
            StatementIr::AsyncGeneratorLoop(plan) => {
                for region in plan.regions() {
                    scopes(&region.block().statements, output);
                }
            }
            StatementIr::AsyncGeneratorWith(plan) => {
                scopes(&plan.body().block().statements, output)
            }
            StatementIr::AsyncGeneratorForOf(plan) => {
                scopes(&plan.body().block().statements, output)
            }
            StatementIr::AsyncGeneratorSwitch(plan) => {
                for region in plan.regions() {
                    scopes(&region.block().statements, output);
                }
            }
            _ => {}
        }
    }
}
fn registrations<'a>(
    items: &'a [StatementIr],
    output: &mut Vec<&'a lila_ir::AsyncGeneratorResourceRegistrationIr>,
) {
    for item in items {
        match item {
            StatementIr::AsyncGeneratorResourceRegistration(operation) => output.push(operation),
            StatementIr::LexicalBlock(items) => registrations(items, output),
            StatementIr::EmptyStatementCompletion(item) => {
                registrations(std::slice::from_ref(item.statement()), output)
            }
            _ => {}
        }
    }
}
#[test]
fn mixed_resource_list_uses_one_capability_for_staged_sync_and_async_acquisition() {
    let function=values("async function* values(p){yield 0;using first=await(yield p),second=await(yield p);await using third=await(yield p);yield first;}");
    let mut owners = Vec::new();
    scopes(&function.body.statements, &mut owners);
    assert_eq!(owners.len(), 1);
    let plan = owners[0];
    assert_eq!(plan.capacity(), 3);
    assert_eq!(
        (
            plan.entry_state(),
            plan.body().end_state(),
            plan.exit_state()
        ),
        (0, 8, 11)
    );
    let AsyncGeneratorResourceCapabilityIr::Async(capability) = plan.capability() else {
        panic!("one original async disposal capability");
    };
    assert_eq!(
        (
            capability.finalizer().dispose_state(),
            capability.finalizer().resume_state(),
            capability.finalizer().exit_state()
        ),
        (9, 10, 11)
    );
    let mut operations = Vec::new();
    registrations(&plan.body().block().statements, &mut operations);
    assert_eq!(
        operations
            .iter()
            .map(|operation| operation.hint())
            .collect::<Vec<_>>(),
        [
            ResourceDisposalHintIr::Sync,
            ResourceDisposalHintIr::Sync,
            ResourceDisposalHintIr::Async
        ]
    );
    let mut names = std::collections::BTreeSet::new();
    for operation in operations {
        assert_eq!(operation.capability_binding(), plan.capability_binding());
        let ExprIr::Identifier(name) = &operation.initializer().expr else {
            panic!("pure retained initializer");
        };
        assert!(names.insert(name));
        assert_ne!(name, &plan.capability_binding().name);
        assert_eq!(
            function
                .owned_env_bindings
                .iter()
                .filter(|row| &row.name == name)
                .count(),
            1
        );
    }
    let tape = function.resumable_plan.as_ref().unwrap();
    assert_eq!(tape.state_count, 12);
    assert_eq!(tape.suspension_points.len(), 8);
    assert!(tape
        .suspension_points
        .iter()
        .all(|point| point.resume_environment == ResumableResumeEnvironmentIr::InvocationOuter));
}
#[test]
fn mixed_resource_nested_blocks_own_distinct_capabilities_and_single_original_finalizers() {
    let function=values("async function* values(p){using first=await(yield p);{await using second=await(yield p),third=null;yield ()=>second;}yield first;}");
    let mut owners = Vec::new();
    scopes(&function.body.statements, &mut owners);
    assert_eq!(owners.len(), 2);
    assert_eq!(owners[0].capacity(), 1);
    assert_eq!(owners[1].capacity(), 2);
    assert_ne!(
        owners[0].capability_binding().slot,
        owners[1].capability_binding().slot
    );
    assert!(matches!(
        owners[0].capability(),
        AsyncGeneratorResourceCapabilityIr::Sync(_)
    ));
    let AsyncGeneratorResourceCapabilityIr::Async(capability) = owners[1].capability() else {
        panic!("actual nested async scope");
    };
    let plan = function.resumable_plan.as_ref().unwrap();
    assert!(plan
        .resume_environment_plan()
        .enclosing_scope_resume_states()
        .contains(&capability.finalizer().resume_state()));
    assert!(plan
        .suspension_points
        .iter()
        .all(|point| point.resume_state != capability.finalizer().resume_state()));
}
#[test]
fn mixed_resource_body_keeps_complete_iterator_and_reference_owners() {
    for source in [
        "async function* values(p,scope){using resource=await(yield p);with(await(yield scope)){for await(const value of p){yield value;}}}",
        "async function* values(p){while(yield true){await using resource=await(yield p);yield resource;break;}}",
    ] {let function=values(source);let mut owners=Vec::new();scopes(&function.body.statements,&mut owners);assert_eq!(owners.len(),1);}
    let function=values("async function* values(p){switch(yield 1){case 1:{await using resource=await(yield p);yield resource;}}}");
    assert!(function.body.statements.iter().any(|statement|matches!(statement,StatementIr::AsyncGeneratorSwitch(plan) if plan.resource().is_none())));
    let mut owners = Vec::new();
    scopes(&function.body.statements, &mut owners);
    assert_eq!(owners.len(), 1);
    assert_eq!(owners[0].capacity(), 1);
    let function = values("async function* values(p){for(using resource of p){yield resource;}}");
    assert!(function.body.statements.iter().any(|statement|matches!(statement,StatementIr::AsyncGeneratorForOf(plan) if plan.resource().is_some())));
}

#[test]
fn mixed_classic_resource_head_keeps_one_capability_across_all_loop_phases() {
    let function=values("async function* values(p){for(await using first=await(yield p),second=null;await(yield true);yield 0){yield ()=>first;}}");
    let [StatementIr::AsyncGeneratorLoop(plan)] = function.body.statements.as_slice() else {
        panic!("complete classic loop");
    };
    let resource = plan.resource().expect("one whole classic lifetime");
    assert_eq!(resource.capacity(), 2);
    assert_eq!(
        (
            resource.entry_state(),
            resource.body_end_state(),
            resource.exit_state()
        ),
        (0, 9, 12)
    );
    let AsyncGeneratorResourceCapabilityIr::Async(capability) = resource.capability() else {
        panic!("actual shared async capability");
    };
    assert_eq!(
        (
            capability.finalizer().dispose_state(),
            capability.finalizer().resume_state()
        ),
        (10, 11)
    );
    let mut operations = Vec::new();
    registrations(
        &plan.initialization().unwrap().block().statements,
        &mut operations,
    );
    assert_eq!(operations.len(), 2);
    assert!(operations
        .iter()
        .all(|operation| operation.capability_binding() == resource.capability_binding()));
    assert_eq!(plan.exit_state(), resource.exit_state());
    assert!(plan.lexical_environment().is_some());
    let source = function.resumable_plan.as_ref().unwrap();
    assert_eq!(source.state_count, 13);
    assert!(source
        .resume_environment_plan()
        .enclosing_scope_resume_states()
        .contains(&11));
    assert!(source
        .suspension_points
        .iter()
        .all(|point| point.resume_state != 11));
}

#[test]
fn mixed_case_lexical_blocks_dispose_their_own_resources_before_fallthrough() {
    let function=values("async function* values(p){switch(yield p){case 1:{using first=await(yield p);yield first;}case 2:{await using second=await(yield p);yield ()=>second;}}}");
    let [StatementIr::AsyncGeneratorSwitch(plan)] = function.body.statements.as_slice() else {
        panic!("complete CaseBlock");
    };
    assert!(
        plan.resource().is_none(),
        "each lexical block owns disposal"
    );
    let mut owners = Vec::new();
    scopes(&function.body.statements, &mut owners);
    assert_eq!(owners.len(), 2);
    assert_ne!(
        owners[0].capability_binding(),
        owners[1].capability_binding()
    );
    for (case, resource) in plan.cases().iter().zip(&owners) {
        assert_eq!(resource.capacity(), 1);
        assert_eq!(resource.entry_state(), case.body().entry_state());
        assert_eq!(resource.exit_state(), case.body().end_state());
        let mut operations = Vec::new();
        registrations(&resource.body().block().statements, &mut operations);
        assert_eq!(operations.len(), 1);
        assert_eq!(
            operations[0].capability_binding(),
            resource.capability_binding()
        );
        assert_ne!(
            resource.capability_binding().slot,
            plan.discriminant_binding().slot
        );
        assert_ne!(
            resource.capability_binding().slot,
            plan.value_binding().slot
        );
    }
    assert_eq!((owners[0].entry_state(), owners[0].exit_state()), (5, 9));
    assert_eq!((owners[1].entry_state(), owners[1].exit_state()), (10, 16));
    assert!(owners[0].exit_state() < owners[1].entry_state());
    assert!(matches!(
        owners[0].capability(),
        AsyncGeneratorResourceCapabilityIr::Sync(_)
    ));
    assert!(plan
        .cases()
        .iter()
        .all(|case| case.body().block().lexical_environment.is_none()));
    let AsyncGeneratorResourceCapabilityIr::Async(capability) = owners[1].capability() else {
        panic!("the second block owns its async finalizer");
    };
    assert_eq!(capability.finalizer().dispose_state(), 14);
    assert_eq!(plan.exit_state(), capability.finalizer().exit_state() + 1);
    assert_eq!(function.resumable_plan.as_ref().unwrap().state_count, 18);
}

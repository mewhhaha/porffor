use super::*;
use crate::async_generator_source::AsyncGeneratorForOfSource;
use crate::generator_loop_control::{mixed_sequence_end, sequence_end};
use crate::{AsyncGeneratorLoopExpressionIr, AsyncGeneratorLoopRegionIr};
use boa_ast::statement::iteration::ForOfLoop;
use boa_ast::visitor::{VisitWith, Visitor};
use std::ops::ControlFlow;

const SYNC: &str = "async function* g(input){for(const [item=await(yield 'init')] of await(yield 'head')){yield ()=>item;await 0;}}";
const AWAITED: &str = "async function* g(input){for await(const [item=await(yield 'init')] of await(yield 'head')){yield ()=>item;await 0;}}";

fn with_plan(
    source: &str,
    check: impl FnOnce(&ForOfLoop, &AsyncGeneratorForOfIr, &[OwnedEnvBindingIr]),
) {
    let parsed = lila_front::parse(source, lila_front::ParseOptions::script()).unwrap();
    let program = crate::lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let function = program
        .script
        .as_ref()
        .unwrap()
        .functions
        .iter()
        .find(|function| function.name == "g")
        .unwrap();
    let plan = function
        .body
        .statements
        .iter()
        .find_map(|statement| match statement {
            StatementIr::AsyncGeneratorForOf(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .expect("actual complete mixed iterator owner");
    parsed
        .as_script()
        .unwrap()
        .with_compiler_session(|script, _| {
            struct Find<'ast>(Option<&'ast ForOfLoop>);
            impl<'ast> Visitor<'ast> for Find<'ast> {
                type BreakTy = ();
                fn visit_for_of_loop(&mut self, source: &'ast ForOfLoop) -> ControlFlow<()> {
                    self.0 = Some(source);
                    ControlFlow::Break(())
                }
            }
            let mut found = Find(None);
            let _ = script.visit_with(&mut found);
            check(found.0.unwrap(), plan, &function.owned_env_bindings);
        });
}

fn rebuild(
    source: &ForOfLoop,
    plan: AsyncGeneratorForOfIr,
    inventory: &[OwnedEnvBindingIr],
) -> Result<AsyncGeneratorForOfIr, AsyncGeneratorForOfError> {
    AsyncGeneratorForOfIr::new(
        AsyncGeneratorForOfSource::for_execution(source, plan.execution())
            .unwrap()
            .states(plan.entry_state())
            .unwrap(),
        plan.head,
        plan.head_binding,
        plan.incoming_binding,
        plan.value_binding,
        plan.initializer,
        plan.body,
        inventory,
    )
}

#[test]
fn complete_mixed_iterators_bind_implicit_and_explicit_points_to_distinct_phases() {
    for source in [SYNC, AWAITED] {
        with_plan(source, |source, plan, inventory| {
            assert_eq!(rebuild(source, plan.clone(), inventory).unwrap(), *plan);
            assert!(plan.head().region().end_state() < plan.acquisition_state());
            assert!(plan.advance_state() < plan.initialization().entry_state());
            assert!(plan.initialization().end_state() < plan.body().entry_state());
            assert_eq!(plan.continue_state(), plan.advance_state());
            let implicit: Vec<_> = plan
                .suspensions()
                .iter()
                .filter(|point| {
                    matches!(
                        point.kind,
                        ResumableSuspensionKindIr::ForAwaitNext
                            | ResumableSuspensionKindIr::ForAwaitClose
                    )
                })
                .collect();
            assert_eq!(implicit.len(), if source.r#await() { 2 } else { 0 });
            assert!(implicit
                .iter()
                .all(|point| point.resume_environment
                    == ResumableResumeEnvironmentIr::SavedLexicalChain));
            let statement = StatementIr::AsyncGeneratorForOf(Box::new(plan.clone()));
            assert_eq!(
                mixed_sequence_end(std::slice::from_ref(&statement), plan.entry_state()),
                Ok(plan.exit_state())
            );
            assert!(sequence_end(std::slice::from_ref(&statement), plan.entry_state()).is_err());
            assert!(crate::async_switch::sequence_exit(
                std::slice::from_ref(&statement),
                plan.entry_state()
            )
            .is_err());
            assert!(crate::SynchronousLoopBodyIr::new(&statement).is_err());
        });
    }
}

#[test]
fn mixed_iterator_carrier_requires_three_distinct_actual_invocation_cells_and_head_publication() {
    with_plan(AWAITED, |source, plan, inventory| {
        let missing: Vec<_> = inventory
            .iter()
            .filter(|binding| *binding != plan.head_binding())
            .cloned()
            .collect();
        assert_eq!(
            rebuild(source, plan.clone(), &missing),
            Err(AsyncGeneratorForOfError::UnallocatedBinding)
        );
        let mut aliased = plan.clone();
        aliased.value_binding = plan.head_binding.clone();
        assert_eq!(
            rebuild(source, aliased, inventory),
            Err(AsyncGeneratorForOfError::AliasedBindings)
        );
        let mut ambiguous = inventory.to_vec();
        ambiguous.push(OwnedEnvBindingIr {
            mutability: crate::EnvironmentBindingMutabilityIr::Mutable,
            name: "foreign".into(),
            slot: plan.incoming_binding.slot,
        });
        assert_eq!(
            rebuild(source, plan.clone(), &ambiguous),
            Err(AsyncGeneratorForOfError::UnallocatedBinding)
        );
        let mut no_publication = plan.clone();
        let mut block = plan.head().region().block().clone();
        block.statements.pop();
        let states = AsyncGeneratorForOfSource::new(source)
            .unwrap()
            .states(plan.entry_state())
            .unwrap();
        no_publication.head = AsyncGeneratorLoopExpressionIr::new(
            AsyncGeneratorLoopRegionIr::new(block, states.head()).unwrap(),
            plan.head().value().clone(),
        )
        .into();
        assert_eq!(
            rebuild(source, no_publication, inventory),
            Err(AsyncGeneratorForOfError::MissingHeadPublication)
        );
    });
}

#[test]
fn checked_iterator_initializer_cannot_be_rebound_to_an_equal_shaped_foreign_source_or_input() {
    with_plan(SYNC, |source, plan, inventory| {
        let mut input = plan.clone();
        input.incoming_binding = plan.value_binding.clone();
        assert_eq!(
            rebuild(source, input, inventory),
            Err(AsyncGeneratorForOfError::ForeignInitializer)
        );
        with_plan(SYNC, |other_source, _, _| {
            assert_eq!(
                rebuild(other_source, plan.clone(), inventory),
                Err(AsyncGeneratorForOfError::ForeignInitializer)
            );
        });
    });
}

#[test]
fn complete_iterator_cannot_relabel_an_original_protocol_as_mixed() {
    for (source, execution) in [
        (
            "function* g(input){for(const [item=yield 'init'] of yield input){yield item;}}",
            ResumableRegionProtocolIr::Generator,
        ),
        (
            "async function g(input){for await(const [item=await 1] of await input){await item;}}",
            ResumableRegionProtocolIr::Async,
        ),
    ] {
        with_plan(source, |source, plan, inventory| {
            assert_eq!(plan.execution(), execution);
            assert_eq!(rebuild(source, plan.clone(), inventory).unwrap(), *plan);
            let opposite = AsyncGeneratorForOfSource::for_execution(
                source,
                ResumableRegionProtocolIr::AsyncGenerator,
            )
            .unwrap()
            .states(plan.entry_state())
            .unwrap();
            assert_eq!(
                AsyncGeneratorForOfIr::new(
                    opposite,
                    plan.head.clone(),
                    plan.head_binding.clone(),
                    plan.incoming_binding.clone(),
                    plan.value_binding.clone(),
                    plan.initializer.clone(),
                    plan.body.clone(),
                    inventory
                ),
                Err(AsyncGeneratorForOfError::InvalidStates)
            );
        });
    }
}

#[test]
fn resource_iterator_owns_the_original_capability_until_before_close_or_advance() {
    for source in [
        "async function* g(input){for(using item of input){yield await item;}}",
        "async function* g(input){for await(await using item of input){yield await item;}}",
    ] {
        with_plan(source, |source, plan, inventory| {
            assert_eq!(rebuild(source, plan.clone(), inventory).unwrap(), *plan);
            let resource = plan.resource().expect("actual per-key resource head");
            let iteration_exit = match plan.protocol() {
                AsyncGeneratorIteratorProtocolIr::Sync => plan.exit_state(),
                AsyncGeneratorIteratorProtocolIr::Awaited {
                    close_suspend_state,
                    ..
                } => close_suspend_state,
            };
            assert_eq!(resource.exit_state(), iteration_exit);
            assert!(resource.exit_state() > plan.body().end_state());
            match resource.capability() {
                crate::AsyncGeneratorResourceCapabilityIr::Sync(_) => {
                    assert_eq!(resource.exit_state(), plan.body().end_state() + 1)
                }
                crate::AsyncGeneratorResourceCapabilityIr::Async(capability) => {
                    assert_eq!(
                        capability.finalizer().entry_state(),
                        plan.initialization().entry_state()
                    );
                    assert_eq!(
                        capability.finalizer().dispose_state(),
                        plan.body().end_state() + 1
                    );
                }
            }
            let missing: Vec<_> = inventory
                .iter()
                .filter(|row| *row != resource.capability_binding())
                .cloned()
                .collect();
            assert_eq!(
                rebuild(source, plan.clone(), &missing),
                Err(AsyncGeneratorForOfError::UnallocatedBinding)
            );
            assert!(mixed_sequence_end(
                &plan.initialization().block().statements,
                plan.initialization().entry_state()
            )
            .is_err());
        });
    }
}

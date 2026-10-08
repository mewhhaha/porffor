use super::*;
use crate::async_generator_source::AsyncGeneratorClassicLoopSource;
use crate::{ResumableResumeEnvironmentIr, ResumableSuspensionKindIr, StatementIr};
use boa_ast::statement::iteration::ForLoop;
use boa_ast::visitor::{VisitWith, Visitor};
use std::ops::ControlFlow;

const SOURCE: &str =
    "async function* g(input){for(let i=await input;yield i;i=await input){yield i;await input;}}";

fn actual_loop(
    source: &str,
    consume: impl FnOnce(&ForLoop, &AsyncGeneratorLoopIr, &[OwnedEnvBindingIr]),
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
            StatementIr::AsyncGeneratorLoop(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .expect("actual checked mixed classic loop");
    parsed
        .as_script()
        .unwrap()
        .with_compiler_session(|script, _| {
            struct Find<'ast>(Vec<&'ast ForLoop>);
            impl<'ast> Visitor<'ast> for Find<'ast> {
                type BreakTy = ();
                fn visit_statement(&mut self, source: &'ast boa_ast::Statement) -> ControlFlow<()> {
                    if let boa_ast::Statement::ForLoop(source) = source {
                        self.0.push(source);
                    }
                    source.visit_with(self)
                }
            }
            let mut found = Find(Vec::new());
            let _ = script.visit_with(&mut found);
            consume(found.0[0], plan, &function.owned_env_bindings);
        });
}

fn rebuild(
    source: &ForLoop,
    plan: AsyncGeneratorLoopIr,
    inventory: &[OwnedEnvBindingIr],
) -> Result<AsyncGeneratorLoopIr, AsyncGeneratorControlError> {
    let states = AsyncGeneratorClassicLoopSource::for_loop(source)
        .unwrap()
        .states(plan.entry_state())
        .unwrap();
    AsyncGeneratorLoopIr::checked(
        states,
        plan.initialization,
        plan.test,
        plan.body,
        plan.update,
        plan.lexical_environment,
        plan.value_binding,
        plan.resource,
        inventory,
    )
}

#[test]
fn plain_async_loop_rejects_foreign_protocols_and_missing_phases() {
    const PLAIN: &str =
        "async function g(input){for(let i=await input;await input;i=await input){await input;}}";
    actual_loop(PLAIN, |source, plan, inventory| {
        use crate::async_generator_source::PlainAsyncClassicLoopSource;
        let rebuild = |plan: AsyncGeneratorLoopIr| {
            AsyncGeneratorLoopIr::new_plain_async(
                PlainAsyncClassicLoopSource::For(source)
                    .plan(plan.entry_state())
                    .unwrap(),
                plan.initialization,
                plan.test,
                plan.body,
                plan.update,
                plan.lexical_environment,
                plan.value_binding,
                plan.resource,
                inventory,
            )
        };
        assert_eq!(plan.execution(), ResumableRegionProtocolIr::Async);
        assert_eq!(rebuild(plan.clone()).unwrap(), *plan);
        let mut missing = plan.clone();
        missing.update = None;
        assert_eq!(
            rebuild(missing),
            Err(AsyncGeneratorControlError::InvalidPhases)
        );
        let states = PlainAsyncClassicLoopSource::For(source)
            .plan(plan.entry_state())
            .unwrap();
        let mut foreign = plan.clone();
        foreign.body = AsyncGeneratorLoopRegionIr::new(plan.body.block().clone(), states.body())
            .unwrap()
            .into();
        assert_eq!(
            rebuild(foreign),
            Err(AsyncGeneratorControlError::ForeignContinuation)
        );
        let mut erased = plan.body.block().clone();
        assert!(remove_await(&mut erased.statements));
        assert!(
            ResumableRegionIr::new(erased, states.body(), ResumableRegionProtocolIr::Async)
                .is_err()
        );
        let statement = StatementIr::AsyncGeneratorLoop(Box::new(plan.clone()));
        assert!(mixed_sequence_end(std::slice::from_ref(&statement), plan.entry_state()).is_err());
        assert_eq!(
            crate::async_switch::sequence_exit(
                std::slice::from_ref(&statement),
                plan.entry_state()
            ),
            Ok(plan.exit_state())
        );
    });
}

#[test]
fn mixed_loop_consumes_actual_tape_cells_and_closed_protocol() {
    actual_loop(SOURCE, |source, plan, inventory| {
        assert_eq!(rebuild(source, plan.clone(), inventory).unwrap(), *plan);
        assert_eq!(
            plan.suspensions
                .iter()
                .map(|point| point.kind)
                .collect::<Vec<_>>(),
            vec![
                ResumableSuspensionKindIr::Await,
                ResumableSuspensionKindIr::Yield,
                ResumableSuspensionKindIr::Yield,
                ResumableSuspensionKindIr::Await,
                ResumableSuspensionKindIr::Await,
            ]
        );
        assert!(
            plan.suspensions
                .iter()
                .all(|point| point.resume_environment
                    == ResumableResumeEnvironmentIr::InvocationOuter)
        );
        let statement = StatementIr::AsyncGeneratorLoop(Box::new(plan.clone()));
        assert_eq!(
            mixed_sequence_end(std::slice::from_ref(&statement), plan.entry_state()),
            Ok(plan.exit_state())
        );
        assert_eq!(
            crate::generator_loop_control::sequence_end(
                std::slice::from_ref(&statement),
                plan.entry_state()
            ),
            Err(AsyncGeneratorControlError::ForeignContinuation)
        );
        assert_eq!(
            plan.continue_state(),
            plan.update().unwrap().region().entry_state()
        );
    });
}

#[test]
fn classic_resource_carrier_requires_its_source_lifetime_and_disjoint_loop_value() {
    const RESOURCE: &str = "async function* g(input){for(await using item=await(yield input);await input;yield 0){yield ()=>item;continue;}}";
    actual_loop(RESOURCE, |source, plan, inventory| {
        assert_eq!(rebuild(source, plan.clone(), inventory).unwrap(), *plan);
        let resource = plan.resource().unwrap();
        assert_eq!(resource.entry_state(), plan.entry_state());
        assert_eq!(
            resource.body_end_state(),
            plan.update().unwrap().region().end_state()
        );
        assert_eq!(resource.exit_state(), plan.exit_state());
        assert!(plan
            .lexical_environment()
            .unwrap()
            .per_iteration_slots
            .is_empty());
        let mut missing = plan.clone();
        missing.resource = None;
        assert_eq!(
            rebuild(source, missing, inventory),
            Err(AsyncGeneratorControlError::InvalidPhases)
        );
        let mut aliased = plan.clone();
        aliased.value_binding = resource.capability_binding().clone();
        assert_eq!(
            rebuild(source, aliased, inventory),
            Err(AsyncGeneratorControlError::InvalidPhases)
        );
        actual_loop(RESOURCE, |foreign, _, _| {
            assert_eq!(
                rebuild(foreign, plan.clone(), inventory),
                Err(AsyncGeneratorControlError::InvalidPhases)
            );
        });
    });
}

#[test]
fn mixed_loop_rejects_missing_duplicated_and_aliased_completion_cells() {
    actual_loop(SOURCE, |source, plan, inventory| {
        let missing = inventory
            .iter()
            .filter(|binding| *binding != plan.value_binding())
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(
            rebuild(source, plan.clone(), &missing),
            Err(AsyncGeneratorControlError::InvalidPhases)
        );
        let mut duplicated = inventory.to_vec();
        duplicated.push(plan.value_binding.clone());
        assert_eq!(
            rebuild(source, plan.clone(), &duplicated),
            Err(AsyncGeneratorControlError::InvalidPhases)
        );
        let mut alias = plan.value_binding.clone();
        alias.name.push_str("alias");
        let mut aliased = inventory.to_vec();
        aliased.push(alias);
        assert_eq!(
            rebuild(source, plan.clone(), &aliased),
            Err(AsyncGeneratorControlError::InvalidPhases)
        );
    });
}

#[test]
fn mixed_loop_rejects_missing_or_overlapping_source_phases() {
    actual_loop(SOURCE, |source, plan, inventory| {
        let mut missing = plan.clone();
        missing.update = None;
        assert_eq!(
            rebuild(source, missing, inventory),
            Err(AsyncGeneratorControlError::InvalidPhases)
        );
        let mut overlapping = plan.clone();
        overlapping.test =
            ResumableExpressionIr::new(overlapping.body.clone(), overlapping.test.value().clone());
        assert_eq!(
            rebuild(source, overlapping, inventory),
            Err(AsyncGeneratorControlError::InvalidPhases)
        );
    });
}

fn remove_await(statements: &mut Vec<StatementIr>) -> bool {
    for index in 0..statements.len() {
        if matches!(&statements[index], StatementIr::AsyncAwait { .. }) {
            statements.remove(index);
            return true;
        }
        let removed = match &mut statements[index] {
            StatementIr::LexicalBlock(statements) => remove_await(statements),
            StatementIr::Block(block) => remove_await(&mut block.statements),
            _ => false,
        };
        if removed {
            return true;
        }
    }
    false
}

#[test]
fn mixed_loop_rejects_a_lost_await_even_with_retained_range_words() {
    actual_loop(SOURCE, |source, plan, inventory| {
        let mut damaged = plan.clone();
        let mut body = damaged.body.as_mixed().unwrap().clone();
        assert!(remove_await(&mut body.block.statements));
        damaged.body = body.into();
        assert_eq!(
            rebuild(source, damaged, inventory),
            Err(AsyncGeneratorControlError::UnconsumedSourceSuspension)
        );
    });
}

#[test]
fn mixed_loop_rejects_equal_extent_source_with_a_different_suspension_kind() {
    actual_loop(SOURCE, |_, plan, inventory| {
        let alternate = SOURCE.replace("yield i;await input;", "await i;await input;");
        let parsed = lila_front::parse(&alternate, lila_front::ParseOptions::script()).unwrap();
        parsed
            .as_script()
            .unwrap()
            .with_compiler_session(|script, _| {
                struct Find<'ast>(Option<&'ast ForLoop>);
                impl<'ast> Visitor<'ast> for Find<'ast> {
                    type BreakTy = ();
                    fn visit_statement(
                        &mut self,
                        source: &'ast boa_ast::Statement,
                    ) -> ControlFlow<()> {
                        if let boa_ast::Statement::ForLoop(source) = source {
                            self.0 = Some(source);
                        }
                        source.visit_with(self)
                    }
                }
                let mut found = Find(None);
                let _ = script.visit_with(&mut found);
                assert_eq!(
                    rebuild(found.0.unwrap(), plan.clone(), inventory),
                    Err(AsyncGeneratorControlError::UnconsumedSourceSuspension)
                );
            });
    });
}

#[test]
fn mixed_try_rejects_an_unpaired_async_owner_before_publication() {
    actual_loop("async function* g(input){for(let i=0;i<1;i++){try{yield i;await input;}finally{await input;}}}",
        |source, plan, inventory| {
            let mut body = plan.body.as_mixed().unwrap().clone();
            let StatementIr::TryFinally { async_plan, .. } = &mut body.block.statements[0] else {
                panic!("actual source TryFinally");
            };
            async_plan.as_mut().unwrap().exit_state += 1;
            assert!(matches!(mixed_sequence_end(&body.block.statements, body.entry_state),
                Err(AsyncGeneratorControlError::ForeignContinuation)));
            // The source tape is still exact, so the region constructor must
            // validate the paired owner rather than rely on a suspension count.
            let states = AsyncGeneratorClassicLoopSource::for_loop(source).unwrap().states(plan.entry_state()).unwrap();
            assert_eq!(AsyncGeneratorLoopRegionIr::new(body.block, states.body()),
                Err(AsyncGeneratorControlError::ForeignContinuation));
            assert!(rebuild(source, plan.clone(), inventory).is_ok());
        });
}

#[test]
fn mixed_eager_phases_have_distinct_entries_without_fake_suspensions() {
    actual_loop(
        "async function* g(){for(let i=0;i<1;i++){i;}}",
        |source, plan, inventory| {
            assert!(plan.suspensions.is_empty());
            let regions = plan.regions().collect::<Vec<_>>();
            for pair in regions.windows(2) {
                assert_eq!(
                    pair[0].end_state().checked_add(1),
                    Some(pair[1].entry_state())
                );
            }
            assert_eq!(rebuild(source, plan.clone(), inventory).unwrap(), *plan);
        },
    );
}

fn actual_function_plan(source: &str) -> crate::ResumablePlanIr {
    let parsed = lila_front::parse(source, lila_front::ParseOptions::script()).unwrap();
    let program = crate::lower(&parsed);
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    program
        .script
        .unwrap()
        .functions
        .into_iter()
        .find(|function| function.name == "g")
        .unwrap()
        .resumable_plan
        .unwrap()
}

#[test]
fn retained_branch_relocation_updates_both_certified_state_lists_atomically() {
    let original = actual_function_plan(SOURCE);
    let branch_end = original.suspension_points[0].resume_state;
    assert!(!original
        .resume_environment_plan
        .enclosing_scope_resume_states()
        .is_empty());
    let mut relocated = original.clone();
    relocated.insert_legacy_branch_exit(1, branch_end).unwrap();
    assert!(relocated.matches_resume_environment_plan());
    assert_eq!(relocated.state_count, original.state_count + 1);
    for (index, (before, after)) in original
        .suspension_points
        .iter()
        .zip(&relocated.suspension_points)
        .enumerate()
    {
        assert_eq!(after.kind, before.kind);
        assert_eq!(after.resume_environment, before.resume_environment);
        assert_eq!(
            after.suspend_state,
            before.suspend_state + u32::from(index >= 1)
        );
        assert_eq!(
            after.resume_state,
            before.resume_state + u32::from(index >= 1)
        );
    }
    assert_eq!(
        relocated
            .resume_environment_plan
            .enclosing_scope_resume_states(),
        original
            .resume_environment_plan
            .enclosing_scope_resume_states()
            .iter()
            .map(|state| *state + u32::from(*state > branch_end))
            .collect::<Vec<_>>()
    );
    for damaged in [
        {
            let mut plan = original.clone();
            plan.state_count = u32::MAX;
            plan
        },
        {
            let mut plan = original.clone();
            plan.suspension_points[0].resume_environment =
                ResumableResumeEnvironmentIr::SavedLexicalChain;
            plan
        },
    ] {
        let before = damaged.clone();
        let mut plan = damaged;
        assert!(plan.insert_legacy_branch_exit(1, branch_end).is_err());
        assert_eq!(plan, before);
    }
}

#[test]
fn async_generator_methods_retain_complete_literal_loop_suspensions() {
    for source in [
        "const object={async *g(p){while(true){yield {a:await p};}}};",
        "class C{async *g(p){while(true){yield {a:await p};}}}",
    ] {
        let parsed = lila_front::parse(source, lila_front::ParseOptions::script()).unwrap();
        let program = crate::lower(&parsed);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let loop_plan = program
            .script
            .as_ref()
            .unwrap()
            .functions
            .iter()
            .flat_map(|function| &function.body.statements)
            .find_map(|statement| match statement {
                StatementIr::AsyncGeneratorLoop(plan) => Some(plan),
                _ => None,
            })
            .expect("method retains its complete loop owner");
        assert_eq!(
            loop_plan
                .suspensions()
                .iter()
                .map(|point| point.kind)
                .collect::<Vec<_>>(),
            [
                ResumableSuspensionKindIr::Await,
                ResumableSuspensionKindIr::Yield
            ]
        );
    }
}

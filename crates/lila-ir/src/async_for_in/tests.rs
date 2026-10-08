use super::protocol_test_support::*;
use super::*;
use crate::async_generator_source::AsyncGeneratorForInSource;
use crate::*;

const SOURCE: &str = "async function g(view){for(let key in await view){await key;await key;}}";

#[test]
fn async_for_in_constructor_consumes_actual_tape_and_four_original_cells() {
    actual_plan(SOURCE, |sources, interner, plan, inventory| {
        assert_eq!(
            rebuild(sources[0], interner, plan.clone(), inventory).unwrap(),
            *plan
        );
        assert_eq!(plan.execution(), ResumableRegionProtocolIr::Async);
        let statement = StatementIr::AsyncGeneratorForIn(Box::new(plan.clone()));
        assert_eq!(
            crate::async_switch::sequence_exit(
                std::slice::from_ref(&statement),
                plan.entry_state()
            ),
            Ok(plan.exit_state())
        );
        let mut points = Vec::new();
        crate::async_with::collect_async_suspensions(std::slice::from_ref(&statement), &mut points);
        let source =
            AsyncGeneratorForInSource::for_execution(sources[0], plan.execution()).unwrap();
        assert_eq!(
            points,
            source.states(plan.entry_state()).unwrap().suspensions()
        );
        assert_eq!(points.len(), 3);
        assert_eq!(plan.continue_state(), plan.advance_state());
        for binding in [
            plan.head_binding(),
            plan.enumerator_binding(),
            plan.key_binding(),
            plan.value_binding(),
        ] {
            assert!(inventory.contains(binding));
        }
    });
}

#[test]
fn async_for_in_constructor_requires_unique_actual_invocation_inventory() {
    actual_plan(SOURCE, |sources, interner, plan, inventory| {
        for binding in [
            plan.head_binding(),
            plan.enumerator_binding(),
            plan.key_binding(),
            plan.value_binding(),
        ] {
            let mut missing = inventory.to_vec();
            missing.retain(|row| row != binding);
            let mut duplicate = inventory.to_vec();
            duplicate.push(binding.clone());
            let mut alias = inventory.to_vec();
            alias.push(OwnedEnvBindingIr {
                name: "foreign.cell".into(),
                slot: binding.slot,
            });
            for inventory in [missing, duplicate, alias] {
                assert_eq!(
                    rebuild(sources[0], interner, plan.clone(), &inventory),
                    Err(GeneratorForInControlError::UnallocatedBinding)
                );
            }
        }
        let mut aliased = ForInInputs::from_plan(plan);
        aliased.key_binding = aliased.head_binding.clone();
        assert_eq!(
            rebuild_with_inputs(sources[0], interner, plan.clone(), aliased, inventory),
            Err(GeneratorForInControlError::AliasedBindings)
        );
    });
}

#[test]
fn async_for_in_constructor_requires_completed_raw_head_publication() {
    actual_plan(SOURCE, |sources, interner, plan, inventory| {
        let missing = change_head(sources[0], plan, |statements| {
            statements.pop();
        })
        .unwrap();
        let mut foreign = plan.clone();
        foreign.head = ResumableExpressionIr::new(
            plan.head.region().clone(),
            TypedExpr::from_info(
                plan.head.value().value_info(),
                ExprIr::Identifier("foreign.head".into()),
            ),
        );
        for changed in [missing, foreign] {
            assert_eq!(
                rebuild(sources[0], interner, changed, inventory),
                Err(GeneratorForInControlError::MissingHeadPublication)
            );
        }
    });
}

#[test]
fn async_for_in_constructor_rejects_bare_key_reads_and_empty_initialization() {
    actual_plan(
        "async function g(view){var key;for(key in await view){await key;}}",
        |sources, interner, plan, inventory| {
            // The only constructor accepts the original opaque binding proof;
            // even exchanging two otherwise allocated cells must reject it.
            assert!(!plan.initialization().statements.is_empty());
            let mut changed = ForInInputs::from_plan(plan);
            changed.exchange_key_and_value();
            assert_eq!(
                rebuild_with_inputs(sources[0], interner, plan.clone(), changed, inventory),
                Err(GeneratorForInControlError::InvalidInitialization)
            );
            actual_plan(
                "async function g(view){for(let key in await view){await key;}}",
                |_, _, foreign, _| {
                    let mut changed = ForInInputs::from_plan(plan);
                    changed.initialization = foreign.storage.initialization().clone();
                    assert_eq!(
                        rebuild_with_inputs(sources[0], interner, plan.clone(), changed, inventory),
                        Err(GeneratorForInControlError::ForeignSourceHead)
                    );
                },
            );
        },
    );
}

#[test]
fn async_for_in_constructor_rejects_same_shaped_foreign_source_head() {
    actual_plan("async function g(view){for(let key in await view){await key;}for(let key in await view){await key;}}", |sources, interner, plan, inventory| {
        assert_eq!(rebuild(sources[1], interner, plan.clone(), inventory), Err(GeneratorForInControlError::ForeignSourceHead));
    });
}

#[test]
fn async_for_in_constructor_rejects_displaced_await_and_equal_extent_source_loss() {
    actual_plan(SOURCE, |sources, interner, plan, inventory| {
        let displaced = change_head(sources[0], plan, |statements| {
            let StatementIr::AsyncAwait { resume_state, .. } = statements
                .iter_mut()
                .find(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
                .unwrap()
            else {
                unreachable!()
            };
            *resume_state += 1;
        });
        assert!(
            displaced.is_err(),
            "the region constructor rejects a displaced Await before composition"
        );
        actual_plan(
            "async function g(view){for(let key in await view){with(view){key;}}}",
            |_, _, nested, _| {
                assert_eq!(
                    (plan.body.entry_state(), plan.body.end_state()),
                    (nested.body.entry_state(), nested.body.end_state())
                );
                let mut lost = plan.clone();
                lost.body = nested.body.clone();
                assert_eq!(
                    rebuild(sources[0], interner, lost, inventory),
                    Err(GeneratorForInControlError::UnconsumedSourceSuspension)
                );
            },
        );
    });
}

#[test]
fn async_for_in_without_await_retains_whole_iteration_phases_and_foreign_refusal() {
    actual_plan(
        "async function g(view){for(var key in view){key;}}",
        |sources, interner, plan, inventory| {
            assert_eq!(
                rebuild(sources[0], interner, plan.clone(), inventory).unwrap(),
                *plan
            );
            assert!(plan.suspensions().is_empty());
            let statement = StatementIr::AsyncGeneratorForIn(Box::new(plan.clone()));
            assert!(crate::generator_loop_control::sequence_end(
                std::slice::from_ref(&statement),
                plan.entry_state()
            )
            .is_err());
        },
    );
}

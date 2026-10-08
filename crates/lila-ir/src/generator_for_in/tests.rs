use super::protocol_test_support::*;
use super::*;
use crate::async_generator_source::AsyncGeneratorForInSource;
use crate::*;

const SOURCE: &str = "function* g(view){for(let key in yield 'head'){yield key;}}";

#[test]
fn for_in_constructor_consumes_actual_source_points_and_four_original_invocation_cells() {
    actual_plan(SOURCE, |sources, interner, plan, inventory| {
        assert_eq!(
            rebuild(sources[0], interner, plan.clone(), inventory).unwrap(),
            *plan
        );
        assert_eq!(plan.execution(), ResumableRegionProtocolIr::Generator);
        let statement = StatementIr::AsyncGeneratorForIn(Box::new(plan.clone()));
        assert_eq!(
            crate::generator_loop_control::sequence_end(
                std::slice::from_ref(&statement),
                plan.entry_state()
            ),
            Ok(plan.exit_state())
        );
        let mut points = Vec::new();
        crate::generator_loop_control::collect_suspensions(
            std::slice::from_ref(&statement),
            &mut points,
        );
        let states = AsyncGeneratorForInSource::for_execution(sources[0], plan.execution())
            .unwrap()
            .states(plan.entry_state())
            .unwrap();
        assert_eq!(
            points,
            states
                .suspensions()
                .iter()
                .map(|point| GeneratorSuspensionPointIr {
                    suspend_state: point.suspend_state,
                    resume_state: point.resume_state,
                })
                .collect::<Vec<_>>()
        );
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
fn for_in_constructor_rejects_unallocated_and_aliased_cursor_key_or_value_cells() {
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
            let mut foreign = inventory.to_vec();
            foreign.push(OwnedEnvBindingIr {
                name: "foreign.enumeration".into(),
                slot: binding.slot,
            });
            for inventory in [missing, duplicate, foreign] {
                assert_eq!(
                    rebuild(sources[0], interner, plan.clone(), &inventory),
                    Err(GeneratorForInControlError::UnallocatedBinding)
                );
            }
        }
        let mut altered = ForInInputs::from_plan(plan);
        altered.key_binding = altered.head_binding.clone();
        assert_eq!(
            rebuild_with_inputs(sources[0], interner, plan.clone(), altered, inventory),
            Err(GeneratorForInControlError::AliasedBindings)
        );
    });
}

#[test]
fn for_in_constructor_rejects_foreign_same_shaped_source_and_incorrect_body_range() {
    let source =
        format!("{SOURCE} function* h(view){{for(let key in yield 'head'){{yield key;}}}}");
    actual_plan(&source, |sources, interner, plan, inventory| {
        assert_eq!(
            rebuild(sources[1], interner, plan.clone(), inventory),
            Err(GeneratorForInControlError::ForeignSourceHead)
        );
        let mut altered = plan.clone();
        altered.body = plan.head.region().clone();
        assert_eq!(
            rebuild(sources[0], interner, altered, inventory),
            Err(GeneratorForInControlError::InvalidStates)
        );
    });
}

#[test]
fn for_in_constructor_requires_actual_head_publication_and_retained_string_key_consumption() {
    actual_plan(SOURCE, |sources, interner, plan, inventory| {
        let absent = change_head(sources[0], plan, |statements| {
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
        for changed in [absent, foreign] {
            assert_eq!(
                rebuild(sources[0], interner, changed, inventory),
                Err(GeneratorForInControlError::MissingHeadPublication)
            );
        }
        // A bare Block cannot supply the opaque initializer. Its retained key
        // is still checked against the real invocation cell at composition.
        let mut altered = ForInInputs::from_plan(plan);
        altered.exchange_key_and_value();
        assert_eq!(
            rebuild_with_inputs(sources[0], interner, plan.clone(), altered, inventory),
            Err(GeneratorForInControlError::InvalidInitialization)
        );
    });
}

#[test]
fn for_in_constructor_consumes_actual_var_and_pattern_binding_leaves() {
    for source in [
        "function* g(view){for(var key in yield 'head'){yield key;}}",
        "function* g(view){for(var [first,...rest] in yield 'head'){yield first;}}",
        "function* g(view){for(var {length:size} in yield 'head'){yield size;}}",
    ] {
        actual_plan(source, |sources, interner, plan, inventory| {
            assert_eq!(
                rebuild(sources[0], interner, plan.clone(), inventory).unwrap(),
                *plan
            );
            let mut altered = ForInInputs::from_plan(plan);
            altered.exchange_key_and_value();
            assert_eq!(
                rebuild_with_inputs(sources[0], interner, plan.clone(), altered, inventory),
                Err(GeneratorForInControlError::InvalidInitialization)
            );
        });
    }
}

#[test]
fn for_in_constructor_rejects_foreign_original_tdz_and_iteration_binding_domains() {
    let source = "function* g(view){for(let key in yield 'head'){yield ()=>key;}}";
    actual_plan(source, |sources, interner, plan, inventory| {
        let mut tdz = ForInInputs::from_plan(plan);
        tdz.lexical_environment
            .as_mut()
            .unwrap()
            .tdz_binding_names
            .push("foreign.tdz".into());
        let mut iteration = ForInInputs::from_plan(plan);
        iteration
            .lexical_environment
            .as_mut()
            .unwrap()
            .iteration_environment
            .as_mut()
            .expect("actual captured per-key binding")
            .bindings[0]
            .name = plan.head_binding().name.clone();
        let mut object = ForInInputs::from_plan(plan);
        object
            .lexical_environment
            .as_mut()
            .unwrap()
            .iteration_environment
            .as_mut()
            .unwrap()
            .eval_environment = Some(EvalEnvironmentRoleIr::WithObject { object_slot: 0 });
        for altered in [tdz, iteration, object] {
            assert_eq!(
                rebuild_with_inputs(sources[0], interner, plan.clone(), altered, inventory),
                Err(GeneratorForInControlError::ForeignLexicalEnvironment)
            );
        }
    });
}

#[test]
fn for_in_constructor_rejects_bare_key_reads_for_actual_nonbinding_head_destinations() {
    for source in [
        "function* g(view){let target;for(target in yield 'head'){yield target;}}",
        "function* g(view,target){for(target.slot in yield 'head'){yield target.slot;}}",
        "function* g(view,target){for([target.slot] in yield 'head'){yield target.slot;}}",
    ] {
        actual_plan(source, |sources, interner, plan, inventory| {
            assert!(!plan.initialization().statements.is_empty());
            let mut altered = ForInInputs::from_plan(plan);
            altered.exchange_key_and_value();
            assert_eq!(
                rebuild_with_inputs(sources[0], interner, plan.clone(), altered, inventory),
                Err(GeneratorForInControlError::InvalidInitialization)
            );
        });
    }
}

#[test]
fn for_in_real_ignored_named_function_reference_and_abrupt_const_put_keep_source_admission() {
    for source in [
        "var original=function* g(view){for(g in view){yield typeof g;}};",
        "function* g(view){const target=1;for(target in view){yield target;}}",
        "var original=function* g(view){with(view){for(g in {a:1}){yield typeof g;}}};",
    ] {
        actual_plan(source, |_, _, plan, _| {
            assert_eq!(plan.head_mode(), BindingMode::Var);
            assert!(!plan.initialization().statements.is_empty());
        });
    }
}

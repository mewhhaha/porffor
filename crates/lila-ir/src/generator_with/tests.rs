use super::*;
use crate::{EvalEnvironmentRoleIr, ExprIr, StatementIr};
use boa_ast::statement::With;
use boa_ast::visitor::{VisitWith, Visitor};
use std::ops::ControlFlow;

const SOURCE: &str = "function* g(scope){with(yield 'head'){yield 'first';yield 'second';}}";

fn with_plan(
    source: &str,
    consume: impl FnOnce(&With, &OrdinaryGeneratorWithIr, &[OwnedEnvBindingIr]),
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
            StatementIr::OrdinaryGeneratorWith(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .expect("actual source-produced complete With owner");
    parsed
        .as_script()
        .unwrap()
        .with_compiler_session(|script, _| {
            struct Find<'ast>(Option<&'ast With>);
            impl<'ast> Visitor<'ast> for Find<'ast> {
                type BreakTy = ();
                fn visit_statement(&mut self, source: &'ast boa_ast::Statement) -> ControlFlow<()> {
                    if let boa_ast::Statement::With(with) = source {
                        self.0 = Some(with);
                        return ControlFlow::Break(());
                    }
                    source.visit_with(self)
                }
            }
            let mut found = Find(None);
            let _ = script.visit_with(&mut found);
            consume(found.0.unwrap(), plan, &function.owned_env_bindings);
        });
}

fn rebuild(
    source: &With,
    plan: OrdinaryGeneratorWithIr,
    inventory: &[OwnedEnvBindingIr],
) -> Result<OrdinaryGeneratorWithIr, GeneratorWithControlError> {
    let object_binding = plan.object_binding().clone();
    let lexical_environment = plan.lexical_environment().clone();
    rebuild_with_environment(source, plan, object_binding, lexical_environment, inventory)
}

fn rebuild_with_environment(
    source: &With,
    plan: OrdinaryGeneratorWithIr,
    object_binding: OwnedEnvBindingIr,
    lexical_environment: LexicalEnvironmentIr,
    inventory: &[OwnedEnvBindingIr],
) -> Result<OrdinaryGeneratorWithIr, GeneratorWithControlError> {
    let states = crate::lowering_helpers::GeneratorWithSource::new(source)
        .unwrap()
        .states(plan.entry_state())
        .unwrap();
    let head_binding = plan.head_binding().clone();
    OrdinaryGeneratorWithIr::new(
        states,
        plan.head,
        head_binding,
        object_binding,
        lexical_environment,
        plan.body,
        inventory,
    )
}

fn change_head(
    plan: &OrdinaryGeneratorWithIr,
    change: impl FnOnce(&mut Vec<StatementIr>),
) -> OrdinaryGeneratorWithIr {
    let mut changed = plan.clone();
    let mut block = changed.head.region().clone().into_block();
    change(&mut block.statements);
    let range = crate::generator_loop_control::GeneratorLoopSourceRange {
        entry: plan.head.region().entry_state(),
        end: plan.head.region().end_state(),
    };
    changed.head = GeneratorLoopExpressionIr::new(
        GeneratorLoopRegionIr::new(block, range).unwrap(),
        plan.head.value().clone(),
    );
    changed
}

#[test]
fn with_constructor_rejects_matching_head_annotations_outside_to_object_codomain() {
    with_plan(SOURCE, |source, plan, inventory| {
        let false_info = crate::TypedExpr::undefined().value_info();
        let mut changed = change_head(plan, |statements| {
            let StatementIr::Lexical { init, .. } = statements.last_mut().unwrap() else {
                unreachable!()
            };
            *init = crate::TypedExpr::from_info(false_info.clone(), init.expr.clone());
        });
        let value = crate::TypedExpr::from_info(false_info, changed.head.value().expr.clone());
        changed.head = GeneratorLoopExpressionIr::new(changed.head.region().clone(), value);
        assert_eq!(
            rebuild(source, changed, inventory),
            Err(GeneratorWithControlError::ForeignHead)
        );
    });
}

#[test]
fn with_constructor_consumes_actual_source_points_and_two_separate_cell_domains() {
    with_plan(SOURCE, |source, plan, inventory| {
        assert_eq!(rebuild(source, plan.clone(), inventory).unwrap(), *plan);
        let statement = StatementIr::OrdinaryGeneratorWith(Box::new(plan.clone()));
        assert_eq!(
            crate::generator_loop_control::sequence_end(
                std::slice::from_ref(&statement),
                plan.entry_state()
            ),
            Ok(plan.exit_state())
        );
        let states = crate::lowering_helpers::GeneratorWithSource::new(source)
            .unwrap()
            .states(plan.entry_state())
            .unwrap();
        let mut points = Vec::new();
        collect_suspensions(std::slice::from_ref(&statement), &mut points);
        assert_eq!(points, states.suspensions());
        assert_ne!(plan.head_binding().name, plan.object_binding().name);
        assert_eq!(plan.object_binding().slot, 0);
        assert!(inventory.contains(plan.head_binding()));
    });
}

#[test]
fn with_constructor_rejects_missing_duplicated_and_aliased_head_allocations() {
    with_plan(SOURCE, |source, plan, inventory| {
        let mut missing = inventory.to_vec();
        missing.retain(|binding| binding != plan.head_binding());
        let mut duplicate = inventory.to_vec();
        duplicate.push(plan.head_binding().clone());
        let mut alias = inventory.to_vec();
        alias.push(OwnedEnvBindingIr {
            mutability: crate::EnvironmentBindingMutabilityIr::Mutable,
            name: "foreign.head".into(),
            slot: plan.head_binding().slot,
        });
        for inventory in [missing, duplicate, alias] {
            assert_eq!(
                rebuild(source, plan.clone(), &inventory),
                Err(GeneratorWithControlError::UnallocatedHead)
            );
        }
    });
}

#[test]
fn with_constructor_requires_completed_to_object_and_matching_retained_head_read() {
    with_plan(SOURCE, |source, plan, inventory| {
        let missing = change_head(plan, |statements| {
            statements.pop();
        });
        let raw = change_head(plan, |statements| {
            let StatementIr::Lexical { init, .. } = statements.last_mut().unwrap() else {
                unreachable!()
            };
            *init = crate::TypedExpr::from_info(
                plan.head.value().value_info(),
                ExprIr::Identifier(plan.head_binding().name.clone()),
            );
        });
        for changed in [missing, raw] {
            assert_eq!(
                rebuild(source, changed, inventory),
                Err(GeneratorWithControlError::ForeignHead)
            );
        }
        let mut foreign = plan.clone();
        foreign.head = GeneratorLoopExpressionIr::new(
            plan.head.region().clone(),
            crate::TypedExpr::from_info(
                plan.head.value().value_info(),
                ExprIr::Identifier("global.head".into()),
            ),
        );
        assert_eq!(
            rebuild(source, foreign, inventory),
            Err(GeneratorWithControlError::UnallocatedHead)
        );
    });
}

#[test]
fn with_constructor_rejects_foreign_object_records_and_duplicate_body_environment() {
    with_plan(SOURCE, |source, plan, inventory| {
        let mut shifted_object = plan.object_binding().clone();
        shifted_object.slot = 1;
        let mut shifted_environment = plan.lexical_environment().clone();
        shifted_environment.bindings[0].slot = 1;
        shifted_environment.eval_environment =
            Some(EvalEnvironmentRoleIr::WithObject { object_slot: 1 });
        let mut role = plan.lexical_environment().clone();
        role.eval_environment = None;
        for (object_binding, lexical_environment) in [
            (shifted_object, shifted_environment),
            (plan.object_binding().clone(), role),
        ] {
            assert_eq!(
                rebuild_with_environment(
                    source,
                    plan.clone(),
                    object_binding,
                    lexical_environment,
                    inventory
                ),
                Err(GeneratorWithControlError::ForeignObjectEnvironment)
            );
        }
        let mut duplicate = plan.clone();
        let mut block = duplicate.body.clone().into_block();
        block.lexical_environment = Some(plan.lexical_environment().clone());
        duplicate.body = GeneratorLoopRegionIr::new(
            block,
            crate::generator_loop_control::GeneratorLoopSourceRange {
                entry: plan.body.entry_state(),
                end: plan.body.end_state(),
            },
        )
        .unwrap();
        assert_eq!(
            rebuild(source, duplicate, inventory),
            Err(GeneratorWithControlError::InvalidStates)
        );
    });
}

#[test]
fn with_constructor_refuses_equal_extent_body_that_loses_actual_source_suspensions() {
    with_plan(SOURCE, |source, plan, inventory| {
        with_plan(
            "function* g(scope){with(yield 'head'){with(scope){value;}}}",
            |_, nested, _| {
                assert_eq!(
                    (plan.body.entry_state(), plan.body.end_state()),
                    (nested.body.entry_state(), nested.body.end_state())
                );
                let mut changed = plan.clone();
                changed.body = nested.body.clone();
                assert_eq!(
                    rebuild(source, changed, inventory),
                    Err(GeneratorWithControlError::UnconsumedSourceSuspension)
                );
            },
        );
    });
}

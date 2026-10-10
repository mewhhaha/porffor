use super::*;
use crate::{EvalEnvironmentRoleIr, ExprIr};
use boa_ast::statement::With;
use boa_ast::visitor::{VisitWith, Visitor};
use std::ops::ControlFlow;

const SOURCE: &str = "async function g(scope){with(await scope){await 'first';await 'second';}}";

fn with_plan(
    source: &str,
    consume: impl FnOnce(&With, &AsyncFunctionWithIr, &[OwnedEnvBindingIr]),
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
            StatementIr::AsyncFunctionWith(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .expect("actual source-produced complete Async With owner");
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
    plan: AsyncFunctionWithIr,
    inventory: &[OwnedEnvBindingIr],
) -> Result<AsyncFunctionWithIr, AsyncWithControlError> {
    let object_binding = plan.object_binding().clone();
    let lexical_environment = plan.lexical_environment().clone();
    rebuild_with_environment(source, plan, object_binding, lexical_environment, inventory)
}

fn rebuild_with_environment(
    source: &With,
    plan: AsyncFunctionWithIr,
    object_binding: OwnedEnvBindingIr,
    lexical_environment: LexicalEnvironmentIr,
    inventory: &[OwnedEnvBindingIr],
) -> Result<AsyncFunctionWithIr, AsyncWithControlError> {
    let states = crate::lowering_helpers::AsyncWithSource::new(source)
        .unwrap()
        .states(plan.entry_state())
        .unwrap();
    let head_binding = plan.head_binding().clone();
    AsyncFunctionWithIr::new(
        states,
        plan.head,
        plan.head_value,
        head_binding,
        object_binding,
        lexical_environment,
        plan.body,
        inventory,
    )
}

#[test]
fn async_with_constructor_consumes_actual_source_phases_and_separate_cell_domains() {
    with_plan(SOURCE, |source, plan, inventory| {
        assert_eq!(rebuild(source, plan.clone(), inventory).unwrap(), *plan);
        let statement = StatementIr::AsyncFunctionWith(Box::new(plan.clone()));
        assert_eq!(
            crate::async_switch::sequence_exit(
                std::slice::from_ref(&statement),
                plan.entry_state()
            ),
            Ok(plan.exit_state())
        );
        let states = crate::lowering_helpers::AsyncWithSource::new(source)
            .unwrap()
            .states(plan.entry_state())
            .unwrap();
        let mut points = Vec::new();
        collect_awaits(std::slice::from_ref(&statement), &mut points);
        assert_eq!(points, states.awaits());
        assert_eq!(points.len(), 3);
        assert_ne!(plan.head_binding().name, plan.object_binding().name);
        assert_eq!(plan.object_binding().slot, 0);
        assert!(inventory.contains(plan.head_binding()));
    });
}

#[test]
fn async_with_constructor_requires_unique_actual_head_inventory() {
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
                Err(AsyncWithControlError::UnallocatedHead)
            );
        }
    });
}

#[test]
fn async_with_constructor_requires_completed_to_object_and_exact_retained_read() {
    with_plan(SOURCE, |source, plan, inventory| {
        let mut missing = plan.clone();
        missing.head.statements.pop();
        let mut raw = plan.clone();
        let StatementIr::Lexical { init, .. } = raw.head.statements.last_mut().unwrap() else {
            unreachable!()
        };
        *init = TypedExpr::from_info(
            plan.head_value.value_info(),
            ExprIr::Identifier(plan.head_binding().name.clone()),
        );
        for changed in [missing, raw] {
            assert_eq!(
                rebuild(source, changed, inventory),
                Err(AsyncWithControlError::ForeignHead)
            );
        }
        let mut foreign = plan.clone();
        foreign.head_value = TypedExpr::from_info(
            plan.head_value.value_info(),
            ExprIr::Identifier("foreign.head".into()),
        );
        assert_eq!(
            rebuild(source, foreign, inventory),
            Err(AsyncWithControlError::UnallocatedHead)
        );
    });
}

#[test]
fn async_with_constructor_requires_original_object_record_and_single_body_owner() {
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
                Err(AsyncWithControlError::ForeignObjectEnvironment)
            );
        }
        let mut duplicate = plan.clone();
        duplicate.body.lexical_environment = Some(plan.lexical_environment().clone());
        assert_eq!(
            rebuild(source, duplicate, inventory),
            Err(AsyncWithControlError::InvalidStates)
        );
    });
}

#[test]
fn async_with_constructor_rejects_matching_head_annotations_outside_to_object_codomain() {
    with_plan(SOURCE, |source, plan, inventory| {
        let mut changed = plan.clone();
        let StatementIr::Lexical { init, .. } = changed.head.statements.last_mut().unwrap() else {
            unreachable!()
        };
        let false_info = TypedExpr::undefined().value_info();
        *init = TypedExpr::from_info(false_info.clone(), init.expr.clone());
        changed.head_value = TypedExpr::from_info(false_info, changed.head_value.expr.clone());
        assert_eq!(
            rebuild(source, changed, inventory),
            Err(AsyncWithControlError::ForeignHead)
        );
    });
}

#[test]
fn async_with_constructor_rejects_displaced_await_and_equal_extent_source_loss() {
    with_plan(SOURCE, |source, plan, inventory| {
        let mut displaced = plan.clone();
        let StatementIr::AsyncAwait { resume_state, .. } = displaced
            .head
            .statements
            .iter_mut()
            .find(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
            .unwrap()
        else {
            unreachable!()
        };
        *resume_state += 1;
        assert_eq!(
            rebuild(source, displaced, inventory),
            Err(AsyncWithControlError::InvalidStates)
        );
        with_plan(
            "async function g(scope){with(await scope){with(scope){value;}}}",
            |_, nested, _| {
                assert_eq!(
                    (plan.body_entry, plan.body_end),
                    (nested.body_entry, nested.body_end)
                );
                let mut lost = plan.clone();
                lost.body = nested.body.clone();
                assert_eq!(
                    rebuild(source, lost, inventory),
                    Err(AsyncWithControlError::UnconsumedSourceAwait)
                );
            },
        );
    });
}

#[test]
fn async_with_without_await_still_owns_environment_entry_and_exit() {
    with_plan(
        "async function g(scope){with(scope){value;}}",
        |source, plan, inventory| {
            assert_eq!(rebuild(source, plan.clone(), inventory).unwrap(), *plan);
            assert!(plan.awaits.is_empty());
            assert_eq!(plan.body_entry, plan.head_ready + 1);
            assert_eq!(plan.exit, plan.body_end + 1);
            assert!(plan.exit > plan.entry);
            let statement = StatementIr::AsyncFunctionWith(Box::new(plan.clone()));
            assert!(crate::ir::statement_contains_async_with(&statement));
            assert!(crate::generator_loop_control::sequence_end(
                std::slice::from_ref(&statement),
                plan.entry
            )
            .is_err());
        },
    );
}

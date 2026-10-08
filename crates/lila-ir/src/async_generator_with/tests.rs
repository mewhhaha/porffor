use super::*;
use crate::{BlockIr, ExprIr, GeneratorResumeModeIr, StatementIr, YieldForm};
use boa_ast::statement::With;
use boa_ast::visitor::{VisitWith, Visitor};
use std::ops::ControlFlow;

const SOURCE: &str = "async function* g(scope){with(await (yield 'head')){yield 'first';await scope;yield 'second';}}";

fn with_plan(consume: impl FnOnce(&With, &AsyncGeneratorWithIr, &[OwnedEnvBindingIr])) {
    let parsed = lila_front::parse(SOURCE, lila_front::ParseOptions::script()).unwrap();
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
            StatementIr::AsyncGeneratorWith(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .expect("actual mixed source-produced complete With");
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

fn states(source: &With, plan: &AsyncGeneratorWithIr) -> AsyncGeneratorWithSourceStates {
    crate::async_generator_source::AsyncGeneratorWithSource::new(source)
        .unwrap()
        .states(plan.entry_state())
        .unwrap()
}

fn rebuild(
    source: &With,
    plan: &AsyncGeneratorWithIr,
    inventory: &[OwnedEnvBindingIr],
) -> Result<AsyncGeneratorWithIr, AsyncGeneratorWithControlError> {
    AsyncGeneratorWithIr::new(
        states(source, plan),
        plan.head.clone(),
        plan.head_binding().clone(),
        plan.object_binding().clone(),
        plan.lexical_environment().clone(),
        plan.body.clone(),
        inventory,
    )
}

fn change_head(
    source: &With,
    plan: &AsyncGeneratorWithIr,
    change: impl FnOnce(&mut BlockIr),
) -> AsyncGeneratorWithIr {
    let mut changed = plan.clone();
    let mut block = plan.head.region().block().clone();
    change(&mut block);
    changed.head = AsyncGeneratorLoopExpressionIr::new(
        AsyncGeneratorLoopRegionIr::new(block, states(source, plan).head()).unwrap(),
        plan.head.value().clone(),
    );
    changed
}

#[test]
fn mixed_with_constructor_consumes_both_protocols_and_separate_cell_domains() {
    with_plan(|source, plan, inventory| {
        assert_eq!(rebuild(source, plan, inventory).unwrap(), *plan);
        let statement = StatementIr::AsyncGeneratorWith(Box::new(plan.clone()));
        let statements = std::slice::from_ref(&statement);
        assert_eq!(
            crate::generator_loop_control::mixed_sequence_end(statements, plan.entry_state()),
            Ok(plan.exit_state())
        );
        assert_eq!(
            crate::generator_loop_control::sequence_end(statements, plan.entry_state()),
            Err(crate::generator_loop_control::GeneratorLoopControlError::ForeignContinuation)
        );
        let mut actual = Vec::new();
        collect_mixed_suspensions(statements, &mut actual);
        assert_eq!(actual, states(source, plan).suspensions());
        assert_eq!(actual.len(), 5);
        assert!(actual
            .iter()
            .any(|point| point.kind == crate::ResumableSuspensionKindIr::Await));
        assert!(actual
            .iter()
            .any(|point| point.kind == crate::ResumableSuspensionKindIr::Yield));
        assert_ne!(plan.head_binding().name, plan.object_binding().name);
        assert_eq!(plan.object_binding().slot, 0);
        assert!(inventory.contains(plan.head_binding()));
        assert!(!inventory.contains(plan.object_binding()));
    });
}

#[test]
fn mixed_with_constructor_rejects_missing_duplicated_and_aliased_head_cells() {
    with_plan(|source, plan, inventory| {
        let mut missing = inventory.to_vec();
        missing.retain(|binding| binding != plan.head_binding());
        let mut duplicate = inventory.to_vec();
        duplicate.push(plan.head_binding().clone());
        let mut alias = inventory.to_vec();
        let mut row = plan.head_binding().clone();
        row.name = "foreign.alias".into();
        alias.push(row);
        for rows in [missing, duplicate, alias] {
            assert_eq!(
                rebuild(source, plan, &rows),
                Err(AsyncGeneratorWithControlError::ObjectEnvironment(
                    WithObjectEnvironmentError::UnallocatedHead
                ))
            );
        }
    });
}

#[test]
fn mixed_with_constructor_rederives_to_object_codomain_before_publication() {
    with_plan(|source, plan, inventory| {
        let false_info = TypedExpr::undefined().value_info();
        let mut changed = change_head(source, plan, |block| {
            let StatementIr::Lexical { init, .. } = block.statements.last_mut().unwrap() else {
                unreachable!()
            };
            *init = TypedExpr::from_info(false_info.clone(), init.expr.clone());
        });
        changed.head = AsyncGeneratorLoopExpressionIr::new(
            changed.head.region().clone(),
            TypedExpr::from_info(false_info, changed.head.value().expr.clone()),
        );
        let missing = change_head(source, plan, |block| {
            block.statements.pop();
        });
        let raw = change_head(source, plan, |block| {
            let StatementIr::Lexical { init, .. } = block.statements.last_mut().unwrap() else {
                unreachable!()
            };
            *init = TypedExpr::from_info(
                init.value_info(),
                ExprIr::Identifier(plan.head_binding().name.clone()),
            );
        });
        for invalid in [changed, missing, raw] {
            assert_eq!(
                rebuild(source, &invalid, inventory),
                Err(AsyncGeneratorWithControlError::ObjectEnvironment(
                    WithObjectEnvironmentError::ForeignHead
                ))
            );
        }
    });
}

#[test]
fn mixed_with_constructor_requires_one_original_object_record_and_body_owner() {
    with_plan(|source, plan, inventory| {
        let mut environment = plan.lexical_environment().clone();
        environment.eval_environment = None;
        assert_eq!(
            AsyncGeneratorWithIr::new(
                states(source, plan),
                plan.head.clone(),
                plan.head_binding().clone(),
                plan.object_binding().clone(),
                environment,
                plan.body.clone(),
                inventory
            ),
            Err(AsyncGeneratorWithControlError::ObjectEnvironment(
                WithObjectEnvironmentError::ForeignObjectEnvironment
            ))
        );
        let mut block = plan.body.block().clone();
        block.lexical_environment = Some(plan.lexical_environment().clone());
        let mut duplicate = plan.clone();
        duplicate.body =
            AsyncGeneratorLoopRegionIr::new(block, states(source, plan).body()).unwrap();
        assert_eq!(
            rebuild(source, &duplicate, inventory),
            Err(AsyncGeneratorWithControlError::InvalidStates)
        );
        let shifted = crate::async_generator_source::AsyncGeneratorWithSource::new(source)
            .unwrap()
            .states(plan.entry_state() + 1)
            .unwrap();
        assert_eq!(
            AsyncGeneratorWithIr::new(
                shifted,
                plan.head.clone(),
                plan.head_binding().clone(),
                plan.object_binding().clone(),
                plan.lexical_environment().clone(),
                plan.body.clone(),
                inventory
            ),
            Err(AsyncGeneratorWithControlError::InvalidStates)
        );
    });
}

#[test]
fn mixed_with_constructor_rejects_equal_extent_protocol_changes_and_unowned_branches() {
    with_plan(|source, plan, inventory| {
        let changed = change_head(source, plan, |block| {
            let statement = block
                .statements
                .iter_mut()
                .find(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
                .unwrap();
            let StatementIr::AsyncAwait {
                value,
                suspend_state,
                resume_state,
                ..
            } = statement.clone()
            else {
                unreachable!()
            };
            *statement = StatementIr::GeneratorYield {
                value,
                suspend_state,
                resume_state,
                form: YieldForm::Plain,
                resume_mode: GeneratorResumeModeIr::Ignore,
            };
        });
        assert_eq!(
            rebuild(source, &changed, inventory),
            Err(AsyncGeneratorWithControlError::SourceTape(
                AsyncGeneratorControlError::UnconsumedSourceSuspension
            ))
        );
        let branch = change_head(source, plan, |block| {
            block
                .statements
                .insert(0, StatementIr::Break { label: None });
        });
        assert_eq!(
            rebuild(source, &branch, inventory),
            Err(AsyncGeneratorWithControlError::ForeignContinuation)
        );
    });
}

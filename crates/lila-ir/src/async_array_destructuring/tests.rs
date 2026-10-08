use super::*;
use boa_ast::pattern::{ArrayPatternElement, Pattern};
use boa_ast::visitor::{VisitWith, Visitor};
use std::ops::ControlFlow;

const SOURCE: &str = "async function values(input, target, p, q) { var tail; return ([, target[await 'key'] = (await p, await q), ...tail] = input); }";

fn find_plan(statements: &[StatementIr]) -> Option<&AsyncFunctionArrayDestructuringIr> {
    statements.iter().find_map(|statement| match statement {
        StatementIr::AsyncFunctionArrayDestructuring(plan) => Some(plan.as_ref()),
        StatementIr::Block(block) => find_plan(&block.statements),
        StatementIr::LexicalBlock(statements) => find_plan(statements),
        StatementIr::EmptyStatementCompletion(item) => {
            find_plan(std::slice::from_ref(item.statement()))
        }
        _ => None,
    })
}

fn with_plan(
    source: &str,
    consume: impl FnOnce(&Pattern, &AsyncFunctionArrayDestructuringIr, &[OwnedEnvBindingIr]),
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
        .find(|function| function.name == "values")
        .unwrap();
    let plan = find_plan(&function.body.statements).expect("actual source-produced Async owner");
    parsed
        .as_script()
        .unwrap()
        .with_compiler_session(|script, _| {
            struct Find<'ast>(Option<&'ast Pattern>);
            impl<'ast> Visitor<'ast> for Find<'ast> {
                type BreakTy = ();
                fn visit_pattern(&mut self, source: &'ast Pattern) -> ControlFlow<()> {
                    if matches!(source, Pattern::Array(_)) {
                        self.0 = Some(source);
                        return ControlFlow::Break(());
                    }
                    source.visit_with(self)
                }
            }
            let mut found = Find(None);
            let _ = script.visit_with(&mut found);
            consume(
                found.0.expect("actual ArrayPattern AST"),
                plan,
                &function.owned_env_bindings,
            );
        });
}

fn rebuild(
    source: &Pattern,
    plan: AsyncFunctionArrayDestructuringIr,
    inventory: &[OwnedEnvBindingIr],
) -> Result<AsyncFunctionArrayDestructuringIr, AsyncArrayDestructuringError> {
    let states = crate::lowering_helpers::AsyncArrayPatternSource::new(source)
        .unwrap()
        .states(plan.entry_state())
        .unwrap();
    AsyncFunctionArrayDestructuringIr::new(
        states,
        plan.raw_source,
        plan.storage,
        plan.body,
        inventory,
    )
}

fn change_body(
    plan: &AsyncFunctionArrayDestructuringIr,
    change: impl FnOnce(&mut Vec<StatementIr>),
) -> AsyncFunctionArrayDestructuringIr {
    let mut result = plan.clone();
    change(&mut result.body.statements);
    result
}

#[test]
fn actual_async_array_owner_consumes_source_tape_without_exposing_bare_iterator_operations() {
    with_plan(SOURCE, |source, plan, inventory| {
        assert_eq!(rebuild(source, plan.clone(), inventory).unwrap(), *plan);
        assert!(plan.contains_await());
        let statement = StatementIr::AsyncFunctionArrayDestructuring(Box::new(plan.clone()));
        assert_eq!(
            crate::async_switch::sequence_exit(
                std::slice::from_ref(&statement),
                plan.entry_state()
            ),
            Ok(plan.exit_state())
        );
        assert!(crate::generator_loop_control::sequence_end(
            std::slice::from_ref(&statement),
            plan.entry_state()
        )
        .is_err());
        assert!(crate::SynchronousLoopBodyIr::new(&statement).is_err());
        let bare = crate::ArrayDestructuringOperationIr::elision(plan.storage());
        assert!(crate::async_switch::sequence_exit(
            std::slice::from_ref(&bare),
            plan.body_entry_state()
        )
        .is_err());
        assert!(crate::AsyncFunctionForOfBodyIr::new(vec![bare], plan.body_entry_state()).is_err());
        let body =
            crate::AsyncFunctionForOfBodyIr::new(vec![statement], plan.entry_state()).unwrap();
        assert_eq!(body.exit_state(), plan.exit_state());
    });
}

#[test]
fn async_array_constructor_rejects_removed_extra_and_substituted_protocol_operations() {
    with_plan(SOURCE, |source, plan, inventory| {
        let removed = change_body(plan, |body| {
            body.retain(|statement| !matches!(statement,
            StatementIr::ArrayDestructuringOperation(operation) if operation.kind() == ArrayDestructuringOperationKindIr::Elision))
        });
        let extra = change_body(plan, |body| {
            body.push(crate::ArrayDestructuringOperationIr::elision(
                plan.storage(),
            ))
        });
        let substituted = change_body(plan, |body| {
            let step = body.iter_mut().find(|statement| matches!(statement,
                StatementIr::ArrayDestructuringOperation(operation) if operation.kind() == ArrayDestructuringOperationKindIr::StepValue)).unwrap();
            let StatementIr::ArrayDestructuringOperation(operation) = step else {
                unreachable!()
            };
            let (mut replacement, _) = crate::ArrayDestructuringOperationIr::rest_array(
                plan.storage(),
                operation.result_binding().unwrap().clone(),
                inventory,
            )
            .unwrap();
            *step = replacement.pop().unwrap();
        });
        for changed in [removed, extra, substituted] {
            assert_eq!(
                rebuild(source, changed, inventory),
                Err(AsyncArrayDestructuringError::UnconsumedSourceOperation)
            );
        }
    });
}

#[test]
fn async_array_constructor_requires_exact_original_storage_and_complete_cell_inventory() {
    with_plan(SOURCE, |source, plan, inventory| {
        let foreign = ArrayIteratorStorageIr::new(plan.raw_binding.clone(), inventory).unwrap();
        let changed = change_body(plan, |body| {
            body.push(crate::ArrayDestructuringOperationIr::elision(&foreign))
        });
        assert_eq!(
            rebuild(source, changed, inventory),
            Err(AsyncArrayDestructuringError::ForeignIteratorOperation)
        );
        let result = plan
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::ArrayDestructuringOperation(operation) => operation.result_binding(),
                _ => None,
            })
            .unwrap();
        let mut missing = inventory.to_vec();
        missing.retain(|row| row != result);
        assert_eq!(
            rebuild(source, plan.clone(), &missing),
            Err(AsyncArrayDestructuringError::UnallocatedPatternCell)
        );
        let aliased = change_body(plan, |body| {
            let (mut replacement, _) = crate::ArrayDestructuringOperationIr::step_value(
                plan.storage(),
                plan.raw_binding.clone(),
                inventory,
            )
            .unwrap();
            body.push(replacement.pop().unwrap());
        });
        assert_eq!(
            rebuild(source, aliased, inventory),
            Err(AsyncArrayDestructuringError::AliasedPatternCell)
        );
        let mut ambiguous = inventory.to_vec();
        ambiguous.push(OwnedEnvBindingIr {
            name: "foreign source".into(),
            slot: plan.raw_binding.slot,
        });
        assert_eq!(
            rebuild(source, plan.clone(), &ambiguous),
            Err(AsyncArrayDestructuringError::UnallocatedPatternCell)
        );
    });
}

#[test]
fn async_array_constructor_rejects_missing_displaced_awaits_and_foreign_statement_owners() {
    with_plan(SOURCE, |source, plan, inventory| {
        let missing = change_body(plan, |body| {
            let position = body
                .iter()
                .position(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
                .unwrap();
            body.remove(position);
        });
        let displaced = change_body(plan, |body| {
            let StatementIr::AsyncAwait { resume_state, .. } = body
                .iter_mut()
                .find(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
                .unwrap()
            else {
                unreachable!()
            };
            *resume_state += 1;
        });
        for changed in [missing, displaced] {
            assert_eq!(
                rebuild(source, changed, inventory),
                Err(AsyncArrayDestructuringError::InvalidStates)
            );
        }
        let foreign = change_body(plan, |body| {
            body.push(StatementIr::Return(TypedExpr::undefined()))
        });
        assert_eq!(
            rebuild(source, foreign, inventory),
            Err(AsyncArrayDestructuringError::ForeignBody)
        );
    });
}

#[test]
fn nested_async_array_constructor_rejects_a_valid_child_that_aliases_the_parent_source() {
    with_plan(
        "async function values(input,p) { let [[received=await p]]=input; return received; }",
        |source, plan, inventory| {
            let Pattern::Array(pattern) = source else {
                unreachable!()
            };
            let ArrayPatternElement::Pattern {
                pattern: nested_source,
                ..
            } = &pattern.bindings()[0]
            else {
                unreachable!()
            };
            let position = plan
                .body
                .statements
                .iter()
                .position(|statement| {
                    matches!(statement, StatementIr::AsyncFunctionArrayDestructuring(_))
                })
                .unwrap();
            let StatementIr::AsyncFunctionArrayDestructuring(child) =
                &plan.body.statements[position]
            else {
                unreachable!()
            };
            let mut changed_child = child.as_ref().clone();
            changed_child.raw_source = plan.raw_source.clone();
            let valid_child = rebuild(nested_source, changed_child, inventory).unwrap();
            let changed = change_body(plan, |body| {
                body[position] = StatementIr::AsyncFunctionArrayDestructuring(Box::new(valid_child))
            });
            assert_eq!(
                rebuild(source, changed, inventory),
                Err(AsyncArrayDestructuringError::AliasedPatternCell)
            );
        },
    );
}

#[test]
fn eager_defaults_inside_an_async_pattern_cannot_hide_a_suspension_or_iterator_operation() {
    with_plan("async function values(input,p) { let [plain=function(){},waited=await p]=input; return waited; }", |source, plan, inventory| {
        assert_eq!(rebuild(source, plan.clone(), inventory).unwrap(), *plan);
        for hidden in [
            StatementIr::AsyncAwait { value: TypedExpr::undefined(), suspend_state: plan.body_entry_state(), resume_state: plan.body_entry_state()+1, resume_mode: crate::AsyncResumeModeIr::Ignore },
            crate::ArrayDestructuringOperationIr::elision(plan.storage()),
        ] {
            let changed = change_body(plan, |body| {
                let StatementIr::If { then_branch, .. } = body.iter_mut().find(|statement| matches!(statement, StatementIr::If { .. })).unwrap() else { unreachable!() };
                *then_branch = Box::new(hidden);
            });
            assert_eq!(rebuild(source, changed, inventory), Err(AsyncArrayDestructuringError::InvalidStates));
        }
    });
}

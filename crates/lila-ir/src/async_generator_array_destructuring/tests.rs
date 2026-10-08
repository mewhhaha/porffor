use super::*;
use crate::async_generator_source::AsyncGeneratorArrayPatternSource;
use crate::{ArrayDestructuringOperationIr, GeneratorResumeModeIr, YieldForm};
use boa_ast::pattern::{ArrayPatternElement, Pattern};
use boa_ast::visitor::{VisitWith, Visitor};
use std::ops::ControlFlow;

const SOURCE: &str = "async function* values(input,target){var tail;return ([,target[await(yield 'key')]=await(yield 'default'),...tail]=input);}";

fn find_plan(statements: &[StatementIr]) -> Option<&AsyncGeneratorArrayDestructuringIr> {
    statements.iter().find_map(|statement| match statement {
        StatementIr::AsyncGeneratorArrayDestructuring(plan) => Some(plan.as_ref()),
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
    check: impl FnOnce(&Pattern, &AsyncGeneratorArrayDestructuringIr, &[OwnedEnvBindingIr]),
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
    let plan = find_plan(&function.body.statements).expect("actual mixed pattern owner");
    parsed
        .as_script()
        .unwrap()
        .with_compiler_session(|script, _| {
            struct Find<'ast>(Option<&'ast Pattern>);
            impl<'ast> Visitor<'ast> for Find<'ast> {
                type BreakTy = ();
                fn visit_pattern(&mut self, pattern: &'ast Pattern) -> ControlFlow<()> {
                    if matches!(pattern, Pattern::Array(_)) {
                        self.0 = Some(pattern);
                        return ControlFlow::Break(());
                    }
                    pattern.visit_with(self)
                }
            }
            let mut found = Find(None);
            let _ = script.visit_with(&mut found);
            check(found.0.unwrap(), plan, &function.owned_env_bindings);
        });
}

fn rebuild(
    source: &Pattern,
    plan: &AsyncGeneratorArrayDestructuringIr,
    body: BlockIr,
    inventory: &[OwnedEnvBindingIr],
) -> Result<AsyncGeneratorArrayDestructuringIr, Error> {
    AsyncGeneratorArrayDestructuringIr::new(
        AsyncGeneratorArrayPatternSource::new(source)
            .unwrap()
            .states(plan.entry_state())
            .unwrap(),
        plan.raw_source().clone(),
        plan.storage().clone(),
        body,
        inventory,
    )
}

#[test]
fn mixed_array_owner_admits_both_protocols_but_bare_operations_have_no_region_authority() {
    with_plan(SOURCE, |source, plan, inventory| {
        assert_eq!(
            rebuild(source, plan, plan.body().block().clone(), inventory).unwrap(),
            *plan
        );
        for kind in [
            crate::ResumableSuspensionKindIr::Await,
            crate::ResumableSuspensionKindIr::Yield,
        ] {
            assert!(plan.suspensions().iter().any(|point| point.kind == kind));
        }
        let statement = StatementIr::AsyncGeneratorArrayDestructuring(Box::new(plan.clone()));
        assert_eq!(
            mixed_sequence_end(std::slice::from_ref(&statement), plan.entry_state()),
            Ok(plan.exit_state())
        );
        assert!(crate::generator_loop_control::sequence_end(
            std::slice::from_ref(&statement),
            plan.entry_state()
        )
        .is_err());
        assert!(crate::async_switch::sequence_exit(
            std::slice::from_ref(&statement),
            plan.entry_state()
        )
        .is_err());
        assert!(crate::SynchronousLoopBodyIr::new(&statement).is_err());
        let bare = ArrayDestructuringOperationIr::elision(plan.storage());
        assert!(mixed_sequence_end(&[bare], plan.body_entry_state()).is_err());
    });
}

#[test]
fn mixed_array_constructor_rejects_missing_extra_and_foreign_protocol_operations() {
    with_plan(SOURCE, |source, plan, inventory| {
        let mut missing = plan.body().block().clone();
        missing.statements.retain(|statement| !matches!(statement,
            StatementIr::ArrayDestructuringOperation(operation) if operation.kind() == ArrayDestructuringOperationKindIr::Elision));
        let mut extra = plan.body().block().clone();
        extra
            .statements
            .push(ArrayDestructuringOperationIr::elision(plan.storage()));
        for body in [missing, extra] {
            assert_eq!(
                rebuild(source, plan, body, inventory),
                Err(Error::UnconsumedSourceOperation)
            );
        }
        let foreign = ArrayIteratorStorageIr::new(plan.raw_binding.clone(), inventory).unwrap();
        let mut body = plan.body().block().clone();
        body.statements
            .push(ArrayDestructuringOperationIr::elision(&foreign));
        assert_eq!(
            rebuild(source, plan, body, inventory),
            Err(Error::ForeignIteratorOperation)
        );
    });
}

#[test]
fn mixed_array_constructor_checks_all_result_cells_and_the_kind_of_each_suspension() {
    with_plan(SOURCE, |source, plan, inventory| {
        let result = plan
            .body()
            .block()
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::ArrayDestructuringOperation(operation) => operation.result_binding(),
                _ => None,
            })
            .unwrap();
        let missing: Vec<_> = inventory
            .iter()
            .filter(|row| *row != result)
            .cloned()
            .collect();
        assert_eq!(
            rebuild(source, plan, plan.body().block().clone(), &missing),
            Err(Error::UnallocatedPatternCell)
        );
        let mut ambiguous = inventory.to_vec();
        ambiguous.push(OwnedEnvBindingIr {
            name: "foreign".into(),
            slot: plan.raw_binding.slot,
        });
        assert_eq!(
            rebuild(source, plan, plan.body().block().clone(), &ambiguous),
            Err(Error::UnallocatedPatternCell)
        );
        let mut wrong_kind = plan.body().block().clone();
        let awaited = wrong_kind
            .statements
            .iter_mut()
            .find(|statement| matches!(statement, StatementIr::AsyncAwait { .. }))
            .unwrap();
        let StatementIr::AsyncAwait {
            value,
            suspend_state,
            resume_state,
            ..
        } = awaited
        else {
            unreachable!()
        };
        *awaited = StatementIr::GeneratorYield {
            value: value.clone(),
            form: YieldForm::Plain,
            suspend_state: *suspend_state,
            resume_state: *resume_state,
            resume_mode: GeneratorResumeModeIr::Ignore,
        };
        assert_eq!(
            rebuild(source, plan, wrong_kind, inventory),
            Err(Error::UnconsumedSourceSuspension)
        );
        let mut abrupt = plan.body().block().clone();
        abrupt
            .statements
            .push(StatementIr::Return(TypedExpr::undefined()));
        assert_eq!(
            rebuild(source, plan, abrupt, inventory),
            Err(Error::ForeignBody)
        );
    });
}

#[test]
fn nested_mixed_array_owners_cannot_alias_a_parent_cell_even_when_each_owner_is_valid() {
    with_plan(
        "async function* values(input){let [[item=await(yield 'nested')]]=input;return item;}",
        |source, plan, inventory| {
            let Pattern::Array(array) = source else {
                unreachable!()
            };
            let ArrayPatternElement::Pattern {
                pattern: child_source,
                ..
            } = &array.bindings()[0]
            else {
                unreachable!()
            };
            let mut body = plan.body().block().clone();
            let statement = body
                .statements
                .iter_mut()
                .find(|statement| {
                    matches!(statement, StatementIr::AsyncGeneratorArrayDestructuring(_))
                })
                .unwrap();
            let StatementIr::AsyncGeneratorArrayDestructuring(child) = statement else {
                unreachable!()
            };
            let replacement = AsyncGeneratorArrayDestructuringIr::new(
                AsyncGeneratorArrayPatternSource::new(child_source)
                    .unwrap()
                    .states(child.entry_state())
                    .unwrap(),
                plan.raw_source().clone(),
                child.storage().clone(),
                child.body().block().clone(),
                inventory,
            )
            .unwrap();
            *statement = StatementIr::AsyncGeneratorArrayDestructuring(Box::new(replacement));
            assert_eq!(
                rebuild(source, plan, body, inventory),
                Err(Error::AliasedPatternCell)
            );
        },
    );
}

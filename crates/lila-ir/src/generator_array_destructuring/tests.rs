use super::*;
use boa_ast::pattern::Pattern;
use boa_ast::visitor::{VisitWith, Visitor};
use std::ops::ControlFlow;

const SOURCE: &str = "function* g(input, target) { var tail; return ([, target[yield 'key'] = yield 'default', ...tail] = input); }";

fn find_plan(statements: &[StatementIr]) -> Option<&OrdinaryGeneratorArrayDestructuringIr> {
    statements.iter().find_map(|statement| match statement {
        StatementIr::OrdinaryGeneratorArrayDestructuring(plan) => Some(plan.as_ref()),
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
    consume: impl FnOnce(&Pattern, &OrdinaryGeneratorArrayDestructuringIr, &[OwnedEnvBindingIr]),
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
    let plan = find_plan(&function.body.statements).expect("actual source-produced array owner");
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
                found.0.expect("actual complete ArrayPattern AST"),
                plan,
                &function.owned_env_bindings,
            );
        });
}

fn rebuild(
    source: &Pattern,
    plan: OrdinaryGeneratorArrayDestructuringIr,
    inventory: &[OwnedEnvBindingIr],
) -> Result<OrdinaryGeneratorArrayDestructuringIr, GeneratorArrayDestructuringError> {
    let states = crate::lowering_helpers::GeneratorArrayPatternSource::new(source)
        .unwrap()
        .states(plan.entry_state())
        .unwrap();
    OrdinaryGeneratorArrayDestructuringIr::new(
        states,
        plan.raw_source,
        plan.storage,
        plan.body,
        inventory,
    )
}

fn change_body(
    plan: &OrdinaryGeneratorArrayDestructuringIr,
    change: impl FnOnce(&mut Vec<StatementIr>),
) -> OrdinaryGeneratorArrayDestructuringIr {
    let mut result = plan.clone();
    let mut block = result.body.clone().into_block();
    change(&mut block.statements);
    result.body = GeneratorLoopRegionIr::new(
        block,
        crate::generator_loop_control::GeneratorLoopSourceRange {
            entry: plan.body.entry_state(),
            end: plan.body.end_state(),
        },
    )
    .unwrap();
    result
}

#[test]
fn complete_array_constructor_consumes_actual_source_tape_and_native_storage_inventory() {
    with_plan(SOURCE, |source, plan, inventory| {
        assert_eq!(rebuild(source, plan.clone(), inventory).unwrap(), *plan);
        let statement = StatementIr::OrdinaryGeneratorArrayDestructuring(Box::new(plan.clone()));
        assert_eq!(
            sequence_end(std::slice::from_ref(&statement), plan.entry_state()),
            Ok(plan.exit_state())
        );
        let states = crate::lowering_helpers::GeneratorArrayPatternSource::new(source)
            .unwrap()
            .states(plan.entry_state())
            .unwrap();
        let mut points = Vec::new();
        collect_suspensions(std::slice::from_ref(&statement), &mut points);
        assert_eq!(points, states.suspensions());
        assert_eq!(
            states
                .operations()
                .iter()
                .map(|(kind, _)| *kind)
                .collect::<Vec<_>>(),
            [
                ArrayDestructuringOperationKindIr::Elision,
                ArrayDestructuringOperationKindIr::StepValue,
                ArrayDestructuringOperationKindIr::RestArray
            ]
        );
        assert_ne!(plan.raw_binding.slot, plan.storage.binding().slot);
    });
}

#[test]
fn complete_array_constructor_rejects_removed_extra_and_substituted_protocol_operations() {
    with_plan(SOURCE, |source, plan, inventory| {
        let removed = change_body(plan, |body| {
            body.retain(|statement| !matches!(statement, StatementIr::ArrayDestructuringOperation(operation) if operation.kind() == ArrayDestructuringOperationKindIr::Elision));
        });
        let extra = change_body(plan, |body| {
            body.push(crate::ArrayDestructuringOperationIr::elision(&plan.storage))
        });
        let substituted = change_body(plan, |body| {
            let step = body.iter_mut().find(|statement| matches!(statement, StatementIr::ArrayDestructuringOperation(operation) if operation.kind() == ArrayDestructuringOperationKindIr::StepValue)).unwrap();
            let StatementIr::ArrayDestructuringOperation(operation) = step else {
                unreachable!()
            };
            let result = operation.result_binding().unwrap().clone();
            let (mut replacement, _) =
                crate::ArrayDestructuringOperationIr::rest_array(&plan.storage, result, inventory)
                    .unwrap();
            *step = replacement.pop().unwrap();
        });
        for changed in [removed, extra, substituted] {
            assert_eq!(
                rebuild(source, changed, inventory),
                Err(GeneratorArrayDestructuringError::UnconsumedSourceOperation)
            );
        }
    });
}

#[test]
fn complete_array_constructor_rejects_foreign_records_missing_outputs_and_global_cell_aliases() {
    with_plan(SOURCE, |source, plan, inventory| {
        let foreign =
            crate::ArrayIteratorStorageIr::new(plan.raw_binding.clone(), inventory).unwrap();
        let changed = change_body(plan, |body| {
            body.push(crate::ArrayDestructuringOperationIr::elision(&foreign))
        });
        assert_eq!(
            rebuild(source, changed, inventory),
            Err(GeneratorArrayDestructuringError::ForeignIteratorOperation)
        );
        let result = plan
            .body
            .block()
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
            Err(GeneratorArrayDestructuringError::UnallocatedPatternCell)
        );
        let aliased = change_body(plan, |body| {
            let (mut publication, _) = crate::ArrayDestructuringOperationIr::step_value(
                &plan.storage,
                plan.raw_binding.clone(),
                inventory,
            )
            .unwrap();
            body.push(publication.pop().unwrap());
        });
        assert_eq!(
            rebuild(source, aliased, inventory),
            Err(GeneratorArrayDestructuringError::AliasedPatternCell)
        );
        let mut raw_alias = inventory.to_vec();
        raw_alias.push(OwnedEnvBindingIr {
            mutability: crate::EnvironmentBindingMutabilityIr::Mutable,
            name: "foreign source".into(),
            slot: plan.raw_binding.slot,
        });
        assert_eq!(
            rebuild(source, plan.clone(), &raw_alias),
            Err(GeneratorArrayDestructuringError::UnallocatedSource)
        );
    });
}

#[test]
fn complete_array_constructor_keeps_suspended_class_prefixes_inside_the_close_region() {
    with_plan("function* g(input) { var value; return ([value = class extends (yield 'base') { [yield 'key']() {} }] = input); }", |source, plan, inventory| {
        assert_eq!(rebuild(source, plan.clone(), inventory).unwrap(), *plan);
        let mut points = Vec::new();
        collect_suspensions(&plan.body.block().statements, &mut points);
        assert_eq!(points.len(), 2);
    });
}

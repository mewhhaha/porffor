use super::*;
use crate::async_generator_source::AsyncGeneratorResourceScopeSource;
use crate::generator_loop_control::{mixed_sequence_end, sequence_end};
use boa_ast::visitor::{VisitWith, Visitor};
use boa_ast::{StatementList, StatementListItem};
use std::ops::ControlFlow;

const SOURCE: &str = "async function* resources(input){using first=input,second=input;await using third=input;yield await input;}";

fn with_plan(
    check: impl FnOnce(&[StatementListItem], &AsyncGeneratorResourceScopeIr, &[OwnedEnvBindingIr]),
) {
    with_protocol_plan(SOURCE, ResumableRegionProtocolIr::AsyncGenerator, check)
}
fn with_protocol_plan(
    source: &str,
    execution: ResumableRegionProtocolIr,
    check: impl FnOnce(&[StatementListItem], &AsyncGeneratorResourceScopeIr, &[OwnedEnvBindingIr]),
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
        .find(|function| function.name == "resources")
        .unwrap();
    let StatementIr::AsyncGeneratorResourceScope(plan) = &function.body.statements[0] else {
        panic!("one actual complete resource scope");
    };
    parsed
        .as_script()
        .unwrap()
        .with_compiler_session(|script, _| {
            struct Find<'ast>(Option<&'ast [StatementListItem]>, ResumableRegionProtocolIr);
            impl<'ast> Visitor<'ast> for Find<'ast> {
                type BreakTy = ();
                fn visit_function_body(
                    &mut self,
                    source: &'ast boa_ast::function::FunctionBody,
                ) -> ControlFlow<()> {
                    self.visit_statement_list(source.statement_list())
                }
                fn visit_statement_list(&mut self, source: &'ast StatementList) -> ControlFlow<()> {
                    if AsyncGeneratorResourceScopeSource::for_protocol(source.statements(), self.1)
                        .is_some()
                    {
                        self.0 = Some(source.statements());
                        ControlFlow::Break(())
                    } else {
                        source.visit_with(self)
                    }
                }
            }
            let mut found = Find(None, execution);
            let _ = script.visit_with(&mut found);
            check(found.0.unwrap(), plan, &function.owned_env_bindings);
        });
}

fn rebuild(
    source: &[StatementListItem],
    plan: &AsyncGeneratorResourceScopeIr,
    body: BlockIr,
    inventory: &[OwnedEnvBindingIr],
) -> Result<AsyncGeneratorResourceScopeIr, AsyncGeneratorResourceError> {
    AsyncGeneratorResourceScopeIr::new(
        AsyncGeneratorResourceScopeSource::for_protocol(source, plan.execution())
            .unwrap()
            .states(plan.entry_state())
            .unwrap(),
        plan.capability_binding.clone(),
        body,
        inventory,
    )
}

#[test]
fn ordinary_and_async_resource_proofs_reject_erased_suspensions_and_foreign_protocols() {
    for (source,execution) in [
        ("function* resources(input){using first=yield input,second=yield input;yield first;}",ResumableRegionProtocolIr::Generator),
        ("async function resources(input){await using first=await input;using second=await input;return first;}",ResumableRegionProtocolIr::Async),
    ] {
        with_protocol_plan(source,execution,|ast,plan,inventory| {
            assert_eq!(plan.execution(),execution);
            assert_eq!(rebuild(ast,plan,plan.body().block().clone(),inventory).unwrap(),*plan);
            let mut erased=flattened_body(plan);
            let index=erased.statements.iter().position(|statement|matches!(statement,
                StatementIr::GeneratorYield{..}|StatementIr::AsyncAwait{..})).unwrap();
            erased.statements.remove(index);
            assert!(rebuild(ast,plan,erased,inventory).is_err());
            let other=match execution {ResumableRegionProtocolIr::Generator=>ResumableRegionProtocolIr::Async,
                ResumableRegionProtocolIr::Async=>ResumableRegionProtocolIr::Generator,
                ResumableRegionProtocolIr::AsyncGenerator=>unreachable!()};
            assert!(AsyncGeneratorResourceScopeSource::for_protocol(ast,other).is_none());
            let statement=StatementIr::AsyncGeneratorResourceScope(Box::new(plan.clone()));
            match execution {
                ResumableRegionProtocolIr::Generator=>{
                    assert_eq!(sequence_end(std::slice::from_ref(&statement),plan.entry_state()),Ok(plan.exit_state()));
                    assert!(crate::async_switch::sequence_exit(std::slice::from_ref(&statement),plan.entry_state()).is_err());
                }
                ResumableRegionProtocolIr::Async=>{
                    assert_eq!(crate::async_switch::sequence_exit(std::slice::from_ref(&statement),plan.entry_state()),Ok(plan.exit_state()));
                    assert!(sequence_end(std::slice::from_ref(&statement),plan.entry_state()).is_err());
                }
                ResumableRegionProtocolIr::AsyncGenerator=>unreachable!(),
            }
            assert!(mixed_sequence_end(std::slice::from_ref(&statement),plan.entry_state()).is_err());
            for item in flattened_body(plan).statements {
                if let StatementIr::AsyncGeneratorResourceRegistration(_)=&item {
                    assert!(ResumableRegionIr::new(BlockIr {statements:vec![item.clone()],result_kind:crate::ValueKind::Undefined,lexical_environment:None},
                        AsyncGeneratorResourceScopeSource::for_protocol(ast,execution).unwrap().states(plan.entry_state()).unwrap().body(),execution).is_err());
                }
            }
        });
    }
}

fn flattened_body(plan: &AsyncGeneratorResourceScopeIr) -> BlockIr {
    fn append(statement: &StatementIr, output: &mut Vec<StatementIr>) {
        match statement {
            StatementIr::EmptyStatementCompletion(item) => append(item.statement(), output),
            StatementIr::LexicalBlock(items) => {
                for item in items {
                    append(item, output);
                }
            }
            StatementIr::Block(block) if block.lexical_environment.is_none() => {
                for item in &block.statements {
                    append(item, output);
                }
            }
            _ => output.push(statement.clone()),
        }
    }
    let mut body = plan.body().block().clone();
    body.statements.clear();
    for statement in &plan.body().block().statements {
        append(statement, &mut body.statements);
    }
    body
}

#[test]
fn resource_registration_authority_belongs_only_to_the_checked_complete_scope() {
    with_plan(|source, plan, inventory| {
        assert_eq!(
            rebuild(source, plan, plan.body().block().clone(), inventory).unwrap(),
            *plan
        );
        let scope = StatementIr::AsyncGeneratorResourceScope(Box::new(plan.clone()));
        assert_eq!(
            mixed_sequence_end(std::slice::from_ref(&scope), plan.entry_state()),
            Ok(plan.exit_state())
        );
        assert!(sequence_end(std::slice::from_ref(&scope), plan.entry_state()).is_err());
        assert!(crate::async_switch::sequence_exit(
            std::slice::from_ref(&scope),
            plan.entry_state()
        )
        .is_err());
        assert!(crate::SynchronousLoopBodyIr::new(&scope).is_err());
        for item in flattened_body(plan).statements {
            if let StatementIr::AsyncGeneratorResourceRegistration(operation) = &item {
                assert!(mixed_sequence_end(
                    std::slice::from_ref(&item),
                    operation.source().register_state()
                )
                .is_err());
            }
        }
    });
}

#[test]
fn resource_scope_rejects_missing_reordered_and_foreign_source_registrations() {
    with_plan(|source, plan, inventory| {
        let body = flattened_body(plan);
        let indices: Vec<_> = body
            .statements
            .iter()
            .enumerate()
            .filter_map(|(index, statement)| {
                matches!(
                    statement,
                    StatementIr::AsyncGeneratorResourceRegistration(_)
                )
                .then_some(index)
            })
            .collect();
        assert_eq!(indices.len(), 3);
        let mut missing = body.clone();
        missing.statements.remove(indices[0]);
        assert_eq!(
            rebuild(source, plan, missing, inventory),
            Err(AsyncGeneratorResourceError::UnconsumedSourceRegistration)
        );
        let mut reordered = body.clone();
        reordered.statements.swap(indices[0], indices[1]);
        assert_eq!(
            rebuild(source, plan, reordered, inventory),
            Err(AsyncGeneratorResourceError::UnconsumedSourceRegistration)
        );
        let mut duplicate = body.clone();
        duplicate
            .statements
            .insert(indices[0], body.statements[indices[0]].clone());
        assert_eq!(
            rebuild(source, plan, duplicate, inventory),
            Err(AsyncGeneratorResourceError::AliasedStorage)
        );
        with_plan(|foreign, _, _| {
            assert_eq!(
                rebuild(foreign, plan, body, inventory),
                Err(AsyncGeneratorResourceError::UnconsumedSourceRegistration)
            );
        });
    });
}

#[test]
fn resource_scope_requires_actual_disjoint_capability_and_initializer_cells_and_exact_suspend_kinds(
) {
    with_plan(|source, plan, inventory| {
        let body = flattened_body(plan);
        let missing_capability: Vec<_> = inventory
            .iter()
            .filter(|row| *row != plan.capability_binding())
            .cloned()
            .collect();
        assert_eq!(
            rebuild(source, plan, body.clone(), &missing_capability),
            Err(AsyncGeneratorResourceError::InvalidCapability)
        );
        let operation = body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::AsyncGeneratorResourceRegistration(operation) => Some(operation),
                _ => None,
            })
            .unwrap();
        let ExprIr::Identifier(name) = &operation.initializer().expr else {
            unreachable!()
        };
        let initializer = inventory.iter().find(|row| &row.name == name).unwrap();
        let missing_initializer: Vec<_> = inventory
            .iter()
            .filter(|row| *row != initializer)
            .cloned()
            .collect();
        assert_eq!(
            rebuild(source, plan, body.clone(), &missing_initializer),
            Err(AsyncGeneratorResourceError::UnallocatedInitializer)
        );
        let mut ambiguous = inventory.to_vec();
        ambiguous.push(OwnedEnvBindingIr {
            mutability: crate::EnvironmentBindingMutabilityIr::Mutable,
            name: "foreign".into(),
            slot: initializer.slot,
        });
        assert_eq!(
            rebuild(source, plan, body.clone(), &ambiguous),
            Err(AsyncGeneratorResourceError::AliasedStorage)
        );
        let mut wrong_kind = body;
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
            form: crate::YieldForm::Plain,
            suspend_state: *suspend_state,
            resume_state: *resume_state,
            resume_mode: crate::GeneratorResumeModeIr::Ignore,
        };
        assert_eq!(
            rebuild(source, plan, wrong_kind, inventory),
            Err(AsyncGeneratorResourceError::UnconsumedSourceSuspension)
        );
    });
}

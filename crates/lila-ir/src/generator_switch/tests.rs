use super::*;
use crate::LexicalEnvironmentInitializationIr;
use boa_ast::visitor::{VisitWith, Visitor};
use boa_ast::{statement::Switch as AstSwitch, Statement};
use std::ops::ControlFlow;

const SOURCE: &str = "function* g() { switch (yield 'discriminant') { case yield 'first': yield 'first body'; break; default: yield 'default body'; case yield 'last': yield 'last body'; } }";

fn with_plan(
    source: &str,
    consume: impl FnOnce(&AstSwitch, &OrdinaryGeneratorSwitchIr, &[OwnedEnvBindingIr]),
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
            StatementIr::OrdinaryGeneratorSwitch(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .expect("actual checked Switch producer");
    parsed
        .as_script()
        .unwrap()
        .with_compiler_session(|script, _| {
            struct Find<'ast>(Option<&'ast AstSwitch>);
            impl<'ast> Visitor<'ast> for Find<'ast> {
                type BreakTy = ();
                fn visit_statement(&mut self, source: &'ast Statement) -> ControlFlow<()> {
                    if let Statement::Switch(source) = source {
                        self.0 = Some(source);
                        return ControlFlow::Break(());
                    }
                    source.visit_with(self)
                }
            }
            let mut found = Find(None);
            let _ = script.visit_with(&mut found);
            consume(
                found.0.expect("actual Switch AST"),
                plan,
                &function.owned_env_bindings,
            );
        });
}

fn rebuild(
    source: &AstSwitch,
    plan: OrdinaryGeneratorSwitchIr,
    owned_bindings: &[OwnedEnvBindingIr],
) -> Result<OrdinaryGeneratorSwitchIr, GeneratorSwitchControlError> {
    let discriminant_binding = plan.discriminant_binding().clone();
    let value_binding = plan.value_binding().clone();
    let lexical_environment = plan.lexical_environment().cloned();
    rebuild_with_storage(
        source,
        plan,
        discriminant_binding,
        value_binding,
        lexical_environment,
        owned_bindings,
    )
}

fn rebuild_with_storage(
    source: &AstSwitch,
    plan: OrdinaryGeneratorSwitchIr,
    discriminant_binding: OwnedEnvBindingIr,
    value_binding: OwnedEnvBindingIr,
    lexical_environment: Option<LexicalEnvironmentIr>,
    owned_bindings: &[OwnedEnvBindingIr],
) -> Result<OrdinaryGeneratorSwitchIr, GeneratorSwitchControlError> {
    let (_, states) =
        crate::lowering_helpers::CheckedGeneratorSwitchSource::new(source, plan.entry_state())
            .unwrap()
            .into_parts();
    OrdinaryGeneratorSwitchIr::new(
        states,
        plan.discriminant,
        discriminant_binding,
        lexical_environment,
        plan.lexical_declarations,
        plan.cases,
        value_binding,
        owned_bindings,
    )
}

#[test]
fn switch_constructor_consumes_actual_default_middle_source_and_allocated_cells() {
    with_plan(SOURCE, |source, plan, owned| {
        let admitted = rebuild(source, plan.clone(), owned).unwrap();
        assert_eq!(&admitted, plan);
        assert!(owned.contains(admitted.discriminant_binding()));
        assert!(owned.contains(admitted.value_binding()));
        assert_ne!(
            admitted.discriminant_binding().slot,
            admitted.value_binding().slot
        );
        let [first, default, last] = admitted.cases() else {
            panic!("three cases")
        };
        assert!(first.selector().is_some());
        assert!(default.selector().is_none());
        assert_eq!(
            last.selector().unwrap().region().end_state() + 1,
            admitted.fallback_state()
        );
        assert_eq!(admitted.fallback_state() + 1, first.body().entry_state());
        assert_eq!(first.body().end_state() + 1, default.body().entry_state());
        assert_eq!(default.body().end_state() + 1, last.body().entry_state());
        assert_eq!(last.body().end_state() + 1, admitted.exit_state());
        let statement = StatementIr::OrdinaryGeneratorSwitch(Box::new(admitted));
        assert_eq!(
            sequence_end(std::slice::from_ref(&statement), plan.entry_state()),
            Ok(plan.exit_state())
        );
        let mut actual = Vec::new();
        collect_suspensions(std::slice::from_ref(&statement), &mut actual);
        let checked =
            crate::lowering_helpers::CheckedGeneratorSwitchSource::new(source, plan.entry_state())
                .unwrap();
        assert_eq!(actual, checked.states().suspensions());
    });
}

#[test]
fn switch_constructor_rejects_missing_duplicated_and_aliased_retained_allocations() {
    with_plan(SOURCE, |source, plan, owned| {
        let mut missing = owned.to_vec();
        missing.retain(|binding| binding != plan.value_binding());
        assert_eq!(
            rebuild(source, plan.clone(), &missing),
            Err(GeneratorSwitchControlError::UnallocatedBinding)
        );
        let mut duplicate = owned.to_vec();
        duplicate.push(plan.discriminant_binding().clone());
        assert_eq!(
            rebuild(source, plan.clone(), &duplicate),
            Err(GeneratorSwitchControlError::UnallocatedBinding)
        );
        for alias in [
            OwnedEnvBindingIr {
                mutability: crate::EnvironmentBindingMutabilityIr::Mutable,
                name: "foreign retained cell".into(),
                slot: plan.value_binding().slot,
            },
            OwnedEnvBindingIr {
                mutability: crate::EnvironmentBindingMutabilityIr::Mutable,
                name: plan.value_binding().name.clone(),
                slot: u32::MAX,
            },
        ] {
            let mut inventory = owned.to_vec();
            inventory.push(alias);
            assert_eq!(
                rebuild(source, plan.clone(), &inventory),
                Err(GeneratorSwitchControlError::UnallocatedBinding)
            );
        }
        assert_eq!(
            rebuild_with_storage(
                source,
                plan.clone(),
                plan.discriminant_binding().clone(),
                plan.discriminant_binding().clone(),
                plan.lexical_environment().cloned(),
                owned,
            ),
            Err(GeneratorSwitchControlError::AliasedBindings)
        );
        let mut foreign = plan.clone();
        foreign.discriminant = GeneratorLoopExpressionIr::new(
            foreign.discriminant.region().clone(),
            crate::TypedExpr::undefined(),
        );
        assert_eq!(
            rebuild(source, foreign, owned),
            Err(GeneratorSwitchControlError::MissingDiscriminantPublication)
        );
    });
}

#[test]
fn switch_constructor_rejects_reordered_selectors_bodies_and_default_destinations() {
    with_plan(SOURCE, |source, plan, owned| {
        let mut removed = plan.clone();
        removed.cases.pop();
        assert_eq!(
            rebuild(source, removed, owned),
            Err(GeneratorSwitchControlError::CaseCount)
        );
        let mut reordered = plan.clone();
        reordered.cases.swap(0, 2);
        assert_eq!(
            rebuild(source, reordered, owned),
            Err(GeneratorSwitchControlError::CaseOrder)
        );
        let mut duplicate_default = plan.clone();
        duplicate_default.cases[0].selector = None;
        assert_eq!(
            rebuild(source, duplicate_default, owned),
            Err(GeneratorSwitchControlError::DuplicateDefault)
        );
        let mut misplaced_default = plan.clone();
        misplaced_default.cases[1].selector = misplaced_default.cases[0].selector.take();
        assert_eq!(
            rebuild(source, misplaced_default, owned),
            Err(GeneratorSwitchControlError::CaseOrder)
        );
        let mut foreign_branch = plan.clone();
        let selector = foreign_branch.cases[0].selector.as_ref().unwrap().clone();
        let mut block = selector.region().block().clone();
        block.statements.push(StatementIr::Continue { label: None });
        let region = GeneratorLoopRegionIr::new(block, range_of(selector.region())).unwrap();
        foreign_branch.cases[0].selector = Some(GeneratorLoopExpressionIr::new(
            region,
            selector.value().clone(),
        ));
        assert_eq!(
            rebuild(source, foreign_branch, owned),
            Err(GeneratorSwitchControlError::UnownedHeadBranch)
        );
    });
}

#[test]
fn switch_constructor_requires_one_uninitialized_caseblock_environment() {
    with_plan(SOURCE, |source, plan, owned| {
        let environment = LexicalEnvironmentIr {
            initialization: LexicalEnvironmentInitializationIr::Uninitialized,
            eval_environment: None,
            bindings: vec![OwnedEnvBindingIr {
                mutability: crate::EnvironmentBindingMutabilityIr::Mutable,
                name: "caseblock local".into(),
                slot: plan.value_binding().slot,
            }],
        };
        assert!(
            rebuild_with_storage(
                source,
                plan.clone(),
                plan.discriminant_binding().clone(),
                plan.value_binding().clone(),
                Some(environment.clone()),
                owned,
            )
            .is_ok(),
            "slots belong to separate records"
        );
        let initialized = LexicalEnvironmentIr {
            initialization: LexicalEnvironmentInitializationIr::FunctionBody {
                bindings: Vec::new(),
            },
            ..environment.clone()
        };
        assert_eq!(
            rebuild_with_storage(
                source,
                plan.clone(),
                plan.discriminant_binding().clone(),
                plan.value_binding().clone(),
                Some(initialized),
                owned,
            ),
            Err(GeneratorSwitchControlError::InvalidCaseBlockEnvironment)
        );
        let shadows_cell = LexicalEnvironmentIr {
            bindings: vec![plan.discriminant_binding().clone()],
            ..environment.clone()
        };
        assert_eq!(
            rebuild_with_storage(
                source,
                plan.clone(),
                plan.discriminant_binding().clone(),
                plan.value_binding().clone(),
                Some(shadows_cell),
                owned,
            ),
            Err(GeneratorSwitchControlError::InvalidCaseBlockEnvironment)
        );
        let mut case_body = plan.cases[0].body().block().clone();
        case_body.lexical_environment = Some(environment);
        let region = GeneratorLoopRegionIr::new(case_body, range_of(plan.cases[0].body())).unwrap();
        assert_eq!(
            OrdinaryGeneratorSwitchCaseIr::new(plan.cases[0].selector().cloned(), region),
            Err(GeneratorSwitchControlError::PerCaseEnvironment)
        );
        let mut declarations = plan.clone();
        declarations
            .lexical_declarations
            .push(StatementIr::GeneratorYield {
                value: crate::TypedExpr::undefined(),
                form: crate::YieldForm::Plain,
                suspend_state: plan.case_block_entry_state(),
                resume_state: plan.case_block_entry_state() + 1,
                resume_mode: crate::GeneratorResumeModeIr::Ignore,
            });
        assert_eq!(
            rebuild(source, declarations, owned),
            Err(GeneratorSwitchControlError::CaseOrder)
        );
    });
}

#[test]
fn switch_head_branch_proof_owns_unlabelled_break_but_exposes_continue_and_foreign_labels() {
    let switch_with = |statement| StatementIr::Switch {
        discriminant: crate::TypedExpr::undefined(),
        lexical_environment: None,
        lexical_declarations: Vec::new(),
        cases: vec![crate::SwitchCaseIr {
            condition: None,
            body: crate::BlockIr {
                statements: vec![statement],
                result_kind: crate::ValueKind::Undefined,
                lexical_environment: None,
            },
        }],
    };
    assert!(!has_unowned_head_branch(&switch_with(StatementIr::Break {
        label: None
    })));
    assert!(has_unowned_head_branch(&switch_with(
        StatementIr::Continue { label: None }
    )));
    let eager_switch_with_yield = switch_with(StatementIr::GeneratorYield {
        value: crate::TypedExpr::undefined(),
        form: crate::YieldForm::Plain,
        suspend_state: 7,
        resume_state: 8,
        resume_mode: crate::GeneratorResumeModeIr::Ignore,
    });
    let mut points = Vec::new();
    collect_suspensions(std::slice::from_ref(&eager_switch_with_yield), &mut points);
    assert_eq!(
        points,
        [crate::GeneratorSuspensionPointIr {
            suspend_state: 7,
            resume_state: 8
        }]
    );
    assert!(
        sequence_end(std::slice::from_ref(&eager_switch_with_yield), 7).is_err(),
        "eager Switch cannot hide an actual suspension"
    );
    assert!(has_unowned_head_branch(&switch_with(StatementIr::Break {
        label: Some("outer".into())
    })));
    with_plan(SOURCE, |_, plan, _| {
        assert!(!has_unowned_head_branch(
            &StatementIr::OrdinaryGeneratorSwitch(Box::new(plan.clone()))
        ));
    });
}

#[test]
fn nested_actual_empty_declarations_keep_their_whole_suspending_payload() {
    fn collect_empty<'ir>(
        statement: &'ir StatementIr,
        items: &mut Vec<&'ir EmptyStatementCompletionIr>,
    ) {
        match statement {
            StatementIr::EmptyStatementCompletion(item) => {
                items.push(item);
                collect_empty(item.statement(), items);
            }
            StatementIr::Block(block) => {
                for statement in &block.statements {
                    collect_empty(statement, items);
                }
            }
            StatementIr::LexicalBlock(statements) => {
                for statement in statements {
                    collect_empty(statement, items);
                }
            }
            StatementIr::If {
                then_branch,
                else_branch,
                ..
            } => {
                collect_empty(then_branch, items);
                if let Some(statement) = else_branch {
                    collect_empty(statement, items);
                }
            }
            StatementIr::OrdinaryGeneratorIf(plan) => {
                for statement in plan
                    .then_branch()
                    .block()
                    .statements
                    .iter()
                    .chain(&plan.else_branch().block().statements)
                {
                    collect_empty(statement, items);
                }
            }
            _ => {}
        }
    }
    with_plan("function* g() { switch (0) { case 0: 7; if (true) { var v = yield 'var'; let w = yield 'let'; } break; } }", |source, plan, owned| {
        let admitted = rebuild(source, plan.clone(), owned).unwrap();
        let mut items = Vec::new();
        for statement in &admitted.cases()[0].body().block().statements {
            collect_empty(statement, &mut items);
        }
        let suspended_items = items.iter().filter(|item| {
            let mut points = Vec::new();
            collect_suspensions(std::slice::from_ref(item.statement()), &mut points);
            !points.is_empty()
        }).collect::<Vec<_>>();
        assert_eq!(suspended_items.len(), 2, "actual nested var and let sources carry Empty completion");
        for item in suspended_items {
            let mut points = Vec::new();
            collect_suspensions(std::slice::from_ref(item.statement()), &mut points);
            assert_eq!(points.len(), 1);
            assert_eq!(points[0].resume_state, points[0].suspend_state + 1);
            assert_eq!(sequence_end(std::slice::from_ref(item.statement()), points[0].suspend_state), Ok(points[0].resume_state));
            let wrapper = StatementIr::EmptyStatementCompletion(Box::new((**item).clone()));
            let mut wrapped_points = Vec::new();
            collect_suspensions(std::slice::from_ref(&wrapper), &mut wrapped_points);
            assert_eq!(wrapped_points, points, "the original payload is neither replaced nor copied into another graph");
        }
        let checked = crate::lowering_helpers::CheckedGeneratorSwitchSource::new(source, admitted.entry_state()).unwrap();
        let mut points = Vec::new();
        for region in admitted.regions() { collect_suspensions(&region.block().statements, &mut points); }
        assert_eq!(points, checked.states().suspensions());
    });
}

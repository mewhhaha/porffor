use super::*;
use crate::async_generator_source::AsyncGeneratorForInSource;
use crate::generator_loop_control::{
    collect_mixed_suspensions, mixed_sequence_end, sequence_end, GeneratorLoopControlError,
};
use crate::{
    ExprIr, GeneratorResumeModeIr, StatementIr, TypedExpr, ValueInfo, ValueKind, YieldForm,
};
use boa_ast::statement::iteration::ForInLoop;
use boa_ast::visitor::{VisitWith, Visitor};
use boa_interner::Interner;
use std::ops::ControlFlow;

const SOURCE: &str = "async function* g(input){for(let key in await(yield 'head')){await(yield key);yield function read(){return key;};}}";
const CAPTURED_HEAD: &str = "async function* g(input){for(let key in (yield function head(){return key;},await(yield 'head'))){await(yield key);yield function read(){return key;};}}";

fn actual_plan(
    source: &str,
    check: impl FnOnce(&[&ForInLoop], &Interner, &AsyncGeneratorForInIr, &[OwnedEnvBindingIr]),
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
            StatementIr::AsyncGeneratorForIn(plan) => Some(plan.as_ref()),
            _ => None,
        })
        .expect("actual source-produced complete mixed enumeration");
    parsed
        .as_script()
        .unwrap()
        .with_compiler_session(|script, interner| {
            #[derive(Default)]
            struct Sources<'ast>(Vec<&'ast ForInLoop>);
            impl<'ast> Visitor<'ast> for Sources<'ast> {
                type BreakTy = ();
                fn visit_for_in_loop(&mut self, source: &'ast ForInLoop) -> ControlFlow<()> {
                    self.0.push(source);
                    source.visit_with(self)
                }
            }
            let mut sources = Sources::default();
            let _ = script.visit_with(&mut sources);
            check(&sources.0, interner, plan, &function.owned_env_bindings);
        });
}

#[derive(Clone)]
struct Inputs {
    head_binding: OwnedEnvBindingIr,
    initialization: CheckedAsyncGeneratorForInInitializer,
    environment: Option<ForInOfEnvironmentIr>,
    enumerator: OwnedEnvBindingIr,
    key: OwnedEnvBindingIr,
    value: OwnedEnvBindingIr,
}
impl Inputs {
    fn from_plan(plan: &AsyncGeneratorForInIr) -> Self {
        Self {
            head_binding: plan.head_binding().clone(),
            initialization: plan.storage.initialization().clone(),
            environment: plan.lexical_environment().cloned(),
            enumerator: plan.enumerator_binding().clone(),
            key: plan.key_binding().clone(),
            value: plan.value_binding().clone(),
        }
    }
}

fn rebuild(
    source: &ForInLoop,
    interner: &Interner,
    plan: AsyncGeneratorForInIr,
    input: Inputs,
    inventory: &[OwnedEnvBindingIr],
) -> Result<AsyncGeneratorForInIr, GeneratorForInControlError> {
    let checked = AsyncGeneratorForInSource::for_execution(source, plan.execution()).unwrap();
    AsyncGeneratorForInIr::new(
        checked.states(plan.entry_state()).unwrap(),
        checked.checked_head(interner).unwrap(),
        plan.head,
        input.head_binding,
        input.initialization,
        input.environment,
        input.enumerator,
        input.key,
        input.value,
        plan.body,
        inventory,
    )
}

fn change_head(
    source: &ForInLoop,
    plan: &AsyncGeneratorForInIr,
    change: impl FnOnce(&mut BlockIr),
) -> AsyncGeneratorForInIr {
    let mut changed = plan.clone();
    let mut head = plan.head().region().block().clone();
    change(&mut head);
    let states = AsyncGeneratorForInSource::for_execution(source, plan.execution())
        .unwrap()
        .states(plan.entry_state())
        .unwrap();
    changed.head = ResumableExpressionIr::new(
        ResumableRegionIr::new(head, states.head(), plan.execution()).unwrap(),
        plan.head().value().clone(),
    );
    changed
}

#[test]
fn complete_for_in_keeps_original_protocol_and_rejects_foreign_execution_tape() {
    for (source, execution) in [
        (
            "function* g(input,target){for(target[yield 'key'] in yield 'head'){yield target;}}",
            ResumableRegionProtocolIr::Generator,
        ),
        (
            "async function g(input,p){for(const [a,b,c=await p] in await input){await c;}}",
            ResumableRegionProtocolIr::Async,
        ),
    ] {
        actual_plan(source, |sources, interner, plan, inventory| {
            assert_eq!(plan.execution(), execution);
            assert_eq!(
                rebuild(
                    sources[0],
                    interner,
                    plan.clone(),
                    Inputs::from_plan(plan),
                    inventory
                )
                .unwrap(),
                *plan
            );
            assert!(
                plan.initialization_region().end_state()
                    > plan.initialization_region().entry_state()
            );
            let foreign = AsyncGeneratorForInSource::new(sources[0]).unwrap();
            assert_eq!(
                AsyncGeneratorForInIr::new(
                    foreign.states(plan.entry_state()).unwrap(),
                    foreign.checked_head(interner).unwrap(),
                    plan.head.clone(),
                    plan.head_binding().clone(),
                    plan.storage.initialization().clone(),
                    plan.lexical_environment().cloned(),
                    plan.enumerator_binding().clone(),
                    plan.key_binding().clone(),
                    plan.value_binding().clone(),
                    plan.body.clone(),
                    inventory
                ),
                Err(GeneratorForInControlError::InvalidStates)
            );
        });
    }
}

#[test]
fn mixed_for_in_constructor_consumes_actual_phases_tape_and_four_original_cells() {
    actual_plan(SOURCE, |sources, interner, plan, inventory| {
        assert_eq!(
            rebuild(
                sources[0],
                interner,
                plan.clone(),
                Inputs::from_plan(plan),
                inventory
            )
            .unwrap(),
            *plan
        );
        let rows = [
            plan.head_binding(),
            plan.enumerator_binding(),
            plan.key_binding(),
            plan.value_binding(),
        ];
        assert!(rows.iter().all(|row| inventory.contains(row)));
        let slots = rows
            .iter()
            .map(|row| row.slot)
            .collect::<std::collections::BTreeSet<_>>();
        assert_eq!(slots.len(), 4);
        assert_eq!(plan.head().region().end_state() + 1, plan.advance_state());
        assert_eq!(
            plan.advance_state() + 1,
            plan.initialization_region().entry_state()
        );
        assert_eq!(
            plan.initialization_region().end_state() + 1,
            plan.body().entry_state()
        );
        assert_eq!(plan.body().end_state() + 1, plan.exit_state());
        assert_eq!(plan.continue_state(), plan.advance_state());
        assert!(plan
            .suspensions()
            .iter()
            .any(|point| point.kind == crate::ResumableSuspensionKindIr::Await));
        assert!(plan
            .suspensions()
            .iter()
            .any(|point| point.kind == crate::ResumableSuspensionKindIr::Yield));
        let statement = StatementIr::AsyncGeneratorForIn(Box::new(plan.clone()));
        let statements = std::slice::from_ref(&statement);
        assert_eq!(
            mixed_sequence_end(statements, plan.entry_state()),
            Ok(plan.exit_state())
        );
        assert_eq!(
            sequence_end(statements, plan.entry_state()),
            Err(GeneratorLoopControlError::ForeignContinuation)
        );
        let mut points = Vec::new();
        collect_mixed_suspensions(statements, &mut points);
        assert_eq!(points, plan.suspensions());
    });
}

#[test]
fn mixed_for_in_constructor_requires_unique_allocated_head_cursor_key_and_value() {
    actual_plan(SOURCE, |sources, interner, plan, inventory| {
        for cell in [
            plan.head_binding(),
            plan.enumerator_binding(),
            plan.key_binding(),
            plan.value_binding(),
        ] {
            let mut missing = inventory.to_vec();
            missing.retain(|row| row != cell);
            let mut duplicate = inventory.to_vec();
            duplicate.push(cell.clone());
            let mut alias = inventory.to_vec();
            let mut row = cell.clone();
            row.name = "foreign.alias".into();
            alias.push(row);
            for damaged in [missing, duplicate, alias] {
                assert_eq!(
                    rebuild(
                        sources[0],
                        interner,
                        plan.clone(),
                        Inputs::from_plan(plan),
                        &damaged
                    ),
                    Err(GeneratorForInControlError::UnallocatedBinding)
                );
            }
        }
        let mut aliased = Inputs::from_plan(plan);
        aliased.key = aliased.enumerator.clone();
        assert_eq!(
            rebuild(sources[0], interner, plan.clone(), aliased, inventory),
            Err(GeneratorForInControlError::AliasedBindings)
        );
    });
}

#[test]
fn mixed_for_in_constructor_requires_one_completed_head_publication() {
    actual_plan(SOURCE, |sources, interner, plan, inventory| {
        let missing = change_head(sources[0], plan, |head| {
            head.statements.pop();
        });
        let duplicate = change_head(sources[0], plan, |head| {
            head.statements
                .push(head.statements.last().unwrap().clone());
        });
        let mut foreign = plan.clone();
        foreign.head = ResumableExpressionIr::new(
            plan.head().region().clone(),
            TypedExpr::from_info(
                plan.head().value().value_info(),
                ExprIr::Identifier("foreign.head".into()),
            ),
        );
        for changed in [missing, duplicate, foreign] {
            assert_eq!(
                rebuild(
                    sources[0],
                    interner,
                    changed,
                    Inputs::from_plan(plan),
                    inventory
                ),
                Err(GeneratorForInControlError::MissingHeadPublication)
            );
        }
    });
}

#[test]
fn mixed_for_in_constructor_consumes_the_actual_eager_key_initializer() {
    for source in [SOURCE,
        "async function* g(input){for(var [first,...rest] in await(yield 'head')){await 0;yield first;}}",
        "async function* g(input){for(var {length:size} in await(yield 'head')){await 0;yield size;}}"] {
        actual_plan(source, |sources, interner, plan, inventory| {
            assert_eq!(rebuild(sources[0], interner, plan.clone(), Inputs::from_plan(plan), inventory).unwrap(), *plan);
            // A raw empty/read-only Block no longer has the required token
            // type. Even another fully allocated key cannot replace the
            // actual source producer's incoming key authority.
            let mut replaced = Inputs::from_plan(plan);
            replaced.key.name = "foreign.retained.key".into();
            let mut inventory = inventory.to_vec();
            *inventory.iter_mut().find(|row| row.name == plan.key_binding().name).unwrap() = replaced.key.clone();
            assert_eq!(rebuild(sources[0], interner, plan.clone(), replaced, &inventory),
                Err(GeneratorForInControlError::InvalidInitialization));
        });
    }
}

#[test]
fn mixed_for_in_constructor_rejects_same_shaped_foreign_head_and_displaced_body() {
    actual_plan("async function* g(input){for(let key in await(yield 'head')){await(yield key);yield function read(){return key;};}for(let key in await(yield 'head')){await(yield key);yield function read(){return key;};}}",
        |sources, interner, plan, inventory| {
            assert_eq!(sources.len(), 2);
            let checked = AsyncGeneratorForInSource::new(sources[0]).unwrap();
            let foreign = AsyncGeneratorForInSource::new(sources[1]).unwrap();
            assert_eq!(AsyncGeneratorForInIr::new(checked.states(plan.entry_state()).unwrap(), foreign.checked_head(interner).unwrap(),
                plan.head.clone(), plan.head_binding().clone(), plan.storage.initialization().clone(), plan.lexical_environment().cloned(),
                plan.enumerator_binding().clone(), plan.key_binding().clone(), plan.value_binding().clone(), plan.body.clone(), inventory),
                Err(GeneratorForInControlError::ForeignSourceHead));
            let mut changed = plan.clone(); changed.body = plan.head().region().clone();
            assert_eq!(rebuild(sources[0], interner, changed, Inputs::from_plan(plan), inventory), Err(GeneratorForInControlError::InvalidStates));
        });
}

#[test]
fn mixed_for_in_constructor_rejects_equal_extent_kind_changes_and_head_branches() {
    actual_plan(SOURCE, |sources, interner, plan, inventory| {
        let changed = change_head(sources[0], plan, |head| {
            let statement = head
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
            rebuild(
                sources[0],
                interner,
                changed,
                Inputs::from_plan(plan),
                inventory
            ),
            Err(GeneratorForInControlError::UnconsumedSourceSuspension)
        );
        let branch = change_head(sources[0], plan, |head| {
            head.statements
                .insert(0, StatementIr::Break { label: None });
        });
        assert_eq!(
            rebuild(
                sources[0],
                interner,
                branch,
                Inputs::from_plan(plan),
                inventory
            ),
            Err(GeneratorForInControlError::InvalidStates)
        );
    });
}

#[test]
fn mixed_for_in_constructor_retains_original_tdz_and_iteration_cell_domains() {
    actual_plan(CAPTURED_HEAD, |sources, interner, plan, inventory| {
        let environment = plan
            .lexical_environment()
            .expect("actual lexical head environment");
        assert!(
            environment.tdz_environment.is_some() && environment.iteration_environment.is_some()
        );
        assert_eq!(
            rebuild(
                sources[0],
                interner,
                plan.clone(),
                Inputs::from_plan(plan),
                inventory
            )
            .unwrap(),
            *plan
        );
        let mut damaged = Inputs::from_plan(plan);
        let iteration = damaged
            .environment
            .as_mut()
            .unwrap()
            .iteration_environment
            .as_mut()
            .unwrap();
        iteration.bindings[0].name = plan.enumerator_binding().name.clone();
        assert_eq!(
            rebuild(sources[0], interner, plan.clone(), damaged, inventory),
            Err(GeneratorForInControlError::ForeignLexicalEnvironment)
        );
        let mut body = plan.body().block().clone();
        body.lexical_environment = environment.iteration_environment.clone();
        let states = AsyncGeneratorForInSource::new(sources[0])
            .unwrap()
            .states(plan.entry_state())
            .unwrap();
        let mut changed = plan.clone();
        changed.body = ResumableRegionIr::new(body, states.body(), plan.execution()).unwrap();
        assert_eq!(
            rebuild(
                sources[0],
                interner,
                changed,
                Inputs::from_plan(plan),
                inventory
            ),
            Err(GeneratorForInControlError::InvalidStates)
        );
    });
}

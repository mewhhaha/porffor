use super::*;
use crate::async_generator_source::AsyncGeneratorForInSource;
use crate::*;
use boa_ast::statement::iteration::ForInLoop;
use boa_ast::visitor::{VisitWith, Visitor};
use std::ops::ControlFlow;

pub(super) fn actual_plan(
    source: &str,
    consume: impl FnOnce(
        &[&ForInLoop],
        &boa_interner::Interner,
        &AsyncGeneratorForInIr,
        &[OwnedEnvBindingIr],
    ),
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
    fn find(statements: &[StatementIr]) -> Option<&AsyncGeneratorForInIr> {
        statements.iter().find_map(|statement| match statement {
            StatementIr::AsyncGeneratorForIn(plan) => Some(plan.as_ref()),
            StatementIr::Block(block) => find(&block.statements),
            StatementIr::LexicalBlock(statements) => find(statements),
            StatementIr::EmptyStatementCompletion(item) => {
                find(std::slice::from_ref(item.statement()))
            }
            StatementIr::OrdinaryGeneratorWith(plan) => find(&plan.body().block().statements),
            StatementIr::AsyncGeneratorResourceScope(plan) => find(&plan.body().block().statements),
            _ => None,
        })
    }
    let plan =
        find(&function.body.statements).expect("actual source-produced complete ForIn owner");
    parsed
        .as_script()
        .unwrap()
        .with_compiler_session(|script, interner| {
            struct Find<'ast>(Vec<&'ast ForInLoop>);
            impl<'ast> Visitor<'ast> for Find<'ast> {
                type BreakTy = ();
                fn visit_for_in_loop(&mut self, source: &'ast ForInLoop) -> ControlFlow<()> {
                    self.0.push(source);
                    source.visit_with(self)
                }
            }
            let mut sources = Find(Vec::new());
            let _ = script.visit_with(&mut sources);
            consume(&sources.0, interner, plan, &function.owned_env_bindings);
        });
}

#[derive(Clone)]
pub(super) struct ForInInputs {
    pub(super) head_binding: OwnedEnvBindingIr,
    pub(super) initialization: CheckedAsyncGeneratorForInInitializer,
    pub(super) lexical_environment: Option<ForInOfEnvironmentIr>,
    pub(super) enumerator_binding: OwnedEnvBindingIr,
    pub(super) key_binding: OwnedEnvBindingIr,
    pub(super) value_binding: OwnedEnvBindingIr,
}
impl ForInInputs {
    pub(super) fn from_plan(plan: &AsyncGeneratorForInIr) -> Self {
        Self {
            head_binding: plan.head_binding().clone(),
            initialization: plan.storage.initialization().clone(),
            lexical_environment: plan.lexical_environment().cloned(),
            enumerator_binding: plan.enumerator_binding().clone(),
            key_binding: plan.key_binding().clone(),
            value_binding: plan.value_binding().clone(),
        }
    }
    pub(super) fn exchange_key_and_value(&mut self) {
        std::mem::swap(&mut self.key_binding, &mut self.value_binding);
    }
}
pub(super) fn rebuild(
    source: &ForInLoop,
    interner: &boa_interner::Interner,
    plan: AsyncGeneratorForInIr,
    inventory: &[OwnedEnvBindingIr],
) -> Result<AsyncGeneratorForInIr, GeneratorForInControlError> {
    let inputs = ForInInputs::from_plan(&plan);
    rebuild_with_inputs(source, interner, plan, inputs, inventory)
}
pub(super) fn rebuild_with_inputs(
    source: &ForInLoop,
    interner: &boa_interner::Interner,
    plan: AsyncGeneratorForInIr,
    input: ForInInputs,
    inventory: &[OwnedEnvBindingIr],
) -> Result<AsyncGeneratorForInIr, GeneratorForInControlError> {
    let source = AsyncGeneratorForInSource::for_execution(source, plan.execution()).unwrap();
    AsyncGeneratorForInIr::new(
        source.states(plan.entry_state()).unwrap(),
        source.checked_head(interner).unwrap(),
        plan.head,
        input.head_binding,
        input.initialization,
        input.lexical_environment,
        input.enumerator_binding,
        input.key_binding,
        input.value_binding,
        plan.body,
        inventory,
    )
}
pub(super) fn change_head(
    source: &ForInLoop,
    plan: &AsyncGeneratorForInIr,
    change: impl FnOnce(&mut Vec<StatementIr>),
) -> Result<AsyncGeneratorForInIr, crate::generator_loop_control::GeneratorLoopControlError> {
    let states = AsyncGeneratorForInSource::for_execution(source, plan.execution())
        .unwrap()
        .states(plan.entry_state())
        .unwrap();
    let mut block = plan.head().region().block().clone();
    change(&mut block.statements);
    let mut altered = plan.clone();
    altered.head = ResumableExpressionIr::new(
        ResumableRegionIr::new(block, states.head(), plan.execution())?,
        plan.head().value().clone(),
    );
    Ok(altered)
}

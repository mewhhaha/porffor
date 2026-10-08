//! A complete plain Async With retains its original Object Environment Record.

use crate::lowering_helpers::AsyncWithSourceStates;
use crate::with_object_environment::{CheckedWithObjectEnvironmentIr, WithObjectEnvironmentError};
use crate::{BlockIr, LexicalEnvironmentIr, OwnedEnvBindingIr, StatementIr, TypedExpr};

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum AsyncWithControlError {
    InvalidStates,
    ForeignHead,
    UnallocatedHead,
    ForeignObjectEnvironment,
    UnconsumedSourceAwait,
}

impl From<WithObjectEnvironmentError> for AsyncWithControlError {
    fn from(error: WithObjectEnvironmentError) -> Self {
        match error {
            WithObjectEnvironmentError::ForeignHead => Self::ForeignHead,
            WithObjectEnvironmentError::UnallocatedHead => Self::UnallocatedHead,
            WithObjectEnvironmentError::ForeignObjectEnvironment => Self::ForeignObjectEnvironment,
        }
    }
}

/// The boxed head is published before this child environment exists. Resumed
/// body execution reattaches the same child before injecting a rejection.
#[must_use = "the complete With environment must reach its actual statement"]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AsyncFunctionWithIr {
    entry: u32,
    head_ready: u32,
    body_entry: u32,
    body_end: u32,
    exit: u32,
    head: BlockIr,
    head_value: TypedExpr,
    object_environment: CheckedWithObjectEnvironmentIr,
    body: BlockIr,
    awaits: Vec<(u32, u32)>,
}

impl AsyncFunctionWithIr {
    pub(crate) fn new(
        states: AsyncWithSourceStates,
        head: BlockIr,
        head_value: TypedExpr,
        head_binding: OwnedEnvBindingIr,
        object_binding: OwnedEnvBindingIr,
        lexical_environment: LexicalEnvironmentIr,
        body: BlockIr,
        inventory: &[OwnedEnvBindingIr],
    ) -> Result<Self, AsyncWithControlError> {
        if states.head_ready().checked_add(1) != Some(states.body_entry())
            || states.body_end().checked_add(1) != Some(states.exit())
            || head.lexical_environment.is_some()
            || body.lexical_environment.is_some()
            || crate::async_switch::sequence_exit(&head.statements, states.entry()).ok()
                != Some(states.head_ready())
            || crate::async_switch::sequence_exit(&body.statements, states.body_entry()).ok()
                != Some(states.body_end())
        {
            return Err(AsyncWithControlError::InvalidStates);
        }
        let object_environment = CheckedWithObjectEnvironmentIr::new(
            &head,
            &head_value,
            head_binding,
            object_binding,
            lexical_environment,
            inventory,
        )?;
        let mut awaits = Vec::new();
        collect_awaits(&head.statements, &mut awaits);
        collect_awaits(&body.statements, &mut awaits);
        if awaits != states.awaits() {
            return Err(AsyncWithControlError::UnconsumedSourceAwait);
        }
        Ok(Self {
            entry: states.entry(),
            head_ready: states.head_ready(),
            body_entry: states.body_entry(),
            body_end: states.body_end(),
            exit: states.exit(),
            head,
            head_value,
            object_environment,
            body,
            awaits,
        })
    }

    pub const fn entry_state(&self) -> u32 {
        self.entry
    }
    pub const fn head_ready_state(&self) -> u32 {
        self.head_ready
    }
    pub const fn body_entry_state(&self) -> u32 {
        self.body_entry
    }
    pub const fn body_end_state(&self) -> u32 {
        self.body_end
    }
    pub const fn exit_state(&self) -> u32 {
        self.exit
    }
    pub fn head(&self) -> &BlockIr {
        &self.head
    }
    pub fn head_value(&self) -> &TypedExpr {
        &self.head_value
    }
    pub fn head_binding(&self) -> &OwnedEnvBindingIr {
        self.object_environment.head_binding()
    }
    pub fn object_binding(&self) -> &OwnedEnvBindingIr {
        self.object_environment.object_binding()
    }
    pub fn lexical_environment(&self) -> &LexicalEnvironmentIr {
        self.object_environment.lexical_environment()
    }
    pub fn body(&self) -> &BlockIr {
        &self.body
    }
}

/// The range validator has already rejected foreign continuations and bare
/// operations. Read the actual admitted tree in its source reservation order;
/// separately lowered callable expressions never enter this statement census.
pub(crate) fn collect_awaits(statements: &[StatementIr], output: &mut Vec<(u32, u32)>) {
    let mut points = Vec::new();
    collect_async_suspensions(statements, &mut points);
    output.extend(
        points
            .into_iter()
            .map(|point| (point.suspend_state, point.resume_state)),
    );
}

pub(crate) fn collect_async_suspensions(
    statements: &[StatementIr],
    output: &mut Vec<crate::ResumableSuspensionPointIr>,
) {
    for statement in statements {
        match statement {
            StatementIr::AsyncAwait {
                suspend_state,
                resume_state,
                ..
            } => output.push(crate::ResumableSuspensionPointIr {
                kind: crate::ResumableSuspensionKindIr::Await,
                suspend_state: *suspend_state,
                resume_state: *resume_state,
                resume_environment: crate::ResumableResumeEnvironmentIr::InvocationOuter,
            }),
            StatementIr::AsyncFunctionWith(plan) => {
                collect_async_suspensions(&plan.head().statements, output);
                collect_async_suspensions(&plan.body().statements, output);
            }
            StatementIr::AsyncGeneratorLoop(plan) => {
                output.extend_from_slice(plan.suspensions());
            }
            StatementIr::AsyncGeneratorResourceScope(plan) => {
                output.extend_from_slice(plan.suspensions())
            }
            StatementIr::AsyncGeneratorSwitch(plan) => output.extend_from_slice(plan.suspensions()),
            StatementIr::AsyncGeneratorForOf(plan) => output.extend_from_slice(plan.suspensions()),
            StatementIr::AsyncGeneratorForIn(plan) => output.extend_from_slice(plan.suspensions()),
            StatementIr::AsyncFunctionArrayDestructuring(plan) => {
                collect_async_suspensions(&plan.body().statements, output)
            }
            StatementIr::ResumableClassDefinition(plan) => {
                for prefix in plan.prefixes() {
                    collect_async_suspensions(prefix.statements(), output);
                }
            }
            StatementIr::EmptyStatementCompletion(item) => {
                collect_async_suspensions(std::slice::from_ref(item.statement()), output)
            }
            StatementIr::Block(block)
            | StatementIr::SyncDisposableScope { body: block, .. }
            | StatementIr::AsyncDisposableScope { body: block, .. } => {
                collect_async_suspensions(&block.statements, output)
            }
            StatementIr::LexicalBlock(statements)
            | StatementIr::ParameterInitialization { statements, .. } => {
                collect_async_suspensions(statements, output)
            }
            StatementIr::If {
                then_branch,
                else_branch,
                ..
            }
            | StatementIr::AsyncFunctionIf {
                then_branch,
                else_branch,
                ..
            } => {
                collect_async_suspensions(std::slice::from_ref(then_branch.as_ref()), output);
                if let Some(branch) = else_branch {
                    collect_async_suspensions(std::slice::from_ref(branch.as_ref()), output);
                }
            }
            StatementIr::AsyncFunctionSwitch(plan) => {
                for case in plan.cases() {
                    if let Some(crate::AsyncFunctionSwitchSelectorIr::Resumable(selector)) =
                        case.selector()
                    {
                        collect_async_suspensions(selector.prefix(), output);
                    }
                }
                collect_async_suspensions(plan.lexical_declarations(), output);
                for case in plan.cases() {
                    collect_async_suspensions(&case.body().statements, output);
                }
            }
            StatementIr::AsyncFunctionWhile(plan) => {
                collect_async_suspensions(plan.condition_prefix(), output);
                collect_async_suspensions(std::slice::from_ref(plan.body()), output);
            }
            StatementIr::TryCatch {
                try_block,
                catch_block,
                ..
            } => {
                collect_async_suspensions(&try_block.statements, output);
                collect_async_suspensions(&catch_block.statements, output);
            }
            StatementIr::TryFinally {
                try_block,
                finally_block,
                ..
            } => {
                collect_async_suspensions(&try_block.statements, output);
                collect_async_suspensions(&finally_block.statements, output);
            }
            StatementIr::TryCatchFinally {
                try_block,
                catch_block,
                finally_block,
                ..
            } => {
                collect_async_suspensions(&try_block.statements, output);
                collect_async_suspensions(&catch_block.statements, output);
                collect_async_suspensions(&finally_block.statements, output);
            }
            StatementIr::Labelled { statement, .. } => {
                collect_async_suspensions(std::slice::from_ref(statement.as_ref()), output)
            }
            StatementIr::Switch {
                lexical_declarations,
                cases,
                ..
            } => {
                collect_async_suspensions(lexical_declarations, output);
                for case in cases {
                    collect_async_suspensions(&case.body.statements, output);
                }
            }
            StatementIr::For { init, body, .. } => {
                if let Some(crate::ForInitIr::Statements(statements)) = init {
                    collect_async_suspensions(statements, output);
                }
                collect_async_suspensions(std::slice::from_ref(body.as_ref()), output);
            }
            StatementIr::While { body, .. }
            | StatementIr::DoWhile { body, .. }
            | StatementIr::ForOfIterator { body, .. }
            | StatementIr::ForInArray { body, .. }
            | StatementIr::ForInString { body, .. }
            | StatementIr::ForInObject { body, .. } => {
                collect_async_suspensions(std::slice::from_ref(body.as_ref()), output)
            }
            StatementIr::Empty
            | StatementIr::ModuleImportBinding(_)
            | StatementIr::Lexical { .. }
            | StatementIr::AnnexBFunctionCopy { .. }
            | StatementIr::Var(_)
            | StatementIr::DeclarationEvaluation(_)
            | StatementIr::Expression(_)
            | StatementIr::Debugger
            | StatementIr::Throw(_)
            | StatementIr::Return(_)
            | StatementIr::Break { .. }
            | StatementIr::Continue { .. }
            | StatementIr::ArrayDestructuringOperation(_)
            | StatementIr::GeneratorYield { .. }
            | StatementIr::AsyncModuleInstantiation
            | StatementIr::GeneratorLoop { .. }
            | StatementIr::AsyncGeneratorIf(_)
            | StatementIr::AsyncGeneratorWith(_)
            | StatementIr::AsyncGeneratorArrayDestructuring(_)
            | StatementIr::AsyncGeneratorResourceRegistration(_)
            | StatementIr::OrdinaryGeneratorLoop(_)
            | StatementIr::OrdinaryGeneratorIf(_)
            | StatementIr::OrdinaryGeneratorSwitch(_)
            | StatementIr::OrdinaryGeneratorArrayDestructuring(_)
            | StatementIr::OrdinaryGeneratorWith(_)
            | StatementIr::GeneratorIf { .. }
            | StatementIr::AsyncFunctionForOfIterator { .. }
            | StatementIr::GeneratorForOfIterator { .. }
            | StatementIr::ModuleUnitOnce { .. } => {}
        }
    }
}
